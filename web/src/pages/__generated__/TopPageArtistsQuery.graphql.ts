/**
 * @generated SignedSource<<38be83098d499a75fd40be907d20609d>>
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
export type TopPageArtistsQuery$variables = {
  by: SpotifyRankBy;
  filter: SpotifyStatsFilter;
};
export type TopPageArtistsQuery$data = {
  readonly " $fragmentSpreads": FragmentRefs<"TopPageArtists_query">;
};
export type TopPageArtistsQuery = {
  response: TopPageArtistsQuery$data;
  variables: TopPageArtistsQuery$variables;
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
];
return {
  "fragment": {
    "argumentDefinitions": [
      (v0/*:: as any*/),
      (v1/*:: as any*/)
    ],
    "kind": "Fragment",
    "metadata": null,
    "name": "TopPageArtistsQuery",
    "selections": [
      {
        "args": [
          (v2/*:: as any*/),
          (v3/*:: as any*/)
        ],
        "kind": "FragmentSpread",
        "name": "TopPageArtists_query"
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
    "name": "TopPageArtistsQuery",
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
            "concreteType": "SpotifyArtistRankedConnection",
            "kind": "LinkedField",
            "name": "topArtists",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifyArtistRankedEdge",
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
                    "concreteType": "SpotifyArtist",
                    "kind": "LinkedField",
                    "name": "node",
                    "plural": false,
                    "selections": [
                      {
                        "alias": null,
                        "args": null,
                        "kind": "ScalarField",
                        "name": "id",
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
                        "name": "imageUrl",
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
            "key": "TopPageArtists_topArtists",
            "kind": "LinkedHandle",
            "name": "topArtists"
          }
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "f0242595645d45ebb97da411f5f45831",
    "id": null,
    "metadata": {},
    "name": "TopPageArtistsQuery",
    "operationKind": "query",
    "text": "query TopPageArtistsQuery(\n  $filter: SpotifyStatsFilter!\n  $by: SpotifyRankBy!\n) {\n  ...TopPageArtists_query_9zxt2\n}\n\nfragment TopPageArtists_query_9zxt2 on Query {\n  spotifyStats(filter: $filter) {\n    topArtists(first: 50, by: $by) {\n      edges {\n        rank\n        plays\n        minutesPlayed\n        node {\n          id\n          name\n          imageUrl\n          __typename\n        }\n        cursor\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "75e2e185bac8424173c8d545cd15c150";

export default node;
