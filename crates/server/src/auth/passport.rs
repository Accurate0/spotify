use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};

use super::{AuthError, Claims};

pub const PASSPORT_HEADER: &str = "x-graph-passport";
const PASSPORT_ISSUER: &str = "graph-router";

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub audience: String,
    pub jwks: String,
}

impl std::str::FromStr for AuthConfig {
    type Err = String;

    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let config: Self =
            serde_json::from_str(json).map_err(|e| format!("SUBGRAPH_AUTH is not valid: {e}"))?;
        if config.audience.is_empty() || config.jwks.is_empty() {
            return Err("SUBGRAPH_AUTH needs an audience and the router's jwks".to_owned());
        }
        Ok(config)
    }
}

#[derive(Deserialize)]
struct Passport {
    identity: Identity,
}

#[derive(Deserialize)]
struct Identity {
    kind: String,
    subject: String,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    scopes: BTreeSet<String>,
    #[serde(default)]
    groups: Vec<String>,
}

#[derive(Clone)]
struct VerifyingKey {
    key: DecodingKey,
    alg: Option<Algorithm>,
}

#[derive(Clone)]
pub struct Verifier {
    inner: Arc<Inner>,
}

struct Inner {
    client: reqwest::Client,
    config: AuthConfig,
    keys: RwLock<BTreeMap<String, VerifyingKey>>,
    last_refresh: Mutex<Option<Instant>>,
}

impl Verifier {
    const REFRESH_INTERVAL: Duration = Duration::from_secs(3600);
    const UNKNOWN_KID_COOLDOWN: Duration = Duration::from_secs(30);

    pub fn new(config: AuthConfig) -> Self {
        Self {
            inner: Arc::new(Inner {
                client: reqwest::Client::builder()
                    .timeout(Duration::from_secs(20))
                    .user_agent("spotify-server")
                    .build()
                    .unwrap_or_default(),
                config,
                keys: RwLock::new(BTreeMap::new()),
                last_refresh: Mutex::new(None),
            }),
        }
    }

    pub async fn authorize(&self, passport: &str) -> Result<Claims, AuthError> {
        let Passport { identity } = self.verify(passport).await?;
        if identity.kind != "user" {
            return Err(AuthError::Forbidden(format!(
                "{} identities can't use spotify",
                identity.kind
            )));
        }

        Ok(Claims {
            sub: identity.subject,
            preferred_username: identity.username,
            groups: identity.groups,
            scopes: identity.scopes,
        })
    }

    pub async fn refresh(&self) -> Result<usize, AuthError> {
        let mut last = self.inner.last_refresh.lock().await;
        *last = Some(Instant::now());
        self.fetch_keys().await
    }

    // an in-flight refresh may bring the kid; otherwise refresh at most once per cooldown
    async fn refresh_for(&self, kid: &str) -> Result<(), AuthError> {
        let mut last = self.inner.last_refresh.lock().await;
        let known = self.inner.keys.read().await.contains_key(kid);
        if known || last.is_some_and(|at| at.elapsed() < Self::UNKNOWN_KID_COOLDOWN) {
            return Ok(());
        }

        tracing::info!("unknown passport kid {kid}, refreshing jwks");
        *last = Some(Instant::now());
        self.fetch_keys().await.map(|_| ())
    }

    async fn fetch_keys(&self) -> Result<usize, AuthError> {
        let jwks = &self.inner.config.jwks;

        let set: JwkSet = self
            .inner
            .client
            .get(jwks)
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
            return Err(AuthError::Jwks(format!("no usable keys at {jwks}")));
        }

        let count = keys.len();
        *self.inner.keys.write().await = keys;
        tracing::info!(jwks, keys = count, "refreshed passport jwks");
        Ok(count)
    }

    pub async fn run(self) {
        let mut ticker = tokio::time::interval(Self::REFRESH_INTERVAL);
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if let Err(e) = self.refresh().await {
                tracing::warn!("passport jwks refresh failed, keeping previous keys: {e}");
            }
        }
    }

    async fn verify(&self, token: &str) -> Result<Passport, AuthError> {
        let header = decode_header(token)
            .map_err(|e| AuthError::Unauthorized(format!("unreadable passport header: {e}")))?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::Unauthorized("passport has no kid".to_owned()))?;

        let mut key = self.inner.keys.read().await.get(&kid).cloned();
        if key.is_none() {
            self.refresh_for(&kid).await?;
            key = self.inner.keys.read().await.get(&kid).cloned();
        }
        let key = key.ok_or_else(|| AuthError::Unauthorized(format!("no key for kid {kid}")))?;

        let mut validation = Validation::new(key.alg.unwrap_or(header.alg));
        validation.set_issuer(&[PASSPORT_ISSUER]);
        validation.set_audience(&[&self.inner.config.audience]);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        validation.leeway = 5;

        decode::<Passport>(token, &key.key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AuthError::Unauthorized(format!("passport rejected: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};

    // a throwaway P-256 key, only for signing test passports
    const TEST_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgQrxIw5f6gChTjB4J
goqO/XL/jTOI1DURt4md/xUx3qChRANCAARRtn8X4BTpCgfXkHeNNosv447OcYmC
WTbO2iQACXuyX0kacNHv2hA0Kb33waLg/85mkeG1cQVfOEMQq8jA8xNE
-----END PRIVATE KEY-----";
    const TEST_X: &str = "UbZ_F-AU6QoH15B3jTaLL-OOznGJglk2ztokAAl7sl8";
    const TEST_Y: &str = "SRpw0e_aEDQpvffBouD_zmaR4bVxBV84QxCryMDzE0Q";

    async fn authenticator() -> Verifier {
        let auth = Verifier::new(AuthConfig {
            audience: "spotify".to_owned(),
            jwks: "http://router.invalid/.well-known/passport.jwks".to_owned(),
        });
        let set: JwkSet = serde_json::from_value(serde_json::json!({ "keys": [{
            "kty": "EC", "crv": "P-256", "x": TEST_X, "y": TEST_Y, "kid": "router", "alg": "ES256",
        }] }))
        .unwrap();
        auth.inner.keys.write().await.insert(
            "router".to_owned(),
            VerifyingKey {
                key: DecodingKey::from_jwk(&set.keys[0]).unwrap(),
                alg: Some(Algorithm::ES256),
            },
        );
        *auth.inner.last_refresh.lock().await = Some(Instant::now());
        auth
    }

    fn passport(overrides: serde_json::Value) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut claims = serde_json::json!({
            "iss": PASSPORT_ISSUER,
            "aud": "spotify",
            "sub": "uuid",
            "iat": now,
            "exp": now + 60,
            "identity": {
                "issuer": "kanidm",
                "kind": "user",
                "subject": "uuid",
                "username": "anurag",
                "scopes": ["spotify:history:read"],
                "groups": ["spotify_users@idm.example.test"],
            },
        });
        for (key, value) in overrides.as_object().unwrap() {
            claims[key] = value.clone();
        }

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some("router".to_owned());
        let key = EncodingKey::from_ec_pem(TEST_KEY.as_bytes()).unwrap();
        encode(&header, &claims, &key).unwrap()
    }

    #[tokio::test]
    async fn passports_carry_the_caller_and_their_scopes() {
        let claims = authenticator()
            .await
            .authorize(&passport(serde_json::json!({})))
            .await
            .unwrap();
        assert_eq!(claims.preferred_username.as_deref(), Some("anurag"));
        assert!(claims.scopes().contains("spotify:history:read"));
    }

    #[tokio::test]
    async fn passports_for_others_or_from_elsewhere_are_rejected() {
        let auth = authenticator().await;
        for overrides in [
            serde_json::json!({ "aud": "registry" }),
            serde_json::json!({ "iss": "https://idm.anurag.sh/oauth2/openid/spotify" }),
            serde_json::json!({ "exp": 1 }),
        ] {
            assert!(matches!(
                auth.authorize(&passport(overrides)).await,
                Err(AuthError::Unauthorized(_))
            ));
        }
        assert!(matches!(
            auth.authorize("not.a.jwt").await,
            Err(AuthError::Unauthorized(_))
        ));
    }

    #[tokio::test]
    async fn external_identities_cannot_use_spotify() {
        let auth = authenticator().await;
        let forgejo = passport(serde_json::json!({ "identity": {
            "issuer": "forgejo", "kind": "external", "subject": "repo:anurag/k8s",
        } }));
        assert!(matches!(
            auth.authorize(&forgejo).await,
            Err(AuthError::Forbidden(_))
        ));
    }

    #[test]
    fn subgraph_auth_needs_an_audience_and_jwks() {
        let config: AuthConfig =
            r#"{"audience":"spotify","jwks":"http://router/.well-known/passport.jwks","grants":{}}"#
                .parse()
                .unwrap();
        assert_eq!(config.audience, "spotify");
        assert!(r#"{"providers":[]}"#.parse::<AuthConfig>().is_err());
        assert!(r#"{"audience":"","jwks":"x"}"#.parse::<AuthConfig>().is_err());
    }
}
