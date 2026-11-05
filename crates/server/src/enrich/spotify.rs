//! Minimal Spotify Web API client for enrichment: an app (client credentials) token and
//! single-item lookups. Development Mode apps have no batch endpoints, so callers pace these.

use std::time::{Duration, Instant};

use reqwest::StatusCode;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;

/// A 429 asking us to wait at most this long is retried in place; longer ends the run.
const MAX_INLINE_RETRY_AFTER: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
    pub api_url: String,
    pub accounts_url: String,
}

#[derive(Debug)]
pub enum Fetch<T> {
    Ok(T),
    /// 404 (or 400 for malformed ids): the item no longer exists on Spotify.
    NotFound,
    /// Spotify asked us to stop for this long.
    RateLimited(Duration),
    Failed(String),
}

pub struct SpotifyClient {
    http: reqwest::Client,
    config: SpotifyConfig,
    token: Mutex<Option<(String, Instant)>>,
    /// API requests sent, including token refreshes and retries; counts toward the budget.
    pub requests: std::sync::atomic::AtomicI64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    pub url: String,
    pub width: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SimpleArtist {
    pub id: Option<String>,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Album {
    pub id: Option<String>,
    pub name: String,
    pub album_type: Option<String>,
    pub release_date: Option<String>,
    #[serde(default)]
    pub images: Vec<Image>,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub duration_ms: i32,
    pub explicit: bool,
    pub album: Album,
    #[serde(default)]
    pub artists: Vec<SimpleArtist>,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub name: String,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub images: Vec<Image>,
}

#[derive(Debug, Deserialize)]
pub struct Show {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub images: Vec<Image>,
}

#[derive(Debug, Deserialize)]
pub struct Episode {
    pub duration_ms: i32,
    pub explicit: bool,
    pub show: Show,
}

impl SpotifyClient {
    pub fn new(config: SpotifyConfig) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(20))
                .user_agent("spotify-export-enrich")
                .build()
                .unwrap_or_default(),
            config,
            token: Mutex::new(None),
            requests: Default::default(),
        }
    }

    pub fn requests(&self) -> i64 {
        self.requests.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn count(&self) {
        self.requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    async fn token(&self, refresh: bool) -> Result<String, String> {
        let mut token = self.token.lock().await;
        if !refresh
            && let Some((value, expires)) = token.as_ref()
            && Instant::now() < *expires
        {
            return Ok(value.clone());
        }

        self.count();
        let response = self
            .http
            .post(format!(
                "{}/api/token",
                self.config.accounts_url.trim_end_matches('/')
            ))
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|e| format!("token request failed: {e}"))?;
        if !response.status().is_success() {
            return Err(format!("token request returned {}", response.status()));
        }
        let body: TokenResponse = response
            .json()
            .await
            .map_err(|e| format!("token response unreadable: {e}"))?;

        // Refresh a minute early so a token never expires mid-request.
        let expires = Instant::now() + Duration::from_secs(body.expires_in.saturating_sub(60));
        *token = Some((body.access_token.clone(), expires));
        Ok(body.access_token)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Fetch<T> {
        let url = format!("{}/v1/{path}", self.config.api_url.trim_end_matches('/'));
        let mut refreshed = false;
        let mut waited = false;

        loop {
            let token = match self.token(false).await {
                Ok(token) => token,
                Err(e) => return Fetch::Failed(e),
            };

            self.count();
            let response = match self.http.get(&url).bearer_auth(&token).send().await {
                Ok(response) => response,
                Err(e) => return Fetch::Failed(format!("request failed: {e}")),
            };

            match response.status() {
                status if status.is_success() => {
                    return match response.json().await {
                        Ok(body) => Fetch::Ok(body),
                        Err(e) => Fetch::Failed(format!("unreadable response: {e}")),
                    };
                }
                StatusCode::NOT_FOUND | StatusCode::BAD_REQUEST => return Fetch::NotFound,
                StatusCode::UNAUTHORIZED if !refreshed => {
                    refreshed = true;
                    if let Err(e) = self.token(true).await {
                        return Fetch::Failed(e);
                    }
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    let retry_after = response
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.trim().parse::<u64>().ok())
                        .map(Duration::from_secs)
                        // No header: assume the worst rather than hammering.
                        .unwrap_or(Duration::from_secs(3600));
                    if waited || retry_after > MAX_INLINE_RETRY_AFTER {
                        return Fetch::RateLimited(retry_after);
                    }
                    waited = true;
                    tokio::time::sleep(retry_after).await;
                }
                status => return Fetch::Failed(format!("{path} returned {status}")),
            }
        }
    }

    pub async fn track(&self, id: &str) -> Fetch<Track> {
        self.get(&format!("tracks/{id}")).await
    }

    pub async fn artist(&self, id: &str) -> Fetch<Artist> {
        self.get(&format!("artists/{id}")).await
    }

    /// Episodes need a market with an app token.
    pub async fn episode(&self, id: &str) -> Fetch<Episode> {
        self.get(&format!("episodes/{id}?market=AU")).await
    }

    /// Downloads artwork from Spotify's image CDN (not the Web API; doesn't use the budget).
    pub async fn download(&self, url: &str) -> Result<bytes::Bytes, String> {
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| format!("image request failed: {e}"))?;
        if !response.status().is_success() {
            return Err(format!("image returned {}", response.status()));
        }
        response
            .bytes()
            .await
            .map_err(|e| format!("image unreadable: {e}"))
    }
}

/// The widest image (Spotify usually offers 640, 300 and 64); images without a width sort last.
pub fn largest_image(images: &[Image]) -> Option<&Image> {
    images.iter().max_by_key(|i| i.width.map_or(0, |w| w + 1))
}

/// `1999`, `1999-03` or `1999-03-14`. Spotify uses `0000` for unknown dates.
pub fn release_year(date: &str) -> Option<i32> {
    date.get(..4)?.parse().ok().filter(|year| *year > 0)
}

/// `spotify:track:abc` -> `abc`.
pub fn uri_id(uri: &str) -> Option<&str> {
    uri.rsplit_once(':')
        .map(|(_, id)| id)
        .filter(|id| !id.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_largest_image() {
        let images = vec![
            Image {
                url: "300".into(),
                width: Some(300),
            },
            Image {
                url: "640".into(),
                width: Some(640),
            },
            Image {
                url: "64".into(),
                width: Some(64),
            },
            Image {
                url: "none".into(),
                width: None,
            },
        ];
        assert_eq!(largest_image(&images).unwrap().url, "640");
        assert_eq!(
            largest_image(&[Image {
                url: "x".into(),
                width: None
            }])
            .unwrap()
            .url,
            "x"
        );
        assert!(largest_image(&[]).is_none());
    }

    #[test]
    fn parses_release_years_of_any_precision() {
        assert_eq!(release_year("1999"), Some(1999));
        assert_eq!(release_year("1999-03"), Some(1999));
        assert_eq!(release_year("1999-03-14"), Some(1999));
        assert_eq!(release_year("0000"), None);
        assert_eq!(release_year("99"), None);
    }

    #[test]
    fn extracts_ids_from_uris() {
        assert_eq!(uri_id("spotify:track:abc123"), Some("abc123"));
        assert_eq!(uri_id("spotify:episode:e"), Some("e"));
        assert_eq!(uri_id("nope"), None);
    }
}
