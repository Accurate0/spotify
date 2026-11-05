/**
 * @generated SignedSource<<6a4cd067194c8806e26e85dfe651c581>>
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
export type ArtistPageQuery$variables = {
  filter: SpotifyStatsFilter;
  name: string;
};
export type ArtistPageQuery$data = {
  readonly spotifyArtist: {
    readonly genres: ReadonlyArray<string>;
    readonly imageUrl: string | null | undefined;
    readonly name: string;
    readonly stats: {
      readonly summary: {
        readonly " $fragmentSpreads": FragmentRefs<"StatTiles_summary">;
      };
      readonly timeline: ReadonlyArray<{
        readonly " $fragmentSpreads": FragmentRefs<"TimelineChart_points">;
      }>;
      readonly " $fragmentSpreads": FragmentRefs<"Firsts_stats">;
    };
    readonly " $fragmentSpreads": FragmentRefs<"ArtistPageTracks_artist">;
  } | null | undefined;
};
export type ArtistPageQuery = {
  response: ArtistPageQuery$data;
  variables: ArtistPageQuery$variables;
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
  "name": "name"
},
v2 = [
  {
    "kind": "Variable",
    "name": "name",
    "variableName": "name"
  }
],
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v4 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "imageUrl",
  "storageKey": null
},
v5 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "genres",
  "storageKey": null
},
v6 = [
  {
    "kind": "Variable",
    "name": "filter",
    "variableName": "filter"
  }
],
v7 = [
  {
    "kind": "Literal",
    "name": "bucket",
    "value": "MONTH"
  }
],
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
  "name": "uri",
  "storageKey": null
},
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
  "name": "start",
  "storageKey": null
},
v13 = [
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
    "name": "ArtistPageQuery",
    "selections": [
      {
        "alias": null,
        "args": (v2/*:: as any*/),
        "concreteType": "SpotifyArtist",
        "kind": "LinkedField",
        "name": "spotifyArtist",
        "plural": false,
        "selections": [
          (v3/*:: as any*/),
          (v4/*:: as any*/),
          (v5/*:: as any*/),
          {
            "alias": null,
            "args": (v6/*:: as any*/),
            "concreteType": "SpotifyStats",
            "kind": "LinkedField",
            "name": "stats",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifySummary",
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
                "args": (v7/*:: as any*/),
                "concreteType": "SpotifyTimelinePoint",
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
            "args": (v6/*:: as any*/),
            "kind": "FragmentSpread",
            "name": "ArtistPageTracks_artist"
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
    "name": "ArtistPageQuery",
    "selections": [
      {
        "alias": null,
        "args": (v2/*:: as any*/),
        "concreteType": "SpotifyArtist",
        "kind": "LinkedField",
        "name": "spotifyArtist",
        "plural": false,
        "selections": [
          (v3/*:: as any*/),
          (v4/*:: as any*/),
          (v5/*:: as any*/),
          {
            "alias": null,
            "args": (v6/*:: as any*/),
            "concreteType": "SpotifyStats",
            "kind": "LinkedField",
            "name": "stats",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifySummary",
                "kind": "LinkedField",
                "name": "summary",
                "plural": false,
                "selections": [
                  (v8/*:: as any*/),
                  (v9/*:: as any*/),
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
                "concreteType": "SpotifyPlay",
                "kind": "LinkedField",
                "name": "firstPlay",
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
                    "concreteType": "SpotifyItem",
                    "kind": "LinkedField",
                    "name": "item",
                    "plural": false,
                    "selections": [
                      (v10/*:: as any*/),
                      (v3/*:: as any*/),
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
                "concreteType": "SpotifyDayStat",
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
                  (v8/*:: as any*/),
                  (v9/*:: as any*/)
                ],
                "storageKey": "topDays(first:1)"
              },
              {
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
                    "concreteType": "SpotifyStreak",
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
                      (v12/*:: as any*/),
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
                "args": (v7/*:: as any*/),
                "concreteType": "SpotifyTimelinePoint",
                "kind": "LinkedField",
                "name": "timeline",
                "plural": true,
                "selections": [
                  (v12/*:: as any*/),
                  (v8/*:: as any*/),
                  (v9/*:: as any*/)
                ],
                "storageKey": "timeline(bucket:\"MONTH\")"
              },
              {
                "alias": null,
                "args": (v13/*:: as any*/),
                "concreteType": "SpotifyItemRankedConnection",
                "kind": "LinkedField",
                "name": "topItems",
                "plural": false,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "concreteType": "SpotifyItemRankedEdge",
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
                      (v8/*:: as any*/),
                      (v9/*:: as any*/),
                      {
                        "alias": null,
                        "args": null,
                        "concreteType": "SpotifyItem",
                        "kind": "LinkedField",
                        "name": "node",
                        "plural": false,
                        "selections": [
                          (v11/*:: as any*/),
                          (v10/*:: as any*/),
                          (v3/*:: as any*/),
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
                            "concreteType": "SpotifyAlbum",
                            "kind": "LinkedField",
                            "name": "album",
                            "plural": false,
                            "selections": [
                              (v3/*:: as any*/)
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
                "storageKey": "topItems(first:20)"
              },
              {
                "alias": null,
                "args": (v13/*:: as any*/),
                "filters": null,
                "handle": "connection",
                "key": "ArtistPageTracks_topItems",
                "kind": "LinkedHandle",
                "name": "topItems"
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
    "cacheID": "60f5cc03d897c041966b0298350501ae",
    "id": null,
    "metadata": {},
    "name": "ArtistPageQuery",
    "operationKind": "query",
    "text": "query ArtistPageQuery(\n  $name: String!\n  $filter: SpotifyStatsFilter!\n) {\n  spotifyArtist(name: $name) {\n    name\n    imageUrl\n    genres\n    stats(filter: $filter) {\n      summary {\n        ...StatTiles_summary\n      }\n      ...Firsts_stats\n      timeline(bucket: MONTH) {\n        ...TimelineChart_points\n      }\n    }\n    ...ArtistPageTracks_artist_Vt7Yj\n    id\n  }\n}\n\nfragment ArtistPageTracks_artist_Vt7Yj on SpotifyArtist {\n  stats(filter: $filter) {\n    topItems(first: 20) {\n      edges {\n        rank\n        plays\n        minutesPlayed\n        node {\n          id\n          uri\n          name\n          artworkUrl\n          album {\n            name\n          }\n          __typename\n        }\n        cursor\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n  id\n}\n\nfragment Firsts_stats on SpotifyStats {\n  firstPlay {\n    ts\n    item {\n      uri\n      name\n      id\n    }\n  }\n  topDays(first: 1) {\n    date\n    plays\n    minutesPlayed\n  }\n  listeningDays {\n    activeDays\n    longestStreak {\n      days\n      start\n      end\n    }\n  }\n}\n\nfragment StatTiles_summary on SpotifySummary {\n  plays\n  minutesPlayed\n  distinctTracks\n  distinctArtists\n  skipRate\n  firstPlayed\n  lastPlayed\n}\n\nfragment TimelineChart_points on SpotifyTimelinePoint {\n  start\n  plays\n  minutesPlayed\n}\n"
  }
};
})();

(node as any).hash = "7e26b3dce562bcfbe044b28ba9d49903";

export default node;
