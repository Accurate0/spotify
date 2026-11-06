import { graphql, usePaginationFragment } from "react-relay";
import { Link } from "react-router";
import type { RecentListens_query$key } from "./__generated__/RecentListens_query.graphql";
import Artwork from "@/components/Artwork";
import { formatDateTime, formatPlayLength } from "@/lib/format";
import { artistPath, itemPath } from "@/lib/paths";

const PAGE = 20;

/** The newest plays in the filter, including recently played tracks no export covers yet. */
export default function RecentListens({ query }: { query: RecentListens_query$key }) {
  const { data, loadNext, hasNext, isLoadingNext } = usePaginationFragment(
    graphql`
      fragment RecentListens_query on Query
      @refetchable(queryName: "RecentListensPaginationQuery")
      @argumentDefinitions(
        filter: { type: "SpotifyStatsFilter!" }
        first: { type: "Int", defaultValue: 10 }
        after: { type: "String" }
      ) {
        spotifyStats(filter: $filter) {
          plays(first: $first, after: $after) @connection(key: "RecentListens_plays") {
            edges {
              node {
                ts
                msPlayed
                skipped
                source
                name
                artistName
                item {
                  uri
                  artworkUrl
                }
                account {
                  displayName
                }
              }
            }
          }
        }
      }
    `,
    query,
  );

  const edges = data.spotifyStats.plays.edges;
  if (edges.length === 0) {
    return <p className="text-muted-foreground text-sm">No plays in this range.</p>;
  }

  return (
    <div>
      <ol className="divide-y">
        {edges.map(({ node: p }, i) => {
          const title = p.name ?? "Unknown";
          return (
            <li key={`${p.ts}-${i}`} className="flex items-center gap-3 py-2">
              <Artwork url={p.item?.artworkUrl} alt={title} />
              <div className="min-w-0 flex-1">
                <div className="truncate text-sm font-medium">
                  {p.item ? (
                    <Link to={itemPath(p.item.uri)} className="hover:underline">
                      {title}
                    </Link>
                  ) : (
                    title
                  )}
                </div>
                {p.artistName && (
                  <div className="text-muted-foreground truncate text-xs">
                    <Link to={artistPath(p.artistName)} className="hover:underline">
                      {p.artistName}
                    </Link>
                  </div>
                )}
              </div>
              <div className="text-muted-foreground shrink-0 text-right text-xs tabular-nums">
                <div>{formatDateTime(p.ts)}</div>
                <div>
                  {p.account.displayName}
                  {p.skipped && " · skipped"}
                  {p.source === "RECENT" ? (
                    <span title="From Spotify's recently played; the next export fills in its length"> · new</span>
                  ) : (
                    ` · ${formatPlayLength(p.msPlayed)}`
                  )}
                </div>
              </div>
            </li>
          );
        })}
      </ol>
      {hasNext && (
        <button
          type="button"
          onClick={() => loadNext(PAGE)}
          disabled={isLoadingNext}
          className="hover:bg-accent mt-4 w-full rounded-md border py-2 text-sm disabled:opacity-60"
        >
          {isLoadingNext ? "Loading…" : "Load more"}
        </button>
      )}
    </div>
  );
}
