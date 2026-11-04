import { graphql, useLazyLoadQuery, usePaginationFragment } from "react-relay";
import { Link, useParams } from "react-router";
import type { ArtistPageQuery } from "./__generated__/ArtistPageQuery.graphql";
import type { ArtistPageTracks_artist$key } from "./__generated__/ArtistPageTracks_artist.graphql";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import StatTiles from "@/components/StatTiles";
import Firsts from "@/components/Firsts";
import TimelineChart from "@/components/TimelineChart";
import RankRow from "@/components/RankRow";
import { useFilter } from "@/lib/filter";
import { itemPath } from "@/lib/paths";

function ArtistTracks({ artist }: { artist: ArtistPageTracks_artist$key }) {
  const { data, loadNext, hasNext, isLoadingNext } = usePaginationFragment(
    graphql`
      fragment ArtistPageTracks_artist on Artist
      @refetchable(queryName: "ArtistPageTracksPaginationQuery")
      @argumentDefinitions(
        filter: { type: "StatsFilter!" }
        first: { type: "Int", defaultValue: 20 }
        after: { type: "String" }
      ) {
        stats(filter: $filter) {
          topItems(first: $first, after: $after) @connection(key: "ArtistPageTracks_topItems") {
            edges {
              rank
              plays
              minutesPlayed
              node {
                id
                uri
                name
                album {
                  name
                }
              }
            }
          }
        }
      }
    `,
    artist,
  );

  const edges = data.stats.topItems.edges;
  const max = edges[0]?.minutesPlayed ?? 0;

  return (
    <div>
      <ol className="divide-y">
        {edges.map((e) => (
          <RankRow
            key={e.node.id}
            rank={e.rank}
            title={<Link to={itemPath(e.node.uri)} className="hover:underline">{e.node.name ?? e.node.uri}</Link>}
            subtitle={e.node.album?.name}
            minutes={e.minutesPlayed}
            plays={e.plays}
            max={max}
            by="MINUTES_PLAYED"
          />
        ))}
      </ol>
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

export default function ArtistPage() {
  const { name = "" } = useParams();
  const { filter } = useFilter();

  const data = useLazyLoadQuery<ArtistPageQuery>(
    graphql`
      query ArtistPageQuery($name: String!, $filter: StatsFilter!) {
        artist(name: $name) {
          name
          stats(filter: $filter) {
            summary {
              ...StatTiles_summary
            }
            ...Firsts_stats
            timeline(bucket: MONTH) {
              ...TimelineChart_points
            }
          }
          ...ArtistPageTracks_artist @arguments(filter: $filter)
        }
      }
    `,
    { name, filter },
  );

  const artist = data.artist;
  if (!artist) {
    return <p className="text-muted-foreground">No artist named “{name}”.</p>;
  }

  return (
    <div className="space-y-6">
      <header>
        <p className="text-muted-foreground text-sm">Artist</p>
        <h2 className="font-display text-3xl font-semibold tracking-tight">{artist.name}</h2>
      </header>

      <StatTiles summary={artist.stats.summary} scope="artist" />
      <Firsts stats={artist.stats} showFirstTrack />

      <Card>
        <CardHeader>
          <CardTitle>Listening over time</CardTitle>
        </CardHeader>
        <CardContent>
          <TimelineChart points={artist.stats.timeline} bucket="MONTH" />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Top tracks</CardTitle>
        </CardHeader>
        <CardContent>
          <ArtistTracks artist={artist} />
        </CardContent>
      </Card>
    </div>
  );
}
