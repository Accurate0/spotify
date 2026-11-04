ARG BINARY_NAME

FROM rust:1.99.1-slim-trixie AS builder
ARG BINARY_NAME

WORKDIR /app

# The CLI's build script reads schema.graphql; queries compile against the .sqlx cache.
COPY Cargo.toml Cargo.lock schema.graphql ./
COPY .sqlx ./.sqlx
COPY crates ./crates

ENV SQLX_OFFLINE=true
RUN \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release --bin ${BINARY_NAME} --bin spotify && \
    cp target/release/${BINARY_NAME} /app/${BINARY_NAME} && \
    cp target/release/spotify /app/spotify

FROM debian:trixie-slim AS final
ARG BINARY_NAME

RUN apt-get update -y && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*
RUN useradd \
    --no-create-home \
    --home-dir "/nonexistent" \
    --shell "/sbin/nologin" \
    --uid "10001" \
    appuser

# The CLI ships alongside the server so imports can also run from inside the cluster.
COPY --from=builder /app/${BINARY_NAME} /usr/local/bin/${BINARY_NAME}
COPY --from=builder /app/spotify /usr/local/bin/spotify

USER appuser

WORKDIR /opt/${BINARY_NAME}

RUN ln -s /usr/local/bin/${BINARY_NAME} executable
ENTRYPOINT ["./executable"]
EXPOSE 8000/tcp
