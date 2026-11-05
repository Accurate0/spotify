/**
 * @generated SignedSource<<a1e1253449744815f3a393877234a994>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type SpotifyItemKind = "AUDIOBOOK" | "EPISODE" | "TRACK" | "%future added value";
export type SpotifyStatsFilter = {
  accounts?: ReadonlyArray<string>;
  from?: string | null | undefined;
  kinds?: ReadonlyArray<SpotifyItemKind>;
  minMsPlayed?: number;
  timeZone?: string;
  to?: string | null | undefined;
};
export type PlayHistoryPaginationQuery$variables = {
  after?: string | null | undefined;
  filter: SpotifyStatsFilter;
  first?: number | null | undefined;
  id: string;
};
export type PlayHistoryPaginationQuery$data = {
  readonly node: {
    readonly " $fragmentSpreads": FragmentRefs<"PlayHistory_item">;
  } | null | undefined;
};
export type PlayHistoryPaginationQuery = {
  response: PlayHistoryPaginationQuery$data;
  variables: PlayHistoryPaginationQuery$variables;
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
    "name": "filter"
  },
  {
    "defaultValue": 20,
    "kind": "LocalArgument",
    "name": "first"
  },
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "id"
  }
],
v1 = [
  {
    "kind": "Variable",
    "name": "id",
    "variableName": "id"
  }
],
v2 = {
  "kind": "Variable",
  "name": "after",
  "variableName": "after"
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
v5 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "__typename",
  "storageKey": null
},
v6 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v7 = [
  (v2/*:: as any*/),
  (v4/*:: as any*/)
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "PlayHistoryPaginationQuery",
    "selections": [
      {
        "alias": null,
        "args": (v1/*:: as any*/),
        "concreteType": null,
        "kind": "LinkedField",
        "name": "node",
        "plural": false,
        "selections": [
          {
            "args": [
              (v2/*:: as any*/),
              (v3/*:: as any*/),
              (v4/*:: as any*/)
            ],
            "kind": "FragmentSpread",
            "name": "PlayHistory_item"
          }
        ],
        "storageKey": null
      }
    ],
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "PlayHistoryPaginationQuery",
    "selections": [
      {
        "alias": null,
        "args": (v1/*:: as any*/),
        "concreteType": null,
        "kind": "LinkedField",
        "name": "node",
        "plural": false,
        "selections": [
          (v5/*:: as any*/),
          (v6/*:: as any*/),
          {
            "kind": "InlineFragment",
            "selections": [
              {
                "alias": null,
                "args": [
                  (v3/*:: as any*/)
                ],
                "concreteType": "SpotifyStats",
                "kind": "LinkedField",
                "name": "stats",
                "plural": false,
                "selections": [
                  {
                    "alias": null,
                    "args": (v7/*:: as any*/),
                    "concreteType": "SpotifyPlayConnection",
                    "kind": "LinkedField",
                    "name": "plays",
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
                                  },
                                  (v6/*:: as any*/)
                                ],
                                "storageKey": null
                              },
                              (v5/*:: as any*/)
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
                    "args": (v7/*:: as any*/),
                    "filters": null,
                    "handle": "connection",
                    "key": "PlayHistory_plays",
                    "kind": "LinkedHandle",
                    "name": "plays"
                  }
                ],
                "storageKey": null
              }
            ],
            "type": "SpotifyItem",
            "abstractKey": null
          }
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "c66c3aca62f9c0ea80a667664c390f90",
    "id": null,
    "metadata": {},
    "name": "PlayHistoryPaginationQuery",
    "operationKind": "query",
    "text": "query PlayHistoryPaginationQuery(\n  $after: String\n  $filter: SpotifyStatsFilter!\n  $first: Int = 20\n  $id: ID!\n) {\n  node(id: $id) {\n    __typename\n    ...PlayHistory_item_G9cLv\n    id\n  }\n}\n\nfragment PlayHistory_item_G9cLv on SpotifyItem {\n  stats(filter: $filter) {\n    plays(first: $first, after: $after) {\n      edges {\n        node {\n          ts\n          msPlayed\n          skipped\n          account {\n            displayName\n            id\n          }\n          __typename\n        }\n        cursor\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n  id\n}\n"
  }
};
})();

(node as any).hash = "940c285d18eab509d38ee4c68212200f";

export default node;
