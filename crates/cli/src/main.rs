mod client;
mod graphql;
mod import;
mod stats;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
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
    let cli = Cli::parse();
    let client = Client::new(&cli.url, cli.token)?;

    match cli.command {
        Command::Accounts {
            command: AccountsCommand::List,
        } => {
            let accounts = client.run(AccountsQuery::build(())).await?.accounts;
            if accounts.is_empty() {
                println!("no accounts, add one with `spotify accounts add <slug>`");
            }
            for account in accounts {
                println!(
                    "{:<12} {:<24} {:>8} plays  {}",
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
                .create_account
                .account;
            println!("created account {}", account.slug);
        }
        Command::Import {
            path,
            account,
            dry_run,
        } => import::run(&client, &account, &path, dry_run).await?,
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
        Command::Artist {
            name,
            filter,
            limit,
        } => stats::artist(&client, &name, &filter, limit).await?,
    }

    Ok(())
}
