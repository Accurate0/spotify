/**
 * @generated SignedSource<<b33e59cfdacf90a7c8ae526c86a421e3>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type ItemKind = "AUDIOBOOK" | "EPISODE" | "TRACK" | "%future added value";
export type TimeBucket = "DAY" | "MONTH" | "WEEK" | "YEAR" | "%future added value";
export type StatsFilter = {
  accounts?: ReadonlyArray<string>;
  from?: string | null | undefined;
  kinds?: ReadonlyArray<ItemKind>;
  minMsPlayed?: number;
  timeZone?: string;
  to?: string | null | undefined;
};
export type OverviewPageQuery$variables = {
  bucket: TimeBucket;
  filter: StatsFilter;
};
export type OverviewPageQuery$data = {
  readonly stats: {
    readonly heatmap: ReadonlyArray<{
      readonly " $fragmentSpreads": FragmentRefs<"Heatmap_cells">;
    }>;
    readonly summary: {
      readonly " $fragmentSpreads": FragmentRefs<"StatTiles_summary">;
    };
    readonly timeline: ReadonlyArray<{
      readonly " $fragmentSpreads": FragmentRefs<"TimelineChart_points">;
    }>;
    readonly topArtists: {
      readonly edges: ReadonlyArray<{
        readonly minutesPlayed: number;
        readonly node: {
          readonly id: string;
          readonly name: string;
        };
        readonly plays: number;
        readonly rank: number;
      }>;
    };
    readonly topItems: {
      readonly edges: ReadonlyArray<{
        readonly minutesPlayed: number;
        readonly node: {
          readonly artistName: string | null | undefined;
          readonly id: string;
          readonly name: string | null | undefined;
          readonly uri: string;
        };
        readonly plays: number;
        readonly rank: number;
      }>;
    };
  };
};
export type OverviewPageQuery = {
  response: OverviewPageQuery$data;
  variables: OverviewPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "bucket"
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
v3 = [
  {
    "kind": "Variable",
    "name": "bucket",
    "variableName": "bucket"
  }
],
v4 = [
  {
    "kind": "Literal",
    "name": "first",
    "value": 5
  }
],
v5 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "rank",
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
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v9 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v10 = {
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
        (v5/*:: as any*/),
        (v6/*:: as any*/),
        (v7/*:: as any*/),
        {
          "alias": null,
          "args": null,
          "concreteType": "Artist",
          "kind": "LinkedField",
          "name": "node",
          "plural": false,
          "selections": [
            (v8/*:: as any*/),
            (v9/*:: as any*/)
          ],
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "storageKey": "topArtists(first:5)"
},
v11 = {
  "alias": null,
  "args": (v4/*:: as any*/),
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
        (v5/*:: as any*/),
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
            (v8/*:: as any*/),
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "uri",
              "storageKey": null
            },
            (v9/*:: as any*/),
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "artistName",
              "storageKey": null
            }
          ],
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "storageKey": "topItems(first:5)"
};
return {
  "fragment": {
    "argumentDefinitions": [
      (v0/*:: as any*/),
      (v1/*:: as any*/)
    ],
    "kind": "Fragment",
    "metadata": null,
    "name": "OverviewPageQuery",
    "selections": [
      {
        "alias": null,
        "args": (v2/*:: as any*/),
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
            "args": (v3/*:: as any*/),
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
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "concreteType": "HeatmapCell",
            "kind": "LinkedField",
            "name": "heatmap",
            "plural": true,
            "selections": [
              {
                "args": null,
                "kind": "FragmentSpread",
                "name": "Heatmap_cells"
              }
            ],
            "storageKey": null
          },
          (v10/*:: as any*/),
          (v11/*:: as any*/)
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
    "name": "OverviewPageQuery",
    "selections": [
      {
        "alias": null,
        "args": (v2/*:: as any*/),
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
            "args": (v3/*:: as any*/),
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
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "concreteType": "HeatmapCell",
            "kind": "LinkedField",
            "name": "heatmap",
            "plural": true,
            "selections": [
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "dayOfWeek",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "hour",
                "storageKey": null
              },
              (v6/*:: as any*/),
              (v7/*:: as any*/)
            ],
            "storageKey": null
          },
          (v10/*:: as any*/),
          (v11/*:: as any*/)
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "2e5902c2096f797577eeb28b34486a85",
    "id": null,
    "metadata": {},
    "name": "OverviewPageQuery",
    "operationKind": "query",
    "text": "query OverviewPageQuery(\n  $filter: StatsFilter!\n  $bucket: TimeBucket!\n) {\n  stats(filter: $filter) {\n    summary {\n      ...StatTiles_summary\n    }\n    timeline(bucket: $bucket) {\n      ...TimelineChart_points\n    }\n    heatmap {\n      ...Heatmap_cells\n    }\n    topArtists(first: 5) {\n      edges {\n        rank\n        plays\n        minutesPlayed\n        node {\n          id\n          name\n        }\n      }\n    }\n    topItems(first: 5) {\n      edges {\n        rank\n        plays\n        minutesPlayed\n        node {\n          id\n          uri\n          name\n          artistName\n        }\n      }\n    }\n  }\n}\n\nfragment Heatmap_cells on HeatmapCell {\n  dayOfWeek\n  hour\n  plays\n  minutesPlayed\n}\n\nfragment StatTiles_summary on Summary {\n  plays\n  minutesPlayed\n  distinctTracks\n  distinctArtists\n  skipRate\n  firstPlayed\n  lastPlayed\n}\n\nfragment TimelineChart_points on TimelinePoint {\n  start\n  plays\n  minutesPlayed\n}\n"
  }
};
})();

(node as any).hash = "3ccbe823294e9405db4ab85476f31e89";

export default node;
