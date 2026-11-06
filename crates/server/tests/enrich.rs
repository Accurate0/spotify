//! Enrichment against a mock Spotify API: queue order, upserts, artwork, not-found handling,
//! rate limits and the daily budget.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use common::Fixture;
use serde_json::{Value, json};
use spotify_server::enrich::artwork::ArtworkStore;
use spotify_server::enrich::spotify::{SpotifyClient, SpotifyConfig};
use spotify_server::enrich::{EnrichConfig, run};
use sqlx::PgPool;
use tower::ServiceExt;

#[derive(Clone, Default)]
struct Mock {
    base: Arc<Mutex<String>>,
    /// Paths requested, in order (token requests included).
    log: Arc<Mutex<Vec<String>>>,
}

impl Mock {
    fn record(&self, path: String) {
        self.log.lock().unwrap().push(path);
    }

    fn image(&self, name: &str) -> Value {
        json!([{"url": format!("{}/img/{name}", self.base.lock().unwrap()), "width": 300}])
    }

    fn api_calls(&self) -> Vec<String> {
        self.log
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.starts_with("/v1/"))
            .cloned()
            .collect()
    }
}

async fn token(State(mock): State<Mock>) -> Response {
    mock.record("/api/token".into());
    axum::Json(json!({"access_token": "token", "token_type": "Bearer", "expires_in": 3600}))
        .into_response()
}

async fn track(State(mock): State<Mock>, Path(id): Path<String>) -> Response {
    mock.record(format!("/v1/tracks/{id}"));
    let (album, artists) = match id.as_str() {
        "gone" => return StatusCode::NOT_FOUND.into_response(),
        "limited" => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, "120")],
            )
                .into_response();
        }
        "t1" => ("al1", json!([{"id": "ar1", "name": "Alpha"}])),
        _ => (
            "al2",
            json!([{"id": "ar1", "name": "Alpha"}, {"id": "ar2", "name": "Guest"}]),
        ),
    };
    axum::Json(json!({
        "id": id, "name": id, "duration_ms": 200_000, "explicit": id == "t2",
        "album": {"id": album, "name": format!("Album {album}"), "album_type": "album",
                  "release_date": if album == "al1" { "1999-03-14" } else { "2021" },
                  "images": mock.image(album)},
        "artists": artists,
    }))
    .into_response()
}

async fn artist(State(mock): State<Mock>, Path(id): Path<String>) -> Response {
    mock.record(format!("/v1/artists/{id}"));
    axum::Json(json!({
        "id": id, "name": if id == "ar1" { "Alpha" } else { "Guest" },
        "genres": if id == "ar1" { json!(["indie pop", "dream pop"]) } else { json!([]) },
        "images": mock.image(&id),
    }))
    .into_response()
}

async fn image(Path(name): Path<String>) -> Response {
    (
        [(header::CONTENT_TYPE, "image/jpeg")],
        format!("jpeg:{name}"),
    )
        .into_response()
}

async fn spawn_mock() -> Mock {
    let mock = Mock::default();
    let app = Router::new()
        .route("/api/token", post(token))
        .route("/v1/tracks/{id}", get(track))
        .route("/v1/artists/{id}", get(artist))
        .route("/img/{name}", get(image))
        .with_state(mock.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    *mock.base.lock().unwrap() = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    mock
}

fn client(mock: &Mock) -> SpotifyClient {
    let base = mock.base.lock().unwrap().clone();
    SpotifyClient::new(SpotifyConfig {
        client_id: "id".into(),
        client_secret: "secret".into(),
        api_url: base.clone(),
        accounts_url: base,
    })
}

fn config(max_requests_per_run: i64, daily_requests: i64) -> EnrichConfig {
    EnrichConfig {
        request_interval: Duration::ZERO,
        max_requests_per_run,
        daily_requests,
        images_per_run: 100,
        image_interval: Duration::ZERO,
    }
}

fn play(ts: &str, id: &str, ms: i64) -> Value {
    json!({"ts": ts, "platform": "android", "ms_played": ms,
           "master_metadata_track_name": id, "master_metadata_album_artist_name": "Alpha",
           "master_metadata_album_album_name": "Album", "spotify_track_uri": format!("spotify:track:{id}"),
           "reason_start": "clickrow", "reason_end": "trackdone"})
}

/// t1 is the most listened, then t2.
async fn setup(pool: PgPool, extra: &[(&str, i64)]) -> Fixture {
    let f = Fixture::new(pool).await;
    let mut rows = vec![
        play("2024-01-01T00:00:00Z", "t1", 300_000),
        play("2024-01-02T00:00:00Z", "t2", 100_000),
    ];
    for (i, (id, ms)) in extra.iter().enumerate() {
        rows.push(play(&format!("2024-02-0{}T00:00:00Z", i + 1), id, *ms));
    }
    f.import("2024-12-17T00:00:00Z", &rows).await;
    f
}

#[sqlx::test(migrations = "./migrations")]
async fn fetches_tracks_then_artists_and_stores_artwork(pool: PgPool) {
    let f = setup(pool, &[]).await;
    let mock = spawn_mock().await;
    let store = ArtworkStore::memory();

    let report = run(&f.pool, &client(&mock), Some(&store), &config(50, 5000))
        .await
        .unwrap();
    assert_eq!(
        (report.items_done, report.artists_done, report.images),
        (2, 2, 4)
    );
    // One token request plus four lookups.
    assert_eq!(report.api_requests, 5);
    assert_eq!(
        mock.api_calls(),
        [
            "/v1/tracks/t1",
            "/v1/tracks/t2",
            "/v1/artists/ar1",
            "/v1/artists/ar2"
        ],
        "most listened first; ar1 (on both tracks) outranks ar2"
    );

    let meta = sqlx::query!(
        "SELECT m.duration_ms, m.explicit, a.release_year, a.image_key
         FROM item_meta m JOIN spotify_albums a ON a.id = m.album_id
         WHERE m.uri = 'spotify:track:t1'"
    )
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(meta.duration_ms, Some(200_000));
    assert_eq!(meta.explicit, Some(false));
    assert_eq!(meta.release_year, Some(1999));
    assert_eq!(meta.image_key.as_deref(), Some("albums/al1.jpg"));

    let credits = sqlx::query_scalar!(
        "SELECT artist_id FROM item_artists WHERE item_uri = 'spotify:track:t2' ORDER BY position"
    )
    .fetch_all(&f.pool)
    .await
    .unwrap();
    assert_eq!(credits, ["ar1", "ar2"]);

    let genres = sqlx::query_scalar!(
        "SELECT s.genres FROM artist_by_name b JOIN spotify_artists s ON s.id = b.artist_id
         WHERE b.name = 'Alpha'"
    )
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(genres, ["indie pop", "dream pop"]);

    assert_eq!(
        store.get("artists/ar1.jpg").await.unwrap().unwrap(),
        b"jpeg:ar1"
    );

    // Nothing left: a second run makes no API calls.
    let again = run(&f.pool, &client(&mock), Some(&store), &config(50, 5000))
        .await
        .unwrap();
    assert_eq!(again.api_requests, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn missing_tracks_are_marked_and_never_retried(pool: PgPool) {
    let f = setup(pool, &[("gone", 500_000)]).await;
    let mock = spawn_mock().await;

    let report = run(&f.pool, &client(&mock), None, &config(50, 5000))
        .await
        .unwrap();
    assert_eq!(report.not_found, 1);
    let status =
        sqlx::query_scalar!("SELECT status FROM item_meta WHERE uri = 'spotify:track:gone'")
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(status, "not_found");

    run(&f.pool, &client(&mock), None, &config(50, 5000))
        .await
        .unwrap();
    let gone_calls = mock
        .api_calls()
        .iter()
        .filter(|p| p.ends_with("/gone"))
        .count();
    assert_eq!(gone_calls, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn long_rate_limit_pauses_until_retry_after(pool: PgPool) {
    // The rate-limited track is the most listened, so it's fetched first.
    let f = setup(pool, &[("limited", 900_000)]).await;
    let mock = spawn_mock().await;

    let report = run(&f.pool, &client(&mock), None, &config(50, 5000))
        .await
        .unwrap();
    let until = report.rate_limited_until.expect("rate limited");
    assert!(until > chrono::Utc::now() + chrono::Duration::seconds(100));
    assert_eq!(report.items_done, 0, "the run stops at the 429");

    let calls_before = mock.api_calls().len();
    let next = run(&f.pool, &client(&mock), None, &config(50, 5000))
        .await
        .unwrap();
    assert!(next.skipped.unwrap().starts_with("rate limited until"));
    assert_eq!(
        mock.api_calls().len(),
        calls_before,
        "no calls while paused"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn daily_budget_caps_requests_across_runs(pool: PgPool) {
    let f = setup(pool, &[]).await;
    let mock = spawn_mock().await;

    // Token + t1 + t2 = 3 requests, then the budget is gone.
    let first = run(&f.pool, &client(&mock), None, &config(50, 3))
        .await
        .unwrap();
    assert_eq!((first.api_requests, first.items_done), (3, 2));

    let second = run(&f.pool, &client(&mock), None, &config(50, 3))
        .await
        .unwrap();
    assert_eq!(second.api_requests, 0);
    assert!(second.skipped.unwrap().starts_with("daily budget used"));
}

#[sqlx::test(migrations = "./migrations")]
async fn artwork_is_served_with_long_cache(pool: PgPool) {
    let store = Arc::new(ArtworkStore::memory());
    store
        .put("albums/al1.jpg", b"jpeg", "image/jpeg")
        .await
        .unwrap();
    let app = spotify_server::router(pool, None, Some(store), "https://spotify.test", None);

    let response = app
        .clone()
        .oneshot(Request::get("/art/albums/al1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );

    for path in [
        "/art/albums/missing",
        "/art/tracks/al1",
        "/art/albums/..%2Fsecret",
    ] {
        let response = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn unreachable_bucket_keeps_artwork_urls_for_a_later_run(pool: PgPool) {
    let f = setup(pool, &[]).await;
    let mock = spawn_mock().await;
    // Nothing listens on port 1: every upload fails to connect, as when a network policy
    // blocks the bucket.
    let unreachable = ArtworkStore::s3(
        "spotify-artwork",
        "http://127.0.0.1:1",
        "garage",
        Some("k"),
        Some("s"),
    )
    .unwrap();

    let report = run(
        &f.pool,
        &client(&mock),
        Some(&unreachable),
        &config(50, 5000),
    )
    .await
    .unwrap();
    assert_eq!((report.items_done, report.images), (2, 0));

    let pending = sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM spotify_albums
           WHERE image_source_url IS NOT NULL AND image_key IS NULL"#
    )
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(pending, 2, "both album urls survive the failed uploads");

    // Once the bucket works, the next run stores them.
    let store = ArtworkStore::memory();
    let later = run(&f.pool, &client(&mock), Some(&store), &config(50, 5000))
        .await
        .unwrap();
    assert_eq!(later.images, 4);
}
