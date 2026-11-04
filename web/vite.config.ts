import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import relay from "vite-plugin-relay";
import path from "path";

// In dev, proxy the API to a locally running backend (cargo run serves on :8000) so the
// SPA uses the same relative /graphql URL as in production, where both share a host.
const BACKEND = process.env.BACKEND_URL ?? "http://localhost:8000";

export default defineConfig({
  plugins: [relay, react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    proxy: {
      "/graphql": { target: BACKEND, changeOrigin: true },
    },
  },
});
