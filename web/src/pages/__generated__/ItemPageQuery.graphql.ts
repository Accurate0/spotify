/**
 * @generated SignedSource<<2955855061c8a13f2103ab43b3c652a3>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type ItemKind = "AUDIOBOOK" | "EPISODE" | "TRACK" | "%future added value";
export type StatsFilter = {
  accounts?: ReadonlyArray<string>;
  from?: string | null | undefined;
  kinds?: ReadonlyArray<ItemKind>;
  minMsPlayed?: number;
  timeZone?: string;
  to?: string | null | undefined;
};
export type ItemPageQuery$variables = {
  filter: StatsFilter;
  uri: string;
};
export type ItemPageQuery$data = {
  readonly item: {
    readonly album: {
      readonly name: string;
    } | null | undefined;
    readonly artist: {
      readonly name: string;
    } | null | undefined;
    readonly artistName: string | null | undefined;
    readonly kind: ItemKind;
    readonly name: string | null | undefined;
    readonly stats: {
      readonly summary: {
        readonly " $fragmentSpreads": FragmentRefs<"StatTiles_summary">;
      };
      readonly timeline: ReadonlyArray<{
        readonly " $fragmentSpreads": FragmentRefs<"TimelineChart_points">;
      }>;
      readonly " $fragmentSpreads": FragmentRefs<"Firsts_stats">;
    };
    readonly uri: string;
    readonly " $fragmentSpreads": FragmentRefs<"PlayHistory_item">;
  } | null | undefined;
};
export type ItemPageQuery = {
  response: ItemPageQuery$data;
  variables: ItemPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "filter"
},
v1 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "uri"
},
v2 = [
  {
    "kind": "Variable",
    "name": "uri",
    "variableName": "uri"
  }
],
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "uri",
  "storageKey": null
},
v4 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "kind",
  "storageKey": null
},
v5 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v6 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "artistName",
  "storageKey": null
},
v7 = [
  (v5/*:: as any*/)
],
v8 = {
  "alias": null,
  "args": null,
  "concreteType": "Album",
  "kind": "LinkedField",
  "name": "album",
  "plural": false,
  "selections": (v7/*:: as any*/),
  "storageKey": null
},
v9 = [
  {
    "kind": "Variable",
    "name": "filter",
    "variableName": "filter"
  }
],
v10 = [
  {
    "kind": "Literal",
    "name": "bucket",
    "value": "MONTH"
  }
],
v11 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v12 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "plays",
  "storageKey": null
},
v13 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "minutesPlayed",
  "storageKey": null
},
v14 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "ts",
  "storageKey": null
},
v15 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "start",
  "storageKey": null
},
v16 = [
  {
    "kind": "Literal",
    "name": "first",
    "value": 20
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
    "name": "ItemPageQuery",
    "selections": [
      {
        "alias": null,
        "args": (v2/*:: as any*/),
        "concreteType": "Item",
        "kind": "LinkedField",
        "name": "item",
        "plural": false,
        "selections": [
          (v3/*:: as any*/),
          (v4/*:: as any*/),
          (v5/*:: as any*/),
          (v6/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "concreteType": "Artist",
            "kind": "LinkedField",
            "name": "artist",
            "plural": false,
            "selections": (v7/*:: as any*/),
            "storageKey": null
          },
          (v8/*:: as any*/),
          {
            "alias": null,
            "args": (v9/*:: as any*/),
            "concreteType": "Stats",
            "kind": "LinkedField",
            "name": "stats",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "Summary",
                "kind": "LinkedField",
                "name": "summary",
                "plural": false,
                "selections": [
                  {
                    "args": null,
                    "kind": "FragmentSpread",
                    "name": "StatTiles_summary"
                  }
                ],
                "storageKey": null
              },
              {
                "args": null,
                "kind": "FragmentSpread",
                "name": "Firsts_stats"
              },
              {
                "alias": null,
                "args": (v10/*:: as any*/),
                "concreteType": "TimelinePoint",
                "kind": "LinkedField",
                "name": "timeline",
                "plural": true,
                "selections": [
                  {
                    "args": null,
                    "kind": "FragmentSpread",
                    "name": "TimelineChart_points"
                  }
                ],
                "storageKey": "timeline(bucket:\"MONTH\")"
              }
            ],
            "storageKey": null
          },
          {
            "args": (v9/*:: as any*/),
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
    "argumentDefinitions": [
      (v1/*:: as any*/),
      (v0/*:: as any*/)
    ],
    "kind": "Operation",
    "name": "ItemPageQuery",
    "selections": [
      {
        "alias": null,
        "args": (v2/*:: as any*/),
        "concreteType": "Item",
        "kind": "LinkedField",
        "name": "item",
        "plural": false,
        "selections": [
          (v3/*:: as any*/),
          (v4/*:: as any*/),
          (v5/*:: as any*/),
          (v6/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "concreteType": "Artist",
            "kind": "LinkedField",
            "name": "artist",
            "plural": false,
            "selections": [
              (v5/*:: as any*/),
              (v11/*:: as any*/)
            ],
            "storageKey": null
          },
          (v8/*:: as any*/),
          {
            "alias": null,
            "args": (v9/*:: as any*/),
            "concreteType": "Stats",
            "kind": "LinkedField",
            "name": "stats",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "Summary",
                "kind": "LinkedField",
                "name": "summary",
                "plural": false,
                "selections": [
                  (v12/*:: as any*/),
                  (v13/*:: as any*/),
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "distinctTracks",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "distinctArtists",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "skipRate",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "firstPlayed",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "lastPlayed",
                    "storageKey": null
                  }
                ],
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "concreteType": "Play",
                "kind": "LinkedField",
                "name": "firstPlay",
                "plural": false,
                "selections": [
                  (v14/*:: as any*/),
                  {
                    "alias": null,
                    "args": null,
                    "concreteType": "Item",
                    "kind": "LinkedField",
                    "name": "item",
                    "plural": false,
                    "selections": [
                      (v3/*:: as any*/),
                      (v5/*:: as any*/),
                      (v11/*:: as any*/)
                    ],
                    "storageKey": null
                  }
                ],
                "storageKey": null
              },
              {
                "alias": null,
                "args": [
                  {
                    "kind": "Literal",
                    "name": "first",
                    "value": 1
                  }
                ],
                "concreteType": "DayStat",
                "kind": "LinkedField",
                "name": "topDays",
                "plural": true,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "date",
                    "storageKey": null
                  },
                  (v12/*:: as any*/),
                  (v13/*:: as any*/)
                ],
                "storageKey": "topDays(first:1)"
              },
              {
                "alias": null,
                "args": null,
                "concreteType": "ListeningDays",
                "kind": "LinkedField",
                "name": "listeningDays",
                "plural": false,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "activeDays",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "concreteType": "Streak",
                    "kind": "LinkedField",
                    "name": "longestStreak",
                    "plural": false,
                    "selections": [
                      {
                        "alias": null,
                        "args": null,
                        "kind": "ScalarField",
                        "name": "days",
                        "storageKey": null
                      },
                      (v15/*:: as any*/),
                      {
                        "alias": null,
                        "args": null,
                        "kind": "ScalarField",
                        "name": "end",
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
                "args": (v10/*:: as any*/),
                "concreteType": "TimelinePoint",
                "kind": "LinkedField",
                "name": "timeline",
                "plural": true,
                "selections": [
                  (v15/*:: as any*/),
                  (v12/*:: as any*/),
                  (v13/*:: as any*/)
                ],
                "storageKey": "timeline(bucket:\"MONTH\")"
              },
              {
                "alias": null,
                "args": (v16/*:: as any*/),
                "concreteType": "PlayConnection",
                "kind": "LinkedField",
                "name": "plays",
                "plural": false,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "concreteType": "PlayEdge",
                    "kind": "LinkedField",
                    "name": "edges",
                    "plural": true,
                    "selections": [
                      {
                        "alias": null,
                        "args": null,
                        "concreteType": "Play",
                        "kind": "LinkedField",
                        "name": "node",
                        "plural": false,
                        "selections": [
                          (v14/*:: as any*/),
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
                            "concreteType": "Account",
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
                              (v11/*:: as any*/)
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
                "storageKey": "plays(first:20)"
              },
              {
                "alias": null,
                "args": (v16/*:: as any*/),
                "filters": null,
                "handle": "connection",
                "key": "PlayHistory_plays",
                "kind": "LinkedHandle",
                "name": "plays"
              }
            ],
            "storageKey": null
          },
          (v11/*:: as any*/)
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "57b072aa84f444a582cd8c47ce34e36e",
    "id": null,
    "metadata": {},
    "name": "ItemPageQuery",
    "operationKind": "query",
    "text": "query ItemPageQuery(\n  $uri: String!\n  $filter: StatsFilter!\n) {\n  item(uri: $uri) {\n    uri\n    kind\n    name\n    artistName\n    artist {\n      name\n      id\n    }\n    album {\n      name\n    }\n    stats(filter: $filter) {\n      summary {\n        ...StatTiles_summary\n      }\n      ...Firsts_stats\n      timeline(bucket: MONTH) {\n        ...TimelineChart_points\n      }\n    }\n    ...PlayHistory_item_Vt7Yj\n    id\n  }\n}\n\nfragment Firsts_stats on Stats {\n  firstPlay {\n    ts\n    item {\n      uri\n      name\n      id\n    }\n  }\n  topDays(first: 1) {\n    date\n    plays\n    minutesPlayed\n  }\n  listeningDays {\n    activeDays\n    longestStreak {\n      days\n      start\n      end\n    }\n  }\n}\n\nfragment PlayHistory_item_Vt7Yj on Item {\n  stats(filter: $filter) {\n    plays(first: 20) {\n      edges {\n        node {\n          ts\n          msPlayed\n          skipped\n          account {\n            displayName\n            id\n          }\n          __typename\n        }\n        cursor\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n  id\n}\n\nfragment StatTiles_summary on Summary {\n  plays\n  minutesPlayed\n  distinctTracks\n  distinctArtists\n  skipRate\n  firstPlayed\n  lastPlayed\n}\n\nfragment TimelineChart_points on TimelinePoint {\n  start\n  plays\n  minutesPlayed\n}\n"
  }
};
})();

(node as any).hash = "ead202d4d4d3462e6e91bb5de9abbe18";

export default node;
