//! `spotify insights`, `spotify breakdown` and `spotify skipped`, plus the firsts and peaks
//! shown on track and artist views.

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use clap::ValueEnum;
use cynic::QueryBuilder;

use crate::client::Client;
use crate::graphql::schema;
use crate::stats::{
    Artist, FilterArgs, FilterVariables, FilterVariablesFields, Item, StatsFilter, duration,
    item_label,
};

#[derive(cynic::QueryFragment, Debug)]
pub struct Streak {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub days: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ListeningDays")]
pub struct StreakDays {
    pub active_days: i32,
    pub span_days: i32,
    pub longest_streak: Option<Streak>,
    pub latest_streak: Option<Streak>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DayStat")]
pub struct PeakDay {
    pub date: NaiveDate,
    pub plays: i32,
    pub minutes_played: i32,
    pub top_artist: Option<Artist>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Play")]
pub struct FirstPlay {
    pub ts: DateTime<Utc>,
    pub item: Option<Item>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Session {
    pub start: DateTime<Utc>,
    pub plays: i32,
    pub minutes_played: i32,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Sessions {
    pub count: i32,
    pub average_minutes: Option<f64>,
    pub longest: Option<Session>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct RepeatDay {
    pub date: NaiveDate,
    pub plays: i32,
    pub item: Item,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Discovery {
    pub artist: Artist,
    pub first_played: DateTime<Utc>,
    pub first_item: Option<Item>,
    pub minutes_played: i32,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DiscoveryEdge {
    pub node: Discovery,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DiscoveryConnection {
    pub edges: Vec<DiscoveryEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ForgottenArtist {
    pub artist: Artist,
    pub minutes_played: i32,
    pub last_played: DateTime<Utc>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ForgottenArtistEdge {
    pub node: ForgottenArtist,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ForgottenArtistConnection {
    pub edges: Vec<ForgottenArtistEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Stats")]
pub struct InsightsStats {
    pub listening_days: StreakDays,
    pub sessions: Sessions,
    #[arguments(first: 5)]
    pub top_days: Vec<PeakDay>,
    #[arguments(first: 5)]
    pub on_repeat: Vec<RepeatDay>,
    #[arguments(first: 10)]
    pub discoveries: DiscoveryConnection,
    #[arguments(first: 10)]
    pub forgotten_artists: ForgottenArtistConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FilterVariables")]
pub struct InsightsQuery {
    #[arguments(filter: $filter)]
    pub stats: InsightsStats,
}

#[derive(cynic::Enum, Clone, Copy, Debug, ValueEnum)]
pub enum BreakdownDimension {
    Device,
    Country,
    Shuffle,
    Offline,
    StartReason,
    EndReason,
    Decade,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Completion {
    pub average: Option<f64>,
    pub finished_share: Option<f64>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Enrichment {
    pub tracks_enriched: i32,
    pub tracks_pending: i32,
    pub tracks_missing: i32,
    pub artists_enriched: i32,
    pub artists_pending: i32,
    pub artwork_stored: i32,
    pub listening_coverage: f64,
    pub requests_last_24h: i32,
    pub last_run: Option<DateTime<Utc>>,
    pub rate_limited_until: Option<DateTime<Utc>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct EnrichmentQuery {
    pub enrichment: Enrichment,
}

/// Progress of the Spotify Web API backfill.
pub async fn enrichment(client: &Client, tz: Tz) -> anyhow::Result<()> {
    let e = client.run(EnrichmentQuery::build(())).await?.enrichment;
    println!(
        "Tracks     {} enriched, {} pending, {} no longer on Spotify",
        e.tracks_enriched, e.tracks_pending, e.tracks_missing
    );
    println!(
        "Artists    {} enriched, {} pending",
        e.artists_enriched, e.artists_pending
    );
    println!("Artwork    {} images stored", e.artwork_stored);
    println!(
        "Coverage   {:.1}% of listening time",
        e.listening_coverage * 100.0
    );
    println!("Requests   {} in the last 24h", e.requests_last_24h);
    if let Some(last) = e.last_run {
        println!(
            "Last run   {}",
            last.with_timezone(&tz).format("%Y-%m-%d %H:%M")
        );
    }
    if let Some(until) = e.rate_limited_until {
        println!(
            "Paused     rate limited until {}",
            until.with_timezone(&tz).format("%Y-%m-%d %H:%M")
        );
    }
    Ok(())
}

#[derive(cynic::QueryVariables)]
pub struct BreakdownVariables {
    pub filter: StatsFilter,
    pub by: BreakdownDimension,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct BreakdownRow {
    pub label: String,
    pub plays: i32,
    pub minutes_played: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Stats", variables = "BreakdownVariables")]
pub struct BreakdownStats {
    #[arguments(by: $by)]
    pub breakdown: Vec<BreakdownRow>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "BreakdownVariables")]
pub struct BreakdownQuery {
    #[arguments(filter: $filter)]
    pub stats: BreakdownStats,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum SkipOrder {
    MostSkipped,
    LeastSkipped,
}

#[derive(cynic::QueryVariables)]
pub struct ArtistSkipsVariables {
    pub filter: StatsFilter,
    pub first: i32,
    pub min_plays: i32,
    pub order: SkipOrder,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ArtistSkipRate {
    pub rank: i32,
    pub artist: Artist,
    pub plays: i32,
    pub skips: i32,
    pub skip_rate: f64,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Stats", variables = "ArtistSkipsVariables")]
pub struct ArtistSkipsStats {
    #[arguments(order: $order, first: $first, minPlays: $min_plays)]
    pub artist_skip_rates: Vec<ArtistSkipRate>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ArtistSkipsVariables")]
pub struct ArtistSkipsQuery {
    #[arguments(filter: $filter)]
    pub stats: ArtistSkipsStats,
}

#[derive(cynic::QueryVariables)]
pub struct SkippedVariables {
    pub filter: StatsFilter,
    pub first: i32,
    pub min_plays: i32,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SkippedItem {
    pub item: Item,
    pub plays: i32,
    pub skips: i32,
    pub skip_rate: f64,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Stats", variables = "SkippedVariables")]
pub struct SkippedStats {
    #[arguments(first: $first, minPlays: $min_plays)]
    pub most_skipped: Vec<SkippedItem>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "SkippedVariables")]
pub struct SkippedQuery {
    #[arguments(filter: $filter)]
    pub stats: SkippedStats,
}

fn date(ts: DateTime<Utc>, tz: Tz) -> String {
    ts.with_timezone(&tz).format("%Y-%m-%d").to_string()
}

fn streak(s: &Streak) -> String {
    format!("{} days ({} – {})", s.days, s.start, s.end)
}

/// Firsts and peaks under a track or artist summary.
pub fn print_firsts(first: Option<&FirstPlay>, days: &StreakDays, peak: &[PeakDay], tz: Tz) {
    if let Some(first) = first {
        let track = first.item.as_ref().map(item_label);
        println!(
            "First listened  {}{}",
            date(first.ts, tz),
            track.map(|t| format!(" ({t})")).unwrap_or_default()
        );
    }
    if let Some(day) = peak.first() {
        println!(
            "Peak day        {} ({}, {} plays)",
            day.date,
            duration(day.minutes_played),
            day.plays
        );
    }
    if let Some(longest) = &days.longest_streak {
        println!("Longest streak  {}", streak(longest));
    }
    println!("Active days     {}", days.active_days);
}

pub async fn insights(client: &Client, filter: &FilterArgs) -> anyhow::Result<()> {
    let tz = filter.tz();
    let stats = client
        .run(InsightsQuery::build(FilterVariables {
            filter: filter.resolve(client).await?,
        }))
        .await?
        .stats;

    let days = &stats.listening_days;
    println!("Days");
    println!(
        "  {} active of {} ({:.0}%)",
        days.active_days,
        days.span_days,
        100.0 * days.active_days as f64 / days.span_days.max(1) as f64
    );
    if let Some(s) = &days.longest_streak {
        println!("  Longest streak  {}", streak(s));
    }
    if let Some(s) = &days.latest_streak {
        println!("  Latest streak   {}", streak(s));
    }

    let sessions = &stats.sessions;
    println!("\nSessions (split on 30 minute gaps)");
    println!(
        "  {} sessions, {} on average",
        sessions.count,
        duration(sessions.average_minutes.unwrap_or(0.0).round() as i32)
    );
    if let Some(longest) = &sessions.longest {
        println!(
            "  Longest: {} on {} ({} plays)",
            duration(longest.minutes_played),
            date(longest.start, tz),
            longest.plays
        );
    }

    println!("\nBiggest days");
    for day in &stats.top_days {
        println!(
            "  {}  {:>8}  {:>4} plays  {}",
            day.date,
            duration(day.minutes_played),
            day.plays,
            day.top_artist
                .as_ref()
                .map(|a| a.name.as_str())
                .unwrap_or("")
        );
    }

    println!("\nOn repeat");
    for r in &stats.on_repeat {
        println!("  {:>3}× on {}  {}", r.plays, r.date, item_label(&r.item));
    }

    println!("\nDiscoveries (first heard in this range)");
    for e in &stats.discoveries.edges {
        let d = &e.node;
        println!(
            "  {:>8}  {:<28} since {}{}",
            duration(d.minutes_played),
            d.artist.name,
            date(d.first_played, tz),
            d.first_item
                .as_ref()
                .and_then(|i| i.name.as_deref())
                .map(|n| format!(" via {n}"))
                .unwrap_or_default()
        );
    }

    if !stats.forgotten_artists.edges.is_empty() {
        println!("\nForgotten favourites (nothing in the last year)");
        for e in &stats.forgotten_artists.edges {
            let f = &e.node;
            println!(
                "  {:>8}  {:<28} last {}",
                duration(f.minutes_played),
                f.artist.name,
                date(f.last_played, tz)
            );
        }
    }
    Ok(())
}

pub async fn breakdown(
    client: &Client,
    filter: &FilterArgs,
    by: BreakdownDimension,
) -> anyhow::Result<()> {
    let rows = client
        .run(BreakdownQuery::build(BreakdownVariables {
            filter: filter.resolve(client).await?,
            by,
        }))
        .await?
        .stats
        .breakdown;

    let total: i64 = rows
        .iter()
        .map(|r| r.minutes_played as i64)
        .sum::<i64>()
        .max(1);
    for row in rows {
        let share = row.minutes_played as f64 / total as f64;
        println!(
            "{:<28} {:>12} {:>5.1}%  {:>7} plays  {}",
            row.label,
            duration(row.minutes_played),
            share * 100.0,
            row.plays,
            "█".repeat((share * 30.0).round() as usize)
        );
    }
    Ok(())
}

/// Artist skip-rate leaderboard, most or least skipped first.
pub async fn skipped_artists(
    client: &Client,
    filter: &FilterArgs,
    limit: i32,
    min_plays: i32,
    least: bool,
) -> anyhow::Result<()> {
    let rows = client
        .run(ArtistSkipsQuery::build(ArtistSkipsVariables {
            filter: filter.resolve(client).await?,
            first: limit,
            min_plays,
            order: if least {
                SkipOrder::LeastSkipped
            } else {
                SkipOrder::MostSkipped
            },
        }))
        .await?
        .stats
        .artist_skip_rates;

    if rows.is_empty() {
        println!("no artists with at least {min_plays} plays that record skips");
    }
    for r in rows {
        println!(
            "{:>4}  {:>5.1}%  {:>5}/{:<5} skipped  {}",
            r.rank,
            r.skip_rate * 100.0,
            r.skips,
            r.plays,
            r.artist.name
        );
    }
    Ok(())
}

pub async fn skipped(
    client: &Client,
    filter: &FilterArgs,
    limit: i32,
    min_plays: i32,
) -> anyhow::Result<()> {
    let items = client
        .run(SkippedQuery::build(SkippedVariables {
            filter: filter.resolve(client).await?,
            first: limit,
            min_plays,
        }))
        .await?
        .stats
        .most_skipped;

    if items.is_empty() {
        println!("no items with at least {min_plays} plays that record skips");
    }
    for s in items {
        println!(
            "{:>5.0}%  {:>3}/{:<3} skipped  {}",
            s.skip_rate * 100.0,
            s.skips,
            s.plays,
            item_label(&s.item)
        );
    }
    Ok(())
}
