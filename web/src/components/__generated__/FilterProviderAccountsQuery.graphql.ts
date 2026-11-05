/**
 * @generated SignedSource<<136402ecb9a389a1b3fcfa594b8f6723>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type FilterProviderAccountsQuery$variables = Record<PropertyKey, never>;
export type FilterProviderAccountsQuery$data = {
  readonly spotifyAccounts: ReadonlyArray<{
    readonly displayName: string;
    readonly id: string;
    readonly slug: string;
  }>;
  readonly spotifyStats: {
    readonly summary: {
      readonly firstPlayed: string | null | undefined;
      readonly lastPlayed: string | null | undefined;
    };
  };
};
export type FilterProviderAccountsQuery = {
  response: FilterProviderAccountsQuery$data;
  variables: FilterProviderAccountsQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "alias": null,
    "args": null,
    "concreteType": "SpotifyAccount",
    "kind": "LinkedField",
    "name": "spotifyAccounts",
    "plural": true,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "id",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "slug",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "displayName",
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
        "name": "filter",
        "value": {
          "kinds": [
            "TRACK",
            "EPISODE",
            "AUDIOBOOK"
          ],
          "minMsPlayed": 0
        }
      }
    ],
    "concreteType": "SpotifyStats",
    "kind": "LinkedField",
    "name": "spotifyStats",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "SpotifySummary",
        "kind": "LinkedField",
        "name": "summary",
        "plural": false,
        "selections": [
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
      }
    ],
    "storageKey": "spotifyStats(filter:{\"kinds\":[\"TRACK\",\"EPISODE\",\"AUDIOBOOK\"],\"minMsPlayed\":0})"
  }
];
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "FilterProviderAccountsQuery",
    "selections": (v0/*:: as any*/),
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "FilterProviderAccountsQuery",
    "selections": (v0/*:: as any*/)
  },
  "params": {
    "cacheID": "9c6f30ea9ab53b895cf38c0f2dc9dc52",
    "id": null,
    "metadata": {},
    "name": "FilterProviderAccountsQuery",
    "operationKind": "query",
    "text": "query FilterProviderAccountsQuery {\n  spotifyAccounts {\n    id\n    slug\n    displayName\n  }\n  spotifyStats(filter: {kinds: [TRACK, EPISODE, AUDIOBOOK], minMsPlayed: 0}) {\n    summary {\n      firstPlayed\n      lastPlayed\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "008084e6a04613fc26d6b3b5c9ada434";

export default node;
