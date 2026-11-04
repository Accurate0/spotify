/**
 * @generated SignedSource<<621ed1c77c4d453d91b406ab0c6b4d8f>>
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
export type TopPageArtistsPaginationQuery$variables = {
  after?: string | null | undefined;
  by: RankBy;
  filter: StatsFilter;
  first?: number | null | undefined;
};
export type TopPageArtistsPaginationQuery$data = {
  readonly " $fragmentSpreads": FragmentRefs<"TopPageArtists_query">;
};
export type TopPageArtistsPaginationQuery = {
  response: TopPageArtistsPaginationQuery$data;
  variables: TopPageArtistsPaginationQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
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
v1 = {
  "kind": "Variable",
  "name": "after",
  "variableName": "after"
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
v4 = {
  "kind": "Variable",
  "name": "first",
  "variableName": "first"
},
v5 = [
  (v1/*:: as any*/),
  (v2/*:: as any*/),
  (v4/*:: as any*/)
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "TopPageArtistsPaginationQuery",
    "selections": [
      {
        "args": [
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          (v4/*:: as any*/)
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
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "TopPageArtistsPaginationQuery",
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
            "args": (v5/*:: as any*/),
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
            "args": (v5/*:: as any*/),
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
    "cacheID": "7bd472c1e54a4075f77bb03e2e1880ba",
    "id": null,
    "metadata": {},
    "name": "TopPageArtistsPaginationQuery",
    "operationKind": "query",
    "text": "query TopPageArtistsPaginationQuery(\n  $after: String\n  $by: RankBy!\n  $filter: StatsFilter!\n  $first: Int = 50\n) {\n  ...TopPageArtists_query_19NMb\n}\n\nfragment TopPageArtists_query_19NMb on Query {\n  stats(filter: $filter) {\n    topArtists(first: $first, after: $after, by: $by) {\n      edges {\n        rank\n        plays\n        minutesPlayed\n        node {\n          id\n          name\n          __typename\n        }\n        cursor\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "cb30ece6599ecc8d37aeb6a029bc367e";

export default node;
