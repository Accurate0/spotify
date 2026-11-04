/**
 * @generated SignedSource<<70c8b58b18dda3d00dfc68a2a153e3aa>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type Heatmap_cells$data = ReadonlyArray<{
  readonly dayOfWeek: number;
  readonly hour: number;
  readonly minutesPlayed: number;
  readonly plays: number;
  readonly " $fragmentType": "Heatmap_cells";
}>;
export type Heatmap_cells$key = ReadonlyArray<{
  readonly " $data"?: Heatmap_cells$data;
  readonly " $fragmentSpreads": FragmentRefs<"Heatmap_cells">;
}>;

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": {
    "plural": true
  },
  "name": "Heatmap_cells",
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
  "type": "HeatmapCell",
  "abstractKey": null
};

(node as any).hash = "9f11fec855e6fa45dae94ebb374cef36";

export default node;
