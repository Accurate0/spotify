import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import relay from "vite-plugin-relay";
import path from "path";

// In dev, proxy /v1/graphql to a locally running backend (cargo run serves on :8000); production
// talks to the graph router instead.
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
      "/v1/graphql": { target: BACKEND, changeOrigin: true },
    },
  },
});
