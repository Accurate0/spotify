pub mod auth;
pub mod enrich;
pub mod graphql;
pub mod import;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Router, middleware};
use enrich::artwork::ArtworkStore;
use sqlx::PgPool;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

/// Export files are up to ~13 MB; leave headroom for multipart overhead and larger exports.
const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;

/// The API. With `auth` set, every /graphql request needs a valid Kanidm bearer token; `None`
/// is for local development only. /health and /art (Spotify artwork, which `<img>` tags fetch
/// without a token) are always open.
pub fn router(
    pool: PgPool,
    auth: Option<auth::Authenticator>,
    artwork: Option<Arc<ArtworkStore>>,
) -> Router {
    let schema = graphql::schema(pool);

    let mut api = Router::new().route("/graphql", get(graphiql).post_service(GraphQL::new(schema)));
    if let Some(auth) = auth {
        api = api.layer(middleware::from_fn_with_state(auth, auth::require_auth));
    }

    api.route("/health", get(|| async { "ok" }))
        .route("/art/{kind}/{id}", get(artwork_image).with_state(artwork))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .layer(TraceLayer::new_for_http())
}

/// Serves stored artwork: `/art/albums/<id>` or `/art/artists/<id>`. Keys never change once
/// written, so responses are cacheable forever.
async fn artwork_image(
    State(store): State<Option<Arc<ArtworkStore>>>,
    Path((kind, id)): Path<(String, String)>,
) -> Response {
    let valid_id =
        !id.is_empty() && id.len() <= 64 && id.bytes().all(|b| b.is_ascii_alphanumeric());
    if !matches!(kind.as_str(), "albums" | "artists") || !valid_id {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(store) = store else {
        return StatusCode::NOT_FOUND.into_response();
    };

    match store.get(&format!("{kind}/{id}.jpg")).await {
        Ok(Some(bytes)) => (
            [
                (header::CONTENT_TYPE, "image/jpeg"),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            bytes,
        )
            .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::warn!(%kind, %id, error = %e, "artwork read failed");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
