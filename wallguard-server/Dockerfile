FROM rust:latest AS builder

WORKDIR /wallguard-server

RUN apt-get update && \
    apt-get install -y --no-install-recommends cmake protobuf-compiler && \
    apt-get clean && \ 
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends libgcc-s1 libstdc++6 ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /wallguard-server/target/release/wallguard-server .
COPY --from=builder /wallguard-server/tls ./tls/

EXPOSE 50051
CMD ["./wallguard-server"]