/**
 * @generated SignedSource<<3f52dc91c2a7980da0395fb92b8029c1>>
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
export type TopPageTracksPaginationQuery$variables = {
  after?: string | null | undefined;
  by: RankBy;
  filter: StatsFilter;
  first?: number | null | undefined;
};
export type TopPageTracksPaginationQuery$data = {
  readonly " $fragmentSpreads": FragmentRefs<"TopPageTracks_query">;
};
export type TopPageTracksPaginationQuery = {
  response: TopPageTracksPaginationQuery$data;
  variables: TopPageTracksPaginationQuery$variables;
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
    "name": "TopPageTracksPaginationQuery",
    "selections": [
      {
        "args": [
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          (v4/*:: as any*/)
        ],
        "kind": "FragmentSpread",
        "name": "TopPageTracks_query"
      }
    ],
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "TopPageTracksPaginationQuery",
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
            "concreteType": "RankedItemConnection",
            "kind": "LinkedField",
            "name": "topItems",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "RankedItemEdge",
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
                    "concreteType": "Item",
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
                        "name": "uri",
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
            "key": "TopPageTracks_topItems",
            "kind": "LinkedHandle",
            "name": "topItems"
          }
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "8febc20873225a0fb9e839499c72cd83",
    "id": null,
    "metadata": {},
    "name": "TopPageTracksPaginationQuery",
    "operationKind": "query",
    "text": "query TopPageTracksPaginationQuery(\n  $after: String\n  $by: RankBy!\n  $filter: StatsFilter!\n  $first: Int = 50\n) {\n  ...TopPageTracks_query_19NMb\n}\n\nfragment TopPageTracks_query_19NMb on Query {\n  stats(filter: $filter) {\n    topItems(first: $first, after: $after, by: $by) {\n      edges {\n        rank\n        plays\n        minutesPlayed\n        node {\n          id\n          uri\n          name\n          artistName\n          __typename\n        }\n        cursor\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "cfb52510986633cf98ab08c20bad3b51";

export default node;
