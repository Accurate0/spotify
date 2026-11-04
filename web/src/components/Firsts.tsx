import { graphql, useFragment } from "react-relay";
import { Link } from "react-router";
import type { Firsts_stats$key } from "./__generated__/Firsts_stats.graphql";
import { formatDate, formatDay, formatDuration, formatNumber } from "@/lib/format";
import { itemPath } from "@/lib/paths";

function Fact({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="min-w-0">
      <dt className="text-muted-foreground text-sm">{label}</dt>
      <dd className="mt-0.5 truncate font-medium">{children}</dd>
    </div>
  );
}

/** First listen, peak day and streaks for an artist or track page. */
export default function Firsts({ stats, showFirstTrack }: { stats: Firsts_stats$key; showFirstTrack: boolean }) {
  const data = useFragment(
    graphql`
      fragment Firsts_stats on Stats {
        firstPlay {
          ts
          item {
            uri
            name
          }
        }
        topDays(first: 1) {
          date
          plays
          minutesPlayed
        }
        listeningDays {
          activeDays
          longestStreak {
            days
            start
            end
          }
        }
      }
    `,
    stats,
  );

  const peak = data.topDays[0];
  const streak = data.listeningDays.longestStreak;

  return (
    <dl className="bg-card grid grid-cols-2 gap-4 rounded-xl border px-5 py-4 md:grid-cols-4">
      <Fact label="First listened">
        {data.firstPlay ? formatDate(data.firstPlay.ts) : "–"}
        {showFirstTrack && data.firstPlay?.item && (
          <div className="text-muted-foreground truncate text-sm font-normal">
            via{" "}
            <Link to={itemPath(data.firstPlay.item.uri)} className="hover:underline">
              {data.firstPlay.item.name ?? data.firstPlay.item.uri}
            </Link>
          </div>
        )}
      </Fact>
      <Fact label="Peak day">
        {peak ? formatDay(peak.date) : "–"}
        {peak && (
          <div className="text-muted-foreground text-sm font-normal">
            {formatDuration(peak.minutesPlayed)} · {formatNumber(peak.plays)} plays
          </div>
        )}
      </Fact>
      <Fact label="Longest streak">
        {streak ? `${streak.days} ${streak.days === 1 ? "day" : "days"}` : "–"}
        {streak && streak.days > 1 && (
          <div className="text-muted-foreground text-sm font-normal">
            {formatDay(streak.start)} – {formatDay(streak.end)}
          </div>
        )}
      </Fact>
      <Fact label="Days listened">{formatNumber(data.listeningDays.activeDays)}</Fact>
    </dl>
  );
}
