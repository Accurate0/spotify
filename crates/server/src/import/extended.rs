//! Merges extended streaming history into `plays`.
//!
//! The same play appears in every export that covers its time range, but Spotify changed how
//! it records `ts` between exports, so a play's timestamp can differ by a few seconds (mostly
//! one). Rows are matched to existing plays on account, item, ms_played and platform with `ts`
//! within ±3s.
//! Reasons only have to agree when both sides have one, since older exports have null
//! `reason_end` where newer ones do not. On a match, the newer export's non-null values win.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use spotify_export::ExtendedPlay;
// Dynamic SQL here only interpolates the constants above, never input.
use sqlx::{AssertSqlSafe, Postgres, Transaction};

use super::RowCounts;

/// Columns copied from `staged` onto a matched play. The newer export's value wins unless it
/// is null.
const MERGED_COLUMNS: &[&str] = &[
    "platform",
    "conn_country",
    "reason_start",
    "reason_end",
    "shuffle",
    "skipped",
    "offline",
    "offline_ts",
    "incognito",
];

const MATCH_CONDITIONS: &str = "
    p.account_id = $1
    AND p.ms_played = s.ms_played
    AND p.ts BETWEEN s.ts - interval '3 seconds' AND s.ts + interval '3 seconds'
    AND p.platform IS NOT DISTINCT FROM s.platform
    AND (p.reason_start IS NULL OR s.reason_start IS NULL OR p.reason_start = s.reason_start)
    AND (p.reason_end IS NULL OR s.reason_end IS NULL OR p.reason_end = s.reason_end)";

pub async fn merge(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
    file_id: i64,
    as_of: DateTime<Utc>,
    plays: &[ExtendedPlay],
) -> Result<RowCounts, sqlx::Error> {
    stage(tx, plays).await?;
    upsert_items(tx, as_of).await?;

    let matches = match_existing(tx, account_id).await?;
    let (sids, pids): (Vec<i64>, Vec<i64>) = matches.into_iter().unzip();

    let merged = update_matched(tx, file_id, as_of, &sids, &pids).await?;
    let inserted = insert_unmatched(tx, account_id, file_id, as_of, &sids).await?;

    Ok(RowCounts {
        total: plays.len() as i32,
        inserted: inserted as i32,
        merged: merged as i32,
    })
}

/// Loads the file into a temp table, collapsing rows that are exact duplicates within the file
/// (old exports repeat some 0ms plays verbatim).
async fn stage(
    tx: &mut Transaction<'_, Postgres>,
    plays: &[ExtendedPlay],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TEMP TABLE staged (
            sid BIGINT GENERATED ALWAYS AS IDENTITY,
            ts TIMESTAMPTZ NOT NULL,
            uri TEXT,
            kind TEXT,
            name TEXT,
            artist TEXT,
            album TEXT,
            ms_played INT NOT NULL,
            platform TEXT,
            conn_country TEXT,
            reason_start TEXT,
            reason_end TEXT,
            shuffle BOOLEAN,
            skipped BOOLEAN,
            offline BOOLEAN,
            offline_ts TIMESTAMPTZ,
            incognito BOOLEAN,
            is_video BOOLEAN NOT NULL
        ) ON COMMIT DROP",
    )
    .execute(&mut **tx)
    .await?;

    macro_rules! column {
        ($field:ident) => {
            plays.iter().map(|p| p.$field.clone()).collect::<Vec<_>>()
        };
    }

    sqlx::query(
        "INSERT INTO staged (ts, uri, kind, name, artist, album, ms_played, platform, conn_country,
                             reason_start, reason_end, shuffle, skipped, offline, offline_ts,
                             incognito, is_video)
         SELECT DISTINCT * FROM UNNEST(
            $1::timestamptz[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[],
            $7::int[], $8::text[], $9::text[], $10::text[], $11::text[], $12::bool[],
            $13::bool[], $14::bool[], $15::timestamptz[], $16::bool[], $17::bool[]
         )",
    )
    .bind(column!(ts))
    .bind(column!(uri))
    .bind(
        plays
            .iter()
            .map(|p| p.kind.map(|k| k.as_str()))
            .collect::<Vec<_>>(),
    )
    .bind(column!(name))
    .bind(column!(artist))
    .bind(column!(album))
    .bind(column!(ms_played))
    .bind(column!(platform))
    .bind(column!(conn_country))
    .bind(column!(reason_start))
    .bind(column!(reason_end))
    .bind(column!(shuffle))
    .bind(column!(skipped))
    .bind(column!(offline))
    .bind(column!(offline_ts))
    .bind(column!(incognito))
    .bind(column!(is_video))
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn upsert_items(
    tx: &mut Transaction<'_, Postgres>,
    as_of: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO items (uri, kind, name, artist, album, as_of)
         SELECT DISTINCT ON (uri) uri, kind, name, artist, album, $1
         FROM staged
         WHERE uri IS NOT NULL
         ORDER BY uri, ts DESC
         ON CONFLICT (uri) DO UPDATE SET
            name = coalesce(EXCLUDED.name, items.name),
            artist = coalesce(EXCLUDED.artist, items.artist),
            album = coalesce(EXCLUDED.album, items.album),
            as_of = EXCLUDED.as_of
         WHERE items.as_of <= EXCLUDED.as_of",
    )
    .bind(as_of)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Pairs staged rows with existing plays one-to-one, closest timestamps first, so two staged
/// rows can never merge into the same play.
async fn match_existing(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
) -> Result<Vec<(i64, i64)>, sqlx::Error> {
    // Two arms so the URI arm can use plays_dedupe_idx; plays without a URI match on raw names.
    let query = format!(
        "SELECT s.sid, p.id, abs(extract(epoch FROM s.ts - p.ts))::float8 AS diff
         FROM staged s JOIN plays p ON p.item_uri = s.uri AND {MATCH_CONDITIONS}
         UNION ALL
         SELECT s.sid, p.id, abs(extract(epoch FROM s.ts - p.ts))::float8
         FROM staged s JOIN plays p ON p.item_uri IS NULL AND s.uri IS NULL
            AND p.raw_name IS NOT DISTINCT FROM s.name
            AND p.raw_artist IS NOT DISTINCT FROM s.artist
            AND {MATCH_CONDITIONS}"
    );

    let mut candidates: Vec<(i64, i64, f64)> = sqlx::query_as(AssertSqlSafe(query))
        .bind(account_id)
        .fetch_all(&mut **tx)
        .await?;

    candidates.sort_by(|a, b| a.2.total_cmp(&b.2).then(a.0.cmp(&b.0)));

    let mut used_staged = HashSet::new();
    let mut used_plays = HashSet::new();

    Ok(candidates
        .into_iter()
        .filter(|&(sid, pid, _)| {
            !used_staged.contains(&sid)
                && !used_plays.contains(&pid)
                && used_staged.insert(sid)
                && used_plays.insert(pid)
        })
        .map(|(sid, pid, _)| (sid, pid))
        .collect())
}

async fn update_matched(
    tx: &mut Transaction<'_, Postgres>,
    file_id: i64,
    as_of: DateTime<Utc>,
    sids: &[i64],
    pids: &[i64],
) -> Result<u64, sqlx::Error> {
    if sids.is_empty() {
        return Ok(0);
    }

    // $1 is newer-or-equal than the row's current source: prefer staged values, else keep.
    let sets = MERGED_COLUMNS
        .iter()
        .map(|c| {
            format!(
                "{c} = CASE WHEN $1 >= p.source_as_of THEN coalesce(s.{c}, p.{c}) ELSE coalesce(p.{c}, s.{c}) END"
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");

    let query = format!(
        "UPDATE plays p SET
            ts = CASE WHEN $1 >= p.source_as_of THEN s.ts ELSE p.ts END,
            {sets},
            source_as_of = greatest(p.source_as_of, $1),
            last_file_id = $2
         FROM UNNEST($3::bigint[], $4::bigint[]) AS m(sid, pid)
         JOIN staged s ON s.sid = m.sid
         WHERE p.id = m.pid"
    );

    let result = sqlx::query(AssertSqlSafe(query))
        .bind(as_of)
        .bind(file_id)
        .bind(sids)
        .bind(pids)
        .execute(&mut **tx)
        .await?;

    Ok(result.rows_affected())
}

async fn insert_unmatched(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
    file_id: i64,
    as_of: DateTime<Utc>,
    matched_sids: &[i64],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO plays (account_id, ts, item_uri, ms_played, platform, conn_country,
                            reason_start, reason_end, shuffle, skipped, offline, offline_ts,
                            incognito, is_video, raw_name, raw_artist, raw_album,
                            source_as_of, first_file_id, last_file_id)
         SELECT $1, ts, uri, ms_played, platform, conn_country, reason_start, reason_end,
                shuffle, skipped, offline, offline_ts, incognito, is_video,
                CASE WHEN uri IS NULL THEN name END,
                CASE WHEN uri IS NULL THEN artist END,
                CASE WHEN uri IS NULL THEN album END,
                $2, $3, $3
         FROM staged
         WHERE NOT (sid = ANY($4))",
    )
    .bind(account_id)
    .bind(as_of)
    .bind(file_id)
    .bind(matched_sids)
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}
