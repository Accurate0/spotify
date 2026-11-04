mod client;
mod graphql;
mod import;

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
    }

    Ok(())
}
