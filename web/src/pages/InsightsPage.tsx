import type { ReactNode } from "react";
import { graphql, useLazyLoadQuery } from "react-relay";
import { Link } from "react-router";
import type { InsightsPageQuery } from "./__generated__/InsightsPageQuery.graphql";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import ShareList from "@/components/ShareList";
import { useFilter, type StatsFilter } from "@/lib/filter";
import { formatDate, formatDay, formatDuration, formatNumber, formatPercent } from "@/lib/format";
import { artistPath, itemPath } from "@/lib/paths";

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

function Empty() {
  return <p className="text-muted-foreground text-sm">Nothing in this range.</p>;
}

function Row({ title, subtitle, value }: { title: ReactNode; subtitle?: ReactNode; value: ReactNode }) {
  return (
    <li className="flex items-center justify-between gap-3 py-2">
      <div className="min-w-0">
        <div className="truncate font-medium">{title}</div>
        {subtitle && <div className="text-muted-foreground truncate text-sm">{subtitle}</div>}
      </div>
      <div className="shrink-0 text-right text-sm tabular-nums">{value}</div>
    </li>
  );
}

const regions = new Intl.DisplayNames(undefined, { type: "region" });

/** "AU" → "Australia"; leaves anything Intl doesn't know (e.g. "ZZ") as is. */
function countryName(code: string): string {
  try {
    return regions.of(code) ?? code;
  } catch {
    return code;
  }
}

export default function InsightsPage() {
  const { filter } = useFilter();
  // Skip rates need every play, including the short ones the default filter drops.
  const allPlays: StatsFilter = { ...filter, minMsPlayed: 0 };

  const data = useLazyLoadQuery<InsightsPageQuery>(
    graphql`
      query InsightsPageQuery($filter: StatsFilter!, $allPlays: StatsFilter!) {
        stats(filter: $filter) {
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
            }
          }
          discoveries(first: 10) {
            edges {
              node {
                artist {
                  id
                  name
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
        skips: stats(filter: $allPlays) {
          mostSkipped(first: 8) {
            plays
            skips
            skipRate
            item {
              id
              uri
              name
              artistName
            }
          }
        }
      }
    `,
    { filter, allPlays },
  );

  const s = data.stats;
  const days = s.listeningDays;
  const sessions = s.sessions;

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
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
          label="Longest session"
          value={sessions.longest ? formatDuration(sessions.longest.minutesPlayed) : "–"}
          detail={
            sessions.longest
              ? `${formatDate(sessions.longest.start)} · ${sessions.longest.plays} plays`
              : undefined
          }
        />
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <Section title="Discoveries" description="Artists first heard in this range">
          {s.discoveries.edges.length === 0 ? (
            <Empty />
          ) : (
            <ol className="divide-y">
              {s.discoveries.edges.map(({ node: d }) => (
                <Row
                  key={d.artist.id}
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

        <Section title="Forgotten favourites" description="Big artists with nothing in the last year of this range">
          {s.forgottenArtists.edges.length === 0 ? (
            <Empty />
          ) : (
            <ol className="divide-y">
              {s.forgottenArtists.edges.map(({ node: f }) => (
                <Row
                  key={f.artist.id}
                  title={<Link to={artistPath(f.artist.name)} className="hover:underline">{f.artist.name}</Link>}
                  subtitle={`Last played ${formatDate(f.lastPlayed)}`}
                  value={formatDuration(f.minutesPlayed)}
                />
              ))}
            </ol>
          )}
        </Section>

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
