//! Kanidm OIDC bearer auth for /graphql. Tokens come from the SPA (oidc-client-ts) or the CLI
//! (`ortzi spotify` passes its login token as SPOTIFY_TOKEN); both use the `spotify` public
//! client, so they share an issuer and audience. Adapted from ai-gateway's authenticator.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use tokio::sync::RwLock;

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

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Jwks(_) | Self::Http(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        (status, self.to_string()).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// e.g. `https://idm.anurag.sh/oauth2/openid/spotify`.
    pub issuer: String,
    pub audience: String,
    /// The caller must hold one of these groups (short name or SPN). Empty allows anyone the
    /// issuer will sign for.
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    #[serde(default)]
    pub preferred_username: Option<String>,
    #[serde(default)]
    pub groups: Option<Vec<String>>,
}

impl Claims {
    fn identity(&self) -> &str {
        self.preferred_username.as_deref().unwrap_or(&self.sub)
    }
}

#[derive(Deserialize)]
struct UserInfo {
    #[serde(default)]
    groups: Vec<String>,
}

/// Kanidm reports groups as SPNs (`name@domain`) or UUIDs; match on either full SPN or name.
fn groups_match(held: &[String], allowed: &[String]) -> bool {
    held.iter().any(|group| {
        let name = group
            .split_once('@')
            .map_or(group.as_str(), |(name, _)| name);
        allowed.iter().any(|a| a == group || a == name)
    })
}

#[derive(Clone)]
struct VerifyingKey {
    key: DecodingKey,
    alg: Option<Algorithm>,
}

#[derive(Clone)]
pub struct Authenticator {
    inner: Arc<Inner>,
}

struct Inner {
    client: reqwest::Client,
    config: AuthConfig,
    jwks_uri: String,
    userinfo_endpoint: String,
    keys: RwLock<BTreeMap<String, VerifyingKey>>,
}

impl Authenticator {
    const REFRESH_INTERVAL: Duration = Duration::from_secs(3600);

    pub fn new(config: AuthConfig) -> Self {
        let base = config.issuer.trim_end_matches('/').to_owned();
        Self {
            inner: Arc::new(Inner {
                client: reqwest::Client::builder()
                    .timeout(Duration::from_secs(20))
                    .user_agent("spotify-server")
                    .build()
                    .unwrap_or_default(),
                jwks_uri: format!("{base}/public_key.jwk"),
                userinfo_endpoint: format!("{base}/userinfo"),
                config,
                keys: RwLock::new(BTreeMap::new()),
            }),
        }
    }

    pub async fn authorize(&self, token: &str) -> Result<Claims, AuthError> {
        let claims = self.verify(token).await?;
        let allowed = &self.inner.config.groups;
        if allowed.is_empty() {
            return Ok(claims);
        }

        let held = match &claims.groups {
            Some(groups) => groups.clone(),
            None => self.userinfo_groups(token).await?,
        };
        if !groups_match(&held, allowed) {
            return Err(AuthError::Forbidden(format!(
                "{} is not in any of {allowed:?}",
                claims.identity()
            )));
        }
        Ok(claims)
    }

    async fn userinfo_groups(&self, token: &str) -> Result<Vec<String>, AuthError> {
        let info: UserInfo = self
            .inner
            .client
            .get(&self.inner.userinfo_endpoint)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(info.groups)
    }

    pub async fn refresh(&self) -> Result<usize, AuthError> {
        let set: JwkSet = self
            .inner
            .client
            .get(&self.inner.jwks_uri)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut keys = BTreeMap::new();
        for jwk in &set.keys {
            let Some(kid) = jwk.common.key_id.clone() else {
                continue;
            };
            let asymmetric = matches!(
                jwk.algorithm,
                AlgorithmParameters::RSA(_)
                    | AlgorithmParameters::EllipticCurve(_)
                    | AlgorithmParameters::OctetKeyPair(_)
            );
            if !asymmetric {
                tracing::warn!("ignoring non-asymmetric jwk {kid}");
                continue;
            }
            match DecodingKey::from_jwk(jwk) {
                Ok(key) => {
                    let alg = jwk
                        .common
                        .key_algorithm
                        .and_then(|a| a.to_string().parse::<Algorithm>().ok());
                    keys.insert(kid, VerifyingKey { key, alg });
                }
                Err(e) => tracing::warn!("ignoring unusable jwk {kid}: {e}"),
            }
        }

        if keys.is_empty() {
            return Err(AuthError::Jwks(format!(
                "no usable keys at {}",
                self.inner.jwks_uri
            )));
        }

        let count = keys.len();
        *self.inner.keys.write().await = keys;
        tracing::info!(keys = count, "refreshed jwks");
        Ok(count)
    }

    /// Refreshes keys hourly; unknown key ids also trigger a refresh on demand.
    pub async fn run(self) {
        let mut ticker = tokio::time::interval(Self::REFRESH_INTERVAL);
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if let Err(e) = self.refresh().await {
                tracing::warn!("jwks refresh failed, keeping previous keys: {e}");
            }
        }
    }

    async fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let header = decode_header(token)
            .map_err(|e| AuthError::Unauthorized(format!("unreadable token header: {e}")))?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::Unauthorized("token has no kid".to_owned()))?;

        let mut key = self.inner.keys.read().await.get(&kid).cloned();
        if key.is_none() {
            tracing::info!("unknown kid {kid}, refreshing jwks");
            self.refresh().await?;
            key = self.inner.keys.read().await.get(&kid).cloned();
        }
        let key = key.ok_or_else(|| AuthError::Unauthorized(format!("no key for kid {kid}")))?;

        let mut validation = Validation::new(key.alg.unwrap_or(header.alg));
        validation.set_issuer(&[&self.inner.config.issuer]);
        validation.set_audience(&[&self.inner.config.audience]);

        decode::<Claims>(token, &key.key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AuthError::Unauthorized(format!("token rejected: {e}")))
    }
}

/// Middleware: requires `Authorization: Bearer <token>` from an allowed user.
pub async fn require_auth(
    State(auth): State<Authenticator>,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AuthError::Unauthorized("missing bearer token".to_owned()))?;

    let claims = auth.authorize(token).await?;
    tracing::debug!(user = claims.identity(), "authorized");
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_match_by_short_name_or_spn() {
        let held = vec![
            "00000000-0000-0000-0000-000000000002".to_owned(),
            "spotify_users@idm.example.test".to_owned(),
        ];
        assert!(groups_match(&held, &["spotify_users".to_owned()]));
        assert!(groups_match(
            &held,
            &["spotify_users@idm.example.test".to_owned()]
        ));
        assert!(!groups_match(&held, &["platform_admins".to_owned()]));
        assert!(!groups_match(&[], &["spotify_users".to_owned()]));
    }

    #[test]
    fn endpoints_derive_from_issuer() {
        let auth = Authenticator::new(AuthConfig {
            issuer: "https://idm.example.test/oauth2/openid/spotify/".to_owned(),
            audience: "spotify".to_owned(),
            groups: vec![],
        });
        assert_eq!(
            auth.inner.jwks_uri,
            "https://idm.example.test/oauth2/openid/spotify/public_key.jwk"
        );
        assert_eq!(
            auth.inner.userinfo_endpoint,
            "https://idm.example.test/oauth2/openid/spotify/userinfo"
        );
    }
}
