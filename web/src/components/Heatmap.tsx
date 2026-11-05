import { useState } from "react";
import { graphql, useFragment } from "react-relay";
import type { Heatmap_cells$key } from "./__generated__/Heatmap_cells.graphql";
import { formatDuration, formatNumber } from "@/lib/format";
import { cn } from "@/lib/utils";

const DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const HOURS = Array.from({ length: 24 }, (_, h) => h);
// Sequential ramp steps from index.css; level 0 (no listening) is the muted surface.
const LEVELS = ["bg-muted", "bg-heat-1", "bg-heat-2", "bg-heat-3", "bg-heat-4", "bg-heat-5", "bg-heat-6"];

type Cell = { minutes: number; plays: number };

function hourLabel(hour: number): string {
  return new Date(2000, 0, 1, hour).toLocaleTimeString(undefined, { hour: "numeric" });
}

/** Listening by weekday and hour. Each cell is its own hover/focus target. */
export default function Heatmap({ cells }: { cells: Heatmap_cells$key }) {
  const data = useFragment(
    graphql`
      fragment Heatmap_cells on SpotifyHeatmapCell @relay(plural: true) {
        dayOfWeek
        hour
        plays
        minutesPlayed
      }
    `,
    cells,
  );
  const [active, setActive] = useState<{ day: number; hour: number } | null>(null);

  const grid: Cell[][] = DAYS.map(() => HOURS.map(() => ({ minutes: 0, plays: 0 })));
  for (const c of data) {
    grid[c.dayOfWeek - 1][c.hour] = { minutes: c.minutesPlayed, plays: c.plays };
  }
  const max = Math.max(1, ...data.map((c) => c.minutesPlayed));
  const level = (minutes: number) =>
    minutes === 0 ? 0 : Math.max(1, Math.ceil((minutes / max) * (LEVELS.length - 1)));

  const activeCell = active && grid[active.day][active.hour];

  return (
    <div>
      <div className="text-muted-foreground mb-3 h-5 text-sm" aria-live="polite">
        {active && activeCell ? (
          <>
            <span className="text-foreground font-semibold">{formatDuration(activeCell.minutes)}</span>
            {" · "}
            {formatNumber(activeCell.plays)} plays · {DAYS[active.day]} {hourLabel(active.hour)}
          </>
        ) : (
          "Hover a cell for details"
        )}
      </div>
      <div className="overflow-x-auto">
        <div
          className="grid min-w-[560px] gap-[2px]"
          style={{ gridTemplateColumns: "2.5rem repeat(24, minmax(0, 1fr))" }}
          onPointerLeave={() => setActive(null)}
        >
          {DAYS.map((day, d) => (
            <div key={day} className="contents">
              <div className="text-muted-foreground flex items-center text-xs">{day}</div>
              {HOURS.map((hour) => {
                const cell = grid[d][hour];
                return (
                  <div
                    key={hour}
                    tabIndex={0}
                    role="img"
                    aria-label={`${day} ${hourLabel(hour)}: ${formatDuration(cell.minutes)}, ${cell.plays} plays`}
                    onPointerEnter={() => setActive({ day: d, hour })}
                    onFocus={() => setActive({ day: d, hour })}
                    onBlur={() => setActive(null)}
                    className={cn(
                      "focus-visible:ring-ring h-6 rounded-[3px] outline-none hover:opacity-80 focus-visible:ring-2",
                      LEVELS[level(cell.minutes)],
                      active?.day === d && active.hour === hour && "ring-foreground ring-2",
                    )}
                  />
                );
              })}
            </div>
          ))}
          <div />
          {HOURS.map((hour) => (
            <div key={hour} className="text-muted-foreground pt-1 text-center text-[10px]">
              {hour % 6 === 0 ? hourLabel(hour) : ""}
            </div>
          ))}
        </div>
      </div>
      <div className="text-muted-foreground mt-3 flex items-center gap-1 text-xs">
        Less
        {LEVELS.map((cls) => (
          <span key={cls} className={cn("size-3 rounded-[3px]", cls)} />
        ))}
        More
      </div>
    </div>
  );
}
