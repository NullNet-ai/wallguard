# Stage 1: build the React UI
FROM node:22-slim AS ui-builder

WORKDIR /build/ui
COPY ui/package.json ui/package-lock.json ./
RUN npm ci
COPY ui/ ./
RUN npm run build

# Stage 2: build the server binary
FROM rust:1.88-slim AS server-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/     crates/
COPY proto/      proto/
COPY migrations/ migrations/
# Embed the compiled React assets into the server binary
COPY --from=ui-builder /build/ui/dist/ ui/dist/

RUN cargo build --release -p wg-server

# Stage 3: minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=server-builder /build/target/release/wg-server /usr/local/bin/wg-server

EXPOSE 4444 50051 7778 9090
EXPOSE 7777/udp

ENTRYPOINT ["/usr/local/bin/wg-server"]
