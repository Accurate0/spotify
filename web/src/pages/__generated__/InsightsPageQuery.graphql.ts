/**
 * @generated SignedSource<<4d0f85b42524aa9f7132d2c356c2e4e9>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type ItemKind = "AUDIOBOOK" | "EPISODE" | "TRACK" | "%future added value";
export type StatsFilter = {
  accounts?: ReadonlyArray<string>;
  from?: string | null | undefined;
  kinds?: ReadonlyArray<ItemKind>;
  minMsPlayed?: number;
  timeZone?: string;
  to?: string | null | undefined;
};
export type InsightsPageQuery$variables = {
  allPlays: StatsFilter;
  filter: StatsFilter;
};
export type InsightsPageQuery$data = {
  readonly skips: {
    readonly mostSkipped: ReadonlyArray<{
      readonly item: {
        readonly artistName: string | null | undefined;
        readonly id: string;
        readonly name: string | null | undefined;
        readonly uri: string;
      };
      readonly plays: number;
      readonly skipRate: number;
      readonly skips: number;
    }>;
  };
  readonly stats: {
    readonly country: ReadonlyArray<{
      readonly label: string;
      readonly minutesPlayed: number;
      readonly plays: number;
    }>;
    readonly device: ReadonlyArray<{
      readonly label: string;
      readonly minutesPlayed: number;
      readonly plays: number;
    }>;
    readonly discoveries: {
      readonly edges: ReadonlyArray<{
        readonly node: {
          readonly artist: {
            readonly id: string;
            readonly name: string;
          };
          readonly firstItem: {
            readonly name: string | null | undefined;
          } | null | undefined;
          readonly firstPlayed: string;
          readonly minutesPlayed: number;
        };
      }>;
    };
    readonly endReason: ReadonlyArray<{
      readonly label: string;
      readonly minutesPlayed: number;
      readonly plays: number;
    }>;
    readonly forgottenArtists: {
      readonly edges: ReadonlyArray<{
        readonly node: {
          readonly artist: {
            readonly id: string;
            readonly name: string;
          };
          readonly lastPlayed: string;
          readonly minutesPlayed: number;
        };
      }>;
    };
    readonly listeningDays: {
      readonly activeDays: number;
      readonly latestStreak: {
        readonly days: number;
        readonly end: string;
        readonly start: string;
      } | null | undefined;
      readonly longestStreak: {
        readonly days: number;
        readonly end: string;
        readonly start: string;
      } | null | undefined;
      readonly spanDays: number;
    };
    readonly offline: ReadonlyArray<{
      readonly label: string;
      readonly minutesPlayed: number;
      readonly plays: number;
    }>;
    readonly onRepeat: ReadonlyArray<{
      readonly date: string;
      readonly item: {
        readonly artistName: string | null | undefined;
        readonly id: string;
        readonly name: string | null | undefined;
        readonly uri: string;
      };
      readonly plays: number;
    }>;
    readonly sessions: {
      readonly averageMinutes: number | null | undefined;
      readonly count: number;
      readonly longest: {
        readonly minutesPlayed: number;
        readonly plays: number;
        readonly start: string;
      } | null | undefined;
    };
    readonly shuffle: ReadonlyArray<{
      readonly label: string;
      readonly minutesPlayed: number;
      readonly plays: number;
    }>;
    readonly topDays: ReadonlyArray<{
      readonly date: string;
      readonly minutesPlayed: number;
      readonly plays: number;
      readonly topArtist: {
        readonly name: string;
      } | null | undefined;
    }>;
  };
};
export type InsightsPageQuery = {
  response: InsightsPageQuery$data;
  variables: InsightsPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "allPlays"
},
v1 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "filter"
},
v2 = [
  {
    "kind": "Variable",
    "name": "filter",
    "variableName": "filter"
  }
],
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "start",
  "storageKey": null
},
v4 = [
  (v3/*:: as any*/),
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "end",
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "days",
    "storageKey": null
  }
],
v5 = {
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
      "kind": "ScalarField",
      "name": "spanDays",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "concreteType": "Streak",
      "kind": "LinkedField",
      "name": "longestStreak",
      "plural": false,
      "selections": (v4/*:: as any*/),
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "concreteType": "Streak",
      "kind": "LinkedField",
      "name": "latestStreak",
      "plural": false,
      "selections": (v4/*:: as any*/),
      "storageKey": null
    }
  ],
  "storageKey": null
},
v6 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "plays",
  "storageKey": null
},
v7 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "minutesPlayed",
  "storageKey": null
},
v8 = {
  "alias": null,
  "args": null,
  "concreteType": "Sessions",
  "kind": "LinkedField",
  "name": "sessions",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "count",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "averageMinutes",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "concreteType": "Session",
      "kind": "LinkedField",
      "name": "longest",
      "plural": false,
      "selections": [
        (v3/*:: as any*/),
        (v6/*:: as any*/),
        (v7/*:: as any*/)
      ],
      "storageKey": null
    }
  ],
  "storageKey": null
},
v9 = [
  {
    "kind": "Literal",
    "name": "first",
    "value": 8
  }
],
v10 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "date",
  "storageKey": null
},
v11 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v12 = [
  (v11/*:: as any*/)
],
v13 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v14 = {
  "alias": null,
  "args": null,
  "concreteType": "Item",
  "kind": "LinkedField",
  "name": "item",
  "plural": false,
  "selections": [
    (v13/*:: as any*/),
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "uri",
      "storageKey": null
    },
    (v11/*:: as any*/),
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "artistName",
      "storageKey": null
    }
  ],
  "storageKey": null
},
v15 = {
  "alias": null,
  "args": (v9/*:: as any*/),
  "concreteType": "RepeatDay",
  "kind": "LinkedField",
  "name": "onRepeat",
  "plural": true,
  "selections": [
    (v10/*:: as any*/),
    (v6/*:: as any*/),
    (v14/*:: as any*/)
  ],
  "storageKey": "onRepeat(first:8)"
},
v16 = [
  {
    "kind": "Literal",
    "name": "first",
    "value": 10
  }
],
v17 = {
  "alias": null,
  "args": null,
  "concreteType": "Artist",
  "kind": "LinkedField",
  "name": "artist",
  "plural": false,
  "selections": [
    (v13/*:: as any*/),
    (v11/*:: as any*/)
  ],
  "storageKey": null
},
v18 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "firstPlayed",
  "storageKey": null
},
v19 = {
  "alias": null,
  "args": (v16/*:: as any*/),
  "concreteType": "ForgottenArtistConnection",
  "kind": "LinkedField",
  "name": "forgottenArtists",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": null,
      "concreteType": "ForgottenArtistEdge",
      "kind": "LinkedField",
      "name": "edges",
      "plural": true,
      "selections": [
        {
          "alias": null,
          "args": null,
          "concreteType": "ForgottenArtist",
          "kind": "LinkedField",
          "name": "node",
          "plural": false,
          "selections": [
            (v17/*:: as any*/),
            (v7/*:: as any*/),
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "lastPlayed",
              "storageKey": null
            }
          ],
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "storageKey": "forgottenArtists(first:10)"
},
v20 = [
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "label",
    "storageKey": null
  },
  (v6/*:: as any*/),
  (v7/*:: as any*/)
],
v21 = {
  "alias": "device",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "DEVICE"
    }
  ],
  "concreteType": "BreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": (v20/*:: as any*/),
  "storageKey": "breakdown(by:\"DEVICE\")"
},
v22 = {
  "alias": "country",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "COUNTRY"
    }
  ],
  "concreteType": "BreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": (v20/*:: as any*/),
  "storageKey": "breakdown(by:\"COUNTRY\")"
},
v23 = {
  "alias": "shuffle",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "SHUFFLE"
    }
  ],
  "concreteType": "BreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": (v20/*:: as any*/),
  "storageKey": "breakdown(by:\"SHUFFLE\")"
},
v24 = {
  "alias": "offline",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "OFFLINE"
    }
  ],
  "concreteType": "BreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": (v20/*:: as any*/),
  "storageKey": "breakdown(by:\"OFFLINE\")"
},
v25 = {
  "alias": "endReason",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "END_REASON"
    }
  ],
  "concreteType": "BreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": (v20/*:: as any*/),
  "storageKey": "breakdown(by:\"END_REASON\")"
},
v26 = {
  "alias": "skips",
  "args": [
    {
      "kind": "Variable",
      "name": "filter",
      "variableName": "allPlays"
    }
  ],
  "concreteType": "Stats",
  "kind": "LinkedField",
  "name": "stats",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": (v9/*:: as any*/),
      "concreteType": "SkippedItem",
      "kind": "LinkedField",
      "name": "mostSkipped",
      "plural": true,
      "selections": [
        (v6/*:: as any*/),
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "skips",
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "skipRate",
          "storageKey": null
        },
        (v14/*:: as any*/)
      ],
      "storageKey": "mostSkipped(first:8)"
    }
  ],
  "storageKey": null
},
v27 = [
  (v11/*:: as any*/),
  (v13/*:: as any*/)
];
return {
  "fragment": {
    "argumentDefinitions": [
      (v0/*:: as any*/),
      (v1/*:: as any*/)
    ],
    "kind": "Fragment",
    "metadata": null,
    "name": "InsightsPageQuery",
    "selections": [
      {
        "alias": null,
        "args": (v2/*:: as any*/),
        "concreteType": "Stats",
        "kind": "LinkedField",
        "name": "stats",
        "plural": false,
        "selections": [
          (v5/*:: as any*/),
          (v8/*:: as any*/),
          {
            "alias": null,
            "args": (v9/*:: as any*/),
            "concreteType": "DayStat",
            "kind": "LinkedField",
            "name": "topDays",
            "plural": true,
            "selections": [
              (v10/*:: as any*/),
              (v6/*:: as any*/),
              (v7/*:: as any*/),
              {
                "alias": null,
                "args": null,
                "concreteType": "Artist",
                "kind": "LinkedField",
                "name": "topArtist",
                "plural": false,
                "selections": (v12/*:: as any*/),
                "storageKey": null
              }
            ],
            "storageKey": "topDays(first:8)"
          },
          (v15/*:: as any*/),
          {
            "alias": null,
            "args": (v16/*:: as any*/),
            "concreteType": "DiscoveryConnection",
            "kind": "LinkedField",
            "name": "discoveries",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "DiscoveryEdge",
                "kind": "LinkedField",
                "name": "edges",
                "plural": true,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "concreteType": "Discovery",
                    "kind": "LinkedField",
                    "name": "node",
                    "plural": false,
                    "selections": [
                      (v17/*:: as any*/),
                      (v18/*:: as any*/),
                      (v7/*:: as any*/),
                      {
                        "alias": null,
                        "args": null,
                        "concreteType": "Item",
                        "kind": "LinkedField",
                        "name": "firstItem",
                        "plural": false,
                        "selections": (v12/*:: as any*/),
                        "storageKey": null
                      }
                    ],
                    "storageKey": null
                  }
                ],
                "storageKey": null
              }
            ],
            "storageKey": "discoveries(first:10)"
          },
          (v19/*:: as any*/),
          (v21/*:: as any*/),
          (v22/*:: as any*/),
          (v23/*:: as any*/),
          (v24/*:: as any*/),
          (v25/*:: as any*/)
        ],
        "storageKey": null
      },
      (v26/*:: as any*/)
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
    "name": "InsightsPageQuery",
    "selections": [
      {
        "alias": null,
        "args": (v2/*:: as any*/),
        "concreteType": "Stats",
        "kind": "LinkedField",
        "name": "stats",
        "plural": false,
        "selections": [
          (v5/*:: as any*/),
          (v8/*:: as any*/),
          {
            "alias": null,
            "args": (v9/*:: as any*/),
            "concreteType": "DayStat",
            "kind": "LinkedField",
            "name": "topDays",
            "plural": true,
            "selections": [
              (v10/*:: as any*/),
              (v6/*:: as any*/),
              (v7/*:: as any*/),
              {
                "alias": null,
                "args": null,
                "concreteType": "Artist",
                "kind": "LinkedField",
                "name": "topArtist",
                "plural": false,
                "selections": (v27/*:: as any*/),
                "storageKey": null
              }
            ],
            "storageKey": "topDays(first:8)"
          },
          (v15/*:: as any*/),
          {
            "alias": null,
            "args": (v16/*:: as any*/),
            "concreteType": "DiscoveryConnection",
            "kind": "LinkedField",
            "name": "discoveries",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "DiscoveryEdge",
                "kind": "LinkedField",
                "name": "edges",
                "plural": true,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "concreteType": "Discovery",
                    "kind": "LinkedField",
                    "name": "node",
                    "plural": false,
                    "selections": [
                      (v17/*:: as any*/),
                      (v18/*:: as any*/),
                      (v7/*:: as any*/),
                      {
                        "alias": null,
                        "args": null,
                        "concreteType": "Item",
                        "kind": "LinkedField",
                        "name": "firstItem",
                        "plural": false,
                        "selections": (v27/*:: as any*/),
                        "storageKey": null
                      }
                    ],
                    "storageKey": null
                  }
                ],
                "storageKey": null
              }
            ],
            "storageKey": "discoveries(first:10)"
          },
          (v19/*:: as any*/),
          (v21/*:: as any*/),
          (v22/*:: as any*/),
          (v23/*:: as any*/),
          (v24/*:: as any*/),
          (v25/*:: as any*/)
        ],
        "storageKey": null
      },
      (v26/*:: as any*/)
    ]
  },
  "params": {
    "cacheID": "e161c81dd0610c762d09eb603bbbca99",
    "id": null,
    "metadata": {},
    "name": "InsightsPageQuery",
    "operationKind": "query",
    "text": "query InsightsPageQuery(\n  $filter: StatsFilter!\n  $allPlays: StatsFilter!\n) {\n  stats(filter: $filter) {\n    listeningDays {\n      activeDays\n      spanDays\n      longestStreak {\n        start\n        end\n        days\n      }\n      latestStreak {\n        start\n        end\n        days\n      }\n    }\n    sessions {\n      count\n      averageMinutes\n      longest {\n        start\n        plays\n        minutesPlayed\n      }\n    }\n    topDays(first: 8) {\n      date\n      plays\n      minutesPlayed\n      topArtist {\n        name\n        id\n      }\n    }\n    onRepeat(first: 8) {\n      date\n      plays\n      item {\n        id\n        uri\n        name\n        artistName\n      }\n    }\n    discoveries(first: 10) {\n      edges {\n        node {\n          artist {\n            id\n            name\n          }\n          firstPlayed\n          minutesPlayed\n          firstItem {\n            name\n            id\n          }\n        }\n      }\n    }\n    forgottenArtists(first: 10) {\n      edges {\n        node {\n          artist {\n            id\n            name\n          }\n          minutesPlayed\n          lastPlayed\n        }\n      }\n    }\n    device: breakdown(by: DEVICE) {\n      label\n      plays\n      minutesPlayed\n    }\n    country: breakdown(by: COUNTRY) {\n      label\n      plays\n      minutesPlayed\n    }\n    shuffle: breakdown(by: SHUFFLE) {\n      label\n      plays\n      minutesPlayed\n    }\n    offline: breakdown(by: OFFLINE) {\n      label\n      plays\n      minutesPlayed\n    }\n    endReason: breakdown(by: END_REASON) {\n      label\n      plays\n      minutesPlayed\n    }\n  }\n  skips: stats(filter: $allPlays) {\n    mostSkipped(first: 8) {\n      plays\n      skips\n      skipRate\n      item {\n        id\n        uri\n        name\n        artistName\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "6638df36d5ce8de8059951aca25f1b79";

export default node;
