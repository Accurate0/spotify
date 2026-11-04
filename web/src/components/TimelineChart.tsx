import { useState } from "react";
import { graphql, useFragment } from "react-relay";
import {
  Bar,
  BarChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
  type TooltipContentProps,
} from "recharts";
import type { NameType, ValueType } from "recharts/types/component/DefaultTooltipContent";
import type { TimelineChart_points$key } from "./__generated__/TimelineChart_points.graphql";
import { formatCompact, formatDuration, formatNumber } from "@/lib/format";

export type Bucket = "DAY" | "WEEK" | "MONTH" | "YEAR";

type Point = { start: string; label: string; minutes: number; value: number; plays: number };

function bucketLabel(iso: string, bucket: Bucket, long = false): string {
  const date = new Date(iso);
  switch (bucket) {
    case "YEAR":
      return String(date.getFullYear());
    case "MONTH":
      return date.toLocaleDateString(undefined, {
        month: long ? "long" : "short",
        year: "numeric",
      });
    default:
      return date.toLocaleDateString(undefined, {
        day: "numeric",
        month: "short",
        ...(long ? { year: "numeric" } : {}),
      });
  }
}

function TimelineTooltip({ active, payload }: TooltipContentProps<ValueType, NameType>) {
  const point = active ? (payload?.[0]?.payload as Point | undefined) : undefined;
  if (!point) return null;
  // Values lead, labels follow.
  return (
    <div className="bg-popover text-popover-foreground rounded-md border px-3 py-2 text-sm shadow-sm">
      <div className="font-semibold">{formatDuration(point.minutes)}</div>
      <div className="text-muted-foreground">
        {formatNumber(point.plays)} plays · {point.label}
      </div>
    </div>
  );
}

/** Listening time per bucket: one series, so one hue and no legend. */
export default function TimelineChart({
  points,
  bucket,
}: {
  points: TimelineChart_points$key;
  bucket: Bucket;
}) {
  const data = useFragment(
    graphql`
      fragment TimelineChart_points on TimelinePoint @relay(plural: true) {
        start
        plays
        minutesPlayed
      }
    `,
    points,
  );
  const [showTable, setShowTable] = useState(false);

  // Plot whole hours once buckets reach a few hours, minutes below that, so axis ticks
  // land on clean values in either case.
  const inHours = Math.max(0, ...data.map((p) => p.minutesPlayed)) >= 180;
  const unit = inHours ? "h" : "m";
  const rows: Point[] = data.map((p) => ({
    start: p.start,
    label: bucketLabel(p.start, bucket, true),
    minutes: p.minutesPlayed,
    value: inHours ? p.minutesPlayed / 60 : p.minutesPlayed,
    plays: p.plays,
  }));

  if (rows.length === 0) {
    return <p className="text-muted-foreground py-12 text-center text-sm">No listening in this range.</p>;
  }

  return (
    <div>
      {showTable ? (
        <div className="max-h-72 overflow-y-auto">
          <table className="w-full text-sm tabular-nums">
            <thead className="text-muted-foreground text-left">
              <tr>
                <th className="py-1 font-medium">Period</th>
                <th className="py-1 text-right font-medium">Listening</th>
                <th className="py-1 text-right font-medium">Plays</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r) => (
                <tr key={r.start} className="border-t">
                  <td className="py-1">{r.label}</td>
                  <td className="py-1 text-right">{formatDuration(r.minutes)}</td>
                  <td className="py-1 text-right">{formatNumber(r.plays)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="h-72">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={rows} margin={{ top: 8, right: 8, bottom: 0, left: 0 }} barCategoryGap={2}>
              <CartesianGrid vertical={false} stroke="var(--gridline)" />
              <XAxis
                dataKey="start"
                tickFormatter={(iso: string) => bucketLabel(iso, bucket)}
                tickLine={false}
                axisLine={{ stroke: "var(--border)" }}
                tick={{ fill: "var(--muted-foreground)", fontSize: 12 }}
                minTickGap={24}
              />
              <YAxis
                dataKey="value"
                allowDecimals={false}
                tickFormatter={(v: number) => `${formatCompact(v)}${unit}`}
                tickLine={false}
                axisLine={false}
                tick={{ fill: "var(--muted-foreground)", fontSize: 12 }}
                width={48}
              />
              <Tooltip content={TimelineTooltip} cursor={{ fill: "var(--muted)" }} />
              <Bar
                dataKey="value"
                fill="var(--series-1)"
                radius={[4, 4, 0, 0]}
                maxBarSize={24}
                isAnimationActive={false}
              />
            </BarChart>
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
