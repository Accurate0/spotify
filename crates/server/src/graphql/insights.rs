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
use super::page::{Page, PlainConnection};
use super::stats::{Play, Scope};

#[derive(SimpleObject)]
pub struct Streak {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub days: i32,
}

#[derive(SimpleObject)]
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
pub struct DayStat {
    pub date: NaiveDate,
    pub plays: i32,
    pub minutes_played: i32,
    /// The artist with the most listening that day.
    pub top_artist: Option<Artist>,
}

#[derive(SimpleObject)]
pub struct RepeatDay {
    pub item: Item,
    /// The day this item was played most.
    pub date: NaiveDate,
    pub plays: i32,
}

#[derive(SimpleObject)]
pub struct SkippedItem {
    pub item: Item,
    /// Plays where the export records whether it was skipped.
    pub plays: i32,
    pub skips: i32,
    pub skip_rate: f64,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
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
        }
    }
}

#[derive(SimpleObject)]
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
pub struct Session {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub plays: i32,
    pub minutes_played: i32,
}

#[derive(SimpleObject)]
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

#[Object]
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
                  END AS key,
                  count(*)::int AS "plays!",
                  (sum(ms_played) / 60000)::int AS "minutes!"
           FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
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
