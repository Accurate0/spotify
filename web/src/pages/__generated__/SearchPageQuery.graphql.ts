/**
 * @generated SignedSource<<9362c2462104a8e68c8466977d56fb15>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type ItemKind = "AUDIOBOOK" | "EPISODE" | "TRACK" | "%future added value";
export type SearchPageQuery$variables = {
  query: string;
};
export type SearchPageQuery$data = {
  readonly search: {
    readonly artists: {
      readonly edges: ReadonlyArray<{
        readonly node: {
          readonly id: string;
          readonly name: string;
        };
      }>;
    };
    readonly items: {
      readonly edges: ReadonlyArray<{
        readonly node: {
          readonly album: {
            readonly name: string;
          } | null | undefined;
          readonly artistName: string | null | undefined;
          readonly id: string;
          readonly kind: ItemKind;
          readonly name: string | null | undefined;
          readonly uri: string;
        };
      }>;
    };
  };
};
export type SearchPageQuery = {
  response: SearchPageQuery$data;
  variables: SearchPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "query"
  }
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
},
v3 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Variable",
        "name": "query",
        "variableName": "query"
      }
    ],
    "concreteType": "Search",
    "kind": "LinkedField",
    "name": "search",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": [
          {
            "kind": "Literal",
            "name": "first",
            "value": 8
          }
        ],
        "concreteType": "ArtistConnection",
        "kind": "LinkedField",
        "name": "artists",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "ArtistEdge",
            "kind": "LinkedField",
            "name": "edges",
            "plural": true,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "Artist",
                "kind": "LinkedField",
                "name": "node",
                "plural": false,
                "selections": [
                  (v1/*:: as any*/),
                  (v2/*:: as any*/)
                ],
                "storageKey": null
              }
            ],
            "storageKey": null
          }
        ],
        "storageKey": "artists(first:8)"
      },
      {
        "alias": null,
        "args": [
          {
            "kind": "Literal",
            "name": "first",
            "value": 25
          }
        ],
        "concreteType": "ItemConnection",
        "kind": "LinkedField",
        "name": "items",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "ItemEdge",
            "kind": "LinkedField",
            "name": "edges",
            "plural": true,
            "selections": [
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
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "kind",
                    "storageKey": null
                  },
                  (v2/*:: as any*/),
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "artistName",
                    "storageKey": null
                  },
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
                  }
                ],
                "storageKey": null
              }
            ],
            "storageKey": null
          }
        ],
        "storageKey": "items(first:25)"
      }
    ],
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "SearchPageQuery",
    "selections": (v3/*:: as any*/),
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "SearchPageQuery",
    "selections": (v3/*:: as any*/)
  },
  "params": {
    "cacheID": "fafbd9f78a72ddcd2d79af25a75b8169",
    "id": null,
    "metadata": {},
    "name": "SearchPageQuery",
    "operationKind": "query",
    "text": "query SearchPageQuery(\n  $query: String!\n) {\n  search(query: $query) {\n    artists(first: 8) {\n      edges {\n        node {\n          id\n          name\n        }\n      }\n    }\n    items(first: 25) {\n      edges {\n        node {\n          id\n          uri\n          kind\n          name\n          artistName\n          album {\n            name\n          }\n        }\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "54f2c56e4d8edb20bf1a0eba4e3628d4";

export default node;
