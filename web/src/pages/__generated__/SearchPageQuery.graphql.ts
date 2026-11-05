/**
 * @generated SignedSource<<aacd04495365ee3bb57674582d2ea5aa>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type SpotifyItemKind = "AUDIOBOOK" | "EPISODE" | "TRACK" | "%future added value";
export type SearchPageQuery$variables = {
  query: string;
};
export type SearchPageQuery$data = {
  readonly spotifySearch: {
    readonly artists: {
      readonly edges: ReadonlyArray<{
        readonly node: {
          readonly id: string;
          readonly imageUrl: string | null | undefined;
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
          readonly artworkUrl: string | null | undefined;
          readonly id: string;
          readonly kind: SpotifyItemKind;
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
    "concreteType": "SpotifySearch",
    "kind": "LinkedField",
    "name": "spotifySearch",
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
        "concreteType": "SpotifyArtistConnection",
        "kind": "LinkedField",
        "name": "artists",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "SpotifyArtistEdge",
            "kind": "LinkedField",
            "name": "edges",
            "plural": true,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifyArtist",
                "kind": "LinkedField",
                "name": "node",
                "plural": false,
                "selections": [
                  (v1/*:: as any*/),
                  (v2/*:: as any*/),
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "imageUrl",
                    "storageKey": null
                  }
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
        "concreteType": "SpotifyItemConnection",
        "kind": "LinkedField",
        "name": "items",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "SpotifyItemEdge",
            "kind": "LinkedField",
            "name": "edges",
            "plural": true,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "SpotifyItem",
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
                    "kind": "ScalarField",
                    "name": "artworkUrl",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "concreteType": "SpotifyAlbum",
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
    "cacheID": "bfeb8dac2092f6caadb4a30c93e3a6c4",
    "id": null,
    "metadata": {},
    "name": "SearchPageQuery",
    "operationKind": "query",
    "text": "query SearchPageQuery(\n  $query: String!\n) {\n  spotifySearch(query: $query) {\n    artists(first: 8) {\n      edges {\n        node {\n          id\n          name\n          imageUrl\n        }\n      }\n    }\n    items(first: 25) {\n      edges {\n        node {\n          id\n          uri\n          kind\n          name\n          artistName\n          artworkUrl\n          album {\n            name\n          }\n        }\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "cdeeaf2a81f936137f8659617b710eec";

export default node;
