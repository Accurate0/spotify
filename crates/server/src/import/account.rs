//! Inserts for account data files. Event kinds (basic history, searches) dedupe on a natural
//! key across exports; snapshot kinds are stored per file and resolved to the newest export by
//! the `current_snapshot_files` view.

use chrono::{DateTime, NaiveDate, Utc};
use spotify_export::{BasicPlay, Follow, LibraryItem, Marquee, Playlist, Search};
use sqlx::{Postgres, Transaction};

use super::RowCounts;

type Tx<'a> = Transaction<'a, Postgres>;

fn counts(total: usize, inserted: u64) -> RowCounts {
    RowCounts {
        total: total as i32,
        inserted: inserted as i32,
        // Rows already present from an overlapping export.
        merged: (total as u64).saturating_sub(inserted) as i32,
    }
}

pub async fn basic_history(
    tx: &mut Tx<'_>,
    account_id: i64,
    file_id: i64,
    plays: &[BasicPlay],
) -> Result<RowCounts, sqlx::Error> {
    let end_time: Vec<DateTime<Utc>> = plays.iter().map(|p| p.end_time).collect();
    let artist: Vec<Option<String>> = plays.iter().map(|p| p.artist.clone()).collect();
    let track: Vec<Option<String>> = plays.iter().map(|p| p.track.clone()).collect();
    let ms_played: Vec<i32> = plays.iter().map(|p| p.ms_played).collect();

    let result = sqlx::query!(
        "INSERT INTO basic_plays (account_id, file_id, end_time, artist, track, ms_played)
         SELECT $1, $2, u.end_time, u.artist, u.track, u.ms_played
         FROM UNNEST($3::timestamptz[], $4::text[], $5::text[], $6::int[])
            AS u(end_time, artist, track, ms_played)
         ON CONFLICT DO NOTHING",
        account_id,
        file_id,
        &end_time,
        &artist as &[Option<String>],
        &track as &[Option<String>],
        &ms_played,
    )
    .execute(&mut **tx)
    .await?;

    Ok(counts(plays.len(), result.rows_affected()))
}

pub async fn searches(
    tx: &mut Tx<'_>,
    account_id: i64,
    file_id: i64,
    searches: &[Search],
) -> Result<RowCounts, sqlx::Error> {
    let ts: Vec<DateTime<Utc>> = searches.iter().map(|s| s.ts).collect();
    let platform: Vec<Option<String>> = searches.iter().map(|s| s.platform.clone()).collect();
    let query: Vec<String> = searches.iter().map(|s| s.query.clone()).collect();
    // Postgres arrays can't be ragged, so interaction URIs travel as JSON arrays.
    let uris: Vec<serde_json::Value> = searches
        .iter()
        .map(|s| serde_json::json!(s.interaction_uris))
        .collect();

    let result = sqlx::query!(
        "INSERT INTO searches (account_id, file_id, ts, platform, query, interaction_uris)
         SELECT $1, $2, u.ts, u.platform, u.query,
                ARRAY(SELECT jsonb_array_elements_text(u.uris))
         FROM UNNEST($3::timestamptz[], $4::text[], $5::text[], $6::jsonb[])
            AS u(ts, platform, query, uris)
         ON CONFLICT DO NOTHING",
        account_id,
        file_id,
        &ts,
        &platform as &[Option<String>],
        &query,
        &uris,
    )
    .execute(&mut **tx)
    .await?;

    Ok(counts(searches.len(), result.rows_affected()))
}

pub async fn library(
    tx: &mut Tx<'_>,
    account_id: i64,
    file_id: i64,
    items: &[LibraryItem],
) -> Result<RowCounts, sqlx::Error> {
    let kind: Vec<&str> = items.iter().map(|i| i.kind.as_str()).collect();
    let uri: Vec<Option<String>> = items.iter().map(|i| i.uri.clone()).collect();
    let name: Vec<String> = items.iter().map(|i| i.name.clone()).collect();
    let artist: Vec<Option<String>> = items.iter().map(|i| i.artist.clone()).collect();
    let album: Vec<Option<String>> = items.iter().map(|i| i.album.clone()).collect();

    let result = sqlx::query!(
        "INSERT INTO library_items (account_id, file_id, kind, uri, name, artist, album)
         SELECT $1, $2, u.kind, u.uri, u.name, u.artist, u.album
         FROM UNNEST($3::text[], $4::text[], $5::text[], $6::text[], $7::text[])
            AS u(kind, uri, name, artist, album)",
        account_id,
        file_id,
        &kind as &[&str],
        &uri as &[Option<String>],
        &name,
        &artist as &[Option<String>],
        &album as &[Option<String>],
    )
    .execute(&mut **tx)
    .await?;

    Ok(counts(items.len(), result.rows_affected()))
}

/// Counts are playlist items; playlists themselves are few.
pub async fn playlists(
    tx: &mut Tx<'_>,
    account_id: i64,
    file_id: i64,
    playlists: &[Playlist],
) -> Result<RowCounts, sqlx::Error> {
    let mut total = 0;

    for playlist in playlists {
        let playlist_id = sqlx::query_scalar!(
            "INSERT INTO playlists (account_id, file_id, name, last_modified, description, followers)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id",
            account_id,
            file_id,
            playlist.name,
            playlist.last_modified,
            playlist.description,
            playlist.followers,
        )
        .fetch_one(&mut **tx)
        .await?;

        let items = &playlist.items;
        let uri: Vec<Option<String>> = items.iter().map(|i| i.uri.clone()).collect();
        let name: Vec<Option<String>> = items.iter().map(|i| i.name.clone()).collect();
        let artist: Vec<Option<String>> = items.iter().map(|i| i.artist.clone()).collect();
        let album: Vec<Option<String>> = items.iter().map(|i| i.album.clone()).collect();
        let added_date: Vec<Option<NaiveDate>> = items.iter().map(|i| i.added_date).collect();

        sqlx::query!(
            "INSERT INTO playlist_items (playlist_id, position, uri, name, artist, album, added_date)
             SELECT $1, u.ord - 1, u.uri, u.name, u.artist, u.album, u.added_date
             FROM UNNEST($2::text[], $3::text[], $4::text[], $5::text[], $6::date[])
                WITH ORDINALITY AS u(uri, name, artist, album, added_date, ord)",
            playlist_id,
            &uri as &[Option<String>],
            &name as &[Option<String>],
            &artist as &[Option<String>],
            &album as &[Option<String>],
            &added_date as &[Option<NaiveDate>],
        )
        .execute(&mut **tx)
        .await?;

        total += items.len();
    }

    Ok(counts(total, total as u64))
}

pub async fn follow(
    tx: &mut Tx<'_>,
    account_id: i64,
    file_id: i64,
    follow: Follow,
) -> Result<RowCounts, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO follow_snapshots (file_id, account_id, followers, following, dismissing)
         VALUES ($1, $2, $3, $4, $5)",
        file_id,
        account_id,
        follow.follower_count,
        follow.following_users_count,
        follow.dismissing_users_count,
    )
    .execute(&mut **tx)
    .await?;

    Ok(counts(1, 1))
}

pub async fn marquee(
    tx: &mut Tx<'_>,
    account_id: i64,
    file_id: i64,
    rows: &[Marquee],
) -> Result<RowCounts, sqlx::Error> {
    let artist: Vec<String> = rows.iter().map(|r| r.artist_name.clone()).collect();
    let segment: Vec<String> = rows.iter().map(|r| r.segment.clone()).collect();

    let result = sqlx::query!(
        "INSERT INTO marquee (account_id, file_id, artist, segment)
         SELECT $1, $2, u.artist, u.segment
         FROM UNNEST($3::text[], $4::text[]) AS u(artist, segment)",
        account_id,
        file_id,
        &artist,
        &segment,
    )
    .execute(&mut **tx)
    .await?;

    Ok(counts(rows.len(), result.rows_affected()))
}

pub async fn inferences(
    tx: &mut Tx<'_>,
    account_id: i64,
    file_id: i64,
    values: &[String],
) -> Result<RowCounts, sqlx::Error> {
    let result = sqlx::query!(
        "INSERT INTO inferences (account_id, file_id, value)
         SELECT $1, $2, u.value FROM UNNEST($3::text[]) AS u(value)",
        account_id,
        file_id,
        values,
    )
    .execute(&mut **tx)
    .await?;

    Ok(counts(values.len(), result.rows_affected()))
}

/// Fills the account's Spotify username from `Userdata.json` if it isn't already known.
pub async fn username(
    tx: &mut Tx<'_>,
    account_id: i64,
    username: Option<&str>,
) -> Result<RowCounts, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE accounts SET spotify_username = $2 WHERE id = $1 AND spotify_username IS NULL",
        account_id,
        username,
    )
    .execute(&mut **tx)
    .await?;

    Ok(counts(username.is_some() as usize, result.rows_affected()))
}
