FROM rust:1.85-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release --bin ajen-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ajen-server /usr/local/bin/
COPY employee-manifests/ /app/employee-manifests/
WORKDIR /app

EXPOSE 3000

CMD ["ajen-server", "--port", "3000", "--manifests-dir", "/app/employee-manifests", "--workspace-dir", "/app/workspaces"]
