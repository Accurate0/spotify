pub mod auth;
pub mod graphql;
pub mod import;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Router, middleware};
use sqlx::PgPool;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

/// Export files are up to ~13 MB; leave headroom for multipart overhead and larger exports.
const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;

/// The API. With `auth` set, every /graphql request needs a valid Kanidm bearer token; `None`
/// is for local development only. /health is always open for probes.
pub fn router(pool: PgPool, auth: Option<auth::Authenticator>) -> Router {
    let schema = graphql::schema(pool);

    let mut api = Router::new().route("/graphql", get(graphiql).post_service(GraphQL::new(schema)));
    if let Some(auth) = auth {
        api = api.layer(middleware::from_fn_with_state(auth, auth::require_auth));
    }

    api.route("/health", get(|| async { "ok" }))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .layer(TraceLayer::new_for_http())
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
