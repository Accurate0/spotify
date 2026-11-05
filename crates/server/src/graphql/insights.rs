//! Derived stats beyond totals and rankings: firsts, streaks, sessions, discoveries, days on
//! repeat, skips, forgotten favourites and listening-context breakdowns. Each runs over the
//! same `Scope` as the rest of `Stats`, so they also work narrowed to an artist or item.
//! Days are calendar days in the scope's time zone.

use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, Enum, Object, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;

use super::catalog::{Artist, Item};
use super::loaders::ItemLoader;
use super::page::{Page, PlainConnection, RankedConnection};
use super::stats::{Play, Scope};

#[derive(SimpleObject)]
#[graphql(name = "SpotifyStreak")]
pub struct Streak {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub days: i32,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyListeningDays")]
pub struct ListeningDays {
    /// Days with at least one play.
    pub active_days: i32,
    /// Days from the first to the last play, inclusive.
    pub span_days: i32,
    /// Longest run of consecutive active days.
    pub longest_streak: Option<Streak>,
    /// The run ending on the last active day.
    pub latest_streak: Option<Streak>,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyDayStat")]
pub struct DayStat {
    pub date: NaiveDate,
    pub plays: i32,
    pub minutes_played: i32,
    /// The artist with the most listening that day.
    pub top_artist: Option<Artist>,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyRepeatDay")]
pub struct RepeatDay {
    pub item: Item,
    /// The day this item was played most.
    pub date: NaiveDate,
    pub plays: i32,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifySkippedItem")]
pub struct SkippedItem {
    pub item: Item,
    /// Plays where the export records whether it was skipped.
    pub plays: i32,
    pub skips: i32,
    pub skip_rate: f64,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyArtistSkipRate")]
pub struct ArtistSkipRate {
    pub artist: Artist,
    /// Plays where the export records whether it was skipped.
    pub plays: i32,
    pub skips: i32,
    pub skip_rate: f64,
    /// 1-based position in the leaderboard.
    pub rank: i32,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq, Default)]
#[graphql(name = "SpotifySkipOrder")]
pub enum SkipOrder {
    /// Highest skip rate first.
    #[default]
    MostSkipped,
    /// Lowest skip rate first: artists you let play.
    LeastSkipped,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(name = "SpotifyBreakdownDimension")]
pub enum BreakdownDimension {
    /// Device family: Android, iOS, macOS, Windows, Linux, Web player, Other.
    Device,
    /// Country the stream was played from.
    Country,
    Shuffle,
    Offline,
    /// Why playback started (finished previous track, skipped forward, picked, ...).
    StartReason,
    /// Why playback ended.
    EndReason,
    /// Release decade of the track's album (needs enrichment).
    Decade,
}

impl BreakdownDimension {
    fn as_str(self) -> &'static str {
        match self {
            Self::Device => "device",
            Self::Country => "country",
            Self::Shuffle => "shuffle",
            Self::Offline => "offline",
            Self::StartReason => "start",
            Self::EndReason => "end",
            Self::Decade => "decade",
        }
    }
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyBreakdownRow")]
pub struct BreakdownRow {
    /// Raw value, e.g. `fwdbtn` or `AU`; null when the export doesn't record it.
    pub key: Option<String>,
    /// Human-readable form of `key`.
    pub label: String,
    pub plays: i32,
    pub minutes_played: i32,
}

/// Readable labels for Spotify's `reason_start` / `reason_end` codes.
fn reason_label(code: &str) -> String {
    match code {
        "trackdone" => "Previous track finished",
        "fwdbtn" => "Skipped forward",
        "backbtn" => "Skipped back",
        "clickrow" => "Picked from a list",
        "clickside" => "Picked from the sidebar",
        "playbtn" => "Play button",
        "appload" => "App opened",
        "remote" => "Remote control",
        "popup" => "Pop-up",
        "endplay" => "Stopped",
        "logout" => "Logged out",
        "unexpected-exit" => "App closed",
        "unexpected-exit-while-paused" => "App closed while paused",
        "trackerror" => "Playback error",
        "unknown" => "Unknown",
        other => return other.to_owned(),
    }
    .to_owned()
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifySession")]
pub struct Session {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub plays: i32,
    pub minutes_played: i32,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifySessions")]
pub struct Sessions {
    /// Listening sessions: plays separated by less than 30 minutes of silence.
    pub count: i32,
    pub average_minutes: Option<f64>,
    pub longest: Option<Session>,
}

pub struct Discovery {
    artist: String,
    first_played: DateTime<Utc>,
    first_item_uri: Option<String>,
    plays: i32,
    minutes_played: i32,
}

#[Object(name = "SpotifyDiscovery")]
impl Discovery {
    async fn artist(&self) -> Artist {
        Artist {
            name: self.artist.clone(),
        }
    }

    /// First play ever (within the filter's accounts and kinds), which falls in the range.
    async fn first_played(&self) -> DateTime<Utc> {
        self.first_played
    }

    /// The first track heard by this artist.
    async fn first_item(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Item>> {
        let Some(uri) = &self.first_item_uri else {
            return Ok(None);
        };
        Ok(ctx
            .data::<DataLoader<ItemLoader>>()?
            .load_one(uri.clone())
            .await?)
    }

    /// Plays in the range.
    async fn plays(&self) -> i32 {
        self.plays
    }

    async fn minutes_played(&self) -> i32 {
        self.minutes_played
    }
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyForgottenArtist")]
pub struct ForgottenArtist {
    pub artist: Artist,
    pub plays: i32,
    pub minutes_played: i32,
    pub last_played: DateTime<Utc>,
}

fn item(
    uri: String,
    kind: String,
    name: Option<String>,
    artist: Option<String>,
    album: Option<String>,
) -> Item {
    Item {
        uri,
        kind,
        name,
        artist,
        album,
    }
}

pub async fn first_play(pool: &PgPool, scope: &Scope) -> sqlx::Result<Option<Play>> {
    scoped!(
        query_as!(Play),
        scope,
        r#"SELECT account_id AS "account_id!", ts AS "ts!", item_uri, name, artist,
                  ms_played AS "ms_played!", skipped, is_extended AS "is_extended!"
           FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
           ORDER BY ts
           LIMIT 1"#
    )
    .fetch_optional(pool)
    .await
}

pub async fn listening_days(pool: &PgPool, scope: &Scope) -> sqlx::Result<ListeningDays> {
    let row = scoped!(
        query!,
        scope,
        r#"WITH days AS (
               SELECT DISTINCT (ts AT TIME ZONE $8)::date AS d
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
           ),
           -- Consecutive days share d - row_number(), which identifies each run.
           runs AS (
               SELECT min(d) AS start_d, max(d) AS end_d, count(*)::int AS len
               FROM (SELECT d, d - (row_number() OVER (ORDER BY d))::int AS grp FROM days) g
               GROUP BY grp
           )
           SELECT (SELECT count(*)::int FROM days) AS "active!",
                  (SELECT max(d) - min(d) + 1 FROM days) AS span,
                  l.start_d AS longest_start, l.end_d AS longest_end, l.len AS longest_len,
                  r.start_d AS latest_start, r.end_d AS latest_end, r.len AS latest_len
           FROM (SELECT 1) one
           LEFT JOIN LATERAL (SELECT * FROM runs ORDER BY len DESC, end_d DESC LIMIT 1) l ON TRUE
           LEFT JOIN LATERAL (SELECT * FROM runs ORDER BY end_d DESC LIMIT 1) r ON TRUE"#,
        scope.time_zone,
    )
    .fetch_one(pool)
    .await?;

    let streak = |start: Option<NaiveDate>, end: Option<NaiveDate>, days: Option<i32>| {
        Some(Streak {
            start: start?,
            end: end?,
            days: days?,
        })
    };

    Ok(ListeningDays {
        active_days: row.active,
        span_days: row.span.unwrap_or(0),
        longest_streak: streak(row.longest_start, row.longest_end, row.longest_len),
        latest_streak: streak(row.latest_start, row.latest_end, row.latest_len),
    })
}

pub async fn top_days(pool: &PgPool, scope: &Scope, first: i32) -> sqlx::Result<Vec<DayStat>> {
    let rows = scoped!(
        query!,
        scope,
        r#"WITH p AS MATERIALIZED (
               SELECT (ts AT TIME ZONE $8)::date AS day, artist, ms_played
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
           ),
           d AS (
               SELECT day, count(*)::int AS plays, sum(ms_played) AS ms
               FROM p GROUP BY day
               ORDER BY ms DESC, day DESC
               LIMIT $9
           ),
           a AS (
               SELECT DISTINCT ON (p.day) p.day, p.artist
               FROM p JOIN d USING (day)
               WHERE p.artist IS NOT NULL
               GROUP BY p.day, p.artist
               ORDER BY p.day, sum(p.ms_played) DESC
           )
           SELECT d.day AS "day!", d.plays AS "plays!", (d.ms / 60000)::int AS "minutes!", a.artist
           FROM d LEFT JOIN a USING (day)
           ORDER BY d.ms DESC, d.day DESC"#,
        scope.time_zone,
        first.clamp(1, 100) as i64,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| DayStat {
            date: r.day,
            plays: r.plays,
            minutes_played: r.minutes,
            top_artist: r.artist.map(|name| Artist { name }),
        })
        .collect())
}

pub async fn on_repeat(pool: &PgPool, scope: &Scope, first: i32) -> sqlx::Result<Vec<RepeatDay>> {
    let rows = scoped!(
        query!,
        scope,
        r#"WITH per_day AS (
               SELECT item_uri, (ts AT TIME ZONE $8)::date AS day, count(*) AS plays
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
               WHERE item_uri IS NOT NULL
               GROUP BY 1, 2
           ),
           best AS (
               SELECT DISTINCT ON (item_uri) item_uri, day, plays
               FROM per_day
               ORDER BY item_uri, plays DESC, day DESC
           )
           SELECT i.uri, i.kind, i.name, i.artist, i.album,
                  b.day AS "day!", b.plays::int AS "plays!"
           FROM best b JOIN items i ON i.uri = b.item_uri
           ORDER BY b.plays DESC, b.day DESC
           LIMIT $9"#,
        scope.time_zone,
        first.clamp(1, 100) as i64,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| RepeatDay {
            item: item(r.uri, r.kind, r.name, r.artist, r.album),
            date: r.day,
            plays: r.plays,
        })
        .collect())
}

pub async fn most_skipped(
    pool: &PgPool,
    scope: &Scope,
    first: i32,
    min_plays: i32,
) -> sqlx::Result<Vec<SkippedItem>> {
    // Skips are mostly short plays, so this always counts every play regardless of the
    // filter's minimum (least($5, 0) is 0 since the minimum is never negative).
    let rows = scoped!(
        query!,
        scope,
        r#"SELECT i.uri, i.kind, i.name, i.artist, i.album,
                  count(*)::int AS "plays!",
                  count(*) FILTER (WHERE p.skipped)::int AS "skips!"
           FROM scoped_plays($1, $2, $3, $4, least($5, 0), $6, $7) p
           JOIN items i ON i.uri = p.item_uri
           WHERE p.skipped IS NOT NULL
           GROUP BY i.uri
           HAVING count(*) >= $8
           ORDER BY count(*) FILTER (WHERE p.skipped)::float8 / count(*) DESC, count(*) DESC, i.uri
           LIMIT $9"#,
        min_plays.max(1) as i64,
        first.clamp(1, 100) as i64,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SkippedItem {
            skip_rate: r.skips as f64 / r.plays as f64,
            item: item(r.uri, r.kind, r.name, r.artist, r.album),
            plays: r.plays,
            skips: r.skips,
        })
        .collect())
}

pub async fn artist_skip_rates(
    pool: &PgPool,
    scope: &Scope,
    order: SkipOrder,
    first: i32,
    min_plays: i32,
) -> sqlx::Result<Vec<ArtistSkipRate>> {
    // Like most_skipped, counts every play regardless of the filter's minimum length.
    let rows = scoped!(
        query!,
        scope,
        r#"SELECT artist AS "artist!",
                  count(*)::int AS "plays!",
                  count(*) FILTER (WHERE skipped)::int AS "skips!"
           FROM scoped_plays($1, $2, $3, $4, least($5, 0), $6, $7)
           WHERE skipped IS NOT NULL AND artist IS NOT NULL
           GROUP BY artist
           HAVING count(*) >= $8
           ORDER BY CASE WHEN $9 THEN 1 ELSE -1 END
                        * count(*) FILTER (WHERE skipped)::float8 / count(*) DESC,
                    count(*) DESC, artist
           LIMIT $10"#,
        min_plays.max(1) as i64,
        order == SkipOrder::MostSkipped,
        first.clamp(1, 100) as i64,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| ArtistSkipRate {
            skip_rate: r.skips as f64 / r.plays as f64,
            artist: Artist { name: r.artist },
            plays: r.plays,
            skips: r.skips,
            rank: i as i32 + 1,
        })
        .collect())
}

pub async fn breakdown(
    pool: &PgPool,
    scope: &Scope,
    by: BreakdownDimension,
) -> sqlx::Result<Vec<BreakdownRow>> {
    let rows = scoped!(
        query!,
        scope,
        r#"SELECT CASE $8::text
                      WHEN 'device' THEN device_family(platform)
                      WHEN 'country' THEN conn_country
                      WHEN 'shuffle' THEN CASE shuffle WHEN TRUE THEN 'on' WHEN FALSE THEN 'off' END
                      WHEN 'offline' THEN CASE offline WHEN TRUE THEN 'offline' WHEN FALSE THEN 'online' END
                      WHEN 'start' THEN reason_start
                      WHEN 'end' THEN reason_end
                      WHEN 'decade' THEN (al.release_year / 10 * 10)::text || 's'
                  END AS key,
                  count(*)::int AS "plays!",
                  (sum(ms_played) / 60000)::int AS "minutes!"
           FROM scoped_plays($1, $2, $3, $4, $5, $6, $7) p
           LEFT JOIN item_meta m ON m.uri = p.item_uri
           LEFT JOIN spotify_albums al ON al.id = m.album_id
           GROUP BY 1
           ORDER BY 3 DESC, 1"#,
        by.as_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let label = match (by, r.key.as_deref()) {
                (BreakdownDimension::Decade, None) => "Unknown yet".to_owned(),
                (_, None) => "Not recorded".to_owned(),
                (BreakdownDimension::Shuffle, Some(k)) => format!("Shuffle {k}"),
                (BreakdownDimension::Offline, Some("offline")) => "Offline".to_owned(),
                (BreakdownDimension::Offline, Some(_)) => "Online".to_owned(),
                (BreakdownDimension::StartReason | BreakdownDimension::EndReason, Some(k)) => {
                    reason_label(k)
                }
                (_, Some(k)) => k.to_owned(),
            };
            BreakdownRow {
                key: r.key,
                label,
                plays: r.plays,
                minutes_played: r.minutes,
            }
        })
        .collect())
}

pub async fn sessions(pool: &PgPool, scope: &Scope) -> sqlx::Result<Sessions> {
    let row = scoped!(
        query!,
        scope,
        r#"WITH p AS (
               SELECT account_id, ts, ms_played,
                      ts - make_interval(secs => ms_played / 1000.0) AS started
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
           ),
           -- A play starting more than 30 minutes after the previous one ended opens a session.
           marked AS (
               SELECT *, CASE WHEN lag(ts) OVER w IS NULL
                                OR started - lag(ts) OVER w > interval '30 minutes'
                              THEN 1 ELSE 0 END AS boundary
               FROM p
               WINDOW w AS (PARTITION BY account_id ORDER BY ts)
           ),
           numbered AS (
               SELECT *, sum(boundary) OVER (PARTITION BY account_id ORDER BY ts) AS sid
               FROM marked
           ),
           s AS (
               SELECT account_id, sid, min(started) AS start_ts, max(ts) AS end_ts,
                      count(*)::int AS plays, sum(ms_played) AS ms
               FROM numbered
               GROUP BY account_id, sid
           )
           SELECT (SELECT count(*)::int FROM s) AS "count!",
                  (SELECT avg(ms)::float8 / 60000 FROM s) AS average_minutes,
                  l.start_ts, l.end_ts, l.plays, (l.ms / 60000)::int AS minutes
           FROM (SELECT 1) one
           LEFT JOIN LATERAL (SELECT * FROM s ORDER BY ms DESC LIMIT 1) l ON TRUE"#
    )
    .fetch_one(pool)
    .await?;

    let longest = match (row.start_ts, row.end_ts, row.plays, row.minutes) {
        (Some(start), Some(end), Some(plays), Some(minutes)) => Some(Session {
            start,
            end,
            plays,
            minutes_played: minutes,
        }),
        _ => None,
    };

    Ok(Sessions {
        count: row.count,
        average_minutes: row.average_minutes,
        longest,
    })
}

pub async fn discoveries(
    pool: &PgPool,
    scope: &Scope,
    page: &Page,
) -> sqlx::Result<PlainConnection<Discovery>> {
    let rows = scoped!(
        query!,
        scope,
        r#"WITH firsts AS (
               -- First play ever, ignoring the date range.
               SELECT DISTINCT ON (artist) artist, ts AS first_ts, item_uri
               FROM scoped_plays($1, NULL, NULL, $4, $5, $6, $7)
               WHERE artist IS NOT NULL
               ORDER BY artist, ts
           ),
           in_range AS (
               SELECT artist, count(*) AS plays, sum(ms_played) AS ms
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
               WHERE artist IS NOT NULL
               GROUP BY artist
           )
           SELECT f.artist AS "artist!", f.first_ts AS "first_played!", f.item_uri,
                  r.plays::int AS "plays!", (r.ms / 60000)::int AS "minutes!"
           FROM firsts f JOIN in_range r USING (artist)
           WHERE ($2::timestamptz IS NULL OR f.first_ts >= $2)
             AND ($3::timestamptz IS NULL OR f.first_ts < $3)
           ORDER BY r.ms DESC, f.artist
           LIMIT $8 OFFSET $9"#,
        page.fetch(),
        page.offset,
    )
    .fetch_all(pool)
    .await?;

    Ok(page.plain(rows.into_iter().map(|r| Discovery {
        artist: r.artist,
        first_played: r.first_played,
        first_item_uri: r.item_uri,
        plays: r.plays,
        minutes_played: r.minutes,
    })))
}

pub async fn forgotten_artists(
    pool: &PgPool,
    scope: &Scope,
    page: &Page,
) -> sqlx::Result<PlainConnection<ForgottenArtist>> {
    let rows = scoped!(
        query!,
        scope,
        r#"WITH a AS (
               SELECT artist, count(*) AS plays, sum(ms_played) AS ms, max(ts) AS last_ts
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
               WHERE artist IS NOT NULL
               GROUP BY artist
           )
           SELECT artist AS "artist!", plays::int AS "plays!", (ms / 60000)::int AS "minutes!",
                  last_ts AS "last_played!"
           FROM a
           WHERE last_ts < (SELECT max(last_ts) FROM a) - interval '1 year'
           ORDER BY ms DESC, artist
           LIMIT $8 OFFSET $9"#,
        page.fetch(),
        page.offset,
    )
    .fetch_all(pool)
    .await?;

    Ok(page.plain(rows.into_iter().map(|r| ForgottenArtist {
        artist: Artist { name: r.artist },
        plays: r.plays,
        minutes_played: r.minutes,
        last_played: r.last_played,
    })))
}

#[derive(SimpleObject, Clone)]
#[graphql(name = "SpotifyGenre")]
pub struct Genre {
    pub name: String,
}

pub async fn top_genres(
    pool: &PgPool,
    scope: &Scope,
    by_plays: bool,
    page: &Page,
) -> sqlx::Result<RankedConnection<Genre>> {
    // A play counts toward every genre of its track's primary artist, so genre shares overlap.
    let rows = scoped!(
        query!,
        scope,
        r#"SELECT g AS "genre!", count(*)::int AS "plays!", (sum(p.ms_played) / 60000)::int AS "minutes!"
           FROM scoped_plays($1, $2, $3, $4, $5, $6, $7) p
           JOIN item_artists ia ON ia.item_uri = p.item_uri AND ia.position = 0
           JOIN spotify_artists sa ON sa.id = ia.artist_id
           CROSS JOIN LATERAL unnest(sa.genres) AS g
           GROUP BY g
           ORDER BY CASE WHEN $8 THEN count(*) ELSE sum(p.ms_played) END DESC, g
           LIMIT $9 OFFSET $10"#,
        by_plays,
        page.fetch(),
        page.offset,
    )
    .fetch_all(pool)
    .await?;

    Ok(page.ranked(
        rows.into_iter()
            .map(|r| (Genre { name: r.genre }, r.plays, r.minutes)),
    ))
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyGenrePoint")]
pub struct GenrePoint {
    pub start: DateTime<Utc>,
    pub minutes_played: i32,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyGenreSeries")]
pub struct GenreSeries {
    pub genre: String,
    pub points: Vec<GenrePoint>,
}

/// Listening per bucket for the given genres, or the top `first` genres: in the scope's
/// date range, or over all time when `top_over_all_time` (so a chart keeps the same genres,
/// and colours, as the range changes).
pub async fn genre_timeline(
    pool: &PgPool,
    scope: &Scope,
    bucket: &str,
    genres: &[String],
    first: i32,
    top_over_all_time: bool,
) -> sqlx::Result<Vec<GenreSeries>> {
    let genres: Vec<String> = if genres.is_empty() {
        let page = Page::new(Some(first.clamp(1, 8)), None, 5).expect("valid page");
        let ranking_scope = if top_over_all_time {
            scope.clone().all_time()
        } else {
            scope.clone()
        };
        top_genres(pool, &ranking_scope, false, &page)
            .await?
            .edges
            .into_iter()
            .map(|e| e.node.name)
            .collect()
    } else {
        genres.to_vec()
    };

    let rows = scoped!(
        query!,
        scope,
        r#"SELECT g AS "genre!",
                  date_trunc($8, p.ts AT TIME ZONE $9) AT TIME ZONE $9 AS "start!",
                  (sum(p.ms_played) / 60000)::int AS "minutes!"
           FROM scoped_plays($1, $2, $3, $4, $5, $6, $7) p
           JOIN item_artists ia ON ia.item_uri = p.item_uri AND ia.position = 0
           JOIN spotify_artists sa ON sa.id = ia.artist_id
           CROSS JOIN LATERAL unnest(sa.genres) AS g
           WHERE g = ANY($10)
           GROUP BY 1, 2
           ORDER BY 2"#,
        bucket,
        scope.time_zone,
        &genres,
    )
    .fetch_all(pool)
    .await?;

    Ok(genres
        .iter()
        .map(|genre| GenreSeries {
            genre: genre.clone(),
            points: rows
                .iter()
                .filter(|r| &r.genre == genre)
                .map(|r| GenrePoint {
                    start: r.start,
                    minutes_played: r.minutes,
                })
                .collect(),
        })
        .collect())
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyMusicAge")]
pub struct MusicAge {
    /// Average years between a track's release and when you played it.
    pub average_years: Option<f64>,
    /// Share of plays of music released the same year it was played.
    pub new_share: Option<f64>,
    /// Share of plays with a known release year (grows as enrichment runs).
    pub coverage: f64,
    /// The oldest track played in the range.
    pub oldest: Option<Item>,
    pub oldest_year: Option<i32>,
}

pub async fn music_age(pool: &PgPool, scope: &Scope) -> sqlx::Result<MusicAge> {
    let row = scoped!(
        query!,
        scope,
        r#"SELECT
               avg(extract(year FROM p.ts AT TIME ZONE $8) - al.release_year)::float8 AS average_years,
               avg(CASE WHEN extract(year FROM p.ts AT TIME ZONE $8) = al.release_year THEN 1.0
                        WHEN al.release_year IS NOT NULL THEN 0.0 END)::float8 AS new_share,
               coalesce(count(al.release_year)::float8 / nullif(count(*), 0), 0) AS "coverage!"
           FROM scoped_plays($1, $2, $3, $4, $5, $6, $7) p
           LEFT JOIN item_meta m ON m.uri = p.item_uri
           LEFT JOIN spotify_albums al ON al.id = m.album_id"#,
        scope.time_zone,
    )
    .fetch_one(pool)
    .await?;

    let oldest = scoped!(
        query!,
        scope,
        r#"SELECT i.uri, i.kind, i.name, i.artist, i.album, al.release_year AS "year!"
           FROM scoped_plays($1, $2, $3, $4, $5, $6, $7) p
           JOIN item_meta m ON m.uri = p.item_uri
           JOIN spotify_albums al ON al.id = m.album_id
           JOIN items i ON i.uri = p.item_uri
           WHERE al.release_year IS NOT NULL
           ORDER BY al.release_date, i.uri
           LIMIT 1"#
    )
    .fetch_optional(pool)
    .await?;

    Ok(MusicAge {
        average_years: row.average_years,
        new_share: row.new_share,
        coverage: row.coverage,
        oldest_year: oldest.as_ref().map(|o| o.year),
        oldest: oldest.map(|o| item(o.uri, o.kind, o.name, o.artist, o.album)),
    })
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyCompletion")]
pub struct Completion {
    /// Average share of each track played, capped at 100%.
    pub average: Option<f64>,
    /// Share of plays that reached 90% of the track.
    pub finished_share: Option<f64>,
    /// Share of plays with a known duration (grows as enrichment runs).
    pub coverage: f64,
}

pub async fn completion(pool: &PgPool, scope: &Scope) -> sqlx::Result<Completion> {
    // Every play counts, whatever the filter's minimum length: skips are the point here.
    let row = scoped!(
        query!,
        scope,
        r#"SELECT
               avg(least(p.ms_played::float8 / nullif(m.duration_ms, 0), 1)) AS average,
               avg(CASE WHEN m.duration_ms > 0
                        THEN (p.ms_played >= m.duration_ms * 0.9)::int END)::float8 AS finished_share,
               coalesce(count(m.duration_ms)::float8 / nullif(count(*), 0), 0) AS "coverage!"
           FROM scoped_plays($1, $2, $3, $4, least($5, 0), $6, $7) p
           LEFT JOIN item_meta m ON m.uri = p.item_uri AND m.status = 'ok'"#
    )
    .fetch_one(pool)
    .await?;

    Ok(Completion {
        average: row.average,
        finished_share: row.finished_share,
        coverage: row.coverage,
    })
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyAbandonedItem")]
pub struct AbandonedItem {
    pub item: Item,
    pub plays: i32,
    /// Average share of the track played.
    pub average_completion: f64,
}

pub async fn most_abandoned(
    pool: &PgPool,
    scope: &Scope,
    first: i32,
    min_plays: i32,
) -> sqlx::Result<Vec<AbandonedItem>> {
    let rows = scoped!(
        query!,
        scope,
        r#"SELECT i.uri, i.kind, i.name, i.artist, i.album, count(*)::int AS "plays!",
                  avg(least(p.ms_played::float8 / m.duration_ms, 1)) AS "completion!"
           FROM scoped_plays($1, $2, $3, $4, least($5, 0), $6, $7) p
           JOIN item_meta m ON m.uri = p.item_uri AND m.status = 'ok' AND m.duration_ms > 0
           JOIN items i ON i.uri = p.item_uri
           GROUP BY i.uri
           HAVING count(*) >= $8
           ORDER BY 7, count(*) DESC, i.uri
           LIMIT $9"#,
        min_plays.max(1) as i64,
        first.clamp(1, 100) as i64,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| AbandonedItem {
            item: item(r.uri, r.kind, r.name, r.artist, r.album),
            plays: r.plays,
            average_completion: r.completion,
        })
        .collect())
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyEnrichment")]
pub struct Enrichment {
    pub tracks_enriched: i32,
    pub tracks_pending: i32,
    /// Tracks Spotify no longer has.
    pub tracks_missing: i32,
    pub artists_enriched: i32,
    pub artists_pending: i32,
    pub artwork_stored: i32,
    /// Share of all listening time on enriched items.
    pub listening_coverage: f64,
    #[graphql(name = "requestsLast24h")]
    pub requests_last_24h: i32,
    pub last_run: Option<DateTime<Utc>>,
    pub rate_limited_until: Option<DateTime<Utc>>,
}

pub async fn enrichment(pool: &PgPool) -> sqlx::Result<Enrichment> {
    let row = sqlx::query!(
        r#"SELECT
               (SELECT count(*)::int FROM item_meta WHERE status = 'ok') AS "tracks_enriched!",
               (SELECT count(*)::int FROM item_meta WHERE status = 'not_found') AS "tracks_missing!",
               (SELECT count(*)::int FROM enrich_queue WHERE target = 'item') AS "tracks_pending!",
               (SELECT count(*)::int FROM spotify_artists WHERE status = 'ok') AS "artists_enriched!",
               (SELECT count(*)::int FROM enrich_queue WHERE target = 'artist') AS "artists_pending!",
               ((SELECT count(*) FROM spotify_albums WHERE image_key IS NOT NULL)
                 + (SELECT count(*) FROM spotify_artists WHERE image_key IS NOT NULL))::int AS "artwork!",
               (SELECT coalesce(sum(p.ms_played) FILTER (WHERE m.uri IS NOT NULL)::float8
                                / nullif(sum(p.ms_played), 0), 0)
                FROM plays p LEFT JOIN item_meta m ON m.uri = p.item_uri AND m.status = 'ok') AS "coverage!",
               (SELECT coalesce(sum(api_requests), 0)::int FROM enrich_runs
                WHERE started_at > now() - interval '24 hours') AS "requests!",
               (SELECT max(started_at) FROM enrich_runs) AS last_run,
               (SELECT max(rate_limited_until) FROM enrich_runs WHERE rate_limited_until > now())
                   AS rate_limited_until"#
    )
    .fetch_one(pool)
    .await?;

    Ok(Enrichment {
        tracks_enriched: row.tracks_enriched,
        tracks_pending: row.tracks_pending,
        tracks_missing: row.tracks_missing,
        artists_enriched: row.artists_enriched,
        artists_pending: row.artists_pending,
        artwork_stored: row.artwork,
        listening_coverage: row.coverage,
        requests_last_24h: row.requests,
        last_run: row.last_run,
        rate_limited_until: row.rate_limited_until,
    })
}
