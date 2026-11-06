import { graphql, useLazyLoadQuery } from "react-relay";
import { Link } from "react-router";
import type { OverviewPageQuery } from "./__generated__/OverviewPageQuery.graphql";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import StatTiles from "@/components/StatTiles";
import TimelineChart, { type Bucket } from "@/components/TimelineChart";
import Heatmap from "@/components/Heatmap";
import Artwork from "@/components/Artwork";
import RankRow from "@/components/RankRow";
import RecentListens from "@/components/RecentListens";
import { useFilter } from "@/lib/filter";
import { artistPath, itemPath } from "@/lib/paths";

/** Bucket size that gives the timeline a readable number of columns for the range. */
function bucketFor(range: string): Bucket {
  if (range === "30d") return "DAY";
  if (range === "all") return "MONTH";
  return "WEEK";
}

export default function OverviewPage() {
  const { filter, state } = useFilter();
  const bucket = bucketFor(state.range);

  const data = useLazyLoadQuery<OverviewPageQuery>(
    graphql`
      query OverviewPageQuery($filter: SpotifyStatsFilter!, $bucket: SpotifyTimeBucket!) {
        ...RecentListens_query @arguments(filter: $filter)
        spotifyStats(filter: $filter) {
          summary {
            ...StatTiles_summary
          }
          timeline(bucket: $bucket) {
            ...TimelineChart_points
          }
          heatmap {
            ...Heatmap_cells
          }
          topArtists(first: 5) {
            edges {
              rank
              plays
              minutesPlayed
              node {
                id
                name
                imageUrl
              }
            }
          }
          topItems(first: 5) {
            edges {
              rank
              plays
              minutesPlayed
              node {
                id
                uri
                name
                artistName
                artworkUrl
              }
            }
          }
        }
      }
    `,
    { filter, bucket },
  );

  const { spotifyStats: stats } = data;
  const maxArtist = stats.topArtists.edges[0]?.minutesPlayed ?? 0;
  const maxItem = stats.topItems.edges[0]?.minutesPlayed ?? 0;

  return (
    <div className="space-y-6">
      <StatTiles summary={stats.summary} />

      <Card>
        <CardHeader>
          <CardTitle>Listening over time</CardTitle>
        </CardHeader>
        <CardContent>
          <TimelineChart points={stats.timeline} bucket={bucket} />
        </CardContent>
      </Card>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>
              <Link to={{ pathname: "/top/artists", search: window.location.search }} className="hover:underline">
                Top artists
              </Link>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ol>
              {stats.topArtists.edges.map((e) => (
                <RankRow
                  key={e.node.id}
                  rank={e.rank}
                  image={<Artwork url={e.node.imageUrl} alt={e.node.name} round />}
                  title={<Link to={artistPath(e.node.name)} className="hover:underline">{e.node.name}</Link>}
                  minutes={e.minutesPlayed}
                  plays={e.plays}
                  max={maxArtist}
                  by="MINUTES_PLAYED"
                />
              ))}
            </ol>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>
              <Link to={{ pathname: "/top/tracks", search: window.location.search }} className="hover:underline">
                Top tracks
              </Link>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ol>
              {stats.topItems.edges.map((e) => (
                <RankRow
                  key={e.node.id}
                  rank={e.rank}
                  image={<Artwork url={e.node.artworkUrl} alt={e.node.name ?? ""} />}
                  title={<Link to={itemPath(e.node.uri)} className="hover:underline">{e.node.name ?? e.node.uri}</Link>}
                  subtitle={e.node.artistName}
                  minutes={e.minutesPlayed}
                  plays={e.plays}
                  max={maxItem}
                  by="MINUTES_PLAYED"
                />
              ))}
            </ol>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Recent listens</CardTitle>
        </CardHeader>
        <CardContent>
          <RecentListens query={data} />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>When you listen</CardTitle>
        </CardHeader>
        <CardContent>
          <Heatmap cells={stats.heatmap} />
        </CardContent>
      </Card>
    </div>
  );
}
