import { formatDuration, formatNumber } from "@/lib/format";

export type ShareRow = {
  readonly label: string;
  readonly minutesPlayed: number;
  readonly plays: number;
};

/**
 * Part-to-whole as labelled horizontal bars: each row's share of total listening time, in the
 * single sequential hue. Values are text beside the bar, never the bar's colour.
 */
export default function ShareList({ rows, limit = 8 }: { rows: readonly ShareRow[]; limit?: number }) {
  const total = rows.reduce((sum, r) => sum + r.minutesPlayed, 0);
  if (total === 0) {
    return <p className="text-muted-foreground text-sm">Not recorded in this range.</p>;
  }

  // Fold the tail into "Other" rather than listing every minor value.
  const head = rows.slice(0, limit);
  const tail = rows.slice(limit);
  const shown: ShareRow[] =
    tail.length > 0
      ? [
          ...head,
          {
            label: `${tail.length} more`,
            minutesPlayed: tail.reduce((s, r) => s + r.minutesPlayed, 0),
            plays: tail.reduce((s, r) => s + r.plays, 0),
          },
        ]
      : head;

  return (
    <ul className="space-y-2.5">
      {shown.map((r) => {
        const share = r.minutesPlayed / total;
        return (
          <li key={r.label}>
            <div className="flex items-baseline justify-between gap-3 text-sm">
              <span className="truncate">{r.label}</span>
              <span className="text-muted-foreground shrink-0 tabular-nums">
                <span className="text-foreground font-medium">{(share * 100).toFixed(1)}%</span>
                {" · "}
                {formatDuration(r.minutesPlayed)} · {formatNumber(r.plays)} plays
              </span>
            </div>
            <div className="bg-muted mt-1 h-1.5 rounded-full">
              <div
                className="bg-series-1 h-full rounded-full"
                style={{ width: `${Math.max(0.5, share * 100)}%` }}
              />
            </div>
          </li>
        );
      })}
    </ul>
  );
}
