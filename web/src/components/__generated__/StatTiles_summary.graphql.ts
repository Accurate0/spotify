/**
 * @generated SignedSource<<fde759e7864b3c436017edab57b30f68>>
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
  "type": "Summary",
  "abstractKey": null
};

(node as any).hash = "6e82b7df5901cef0bc6e04881701e8c3";

export default node;
