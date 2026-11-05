import { Suspense, useEffect, useState, useTransition } from "react";
import { graphql, useLazyLoadQuery } from "react-relay";
import { Link, useSearchParams } from "react-router";
import { Search } from "lucide-react";
import type { SearchPageQuery } from "./__generated__/SearchPageQuery.graphql";
import Artwork from "@/components/Artwork";
import { artistPath, itemPath } from "@/lib/paths";
import { cn } from "@/lib/utils";

function Results({ query }: { query: string }) {
  const data = useLazyLoadQuery<SearchPageQuery>(
    graphql`
      query SearchPageQuery($query: String!) {
        search(query: $query) {
          artists(first: 8) {
            edges {
              node {
                id
                name
                imageUrl
              }
            }
          }
          items(first: 25) {
            edges {
              node {
                id
                uri
                kind
                name
                artistName
                artworkUrl
                album {
                  name
                }
              }
            }
          }
        }
      }
    `,
    { query },
  );

  const { artists, items } = data.search;
  if (artists.edges.length === 0 && items.edges.length === 0) {
    return <p className="text-muted-foreground text-sm">Nothing matches “{query}”.</p>;
  }

  return (
    <div className="grid gap-8 md:grid-cols-[1fr_2fr]">
      <section>
        <h2 className="text-muted-foreground mb-2 text-sm font-medium">Artists</h2>
        <ul className="divide-y">
          {artists.edges.map((e) => (
            <li key={e.node.id} className="flex items-center gap-3 py-2">
              <Artwork url={e.node.imageUrl} alt={e.node.name} round />
              <Link to={artistPath(e.node.name)} className="font-medium hover:underline">
                {e.node.name}
              </Link>
            </li>
          ))}
        </ul>
      </section>
      <section>
        <h2 className="text-muted-foreground mb-2 text-sm font-medium">Tracks and episodes</h2>
        <ul className="divide-y">
          {items.edges.map((e) => (
            <li key={e.node.id} className="flex items-center gap-3 py-2">
              <Artwork url={e.node.artworkUrl} alt={e.node.name ?? ""} />
              <div className="min-w-0">
                <Link to={itemPath(e.node.uri)} className="font-medium hover:underline">
                  {e.node.name ?? e.node.uri}
                </Link>
                <div className="text-muted-foreground truncate text-sm">
                  {[e.node.artistName, e.node.album?.name].filter(Boolean).join(" · ")}
                  {e.node.kind !== "TRACK" && ` · ${e.node.kind.toLowerCase()}`}
                </div>
              </div>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

export default function SearchPage() {
  const [params, setParams] = useSearchParams();
  const query = params.get("q") ?? "";
  const [input, setInput] = useState(query);
  const [isPending, startTransition] = useTransition();

  // Debounce typing into the URL; results suspend inside a transition so the previous
  // results stay up while the next ones load.
  useEffect(() => {
    const trimmed = input.trim();
    if (trimmed === query) return;
    const timer = setTimeout(() => {
      startTransition(() => setParams(trimmed ? { q: trimmed } : {}, { replace: true }));
    }, 250);
    return () => clearTimeout(timer);
  }, [input, query, setParams]);

  return (
    <div>
      <label className="border-input bg-card focus-within:ring-ring mb-8 flex items-center gap-2 rounded-lg border px-3 focus-within:ring-2">
        <Search className="text-muted-foreground size-4" aria-hidden />
        <input
          autoFocus
          type="search"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Search artists, tracks and podcasts"
          aria-label="Search"
          className="h-11 flex-1 bg-transparent outline-none"
        />
      </label>
      {query && (
        <div className={cn("transition-opacity", isPending && "opacity-60")}>
          <Suspense fallback={<p className="text-muted-foreground text-sm">Searching…</p>}>
            <Results query={query} />
          </Suspense>
        </div>
      )}
    </div>
  );
}
