//! Recently played tracks from the Web API, for the weeks before the next extended history
//! export. An account links its Spotify user once (`link`); `spotify-server recent` then polls
//! each link (`poll`), keeping every play it hasn't seen in `recent_plays`.
//!
//! The API keeps only the last 50 plays and reports no play length, so these are a stopgap:
//! `all_plays` hides them inside `extended_coverage`, and the export's plays take their place
//! once it is imported.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::enrich::spotify::{Fetch, PlayHistory, SpotifyClient, SpotifyConfig, TokenError};
use crate::enrich::store_track;

pub const SCOPE: &str = "user-read-recently-played";

/// Pages fetched per link per run; each holds up to 50 plays, more than Spotify keeps.
const MAX_PAGES: usize = 3;

#[derive(Debug)]
pub struct Link {
    pub account_id: i64,
    pub spotify_user_id: String,
    pub linked_at: DateTime<Utc>,
    pub last_played_at: Option<DateTime<Utc>>,
    pub last_polled_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl Link {
    pub async fn by_account(pool: &PgPool, account_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Link,
            "SELECT account_id, spotify_user_id, linked_at, last_played_at, last_polled_at,
                    revoked_at
             FROM spotify_links WHERE account_id = $1",
            account_id
        )
        .fetch_optional(pool)
        .await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkError {
    #[error("spotify rejected the authorization code: {0}")]
    Exchange(String),
    #[error("spotify didn't grant {SCOPE}")]
    MissingScope,
    #[error("couldn't read the spotify user: {0}")]
    User(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// Exchanges an authorization code for the account's refresh token and stores the link. Linking
/// a different Spotify user starts its recent plays afresh.
pub async fn link(
    pool: &PgPool,
    config: SpotifyConfig,
    account_id: i64,
    code: &str,
    redirect_uri: &str,
) -> Result<Link, LinkError> {
    let (client, scope) = SpotifyClient::from_code(config, code, redirect_uri)
        .await
        .map_err(|e| LinkError::Exchange(e.to_string()))?;
    if !scope.split_whitespace().any(|s| s == SCOPE) {
        return Err(LinkError::MissingScope);
    }
    let user = match client.me().await {
        Fetch::Ok(user) => user,
        Fetch::NotFound => return Err(LinkError::User("not found".into())),
        Fetch::RateLimited(wait) => {
            return Err(LinkError::User(format!("rate limited for {wait:?}")));
        }
        Fetch::Failed(e) => return Err(LinkError::User(e)),
    };
    let refresh_token = client.refresh_token().await.unwrap_or_default();

    Ok(sqlx::query_as!(
        Link,
        "INSERT INTO spotify_links (account_id, spotify_user_id, refresh_token, scope)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (account_id) DO UPDATE SET
            refresh_token = EXCLUDED.refresh_token,
            scope = EXCLUDED.scope,
            linked_at = now(),
            revoked_at = NULL,
            last_played_at = CASE
                WHEN spotify_links.spotify_user_id = EXCLUDED.spotify_user_id
                THEN spotify_links.last_played_at END,
            spotify_user_id = EXCLUDED.spotify_user_id
         RETURNING account_id, spotify_user_id, linked_at, last_played_at, last_polled_at,
                   revoked_at",
        account_id,
        user.id,
        refresh_token,
        scope,
    )
    .fetch_one(pool)
    .await?)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PollReport {
    pub links: i32,
    pub plays: i32,
    pub inserted: i32,
    pub revoked: i32,
    pub failed: i32,
}

/// Fetches new recent plays for every linked account. A failing link is logged and skipped
/// so it can't hold up the others.
pub async fn poll(pool: &PgPool, config: &SpotifyConfig) -> anyhow::Result<PollReport> {
    let links = sqlx::query!(
        "SELECT account_id, refresh_token, last_played_at FROM spotify_links
         WHERE revoked_at IS NULL
         ORDER BY account_id"
    )
    .fetch_all(pool)
    .await?;

    let mut report = PollReport::default();
    for link in links {
        report.links += 1;
        let client = SpotifyClient::for_user(config.clone(), link.refresh_token.clone());

        match client.token(false).await {
            Ok(_) => {}
            Err(TokenError::Revoked) => {
                tracing::warn!(account_id = link.account_id, "spotify link revoked");
                sqlx::query!(
                    "UPDATE spotify_links SET revoked_at = now() WHERE account_id = $1",
                    link.account_id
                )
                .execute(pool)
                .await?;
                report.revoked += 1;
                continue;
            }
            Err(TokenError::Failed(error)) => {
                tracing::warn!(account_id = link.account_id, %error, "spotify token failed");
                report.failed += 1;
                continue;
            }
        }

        let mut after = link.last_played_at;
        let mut failed = false;
        for _ in 0..MAX_PAGES {
            let page = match client.recently_played(after).await {
                Fetch::Ok(page) => page,
                Fetch::RateLimited(wait) => {
                    tracing::warn!(account_id = link.account_id, ?wait, "rate limited");
                    failed = true;
                    break;
                }
                Fetch::NotFound => break,
                Fetch::Failed(error) => {
                    tracing::warn!(account_id = link.account_id, %error, "recently played failed");
                    failed = true;
                    break;
                }
            };
            if page.items.is_empty() {
                break;
            }
            report.plays += page.items.len() as i32;
            report.inserted += store(pool, link.account_id, &page.items).await? as i32;

            let newest = page.items.iter().map(|p| p.played_at).max();
            if newest <= after {
                break;
            }
            after = newest;
        }
        if failed {
            report.failed += 1;
        }

        // Keep the rotated refresh token even when fetching failed, or the next run would
        // present a revoked one.
        let refresh_token = client.refresh_token().await.unwrap_or(link.refresh_token);
        sqlx::query!(
            "UPDATE spotify_links
             SET refresh_token = $2, last_polled_at = now(),
                 last_played_at = greatest(last_played_at, $3)
             WHERE account_id = $1",
            link.account_id,
            refresh_token,
            after,
        )
        .execute(pool)
        .await?;
    }

    Ok(report)
}

/// Stores a page of recent plays and their tracks, returning how many plays were new. Track
/// names follow `items.as_of` like export rows, with each play's time as its as_of, so a newer
/// export's names win. Tracks not yet enriched are filled in from the embedded track, saving
/// enrichment a request each.
pub async fn store(pool: &PgPool, account_id: i64, plays: &[PlayHistory]) -> sqlx::Result<u64> {
    let mut latest: HashMap<&str, &PlayHistory> = HashMap::new();
    for play in plays {
        latest
            .entry(play.track.uri.as_str())
            .and_modify(|p| {
                if play.played_at > p.played_at {
                    *p = play;
                }
            })
            .or_insert(play);
    }
    let tracks: Vec<&PlayHistory> = latest.into_values().collect();
    let uris: Vec<&str> = tracks.iter().map(|p| p.track.uri.as_str()).collect();
    let names: Vec<&str> = tracks.iter().map(|p| p.track.name.as_str()).collect();
    let artists: Vec<Option<&str>> = tracks
        .iter()
        .map(|p| p.track.track.artists.first().map(|a| a.name.as_str()))
        .collect();
    let albums: Vec<&str> = tracks
        .iter()
        .map(|p| p.track.track.album.name.as_str())
        .collect();
    let as_ofs: Vec<DateTime<Utc>> = tracks.iter().map(|p| p.played_at).collect();

    let mut tx = pool.begin().await?;
    sqlx::query!(
        "INSERT INTO items (uri, kind, name, artist, album, as_of)
         SELECT u.uri, 'track', nullif(u.name, ''), nullif(u.artist, ''), nullif(u.album, ''),
                u.as_of
         FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::timestamptz[])
            AS u(uri, name, artist, album, as_of)
         ON CONFLICT (uri) DO UPDATE SET
            name = coalesce(EXCLUDED.name, items.name),
            artist = coalesce(EXCLUDED.artist, items.artist),
            album = coalesce(EXCLUDED.album, items.album),
            as_of = EXCLUDED.as_of
         WHERE items.as_of <= EXCLUDED.as_of",
        &uris as &[&str],
        &names as &[&str],
        &artists as &[Option<&str>],
        &albums as &[&str],
        &as_ofs,
    )
    .execute(&mut *tx)
    .await?;

    // Local files have no catalogue entry to enrich.
    let unenriched = sqlx::query_scalar!(
        r#"SELECT u.uri AS "uri!" FROM UNNEST($1::text[]) AS u(uri)
           LEFT JOIN item_meta m ON m.uri = u.uri
           WHERE u.uri LIKE 'spotify:track:%' AND (m.uri IS NULL OR m.status <> 'ok')"#,
        &uris as &[&str],
    )
    .fetch_all(&mut *tx)
    .await?;
    for play in &tracks {
        if unenriched.iter().any(|uri| uri == &play.track.uri) {
            store_track(&mut tx, &play.track.uri, &play.track.track).await?;
        }
    }

    let played_at: Vec<DateTime<Utc>> = plays.iter().map(|p| p.played_at).collect();
    let item_uris: Vec<&str> = plays.iter().map(|p| p.track.uri.as_str()).collect();
    let contexts: Vec<Option<&str>> = plays
        .iter()
        .map(|p| p.context.as_ref().and_then(|c| c.uri.as_deref()))
        .collect();
    let inserted = sqlx::query!(
        "INSERT INTO recent_plays (account_id, played_at, item_uri, context_uri)
         SELECT $1, u.played_at, u.item_uri, u.context_uri
         FROM UNNEST($2::timestamptz[], $3::text[], $4::text[])
            AS u(played_at, item_uri, context_uri)
         ON CONFLICT (account_id, played_at) DO NOTHING",
        account_id,
        &played_at,
        &item_uris as &[&str],
        &contexts as &[Option<&str>],
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    tx.commit().await?;
    Ok(inserted)
}
