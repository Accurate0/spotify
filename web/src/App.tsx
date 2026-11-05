import { lazy, Suspense } from "react";
import { NavLink, Navigate, Route, Routes, useLocation } from "react-router";
import FilterBar from "./components/FilterBar";
import FilterProvider from "./components/FilterProvider";
import ErrorBoundary from "./components/ErrorBoundary";
import EmptyState from "./components/EmptyState";
import { AUTH_DISABLED, currentUserName, signOut } from "./auth";
import { useFilter } from "./lib/filter";
import { cn } from "@/lib/utils";

// Pages load on demand; the charts library only ships with pages that draw charts.
const OverviewPage = lazy(() => import("./pages/OverviewPage"));
const TopPage = lazy(() => import("./pages/TopPage"));
const InsightsPage = lazy(() => import("./pages/InsightsPage"));
const SearchPage = lazy(() => import("./pages/SearchPage"));
const ItemPage = lazy(() => import("./pages/ItemPage"));
const ArtistPage = lazy(() => import("./pages/ArtistPage"));

function Fallback() {
  return <p className="text-muted-foreground text-sm">Loading…</p>;
}

function Tabs() {
  const { pathname, search } = useLocation();
  // Search has its own query string; the stats pages share the filter's.
  const filterSearch = new URLSearchParams(search);
  filterSearch.delete("q");
  const keep = filterSearch.size > 0 ? `?${filterSearch}` : "";

  const tabs = [
    { to: { pathname: "/", search: keep }, label: "Overview", end: true },
    { to: { pathname: "/top/artists", search: keep }, label: "Top", section: "/top/" },
    { to: { pathname: "/insights", search: keep }, label: "Insights" },
    { to: { pathname: "/search" }, label: "Search" },
  ];

  return (
    <nav className="border-border mb-6 flex gap-1 border-b">
      {tabs.map((t) => (
        <NavLink
          key={t.label}
          to={t.to}
          end={t.end}
          className={({ isActive }) =>
            cn(
              "-mb-px border-b-2 border-transparent px-3 pb-2 text-sm font-medium transition-colors",
              isActive || (t.section && pathname.startsWith(t.section))
                ? "text-foreground border-foreground"
                : "text-muted-foreground hover:text-foreground",
            )
          }
        >
          {t.label}
        </NavLink>
      ))}
    </nav>
  );
}

/** Stats pages: filters on top, content dimmed (not replaced) while a change loads. */
function StatsLayout({ children }: { children: React.ReactNode }) {
  const { isPending } = useFilter();
  const { pathname, search } = useLocation();
  const { years } = useFilter();
  if (years.length === 0) {
    return <EmptyState />;
  }

  return (
    <>
      <FilterBar />
      <div className={cn("transition-opacity", isPending && "opacity-60")}>
        <ErrorBoundary resetKey={pathname + search}>
          <Suspense fallback={<Fallback />}>{children}</Suspense>
        </ErrorBoundary>
      </div>
    </>
  );
}

export default function App() {
  const { pathname, search } = useLocation();
  return (
    <div className="mx-auto max-w-6xl px-4 py-10 sm:px-6">
      <header className="mb-6 flex items-start justify-between gap-4">
        <h1 className="font-display text-4xl font-semibold tracking-tight sm:text-5xl">Listening</h1>
        {!AUTH_DISABLED && (
          <div className="text-muted-foreground flex items-center gap-3 pt-2 text-sm">
            {currentUserName()}
            <button
              type="button"
              onClick={() => void signOut()}
              className="hover:text-foreground underline-offset-2 hover:underline"
            >
              Sign out
            </button>
          </div>
        )}
      </header>
      <Tabs />
      <ErrorBoundary resetKey={pathname + search}>
        <FilterProvider>
          <Routes>
            <Route path="/" element={<StatsLayout><OverviewPage /></StatsLayout>} />
            <Route path="/top" element={<Navigate to="/top/artists" replace />} />
            <Route path="/top/:kind" element={<StatsLayout><TopPage /></StatsLayout>} />
            <Route path="/insights" element={<StatsLayout><InsightsPage /></StatsLayout>} />
            <Route path="/track/:uri" element={<StatsLayout><ItemPage /></StatsLayout>} />
            <Route path="/artist/:name" element={<StatsLayout><ArtistPage /></StatsLayout>} />
            <Route
              path="/search"
              element={
                <Suspense fallback={<Fallback />}>
                  <SearchPage />
                </Suspense>
              }
            />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </FilterProvider>
      </ErrorBoundary>
    </div>
  );
}
