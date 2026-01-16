# --- Build stage ---
FROM rustlang/rust:nightly-bookworm AS builder
WORKDIR /app

# Cache deps
COPY Cargo.toml Cargo.lock ./
COPY services/fulfilment-api/Cargo.toml services/fulfilment-api/Cargo.toml
RUN mkdir -p services/fulfilment-api/src && \
    echo "fn main() {}" > services/fulfilment-api/src/main.rs && \
    cargo build -p fulfilment-api --release && \
    rm -rf services/fulfilment-api/src

# Build real binary
COPY services/fulfilment-api services/fulfilment-api
RUN cargo build -p fulfilment-api --release

# --- Runtime stage ---
FROM debian:bookworm-slim
WORKDIR /app

# Minimal runtime deps
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fulfilment-api /usr/local/bin/fulfilment-api

ENV SERVICE_PORT=8080
EXPOSE 8080

CMD ["fulfilment-api"]