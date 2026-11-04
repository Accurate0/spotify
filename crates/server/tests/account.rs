//! Account data imports: basic history, searches, snapshots and username.

mod common;

use common::Fixture;
use serde_json::json;
use spotify_server::import::ImportOutcome;
use sqlx::PgPool;

fn basic(end_time: &str, track: &str, ms: i64) -> serde_json::Value {
    json!({"endTime": end_time, "artistName": "Artist", "trackName": track, "msPlayed": ms})
}

fn bytes(value: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&value).unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn basic_history_dedupes_across_exports(pool: PgPool) {
    let f = Fixture::new(pool).await;

    let first = f
        .import_file(
            "2021-08-13T00:00:00Z",
            "StreamingHistory0.json",
            &bytes(json!([
                basic("2021-04-05 12:44", "A", 1000),
                basic("2021-08-13 18:36", "B", 2000)
            ])),
        )
        .await;
    assert_eq!(f.counts(first).await, (2, 2, 0));

    // The next account export overlaps: one identical row, one new.
    let second = f
        .import_file(
            "2022-04-12T00:00:00Z",
            "StreamingHistory0.json",
            &bytes(json!([
                basic("2021-08-13 18:36", "B", 2000),
                basic("2022-04-12 17:01", "C", 3000)
            ])),
        )
        .await;
    assert_eq!(f.counts(second).await, (2, 1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn all_plays_fills_gaps_outside_extended_coverage(pool: PgPool) {
    let f = Fixture::new(pool).await;

    let extended = |ts: &str| {
        json!({"ts": ts, "platform": "android", "ms_played": 1000,
               "master_metadata_track_name": "X", "master_metadata_album_artist_name": "Artist",
               "master_metadata_album_album_name": "Album", "spotify_track_uri": "spotify:track:x",
               "reason_start": "clickrow", "reason_end": "trackdone"})
    };
    f.import(
        "2024-01-01T00:00:00Z",
        &[
            extended("2021-01-01T00:00:00Z"),
            extended("2021-12-31T00:00:00Z"),
        ],
    )
    .await;

    f.import_file(
        "2022-04-12T00:00:00Z",
        "StreamingHistory0.json",
        &bytes(json!([
            basic("2020-06-01 10:00", "before", 1000),
            basic("2021-06-01 10:00", "covered", 1000),
            basic("2022-02-01 10:00", "after", 1000),
        ])),
    )
    .await;

    let names = sqlx::query_scalar!(
        r#"SELECT name AS "name!" FROM all_plays WHERE NOT is_extended ORDER BY ts"#
    )
    .fetch_all(&f.pool)
    .await
    .unwrap();
    assert_eq!(names, ["before", "after"]);

    let total = sqlx::query_scalar!(r#"SELECT count(*) AS "count!" FROM all_plays"#)
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(total, 4);
}

#[sqlx::test(migrations = "./migrations")]
async fn searches_dedupe_and_keep_interactions(pool: PgPool) {
    let f = Fixture::new(pool).await;
    let search = json!([{"platform": "ANDROID", "searchTime": "2023-10-09T12:37:14.004Z[UTC]",
                         "searchQuery": "beach", "searchInteractionURIs": ["spotify:artist:a", "spotify:track:b"]}]);

    let first = f
        .import_file(
            "2024-01-03T00:00:00Z",
            "SearchQueries.json",
            &bytes(search.clone()),
        )
        .await;
    assert_eq!(f.counts(first).await, (1, 1, 0));

    // Same search in a later export, alongside a new one (different bytes, so not a duplicate file).
    let mut later = search.as_array().unwrap().clone();
    later.push(
        json!({"platform": "", "searchTime": "2023-11-01T00:00:00.000Z[UTC]",
                      "searchQuery": "bunny", "searchInteractionURIs": []}),
    );
    let second = f
        .import_file(
            "2024-06-01T00:00:00Z",
            "SearchQueries.json",
            &bytes(json!(later)),
        )
        .await;
    assert_eq!(f.counts(second).await, (2, 1, 1));

    let uris = sqlx::query_scalar!("SELECT interaction_uris FROM searches WHERE query = 'beach'")
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(uris, ["spotify:artist:a", "spotify:track:b"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn current_snapshot_is_newest_export(pool: PgPool) {
    let f = Fixture::new(pool).await;
    let library = |track: &str| {
        bytes(
            json!({"tracks": [{"artist": "A", "album": "B", "track": track, "uri": format!("spotify:track:{track}")}],
                     "albums": [], "artists": []}),
        )
    };
    let playlist = |name: &str| {
        bytes(
            json!({"playlists": [{"name": name, "lastModifiedDate": "2023-12-01", "description": null,
                     "numberOfFollowers": 0, "items": [{"track": {"trackName": "T", "artistName": "A",
                     "albumName": "B", "trackUri": "spotify:track:t"}, "episode": null, "localTrack": null}]}]}),
        )
    };

    // Imported newest first: "current" follows as_of, not import order.
    let (newer, newer_as_of) = f.export("2024-01-03T00:00:00Z").await;
    f.import_into(newer, newer_as_of, "YourLibrary.json", &library("new"))
        .await;
    f.import_into(newer, newer_as_of, "Playlist1.json", &playlist("one"))
        .await;
    f.import_into(newer, newer_as_of, "Playlist2.json", &playlist("two"))
        .await;

    let (older, older_as_of) = f.export("2021-08-16T00:00:00Z").await;
    f.import_into(older, older_as_of, "YourLibrary.json", &library("old"))
        .await;
    f.import_into(older, older_as_of, "Playlist1.json", &playlist("ancient"))
        .await;

    let library_names = sqlx::query_scalar!(
        "SELECT l.name FROM library_items l JOIN current_snapshot_files c ON c.id = l.file_id"
    )
    .fetch_all(&f.pool)
    .await
    .unwrap();
    assert_eq!(library_names, ["new"]);

    let playlist_names = sqlx::query_scalar!(
        "SELECT p.name FROM playlists p JOIN current_snapshot_files c ON c.id = p.file_id
         ORDER BY p.name"
    )
    .fetch_all(&f.pool)
    .await
    .unwrap();
    assert_eq!(
        playlist_names,
        ["one", "two"],
        "playlists split across files in one export"
    );

    let all_snapshots = sqlx::query_scalar!(r#"SELECT count(*) AS "count!" FROM library_items"#)
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(all_snapshots, 2, "older snapshots are kept as history");
}

#[sqlx::test(migrations = "./migrations")]
async fn userdata_sets_username_and_private_files_are_skipped(pool: PgPool) {
    let f = Fixture::new(pool).await;

    f.import_file(
        "2024-01-03T00:00:00Z",
        "Userdata.json",
        &bytes(json!({"username": "spotifyuser", "email": "x@example.com", "postalCode": "6000"})),
    )
    .await;
    let username = sqlx::query_scalar!(
        "SELECT spotify_username FROM accounts WHERE id = $1",
        f.account_id
    )
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(username.as_deref(), Some("spotifyuser"));

    let private = f
        .import_file(
            "2024-01-03T00:00:00Z",
            "DuoNewFamily.json",
            &bytes(json!({"address": "1 Example St"})),
        )
        .await;
    assert_eq!(private, ImportOutcome::Unsupported);
    let files = sqlx::query_scalar!(r#"SELECT count(*) AS "count!" FROM export_files"#)
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(files, 1, "private files are not recorded");
}
