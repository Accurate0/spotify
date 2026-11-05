//! Stats, search and node lookups executed through the GraphQL schema.

mod common;

use async_graphql::{Request, Variables};
use common::Fixture;
use serde_json::{Value, json};
use spotify_server::graphql::AppSchema;
use sqlx::PgPool;

fn play(ts: &str, uri: &str, name: &str, artist: &str, ms: i64, skipped: bool) -> Value {
    json!({
        "ts": ts, "platform": "android", "ms_played": ms,
        "master_metadata_track_name": name, "master_metadata_album_artist_name": artist,
        "master_metadata_album_album_name": format!("{artist} album"),
        "spotify_track_uri": uri, "reason_start": "clickrow", "reason_end": "trackdone",
        "skipped": skipped,
    })
}

fn episode(ts: &str, ms: i64) -> Value {
    json!({
        "ts": ts, "platform": "android", "ms_played": ms,
        "episode_name": "Ep", "episode_show_name": "Show", "spotify_episode_uri": "spotify:episode:e",
        "reason_start": "clickrow", "reason_end": "trackdone",
    })
}

/// Two accounts: "me" listens mostly to Alpha, "other" only to Beta.
async fn setup(pool: PgPool) -> (Fixture, AppSchema) {
    let f = Fixture::new(pool.clone()).await;
    f.import(
        "2024-12-17T00:00:00Z",
        &[
            // 2023-12-31T17:00Z is 2024-01-01 01:00 in Perth (UTC+8).
            play(
                "2023-12-31T17:00:00Z",
                "spotify:track:a1",
                "A1",
                "Alpha",
                180_000,
                false,
            ),
            play(
                "2024-01-02T10:00:00Z",
                "spotify:track:a1",
                "A1",
                "Alpha",
                180_000,
                true,
            ),
            play(
                "2024-01-03T10:00:00Z",
                "spotify:track:a2",
                "A2",
                "Alpha",
                120_000,
                false,
            ),
            play(
                "2024-02-01T10:00:00Z",
                "spotify:track:b1",
                "B1",
                "Beta",
                240_000,
                false,
            ),
            // A skip under 30s: excluded by default.
            play(
                "2024-02-02T10:00:00Z",
                "spotify:track:b1",
                "B1",
                "Beta",
                5_000,
                true,
            ),
            episode("2024-02-03T10:00:00Z", 600_000),
        ],
    )
    .await;

    let other = sqlx::query_scalar!(
        "INSERT INTO accounts (slug, display_name) VALUES ('other', 'other') RETURNING id"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let other = Fixture {
        pool: pool.clone(),
        account_id: other,
    };
    other
        .import(
            "2024-12-17T00:00:00Z",
            &[play(
                "2024-03-01T10:00:00Z",
                "spotify:track:b1",
                "B1",
                "Beta",
                300_000,
                false,
            )],
        )
        .await;

    (f, spotify_server::graphql::schema(pool))
}

async fn run(schema: &AppSchema, query: &str, variables: Value) -> Value {
    let response = schema
        .execute(Request::new(query).variables(Variables::from_json(variables)))
        .await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    response.data.into_json().unwrap()
}

async fn account_id(schema: &AppSchema, slug: &str) -> Value {
    run(
        schema,
        "query($slug: String!) { account(slug: $slug) { id } }",
        json!({"slug": slug}),
    )
    .await["account"]["id"]
        .clone()
}

#[sqlx::test(migrations = "./migrations")]
async fn summary_and_rankings(pool: PgPool) {
    let (_f, schema) = setup(pool).await;
    let schema = &schema;
    let me = account_id(schema, "me").await;

    let data = run(
        schema,
        r#"query($filter: StatsFilter!) {
            stats(filter: $filter) {
                summary { plays minutesPlayed distinctTracks distinctArtists skipRate }
                topArtists { edges { rank plays minutesPlayed node { name } } }
                topItems(by: PLAYS, first: 1) {
                    edges { cursor rank plays node { uri name artistName } }
                    pageInfo { hasNextPage }
                }
                topAlbums(first: 1) { edges { node { name artist { name } } } }
            }
        }"#,
        json!({"filter": {"accounts": [me]}}),
    )
    .await;
    let stats = &data["stats"];

    // 4 music plays over 30s: the 5s skip and the episode are excluded.
    assert_eq!(
        stats["summary"],
        json!({"plays": 4, "minutesPlayed": 12, "distinctTracks": 3, "distinctArtists": 2, "skipRate": 0.25})
    );
    assert_eq!(
        stats["topArtists"]["edges"],
        json!([
            {"rank": 1, "plays": 3, "minutesPlayed": 8, "node": {"name": "Alpha"}},
            {"rank": 2, "plays": 1, "minutesPlayed": 4, "node": {"name": "Beta"}},
        ])
    );
    let top = &stats["topItems"];
    assert_eq!(top["edges"][0]["node"]["uri"], "spotify:track:a1");
    assert_eq!(top["edges"][0]["plays"], 2);
    assert_eq!(top["pageInfo"]["hasNextPage"], true);
    assert_eq!(
        stats["topAlbums"]["edges"][0]["node"],
        json!({"name": "Alpha album", "artist": {"name": "Alpha"}})
    );

    // The next page continues the ranking.
    let cursor = top["edges"][0]["cursor"].clone();
    let next = run(
        schema,
        r#"query($filter: StatsFilter!, $after: String) {
            stats(filter: $filter) { topItems(by: PLAYS, first: 1, after: $after) { edges { rank } } }
        }"#,
        json!({"filter": {"accounts": [me]}, "after": cursor}),
    )
    .await;
    assert_eq!(next["stats"]["topItems"]["edges"][0]["rank"], 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn filters(pool: PgPool) {
    let (_f, schema) = setup(pool).await;
    let schema = &schema;

    let plays = |filter: Value| {
        let schema = schema.clone();
        async move {
            run(
                &schema,
                "query($filter: StatsFilter!) { stats(filter: $filter) { summary { plays } } }",
                json!({"filter": filter}),
            )
            .await["stats"]["summary"]["plays"]
                .as_i64()
                .unwrap()
        }
    };

    assert_eq!(plays(json!({})).await, 5, "all accounts");
    assert_eq!(plays(json!({"minMsPlayed": 0})).await, 6, "skips included");
    assert_eq!(
        plays(json!({"kinds": ["TRACK", "EPISODE"]})).await,
        6,
        "episodes included"
    );
    assert_eq!(
        plays(json!({"from": "2024-02-01T00:00:00Z", "to": "2024-03-01T00:00:00Z"})).await,
        1,
        "to is exclusive"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn timeline_and_heatmap_use_time_zone(pool: PgPool) {
    let (_f, schema) = setup(pool).await;
    let schema = &schema;
    let me = account_id(schema, "me").await;

    let query = r#"query($filter: StatsFilter!) {
        stats(filter: $filter) {
            timeline(bucket: MONTH) { start plays }
            heatmap { dayOfWeek hour plays }
        }
    }"#;

    // In Perth the first play is on 2024-01-01 (a Monday) at 01:00.
    let perth = run(schema, query, json!({"filter": {"accounts": [me]}})).await;
    assert_eq!(
        perth["stats"]["timeline"],
        json!([
            {"start": "2023-12-31T16:00:00+00:00", "plays": 3},
            {"start": "2024-01-31T16:00:00+00:00", "plays": 1},
        ])
    );
    assert_eq!(
        perth["stats"]["heatmap"][0],
        json!({"dayOfWeek": 1, "hour": 1, "plays": 1})
    );

    // In UTC it falls in December, on a Sunday.
    let utc = run(
        schema,
        query,
        json!({"filter": {"accounts": [me], "timeZone": "UTC"}}),
    )
    .await;
    assert_eq!(
        utc["stats"]["timeline"][0]["start"],
        "2023-12-01T00:00:00+00:00"
    );
    assert_eq!(utc["stats"]["timeline"][0]["plays"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn item_artist_and_search(pool: PgPool) {
    let (_f, schema) = setup(pool).await;
    let schema = &schema;

    let data = run(
        schema,
        r#"{
            search(query: "alp") {
                items { edges { node { id uri } } }
                artists { edges { node { id name } } }
            }
            item(uri: "spotify:track:b1") {
                stats { summary { plays firstPlayed lastPlayed } }
                plays: stats(filter: { minMsPlayed: 0 }) {
                    plays { edges { node { msPlayed account { slug } item { name } } } }
                }
            }
            artist(name: "Alpha") { stats { topItems { edges { node { name } } } } }
            missing: artist(name: "Nobody") { name }
        }"#,
        json!({}),
    )
    .await;

    let items = data["search"]["items"]["edges"].as_array().unwrap();
    assert_eq!(items.len(), 2, "both Alpha tracks match on artist");
    assert_eq!(
        data["search"]["artists"]["edges"][0]["node"]["name"],
        "Alpha"
    );

    // Across both accounts: 2 plays over 30s.
    let summary = &data["item"]["stats"]["summary"];
    assert_eq!(summary["plays"], 2);
    assert_eq!(summary["firstPlayed"], "2024-02-01T10:00:00+00:00");
    assert_eq!(summary["lastPlayed"], "2024-03-01T10:00:00+00:00");

    let plays = &data["item"]["plays"]["plays"]["edges"];
    assert_eq!(
        plays,
        &json!([
            {"node": {"msPlayed": 300000, "account": {"slug": "other"}, "item": {"name": "B1"}}},
            {"node": {"msPlayed": 5000, "account": {"slug": "me"}, "item": {"name": "B1"}}},
            {"node": {"msPlayed": 240000, "account": {"slug": "me"}, "item": {"name": "B1"}}},
        ])
    );

    let artist_tracks = &data["artist"]["stats"]["topItems"]["edges"];
    assert_eq!(artist_tracks.as_array().unwrap().len(), 2);
    assert_eq!(data["missing"], Value::Null);

    // Relay refetch through node(id).
    let item_id = items[0]["node"]["id"].clone();
    let artist_id = data["search"]["artists"]["edges"][0]["node"]["id"].clone();
    let nodes = run(
        schema,
        r#"query($ids: [ID!]!) { nodes(ids: $ids) { __typename ... on Artist { name } ... on Item { uri } } }"#,
        json!({"ids": [item_id, artist_id]}),
    )
    .await;
    assert_eq!(
        nodes["nodes"][1],
        json!({"__typename": "Artist", "name": "Alpha"})
    );
    assert_eq!(nodes["nodes"][0]["__typename"], "Item");
}

#[sqlx::test(migrations = "./migrations")]
async fn insights(pool: PgPool) {
    let (_f, schema) = setup(pool).await;
    let schema = &schema;
    let me = account_id(schema, "me").await;

    let data = run(
        schema,
        r#"query($filter: StatsFilter!, $me: ID!) {
            stats(filter: $filter) {
                firstPlay { ts item { name } }
                listeningDays { activeDays spanDays longestStreak { start end days } latestStreak { days } }
                topDays(first: 1) { date minutesPlayed topArtist { name } }
                onRepeat(first: 1) { date plays item { name } }
                sessions { count longest { plays } }
                device: breakdown(by: DEVICE) { label plays }
                ends: breakdown(by: END_REASON) { key label }
                discoveries { edges { node { artist { name } firstItem { name } } } }
            }
            all: stats(filter: { accounts: [$me], minMsPlayed: 0 }) {
                mostSkipped(minPlays: 2) { item { name } skips plays }
            }
        }"#,
        json!({"filter": {"accounts": [me]}, "me": me}),
    )
    .await;
    let stats = &data["stats"];

    assert_eq!(stats["firstPlay"]["item"]["name"], "A1");
    // Perth days: Jan 1, 2, 3 (a streak of 3) and Feb 1.
    assert_eq!(
        stats["listeningDays"],
        json!({
            "activeDays": 4,
            "spanDays": 32,
            "longestStreak": {"start": "2024-01-01", "end": "2024-01-03", "days": 3},
            "latestStreak": {"days": 1},
        })
    );
    assert_eq!(
        stats["topDays"][0],
        json!({"date": "2024-02-01", "minutesPlayed": 4, "topArtist": {"name": "Beta"}})
    );
    assert_eq!(stats["onRepeat"][0]["plays"], 1);
    // Plays days apart never share a session.
    assert_eq!(stats["sessions"]["count"], 4);
    assert_eq!(stats["device"], json!([{"label": "Android", "plays": 4}]));
    assert_eq!(
        stats["ends"],
        json!([{"key": "trackdone", "label": "Previous track finished"}])
    );
    assert_eq!(
        stats["discoveries"]["edges"][0]["node"],
        json!({"artist": {"name": "Alpha"}, "firstItem": {"name": "A1"}})
    );

    // a1 and b1 each have one skip in two plays (b1's is the 5s play, which only counts
    // because skips ignore the minimum play length); the tie goes to the lower URI.
    let skipped = &data["all"]["mostSkipped"];
    assert_eq!(skipped[0]["item"]["name"], "A1");
    assert_eq!(skipped[0]["skips"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn discoveries_respect_first_play_and_forgotten_needs_a_year(pool: PgPool) {
    let (_f, schema) = setup(pool).await;
    let schema = &schema;
    let me = account_id(schema, "me").await;

    // Beta was first heard on Feb 1, so a February-only range discovers Beta but not Alpha.
    let feb = run(
        schema,
        r#"query($filter: StatsFilter!) {
            stats(filter: $filter) { discoveries { edges { node { artist { name } } } } }
        }"#,
        json!({"filter": {"accounts": [me], "from": "2024-01-31T16:00:00Z", "to": "2024-02-29T16:00:00Z"}}),
    )
    .await;
    assert_eq!(
        feb["stats"]["discoveries"]["edges"],
        json!([{"node": {"artist": {"name": "Beta"}}}])
    );

    // All plays fall within two months, so nothing is "forgotten".
    let forgotten = run(
        schema,
        r#"{ stats { forgottenArtists { edges { node { artist { name } } } } } }"#,
        json!({}),
    )
    .await;
    assert_eq!(forgotten["stats"]["forgottenArtists"]["edges"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn artist_skip_rate_leaderboard(pool: PgPool) {
    let (_f, schema) = setup(pool).await;
    let schema = &schema;
    let me = account_id(schema, "me").await;

    // Alpha: 1 skip in 3 plays. Beta: 1 skip in 2, the skip being a 5s play that only
    // counts because skip rates ignore the minimum play length.
    let data = run(
        schema,
        r#"query($filter: StatsFilter!) {
            stats(filter: $filter) {
                most: artistSkipRates(minPlays: 2) { rank artist { name } skips plays }
                least: artistSkipRates(order: LEAST_SKIPPED, minPlays: 2) { artist { name } }
                busy: artistSkipRates(minPlays: 3) { artist { name } }
            }
        }"#,
        json!({"filter": {"accounts": [me]}}),
    )
    .await;
    let stats = &data["stats"];

    assert_eq!(
        stats["most"],
        json!([
            {"rank": 1, "artist": {"name": "Beta"}, "skips": 1, "plays": 2},
            {"rank": 2, "artist": {"name": "Alpha"}, "skips": 1, "plays": 3},
        ])
    );
    assert_eq!(
        stats["least"],
        json!([{"artist": {"name": "Alpha"}}, {"artist": {"name": "Beta"}}])
    );
    assert_eq!(stats["busy"], json!([{"artist": {"name": "Alpha"}}]));
}

/// Spotify metadata as `spotify-server enrich` would store it: Alpha's tracks are on a 1999
/// album (with artwork), Beta's on a 2024 one.
async fn seed_enrichment(pool: &PgPool) {
    sqlx::query!(
        "INSERT INTO spotify_albums (id, name, album_type, release_date, release_year, image_key)
         VALUES ('al_a', 'Alpha album', 'album', '1999-03-14', 1999, 'albums/al_a.jpg'),
                ('al_b', 'Beta album', 'album', '2024', 2024, NULL)"
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO item_meta (uri, duration_ms, explicit, album_id, status)
         VALUES ('spotify:track:a1', 200000, false, 'al_a', 'ok'),
                ('spotify:track:a2', 240000, true, 'al_a', 'ok'),
                ('spotify:track:b1', 240000, false, 'al_b', 'ok')"
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO item_artists (item_uri, position, artist_id, artist_name)
         VALUES ('spotify:track:a1', 0, 'ar_alpha', 'Alpha'),
                ('spotify:track:a2', 0, 'ar_alpha', 'Alpha'),
                ('spotify:track:a2', 1, 'ar_guest', 'Guest'),
                ('spotify:track:b1', 0, 'ar_beta', 'Beta')"
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO spotify_artists (id, name, genres, image_key, status)
         VALUES ('ar_alpha', 'Alpha', '{indie,pop}', 'artists/ar_alpha.jpg', 'ok'),
                ('ar_beta', 'Beta', '{rock}', NULL, 'ok'),
                ('ar_guest', 'Guest', '{}', NULL, 'ok')"
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!("REFRESH MATERIALIZED VIEW artist_by_name")
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn enriched_stats(pool: PgPool) {
    let (_f, schema) = setup(pool.clone()).await;
    seed_enrichment(&pool).await;
    let schema = &schema;
    let me = account_id(schema, "me").await;

    let data = run(
        schema,
        r#"query($filter: StatsFilter!) {
            stats(filter: $filter) {
                topGenres { edges { rank minutesPlayed node { name } } }
                decades: breakdown(by: DECADE) { key label minutesPlayed }
                musicAge { averageYears newShare coverage oldestYear oldest { name } }
                completion { average finishedShare coverage }
                mostAbandoned(minPlays: 2) { item { name } plays averageCompletion }
                topAlbums(first: 1) { edges { node { name artworkUrl releaseYear } } }
                genreTimeline(bucket: YEAR) { genre points { minutesPlayed } }
            }
            item(uri: "spotify:track:a2") {
                artworkUrl durationMs explicit releaseYear releaseDate enriched
                artists { name }
                album { artworkUrl releaseYear }
            }
            bare: item(uri: "spotify:episode:e") { enriched artworkUrl artists { name } }
            artist(name: "Alpha") { genres imageUrl }
            enrichment { tracksEnriched tracksPending artistsEnriched artistsPending artworkStored }
        }"#,
        json!({"filter": {"accounts": [me]}}),
    )
    .await;
    let stats = &data["stats"];

    // Streams over 30s: a1 3m + 3m and a2 2m (Alpha: indie, pop), b1 4m (Beta: rock).
    assert_eq!(
        stats["topGenres"]["edges"],
        json!([
            {"rank": 1, "minutesPlayed": 8, "node": {"name": "indie"}},
            {"rank": 2, "minutesPlayed": 8, "node": {"name": "pop"}},
            {"rank": 3, "minutesPlayed": 4, "node": {"name": "rock"}},
        ])
    );
    assert_eq!(
        stats["decades"],
        json!([
            {"key": "1990s", "label": "1990s", "minutesPlayed": 8},
            {"key": "2020s", "label": "2020s", "minutesPlayed": 4},
        ])
    );

    // Played in 2024 (Perth): three plays of 1999 music, one of 2024 music.
    let age = &stats["musicAge"];
    assert_eq!(age["averageYears"], 18.75);
    assert_eq!(age["newShare"], 0.25);
    assert_eq!(age["coverage"], 1.0);
    assert_eq!(age["oldestYear"], 1999);

    // Every play counts here, including b1's 5s skip: 0.9, 0.9, 0.5, 1.0 and 5/240.
    let completion = &stats["completion"];
    let average = completion["average"].as_f64().unwrap();
    assert!((average - (0.9 + 0.9 + 0.5 + 1.0 + 5.0 / 240.0) / 5.0).abs() < 1e-9);
    assert_eq!(completion["finishedShare"], 0.6);
    assert_eq!(completion["coverage"], 1.0);

    let abandoned = &stats["mostAbandoned"];
    assert_eq!(abandoned[0]["item"]["name"], "B1");
    assert_eq!(abandoned[1]["item"]["name"], "A1");

    assert_eq!(
        stats["topAlbums"]["edges"][0]["node"],
        json!({"name": "Alpha album", "artworkUrl": "/art/albums/al_a", "releaseYear": 1999})
    );
    let timeline = stats["genreTimeline"].as_array().unwrap();
    assert_eq!(timeline.len(), 3);
    assert_eq!(timeline[0]["genre"], "indie");

    assert_eq!(
        data["item"],
        json!({
            "artworkUrl": "/art/albums/al_a", "durationMs": 240000, "explicit": true,
            "releaseYear": 1999, "releaseDate": "1999-03-14", "enriched": true,
            "artists": [{"name": "Alpha"}, {"name": "Guest"}],
            "album": {"artworkUrl": "/art/albums/al_a", "releaseYear": 1999},
        })
    );
    assert_eq!(
        data["bare"],
        json!({"enriched": false, "artworkUrl": null, "artists": []})
    );
    assert_eq!(
        data["artist"],
        json!({"genres": ["indie", "pop"], "imageUrl": "/art/artists/ar_alpha"})
    );
    // The episode is the only item left; every credited artist is fetched.
    assert_eq!(
        data["enrichment"],
        json!({"tracksEnriched": 3, "tracksPending": 1, "artistsEnriched": 3, "artistsPending": 0, "artworkStored": 2})
    );
}
