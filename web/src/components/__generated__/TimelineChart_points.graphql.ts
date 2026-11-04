/**
 * @generated SignedSource<<3eda61c2327b3da1e1d5cf1af584af09>>
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
  "type": "TimelinePoint",
  "abstractKey": null
};

(node as any).hash = "c43fe2744470bba58166896d9f26b7aa";

export default node;
