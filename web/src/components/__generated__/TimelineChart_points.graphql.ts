/**
 * @generated SignedSource<<d94ea5202a2cadbb789c3f78de875c83>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type TimelineChart_points$data = ReadonlyArray<{
  readonly minutesPlayed: number;
  readonly plays: number;
  readonly start: string;
  readonly " $fragmentType": "TimelineChart_points";
}>;
export type TimelineChart_points$key = ReadonlyArray<{
  readonly " $data"?: TimelineChart_points$data;
  readonly " $fragmentSpreads": FragmentRefs<"TimelineChart_points">;
}>;

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": {
    "plural": true
  },
  "name": "TimelineChart_points",
  "selections": [
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
  "type": "SpotifyTimelinePoint",
  "abstractKey": null
};

(node as any).hash = "fe0513065bfb1e34ca8400baf9591811";

export default node;
