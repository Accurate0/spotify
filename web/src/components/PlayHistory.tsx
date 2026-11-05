import { graphql, usePaginationFragment } from "react-relay";
import type { PlayHistory_item$key } from "./__generated__/PlayHistory_item.graphql";
import { formatDateTime, formatPlayLength } from "@/lib/format";

/** Every play of an item, newest first, paginated through the Item node. */
export default function PlayHistory({ item }: { item: PlayHistory_item$key }) {
  const { data, loadNext, hasNext, isLoadingNext } = usePaginationFragment(
    graphql`
      fragment PlayHistory_item on SpotifyItem
      @refetchable(queryName: "PlayHistoryPaginationQuery")
      @argumentDefinitions(
        filter: { type: "SpotifyStatsFilter!" }
        first: { type: "Int", defaultValue: 20 }
        after: { type: "String" }
      ) {
        stats(filter: $filter) {
          plays(first: $first, after: $after) @connection(key: "PlayHistory_plays") {
            edges {
              node {
                ts
                msPlayed
                skipped
                account {
                  displayName
                }
              }
            }
          }
        }
      }
    `,
    item,
  );

  const edges = data.stats.plays.edges;
  if (edges.length === 0) {
    return <p className="text-muted-foreground text-sm">No plays in this range.</p>;
  }

  return (
    <div>
      <table className="w-full text-sm">
        <thead className="text-muted-foreground text-left">
          <tr>
            <th className="py-1 font-medium">When</th>
            <th className="py-1 font-medium">Account</th>
            <th className="py-1 text-right font-medium">Played</th>
          </tr>
        </thead>
        <tbody className="tabular-nums">
          {edges.map((e, i) => (
            <tr key={`${e.node.ts}-${i}`} className="border-t">
              <td className="py-1.5">{formatDateTime(e.node.ts)}</td>
              <td className="py-1.5">{e.node.account.displayName}</td>
              <td className="py-1.5 text-right">
                {e.node.skipped && <span className="text-muted-foreground mr-2 text-xs">skipped</span>}
                {formatPlayLength(e.node.msPlayed)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {hasNext && (
        <button
          type="button"
          onClick={() => loadNext(50)}
          disabled={isLoadingNext}
          className="hover:bg-accent mt-4 w-full rounded-md border py-2 text-sm disabled:opacity-60"
        >
          {isLoadingNext ? "Loading…" : "Load more"}
        </button>
      )}
    </div>
  );
}
