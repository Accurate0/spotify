import { graphql, useFragment } from "react-relay";
import type { StatTiles_summary$key } from "./__generated__/StatTiles_summary.graphql";
import {
  formatCompact,
  formatDate,
  formatDuration,
  formatPercent,
} from "@/lib/format";
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

/**
 * KPI row: listening time leads, then counts. `scope` drops counts that are trivially 1 on
 * an artist or track page.
 */
export default function StatTiles({
  summary,
  scope = "all",
}: {
  summary: StatTiles_summary$key;
  scope?: "all" | "artist" | "item";
}) {
  const data = useFragment(
    graphql`
      fragment StatTiles_summary on SpotifySummary {
        plays
        minutesPlayed
        distinctTracks
        distinctArtists
        skipRate
        firstPlayed
        lastPlayed
      }
    `,
    summary,
  );

  const span =
    data.firstPlayed && data.lastPlayed
      ? `${formatDate(data.firstPlayed)} – ${formatDate(data.lastPlayed)}`
      : undefined;

  return (
    <div
      className={cn(
        "grid grid-cols-2 gap-3",
        scope === "all" ? "md:grid-cols-5" : scope === "artist" ? "md:grid-cols-4" : "md:grid-cols-3",
      )}
    >
      <Tile
        label="Listening time"
        value={formatDuration(data.minutesPlayed)}
        detail={`${formatCompact(Math.round(data.minutesPlayed / 60))} hours`}
      />
      <Tile label="Plays" value={formatCompact(data.plays)} detail={span} />
      {scope !== "item" && <Tile label="Tracks" value={formatCompact(data.distinctTracks)} />}
      {scope === "all" && <Tile label="Artists" value={formatCompact(data.distinctArtists)} />}
      <Tile
        label="Skip rate"
        value={data.skipRate == null ? "–" : formatPercent(data.skipRate)}
        detail="of plays marked skipped"
      />
    </div>
  );
}
