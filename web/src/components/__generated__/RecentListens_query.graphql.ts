/**
 * @generated SignedSource<<4645115423880e6776d109e9d3314a61>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
export type SpotifyPlaySource = "BASIC" | "EXTENDED" | "RECENT" | "%future added value";
import { FragmentRefs } from "relay-runtime";
export type RecentListens_query$data = {
  readonly spotifyStats: {
    readonly plays: {
      readonly edges: ReadonlyArray<{
        readonly node: {
          readonly account: {
            readonly displayName: string;
          };
          readonly artistName: string | null | undefined;
          readonly item: {
            readonly artworkUrl: string | null | undefined;
            readonly uri: string;
          } | null | undefined;
          readonly msPlayed: number | null | undefined;
          readonly name: string | null | undefined;
          readonly skipped: boolean | null | undefined;
          readonly source: SpotifyPlaySource;
          readonly ts: string;
        };
      }>;
    };
  };
  readonly " $fragmentType": "RecentListens_query";
};
export type RecentListens_query$key = {
  readonly " $data"?: RecentListens_query$data;
  readonly " $fragmentSpreads": FragmentRefs<"RecentListens_query">;
};

import RecentListensPaginationQuery_graphql from './RecentListensPaginationQuery.graphql';

const node: ReaderFragment = (function(){
var v0 = [
  "spotifyStats",
  "plays"
];
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
      "name": "filter"
    },
    {
      "defaultValue": 10,
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
      "operation": RecentListensPaginationQuery_graphql
    }
  },
  "name": "RecentListens_query",
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
          "alias": "plays",
          "args": null,
          "concreteType": "SpotifyPlayConnection",
          "kind": "LinkedField",
          "name": "__RecentListens_plays_connection",
          "plural": false,
          "selections": [
            {
              "alias": null,
              "args": null,
              "concreteType": "SpotifyPlayEdge",
              "kind": "LinkedField",
              "name": "edges",
              "plural": true,
              "selections": [
                {
                  "alias": null,
                  "args": null,
                  "concreteType": "SpotifyPlay",
                  "kind": "LinkedField",
                  "name": "node",
                  "plural": false,
                  "selections": [
                    {
                      "alias": null,
                      "args": null,
                      "kind": "ScalarField",
                      "name": "ts",
                      "storageKey": null
                    },
                    {
                      "alias": null,
                      "args": null,
                      "kind": "ScalarField",
                      "name": "msPlayed",
                      "storageKey": null
                    },
                    {
                      "alias": null,
                      "args": null,
                      "kind": "ScalarField",
                      "name": "skipped",
                      "storageKey": null
                    },
                    {
                      "alias": null,
                      "args": null,
                      "kind": "ScalarField",
                      "name": "source",
                      "storageKey": null
                    },
                    {
                      "alias": null,
                      "args": null,
                      "kind": "ScalarField",
                      "name": "name",
                      "storageKey": null
                    },
                    {
                      "alias": null,
                      "args": null,
                      "kind": "ScalarField",
                      "name": "artistName",
                      "storageKey": null
                    },
                    {
                      "alias": null,
                      "args": null,
                      "concreteType": "SpotifyItem",
                      "kind": "LinkedField",
                      "name": "item",
                      "plural": false,
                      "selections": [
                        {
                          "alias": null,
                          "args": null,
                          "kind": "ScalarField",
                          "name": "uri",
                          "storageKey": null
                        },
                        {
                          "alias": null,
                          "args": null,
                          "kind": "ScalarField",
                          "name": "artworkUrl",
                          "storageKey": null
                        }
                      ],
                      "storageKey": null
                    },
                    {
                      "alias": null,
                      "args": null,
                      "concreteType": "SpotifyAccount",
                      "kind": "LinkedField",
                      "name": "account",
                      "plural": false,
                      "selections": [
                        {
                          "alias": null,
                          "args": null,
                          "kind": "ScalarField",
                          "name": "displayName",
                          "storageKey": null
                        }
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

(node as any).hash = "31137643cfc6bd27f684fc4555127775";

export default node;
