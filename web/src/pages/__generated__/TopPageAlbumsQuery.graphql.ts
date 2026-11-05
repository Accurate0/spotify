/**
 * @generated SignedSource<<465a7a1adc750d5e6e91623e7ec9e5c7>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type SpotifyItemKind = "AUDIOBOOK" | "EPISODE" | "TRACK" | "%future added value";
export type SpotifyRankBy = "MINUTES_PLAYED" | "PLAYS" | "%future added value";
export type SpotifyStatsFilter = {
  accounts?: ReadonlyArray<string>;
  from?: string | null | undefined;
  kinds?: ReadonlyArray<SpotifyItemKind>;
  minMsPlayed?: number;
  timeZone?: string;
  to?: string | null | undefined;
};
export type TopPageAlbumsQuery$variables = {
  by: SpotifyRankBy;
  filter: SpotifyStatsFilter;
};
export type TopPageAlbumsQuery$data = {
  readonly " $fragmentSpreads": FragmentRefs<"TopPageAlbums_query">;
};
export type TopPageAlbumsQuery = {
  response: TopPageAlbumsQuery$data;
  variables: TopPageAlbumsQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "by"
},
v1 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "filter"
},
v2 = {
  "kind": "Variable",
  "name": "by",
  "variableName": "by"
},
v3 = {
  "kind": "Variable",
  "name": "filter",
  "variableName": "filter"
},
v4 = [
  (v2/*:: as any*/),
  {
    "kind": "Literal",
    "name": "first",
    "value": 50
  }
],
v5 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
};
return {
  "fragment": {
    "argumentDefinitions": [
      (v0/*:: as any*/),
      (v1/*:: as any*/)
    ],
    "kind": "Fragment",
    "metadata": null,
    "name": "TopPageAlbumsQuery",
    "selections": [
      {
        "args": [
          (v2/*:: as any*/),
          (v3/*:: as any*/)
        ],
        "kind": "FragmentSpread",
        "name": "TopPageAlbums_query"
      }
    ],
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [
      (v1/*:: as any*/),
      (v0/*:: as any*/)
    ],
    "kind": "Operation",
    "name": "TopPageAlbumsQuery",
    "selections": [
      {
        "alias": null,
        "args": [
          (v3/*:: as any*/)
        ],
        "concreteType": "SpotifyStats",
        "kind": "LinkedField",
        "name": "spotifyStats",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": (v4/*:: as any*/),
            "concreteType": "SpotifyAlbumRankedConnection",
            "kind": "LinkedField",
            "name": "topAlbums",
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
                      (v5/*:: as any*/),
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
                          (v5/*:: as any*/),
                          {
                            "alias": null,
                            "args": null,
                            "kind": "ScalarField",
                            "name": "id",
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
          },
          {
            "alias": null,
            "args": (v4/*:: as any*/),
            "filters": [
              "by"
            ],
            "handle": "connection",
            "key": "TopPageAlbums_topAlbums",
            "kind": "LinkedHandle",
            "name": "topAlbums"
          }
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "bf597b447487f84739d64fed9edd9b36",
    "id": null,
    "metadata": {},
    "name": "TopPageAlbumsQuery",
    "operationKind": "query",
    "text": "query TopPageAlbumsQuery(\n  $filter: SpotifyStatsFilter!\n  $by: SpotifyRankBy!\n) {\n  ...TopPageAlbums_query_9zxt2\n}\n\nfragment TopPageAlbums_query_9zxt2 on Query {\n  spotifyStats(filter: $filter) {\n    topAlbums(first: 50, by: $by) {\n      edges {\n        rank\n        plays\n        minutesPlayed\n        node {\n          name\n          artworkUrl\n          artist {\n            name\n            id\n          }\n          __typename\n        }\n        cursor\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "71008b0cf0a93f1d4963a960f1753445";

export default node;
