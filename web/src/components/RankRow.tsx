import type { ReactNode } from "react";
import { formatDuration, formatNumber } from "@/lib/format";

export type RankBy = "MINUTES_PLAYED" | "PLAYS";

/**
 * One ranked entry with an inline magnitude bar. The bar is scaled to the list's leader,
 * in the single sequential hue; the value is text beside it, never the bar's color.
 */
export default function RankRow({
  rank,
  image,
  title,
  subtitle,
  minutes,
  plays,
  max,
  by,
}: {
  rank: number;
  /** Artwork beside the title (see Artwork). */
  image?: ReactNode;
  title: ReactNode;
  subtitle?: ReactNode;
  minutes: number;
  plays: number;
  max: number;
  by: RankBy;
}) {
  const value = by === "PLAYS" ? plays : minutes;
  const width = max > 0 ? Math.max(1, (value / max) * 100) : 0;

  return (
    <li className="grid grid-cols-[2rem_minmax(0,1fr)_6rem] items-center gap-x-3 py-2">
      <span className="text-muted-foreground text-right text-sm tabular-nums">{rank}</span>
      <div className="flex min-w-0 items-center gap-3">
        {image}
        <div className="min-w-0 flex-1">
        <div className="truncate font-medium">{title}</div>
        {subtitle && <div className="text-muted-foreground truncate text-sm">{subtitle}</div>}
        <div className="bg-muted mt-1.5 h-1.5 rounded-full">
          <div className="bg-series-1 h-full rounded-full" style={{ width: `${width}%` }} />
        </div>
        </div>
      </div>
      <div className="text-right text-sm tabular-nums">
        <div className="font-medium">
          {by === "PLAYS" ? `${formatNumber(plays)} plays` : formatDuration(minutes)}
        </div>
        <div className="text-muted-foreground text-xs">
          {by === "PLAYS" ? formatDuration(minutes) : `${formatNumber(plays)} plays`}
        </div>
      </div>
    </li>
  );
}
