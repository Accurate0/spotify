import { graphql, useLazyLoadQuery } from "react-relay";
import { Link, useParams } from "react-router";
import type { ItemPageQuery } from "./__generated__/ItemPageQuery.graphql";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import StatTiles from "@/components/StatTiles";
import Firsts from "@/components/Firsts";
import TimelineChart from "@/components/TimelineChart";
import PlayHistory from "@/components/PlayHistory";
import { useFilter, type StatsFilter } from "@/lib/filter";
import { artistPath } from "@/lib/paths";
import { formatPercent, formatPlayLength } from "@/lib/format";
import Artwork from "@/components/Artwork";

export default function ItemPage() {
  const { uri = "" } = useParams();
  const { filter } = useFilter();
  // A track page always shows that item, whatever kinds the global filter includes.
  const itemFilter: StatsFilter = { ...filter, kinds: ["TRACK", "EPISODE", "AUDIOBOOK"] };

  const data = useLazyLoadQuery<ItemPageQuery>(
    graphql`
      query ItemPageQuery($uri: String!, $filter: SpotifyStatsFilter!) {
        spotifyItem(uri: $uri) {
          uri
          kind
          name
          artistName
          artworkUrl
          durationMs
          explicit
          releaseYear
          artist {
            name
          }
          album {
            name
          }
          stats(filter: $filter) {
            summary {
              ...StatTiles_summary
            }
            completion {
              average
              finishedShare
            }
            ...Firsts_stats
            timeline(bucket: MONTH) {
              ...TimelineChart_points
            }
          }
          ...PlayHistory_item @arguments(filter: $filter)
        }
      }
    `,
    { uri, filter: itemFilter },
  );

  const item = data.spotifyItem;
  if (!item) {
    return <p className="text-muted-foreground">No track or episode {uri}.</p>;
  }

  return (
    <div className="space-y-6">
      <header className="flex flex-col gap-5 sm:flex-row sm:items-end">
        <Artwork url={item.artworkUrl} alt={item.album?.name ?? item.name ?? ""} size="lg" />
        <div className="min-w-0">
          <p className="text-muted-foreground text-sm">{item.kind === "TRACK" ? "Track" : item.kind.toLowerCase()}</p>
          <h2 className="font-display text-3xl font-semibold tracking-tight">{item.name ?? item.uri}</h2>
          <p className="text-muted-foreground mt-1">
            {item.artist ? (
              <Link to={artistPath(item.artist.name)} className="text-foreground hover:underline">
                {item.artist.name}
              </Link>
            ) : (
              item.artistName
            )}
            {item.album && ` · ${item.album.name}`}
          </p>
          <p className="text-muted-foreground mt-1 text-sm">
            {[
              item.releaseYear && String(item.releaseYear),
              item.durationMs && formatPlayLength(item.durationMs),
              item.explicit && "Explicit",
            ]
              .filter(Boolean)
              .join(" · ")}
          </p>
          {item.stats.completion.average != null && (
            <p className="text-muted-foreground mt-1 text-sm">
              <span className="text-foreground font-medium">{formatPercent(item.stats.completion.average)}</span>{" "}
              played on average · {formatPercent(item.stats.completion.finishedShare ?? 0)} of plays finished
            </p>
          )}
        </div>
      </header>

      <StatTiles summary={item.stats.summary} scope="item" />
      <Firsts stats={item.stats} showFirstTrack={false} />

      <Card>
        <CardHeader>
          <CardTitle>Listening over time</CardTitle>
        </CardHeader>
        <CardContent>
          <TimelineChart points={item.stats.timeline} bucket="MONTH" />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Plays</CardTitle>
        </CardHeader>
        <CardContent>
          <PlayHistory item={item} />
        </CardContent>
      </Card>
    </div>
  );
}
