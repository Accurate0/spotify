//! The router rejects unauthenticated /graphql requests when auth is enabled. No network:
//! requests without a usable token fail before any key lookup.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use spotify_server::auth::{AuthConfig, Authenticator, Provider};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

fn app() -> axum::Router {
    // Never connected: these requests are rejected before reaching GraphQL.
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused@localhost/unused")
        .unwrap();
    let auth = Authenticator::new(AuthConfig {
        providers: vec![Provider {
            issuer: "https://idm.example.test/oauth2/openid/spotify".to_owned(),
            audience: "spotify".to_owned(),
        }],
        groups: vec!["spotify_users".to_owned()],
    });
    spotify_server::router(pool, Some(auth), None)
}

async fn status(request: Request<Body>) -> StatusCode {
    app().oneshot(request).await.unwrap().status()
}

#[tokio::test]
async fn graphql_requires_a_token() {
    let request = Request::post("/graphql")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"query":"{ accounts { slug } }"}"#))
        .unwrap();
    assert_eq!(status(request).await, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn malformed_tokens_are_rejected() {
    let request = Request::post("/graphql")
        .header("authorization", "Bearer not-a-jwt")
        .body(Body::empty())
        .unwrap();
    assert_eq!(status(request).await, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn health_is_open() {
    let request = Request::get("/health").body(Body::empty()).unwrap();
    assert_eq!(status(request).await, StatusCode::OK);
}
