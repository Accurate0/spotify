import { useFilter } from "@/lib/filter";
import { cn } from "@/lib/utils";

const selectClass =
  "border-input bg-card text-foreground focus-visible:ring-ring h-9 rounded-md border px-3 text-sm outline-none focus-visible:ring-2";

function Toggle({
  checked,
  onChange,
  children,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  children: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={cn(
        "h-9 rounded-md border px-3 text-sm transition-colors",
        checked
          ? "border-foreground bg-foreground text-background"
          : "border-input bg-card text-muted-foreground hover:text-foreground",
      )}
    >
      {children}
    </button>
  );
}

/** One row of filters above the content they scope. */
export default function FilterBar() {
  const { state, setState, accounts, years } = useFilter();

  return (
    <div className="mb-8 flex flex-wrap items-center gap-2">
      <select
        aria-label="Date range"
        className={selectClass}
        value={state.range}
        onChange={(e) => setState({ range: e.target.value })}
      >
        <option value="all">All time</option>
        <option value="30d">Last 30 days</option>
        <option value="12m">Last 12 months</option>
        {years.map((y) => (
          <option key={y} value={String(y)}>
            {y}
          </option>
        ))}
      </select>
      <select
        aria-label="Account"
        className={selectClass}
        value={state.account ?? ""}
        onChange={(e) => setState({ account: e.target.value || null })}
      >
        <option value="">All accounts</option>
        {accounts.map((a) => (
          <option key={a.id} value={a.slug}>
            {a.displayName}
          </option>
        ))}
      </select>
      <Toggle checked={state.podcasts} onChange={(podcasts) => setState({ podcasts })}>
        Include podcasts
      </Toggle>
      <Toggle checked={state.allPlays} onChange={(allPlays) => setState({ allPlays })}>
        Count skips
      </Toggle>
    </div>
  );
}
