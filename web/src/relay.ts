import {
  Environment,
  Network,
  RecordSource,
  Store,
  type FetchFunction,
} from "relay-runtime";
import { AUTH_DISABLED, getAccessToken, signInAgain } from "./auth";

// Same-origin: in production the SPA and API share a host (the route sends /graphql to the
// API); in dev Vite proxies /graphql to the local backend.
const ENDPOINT = "/graphql";

const fetchFn: FetchFunction = async (request, variables) => {
  const token = getAccessToken();
  const resp = await fetch(ENDPOINT, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify({ query: request.text, variables }),
  });
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
