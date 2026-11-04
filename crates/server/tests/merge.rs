//! Merge behaviour against a real database. Requires DATABASE_URL (docker compose up db);
//! sqlx::test creates and migrates a fresh database per test.

use chrono::{DateTime, Utc};
use spotify_export::FileKind;
use spotify_server::import::{FileUpload, ImportOutcome, import_file, sha256_hex};
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

struct Fixture {
    pool: PgPool,
    account_id: i64,
}

impl Fixture {
    async fn new(pool: PgPool) -> Self {
        let account_id = sqlx::query_scalar(
            "INSERT INTO accounts (slug, display_name) VALUES ('me', 'me') RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        Self { pool, account_id }
    }

    async fn import(&self, as_of: &str, rows: &[serde_json::Value]) -> ImportOutcome {
        let as_of: DateTime<Utc> = as_of.parse().unwrap();
        let export_id: i64 = sqlx::query_scalar(
            "INSERT INTO exports (account_id, label, as_of) VALUES ($1, 'test', $2) RETURNING id",
        )
        .bind(self.account_id)
        .bind(as_of)
        .fetch_one(&self.pool)
        .await
        .unwrap();

        let bytes = serde_json::to_vec(rows).unwrap();
        let sha256 = sha256_hex(&bytes);
        import_file(
            &self.pool,
            FileUpload {
                account_id: self.account_id,
                export_id,
                as_of,
                name: "Streaming_History_Audio_2023_0.json",
                kind: FileKind::ExtendedAudio,
                sha256: &sha256,
                bytes: &bytes,
            },
        )
        .await
        .unwrap()
    }

    async fn plays(&self) -> Vec<(DateTime<Utc>, Option<String>, i32, Option<String>)> {
        sqlx::query_as(
            "SELECT ts, item_uri, ms_played, reason_end FROM plays ORDER BY ts, ms_played",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    async fn counts(&self, outcome: ImportOutcome) -> (i32, i32, i32) {
        let ImportOutcome::Imported { file_id } = outcome else {
            panic!("expected Imported, got {outcome:?}");
        };
        sqlx::query_as(
            "SELECT rows_total, rows_inserted, rows_merged FROM export_files WHERE id = $1",
        )
        .bind(file_id)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }
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

    let plays = f.plays().await;
    assert_eq!(plays.len(), 1);
    assert_eq!(
        plays[0].0.to_rfc3339(),
        "2023-05-01T10:00:00+00:00",
        "newer ts wins"
    );
    assert_eq!(plays[0].3.as_deref(), Some("trackdone"));

    let name: String = sqlx::query_scalar("SELECT name FROM items WHERE uri = 'spotify:track:a'")
        .fetch_one(&f.pool)
        .await
        .unwrap();
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

    let plays = f.plays().await;
    assert_eq!(plays.len(), 1);
    assert_eq!(plays[0].0.to_rfc3339(), "2023-05-01T10:00:00+00:00");
    assert_eq!(plays[0].3.as_deref(), Some("trackdone"));

    let name: String = sqlx::query_scalar("SELECT name FROM items WHERE uri = 'spotify:track:a'")
        .fetch_one(&f.pool)
        .await
        .unwrap();
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
    assert_eq!(f.plays().await.len(), 3);
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
    assert_eq!(f.plays().await.len(), 2);
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
    assert_eq!(f.plays().await.len(), 1);
}
