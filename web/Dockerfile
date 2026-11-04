# Build context is the repository root: relay-compiler reads ../schema.graphql.
FROM node:26.12.1-slim AS builder
RUN npm install -g corepack@0.36.0 && corepack enable
WORKDIR /app/web

COPY web/package.json web/pnpm-lock.yaml web/pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile

COPY schema.graphql /app/
COPY web ./
RUN pnpm build

FROM nginxinc/nginx-unprivileged:1.31-alpine AS final
COPY web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/web/dist /usr/share/nginx/html
EXPOSE 8080
