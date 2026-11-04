/**
 * @generated SignedSource<<85bb5552414cc9179b7aa7da183160d6>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type FilterProviderAccountsQuery$variables = Record<PropertyKey, never>;
export type FilterProviderAccountsQuery$data = {
  readonly accounts: ReadonlyArray<{
    readonly displayName: string;
    readonly id: string;
    readonly slug: string;
  }>;
  readonly stats: {
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
    "concreteType": "Account",
    "kind": "LinkedField",
    "name": "accounts",
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
    "concreteType": "Stats",
    "kind": "LinkedField",
    "name": "stats",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "Summary",
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
    "storageKey": "stats(filter:{\"kinds\":[\"TRACK\",\"EPISODE\",\"AUDIOBOOK\"],\"minMsPlayed\":0})"
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
    "cacheID": "eacd327497aaf427042ae79ab9ed627e",
    "id": null,
    "metadata": {},
    "name": "FilterProviderAccountsQuery",
    "operationKind": "query",
    "text": "query FilterProviderAccountsQuery {\n  accounts {\n    id\n    slug\n    displayName\n  }\n  stats(filter: {kinds: [TRACK, EPISODE, AUDIOBOOK], minMsPlayed: 0}) {\n    summary {\n      firstPlayed\n      lastPlayed\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "acb9e00a4c5722f4cacd824567b94d9c";

export default node;
