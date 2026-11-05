import { useState, type ReactNode } from "react";
import { graphql, useLazyLoadQuery, usePaginationFragment } from "react-relay";
import { Link, NavLink, useLocation, useParams } from "react-router";
import type { TopPageArtistsQuery } from "./__generated__/TopPageArtistsQuery.graphql";
import type { TopPageArtists_query$key } from "./__generated__/TopPageArtists_query.graphql";
import type { TopPageTracksQuery } from "./__generated__/TopPageTracksQuery.graphql";
import type { TopPageTracks_query$key } from "./__generated__/TopPageTracks_query.graphql";
import type { TopPageAlbumsQuery } from "./__generated__/TopPageAlbumsQuery.graphql";
import type { TopPageAlbums_query$key } from "./__generated__/TopPageAlbums_query.graphql";
import Artwork from "@/components/Artwork";
import RankRow, { type RankBy } from "@/components/RankRow";
import { useFilter } from "@/lib/filter";
import { artistPath, itemPath } from "@/lib/paths";
import { cn } from "@/lib/utils";

const PAGE = 50;

const KINDS = [
  { kind: "artists", label: "Artists" },
  { kind: "tracks", label: "Tracks" },
  { kind: "albums", label: "Albums" },
] as const;

function RankedList({
  children,
  hasNext,
  isLoadingNext,
  loadNext,
}: {
  children: ReactNode;
  hasNext: boolean;
  isLoadingNext: boolean;
  loadNext: () => void;
}) {
  return (
    <div>
      <ol className="divide-y">{children}</ol>
      {hasNext && (
        <button
          type="button"
          onClick={loadNext}
          disabled={isLoadingNext}
          className="hover:bg-accent mt-4 w-full rounded-md border py-2 text-sm disabled:opacity-60"
        >
          {isLoadingNext ? "Loading…" : "Load more"}
        </button>
      )}
    </div>
  );
}

function TopArtists({ by }: { by: RankBy }) {
  const { filter } = useFilter();
  const query = useLazyLoadQuery<TopPageArtistsQuery>(
    graphql`
      query TopPageArtistsQuery($filter: SpotifyStatsFilter!, $by: SpotifyRankBy!) {
        ...TopPageArtists_query @arguments(filter: $filter, by: $by)
      }
    `,
    { filter, by },
  );
  const { data, loadNext, hasNext, isLoadingNext } = usePaginationFragment(
    graphql`
      fragment TopPageArtists_query on Query
      @refetchable(queryName: "TopPageArtistsPaginationQuery")
      @argumentDefinitions(
        filter: { type: "SpotifyStatsFilter!" }
        by: { type: "SpotifyRankBy!" }
        first: { type: "Int", defaultValue: 50 }
        after: { type: "String" }
      ) {
        spotifyStats(filter: $filter) {
          topArtists(first: $first, after: $after, by: $by)
            @connection(key: "TopPageArtists_topArtists") {
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
        }
      }
    `,
    query as TopPageArtists_query$key,
  );
  const edges = data.spotifyStats.topArtists.edges;
  const max = by === "PLAYS" ? (edges[0]?.plays ?? 0) : (edges[0]?.minutesPlayed ?? 0);

  return (
    <RankedList hasNext={hasNext} isLoadingNext={isLoadingNext} loadNext={() => loadNext(PAGE)}>
      {edges.map((e) => (
        <RankRow
          key={e.node.id}
          rank={e.rank}
          image={<Artwork url={e.node.imageUrl} alt={e.node.name} round />}
          title={<Link to={artistPath(e.node.name)} className="hover:underline">{e.node.name}</Link>}
          minutes={e.minutesPlayed}
          plays={e.plays}
          max={max}
          by={by}
        />
      ))}
    </RankedList>
  );
}

function TopTracks({ by }: { by: RankBy }) {
  const { filter } = useFilter();
  const query = useLazyLoadQuery<TopPageTracksQuery>(
    graphql`
      query TopPageTracksQuery($filter: SpotifyStatsFilter!, $by: SpotifyRankBy!) {
        ...TopPageTracks_query @arguments(filter: $filter, by: $by)
      }
    `,
    { filter, by },
  );
  const { data, loadNext, hasNext, isLoadingNext } = usePaginationFragment(
    graphql`
      fragment TopPageTracks_query on Query
      @refetchable(queryName: "TopPageTracksPaginationQuery")
      @argumentDefinitions(
        filter: { type: "SpotifyStatsFilter!" }
        by: { type: "SpotifyRankBy!" }
        first: { type: "Int", defaultValue: 50 }
        after: { type: "String" }
      ) {
        spotifyStats(filter: $filter) {
          topItems(first: $first, after: $after, by: $by)
            @connection(key: "TopPageTracks_topItems") {
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
    query as TopPageTracks_query$key,
  );
  const edges = data.spotifyStats.topItems.edges;
  const max = by === "PLAYS" ? (edges[0]?.plays ?? 0) : (edges[0]?.minutesPlayed ?? 0);

  return (
    <RankedList hasNext={hasNext} isLoadingNext={isLoadingNext} loadNext={() => loadNext(PAGE)}>
      {edges.map((e) => (
        <RankRow
          key={e.node.id}
          rank={e.rank}
          image={<Artwork url={e.node.artworkUrl} alt={e.node.name ?? ""} />}
          title={<Link to={itemPath(e.node.uri)} className="hover:underline">{e.node.name ?? e.node.uri}</Link>}
          subtitle={
            e.node.artistName && (
              <Link to={artistPath(e.node.artistName)} className="hover:underline">{e.node.artistName}</Link>
            )
          }
          minutes={e.minutesPlayed}
          plays={e.plays}
          max={max}
          by={by}
        />
      ))}
    </RankedList>
  );
}

function TopAlbums({ by }: { by: RankBy }) {
  const { filter } = useFilter();
  const query = useLazyLoadQuery<TopPageAlbumsQuery>(
    graphql`
      query TopPageAlbumsQuery($filter: SpotifyStatsFilter!, $by: SpotifyRankBy!) {
        ...TopPageAlbums_query @arguments(filter: $filter, by: $by)
      }
    `,
    { filter, by },
  );
  const { data, loadNext, hasNext, isLoadingNext } = usePaginationFragment(
    graphql`
      fragment TopPageAlbums_query on Query
      @refetchable(queryName: "TopPageAlbumsPaginationQuery")
      @argumentDefinitions(
        filter: { type: "SpotifyStatsFilter!" }
        by: { type: "SpotifyRankBy!" }
        first: { type: "Int", defaultValue: 50 }
        after: { type: "String" }
      ) {
        spotifyStats(filter: $filter) {
          topAlbums(first: $first, after: $after, by: $by)
            @connection(key: "TopPageAlbums_topAlbums") {
            edges {
              rank
              plays
              minutesPlayed
              node {
                name
                artworkUrl
                artist {
                  name
                }
              }
            }
          }
        }
      }
    `,
    query as TopPageAlbums_query$key,
  );
  const edges = data.spotifyStats.topAlbums.edges;
  const max = by === "PLAYS" ? (edges[0]?.plays ?? 0) : (edges[0]?.minutesPlayed ?? 0);

  return (
    <RankedList hasNext={hasNext} isLoadingNext={isLoadingNext} loadNext={() => loadNext(PAGE)}>
      {edges.map((e) => (
        <RankRow
          key={`${e.node.artist?.name}\u001f${e.node.name}`}
          rank={e.rank}
          image={<Artwork url={e.node.artworkUrl} alt={e.node.name} />}
          title={e.node.name}
          subtitle={
            e.node.artist && (
              <Link to={artistPath(e.node.artist.name)} className="hover:underline">{e.node.artist.name}</Link>
            )
          }
          minutes={e.minutesPlayed}
          plays={e.plays}
          max={max}
          by={by}
        />
      ))}
    </RankedList>
  );
}

export default function TopPage() {
  const { kind = "artists" } = useParams();
  const { search } = useLocation();
  const [by, setBy] = useState<RankBy>("MINUTES_PLAYED");

  return (
    <div>
      <div className="mb-6 flex flex-wrap items-center justify-between gap-3">
        <div className="bg-muted inline-flex rounded-lg p-1">
          {KINDS.map((k) => (
            <NavLink
              key={k.kind}
              to={{ pathname: `/top/${k.kind}`, search }}
              className={({ isActive }) =>
                cn(
                  "rounded-md px-3 py-1 text-sm font-medium transition-colors",
                  isActive ? "bg-card text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground",
                )
              }
            >
              {k.label}
            </NavLink>
          ))}
        </div>
        <label className="text-muted-foreground flex items-center gap-2 text-sm">
          Rank by
          <select
            className="border-input bg-card text-foreground h-8 rounded-md border px-2 text-sm"
            value={by}
            onChange={(e) => setBy(e.target.value as RankBy)}
          >
            <option value="MINUTES_PLAYED">Listening time</option>
            <option value="PLAYS">Plays</option>
          </select>
        </label>
      </div>
      {kind === "tracks" ? (
        <TopTracks by={by} />
      ) : kind === "albums" ? (
        <TopAlbums by={by} />
      ) : (
        <TopArtists by={by} />
      )}
    </div>
  );
}
