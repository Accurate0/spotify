/**
 * @generated SignedSource<<c754826ca2c9d7c8e007d7154e110ae6>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type SpotifyItemKind = "AUDIOBOOK" | "EPISODE" | "TRACK" | "%future added value";
export type SpotifyTimeBucket = "DAY" | "MONTH" | "WEEK" | "YEAR" | "%future added value";
export type SpotifyStatsFilter = {
  accounts?: ReadonlyArray<string>;
  from?: string | null | undefined;
  kinds?: ReadonlyArray<SpotifyItemKind>;
  minMsPlayed?: number;
  timeZone?: string;
  to?: string | null | undefined;
};
export type InsightsPageQuery$variables = {
  allPlays: SpotifyStatsFilter;
  bucket: SpotifyTimeBucket;
  filter: SpotifyStatsFilter;
};
export type InsightsPageQuery$data = {
  readonly skips: {
    readonly completion: {
      readonly average: number | null | undefined;
      readonly finishedShare: number | null | undefined;
    };
    readonly leastSkippedArtists: ReadonlyArray<{
      readonly artist: {
        readonly id: string;
        readonly imageUrl: string | null | undefined;
        readonly name: string;
      };
      readonly plays: number;
      readonly rank: number;
      readonly skipRate: number;
      readonly skips: number;
    }>;
    readonly mostAbandoned: ReadonlyArray<{
      readonly averageCompletion: number;
      readonly item: {
        readonly artistName: string | null | undefined;
        readonly artworkUrl: string | null | undefined;
        readonly id: string;
        readonly name: string | null | undefined;
        readonly uri: string;
      };
      readonly plays: number;
    }>;
    readonly mostSkipped: ReadonlyArray<{
      readonly item: {
        readonly artistName: string | null | undefined;
        readonly artworkUrl: string | null | undefined;
        readonly id: string;
        readonly name: string | null | undefined;
        readonly uri: string;
      };
      readonly plays: number;
      readonly skipRate: number;
      readonly skips: number;
    }>;
    readonly mostSkippedArtists: ReadonlyArray<{
      readonly artist: {
        readonly id: string;
        readonly imageUrl: string | null | undefined;
        readonly name: string;
      };
      readonly plays: number;
      readonly rank: number;
      readonly skipRate: number;
      readonly skips: number;
    }>;
  };
  readonly spotifyEnrichment: {
    readonly listeningCoverage: number;
  };
  readonly spotifyStats: {
    readonly country: ReadonlyArray<{
      readonly label: string;
      readonly minutesPlayed: number;
      readonly plays: number;
    }>;
    readonly decades: ReadonlyArray<{
      readonly key: string | null | undefined;
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
            readonly imageUrl: string | null | undefined;
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
            readonly imageUrl: string | null | undefined;
            readonly name: string;
          };
          readonly lastPlayed: string;
          readonly minutesPlayed: number;
        };
      }>;
    };
    readonly genreTimeline: ReadonlyArray<{
      readonly genre: string;
      readonly points: ReadonlyArray<{
        readonly minutesPlayed: number;
        readonly start: string;
      }>;
    }>;
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
    readonly musicAge: {
      readonly averageYears: number | null | undefined;
      readonly newShare: number | null | undefined;
      readonly oldest: {
        readonly artistName: string | null | undefined;
        readonly name: string | null | undefined;
      } | null | undefined;
      readonly oldestYear: number | null | undefined;
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
        readonly artworkUrl: string | null | undefined;
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
    readonly topGenres: {
      readonly edges: ReadonlyArray<{
        readonly minutesPlayed: number;
        readonly node: {
          readonly name: string;
        };
        readonly plays: number;
        readonly rank: number;
      }>;
    };
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
  "name": "bucket"
},
v2 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "filter"
},
v3 = {
  "alias": null,
  "args": null,
  "concreteType": "SpotifyEnrichment",
  "kind": "LinkedField",
  "name": "spotifyEnrichment",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "listeningCoverage",
      "storageKey": null
    }
  ],
  "storageKey": null
},
v4 = [
  {
    "kind": "Variable",
    "name": "filter",
    "variableName": "filter"
  }
],
v5 = {
  "kind": "Literal",
  "name": "first",
  "value": 10
},
v6 = [
  (v5/*:: as any*/)
],
v7 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "rank",
  "storageKey": null
},
v8 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "plays",
  "storageKey": null
},
v9 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "minutesPlayed",
  "storageKey": null
},
v10 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v11 = [
  (v10/*:: as any*/)
],
v12 = {
  "alias": null,
  "args": (v6/*:: as any*/),
  "concreteType": "SpotifyGenreRankedConnection",
  "kind": "LinkedField",
  "name": "topGenres",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": null,
      "concreteType": "SpotifyGenreRankedEdge",
      "kind": "LinkedField",
      "name": "edges",
      "plural": true,
      "selections": [
        (v7/*:: as any*/),
        (v8/*:: as any*/),
        (v9/*:: as any*/),
        {
          "alias": null,
          "args": null,
          "concreteType": "SpotifyGenre",
          "kind": "LinkedField",
          "name": "node",
          "plural": false,
          "selections": (v11/*:: as any*/),
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "storageKey": "topGenres(first:10)"
},
v13 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "start",
  "storageKey": null
},
v14 = {
  "alias": null,
  "args": [
    {
      "kind": "Variable",
      "name": "bucket",
      "variableName": "bucket"
    }
  ],
  "concreteType": "SpotifyGenreSeries",
  "kind": "LinkedField",
  "name": "genreTimeline",
  "plural": true,
  "selections": [
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "genre",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "concreteType": "SpotifyGenrePoint",
      "kind": "LinkedField",
      "name": "points",
      "plural": true,
      "selections": [
        (v13/*:: as any*/),
        (v9/*:: as any*/)
      ],
      "storageKey": null
    }
  ],
  "storageKey": null
},
v15 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "averageYears",
  "storageKey": null
},
v16 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "newShare",
  "storageKey": null
},
v17 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "oldestYear",
  "storageKey": null
},
v18 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "artistName",
  "storageKey": null
},
v19 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "label",
  "storageKey": null
},
v20 = {
  "alias": "decades",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "DECADE"
    }
  ],
  "concreteType": "SpotifyBreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": [
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "key",
      "storageKey": null
    },
    (v19/*:: as any*/),
    (v8/*:: as any*/),
    (v9/*:: as any*/)
  ],
  "storageKey": "breakdown(by:\"DECADE\")"
},
v21 = [
  (v13/*:: as any*/),
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
v22 = {
  "alias": null,
  "args": null,
  "concreteType": "SpotifyListeningDays",
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
      "concreteType": "SpotifyStreak",
      "kind": "LinkedField",
      "name": "longestStreak",
      "plural": false,
      "selections": (v21/*:: as any*/),
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "concreteType": "SpotifyStreak",
      "kind": "LinkedField",
      "name": "latestStreak",
      "plural": false,
      "selections": (v21/*:: as any*/),
      "storageKey": null
    }
  ],
  "storageKey": null
},
v23 = {
  "alias": null,
  "args": null,
  "concreteType": "SpotifySessions",
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
      "concreteType": "SpotifySession",
      "kind": "LinkedField",
      "name": "longest",
      "plural": false,
      "selections": [
        (v13/*:: as any*/),
        (v8/*:: as any*/),
        (v9/*:: as any*/)
      ],
      "storageKey": null
    }
  ],
  "storageKey": null
},
v24 = [
  {
    "kind": "Literal",
    "name": "first",
    "value": 8
  }
],
v25 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "date",
  "storageKey": null
},
v26 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v27 = {
  "alias": null,
  "args": null,
  "concreteType": "SpotifyItem",
  "kind": "LinkedField",
  "name": "item",
  "plural": false,
  "selections": [
    (v26/*:: as any*/),
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "uri",
      "storageKey": null
    },
    (v10/*:: as any*/),
    (v18/*:: as any*/),
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
v28 = {
  "alias": null,
  "args": (v24/*:: as any*/),
  "concreteType": "SpotifyRepeatDay",
  "kind": "LinkedField",
  "name": "onRepeat",
  "plural": true,
  "selections": [
    (v25/*:: as any*/),
    (v8/*:: as any*/),
    (v27/*:: as any*/)
  ],
  "storageKey": "onRepeat(first:8)"
},
v29 = {
  "alias": null,
  "args": null,
  "concreteType": "SpotifyArtist",
  "kind": "LinkedField",
  "name": "artist",
  "plural": false,
  "selections": [
    (v26/*:: as any*/),
    (v10/*:: as any*/),
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "imageUrl",
      "storageKey": null
    }
  ],
  "storageKey": null
},
v30 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "firstPlayed",
  "storageKey": null
},
v31 = {
  "alias": null,
  "args": (v6/*:: as any*/),
  "concreteType": "SpotifyForgottenArtistConnection",
  "kind": "LinkedField",
  "name": "forgottenArtists",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": null,
      "concreteType": "SpotifyForgottenArtistEdge",
      "kind": "LinkedField",
      "name": "edges",
      "plural": true,
      "selections": [
        {
          "alias": null,
          "args": null,
          "concreteType": "SpotifyForgottenArtist",
          "kind": "LinkedField",
          "name": "node",
          "plural": false,
          "selections": [
            (v29/*:: as any*/),
            (v9/*:: as any*/),
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
v32 = [
  (v19/*:: as any*/),
  (v8/*:: as any*/),
  (v9/*:: as any*/)
],
v33 = {
  "alias": "device",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "DEVICE"
    }
  ],
  "concreteType": "SpotifyBreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": (v32/*:: as any*/),
  "storageKey": "breakdown(by:\"DEVICE\")"
},
v34 = {
  "alias": "country",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "COUNTRY"
    }
  ],
  "concreteType": "SpotifyBreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": (v32/*:: as any*/),
  "storageKey": "breakdown(by:\"COUNTRY\")"
},
v35 = {
  "alias": "shuffle",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "SHUFFLE"
    }
  ],
  "concreteType": "SpotifyBreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": (v32/*:: as any*/),
  "storageKey": "breakdown(by:\"SHUFFLE\")"
},
v36 = {
  "alias": "offline",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "OFFLINE"
    }
  ],
  "concreteType": "SpotifyBreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": (v32/*:: as any*/),
  "storageKey": "breakdown(by:\"OFFLINE\")"
},
v37 = {
  "alias": "endReason",
  "args": [
    {
      "kind": "Literal",
      "name": "by",
      "value": "END_REASON"
    }
  ],
  "concreteType": "SpotifyBreakdownRow",
  "kind": "LinkedField",
  "name": "breakdown",
  "plural": true,
  "selections": (v32/*:: as any*/),
  "storageKey": "breakdown(by:\"END_REASON\")"
},
v38 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "skips",
  "storageKey": null
},
v39 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "skipRate",
  "storageKey": null
},
v40 = [
  (v7/*:: as any*/),
  (v8/*:: as any*/),
  (v38/*:: as any*/),
  (v39/*:: as any*/),
  (v29/*:: as any*/)
],
v41 = {
  "alias": "skips",
  "args": [
    {
      "kind": "Variable",
      "name": "filter",
      "variableName": "allPlays"
    }
  ],
  "concreteType": "SpotifyStats",
  "kind": "LinkedField",
  "name": "spotifyStats",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": null,
      "concreteType": "SpotifyCompletion",
      "kind": "LinkedField",
      "name": "completion",
      "plural": false,
      "selections": [
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "average",
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "finishedShare",
          "storageKey": null
        }
      ],
      "storageKey": null
    },
    {
      "alias": null,
      "args": (v24/*:: as any*/),
      "concreteType": "SpotifyAbandonedItem",
      "kind": "LinkedField",
      "name": "mostAbandoned",
      "plural": true,
      "selections": [
        (v8/*:: as any*/),
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "averageCompletion",
          "storageKey": null
        },
        (v27/*:: as any*/)
      ],
      "storageKey": "mostAbandoned(first:8)"
    },
    {
      "alias": "mostSkippedArtists",
      "args": [
        (v5/*:: as any*/),
        {
          "kind": "Literal",
          "name": "order",
          "value": "MOST_SKIPPED"
        }
      ],
      "concreteType": "SpotifyArtistSkipRate",
      "kind": "LinkedField",
      "name": "artistSkipRates",
      "plural": true,
      "selections": (v40/*:: as any*/),
      "storageKey": "artistSkipRates(first:10,order:\"MOST_SKIPPED\")"
    },
    {
      "alias": "leastSkippedArtists",
      "args": [
        (v5/*:: as any*/),
        {
          "kind": "Literal",
          "name": "order",
          "value": "LEAST_SKIPPED"
        }
      ],
      "concreteType": "SpotifyArtistSkipRate",
      "kind": "LinkedField",
      "name": "artistSkipRates",
      "plural": true,
      "selections": (v40/*:: as any*/),
      "storageKey": "artistSkipRates(first:10,order:\"LEAST_SKIPPED\")"
    },
    {
      "alias": null,
      "args": (v24/*:: as any*/),
      "concreteType": "SpotifySkippedItem",
      "kind": "LinkedField",
      "name": "mostSkipped",
      "plural": true,
      "selections": [
        (v8/*:: as any*/),
        (v38/*:: as any*/),
        (v39/*:: as any*/),
        (v27/*:: as any*/)
      ],
      "storageKey": "mostSkipped(first:8)"
    }
  ],
  "storageKey": null
},
v42 = [
  (v10/*:: as any*/),
  (v26/*:: as any*/)
];
return {
  "fragment": {
    "argumentDefinitions": [
      (v0/*:: as any*/),
      (v1/*:: as any*/),
      (v2/*:: as any*/)
    ],
    "kind": "Fragment",
    "metadata": null,
    "name": "InsightsPageQuery",
    "selections": [
      (v3/*:: as any*/),
      {
        "alias": null,
        "args": (v4/*:: as any*/),
        "concreteType": "SpotifyStats",
        "kind": "LinkedField",
        "name": "spotifyStats",
        "plural": false,
        "selections": [
          (v12/*:: as any*/),
          (v14/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "concreteType": "SpotifyMusicAge",
            "kind": "LinkedField",
            "name": "musicAge",
            "plural": false,
            "selections": [
              (v15/*:: as any*/),
              (v16/*:: as any*/),
              (v17/*:: as any*/),
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifyItem",
                "kind": "LinkedField",
                "name": "oldest",
                "plural": false,
                "selections": [
                  (v10/*:: as any*/),
                  (v18/*:: as any*/)
                ],
                "storageKey": null
              }
            ],
            "storageKey": null
          },
          (v20/*:: as any*/),
          (v22/*:: as any*/),
          (v23/*:: as any*/),
          {
            "alias": null,
            "args": (v24/*:: as any*/),
            "concreteType": "SpotifyDayStat",
            "kind": "LinkedField",
            "name": "topDays",
            "plural": true,
            "selections": [
              (v25/*:: as any*/),
              (v8/*:: as any*/),
              (v9/*:: as any*/),
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifyArtist",
                "kind": "LinkedField",
                "name": "topArtist",
                "plural": false,
                "selections": (v11/*:: as any*/),
                "storageKey": null
              }
            ],
            "storageKey": "topDays(first:8)"
          },
          (v28/*:: as any*/),
          {
            "alias": null,
            "args": (v6/*:: as any*/),
            "concreteType": "SpotifyDiscoveryConnection",
            "kind": "LinkedField",
            "name": "discoveries",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifyDiscoveryEdge",
                "kind": "LinkedField",
                "name": "edges",
                "plural": true,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "concreteType": "SpotifyDiscovery",
                    "kind": "LinkedField",
                    "name": "node",
                    "plural": false,
                    "selections": [
                      (v29/*:: as any*/),
                      (v30/*:: as any*/),
                      (v9/*:: as any*/),
                      {
                        "alias": null,
                        "args": null,
                        "concreteType": "SpotifyItem",
                        "kind": "LinkedField",
                        "name": "firstItem",
                        "plural": false,
                        "selections": (v11/*:: as any*/),
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
          (v31/*:: as any*/),
          (v33/*:: as any*/),
          (v34/*:: as any*/),
          (v35/*:: as any*/),
          (v36/*:: as any*/),
          (v37/*:: as any*/)
        ],
        "storageKey": null
      },
      (v41/*:: as any*/)
    ],
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [
      (v2/*:: as any*/),
      (v0/*:: as any*/),
      (v1/*:: as any*/)
    ],
    "kind": "Operation",
    "name": "InsightsPageQuery",
    "selections": [
      (v3/*:: as any*/),
      {
        "alias": null,
        "args": (v4/*:: as any*/),
        "concreteType": "SpotifyStats",
        "kind": "LinkedField",
        "name": "spotifyStats",
        "plural": false,
        "selections": [
          (v12/*:: as any*/),
          (v14/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "concreteType": "SpotifyMusicAge",
            "kind": "LinkedField",
            "name": "musicAge",
            "plural": false,
            "selections": [
              (v15/*:: as any*/),
              (v16/*:: as any*/),
              (v17/*:: as any*/),
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifyItem",
                "kind": "LinkedField",
                "name": "oldest",
                "plural": false,
                "selections": [
                  (v10/*:: as any*/),
                  (v18/*:: as any*/),
                  (v26/*:: as any*/)
                ],
                "storageKey": null
              }
            ],
            "storageKey": null
          },
          (v20/*:: as any*/),
          (v22/*:: as any*/),
          (v23/*:: as any*/),
          {
            "alias": null,
            "args": (v24/*:: as any*/),
            "concreteType": "SpotifyDayStat",
            "kind": "LinkedField",
            "name": "topDays",
            "plural": true,
            "selections": [
              (v25/*:: as any*/),
              (v8/*:: as any*/),
              (v9/*:: as any*/),
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifyArtist",
                "kind": "LinkedField",
                "name": "topArtist",
                "plural": false,
                "selections": (v42/*:: as any*/),
                "storageKey": null
              }
            ],
            "storageKey": "topDays(first:8)"
          },
          (v28/*:: as any*/),
          {
            "alias": null,
            "args": (v6/*:: as any*/),
            "concreteType": "SpotifyDiscoveryConnection",
            "kind": "LinkedField",
            "name": "discoveries",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifyDiscoveryEdge",
                "kind": "LinkedField",
                "name": "edges",
                "plural": true,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "concreteType": "SpotifyDiscovery",
                    "kind": "LinkedField",
                    "name": "node",
                    "plural": false,
                    "selections": [
                      (v29/*:: as any*/),
                      (v30/*:: as any*/),
                      (v9/*:: as any*/),
                      {
                        "alias": null,
                        "args": null,
                        "concreteType": "SpotifyItem",
                        "kind": "LinkedField",
                        "name": "firstItem",
                        "plural": false,
                        "selections": (v42/*:: as any*/),
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
          (v31/*:: as any*/),
          (v33/*:: as any*/),
          (v34/*:: as any*/),
          (v35/*:: as any*/),
          (v36/*:: as any*/),
          (v37/*:: as any*/)
        ],
        "storageKey": null
      },
      (v41/*:: as any*/)
    ]
  },
  "params": {
    "cacheID": "64918948178f7095c1d69c0e39923422",
    "id": null,
    "metadata": {},
    "name": "InsightsPageQuery",
    "operationKind": "query",
    "text": "query InsightsPageQuery(\n  $filter: SpotifyStatsFilter!\n  $allPlays: SpotifyStatsFilter!\n  $bucket: SpotifyTimeBucket!\n) {\n  spotifyEnrichment {\n    listeningCoverage\n  }\n  spotifyStats(filter: $filter) {\n    topGenres(first: 10) {\n      edges {\n        rank\n        plays\n        minutesPlayed\n        node {\n          name\n        }\n      }\n    }\n    genreTimeline(bucket: $bucket) {\n      genre\n      points {\n        start\n        minutesPlayed\n      }\n    }\n    musicAge {\n      averageYears\n      newShare\n      oldestYear\n      oldest {\n        name\n        artistName\n        id\n      }\n    }\n    decades: breakdown(by: DECADE) {\n      key\n      label\n      plays\n      minutesPlayed\n    }\n    listeningDays {\n      activeDays\n      spanDays\n      longestStreak {\n        start\n        end\n        days\n      }\n      latestStreak {\n        start\n        end\n        days\n      }\n    }\n    sessions {\n      count\n      averageMinutes\n      longest {\n        start\n        plays\n        minutesPlayed\n      }\n    }\n    topDays(first: 8) {\n      date\n      plays\n      minutesPlayed\n      topArtist {\n        name\n        id\n      }\n    }\n    onRepeat(first: 8) {\n      date\n      plays\n      item {\n        id\n        uri\n        name\n        artistName\n        artworkUrl\n      }\n    }\n    discoveries(first: 10) {\n      edges {\n        node {\n          artist {\n            id\n            name\n            imageUrl\n          }\n          firstPlayed\n          minutesPlayed\n          firstItem {\n            name\n            id\n          }\n        }\n      }\n    }\n    forgottenArtists(first: 10) {\n      edges {\n        node {\n          artist {\n            id\n            name\n            imageUrl\n          }\n          minutesPlayed\n          lastPlayed\n        }\n      }\n    }\n    device: breakdown(by: DEVICE) {\n      label\n      plays\n      minutesPlayed\n    }\n    country: breakdown(by: COUNTRY) {\n      label\n      plays\n      minutesPlayed\n    }\n    shuffle: breakdown(by: SHUFFLE) {\n      label\n      plays\n      minutesPlayed\n    }\n    offline: breakdown(by: OFFLINE) {\n      label\n      plays\n      minutesPlayed\n    }\n    endReason: breakdown(by: END_REASON) {\n      label\n      plays\n      minutesPlayed\n    }\n  }\n  skips: spotifyStats(filter: $allPlays) {\n    completion {\n      average\n      finishedShare\n    }\n    mostAbandoned(first: 8) {\n      plays\n      averageCompletion\n      item {\n        id\n        uri\n        name\n        artistName\n        artworkUrl\n      }\n    }\n    mostSkippedArtists: artistSkipRates(order: MOST_SKIPPED, first: 10) {\n      rank\n      plays\n      skips\n      skipRate\n      artist {\n        id\n        name\n        imageUrl\n      }\n    }\n    leastSkippedArtists: artistSkipRates(order: LEAST_SKIPPED, first: 10) {\n      rank\n      plays\n      skips\n      skipRate\n      artist {\n        id\n        name\n        imageUrl\n      }\n    }\n    mostSkipped(first: 8) {\n      plays\n      skips\n      skipRate\n      item {\n        id\n        uri\n        name\n        artistName\n        artworkUrl\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "658bdbb9361a88b8b7f4ab95b857d2b2";

export default node;
