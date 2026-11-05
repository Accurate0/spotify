/**
 * @generated SignedSource<<7d89ca8023f0717021295199042a7c63>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type StatTiles_summary$data = {
  readonly distinctArtists: number;
  readonly distinctTracks: number;
  readonly firstPlayed: string | null | undefined;
  readonly lastPlayed: string | null | undefined;
  readonly minutesPlayed: number;
  readonly plays: number;
  readonly skipRate: number | null | undefined;
  readonly " $fragmentType": "StatTiles_summary";
};
export type StatTiles_summary$key = {
  readonly " $data"?: StatTiles_summary$data;
  readonly " $fragmentSpreads": FragmentRefs<"StatTiles_summary">;
};

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": null,
  "name": "StatTiles_summary",
  "selections": [
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
  "type": "SpotifySummary",
  "abstractKey": null
};

(node as any).hash = "66ab38e3646b4b0089475fb1b57b97b0";

export default node;
