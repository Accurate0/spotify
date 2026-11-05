//! Query commands: summary, top lists, timeline, heatmap, search, track and artist.

use anyhow::{Context, bail};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use clap::{Args, ValueEnum};
use cynic::QueryBuilder;

use crate::client::Client;
use crate::graphql::{AccountsQuery, schema};

#[derive(Args, Clone)]
pub struct FilterArgs {
    /// Account slug to include; repeat for several. Defaults to all accounts.
    #[arg(long, short)]
    account: Vec<String>,
    /// Calendar year, in --tz.
    #[arg(long, conflicts_with_all = ["from", "to"])]
    year: Option<i32>,
    /// Start date (YYYY-MM-DD, inclusive), in --tz.
    #[arg(long)]
    from: Option<NaiveDate>,
    /// End date (YYYY-MM-DD, inclusive), in --tz.
    #[arg(long)]
    to: Option<NaiveDate>,
    /// Count every play, including skips under 30 seconds.
    #[arg(long)]
    all_plays: bool,
    /// Include podcast episodes and audiobooks alongside music.
    #[arg(long)]
    podcasts: bool,
    /// Time zone for dates, timelines and the heatmap.
    #[arg(long, env = "SPOTIFY_TZ", default_value = "Australia/Perth")]
    tz: Tz,
}

impl FilterArgs {
    pub async fn resolve(&self, client: &Client) -> anyhow::Result<StatsFilter> {
        let accounts = if self.account.is_empty() {
            None
        } else {
            let all = client.run(AccountsQuery::build(())).await?.spotify_accounts;
            let ids = self
                .account
                .iter()
                .map(|slug| {
                    all.iter()
                        .find(|a| &a.slug == slug)
                        .map(|a| a.id.clone())
                        .with_context(|| format!("no account {slug:?}"))
                })
                .collect::<anyhow::Result<_>>()?;
            Some(ids)
        };

        let (from, to) = self.time_range()?;

        Ok(StatsFilter {
            accounts,
            from,
            to,
            kinds: self
                .podcasts
                .then(|| vec![ItemKind::Track, ItemKind::Episode, ItemKind::Audiobook]),
            min_ms_played: self.all_plays.then_some(0),
            time_zone: Some(self.tz.name().to_owned()),
        })
    }
}

impl FilterArgs {
    pub fn tz(&self) -> Tz {
        self.tz
    }
}

type TimeRange = (Option<DateTime<Utc>>, Option<DateTime<Utc>>);

impl FilterArgs {
    /// `[from, to)` in UTC: local midnights in `--tz`. `--to` is inclusive on the command
    /// line, exclusive in the API.
    fn time_range(&self) -> anyhow::Result<TimeRange> {
        let start_of = |date: NaiveDate| -> anyhow::Result<DateTime<Utc>> {
            self.tz
                .from_local_datetime(&date.and_hms_opt(0, 0, 0).expect("midnight is valid"))
                .earliest()
                .map(|d| d.with_timezone(&Utc))
                .with_context(|| format!("{date} has no midnight in {}", self.tz))
        };

        let (from, to) = match self.year {
            Some(year) => (
                Some(NaiveDate::from_ymd_opt(year, 1, 1).context("invalid year")?),
                Some(NaiveDate::from_ymd_opt(year + 1, 1, 1).context("invalid year")?),
            ),
            None => (self.from, self.to.and_then(|d| d.succ_opt())),
        };

        Ok((
            from.map(start_of).transpose()?,
            to.map(start_of).transpose()?,
        ))
    }
}

#[derive(cynic::InputObject, Clone, Debug)]
#[cynic(graphql_type = "SpotifyStatsFilter")]
pub struct StatsFilter {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<cynic::Id>>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<ItemKind>>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub min_ms_played: Option<i32>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "SpotifyItemKind")]
pub enum ItemKind {
    Track,
    Episode,
    Audiobook,
}

#[derive(cynic::Enum, Clone, Copy, Debug, ValueEnum, Default)]
#[cynic(graphql_type = "SpotifyRankBy")]
pub enum RankBy {
    #[default]
    MinutesPlayed,
    Plays,
}

#[derive(cynic::Enum, Clone, Copy, Debug, ValueEnum)]
#[cynic(graphql_type = "SpotifyTimeBucket")]
pub enum TimeBucket {
    Day,
    Week,
    Month,
    Year,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifySummary")]
pub struct Summary {
    pub plays: i32,
    pub minutes_played: i32,
    pub distinct_tracks: i32,
    pub distinct_artists: i32,
    pub skip_rate: Option<f64>,
    pub first_played: Option<DateTime<Utc>>,
    pub last_played: Option<DateTime<Utc>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyArtist")]
pub struct Artist {
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyItem")]
pub struct Item {
    pub uri: String,
    pub name: Option<String>,
    pub artist_name: Option<String>,
    pub album: Option<AlbumName>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyAlbum")]
pub struct AlbumName {
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyAlbum")]
pub struct Album {
    pub name: String,
    pub artist: Option<Artist>,
}

#[derive(cynic::QueryVariables)]
pub struct FilterVariables {
    pub filter: StatsFilter,
}

#[derive(cynic::QueryVariables)]
pub struct RankVariables {
    pub filter: StatsFilter,
    pub first: i32,
    pub by: RankBy,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyStats")]
pub struct SummaryStats {
    pub summary: Summary,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FilterVariables")]
pub struct SummaryQuery {
    #[arguments(filter: $filter)]
    pub spotify_stats: SummaryStats,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyArtistRankedEdge")]
pub struct RankedArtistEdge {
    pub rank: i32,
    pub plays: i32,
    pub minutes_played: i32,
    pub node: Artist,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyArtistRankedConnection")]
pub struct RankedArtistConnection {
    pub edges: Vec<RankedArtistEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyStats", variables = "RankVariables")]
pub struct TopArtistsStats {
    #[arguments(first: $first, by: $by)]
    pub top_artists: RankedArtistConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "RankVariables")]
pub struct TopArtistsQuery {
    #[arguments(filter: $filter)]
    pub spotify_stats: TopArtistsStats,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyItemRankedEdge")]
pub struct RankedItemEdge {
    pub rank: i32,
    pub plays: i32,
    pub minutes_played: i32,
    pub node: Item,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyItemRankedConnection")]
pub struct RankedItemConnection {
    pub edges: Vec<RankedItemEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyStats", variables = "RankVariables")]
pub struct TopItemsStats {
    #[arguments(first: $first, by: $by)]
    pub top_items: RankedItemConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "RankVariables")]
pub struct TopItemsQuery {
    #[arguments(filter: $filter)]
    pub spotify_stats: TopItemsStats,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyAlbumRankedEdge")]
pub struct RankedAlbumEdge {
    pub rank: i32,
    pub plays: i32,
    pub minutes_played: i32,
    pub node: Album,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyAlbumRankedConnection")]
pub struct RankedAlbumConnection {
    pub edges: Vec<RankedAlbumEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyStats", variables = "RankVariables")]
pub struct TopAlbumsStats {
    #[arguments(first: $first, by: $by)]
    pub top_albums: RankedAlbumConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "RankVariables")]
pub struct TopAlbumsQuery {
    #[arguments(filter: $filter)]
    pub spotify_stats: TopAlbumsStats,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyGenre")]
pub struct Genre {
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyGenreRankedEdge")]
pub struct RankedGenreEdge {
    pub rank: i32,
    pub plays: i32,
    pub minutes_played: i32,
    pub node: Genre,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyGenreRankedConnection")]
pub struct RankedGenreConnection {
    pub edges: Vec<RankedGenreEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyStats", variables = "RankVariables")]
pub struct TopGenresStats {
    #[arguments(first: $first, by: $by)]
    pub top_genres: RankedGenreConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "RankVariables")]
pub struct TopGenresQuery {
    #[arguments(filter: $filter)]
    pub spotify_stats: TopGenresStats,
}

#[derive(cynic::QueryVariables)]
pub struct TimelineVariables {
    pub filter: StatsFilter,
    pub bucket: TimeBucket,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyTimelinePoint")]
pub struct TimelinePoint {
    pub start: DateTime<Utc>,
    pub plays: i32,
    pub minutes_played: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyStats", variables = "TimelineVariables")]
pub struct TimelineStats {
    #[arguments(bucket: $bucket)]
    pub timeline: Vec<TimelinePoint>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "TimelineVariables")]
pub struct TimelineQuery {
    #[arguments(filter: $filter)]
    pub spotify_stats: TimelineStats,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyHeatmapCell")]
pub struct HeatmapCell {
    pub day_of_week: i32,
    pub hour: i32,
    pub minutes_played: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyStats")]
pub struct HeatmapStats {
    pub heatmap: Vec<HeatmapCell>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FilterVariables")]
pub struct HeatmapQuery {
    #[arguments(filter: $filter)]
    pub spotify_stats: HeatmapStats,
}

#[derive(cynic::QueryVariables)]
pub struct SearchVariables {
    pub query: String,
    pub first: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyItemEdge")]
pub struct ItemEdge {
    pub node: Item,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyItemConnection")]
pub struct ItemConnection {
    pub edges: Vec<ItemEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyArtistEdge")]
pub struct ArtistEdge {
    pub node: Artist,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyArtistConnection")]
pub struct ArtistConnection {
    pub edges: Vec<ArtistEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifySearch", variables = "SearchVariables")]
pub struct Search {
    #[arguments(first: $first)]
    pub items: ItemConnection,
    #[arguments(first: $first)]
    pub artists: ArtistConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "SearchVariables")]
pub struct SearchQuery {
    #[arguments(query: $query)]
    pub spotify_search: Search,
}

#[derive(cynic::QueryVariables)]
pub struct ItemVariables {
    pub uri: String,
    pub filter: StatsFilter,
    pub first: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyAccount")]
pub struct AccountSlug {
    pub slug: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyPlay")]
pub struct Play {
    pub ts: DateTime<Utc>,
    pub ms_played: i32,
    pub skipped: Option<bool>,
    pub account: AccountSlug,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyPlayEdge")]
pub struct PlayEdge {
    pub node: Play,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyPlayConnection")]
pub struct PlayConnection {
    pub edges: Vec<PlayEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyStats", variables = "ItemVariables")]
pub struct ItemStats {
    pub summary: Summary,
    pub completion: crate::insights::Completion,
    pub listening_days: crate::insights::StreakDays,
    #[arguments(first: 1)]
    pub top_days: Vec<crate::insights::PeakDay>,
    #[arguments(first: $first)]
    pub plays: PlayConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyItem", variables = "ItemVariables")]
pub struct ItemDetail {
    pub uri: String,
    pub name: Option<String>,
    pub artist_name: Option<String>,
    pub album: Option<Album>,
    pub duration_ms: Option<i32>,
    pub release_year: Option<i32>,
    #[arguments(filter: $filter)]
    pub stats: ItemStats,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ItemVariables")]
pub struct ItemQuery {
    #[arguments(uri: $uri)]
    pub spotify_item: Option<ItemDetail>,
}

#[derive(cynic::QueryVariables)]
pub struct ArtistVariables {
    pub name: String,
    pub filter: StatsFilter,
    pub first: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyItemRankedConnection")]
pub struct ArtistTopItems {
    pub edges: Vec<RankedItemEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyStats", variables = "ArtistVariables")]
pub struct ArtistStats {
    pub summary: Summary,
    pub first_play: Option<crate::insights::FirstPlay>,
    pub listening_days: crate::insights::StreakDays,
    #[arguments(first: 1)]
    pub top_days: Vec<crate::insights::PeakDay>,
    #[arguments(first: $first)]
    pub top_items: ArtistTopItems,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyArtist", variables = "ArtistVariables")]
pub struct ArtistDetail {
    pub name: String,
    #[arguments(filter: $filter)]
    pub stats: ArtistStats,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ArtistVariables")]
pub struct ArtistQuery {
    #[arguments(name: $name)]
    pub spotify_artist: Option<ArtistDetail>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum TopKind {
    Artists,
    Tracks,
    Albums,
    /// From Spotify artist genres; needs enrichment.
    Genres,
}

pub async fn summary(client: &Client, filter: &FilterArgs) -> anyhow::Result<()> {
    let filter = filter.resolve(client).await?;
    let summary = client
        .run(SummaryQuery::build(FilterVariables { filter }))
        .await?
        .spotify_stats
        .summary;
    print_summary(&summary);
    Ok(())
}

pub async fn top(
    client: &Client,
    kind: TopKind,
    filter: &FilterArgs,
    limit: i32,
    by: RankBy,
) -> anyhow::Result<()> {
    let vars = RankVariables {
        filter: filter.resolve(client).await?,
        first: limit,
        by,
    };

    let rows: Vec<(i32, i32, i32, String)> = match kind {
        TopKind::Artists => client
            .run(TopArtistsQuery::build(vars))
            .await?
            .spotify_stats
            .top_artists
            .edges
            .into_iter()
            .map(|e| (e.rank, e.plays, e.minutes_played, e.node.name))
            .collect(),
        TopKind::Tracks => client
            .run(TopItemsQuery::build(vars))
            .await?
            .spotify_stats
            .top_items
            .edges
            .into_iter()
            .map(|e| (e.rank, e.plays, e.minutes_played, item_label(&e.node)))
            .collect(),
        TopKind::Albums => client
            .run(TopAlbumsQuery::build(vars))
            .await?
            .spotify_stats
            .top_albums
            .edges
            .into_iter()
            .map(|e| {
                let label = match e.node.artist {
                    Some(artist) => format!("{} — {}", e.node.name, artist.name),
                    None => e.node.name,
                };
                (e.rank, e.plays, e.minutes_played, label)
            })
            .collect(),
        TopKind::Genres => client
            .run(TopGenresQuery::build(vars))
            .await?
            .spotify_stats
            .top_genres
            .edges
            .into_iter()
            .map(|e| (e.rank, e.plays, e.minutes_played, e.node.name))
            .collect(),
    };

    if rows.is_empty() {
        println!("no plays match");
    }
    for (rank, plays, minutes, label) in rows {
        println!(
            "{rank:>4}  {:>11}  {plays:>6} plays  {label}",
            duration(minutes)
        );
    }
    Ok(())
}

pub async fn timeline(
    client: &Client,
    filter: &FilterArgs,
    bucket: TimeBucket,
) -> anyhow::Result<()> {
    let tz = filter.tz;
    let points = client
        .run(TimelineQuery::build(TimelineVariables {
            filter: filter.resolve(client).await?,
            bucket,
        }))
        .await?
        .spotify_stats
        .timeline;

    let max = points
        .iter()
        .map(|p| p.minutes_played)
        .max()
        .unwrap_or(0)
        .max(1);
    for point in points {
        let start = point.start.with_timezone(&tz);
        let label = match bucket {
            TimeBucket::Year => start.format("%Y").to_string(),
            TimeBucket::Month => start.format("%Y-%m").to_string(),
            TimeBucket::Week | TimeBucket::Day => start.format("%Y-%m-%d").to_string(),
        };
        let width = (point.minutes_played as i64 * 40 / max as i64) as usize;
        println!(
            "{label:<10}  {:>11}  {:>6} plays  {}",
            duration(point.minutes_played),
            point.plays,
            "█".repeat(width)
        );
    }
    Ok(())
}

pub async fn heatmap(client: &Client, filter: &FilterArgs) -> anyhow::Result<()> {
    let cells = client
        .run(HeatmapQuery::build(FilterVariables {
            filter: filter.resolve(client).await?,
        }))
        .await?
        .spotify_stats
        .heatmap;

    let mut grid = [[0i32; 24]; 7];
    for cell in &cells {
        grid[(cell.day_of_week - 1) as usize][cell.hour as usize] = cell.minutes_played;
    }
    let max = cells
        .iter()
        .map(|c| c.minutes_played)
        .max()
        .unwrap_or(0)
        .max(1);
    let shades = [' ', '░', '▒', '▓', '█'];

    println!(
        "     {}",
        (0..24).map(|h| format!("{h:<2}")).collect::<String>()
    );
    for (day, row) in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .iter()
        .zip(grid)
    {
        let line: String = row
            .iter()
            .map(|&m| {
                let shade = shades[(m as usize * (shades.len() - 1)).div_ceil(max as usize)];
                format!("{shade}{shade}")
            })
            .collect();
        println!("{day}  {line}");
    }
    Ok(())
}

pub async fn search(client: &Client, query: &str, limit: i32) -> anyhow::Result<()> {
    let search = client
        .run(SearchQuery::build(SearchVariables {
            query: query.to_owned(),
            first: limit,
        }))
        .await?
        .spotify_search;

    if !search.artists.edges.is_empty() {
        println!("Artists");
        for edge in search.artists.edges {
            println!("  {}", edge.node.name);
        }
    }
    if !search.items.edges.is_empty() {
        println!("Tracks and episodes");
        for edge in search.items.edges {
            println!("  {:<60}  {}", item_label(&edge.node), edge.node.uri);
        }
    }
    Ok(())
}

/// Shows a track or episode, by URI or by the best search match.
pub async fn track(client: &Client, query: &str, filter: &FilterArgs) -> anyhow::Result<()> {
    let uri = if query.starts_with("spotify:") {
        query.to_owned()
    } else {
        let found = client
            .run(SearchQuery::build(SearchVariables {
                query: query.to_owned(),
                first: 1,
            }))
            .await?
            .spotify_search
            .items
            .edges
            .into_iter()
            .next()
            .with_context(|| format!("no track matches {query:?}"))?;
        found.node.uri
    };

    let Some(item) = client
        .run(ItemQuery::build(ItemVariables {
            uri: uri.clone(),
            filter: filter.resolve(client).await?,
            first: 10,
        }))
        .await?
        .spotify_item
    else {
        bail!("no item {uri}");
    };

    println!(
        "{}",
        item_label(&Item {
            uri: item.uri.clone(),
            name: item.name,
            artist_name: item.artist_name,
            album: None,
        })
    );
    if let Some(album) = item.album {
        println!("{}", album.name);
    }
    println!("{}\n", item.uri);
    print_summary(&item.stats.summary);
    if let Some(ms) = item.duration_ms {
        println!("Duration        {}:{:02}", ms / 60_000, ms / 1000 % 60);
    }
    if let Some(year) = item.release_year {
        println!("Released        {year}");
    }
    if let Some(average) = item.stats.completion.average {
        println!(
            "Completion      {:.0}% on average, {:.0}% of plays finished",
            average * 100.0,
            item.stats.completion.finished_share.unwrap_or(0.0) * 100.0
        );
    }
    crate::insights::print_firsts(
        None,
        &item.stats.listening_days,
        &item.stats.top_days,
        filter.tz,
    );

    if !item.stats.plays.edges.is_empty() {
        println!("\nRecent plays");
        for edge in item.stats.plays.edges {
            let play = edge.node;
            println!(
                "  {}  {:>7}  {:<8}{}",
                play.ts.with_timezone(&filter.tz).format("%Y-%m-%d %H:%M"),
                format!("{}s", play.ms_played / 1000),
                play.account.slug,
                if play.skipped == Some(true) {
                    "  skipped"
                } else {
                    ""
                }
            );
        }
    }
    Ok(())
}

pub async fn artist(
    client: &Client,
    name: &str,
    filter: &FilterArgs,
    limit: i32,
) -> anyhow::Result<()> {
    let Some(artist) = client
        .run(ArtistQuery::build(ArtistVariables {
            name: name.to_owned(),
            filter: filter.resolve(client).await?,
            first: limit,
        }))
        .await?
        .spotify_artist
    else {
        bail!("no artist named {name:?}, try `spotify search {name}`");
    };

    println!("{}\n", artist.name);
    print_summary(&artist.stats.summary);
    crate::insights::print_firsts(
        artist.stats.first_play.as_ref(),
        &artist.stats.listening_days,
        &artist.stats.top_days,
        filter.tz,
    );

    if !artist.stats.top_items.edges.is_empty() {
        println!("\nTop tracks");
        for e in artist.stats.top_items.edges {
            println!(
                "{:>4}  {:>11}  {:>6} plays  {}",
                e.rank,
                duration(e.minutes_played),
                e.plays,
                match (&e.node.name, &e.node.album) {
                    (Some(name), Some(album)) => format!("{name}  ({})", album.name),
                    (name, _) => name.clone().unwrap_or_else(|| e.node.uri.clone()),
                }
            );
        }
    }
    Ok(())
}

fn print_summary(summary: &Summary) {
    println!("Listening time  {}", duration(summary.minutes_played));
    println!("Plays           {}", summary.plays);
    println!("Tracks          {}", summary.distinct_tracks);
    println!("Artists         {}", summary.distinct_artists);
    if let Some(rate) = summary.skip_rate {
        println!("Skip rate       {:.1}%", rate * 100.0);
    }
    if let (Some(first), Some(last)) = (summary.first_played, summary.last_played) {
        println!(
            "Between         {} and {} ({} days)",
            first.format("%Y-%m-%d"),
            last.format("%Y-%m-%d"),
            (last.date_naive() - first.date_naive()).num_days()
        );
    }
}

pub fn item_label(item: &Item) -> String {
    match (&item.name, &item.artist_name) {
        (Some(name), Some(artist)) => format!("{name} — {artist}"),
        (Some(name), None) => name.clone(),
        _ => item.uri.clone(),
    }
}

/// Minutes as `1d 2h 3m`, dropping leading zero units.
pub fn duration(minutes: i32) -> String {
    let (days, hours, mins) = (minutes / 1440, minutes % 1440 / 60, minutes % 60);
    match (days, hours) {
        (0, 0) => format!("{mins}m"),
        (0, _) => format!("{hours}h {mins:02}m"),
        _ => format!("{days}d {hours:02}h {mins:02}m"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_durations() {
        assert_eq!(duration(5), "5m");
        assert_eq!(duration(65), "1h 05m");
        assert_eq!(duration(1440 * 3 + 61), "3d 01h 01m");
    }

    fn args(year: Option<i32>, from: Option<&str>, to: Option<&str>) -> FilterArgs {
        FilterArgs {
            account: vec![],
            year,
            from: from.map(|d| d.parse().unwrap()),
            to: to.map(|d| d.parse().unwrap()),
            all_plays: false,
            podcasts: false,
            tz: chrono_tz::Australia::Perth,
        }
    }

    fn range(args: FilterArgs) -> (String, String) {
        let (from, to) = args.time_range().unwrap();
        (from.unwrap().to_rfc3339(), to.unwrap().to_rfc3339())
    }

    #[test]
    fn year_is_local_midnight_to_midnight() {
        assert_eq!(
            range(args(Some(2024), None, None)),
            (
                "2023-12-31T16:00:00+00:00".into(),
                "2024-12-31T16:00:00+00:00".into()
            )
        );
    }

    #[test]
    fn to_date_is_inclusive() {
        assert_eq!(
            range(args(None, Some("2024-03-01"), Some("2024-03-31"))),
            (
                "2024-02-29T16:00:00+00:00".into(),
                "2024-03-31T16:00:00+00:00".into()
            )
        );
    }
}
