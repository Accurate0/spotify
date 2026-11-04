pub mod graphql;
pub mod import;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::Router;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use sqlx::PgPool;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

/// Export files are up to ~13 MB; leave headroom for multipart overhead and larger exports.
const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;

pub fn router(pool: PgPool) -> Router {
    let schema = graphql::builder().data(pool).finish();

    Router::new()
        .route("/graphql", get(graphiql).post_service(GraphQL::new(schema)))
        .route("/health", get(|| async { "ok" }))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .layer(TraceLayer::new_for_http())
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
