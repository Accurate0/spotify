import { UserManager, WebStorageStateStore, type User } from "oidc-client-ts";

// Kanidm OIDC public client, declared as the `spotify` oidc resource in the k8s app.yaml. The
// API validates the access token's audience.
const AUTHORITY = "https://idm.anurag.sh/oauth2/openid/spotify";
const CLIENT_ID = "spotify";
const SCOPE = "openid email profile groups";

// Local development runs the API with AUTH_DISABLED, so skip the Kanidm round-trip. Set
// VITE_AUTH_DISABLED=true to do the same in a production build.
export const AUTH_DISABLED =
  import.meta.env.DEV || import.meta.env.VITE_AUTH_DISABLED === "true";

// Both the signed-in user and the transient signin state (PKCE verifier + state) live in
// localStorage so they survive the full-page redirect to Kanidm and back.
const store = new WebStorageStateStore({ store: window.localStorage });

const userManager = AUTH_DISABLED
  ? null
  : new UserManager({
      authority: AUTHORITY,
      client_id: CLIENT_ID,
      redirect_uri: `${window.location.origin}/callback`,
      post_logout_redirect_uri: window.location.origin,
      response_type: "code",
      scope: SCOPE,
      automaticSilentRenew: true,
      userStore: store,
      stateStore: store,
    });

let currentUser: User | null = null;

export function getAccessToken(): string | null {
  return currentUser?.access_token ?? null;
}

// Resolves once a valid access token is held (immediately when auth is disabled), driving
// the redirect flow when needed.
export async function ensureAuthenticated(): Promise<void> {
  if (!userManager) return;

  const params = new URLSearchParams(window.location.search);
  if (window.location.pathname === "/callback" && params.has("code")) {
    try {
      currentUser = await userManager.signinRedirectCallback();
    } catch (e) {
      // Strip the consumed code/state so a reload can't replay this callback.
      window.history.replaceState({}, "", "/");
      throw new Error(
        `Sign-in failed during token exchange: ${e instanceof Error ? e.message : String(e)}`,
        { cause: e },
      );
    }
    window.history.replaceState({}, "", "/");
  }

  currentUser = currentUser ?? (await userManager.getUser());
  if (!currentUser || currentUser.expired) {
    await userManager.signinRedirect();
    // signinRedirect navigates away; never resolve.
    return new Promise<never>(() => {});
  }

  userManager.events.addUserLoaded((user) => {
    currentUser = user;
  });
}
