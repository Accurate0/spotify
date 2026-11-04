/**
 * @generated SignedSource<<d597ef039d55355af1b4755b6c350caa>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type ArtistPageTracks_artist$data = {
  readonly id: string;
  readonly stats: {
    readonly topItems: {
      readonly edges: ReadonlyArray<{
        readonly minutesPlayed: number;
        readonly node: {
          readonly album: {
            readonly name: string;
          } | null | undefined;
          readonly id: string;
          readonly name: string | null | undefined;
          readonly uri: string;
        };
        readonly plays: number;
        readonly rank: number;
      }>;
    };
  };
  readonly " $fragmentType": "ArtistPageTracks_artist";
};
export type ArtistPageTracks_artist$key = {
  readonly " $data"?: ArtistPageTracks_artist$data;
  readonly " $fragmentSpreads": FragmentRefs<"ArtistPageTracks_artist">;
};

import ArtistPageTracksPaginationQuery_graphql from './ArtistPageTracksPaginationQuery.graphql';

const node: ReaderFragment = (function(){
var v0 = [
  "stats",
  "topItems"
],
v1 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
};
return {
  "argumentDefinitions": [
    {
      "defaultValue": null,
      "kind": "LocalArgument",
      "name": "after"
    },
    {
      "defaultValue": null,
      "kind": "LocalArgument",
      "name": "filter"
    },
    {
      "defaultValue": 20,
      "kind": "LocalArgument",
      "name": "first"
    }
  ],
  "kind": "Fragment",
  "metadata": {
    "connection": [
      {
        "count": "first",
        "cursor": "after",
        "direction": "forward",
        "path": (v0/*:: as any*/)
      }
    ],
    "refetch": {
      "connection": {
        "forward": {
          "count": "first",
          "cursor": "after"
        },
        "backward": null,
        "path": (v0/*:: as any*/)
      },
      "fragmentPathInResult": [
        "node"
      ],
      "operation": ArtistPageTracksPaginationQuery_graphql,
      "identifierInfo": {
        "identifierField": "id",
        "identifierQueryVariableName": "id"
      }
    }
  },
  "name": "ArtistPageTracks_artist",
  "selections": [
    {
      "alias": null,
      "args": [
        {
          "kind": "Variable",
          "name": "filter",
          "variableName": "filter"
        }
      ],
      "concreteType": "Stats",
      "kind": "LinkedField",
      "name": "stats",
      "plural": false,
      "selections": [
        {
          "alias": "topItems",
          "args": null,
          "concreteType": "RankedItemConnection",
          "kind": "LinkedField",
          "name": "__ArtistPageTracks_topItems_connection",
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
                  "concreteType": "Item",
                  "kind": "LinkedField",
                  "name": "node",
                  "plural": false,
                  "selections": [
                    (v1/*:: as any*/),
                    {
                      "alias": null,
                      "args": null,
                      "kind": "ScalarField",
                      "name": "uri",
                      "storageKey": null
                    },
                    (v2/*:: as any*/),
                    {
                      "alias": null,
                      "args": null,
                      "concreteType": "Album",
                      "kind": "LinkedField",
                      "name": "album",
                      "plural": false,
                      "selections": [
                        (v2/*:: as any*/)
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
          "storageKey": null
        }
      ],
      "storageKey": null
    },
    (v1/*:: as any*/)
  ],
  "type": "Artist",
  "abstractKey": null
};
})();

(node as any).hash = "2b9b952f6f59aef0a7667508fc37f819";

export default node;
