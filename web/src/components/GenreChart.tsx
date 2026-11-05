import { useState } from "react";
import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
  type TooltipContentProps,
} from "recharts";
import type { NameType, ValueType } from "recharts/types/component/DefaultTooltipContent";
import { formatDuration } from "@/lib/format";
import type { Bucket } from "./TimelineChart";

export type GenreSeries = {
  readonly genre: string;
  readonly points: readonly { readonly start: string; readonly minutesPlayed: number }[];
};

// Categorical slots 1-5 in fixed order (validated adjacent-pair in both modes). Light slots
// 3-5 are under 3:1 on the card, so identity also comes from the legend, tooltip and table.
const SLOTS = ["var(--series-1)", "var(--series-2)", "var(--series-3)", "var(--series-4)", "var(--series-5)"];

function label(iso: string, bucket: Bucket): string {
  const date = new Date(iso);
  return bucket === "YEAR"
    ? String(date.getFullYear())
    : date.toLocaleDateString(undefined, { month: "short", year: "numeric" });
}

/**
 * Listening per genre over time. `genres` fixes the series and their colours (the account's
 * all-time top genres), so colours follow the genre, not its rank in the current range.
 */
export default function GenreChart({
  genres,
  series,
  bucket,
}: {
  genres: readonly string[];
  series: readonly GenreSeries[];
  bucket: Bucket;
}) {
  const [showTable, setShowTable] = useState(false);
  const color = (genre: string) => SLOTS[genres.indexOf(genre) % SLOTS.length];

  const starts = [...new Set(series.flatMap((s) => s.points.map((p) => p.start)))].sort();
  const rows = starts.map((start) => {
    const row: Record<string, string | number> = { start };
    for (const s of series) {
      row[s.genre] = s.points.find((p) => p.start === start)?.minutesPlayed ?? 0;
    }
    return row;
  });

  if (rows.length === 0 || genres.length === 0) {
    return <p className="text-muted-foreground py-12 text-center text-sm">No genre data in this range yet.</p>;
  }

  function ChartTooltip({ active, payload }: TooltipContentProps<ValueType, NameType>) {
    if (!active || !payload?.length) return null;
    const start = payload[0].payload.start as string;
    return (
      <div className="bg-popover text-popover-foreground rounded-md border px-3 py-2 text-sm shadow-sm">
        <div className="text-muted-foreground mb-1">{label(start, bucket)}</div>
        {[...payload]
          .sort((a, b) => Number(b.value) - Number(a.value))
          .map((p) => (
            <div key={String(p.dataKey)} className="flex items-center gap-2">
              <span className="h-0.5 w-3 rounded-full" style={{ background: color(String(p.dataKey)) }} />
              <span className="font-semibold">{formatDuration(Number(p.value))}</span>
              <span className="text-muted-foreground">{String(p.dataKey)}</span>
            </div>
          ))}
      </div>
    );
  }

  return (
    <div>
      <ul className="mb-3 flex flex-wrap gap-x-4 gap-y-1 text-sm">
        {genres.map((g) => (
          <li key={g} className="flex items-center gap-1.5">
            <span className="h-0.5 w-4 rounded-full" style={{ background: color(g) }} />
            {g}
          </li>
        ))}
      </ul>
      {showTable ? (
        <div className="max-h-72 overflow-auto">
          <table className="w-full text-sm tabular-nums">
            <thead className="text-muted-foreground text-left">
              <tr>
                <th className="py-1 font-medium">Period</th>
                {genres.map((g) => (
                  <th key={g} className="py-1 text-right font-medium">
                    {g}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((r) => (
                <tr key={String(r.start)} className="border-t">
                  <td className="py-1">{label(String(r.start), bucket)}</td>
                  {genres.map((g) => (
                    <td key={g} className="py-1 text-right">
                      {formatDuration(Number(r[g] ?? 0))}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="h-72">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={rows} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
              <CartesianGrid vertical={false} stroke="var(--gridline)" />
              <XAxis
                dataKey="start"
                tickFormatter={(iso: string) => label(iso, bucket)}
                tickLine={false}
                axisLine={{ stroke: "var(--border)" }}
                tick={{ fill: "var(--muted-foreground)", fontSize: 12 }}
                minTickGap={24}
              />
              <YAxis
                allowDecimals={false}
                tickFormatter={(m: number) => `${Math.round(m / 60)}h`}
                tickLine={false}
                axisLine={false}
                tick={{ fill: "var(--muted-foreground)", fontSize: 12 }}
                width={44}
              />
              <Tooltip content={ChartTooltip} cursor={{ stroke: "var(--border)" }} />
              {genres.map((g) => (
                <Line
                  key={g}
                  dataKey={g}
                  type="monotone"
                  stroke={color(g)}
                  strokeWidth={2}
                  dot={false}
                  activeDot={{ r: 4, strokeWidth: 2, stroke: "var(--card)" }}
                  isAnimationActive={false}
                />
              ))}
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}
      <button
        type="button"
        onClick={() => setShowTable(!showTable)}
        className="text-muted-foreground hover:text-foreground mt-2 text-xs underline-offset-2 hover:underline"
      >
        {showTable ? "Show chart" : "Show as table"}
      </button>
    </div>
  );
}
