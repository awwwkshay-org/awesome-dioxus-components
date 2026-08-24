FROM rust:1.96.1-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add wasm32-unknown-unknown \
    && cargo install dioxus-cli@0.7.9 --locked

# Keep the copy list explicit so local environment files never enter a layer.
COPY Cargo.toml Cargo.lock ./
COPY packages/shared/Cargo.toml ./packages/shared/Cargo.toml
COPY packages/shared/src ./packages/shared/src
COPY apps/awesome-dioxus-components-api/Cargo.toml ./apps/awesome-dioxus-components-api/Cargo.toml
COPY apps/awesome-dioxus-components-api/migrations ./apps/awesome-dioxus-components-api/migrations
COPY apps/awesome-dioxus-components-api/src ./apps/awesome-dioxus-components-api/src
COPY apps/awesome-dioxus-components-ui/Cargo.toml ./apps/awesome-dioxus-components-ui/Cargo.toml
COPY apps/awesome-dioxus-components-ui/Dioxus.toml ./apps/awesome-dioxus-components-ui/Dioxus.toml
COPY apps/awesome-dioxus-components-ui/assets ./apps/awesome-dioxus-components-ui/assets
COPY apps/awesome-dioxus-components-ui/src ./apps/awesome-dioxus-components-ui/src

WORKDIR /app/apps/awesome-dioxus-components-ui
RUN dx build --web --release --debug-symbols false

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/dx/awesome-dioxus-components-ui/release/web/server /usr/local/app/server
COPY --from=builder /app/target/dx/awesome-dioxus-components-ui/release/web/public /usr/local/app/public
WORKDIR /usr/local/app
ENV IP=0.0.0.0 PORT=8080 API_URL=http://api:3001
EXPOSE 8080
USER 65532:65532
ENTRYPOINT ["./server"]
