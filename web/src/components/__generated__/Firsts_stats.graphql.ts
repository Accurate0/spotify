/**
 * @generated SignedSource<<91630a0613f794c1437f0fa874d08b26>>
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
      "concreteType": "Play",
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
          "concreteType": "Item",
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
  "type": "Stats",
  "abstractKey": null
};

(node as any).hash = "bb2a9da221886e81998dac487fba18b8";

export default node;
