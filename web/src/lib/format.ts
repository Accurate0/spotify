const integer = new Intl.NumberFormat();
const compact = new Intl.NumberFormat(undefined, {
  notation: "compact",
  maximumFractionDigits: 1,
});

export function formatNumber(value: number): string {
  return integer.format(value);
}

/** 1,284 / 12.9K / 1.2M for stat tiles and axis ticks. */
export function formatCompact(value: number): string {
  return value < 10_000 ? integer.format(value) : compact.format(value);
}

/** Minutes as the two largest units: `12d 4h`, `3h 05m`, `42m`. */
export function formatDuration(minutes: number): string {
  const days = Math.floor(minutes / 1440);
  const hours = Math.floor((minutes % 1440) / 60);
  const mins = minutes % 60;
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${String(mins).padStart(2, "0")}m`;
  return `${mins}m`;
}

/** Minutes as whole hours, for axes and totals: `7,032 h`. */
export function formatHours(minutes: number): string {
  return `${integer.format(Math.round(minutes / 60))} h`;
}

export function formatPercent(ratio: number): string {
  return `${(ratio * 100).toFixed(1)}%`;
}

export function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatPlayLength(ms: number | null | undefined): string {
  // Recent plays from the Web API have no length until an export covers them.
  if (ms == null) return "—";
  const seconds = Math.round(ms / 1000);
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}

/** A calendar date (`2024-02-10`) from the API, which is already in the filter's time zone. */
export function formatDay(date: string): string {
  const [y, m, d] = date.split("-").map(Number);
  return new Date(y, m - 1, d).toLocaleDateString(undefined, {
    weekday: "short",
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}
