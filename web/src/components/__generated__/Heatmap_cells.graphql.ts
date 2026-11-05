/**
 * @generated SignedSource<<03861e2a4bd037510f200befe1ac2696>>
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
  "type": "SpotifyHeatmapCell",
  "abstractKey": null
};

(node as any).hash = "dfbe74023302d950ea279bea2b1896ad";

export default node;
