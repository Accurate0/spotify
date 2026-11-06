//! Recently played tracks against a mock Spotify API: linking, polling with the `after` cursor,
//! refresh token rotation and revocation, and exports taking over the plays they cover.

mod common;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_graphql::{Request, Variables};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use chrono::{DateTime, Utc};
use common::Fixture;
use serde_json::{Value, json};
use spotify_server::enrich::spotify::{PlayHistory, SpotifyConfig};
use spotify_server::recent::{self, LinkError, PollReport};
use sqlx::PgPool;

#[derive(Clone, Default)]
struct Mock {
    base: Arc<Mutex<String>>,
    /// Recently played items, newest first.
    plays: Arc<Mutex<Vec<Value>>>,
    /// Scopes granted by the code exchange.
    scope: Arc<Mutex<String>>,
    /// Refresh tokens Spotify accepts, mapped to the one it rotates to.
    refresh: Arc<Mutex<HashMap<String, String>>>,
    /// `after` values requested from recently-played.
    afters: Arc<Mutex<Vec<i64>>>,
}

async fn token(State(mock): State<Mock>, Form(form): Form<HashMap<String, String>>) -> Response {
    match form["grant_type"].as_str() {
        "authorization_code" if form["code"] == "good" => Json(json!({
            "access_token": "access", "token_type": "Bearer", "expires_in": 3600,
            "refresh_token": "r1", "scope": mock.scope.lock().unwrap().clone(),
        }))
        .into_response(),
        "refresh_token" => match mock.refresh.lock().unwrap().get(&form["refresh_token"]) {
            Some(next) => Json(json!({
                "access_token": "access", "token_type": "Bearer", "expires_in": 3600,
                "refresh_token": next,
            }))
            .into_response(),
            None => (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({"error": "invalid_grant", "error_description": "Refresh token revoked"}),
                ),
            )
                .into_response(),
        },
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_grant"})),
        )
            .into_response(),
    }
}

async fn me() -> Response {
    Json(json!({"id": "spotify-user"})).into_response()
}

async fn recently_played(
    State(mock): State<Mock>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let after: i64 = query["after"].parse().unwrap();
    mock.afters.lock().unwrap().push(after);
    let items: Vec<Value> = mock
        .plays
        .lock()
        .unwrap()
        .iter()
        .filter(|p| {
            let played_at: DateTime<Utc> = p["played_at"].as_str().unwrap().parse().unwrap();
            played_at.timestamp_millis() > after
        })
        .take(50)
        .cloned()
        .collect();
    let cursor = items.first().map(|p| {
        let played_at: DateTime<Utc> = p["played_at"].as_str().unwrap().parse().unwrap();
        played_at.timestamp_millis().to_string()
    });
    Json(json!({"items": items, "cursors": {"after": cursor}, "limit": 50})).into_response()
}

async fn spawn_mock() -> Mock {
    let mock = Mock::default();
    *mock.scope.lock().unwrap() = "user-read-recently-played".into();
    let app = Router::new()
        .route("/api/token", post(token))
        .route("/v1/me", get(me))
        .route("/v1/me/player/recently-played", get(recently_played))
        .with_state(mock.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    *mock.base.lock().unwrap() = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    mock
}

fn config(mock: &Mock) -> SpotifyConfig {
    let base = mock.base.lock().unwrap().clone();
    SpotifyConfig {
        client_id: "id".into(),
        client_secret: "secret".into(),
        api_url: base.clone(),
        accounts_url: base,
    }
}

fn recent_play(played_at: &str, id: &str) -> Value {
    json!({
        "played_at": played_at,
        "context": {"uri": "spotify:playlist:p", "type": "playlist"},
        "track": {
            "uri": format!("spotify:track:{id}"),
            "name": format!("Track {id}"),
            "duration_ms": 200_000,
            "explicit": false,
            "album": {"id": "al1", "name": "Album", "album_type": "album",
                      "release_date": "2020-01-01", "images": []},
            "artists": [{"id": "ar1", "name": "Artist"}, {"id": "ar2", "name": "Guest"}],
        },
    })
}

fn export_play(ts: &str, id: &str, ms: i64) -> Value {
    json!({"ts": ts, "platform": "android", "ms_played": ms,
           "master_metadata_track_name": format!("Track {id}"),
           "master_metadata_album_artist_name": "Artist",
           "master_metadata_album_album_name": "Album",
           "spotify_track_uri": format!("spotify:track:{id}"),
           "reason_start": "clickrow", "reason_end": "trackdone"})
}

async fn linked(pool: PgPool) -> (Fixture, Mock) {
    let f = Fixture::new(pool).await;
    let mock = spawn_mock().await;
    mock.refresh
        .lock()
        .unwrap()
        .insert("r1".into(), "r2".into());
    recent::link(
        &f.pool,
        config(&mock),
        f.account_id,
        "good",
        "http://127.0.0.1/callback",
    )
    .await
    .unwrap();
    (f, mock)
}

#[sqlx::test(migrations = "./migrations")]
async fn polls_new_plays_once_and_keeps_the_rotated_token(pool: PgPool) {
    let (f, mock) = linked(pool).await;
    *mock.plays.lock().unwrap() = vec![
        recent_play("2026-11-05T10:05:00.500Z", "t2"),
        recent_play("2026-11-05T10:00:00Z", "t1"),
    ];

    let report = recent::poll(&f.pool, &config(&mock)).await.unwrap();
    assert_eq!(
        report,
        PollReport {
            links: 1,
            plays: 2,
            inserted: 2,
            ..Default::default()
        }
    );

    let link = sqlx::query!(
        "SELECT spotify_user_id, refresh_token, last_played_at FROM spotify_links
         WHERE account_id = $1",
        f.account_id
    )
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(link.spotify_user_id, "spotify-user");
    assert_eq!(link.refresh_token, "r2");
    assert_eq!(
        link.last_played_at,
        Some("2026-11-05T10:05:00.500Z".parse().unwrap())
    );

    // The embedded track fills items and enrichment, so it isn't queued.
    let item = sqlx::query!(
        "SELECT i.name, i.artist, i.album, m.duration_ms, m.status FROM items i
         JOIN item_meta m USING (uri) WHERE uri = 'spotify:track:t1'"
    )
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(item.name.as_deref(), Some("Track t1"));
    assert_eq!(item.artist.as_deref(), Some("Artist"));
    assert_eq!(item.album.as_deref(), Some("Album"));
    assert_eq!(item.duration_ms, Some(200_000));
    assert_eq!(item.status, "ok");
    let queued = sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM enrich_queue WHERE target = 'item'"#
    )
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(queued, 0);

    // The next poll starts after the newest play and finds nothing new.
    mock.refresh
        .lock()
        .unwrap()
        .insert("r2".into(), "r2".into());
    let report = recent::poll(&f.pool, &config(&mock)).await.unwrap();
    assert_eq!(report.inserted, 0);
    let afters = mock.afters.lock().unwrap().clone();
    let newest: DateTime<Utc> = "2026-11-05T10:05:00.500Z".parse().unwrap();
    assert_eq!(afters.first(), Some(&0));
    assert_eq!(afters.last(), Some(&newest.timestamp_millis()));

    // Storing the same plays again is a no-op.
    let plays: Vec<PlayHistory> = mock
        .plays
        .lock()
        .unwrap()
        .iter()
        .map(|p| serde_json::from_value(p.clone()).unwrap())
        .collect();
    assert_eq!(
        recent::store(&f.pool, f.account_id, &plays).await.unwrap(),
        0
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn marks_revoked_links_and_skips_them(pool: PgPool) {
    let (f, mock) = linked(pool).await;
    mock.refresh.lock().unwrap().clear();

    let report = recent::poll(&f.pool, &config(&mock)).await.unwrap();
    assert_eq!((report.links, report.revoked), (1, 1));
    let revoked = sqlx::query_scalar!(
        "SELECT revoked_at FROM spotify_links WHERE account_id = $1",
        f.account_id
    )
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert!(revoked.is_some());

    let report = recent::poll(&f.pool, &config(&mock)).await.unwrap();
    assert_eq!(report.links, 0);

    // Linking again resumes polling.
    recent::link(
        &f.pool,
        config(&mock),
        f.account_id,
        "good",
        "http://127.0.0.1/callback",
    )
    .await
    .unwrap();
    mock.refresh
        .lock()
        .unwrap()
        .insert("r1".into(), "r1".into());
    let report = recent::poll(&f.pool, &config(&mock)).await.unwrap();
    assert_eq!((report.links, report.revoked), (1, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn rejects_codes_without_the_scope(pool: PgPool) {
    let f = Fixture::new(pool).await;
    let mock = spawn_mock().await;
    *mock.scope.lock().unwrap() = "user-read-email".into();

    let result = recent::link(
        &f.pool,
        config(&mock),
        f.account_id,
        "good",
        "http://127.0.0.1/cb",
    )
    .await;
    assert!(matches!(result, Err(LinkError::MissingScope)), "{result:?}");

    let result = recent::link(
        &f.pool,
        config(&mock),
        f.account_id,
        "bad",
        "http://127.0.0.1/cb",
    )
    .await;
    assert!(matches!(result, Err(LinkError::Exchange(_))), "{result:?}");
}

async fn run(f: &Fixture, query: &str) -> Value {
    let schema = spotify_server::graphql::schema(f.pool.clone(), "https://spotify.test", None);
    let response = schema
        .execute(Request::new(query).variables(Variables::from_json(json!({}))))
        .await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    response.data.into_json().unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn exports_take_over_the_recent_plays_they_cover(pool: PgPool) {
    let f = Fixture::new(pool).await;
    let plays: Vec<PlayHistory> = [
        recent_play("2026-11-05T10:00:00Z", "t3"),
        recent_play("2026-10-01T10:00:02Z", "t2"),
    ]
    .into_iter()
    .map(|p| serde_json::from_value(p).unwrap())
    .collect();
    recent::store(&f.pool, f.account_id, &plays).await.unwrap();

    let query = r#"{ spotifyStats {
        summary { plays minutesPlayed }
        plays { edges { node { ts msPlayed source isExtended item { uri } } } }
        recent: plays(sources: [RECENT]) { edges { node { ts } } }
    } }"#;

    // Before any export, recent plays count (without a length) despite the 30s minimum.
    let data = run(&f, query).await;
    assert_eq!(data["spotifyStats"]["summary"]["plays"], 2);
    assert_eq!(data["spotifyStats"]["summary"]["minutesPlayed"], 0);
    let edges = &data["spotifyStats"]["plays"]["edges"];
    assert_eq!(edges[0]["node"]["source"], "RECENT");
    assert_eq!(edges[0]["node"]["msPlayed"], Value::Null);
    assert_eq!(edges[0]["node"]["isExtended"], false);
    assert_eq!(edges[0]["node"]["item"]["uri"], "spotify:track:t3");

    // An export covering October replaces the October recent play; November's stays.
    f.import(
        "2026-10-20T00:00:00Z",
        &[
            export_play("2026-09-01T10:00:00Z", "t1", 120_000),
            export_play("2026-10-01T10:00:00Z", "t2", 180_000),
            export_play("2026-10-15T10:00:00Z", "t1", 120_000),
        ],
    )
    .await;

    let data = run(&f, query).await;
    let sources: Vec<(String, String)> = data["spotifyStats"]["plays"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["node"]["ts"].as_str().unwrap().to_owned(),
                e["node"]["source"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        sources,
        [
            ("2026-11-05T10:00:00+00:00".to_owned(), "RECENT".to_owned()),
            (
                "2026-10-15T10:00:00+00:00".to_owned(),
                "EXTENDED".to_owned()
            ),
            (
                "2026-10-01T10:00:00+00:00".to_owned(),
                "EXTENDED".to_owned()
            ),
            (
                "2026-09-01T10:00:00+00:00".to_owned(),
                "EXTENDED".to_owned()
            ),
        ]
    );
    assert_eq!(data["spotifyStats"]["summary"]["plays"], 4);
    assert_eq!(data["spotifyStats"]["summary"]["minutesPlayed"], 7);
    assert_eq!(
        data["spotifyStats"]["recent"]["edges"],
        json!([{"node": {"ts": "2026-11-05T10:00:00+00:00"}}])
    );
}
