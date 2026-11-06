use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use spotify_server::auth::{AuthConfig, Authenticator, Provider};
use spotify_server::enrich::artwork::ArtworkStore;
use spotify_server::enrich::spotify::{SpotifyClient, SpotifyConfig};
use spotify_server::enrich::{self, EnrichConfig};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

#[derive(Parser)]
#[command(about = "Spotify export API")]
struct Cli {
    /// Print the GraphQL SDL and exit.
    #[arg(long)]
    print_schema: bool,

    #[arg(long, env = "DATABASE_URL", global = true)]
    database_url: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    serve: ServeArgs,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the API (the default).
    Serve,
    /// Fetch Spotify Web API metadata and artwork for imported items, within the budget.
    Enrich(EnrichArgs),
}

#[derive(Args)]
struct ServeArgs {
    #[arg(long, env = "LISTEN_ADDR", default_value = "0.0.0.0:8000")]
    listen: SocketAddr,

    /// Serve without authentication. Only for local development.
    #[arg(long, env = "AUTH_DISABLED", default_value_t = false)]
    auth_disabled: bool,

    #[arg(
        long,
        env = "OIDC_ISSUER",
        default_value = "https://idm.anurag.sh/oauth2/openid/spotify"
    )]
    oidc_issuer: String,

    #[arg(long, env = "OIDC_AUDIENCE", default_value = "spotify")]
    oidc_audience: String,

    /// Trusted issuers as `<issuer>=<audience>` (comma separated), e.g. this app's own client
    /// and the `graph` federation router's. Replaces OIDC_ISSUER/OIDC_AUDIENCE when set.
    #[arg(long, env = "OIDC_PROVIDERS", value_delimiter = ',')]
    oidc_providers: Vec<Provider>,

    /// Groups allowed to use the API (comma separated); empty allows any signed-in user.
    #[arg(
        long,
        env = "OIDC_GROUPS",
        value_delimiter = ',',
        default_value = "spotify_users"
    )]
    oidc_groups: Vec<String>,

    /// The origin /art is reachable at, for absolute artwork URLs.
    #[arg(
        long,
        env = "PUBLIC_URL",
        default_value = "https://spotify.ortzi.cloud"
    )]
    public_url: String,
}

#[derive(Args)]
struct EnrichArgs {
    #[arg(long, env = "SPOTIFY_CLIENT_ID")]
    client_id: Option<String>,

    #[arg(long, env = "SPOTIFY_CLIENT_SECRET", hide_env_values = true)]
    client_secret: Option<String>,

    #[arg(
        long,
        env = "SPOTIFY_API_URL",
        default_value = "https://api.spotify.com"
    )]
    api_url: String,

    #[arg(
        long,
        env = "SPOTIFY_ACCOUNTS_URL",
        default_value = "https://accounts.spotify.com"
    )]
    accounts_url: String,

    /// Milliseconds between Web API requests.
    #[arg(long, default_value_t = 4000)]
    request_interval_ms: u64,

    /// Web API requests per run.
    #[arg(long, default_value_t = 180)]
    max_requests_per_run: i64,

    /// Web API requests per rolling 24 hours, across runs.
    #[arg(long, default_value_t = 5000)]
    daily_requests: i64,

    /// Artwork downloads per run (from Spotify's image CDN, outside the API budget).
    #[arg(long, default_value_t = 300)]
    images_per_run: i64,

    /// Milliseconds between artwork downloads.
    #[arg(long, default_value_t = 500)]
    image_interval_ms: u64,

    /// Show the budget and the next items to fetch without calling Spotify.
    #[arg(long)]
    dry_run: bool,
}

async fn connect(database_url: Option<String>) -> anyhow::Result<PgPool> {
    let database_url = database_url.context("DATABASE_URL is required")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("connecting to postgres")?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.print_schema {
        print!("{}", spotify_server::graphql::sdl());
        return Ok(());
    }

    spotify_server::telemetry::setup::init();

    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve(cli.database_url, cli.serve).await,
        Command::Enrich(args) => enrich(cli.database_url, args).await,
    }
}

async fn serve(database_url: Option<String>, args: ServeArgs) -> anyhow::Result<()> {
    let auth = if args.auth_disabled {
        tracing::warn!("authentication disabled");
        None
    } else {
        let providers = if args.oidc_providers.is_empty() {
            vec![Provider {
                issuer: args.oidc_issuer,
                audience: args.oidc_audience,
            }]
        } else {
            args.oidc_providers
        };
        let auth = Authenticator::new(AuthConfig {
            providers,
            groups: args
                .oidc_groups
                .into_iter()
                .filter(|g| !g.is_empty())
                .collect(),
        });
        // Kanidm being briefly unreachable shouldn't stop the API starting; unknown keys are
        // fetched on demand.
        if let Err(e) = auth.refresh().await {
            tracing::warn!("initial jwks fetch failed, retrying on demand: {e}");
        }
        tokio::spawn(auth.clone().run());
        Some(auth)
    };

    let artwork = ArtworkStore::from_env()?.map(Arc::new);
    if artwork.is_none() {
        tracing::warn!("no artwork bucket configured, /art will return 404");
    }

    let pool = connect(database_url).await?;
    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    tracing::info!(addr = %args.listen, "listening");

    axum::serve(
        listener,
        spotify_server::router(pool, auth, artwork, &args.public_url),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;

    Ok(())
}

async fn enrich(database_url: Option<String>, args: EnrichArgs) -> anyhow::Result<()> {
    let pool = connect(database_url).await?;
    let config = EnrichConfig {
        request_interval: Duration::from_millis(args.request_interval_ms),
        max_requests_per_run: args.max_requests_per_run,
        daily_requests: args.daily_requests,
        images_per_run: args.images_per_run,
        image_interval: Duration::from_millis(args.image_interval_ms),
    };

    if args.dry_run {
        let budget = enrich::budget(&pool).await?;
        println!(
            "{} of {} requests used in the last 24h; this run may make {}",
            budget.used_last_24h,
            config.daily_requests,
            enrich::run_budget(&config, budget.used_last_24h)
        );
        if let Some(until) = budget.rate_limited_until {
            println!("rate limited until {until}");
        }
        for entry in enrich::next_in_queue(&pool, 10).await? {
            println!("{:<6} {:<8} {}", entry.target, entry.kind, entry.id);
        }
        return Ok(());
    }

    let client = SpotifyClient::new(SpotifyConfig {
        client_id: args.client_id.context("SPOTIFY_CLIENT_ID is required")?,
        client_secret: args
            .client_secret
            .context("SPOTIFY_CLIENT_SECRET is required")?,
        api_url: args.api_url,
        accounts_url: args.accounts_url,
    });
    let store = ArtworkStore::from_env()?;
    if store.is_none() {
        tracing::warn!("no artwork bucket configured, skipping artwork downloads");
    }

    let report = enrich::run(&pool, &client, store.as_ref(), &config).await?;
    tracing::info!(
        api_requests = report.api_requests,
        items = report.items_done,
        artists = report.artists_done,
        not_found = report.not_found,
        failed = report.failed,
        artwork = report.images,
        rate_limited_until = ?report.rate_limited_until,
        skipped = ?report.skipped,
        "enrichment run finished"
    );
    Ok(())
}
