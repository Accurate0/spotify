//! `spotify-server enrich`: fills in Spotify Web API metadata (durations, albums, artist ids,
//! genres, artwork) for imported items, most-listened first, within a strict request budget.
//!
//! Development Mode apps get one item per request and an unpublished rate limit, so each run
//! spends at most `max_requests_per_run`, never more than `daily_requests` in a rolling 24h,
//! with `request_interval` between calls. A long 429 records `rate_limited_until` and every
//! run before then exits without calling Spotify.

pub mod artwork;
pub mod spotify;

use std::time::Duration;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use artwork::ArtworkStore;
use spotify::{Fetch, SpotifyClient, largest_image, release_year, uri_id};

/// Queue entries fetched per database round trip.
const QUEUE_BATCH: i64 = 25;

#[derive(Debug, Clone)]
pub struct EnrichConfig {
    pub request_interval: Duration,
    pub max_requests_per_run: i64,
    pub daily_requests: i64,
    pub images_per_run: i64,
    pub image_interval: Duration,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RunReport {
    /// Why the API phase didn't run, if it didn't.
    pub skipped: Option<String>,
    pub api_requests: i64,
    pub items_done: i32,
    pub artists_done: i32,
    pub not_found: i32,
    pub failed: i32,
    pub images: i32,
    pub rate_limited_until: Option<DateTime<Utc>>,
}

/// Requests the next run may make: the per-run cap, limited by what's left of the rolling
/// 24h budget.
pub fn run_budget(config: &EnrichConfig, used_last_24h: i64) -> i64 {
    config
        .max_requests_per_run
        .min(config.daily_requests - used_last_24h)
        .max(0)
}

pub struct Budget {
    pub used_last_24h: i64,
    pub rate_limited_until: Option<DateTime<Utc>>,
}

pub async fn budget(pool: &PgPool) -> sqlx::Result<Budget> {
    let row = sqlx::query!(
        r#"SELECT
               (SELECT coalesce(sum(api_requests), 0)::bigint FROM enrich_runs
                WHERE started_at > now() - interval '24 hours') AS "used!",
               (SELECT max(rate_limited_until) FROM enrich_runs) AS rate_limited_until"#
    )
    .fetch_one(pool)
    .await?;
    Ok(Budget {
        used_last_24h: row.used,
        rate_limited_until: row.rate_limited_until.filter(|until| *until > Utc::now()),
    })
}

pub struct QueueEntry {
    pub target: String,
    pub id: String,
    pub kind: String,
}

/// Next entries to fetch: music by listening time, then episodes.
pub async fn next_in_queue(pool: &PgPool, limit: i64) -> sqlx::Result<Vec<QueueEntry>> {
    sqlx::query_as!(
        QueueEntry,
        r#"SELECT target AS "target!", id AS "id!", kind AS "kind!"
           FROM enrich_queue
           ORDER BY kind = 'episode', score DESC, id
           LIMIT $1"#,
        limit
    )
    .fetch_all(pool)
    .await
}

enum Outcome {
    Item,
    Artist,
    NotFound,
    Failed,
    RateLimited(Duration),
}

pub async fn run(
    pool: &PgPool,
    client: &SpotifyClient,
    store: Option<&ArtworkStore>,
    config: &EnrichConfig,
) -> anyhow::Result<RunReport> {
    let run_id = sqlx::query_scalar!("INSERT INTO enrich_runs DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await?;
    let mut report = RunReport::default();

    let Budget {
        used_last_24h,
        rate_limited_until,
    } = budget(pool).await?;
    let mut remaining = run_budget(config, used_last_24h);
    tracing::info!(
        budget = remaining,
        used_last_24h,
        daily = config.daily_requests,
        "enrichment run starting"
    );

    if let Some(until) = rate_limited_until {
        report.skipped = Some(format!("rate limited until {until}"));
    } else if remaining == 0 {
        report.skipped = Some(format!(
            "daily budget used ({used_last_24h} of {} requests in 24h)",
            config.daily_requests
        ));
    } else {
        let mut first = true;
        let mut position = 0;
        'queue: while remaining > 0 {
            let entries = next_in_queue(pool, remaining.min(QUEUE_BATCH)).await?;
            if entries.is_empty() {
                tracing::info!("queue empty, everything is enriched");
                break;
            }
            for entry in entries {
                if remaining <= 0 {
                    break 'queue;
                }
                if !first {
                    tokio::time::sleep(config.request_interval).await;
                }
                first = false;

                let before = client.requests();
                let outcome = process(pool, client, &entry).await?;
                let used = client.requests() - before;
                remaining -= used;
                report.api_requests += used;
                sqlx::query!(
                    "UPDATE enrich_runs SET api_requests = $2 WHERE id = $1",
                    run_id,
                    report.api_requests as i32
                )
                .execute(pool)
                .await?;

                position += 1;
                let result = match &outcome {
                    Outcome::Item | Outcome::Artist => "ok",
                    Outcome::NotFound => "not found",
                    Outcome::Failed => "failed",
                    Outcome::RateLimited(_) => "rate limited",
                };
                tracing::info!(n = position, remaining, kind = %entry.kind, id = %entry.id, result, "fetched");

                match outcome {
                    Outcome::Item => report.items_done += 1,
                    Outcome::Artist => report.artists_done += 1,
                    Outcome::NotFound => report.not_found += 1,
                    Outcome::Failed => report.failed += 1,
                    Outcome::RateLimited(wait) => {
                        let until = Utc::now()
                            + chrono::Duration::from_std(wait)
                                .unwrap_or(chrono::Duration::hours(1));
                        tracing::warn!(%until, "spotify rate limited, pausing enrichment");
                        report.rate_limited_until = Some(until);
                        break 'queue;
                    }
                }
            }
        }
    }

    if let Some(reason) = &report.skipped {
        tracing::info!(%reason, "skipping spotify api calls this run");
    }

    if let Some(store) = store {
        report.images = download_artwork(pool, client, store, config).await?;
    }

    if report.items_done + report.artists_done > 0 {
        sqlx::query!("REFRESH MATERIALIZED VIEW CONCURRENTLY artist_by_name")
            .execute(pool)
            .await?;
    }

    sqlx::query!(
        "UPDATE enrich_runs SET finished_at = now(), api_requests = $2, image_downloads = $3,
                items_done = $4, artists_done = $5, rate_limited_until = $6, note = $7
         WHERE id = $1",
        run_id,
        report.api_requests as i32,
        report.images,
        report.items_done,
        report.artists_done,
        report.rate_limited_until,
        report.skipped,
    )
    .execute(pool)
    .await?;

    Ok(report)
}

async fn process(
    pool: &PgPool,
    client: &SpotifyClient,
    entry: &QueueEntry,
) -> anyhow::Result<Outcome> {
    // Artist entries are bare ids; items are URIs (`spotify:track:<id>`).
    let id = if entry.target == "artist" {
        Some(entry.id.as_str())
    } else {
        uri_id(&entry.id)
    };
    let Some(id) = id else {
        mark_item(pool, &entry.id, "not_found", None).await?;
        return Ok(Outcome::NotFound);
    };

    match (entry.target.as_str(), entry.kind.as_str()) {
        ("artist", _) => match client.artist(id).await {
            Fetch::Ok(artist) => {
                let image = largest_image(&artist.images).map(|i| i.url.clone());
                sqlx::query!(
                    "INSERT INTO spotify_artists (id, name, genres, image_source_url, status)
                     VALUES ($1, $2, $3, $4, 'ok')
                     ON CONFLICT (id) DO UPDATE SET
                        name = EXCLUDED.name,
                        genres = EXCLUDED.genres,
                        image_key = CASE
                            WHEN spotify_artists.image_source_url IS DISTINCT FROM EXCLUDED.image_source_url
                            THEN NULL ELSE spotify_artists.image_key END,
                        image_source_url = EXCLUDED.image_source_url,
                        status = 'ok', last_error = NULL, fetched_at = now()",
                    entry.id,
                    artist.name,
                    &artist.genres,
                    image,
                )
                .execute(pool)
                .await?;
                Ok(Outcome::Artist)
            }
            Fetch::NotFound => {
                mark_artist(pool, &entry.id, "not_found", None).await?;
                Ok(Outcome::NotFound)
            }
            Fetch::Failed(error) => {
                tracing::warn!(artist = %entry.id, %error, "artist fetch failed");
                mark_artist(pool, &entry.id, "failed", Some(&error)).await?;
                Ok(Outcome::Failed)
            }
            Fetch::RateLimited(wait) => Ok(Outcome::RateLimited(wait)),
        },
        (_, "episode") => match client.episode(id).await {
            Fetch::Ok(episode) => {
                let image = largest_image(&episode.show.images).map(|i| i.url.clone());
                let mut tx = pool.begin().await?;
                upsert_album(
                    &mut tx,
                    &episode.show.id,
                    &episode.show.name,
                    "show",
                    None,
                    image,
                )
                .await?;
                sqlx::query!(
                    "INSERT INTO item_meta (uri, duration_ms, explicit, album_id, status)
                     VALUES ($1, $2, $3, $4, 'ok')
                     ON CONFLICT (uri) DO UPDATE SET
                        duration_ms = EXCLUDED.duration_ms, explicit = EXCLUDED.explicit,
                        album_id = EXCLUDED.album_id, status = 'ok', last_error = NULL,
                        fetched_at = now()",
                    entry.id,
                    episode.duration_ms,
                    episode.explicit,
                    episode.show.id,
                )
                .execute(&mut *tx)
                .await?;
                tx.commit().await?;
                Ok(Outcome::Item)
            }
            other => item_failure(pool, &entry.id, other).await,
        },
        _ => match client.track(id).await {
            Fetch::Ok(track) => {
                let image = largest_image(&track.album.images).map(|i| i.url.clone());
                let mut tx = pool.begin().await?;
                if let Some(album_id) = &track.album.id {
                    upsert_album(
                        &mut tx,
                        album_id,
                        &track.album.name,
                        track.album.album_type.as_deref().unwrap_or("album"),
                        track.album.release_date.as_deref(),
                        image,
                    )
                    .await?;
                }
                sqlx::query!(
                    "INSERT INTO item_meta (uri, duration_ms, explicit, album_id, status)
                     VALUES ($1, $2, $3, $4, 'ok')
                     ON CONFLICT (uri) DO UPDATE SET
                        duration_ms = EXCLUDED.duration_ms, explicit = EXCLUDED.explicit,
                        album_id = EXCLUDED.album_id, status = 'ok', last_error = NULL,
                        fetched_at = now()",
                    entry.id,
                    track.duration_ms,
                    track.explicit,
                    track.album.id,
                )
                .execute(&mut *tx)
                .await?;

                sqlx::query!("DELETE FROM item_artists WHERE item_uri = $1", entry.id)
                    .execute(&mut *tx)
                    .await?;
                let credited: Vec<_> = track
                    .artists
                    .iter()
                    .filter_map(|a| Some((a.id.clone()?, a.name.clone())))
                    .collect();
                let positions: Vec<i32> = (0..credited.len() as i32).collect();
                let ids: Vec<String> = credited.iter().map(|(id, _)| id.clone()).collect();
                let names: Vec<String> = credited.iter().map(|(_, name)| name.clone()).collect();
                sqlx::query!(
                    "INSERT INTO item_artists (item_uri, position, artist_id, artist_name)
                     SELECT $1, u.position, u.artist_id, u.artist_name
                     FROM UNNEST($2::int[], $3::text[], $4::text[])
                        AS u(position, artist_id, artist_name)",
                    entry.id,
                    &positions,
                    &ids,
                    &names,
                )
                .execute(&mut *tx)
                .await?;
                tx.commit().await?;
                Ok(Outcome::Item)
            }
            other => item_failure(pool, &entry.id, other).await,
        },
    }
}

async fn upsert_album(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: &str,
    name: &str,
    album_type: &str,
    release_date: Option<&str>,
    image: Option<String>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO spotify_albums (id, name, album_type, release_date, release_year, image_source_url)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            album_type = EXCLUDED.album_type,
            release_date = EXCLUDED.release_date,
            release_year = EXCLUDED.release_year,
            image_key = CASE
                WHEN spotify_albums.image_source_url IS DISTINCT FROM EXCLUDED.image_source_url
                THEN NULL ELSE spotify_albums.image_key END,
            image_source_url = EXCLUDED.image_source_url,
            fetched_at = now()",
        id,
        name,
        album_type,
        release_date,
        release_date.and_then(release_year),
        image,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn item_failure<T>(pool: &PgPool, uri: &str, fetch: Fetch<T>) -> anyhow::Result<Outcome> {
    Ok(match fetch {
        Fetch::Ok(_) => unreachable!("successful fetches are handled by the caller"),
        Fetch::NotFound => {
            mark_item(pool, uri, "not_found", None).await?;
            Outcome::NotFound
        }
        Fetch::Failed(error) => {
            tracing::warn!(%uri, %error, "item fetch failed");
            mark_item(pool, uri, "failed", Some(&error)).await?;
            Outcome::Failed
        }
        Fetch::RateLimited(wait) => Outcome::RateLimited(wait),
    })
}

async fn mark_item(
    pool: &PgPool,
    uri: &str,
    status: &str,
    error: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO item_meta (uri, status, attempts, last_error) VALUES ($1, $2, 1, $3)
         ON CONFLICT (uri) DO UPDATE SET
            status = EXCLUDED.status, attempts = item_meta.attempts + 1,
            last_error = EXCLUDED.last_error, fetched_at = now()",
        uri,
        status,
        error,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn mark_artist(
    pool: &PgPool,
    id: &str,
    status: &str,
    error: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO spotify_artists (id, status, attempts, last_error) VALUES ($1, $2, 1, $3)
         ON CONFLICT (id) DO UPDATE SET
            status = EXCLUDED.status, attempts = spotify_artists.attempts + 1,
            last_error = EXCLUDED.last_error, fetched_at = now()",
        id,
        status,
        error,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Downloads artwork that hasn't been stored yet, oldest-fetched (most listened) first.
async fn download_artwork(
    pool: &PgPool,
    client: &SpotifyClient,
    store: &ArtworkStore,
    config: &EnrichConfig,
) -> anyhow::Result<i32> {
    let pending = sqlx::query!(
        r#"SELECT kind AS "kind!", id AS "id!", url AS "url!" FROM (
               SELECT 'albums' AS kind, id, image_source_url AS url, fetched_at
               FROM spotify_albums WHERE image_source_url IS NOT NULL AND image_key IS NULL
               UNION ALL
               SELECT 'artists', id, image_source_url, fetched_at
               FROM spotify_artists
               WHERE status = 'ok' AND image_source_url IS NOT NULL AND image_key IS NULL
           ) pending
           ORDER BY fetched_at, id
           LIMIT $1"#,
        config.images_per_run,
    )
    .fetch_all(pool)
    .await?;

    tracing::info!(pending = pending.len(), "downloading artwork");
    let mut stored = 0;
    for (i, image) in pending.iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(config.image_interval).await;
        }
        let key = format!("{}/{}.jpg", image.kind, image.id);

        // A CDN URL that fails won't fix itself; forget it rather than retrying every run.
        let bytes = match client.download(&image.url).await {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::warn!(%key, %error, "artwork download failed, dropping its url");
                if image.kind == "albums" {
                    sqlx::query!(
                        "UPDATE spotify_albums SET image_source_url = NULL WHERE id = $1",
                        image.id
                    )
                    .execute(pool)
                    .await?;
                } else {
                    sqlx::query!(
                        "UPDATE spotify_artists SET image_source_url = NULL WHERE id = $1",
                        image.id
                    )
                    .execute(pool)
                    .await?;
                }
                continue;
            }
        };

        // The bucket being unreachable says nothing about the image: keep its url and stop
        // for this run, so the next one retries.
        if let Err(error) = store.put(&key, &bytes, "image/jpeg").await {
            tracing::warn!(%key, %error, "artwork bucket unavailable, stopping downloads for this run");
            break;
        }

        if image.kind == "albums" {
            sqlx::query!(
                "UPDATE spotify_albums SET image_key = $2 WHERE id = $1",
                image.id,
                key
            )
            .execute(pool)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE spotify_artists SET image_key = $2 WHERE id = $1",
                image.id,
                key
            )
            .execute(pool)
            .await?;
        }
        stored += 1;
        if stored % 50 == 0 {
            tracing::info!(stored, of = pending.len(), "artwork progress");
        }
    }
    Ok(stored)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(max: i64, daily: i64) -> EnrichConfig {
        EnrichConfig {
            request_interval: Duration::ZERO,
            max_requests_per_run: max,
            daily_requests: daily,
            images_per_run: 0,
            image_interval: Duration::ZERO,
        }
    }

    #[test]
    fn budget_is_per_run_cap_within_daily_remainder() {
        assert_eq!(run_budget(&config(180, 5000), 0), 180);
        assert_eq!(run_budget(&config(180, 5000), 4900), 100);
        assert_eq!(run_budget(&config(180, 5000), 5000), 0);
        assert_eq!(run_budget(&config(180, 5000), 6000), 0);
    }
}
