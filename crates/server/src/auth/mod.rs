pub mod kanidm;
pub mod passport;

use std::collections::BTreeSet;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

pub use kanidm::{KanidmConfig, Provider};
pub use passport::{AuthConfig, PASSPORT_HEADER};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("jwks error: {0}")]
    Jwks(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

/// A GraphQL-shaped JSON body, so a federation router (which rejects non-JSON subgraph
/// responses) can pass the reason on to the caller.
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHENTICATED"),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            Self::Jwks(_) | Self::Http(_) => (StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE"),
        };
        let body = serde_json::json!({
            "errors": [{ "message": self.to_string(), "extensions": { "code": code } }]
        });
        (status, axum::Json(body)).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct Claims {
    pub sub: String,
    pub preferred_username: Option<String>,
    pub groups: Vec<String>,
    pub scopes: BTreeSet<String>,
}

impl Claims {
    fn identity(&self) -> &str {
        self.preferred_username.as_deref().unwrap_or(&self.sub)
    }

    pub fn scopes(&self) -> BTreeSet<&str> {
        self.scopes.iter().map(String::as_str).collect()
    }
}

#[derive(Clone)]
pub struct Authenticator {
    passport: passport::Verifier,
    kanidm: kanidm::Verifier,
}

impl Authenticator {
    pub fn new(passport: AuthConfig, kanidm: KanidmConfig) -> Self {
        Self {
            passport: passport::Verifier::new(passport),
            kanidm: kanidm::Verifier::new(kanidm),
        }
    }

    pub async fn refresh(&self) {
        if let Err(e) = self.passport.refresh().await {
            tracing::warn!("initial passport jwks fetch failed, retrying on demand: {e}");
        }
        if let Err(e) = self.kanidm.refresh().await {
            tracing::warn!("initial kanidm jwks fetch failed, retrying on demand: {e}");
        }
    }

    pub async fn run(self) {
        tokio::join!(self.passport.run(), self.kanidm.run());
    }

    pub async fn authorize(&self, headers: &HeaderMap) -> Result<Claims, AuthError> {
        if let Some(passport) = headers.get(PASSPORT_HEADER) {
            let passport = passport
                .to_str()
                .map_err(|_| AuthError::Unauthorized("unreadable graph passport".to_owned()))?;
            return self.passport.authorize(passport).await;
        }

        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AuthError::Unauthorized("missing bearer token".to_owned()))?;
        self.kanidm.authorize(token).await
    }
}

/// Middleware: requires the router's passport or a Kanidm token from an allowed user.
pub async fn require_auth(
    State(auth): State<Authenticator>,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let claims = auth.authorize(request.headers()).await?;
    tracing::debug!(user = claims.identity(), "authorized");
    let mut request = request;
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn errors_are_graphql_json() {
        let response = AuthError::Unauthorized("missing bearer token".to_owned()).into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            body["errors"][0]["message"],
            "unauthorized: missing bearer token"
        );
        assert_eq!(body["errors"][0]["extensions"]["code"], "UNAUTHENTICATED");
    }
}
