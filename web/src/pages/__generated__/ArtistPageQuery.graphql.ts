/**
 * @generated SignedSource<<7e978fb017d50e82dbbee96b4feb91c5>>
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
export type ArtistPageQuery$variables = {
  filter: StatsFilter;
  name: string;
};
export type ArtistPageQuery$data = {
  readonly artist: {
    readonly name: string;
    readonly stats: {
      readonly summary: {
        readonly " $fragmentSpreads": FragmentRefs<"StatTiles_summary">;
      };
      readonly timeline: ReadonlyArray<{
        readonly " $fragmentSpreads": FragmentRefs<"TimelineChart_points">;
      }>;
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
v4 = [
  {
    "kind": "Variable",
    "name": "filter",
    "variableName": "filter"
  }
],
v5 = [
  {
    "kind": "Literal",
    "name": "bucket",
    "value": "MONTH"
  }
],
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
v8 = [
  {
    "kind": "Literal",
    "name": "first",
    "value": 20
  }
],
v9 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
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
    "name": "ArtistPageQuery",
    "selections": [
      {
        "alias": null,
        "args": (v2/*:: as any*/),
        "concreteType": "Artist",
        "kind": "LinkedField",
        "name": "artist",
        "plural": false,
        "selections": [
          (v3/*:: as any*/),
          {
            "alias": null,
            "args": (v4/*:: as any*/),
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
                "alias": null,
                "args": (v5/*:: as any*/),
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
            "args": (v4/*:: as any*/),
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
        "concreteType": "Artist",
        "kind": "LinkedField",
        "name": "artist",
        "plural": false,
        "selections": [
          (v3/*:: as any*/),
          {
            "alias": null,
            "args": (v4/*:: as any*/),
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
                  (v6/*:: as any*/),
                  (v7/*:: as any*/),
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
                "args": (v5/*:: as any*/),
                "concreteType": "TimelinePoint",
                "kind": "LinkedField",
                "name": "timeline",
                "plural": true,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "start",
                    "storageKey": null
                  },
                  (v6/*:: as any*/),
                  (v7/*:: as any*/)
                ],
                "storageKey": "timeline(bucket:\"MONTH\")"
              },
              {
                "alias": null,
                "args": (v8/*:: as any*/),
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
                      (v6/*:: as any*/),
                      (v7/*:: as any*/),
                      {
                        "alias": null,
                        "args": null,
                        "concreteType": "Item",
                        "kind": "LinkedField",
                        "name": "node",
                        "plural": false,
                        "selections": [
                          (v9/*:: as any*/),
                          {
                            "alias": null,
                            "args": null,
                            "kind": "ScalarField",
                            "name": "uri",
                            "storageKey": null
                          },
                          (v3/*:: as any*/),
                          {
                            "alias": null,
                            "args": null,
                            "concreteType": "Album",
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
                "args": (v8/*:: as any*/),
                "filters": null,
                "handle": "connection",
                "key": "ArtistPageTracks_topItems",
                "kind": "LinkedHandle",
                "name": "topItems"
              }
            ],
            "storageKey": null
          },
          (v9/*:: as any*/)
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "0ba69c28b55d0e73985fecc61ef8c7c4",
    "id": null,
    "metadata": {},
    "name": "ArtistPageQuery",
    "operationKind": "query",
    "text": "query ArtistPageQuery(\n  $name: String!\n  $filter: StatsFilter!\n) {\n  artist(name: $name) {\n    name\n    stats(filter: $filter) {\n      summary {\n        ...StatTiles_summary\n      }\n      timeline(bucket: MONTH) {\n        ...TimelineChart_points\n      }\n    }\n    ...ArtistPageTracks_artist_Vt7Yj\n    id\n  }\n}\n\nfragment ArtistPageTracks_artist_Vt7Yj on Artist {\n  stats(filter: $filter) {\n    topItems(first: 20) {\n      edges {\n        rank\n        plays\n        minutesPlayed\n        node {\n          id\n          uri\n          name\n          album {\n            name\n          }\n          __typename\n        }\n        cursor\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n  id\n}\n\nfragment StatTiles_summary on Summary {\n  plays\n  minutesPlayed\n  distinctTracks\n  distinctArtists\n  skipRate\n  firstPlayed\n  lastPlayed\n}\n\nfragment TimelineChart_points on TimelinePoint {\n  start\n  plays\n  minutesPlayed\n}\n"
  }
};
})();

(node as any).hash = "6d7171a071e3899f010c6e33df758a86";

export default node;
