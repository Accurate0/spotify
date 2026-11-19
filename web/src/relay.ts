import {
  Environment,
  Network,
  RecordSource,
  Store,
  type FetchFunction,
} from "relay-runtime";
import { AUTH_DISABLED, getAccessToken, signInAgain } from "./auth";

// Production goes through the graph router, which serves Relay's `node` across services; in dev
// Vite proxies /v1/graphql to the local backend.
const ENDPOINT = import.meta.env.DEV ? "/v1/graphql" : "https://graph.ortzi.cloud/v1/graphql";

function post(body: object): Promise<Response> {
  const token = getAccessToken();
  return fetch(ENDPOINT, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(body),
  });
}

function documentMissing(body: { errors?: { extensions?: { code?: string } }[] }): boolean {
  return body.errors?.[0]?.extensions?.code === "PERSISTED_DOCUMENT_NOT_FOUND";
}

// Production sends the persisted document id; the local backend only takes query text, and a
// router that hasn't loaded this build's documents yet gets the text on a retry.
const fetchFn: FetchFunction = async (request, variables) => {
  const persisted = !import.meta.env.DEV && request.id;
  let resp = await post(
    persisted ? { documentId: request.id, variables } : { query: request.text, variables },
  );
  if (persisted && documentMissing(await resp.clone().json().catch(() => ({})))) {
    resp = await post({ query: request.text, variables });
  }

  if (resp.status === 401 && !AUTH_DISABLED) {
    // Expired or revoked token: go back through Kanidm rather than showing an error.
    await signInAgain();
  }
  if (resp.status === 403) {
    throw new Error("Your account isn't allowed to use this app (spotify_users group).");
  }
  if (!resp.ok) {
    throw new Error(`GraphQL request failed: ${resp.status}`);
  }
  return await resp.json();
};

export const environment = new Environment({
  network: Network.create(fetchFn),
  store: new Store(new RecordSource()),
});
