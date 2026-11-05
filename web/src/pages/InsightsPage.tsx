import { useState, type ReactNode } from "react";
import { graphql, useLazyLoadQuery } from "react-relay";
import { Link } from "react-router";
import type { InsightsPageQuery } from "./__generated__/InsightsPageQuery.graphql";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import ShareList from "@/components/ShareList";
import Artwork from "@/components/Artwork";
import GenreChart from "@/components/GenreChart";
import RankRow from "@/components/RankRow";
import { useFilter, type StatsFilter } from "@/lib/filter";
import { formatDate, formatDay, formatDuration, formatNumber, formatPercent } from "@/lib/format";
import { artistPath, itemPath } from "@/lib/paths";
import { cn } from "@/lib/utils";

function Tile({ label, value, detail }: { label: string; value: string; detail?: string }) {
  return (
    <div className="bg-card rounded-xl border px-5 py-4">
      <div className="text-muted-foreground text-sm">{label}</div>
      <div className="mt-1 text-2xl font-semibold tracking-tight">{value}</div>
      {detail && <div className="text-muted-foreground mt-0.5 text-xs">{detail}</div>}
    </div>
  );
}

function Section({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}

type SkipRow = {
  readonly rank: number;
  readonly plays: number;
  readonly skips: number;
  readonly skipRate: number;
  readonly artist: { readonly id: string; readonly name: string; readonly imageUrl?: string | null };
};

function ArtistSkips({ most, least }: { most: readonly SkipRow[]; least: readonly SkipRow[] }) {
  const [order, setOrder] = useState<"most" | "least">("most");
  const rows = order === "most" ? most : least;

  return (
    <div>
      <div className="bg-muted mb-3 inline-flex rounded-lg p-1 text-sm">
        {(["most", "least"] as const).map((o) => (
          <button
            key={o}
            type="button"
            onClick={() => setOrder(o)}
            aria-pressed={order === o}
            className={cn(
              "rounded-md px-3 py-1 font-medium transition-colors",
              order === o ? "bg-card text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground",
            )}
          >
            {o === "most" ? "Most skipped" : "Least skipped"}
          </button>
        ))}
      </div>
      {rows.length === 0 ? (
        <Empty />
      ) : (
        <ol className="divide-y">
          {rows.map((r) => (
            <Row
              key={r.artist.id}
              image={<Artwork url={r.artist.imageUrl} alt={r.artist.name} round />}
              title={
                <span>
                  <span className="text-muted-foreground mr-2 tabular-nums">{r.rank}</span>
                  <Link to={artistPath(r.artist.name)} className="hover:underline">{r.artist.name}</Link>
                </span>
              }
              value={
                <>
                  <div className="font-medium">{formatPercent(r.skipRate)}</div>
                  <div className="text-muted-foreground text-xs">
                    {formatNumber(r.skips)} of {formatNumber(r.plays)}
                  </div>
                </>
              }
            />
          ))}
        </ol>
      )}
    </div>
  );
}

function Empty() {
  return <p className="text-muted-foreground text-sm">Nothing in this range.</p>;
}

function Row({
  image,
  title,
  subtitle,
  value,
}: {
  image?: ReactNode;
  title: ReactNode;
  subtitle?: ReactNode;
  value: ReactNode;
}) {
  return (
    <li className="flex items-center justify-between gap-3 py-2">
      <div className="flex min-w-0 items-center gap-3">
        {image}
        <div className="min-w-0">
          <div className="truncate font-medium">{title}</div>
          {subtitle && <div className="text-muted-foreground truncate text-sm">{subtitle}</div>}
        </div>
      </div>
      <div className="shrink-0 text-right text-sm tabular-nums">{value}</div>
    </li>
  );
}

const regions = new Intl.DisplayNames(undefined, { type: "region" });

/** "AU" → "Australia"; leaves anything Intl doesn't know (e.g. "ZZ") as is. */
function countryName(code: string): string {
  // Spotify uses ZZ when it couldn't tell.
  if (code === "ZZ") return "Unknown";
  try {
    return regions.of(code) ?? code;
  } catch {
    return code;
  }
}

export default function InsightsPage() {
  const { filter, state } = useFilter();
  // Skip rates need every play, including the short ones the default filter drops.
  const allPlays: StatsFilter = { ...filter, minMsPlayed: 0 };

  const data = useLazyLoadQuery<InsightsPageQuery>(
    graphql`
      query InsightsPageQuery($filter: SpotifyStatsFilter!, $allPlays: SpotifyStatsFilter!, $bucket: SpotifyTimeBucket!) {
        spotifyEnrichment {
          listeningCoverage
        }
        spotifyStats(filter: $filter) {
          topGenres(first: 10) {
            edges {
              rank
              plays
              minutesPlayed
              node {
                name
              }
            }
          }
          genreTimeline(bucket: $bucket) {
            genre
            points {
              start
              minutesPlayed
            }
          }
          musicAge {
            averageYears
            newShare
            oldestYear
            oldest {
              name
              artistName
            }
          }
          decades: breakdown(by: DECADE) {
            key
            label
            plays
            minutesPlayed
          }
          listeningDays {
            activeDays
            spanDays
            longestStreak {
              start
              end
              days
            }
            latestStreak {
              start
              end
              days
            }
          }
          sessions {
            count
            averageMinutes
            longest {
              start
              plays
              minutesPlayed
            }
          }
          topDays(first: 8) {
            date
            plays
            minutesPlayed
            topArtist {
              name
            }
          }
          onRepeat(first: 8) {
            date
            plays
            item {
              id
              uri
              name
              artistName
              artworkUrl
            }
          }
          discoveries(first: 10) {
            edges {
              node {
                artist {
                  id
                  name
                  imageUrl
                }
                firstPlayed
                minutesPlayed
                firstItem {
                  name
                }
              }
            }
          }
          forgottenArtists(first: 10) {
            edges {
              node {
                artist {
                  id
                  name
                  imageUrl
                }
                minutesPlayed
                lastPlayed
              }
            }
          }
          device: breakdown(by: DEVICE) {
            label
            plays
            minutesPlayed
          }
          country: breakdown(by: COUNTRY) {
            label
            plays
            minutesPlayed
          }
          shuffle: breakdown(by: SHUFFLE) {
            label
            plays
            minutesPlayed
          }
          offline: breakdown(by: OFFLINE) {
            label
            plays
            minutesPlayed
          }
          endReason: breakdown(by: END_REASON) {
            label
            plays
            minutesPlayed
          }
        }
        skips: spotifyStats(filter: $allPlays) {
          completion {
            average
            finishedShare
          }
          mostAbandoned(first: 8) {
            plays
            averageCompletion
            item {
              id
              uri
              name
              artistName
              artworkUrl
            }
          }
          mostSkippedArtists: artistSkipRates(order: MOST_SKIPPED, first: 10) {
            rank
            plays
            skips
            skipRate
            artist {
              id
              name
              imageUrl
            }
          }
          leastSkippedArtists: artistSkipRates(order: LEAST_SKIPPED, first: 10) {
            rank
            plays
            skips
            skipRate
            artist {
              id
              name
              imageUrl
            }
          }
          mostSkipped(first: 8) {
            plays
            skips
            skipRate
            item {
              id
              uri
              name
              artistName
              artworkUrl
            }
          }
        }
      }
    `,
    { filter, allPlays, bucket: state.range === "all" ? "YEAR" : "MONTH" },
  );

  const s = data.spotifyStats;
  const days = s.listeningDays;
  const sessions = s.sessions;

  return (
    <div className="space-y-6">
      {data.spotifyEnrichment.listeningCoverage < 0.99 && (
        <p className="text-muted-foreground text-sm">
          Genres, release years and track lengths cover {formatPercent(data.spotifyEnrichment.listeningCoverage)} of your
          listening so far; Spotify metadata is still being fetched.
        </p>
      )}
      <div className="grid grid-cols-2 gap-3 md:grid-cols-3">
        <Tile
          label="Active days"
          value={formatNumber(days.activeDays)}
          detail={
            days.spanDays > 0
              ? `${formatPercent(days.activeDays / days.spanDays)} of ${formatNumber(days.spanDays)} days`
              : undefined
          }
        />
        <Tile
          label="Longest streak"
          value={days.longestStreak ? `${days.longestStreak.days} days` : "–"}
          detail={
            days.longestStreak
              ? `${formatDay(days.longestStreak.start)} – ${formatDay(days.longestStreak.end)}`
              : undefined
          }
        />
        <Tile
          label="Sessions"
          value={formatNumber(sessions.count)}
          detail={
            sessions.averageMinutes != null
              ? `${formatDuration(Math.round(sessions.averageMinutes))} on average`
              : undefined
          }
        />
        <Tile
          label="Music age"
          value={s.musicAge.averageYears != null ? `${s.musicAge.averageYears.toFixed(1)} years` : "–"}
          detail={
            s.musicAge.newShare != null
              ? `between release and play · ${formatPercent(s.musicAge.newShare)} of plays released that year`
              : undefined
          }
        />
        <Tile
          label="Completion"
          value={data.skips.completion.average != null ? formatPercent(data.skips.completion.average) : "–"}
          detail={
            data.skips.completion.finishedShare != null
              ? `of each track played · ${formatPercent(data.skips.completion.finishedShare)} played to the end`
              : undefined
          }
        />
        <Tile
          label="Longest session"
          value={sessions.longest ? formatDuration(sessions.longest.minutesPlayed) : "–"}
          detail={
            sessions.longest
              ? `${formatDate(sessions.longest.start)} · ${sessions.longest.plays} plays`
              : undefined
          }
        />
      </div>

      <Section
        title="Genres over time"
        description="Your all-time top genres, from the Spotify genres of each track's main artist"
      >
        <GenreChart
          genres={s.genreTimeline.map((g) => g.genre)}
          series={s.genreTimeline}
          bucket={state.range === "all" ? "YEAR" : "MONTH"}
        />
      </Section>

      <div className="grid gap-6 lg:grid-cols-2">
        <Section
          title="Top genres"
          description="A play counts toward every genre of its main artist, so genres overlap"
        >
          {s.topGenres.edges.length === 0 ? (
            <Empty />
          ) : (
            <ol>
              {s.topGenres.edges.map((e) => (
                <RankRow
                  key={e.node.name}
                  rank={e.rank}
                  title={e.node.name}
                  minutes={e.minutesPlayed}
                  plays={e.plays}
                  max={s.topGenres.edges[0].minutesPlayed}
                  by="MINUTES_PLAYED"
                />
              ))}
            </ol>
          )}
        </Section>

        <Section
          title="Release decades"
          description={
            s.musicAge.oldest
              ? `Oldest: ${s.musicAge.oldest.name ?? ""} by ${s.musicAge.oldest.artistName ?? ""} (${s.musicAge.oldestYear})`
              : "When the music you play came out"
          }
        >
          <ShareList
            rows={s.decades
              .filter((d) => d.key != null)
              .sort((a, b) => (a.key ?? "").localeCompare(b.key ?? ""))}
            limit={12}
          />
        </Section>

        <Section title="Most abandoned" description="Tracks with at least 10 plays you stop furthest from the end">
          {data.skips.mostAbandoned.length === 0 ? (
            <Empty />
          ) : (
            <ol className="divide-y">
              {data.skips.mostAbandoned.map((a) => (
                <Row
                  key={a.item.id}
                  image={<Artwork url={a.item.artworkUrl} alt={a.item.name ?? ""} />}
                  title={<Link to={itemPath(a.item.uri)} className="hover:underline">{a.item.name ?? a.item.uri}</Link>}
                  subtitle={a.item.artistName}
                  value={
                    <>
                      <div className="font-medium">{formatPercent(a.averageCompletion)}</div>
                      <div className="text-muted-foreground text-xs">played, {a.plays} plays</div>
                    </>
                  }
                />
              ))}
            </ol>
          )}
        </Section>

        <Section title="Discoveries" description="Artists first heard in this range">
          {s.discoveries.edges.length === 0 ? (
            <Empty />
          ) : (
            <ol className="divide-y">
              {s.discoveries.edges.map(({ node: d }) => (
                <Row
                  key={d.artist.id}
                  image={<Artwork url={d.artist.imageUrl} alt={d.artist.name} round />}
                  title={<Link to={artistPath(d.artist.name)} className="hover:underline">{d.artist.name}</Link>}
                  subtitle={`Since ${formatDate(d.firstPlayed)}${d.firstItem?.name ? ` via ${d.firstItem.name}` : ""}`}
                  value={formatDuration(d.minutesPlayed)}
                />
              ))}
            </ol>
          )}
        </Section>

        <Section title="Biggest days" description="Most listening in a single day, with that day's top artist">
          {s.topDays.length === 0 ? (
            <Empty />
          ) : (
            <ol className="divide-y">
              {s.topDays.map((d) => (
                <Row
                  key={d.date}
                  title={formatDay(d.date)}
                  subtitle={
                    d.topArtist && (
                      <Link to={artistPath(d.topArtist.name)} className="hover:underline">{d.topArtist.name}</Link>
                    )
                  }
                  value={
                    <>
                      <div className="font-medium">{formatDuration(d.minutesPlayed)}</div>
                      <div className="text-muted-foreground text-xs">{formatNumber(d.plays)} plays</div>
                    </>
                  }
                />
              ))}
            </ol>
          )}
        </Section>

        <Section title="On repeat" description="Most plays of one track in a single day">
          {s.onRepeat.length === 0 ? (
            <Empty />
          ) : (
            <ol className="divide-y">
              {s.onRepeat.map((r) => (
                <Row
                  key={r.item.id}
                  image={<Artwork url={r.item.artworkUrl} alt={r.item.name ?? ""} />}
                  title={<Link to={itemPath(r.item.uri)} className="hover:underline">{r.item.name ?? r.item.uri}</Link>}
                  subtitle={`${r.item.artistName ?? ""} · ${formatDay(r.date)}`}
                  value={<span className="font-medium">{r.plays}×</span>}
                />
              ))}
            </ol>
          )}
        </Section>

        <Section title="Most skipped" description="Tracks with at least 10 plays, counting every skip">
          {data.skips.mostSkipped.length === 0 ? (
            <Empty />
          ) : (
            <ol className="divide-y">
              {data.skips.mostSkipped.map((m) => (
                <Row
                  key={m.item.id}
                  image={<Artwork url={m.item.artworkUrl} alt={m.item.name ?? ""} />}
                  title={<Link to={itemPath(m.item.uri)} className="hover:underline">{m.item.name ?? m.item.uri}</Link>}
                  subtitle={m.item.artistName}
                  value={
                    <>
                      <div className="font-medium">{formatPercent(m.skipRate)}</div>
                      <div className="text-muted-foreground text-xs">
                        {m.skips} of {m.plays}
                      </div>
                    </>
                  }
                />
              ))}
            </ol>
          )}
        </Section>

        <Section title="Artist skip rates" description="Artists with at least 50 plays, counting every skip">
          <ArtistSkips most={data.skips.mostSkippedArtists} least={data.skips.leastSkippedArtists} />
        </Section>

        {/* Within a single year nothing can be a year stale, so skip the card when empty. */}
        {s.forgottenArtists.edges.length > 0 && (
          <Section title="Forgotten favourites" description="Big artists with nothing in the last year of this range">
            <ol className="divide-y">
              {s.forgottenArtists.edges.map(({ node: f }) => (
                <Row
                  key={f.artist.id}
                  image={<Artwork url={f.artist.imageUrl} alt={f.artist.name} round />}
                  title={<Link to={artistPath(f.artist.name)} className="hover:underline">{f.artist.name}</Link>}
                  subtitle={`Last played ${formatDate(f.lastPlayed)}`}
                  value={formatDuration(f.minutesPlayed)}
                />
              ))}
            </ol>
          </Section>
        )}

        <Section title="Devices">
          <ShareList rows={s.device} />
        </Section>

        <Section title="How tracks ended">
          <ShareList rows={s.endReason} limit={6} />
        </Section>

        <Section title="Countries">
          <ShareList rows={s.country.map((r) => ({ ...r, label: countryName(r.label) }))} limit={6} />
        </Section>

        <Section title="Shuffle and offline">
          <div className="space-y-6">
            <ShareList rows={s.shuffle} />
            <ShareList rows={s.offline} />
          </div>
        </Section>
      </div>
    </div>
  );
}
