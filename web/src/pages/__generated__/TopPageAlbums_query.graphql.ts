/**
 * @generated SignedSource<<c1b2ce96d2f5726da9cee101b72b191a>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type TopPageAlbums_query$data = {
  readonly spotifyStats: {
    readonly topAlbums: {
      readonly edges: ReadonlyArray<{
        readonly minutesPlayed: number;
        readonly node: {
          readonly artist: {
            readonly name: string;
          } | null | undefined;
          readonly artworkUrl: string | null | undefined;
          readonly name: string;
        };
        readonly plays: number;
        readonly rank: number;
      }>;
    };
  };
  readonly " $fragmentType": "TopPageAlbums_query";
};
export type TopPageAlbums_query$key = {
  readonly " $data"?: TopPageAlbums_query$data;
  readonly " $fragmentSpreads": FragmentRefs<"TopPageAlbums_query">;
};

import TopPageAlbumsPaginationQuery_graphql from './TopPageAlbumsPaginationQuery.graphql';

const node: ReaderFragment = (function(){
var v0 = [
  "spotifyStats",
  "topAlbums"
],
v1 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
};
return {
  "argumentDefinitions": [
    {
      "defaultValue": null,
      "kind": "LocalArgument",
      "name": "after"
    },
    {
      "defaultValue": null,
      "kind": "LocalArgument",
      "name": "by"
    },
    {
      "defaultValue": null,
      "kind": "LocalArgument",
      "name": "filter"
    },
    {
      "defaultValue": 50,
      "kind": "LocalArgument",
      "name": "first"
    }
  ],
  "kind": "Fragment",
  "metadata": {
    "connection": [
      {
        "count": "first",
        "cursor": "after",
        "direction": "forward",
        "path": (v0/*:: as any*/)
      }
    ],
    "refetch": {
      "connection": {
        "forward": {
          "count": "first",
          "cursor": "after"
        },
        "backward": null,
        "path": (v0/*:: as any*/)
      },
      "fragmentPathInResult": [],
      "operation": TopPageAlbumsPaginationQuery_graphql
    }
  },
  "name": "TopPageAlbums_query",
  "selections": [
    {
      "alias": null,
      "args": [
        {
          "kind": "Variable",
          "name": "filter",
          "variableName": "filter"
        }
      ],
      "concreteType": "SpotifyStats",
      "kind": "LinkedField",
      "name": "spotifyStats",
      "plural": false,
      "selections": [
        {
          "alias": "topAlbums",
          "args": [
            {
              "kind": "Variable",
              "name": "by",
              "variableName": "by"
            }
          ],
          "concreteType": "SpotifyAlbumRankedConnection",
          "kind": "LinkedField",
          "name": "__TopPageAlbums_topAlbums_connection",
          "plural": false,
          "selections": [
            {
              "alias": null,
              "args": null,
              "concreteType": "SpotifyAlbumRankedEdge",
              "kind": "LinkedField",
              "name": "edges",
              "plural": true,
              "selections": [
                {
                  "alias": null,
                  "args": null,
                  "kind": "ScalarField",
                  "name": "rank",
                  "storageKey": null
                },
                {
                  "alias": null,
                  "args": null,
                  "kind": "ScalarField",
                  "name": "plays",
                  "storageKey": null
                },
                {
                  "alias": null,
                  "args": null,
                  "kind": "ScalarField",
                  "name": "minutesPlayed",
                  "storageKey": null
                },
                {
                  "alias": null,
                  "args": null,
                  "concreteType": "SpotifyAlbum",
                  "kind": "LinkedField",
                  "name": "node",
                  "plural": false,
                  "selections": [
                    (v1/*:: as any*/),
                    {
                      "alias": null,
                      "args": null,
                      "kind": "ScalarField",
                      "name": "artworkUrl",
                      "storageKey": null
                    },
                    {
                      "alias": null,
                      "args": null,
                      "concreteType": "SpotifyArtist",
                      "kind": "LinkedField",
                      "name": "artist",
                      "plural": false,
                      "selections": [
                        (v1/*:: as any*/)
                      ],
                      "storageKey": null
                    },
                    {
                      "alias": null,
                      "args": null,
                      "kind": "ScalarField",
                      "name": "__typename",
                      "storageKey": null
                    }
                  ],
                  "storageKey": null
                },
                {
                  "alias": null,
                  "args": null,
                  "kind": "ScalarField",
                  "name": "cursor",
                  "storageKey": null
                }
              ],
              "storageKey": null
            },
            {
              "alias": null,
              "args": null,
              "concreteType": "PageInfo",
              "kind": "LinkedField",
              "name": "pageInfo",
              "plural": false,
              "selections": [
                {
                  "alias": null,
                  "args": null,
                  "kind": "ScalarField",
                  "name": "endCursor",
                  "storageKey": null
                },
                {
                  "alias": null,
                  "args": null,
                  "kind": "ScalarField",
                  "name": "hasNextPage",
                  "storageKey": null
                }
              ],
              "storageKey": null
            }
          ],
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "type": "Query",
  "abstractKey": null
};
})();

(node as any).hash = "cd17d435d0ffd46c845a1ecca5582e6d";

export default node;
