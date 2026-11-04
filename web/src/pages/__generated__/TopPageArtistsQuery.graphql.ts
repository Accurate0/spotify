/**
 * @generated SignedSource<<8e7f079aa78d7e7f65ae6f2a14d2224a>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type ItemKind = "AUDIOBOOK" | "EPISODE" | "TRACK" | "%future added value";
export type RankBy = "MINUTES_PLAYED" | "PLAYS" | "%future added value";
export type StatsFilter = {
  accounts?: ReadonlyArray<string>;
  from?: string | null | undefined;
  kinds?: ReadonlyArray<ItemKind>;
  minMsPlayed?: number;
  timeZone?: string;
  to?: string | null | undefined;
};
export type TopPageArtistsQuery$variables = {
  by: RankBy;
  filter: StatsFilter;
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
        "concreteType": "Stats",
        "kind": "LinkedField",
        "name": "stats",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": (v4/*:: as any*/),
            "concreteType": "RankedArtistConnection",
            "kind": "LinkedField",
            "name": "topArtists",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "RankedArtistEdge",
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
                    "concreteType": "Artist",
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
    "cacheID": "597fe1d64ba8b2d2581df948cb7f90ee",
    "id": null,
    "metadata": {},
    "name": "TopPageArtistsQuery",
    "operationKind": "query",
    "text": "query TopPageArtistsQuery(\n  $filter: StatsFilter!\n  $by: RankBy!\n) {\n  ...TopPageArtists_query_9zxt2\n}\n\nfragment TopPageArtists_query_9zxt2 on Query {\n  stats(filter: $filter) {\n    topArtists(first: 50, by: $by) {\n      edges {\n        rank\n        plays\n        minutesPlayed\n        node {\n          id\n          name\n          __typename\n        }\n        cursor\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "72eaf92a6236ecc4e01747fd4a61bcd3";

export default node;
