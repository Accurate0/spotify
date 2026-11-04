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

export default function ItemPage() {
  const { uri = "" } = useParams();
  const { filter } = useFilter();
  // A track page always shows that item, whatever kinds the global filter includes.
  const itemFilter: StatsFilter = { ...filter, kinds: ["TRACK", "EPISODE", "AUDIOBOOK"] };

  const data = useLazyLoadQuery<ItemPageQuery>(
    graphql`
      query ItemPageQuery($uri: String!, $filter: StatsFilter!) {
        item(uri: $uri) {
          uri
          kind
          name
          artistName
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

  const item = data.item;
  if (!item) {
    return <p className="text-muted-foreground">No track or episode {uri}.</p>;
  }

  return (
    <div className="space-y-6">
      <header>
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
