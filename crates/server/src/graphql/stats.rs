//! Listening stats. Every query reads `scoped_plays(...)` (see migrations/0003_stats.sql),
//! whose first seven parameters are always the `Scope`; `scoped!` binds them so queries only
//! list their own parameters, starting at `$8`.

use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, Enum, ID, InputObject, Object, SimpleObject};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::catalog::{Album, Artist, Item};
use super::loaders::{AccountLoader, ItemLoader};
use super::node::{NodeType, decode_row_id};
use super::objects::Account;
use super::page::{Page, PlainConnection, RankedConnection};

macro_rules! scoped {
    (query_as!($ty:ident), $scope:expr, $sql:literal $(, $arg:expr)* $(,)?) => {
        sqlx::query_as!(
            $ty,
            $sql,
            &$scope.accounts,
            $scope.from,
            $scope.to,
            &$scope.kinds,
            $scope.min_ms,
            $scope.item_uri.as_deref(),
            $scope.artist.as_deref()
            $(, $arg)*
        )
    };
    (query!, $scope:expr, $sql:literal $(, $arg:expr)* $(,)?) => {
        sqlx::query!(
            $sql,
            &$scope.accounts,
            $scope.from,
            $scope.to,
            &$scope.kinds,
            $scope.min_ms,
            $scope.item_uri.as_deref(),
            $scope.artist.as_deref()
            $(, $arg)*
        )
    };
}

const DEFAULT_TIME_ZONE: &str = "Australia/Perth";
/// Spotify counts a play as a stream once it passes 30 seconds.
const DEFAULT_MIN_MS_PLAYED: i32 = 30_000;

#[derive(InputObject, Clone)]
pub struct StatsFilter {
    /// Accounts to include; empty means all.
    #[graphql(default)]
    pub accounts: Vec<ID>,
    /// Inclusive start.
    pub from: Option<DateTime<Utc>>,
    /// Exclusive end.
    pub to: Option<DateTime<Utc>>,
    /// Item kinds to include. Account-data plays have no kind and count as tracks.
    #[graphql(default_with = "vec![super::catalog::ItemKind::Track]")]
    pub kinds: Vec<super::catalog::ItemKind>,
    /// Ignore plays shorter than this. Defaults to Spotify's 30 second stream threshold; use
    /// 0 to include skips.
    #[graphql(default = 30000)]
    pub min_ms_played: i32,
    /// IANA time zone for timeline buckets and the heatmap.
    #[graphql(default_with = "DEFAULT_TIME_ZONE.to_owned()")]
    pub time_zone: String,
}

impl Default for StatsFilter {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            from: None,
            to: None,
            kinds: vec![super::catalog::ItemKind::Track],
            min_ms_played: DEFAULT_MIN_MS_PLAYED,
            time_zone: DEFAULT_TIME_ZONE.to_owned(),
        }
    }
}

impl StatsFilter {
    pub fn into_scope(self) -> async_graphql::Result<Scope> {
        let accounts = self
            .accounts
            .iter()
            .map(|id| decode_row_id(id, NodeType::Account))
            .collect::<Result<_, _>>()?;
        let kinds = self
            .kinds
            .into_iter()
            .map(|k| spotify_export::ItemKind::from(k).as_str().to_owned())
            .collect();

        Ok(Scope {
            accounts,
            from: self.from,
            to: self.to,
            kinds,
            min_ms: self.min_ms_played.max(0),
            time_zone: self.time_zone,
            item_uri: None,
            artist: None,
        })
    }
}

/// Resolved filter, plus optional narrowing to one item or artist.
#[derive(Clone)]
pub struct Scope {
    accounts: Vec<i64>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    kinds: Vec<String>,
    min_ms: i32,
    time_zone: String,
    item_uri: Option<String>,
    artist: Option<String>,
}

impl Scope {
    pub fn item(mut self, uri: &str) -> Self {
        self.item_uri = Some(uri.to_owned());
        self
    }

    pub fn artist(mut self, name: &str) -> Self {
        self.artist = Some(name.to_owned());
        self
    }
}

#[derive(Enum, Clone, Copy, PartialEq, Eq, Default)]
pub enum RankBy {
    #[default]
    MinutesPlayed,
    Plays,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum TimeBucket {
    Day,
    Week,
    Month,
    Year,
}

impl TimeBucket {
    fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
        }
    }
}

#[derive(SimpleObject)]
pub struct Summary {
    pub plays: i32,
    pub minutes_played: i32,
    pub distinct_tracks: i32,
    pub distinct_artists: i32,
    /// Share of plays marked skipped, among plays where the export records it.
    pub skip_rate: Option<f64>,
    pub first_played: Option<DateTime<Utc>>,
    pub last_played: Option<DateTime<Utc>>,
}

#[derive(SimpleObject)]
pub struct TimelinePoint {
    /// Start of the bucket in the filter's time zone.
    pub start: DateTime<Utc>,
    pub plays: i32,
    pub minutes_played: i32,
}

#[derive(SimpleObject)]
pub struct HeatmapCell {
    /// ISO day of week: 1 is Monday, 7 is Sunday.
    pub day_of_week: i32,
    pub hour: i32,
    pub plays: i32,
    pub minutes_played: i32,
}

pub struct Play {
    account_id: i64,
    ts: DateTime<Utc>,
    item_uri: Option<String>,
    name: Option<String>,
    artist: Option<String>,
    ms_played: i32,
    skipped: Option<bool>,
    is_extended: bool,
}

#[Object]
impl Play {
    /// When playback ended.
    async fn ts(&self) -> DateTime<Utc> {
        self.ts
    }

    async fn ms_played(&self) -> i32 {
        self.ms_played
    }

    async fn skipped(&self) -> Option<bool> {
        self.skipped
    }

    /// False for account-data plays, which have minute precision and no item.
    async fn is_extended(&self) -> bool {
        self.is_extended
    }

    async fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    async fn artist_name(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    async fn item(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Item>> {
        let Some(uri) = &self.item_uri else {
            return Ok(None);
        };
        Ok(ctx
            .data::<DataLoader<ItemLoader>>()?
            .load_one(uri.clone())
            .await?)
    }

    async fn account(&self, ctx: &Context<'_>) -> async_graphql::Result<Account> {
        ctx.data::<DataLoader<AccountLoader>>()?
            .load_one(self.account_id)
            .await?
            .ok_or_else(|| "account not found".into())
    }
}

pub struct Stats {
    scope: Scope,
}

impl Stats {
    pub fn new(scope: Scope) -> Self {
        Self { scope }
    }
}

#[Object]
impl Stats {
    async fn summary(&self, ctx: &Context<'_>) -> async_graphql::Result<Summary> {
        let pool = ctx.data::<PgPool>()?;
        let row = scoped!(
            query!,
            self.scope,
            // Distinct counts go through subqueries so they hash the ~20k distinct keys
            // instead of sorting every play (which spilled to disk).
            r#"WITH p AS MATERIALIZED (
                   SELECT * FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
               )
               SELECT count(*)::int AS "plays!",
                      (coalesce(sum(ms_played), 0) / 60000)::int AS "minutes!",
                      (SELECT count(*) FROM (
                          SELECT DISTINCT coalesce(item_uri, artist || E'\x1f' || name) AS track
                          FROM p
                       ) t WHERE track IS NOT NULL)::int AS "tracks!",
                      (SELECT count(*) FROM (
                          SELECT DISTINCT artist FROM p WHERE artist IS NOT NULL
                       ) a)::int AS "artists!",
                      avg(CASE WHEN skipped THEN 1.0 WHEN NOT skipped THEN 0.0 END)::float8 AS skip_rate,
                      min(ts) AS first_played,
                      max(ts) AS last_played
               FROM p"#
        )
        .fetch_one(pool)
        .await?;

        Ok(Summary {
            plays: row.plays,
            minutes_played: row.minutes,
            distinct_tracks: row.tracks,
            distinct_artists: row.artists,
            skip_rate: row.skip_rate,
            first_played: row.first_played,
            last_played: row.last_played,
        })
    }

    async fn top_artists(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
        #[graphql(default)] by: RankBy,
    ) -> async_graphql::Result<RankedConnection<Artist>> {
        let pool = ctx.data::<PgPool>()?;
        let page = Page::new(first, after, 10)?;
        let rows = scoped!(
            query!,
            self.scope,
            r#"SELECT artist AS "artist!", count(*)::int AS "plays!",
                      (sum(ms_played) / 60000)::int AS "minutes!"
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
               WHERE artist IS NOT NULL
               GROUP BY artist
               ORDER BY CASE WHEN $8 THEN count(*) ELSE sum(ms_played) END DESC, artist
               LIMIT $9 OFFSET $10"#,
            by == RankBy::Plays,
            page.fetch(),
            page.offset,
        )
        .fetch_all(pool)
        .await?;

        Ok(page.ranked(
            rows.into_iter()
                .map(|r| (Artist { name: r.artist }, r.plays, r.minutes)),
        ))
    }

    /// Only plays with an item (extended history) are ranked; account-data plays have none.
    async fn top_items(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
        #[graphql(default)] by: RankBy,
    ) -> async_graphql::Result<RankedConnection<Item>> {
        let pool = ctx.data::<PgPool>()?;
        let page = Page::new(first, after, 10)?;
        let rows = scoped!(
            query!,
            self.scope,
            r#"SELECT i.uri, i.kind, i.name, i.artist, i.album,
                      count(*)::int AS "plays!", (sum(p.ms_played) / 60000)::int AS "minutes!"
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7) p
               JOIN items i ON i.uri = p.item_uri
               GROUP BY i.uri
               ORDER BY CASE WHEN $8 THEN count(*) ELSE sum(p.ms_played) END DESC, i.uri
               LIMIT $9 OFFSET $10"#,
            by == RankBy::Plays,
            page.fetch(),
            page.offset,
        )
        .fetch_all(pool)
        .await?;

        Ok(page.ranked(rows.into_iter().map(|r| {
            let item = Item {
                uri: r.uri,
                kind: r.kind,
                name: r.name,
                artist: r.artist,
                album: r.album,
            };
            (item, r.plays, r.minutes)
        })))
    }

    async fn top_albums(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
        #[graphql(default)] by: RankBy,
    ) -> async_graphql::Result<RankedConnection<Album>> {
        let pool = ctx.data::<PgPool>()?;
        let page = Page::new(first, after, 10)?;
        let rows = scoped!(
            query!,
            self.scope,
            r#"SELECT album AS "album!", artist, count(*)::int AS "plays!",
                      (sum(ms_played) / 60000)::int AS "minutes!"
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
               WHERE album IS NOT NULL
               GROUP BY album, artist
               ORDER BY CASE WHEN $8 THEN count(*) ELSE sum(ms_played) END DESC, album, artist
               LIMIT $9 OFFSET $10"#,
            by == RankBy::Plays,
            page.fetch(),
            page.offset,
        )
        .fetch_all(pool)
        .await?;

        Ok(page.ranked(rows.into_iter().map(|r| {
            let album = Album {
                name: r.album,
                artist: r.artist,
            };
            (album, r.plays, r.minutes)
        })))
    }

    async fn timeline(
        &self,
        ctx: &Context<'_>,
        bucket: TimeBucket,
    ) -> async_graphql::Result<Vec<TimelinePoint>> {
        let pool = ctx.data::<PgPool>()?;
        let rows = scoped!(
            query!,
            self.scope,
            r#"SELECT date_trunc($8, ts AT TIME ZONE $9) AT TIME ZONE $9 AS "start!",
                      count(*)::int AS "plays!", (sum(ms_played) / 60000)::int AS "minutes!"
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
               GROUP BY 1
               ORDER BY 1"#,
            bucket.as_str(),
            self.scope.time_zone,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| TimelinePoint {
                start: r.start,
                plays: r.plays,
                minutes_played: r.minutes,
            })
            .collect())
    }

    /// Listening by day of week and hour, in the filter's time zone. Empty cells are omitted.
    async fn heatmap(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<HeatmapCell>> {
        let pool = ctx.data::<PgPool>()?;
        let rows = scoped!(
            query!,
            self.scope,
            r#"SELECT extract(isodow FROM ts AT TIME ZONE $8)::int AS "day!",
                      extract(hour FROM ts AT TIME ZONE $8)::int AS "hour!",
                      count(*)::int AS "plays!", (sum(ms_played) / 60000)::int AS "minutes!"
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
               GROUP BY 1, 2
               ORDER BY 1, 2"#,
            self.scope.time_zone,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| HeatmapCell {
                day_of_week: r.day,
                hour: r.hour,
                plays: r.plays,
                minutes_played: r.minutes,
            })
            .collect())
    }

    /// Individual plays, most recent first.
    async fn plays(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
    ) -> async_graphql::Result<PlainConnection<Play>> {
        let pool = ctx.data::<PgPool>()?;
        let page = Page::new(first, after, 20)?;
        let rows = scoped!(
            query_as!(Play),
            self.scope,
            r#"SELECT account_id AS "account_id!", ts AS "ts!", item_uri, name, artist,
                      ms_played AS "ms_played!", skipped, is_extended AS "is_extended!"
               FROM scoped_plays($1, $2, $3, $4, $5, $6, $7)
               ORDER BY ts DESC
               LIMIT $8 OFFSET $9"#,
            page.fetch(),
            page.offset,
        )
        .fetch_all(pool)
        .await?;

        Ok(page.plain(rows))
    }
}
