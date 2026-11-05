/**
 * @generated SignedSource<<a2f2acbd012514c05c36679198a7b0a8>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type Firsts_stats$data = {
  readonly firstPlay: {
    readonly item: {
      readonly name: string | null | undefined;
      readonly uri: string;
    } | null | undefined;
    readonly ts: string;
  } | null | undefined;
  readonly listeningDays: {
    readonly activeDays: number;
    readonly longestStreak: {
      readonly days: number;
      readonly end: string;
      readonly start: string;
    } | null | undefined;
  };
  readonly topDays: ReadonlyArray<{
    readonly date: string;
    readonly minutesPlayed: number;
    readonly plays: number;
  }>;
  readonly " $fragmentType": "Firsts_stats";
};
export type Firsts_stats$key = {
  readonly " $data"?: Firsts_stats$data;
  readonly " $fragmentSpreads": FragmentRefs<"Firsts_stats">;
};

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": null,
  "name": "Firsts_stats",
  "selections": [
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
            }
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
        }
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
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "start",
              "storageKey": null
            },
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
    }
  ],
  "type": "SpotifyStats",
  "abstractKey": null
};

(node as any).hash = "c7d716c8cc9b7e22c8819ccfeed2f6e8";

export default node;
