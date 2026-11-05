import { StrictMode, Suspense } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/fraunces/index.css";
import "@fontsource-variable/hanken-grotesk/index.css";
import "./index.css";
import { BrowserRouter } from "react-router";
import { RelayEnvironmentProvider } from "react-relay";
import App from "./App.tsx";
import { environment } from "./relay";
import { AUTH_DISABLED, SIGNED_OUT_PATH, ensureAuthenticated, signInAgain } from "./auth";

const root = createRoot(document.getElementById("root")!);

if (!AUTH_DISABLED && window.location.pathname === SIGNED_OUT_PATH) {
  root.render(
    <div className="mx-auto grid min-h-screen max-w-md place-items-center px-6 text-center">
      <div>
        <h1 className="font-display mb-2 text-3xl font-semibold">Signed out</h1>
        <p className="text-muted-foreground mb-4 text-sm">You&apos;ve been signed out of Listening.</p>
        <button
          type="button"
          onClick={() => void signInAgain()}
          className="hover:bg-accent rounded-md border px-4 py-2 text-sm"
        >
          Sign in again
        </button>
      </div>
    </div>,
  );
} else {
  try {
    await ensureAuthenticated();
    root.render(
      <StrictMode>
        <RelayEnvironmentProvider environment={environment}>
          <BrowserRouter>
            <Suspense
              fallback={
                <div className="text-muted-foreground grid min-h-screen place-items-center">Loading…</div>
              }
            >
              <App />
            </Suspense>
          </BrowserRouter>
        </RelayEnvironmentProvider>
      </StrictMode>,
    );
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    root.render(
      <div className="mx-auto grid min-h-screen max-w-md place-items-center px-6 text-center">
        <div>
          <h1 className="mb-2 text-lg font-semibold">Sign-in failed</h1>
          <p className="text-muted-foreground mb-4 text-sm">{message}</p>
          <a href="/" className="text-sm underline">
            Try again
          </a>
        </div>
      </div>,
    );
  }
}
