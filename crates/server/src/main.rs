use std::net::SocketAddr;

use anyhow::Context;
use clap::Parser;
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

    // Kanidm JWT validation lands with the deployment work; until then refuse to serve an
    // unauthenticated API unless explicitly asked to.
    anyhow::ensure!(
        args.auth_disabled,
        "authentication is not implemented yet, set AUTH_DISABLED=true for local development"
    );
    tracing::warn!("authentication disabled");

    let database_url = args.database_url.context("DATABASE_URL is required")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("connecting to postgres")?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    tracing::info!(addr = %args.listen, "listening");

    axum::serve(listener, spotify_server::router(pool))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;

    Ok(())
}
