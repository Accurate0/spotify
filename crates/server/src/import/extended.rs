//! Merges extended streaming history into `plays`.
//!
//! The same play appears in every export that covers its time range, but Spotify changed how
//! it records `ts` between exports, so a play's timestamp can differ by a few seconds (mostly
//! one). Rows are matched to existing plays on account, item, ms_played and platform with `ts`
//! within ±3s. Reasons only have to agree when both sides have one, since older exports have
//! null `reason_end` where newer ones do not. On a match, the newer export's non-null values
//! win.
//!
//! Rows are passed to Postgres as UNNEST arrays so every statement is static and checked by
//! the sqlx macros.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use spotify_export::ExtendedPlay;
use sqlx::{Postgres, Transaction};

use super::RowCounts;

pub async fn merge(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
    file_id: i64,
    as_of: DateTime<Utc>,
    plays: &[ExtendedPlay],
) -> Result<RowCounts, sqlx::Error> {
    // Old exports repeat some rows verbatim (mostly 0ms plays from 2015); keep one of each.
    let mut seen = HashSet::new();
    let rows: Vec<&ExtendedPlay> = plays.iter().filter(|p| seen.insert(*p)).collect();

    upsert_items(tx, as_of, &rows).await?;

    let matches = match_existing(tx, account_id, &rows).await?;
    let matched: HashSet<usize> = matches.iter().map(|&(idx, _)| idx).collect();

    let merged = update_matched(tx, file_id, as_of, &rows, &matches).await?;
    let unmatched: Vec<&ExtendedPlay> = rows
        .iter()
        .enumerate()
        .filter(|(idx, _)| !matched.contains(idx))
        .map(|(_, row)| *row)
        .collect();
    let inserted = insert(tx, account_id, file_id, as_of, &unmatched).await?;

    Ok(RowCounts {
        total: plays.len() as i32,
        inserted: inserted as i32,
        merged: merged as i32,
    })
}

/// Item names from this file, taking each URI's most recent play. Only applied when this
/// export is at least as new as the one that last wrote the item.
async fn upsert_items(
    tx: &mut Transaction<'_, Postgres>,
    as_of: DateTime<Utc>,
    rows: &[&ExtendedPlay],
) -> Result<(), sqlx::Error> {
    let mut latest: HashMap<&str, &ExtendedPlay> = HashMap::new();
    for row in rows {
        if let Some(uri) = row.uri.as_deref() {
            latest
                .entry(uri)
                .and_modify(|p| {
                    if row.ts > p.ts {
                        *p = row;
                    }
                })
                .or_insert(row);
        }
    }

    let items: Vec<_> = latest.into_iter().collect();
    let uris: Vec<&str> = items.iter().map(|(uri, _)| *uri).collect();
    let kinds: Vec<&str> = items
        .iter()
        .map(|(_, p)| p.kind.expect("plays with a uri have a kind").as_str())
        .collect();
    let names: Vec<Option<String>> = items.iter().map(|(_, p)| p.name.clone()).collect();
    let artists: Vec<Option<String>> = items.iter().map(|(_, p)| p.artist.clone()).collect();
    let albums: Vec<Option<String>> = items.iter().map(|(_, p)| p.album.clone()).collect();

    sqlx::query!(
        "INSERT INTO items (uri, kind, name, artist, album, as_of)
         SELECT u.uri, u.kind, u.name, u.artist, u.album, $1
         FROM UNNEST($2::text[], $3::text[], $4::text[], $5::text[], $6::text[])
            AS u(uri, kind, name, artist, album)
         ON CONFLICT (uri) DO UPDATE SET
            name = coalesce(EXCLUDED.name, items.name),
            artist = coalesce(EXCLUDED.artist, items.artist),
            album = coalesce(EXCLUDED.album, items.album),
            as_of = EXCLUDED.as_of
         WHERE items.as_of <= EXCLUDED.as_of",
        as_of,
        &uris as &[&str],
        &kinds as &[&str],
        &names as &[Option<String>],
        &artists as &[Option<String>],
        &albums as &[Option<String>],
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Pairs rows (by index) with existing plays one-to-one, closest timestamps first, so two rows
/// can never merge into the same play.
async fn match_existing(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
    rows: &[&ExtendedPlay],
) -> Result<Vec<(usize, i64)>, sqlx::Error> {
    let idx: Vec<i32> = (0..rows.len() as i32).collect();
    let ts: Vec<DateTime<Utc>> = rows.iter().map(|p| p.ts).collect();
    let uri: Vec<Option<String>> = rows.iter().map(|p| p.uri.clone()).collect();
    let ms_played: Vec<i32> = rows.iter().map(|p| p.ms_played).collect();
    let platform: Vec<Option<String>> = rows.iter().map(|p| p.platform.clone()).collect();
    let reason_start: Vec<Option<String>> = rows.iter().map(|p| p.reason_start.clone()).collect();
    let reason_end: Vec<Option<String>> = rows.iter().map(|p| p.reason_end.clone()).collect();
    let name: Vec<Option<String>> = rows.iter().map(|p| p.name.clone()).collect();
    let artist: Vec<Option<String>> = rows.iter().map(|p| p.artist.clone()).collect();

    // Two arms so the URI arm can use plays_dedupe_idx; plays without a URI match on raw names.
    let mut candidates = sqlx::query!(
        r#"WITH s AS (
            SELECT * FROM UNNEST($2::int[], $3::timestamptz[], $4::text[], $5::int[], $6::text[],
                                 $7::text[], $8::text[], $9::text[], $10::text[])
                AS s(idx, ts, uri, ms_played, platform, reason_start, reason_end, name, artist)
         ),
         candidates AS (
            SELECT s.*, p.id AS pid, p.ts AS pts
            FROM s JOIN plays p ON p.account_id = $1 AND p.item_uri = s.uri
                AND p.ms_played = s.ms_played
                AND p.ts BETWEEN s.ts - interval '3 seconds' AND s.ts + interval '3 seconds'
            UNION ALL
            SELECT s.*, p.id, p.ts
            FROM s JOIN plays p ON p.account_id = $1 AND p.item_uri IS NULL AND s.uri IS NULL
                AND p.raw_name IS NOT DISTINCT FROM s.name
                AND p.raw_artist IS NOT DISTINCT FROM s.artist
                AND p.ms_played = s.ms_played
                AND p.ts BETWEEN s.ts - interval '3 seconds' AND s.ts + interval '3 seconds'
         )
         SELECT c.idx AS "idx!", c.pid AS "pid!",
                abs(extract(epoch FROM c.ts - c.pts))::float8 AS "diff!"
         FROM candidates c JOIN plays p ON p.id = c.pid
         WHERE p.platform IS NOT DISTINCT FROM c.platform
           AND (p.reason_start IS NULL OR c.reason_start IS NULL OR p.reason_start = c.reason_start)
           AND (p.reason_end IS NULL OR c.reason_end IS NULL OR p.reason_end = c.reason_end)"#,
        account_id,
        &idx,
        &ts,
        &uri as &[Option<String>],
        &ms_played,
        &platform as &[Option<String>],
        &reason_start as &[Option<String>],
        &reason_end as &[Option<String>],
        &name as &[Option<String>],
        &artist as &[Option<String>],
    )
    .fetch_all(&mut **tx)
    .await?;

    candidates.sort_by(|a, b| a.diff.total_cmp(&b.diff).then(a.idx.cmp(&b.idx)));

    let mut used_rows = HashSet::new();
    let mut used_plays = HashSet::new();

    Ok(candidates
        .into_iter()
        .filter(|c| {
            !used_rows.contains(&c.idx)
                && !used_plays.contains(&c.pid)
                && used_rows.insert(c.idx)
                && used_plays.insert(c.pid)
        })
        .map(|c| (c.idx as usize, c.pid))
        .collect())
}

async fn update_matched(
    tx: &mut Transaction<'_, Postgres>,
    file_id: i64,
    as_of: DateTime<Utc>,
    rows: &[&ExtendedPlay],
    matches: &[(usize, i64)],
) -> Result<u64, sqlx::Error> {
    if matches.is_empty() {
        return Ok(0);
    }

    let row = |idx: usize| rows[idx];
    let pid: Vec<i64> = matches.iter().map(|&(_, pid)| pid).collect();
    let ts: Vec<DateTime<Utc>> = matches.iter().map(|&(i, _)| row(i).ts).collect();
    let platform: Vec<Option<String>> = matches
        .iter()
        .map(|&(i, _)| row(i).platform.clone())
        .collect();
    let conn_country: Vec<Option<String>> = matches
        .iter()
        .map(|&(i, _)| row(i).conn_country.clone())
        .collect();
    let reason_start: Vec<Option<String>> = matches
        .iter()
        .map(|&(i, _)| row(i).reason_start.clone())
        .collect();
    let reason_end: Vec<Option<String>> = matches
        .iter()
        .map(|&(i, _)| row(i).reason_end.clone())
        .collect();
    let shuffle: Vec<Option<bool>> = matches.iter().map(|&(i, _)| row(i).shuffle).collect();
    let skipped: Vec<Option<bool>> = matches.iter().map(|&(i, _)| row(i).skipped).collect();
    let offline: Vec<Option<bool>> = matches.iter().map(|&(i, _)| row(i).offline).collect();
    let offline_ts: Vec<Option<DateTime<Utc>>> =
        matches.iter().map(|&(i, _)| row(i).offline_ts).collect();
    let incognito: Vec<Option<bool>> = matches.iter().map(|&(i, _)| row(i).incognito).collect();

    // When this export is newer than the row's source its values win unless null; otherwise
    // they only fill gaps.
    let result = sqlx::query!(
        "UPDATE plays p SET
            ts = CASE WHEN $1 >= p.source_as_of THEN m.ts ELSE p.ts END,
            platform = CASE WHEN $1 >= p.source_as_of
                THEN coalesce(m.platform, p.platform) ELSE coalesce(p.platform, m.platform) END,
            conn_country = CASE WHEN $1 >= p.source_as_of
                THEN coalesce(m.conn_country, p.conn_country) ELSE coalesce(p.conn_country, m.conn_country) END,
            reason_start = CASE WHEN $1 >= p.source_as_of
                THEN coalesce(m.reason_start, p.reason_start) ELSE coalesce(p.reason_start, m.reason_start) END,
            reason_end = CASE WHEN $1 >= p.source_as_of
                THEN coalesce(m.reason_end, p.reason_end) ELSE coalesce(p.reason_end, m.reason_end) END,
            shuffle = CASE WHEN $1 >= p.source_as_of
                THEN coalesce(m.shuffle, p.shuffle) ELSE coalesce(p.shuffle, m.shuffle) END,
            skipped = CASE WHEN $1 >= p.source_as_of
                THEN coalesce(m.skipped, p.skipped) ELSE coalesce(p.skipped, m.skipped) END,
            offline = CASE WHEN $1 >= p.source_as_of
                THEN coalesce(m.offline, p.offline) ELSE coalesce(p.offline, m.offline) END,
            offline_ts = CASE WHEN $1 >= p.source_as_of
                THEN coalesce(m.offline_ts, p.offline_ts) ELSE coalesce(p.offline_ts, m.offline_ts) END,
            incognito = CASE WHEN $1 >= p.source_as_of
                THEN coalesce(m.incognito, p.incognito) ELSE coalesce(p.incognito, m.incognito) END,
            source_as_of = greatest(p.source_as_of, $1),
            last_file_id = $2
         FROM UNNEST($3::bigint[], $4::timestamptz[], $5::text[], $6::text[], $7::text[],
                     $8::text[], $9::bool[], $10::bool[], $11::bool[], $12::timestamptz[],
                     $13::bool[])
            AS m(pid, ts, platform, conn_country, reason_start, reason_end, shuffle, skipped,
                 offline, offline_ts, incognito)
         WHERE p.id = m.pid",
        as_of,
        file_id,
        &pid,
        &ts,
        &platform as &[Option<String>],
        &conn_country as &[Option<String>],
        &reason_start as &[Option<String>],
        &reason_end as &[Option<String>],
        &shuffle as &[Option<bool>],
        &skipped as &[Option<bool>],
        &offline as &[Option<bool>],
        &offline_ts as &[Option<DateTime<Utc>>],
        &incognito as &[Option<bool>],
    )
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}

async fn insert(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
    file_id: i64,
    as_of: DateTime<Utc>,
    rows: &[&ExtendedPlay],
) -> Result<u64, sqlx::Error> {
    if rows.is_empty() {
        return Ok(0);
    }

    macro_rules! column {
        ($field:ident) => {
            rows.iter().map(|p| p.$field.clone()).collect::<Vec<_>>()
        };
    }

    let ts = column!(ts);
    let uri = column!(uri);
    let ms_played = column!(ms_played);
    let platform = column!(platform);
    let conn_country = column!(conn_country);
    let reason_start = column!(reason_start);
    let reason_end = column!(reason_end);
    let shuffle = column!(shuffle);
    let skipped = column!(skipped);
    let offline = column!(offline);
    let offline_ts = column!(offline_ts);
    let incognito = column!(incognito);
    let is_video = column!(is_video);
    let name = column!(name);
    let artist = column!(artist);
    let album = column!(album);

    let result = sqlx::query!(
        "INSERT INTO plays (account_id, ts, item_uri, ms_played, platform, conn_country,
                            reason_start, reason_end, shuffle, skipped, offline, offline_ts,
                            incognito, is_video, raw_name, raw_artist, raw_album,
                            source_as_of, first_file_id, last_file_id)
         SELECT $1, u.ts, u.uri, u.ms_played, u.platform, u.conn_country, u.reason_start,
                u.reason_end, u.shuffle, u.skipped, u.offline, u.offline_ts, u.incognito,
                u.is_video,
                CASE WHEN u.uri IS NULL THEN u.name END,
                CASE WHEN u.uri IS NULL THEN u.artist END,
                CASE WHEN u.uri IS NULL THEN u.album END,
                $2, $3, $3
         FROM UNNEST($4::timestamptz[], $5::text[], $6::int[], $7::text[], $8::text[],
                     $9::text[], $10::text[], $11::bool[], $12::bool[], $13::bool[],
                     $14::timestamptz[], $15::bool[], $16::bool[], $17::text[], $18::text[],
                     $19::text[])
            AS u(ts, uri, ms_played, platform, conn_country, reason_start, reason_end, shuffle,
                 skipped, offline, offline_ts, incognito, is_video, name, artist, album)",
        account_id,
        as_of,
        file_id,
        &ts,
        &uri as &[Option<String>],
        &ms_played,
        &platform as &[Option<String>],
        &conn_country as &[Option<String>],
        &reason_start as &[Option<String>],
        &reason_end as &[Option<String>],
        &shuffle as &[Option<bool>],
        &skipped as &[Option<bool>],
        &offline as &[Option<bool>],
        &offline_ts as &[Option<DateTime<Utc>>],
        &incognito as &[Option<bool>],
        &is_video,
        &name as &[Option<String>],
        &artist as &[Option<String>],
        &album as &[Option<String>],
    )
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}
