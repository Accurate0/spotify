mod client;
mod graphql;
mod import;
mod insights;
mod link;
mod stats;

use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use cynic::{MutationBuilder, QueryBuilder};

use client::Client;
use graphql::{AccountsQuery, CreateAccount, CreateAccountInput, CreateAccountVariables};

#[derive(Parser)]
#[command(name = "spotify", about = "Import and query Spotify data exports")]
struct Cli {
    /// API base URL.
    #[arg(
        long,
        env = "SPOTIFY_URL",
        default_value = "https://spotify.ortzi.cloud",
        global = true
    )]
    url: String,

    /// Bearer token, provided by `ortzi` when run as `ortzi spotify`.
    #[arg(long, env = "SPOTIFY_TOKEN", hide_env_values = true, global = true)]
    token: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage the accounts exports are imported into.
    Accounts {
        #[command(subcommand)]
        command: AccountsCommand,
    },
    /// Import an export directory or zip. Already-imported files are skipped.
    Import {
        path: PathBuf,
        /// Account slug to import into.
        #[arg(long, short)]
        account: String,
        /// Only show what would be uploaded.
        #[arg(long)]
        dry_run: bool,
    },
    /// Let the server fetch an account's recently played tracks from Spotify, until the next
    /// export covers them. Opens Spotify's sign-in in the browser.
    Link {
        /// Account slug.
        account: String,
    },
    /// Totals: listening time, plays, tracks, artists, skip rate.
    Summary {
        #[command(flatten)]
        filter: stats::FilterArgs,
    },
    /// Most played artists, tracks or albums.
    Top {
        #[arg(value_enum)]
        kind: stats::TopKind,
        #[command(flatten)]
        filter: stats::FilterArgs,
        #[arg(long, short = 'n', default_value_t = 20)]
        limit: i32,
        /// Rank by listening time or by number of plays.
        #[arg(long, value_enum, default_value_t)]
        by: stats::RankBy,
    },
    /// Listening time per day, week, month or year.
    Timeline {
        #[command(flatten)]
        filter: stats::FilterArgs,
        #[arg(long, value_enum, default_value = "month")]
        bucket: stats::TimeBucket,
    },
    /// Listening by weekday and hour.
    Heatmap {
        #[command(flatten)]
        filter: stats::FilterArgs,
    },
    /// Search artists, tracks and episodes by name.
    Search {
        query: String,
        #[arg(long, short = 'n', default_value_t = 10)]
        limit: i32,
    },
    /// Stats and recent plays for a track or episode, by URI or best search match.
    Track {
        query: String,
        #[command(flatten)]
        filter: stats::FilterArgs,
    },
    /// Streaks, sessions, biggest days, tracks on repeat, discoveries and forgotten favourites.
    Insights {
        #[command(flatten)]
        filter: stats::FilterArgs,
    },
    /// Listening split by device, country, shuffle, offline, or how plays started or ended.
    Breakdown {
        #[arg(value_enum)]
        by: insights::BreakdownDimension,
        #[command(flatten)]
        filter: stats::FilterArgs,
    },
    /// Tracks (or artists) you skip most, counting every play including short skips.
    Skipped {
        #[command(flatten)]
        filter: stats::FilterArgs,
        #[arg(long, short = 'n', default_value_t = 20)]
        limit: i32,
        /// Only include entries with at least this many plays [default: 10 tracks, 50 artists].
        #[arg(long)]
        min_plays: Option<i32>,
        /// Rank artists instead of tracks.
        #[arg(long)]
        artists: bool,
        /// With --artists, least skipped first.
        #[arg(long, requires = "artists")]
        least: bool,
    },
    /// Progress of the Spotify Web API backfill (metadata, genres, artwork).
    Enrichment {
        /// Time zone for timestamps.
        #[arg(long, env = "SPOTIFY_TZ", default_value = "Australia/Perth")]
        tz: chrono_tz::Tz,
    },
    /// Stats and top tracks for an artist.
    Artist {
        name: String,
        #[command(flatten)]
        filter: stats::FilterArgs,
        #[arg(long, short = 'n', default_value_t = 10)]
        limit: i32,
    },
}

#[derive(Subcommand)]
enum AccountsCommand {
    List,
    Add {
        slug: String,
        #[arg(long)]
        name: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Shell completions, driven by ortzi (`completions: clap` in ortzi.yaml).
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();
    let client = Client::new(&cli.url, cli.token)?;

    match cli.command {
        Command::Accounts {
            command: AccountsCommand::List,
        } => {
            let accounts = client.run(AccountsQuery::build(())).await?.spotify_accounts;
            if accounts.is_empty() {
                println!("no accounts, add one with `spotify accounts add <slug>`");
            }
            for account in accounts {
                let link = match account.spotify_link {
                    Some(link) if link.revoked_at.is_some() => {
                        format!("  link revoked, run `spotify link {}`", account.slug)
                    }
                    Some(link) => format!("  linked to {}", link.spotify_user_id),
                    None => String::new(),
                };
                println!(
                    "{:<12} {:<24} {:>8} plays  {}{link}",
                    account.slug,
                    account.display_name,
                    account.play_count,
                    account.spotify_username.unwrap_or_default()
                );
            }
        }
        Command::Accounts {
            command: AccountsCommand::Add { slug, name },
        } => {
            let account = client
                .run(CreateAccount::build(CreateAccountVariables {
                    input: CreateAccountInput {
                        slug,
                        display_name: name,
                    },
                }))
                .await?
                .spotify_create_account
                .account;
            println!("created account {}", account.slug);
        }
        Command::Import {
            path,
            account,
            dry_run,
        } => import::run(&client, &account, &path, dry_run).await?,
        Command::Link { account } => link::run(&client, &account).await?,
        Command::Summary { filter } => stats::summary(&client, &filter).await?,
        Command::Top {
            kind,
            filter,
            limit,
            by,
        } => stats::top(&client, kind, &filter, limit, by).await?,
        Command::Timeline { filter, bucket } => stats::timeline(&client, &filter, bucket).await?,
        Command::Heatmap { filter } => stats::heatmap(&client, &filter).await?,
        Command::Search { query, limit } => stats::search(&client, &query, limit).await?,
        Command::Track { query, filter } => stats::track(&client, &query, &filter).await?,
        Command::Insights { filter } => insights::insights(&client, &filter).await?,
        Command::Breakdown { by, filter } => insights::breakdown(&client, &filter, by).await?,
        Command::Skipped {
            filter,
            limit,
            min_plays,
            artists: true,
            least,
        } => {
            insights::skipped_artists(&client, &filter, limit, min_plays.unwrap_or(50), least)
                .await?
        }
        Command::Skipped {
            filter,
            limit,
            min_plays,
            artists: false,
            ..
        } => insights::skipped(&client, &filter, limit, min_plays.unwrap_or(10)).await?,
        Command::Artist {
            name,
            filter,
            limit,
        } => stats::artist(&client, &name, &filter, limit).await?,
        Command::Enrichment { tz } => insights::enrichment(&client, tz).await?,
    }

    Ok(())
}
