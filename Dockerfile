FROM rust:latest AS builder

WORKDIR /wallguard-server

RUN apt-get update && \
    apt-get install -y --no-install-recommends cmake protobuf-compiler libprotobuf-dev && \
    apt-get clean && \ 
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./

COPY . .

RUN cargo build --release -p wallguard-server

FROM debian:trixie-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends libgcc-s1 libstdc++6 ca-certificates openssh-client && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /wallguard-server/target/release/wallguard-server .

EXPOSE 50051
EXPOSE 4444

CMD ["./wallguard-server"]