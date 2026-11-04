//! Extended streaming history merge behaviour against a real database.

mod common;

use chrono::{DateTime, Utc};
use common::Fixture;
use spotify_server::import::ImportOutcome;
use sqlx::PgPool;

fn play(ts: &str, uri: &str, ms: i64, name: &str, reason_end: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "ts": ts,
        "platform": "android",
        "ms_played": ms,
        "conn_country": "AU",
        "ip_addr": "10.0.0.1",
        "master_metadata_track_name": name,
        "master_metadata_album_artist_name": "Artist",
        "master_metadata_album_album_name": "Album",
        "spotify_track_uri": uri,
        "reason_start": "clickrow",
        "reason_end": reason_end,
        "shuffle": false,
        "skipped": null,
        "offline": false,
        "offline_timestamp": 0,
        "incognito_mode": false,
    })
}

struct Play {
    ts: DateTime<Utc>,
    reason_end: Option<String>,
}

async fn plays(pool: &PgPool) -> Vec<Play> {
    sqlx::query_as!(
        Play,
        "SELECT ts, reason_end FROM plays ORDER BY ts, ms_played"
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn item_name(pool: &PgPool, uri: &str) -> Option<String> {
    sqlx::query_scalar!("SELECT name FROM items WHERE uri = $1", uri)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn merges_plays_seconds_apart(pool: PgPool) {
    let f = Fixture::new(pool).await;

    // Older export (Jan 2024): second rounded up, reason_end missing, old name.
    let older = f
        .import(
            "2024-01-18T00:00:00Z",
            &[play(
                "2023-05-01T10:00:01Z",
                "spotify:track:a",
                1000,
                "Timber (feat. Ke$ha)",
                None,
            )],
        )
        .await;
    assert_eq!(f.counts(older).await, (1, 1, 0));

    // Newer export (Dec 2024): same play one second earlier, with reason_end and a new name.
    let newer = f
        .import(
            "2024-12-17T00:00:00Z",
            &[play(
                "2023-05-01T10:00:00Z",
                "spotify:track:a",
                1000,
                "Timber",
                Some("trackdone"),
            )],
        )
        .await;
    assert_eq!(f.counts(newer).await, (1, 0, 1));

    let plays = plays(&f.pool).await;
    assert_eq!(plays.len(), 1);
    assert_eq!(
        plays[0].ts.to_rfc3339(),
        "2023-05-01T10:00:00+00:00",
        "newer ts wins"
    );
    assert_eq!(plays[0].reason_end.as_deref(), Some("trackdone"));

    let name = item_name(&f.pool, "spotify:track:a").await.unwrap();
    assert_eq!(name, "Timber");
}

#[sqlx::test(migrations = "./migrations")]
async fn matches_within_three_seconds_only(pool: PgPool) {
    let f = Fixture::new(pool).await;

    f.import(
        "2024-01-18T00:00:00Z",
        &[
            play(
                "2019-04-07T14:42:00Z",
                "spotify:track:a",
                1000,
                "A",
                Some("trackdone"),
            ),
            play(
                "2019-04-08T14:42:00Z",
                "spotify:track:a",
                1000,
                "A",
                Some("trackdone"),
            ),
        ],
    )
    .await;
    let outcome = f
        .import(
            "2024-12-17T00:00:00Z",
            &[
                play(
                    "2019-04-07T14:41:57Z",
                    "spotify:track:a",
                    1000,
                    "A",
                    Some("trackdone"),
                ),
                play(
                    "2019-04-08T14:41:56Z",
                    "spotify:track:a",
                    1000,
                    "A",
                    Some("trackdone"),
                ),
            ],
        )
        .await;
    assert_eq!(f.counts(outcome).await, (2, 1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn older_export_does_not_overwrite_newer(pool: PgPool) {
    let f = Fixture::new(pool).await;

    f.import(
        "2024-12-17T00:00:00Z",
        &[play(
            "2023-05-01T10:00:00Z",
            "spotify:track:a",
            1000,
            "Timber",
            Some("trackdone"),
        )],
    )
    .await;
    let older = f
        .import(
            "2024-01-18T00:00:00Z",
            &[play(
                "2023-05-01T10:00:01Z",
                "spotify:track:a",
                1000,
                "Timber (feat. Ke$ha)",
                None,
            )],
        )
        .await;
    assert_eq!(f.counts(older).await, (1, 0, 1));

    let plays = plays(&f.pool).await;
    assert_eq!(plays.len(), 1);
    assert_eq!(plays[0].ts.to_rfc3339(), "2023-05-01T10:00:00+00:00");
    assert_eq!(plays[0].reason_end.as_deref(), Some("trackdone"));

    let name = item_name(&f.pool, "spotify:track:a").await.unwrap();
    assert_eq!(name, "Timber", "older export keeps newer item name");
}

#[sqlx::test(migrations = "./migrations")]
async fn keeps_distinct_plays_and_collapses_exact_duplicates(pool: PgPool) {
    let f = Fixture::new(pool).await;
    let ts = "2015-01-01T00:00:00Z";

    let outcome = f
        .import(
            "2024-12-17T00:00:00Z",
            &[
                // Exact duplicate rows within a file collapse into one play.
                play(ts, "spotify:track:a", 0, "A", Some("endplay")),
                play(ts, "spotify:track:a", 0, "A", Some("endplay")),
                // Same ts and uri, different ms_played: a quick restart, kept separately.
                play(ts, "spotify:track:a", 2500, "A", Some("endplay")),
                // Same track replayed two seconds later is a separate play.
                play(
                    "2015-01-01T00:00:02Z",
                    "spotify:track:a",
                    0,
                    "A",
                    Some("endplay"),
                ),
            ],
        )
        .await;
    assert_eq!(f.counts(outcome).await, (4, 3, 0));
    assert_eq!(plays(&f.pool).await.len(), 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn two_rows_never_merge_into_one_play(pool: PgPool) {
    let f = Fixture::new(pool).await;

    f.import(
        "2024-01-18T00:00:00Z",
        &[play(
            "2023-05-01T10:00:01Z",
            "spotify:track:a",
            1000,
            "A",
            Some("trackdone"),
        )],
    )
    .await;

    // Both rows are within 1s of the existing play; only the closest one may merge.
    let outcome = f
        .import(
            "2024-12-17T00:00:00Z",
            &[
                play(
                    "2023-05-01T10:00:00Z",
                    "spotify:track:a",
                    1000,
                    "A",
                    Some("trackdone"),
                ),
                play(
                    "2023-05-01T10:00:01Z",
                    "spotify:track:a",
                    1000,
                    "A",
                    Some("trackdone"),
                ),
            ],
        )
        .await;
    assert_eq!(f.counts(outcome).await, (2, 1, 1));
    assert_eq!(plays(&f.pool).await.len(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn same_file_twice_is_a_duplicate(pool: PgPool) {
    let f = Fixture::new(pool).await;
    let rows = [play(
        "2023-05-01T10:00:00Z",
        "spotify:track:a",
        1000,
        "A",
        None,
    )];

    assert!(matches!(
        f.import("2024-01-18T00:00:00Z", &rows).await,
        ImportOutcome::Imported { .. }
    ));
    assert!(matches!(
        f.import("2024-01-18T00:00:00Z", &rows).await,
        ImportOutcome::Duplicate { .. }
    ));
    assert_eq!(plays(&f.pool).await.len(), 1);
}
