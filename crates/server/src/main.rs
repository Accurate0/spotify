use std::net::SocketAddr;

use anyhow::Context;
use clap::Parser;
use spotify_server::auth::{AuthConfig, Authenticator};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(about = "Spotify export API")]
struct Args {
    /// Print the GraphQL SDL and exit.
    #[arg(long)]
    print_schema: bool,

    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

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

    /// Groups allowed to use the API (comma separated); empty allows any signed-in user.
    #[arg(
        long,
        env = "OIDC_GROUPS",
        value_delimiter = ',',
        default_value = "spotify_users"
    )]
    oidc_groups: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.print_schema {
        print!("{}", spotify_server::graphql::sdl());
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let auth = if args.auth_disabled {
        tracing::warn!("authentication disabled");
        None
    } else {
        let auth = Authenticator::new(AuthConfig {
            issuer: args.oidc_issuer,
            audience: args.oidc_audience,
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

    let database_url = args.database_url.context("DATABASE_URL is required")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("connecting to postgres")?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    tracing::info!(addr = %args.listen, "listening");

    axum::serve(listener, spotify_server::router(pool, auth))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;

    Ok(())
}
