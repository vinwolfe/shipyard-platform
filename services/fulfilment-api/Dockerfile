# NOTE: This Dockerfile intentionally uses `COPY . .` for reliability in early loops.
# When Docker build times become painful, switch to a workspace-safe caching strategy
# using cargo-chef

# --- Build stage ---
FROM rust:bookworm AS builder
WORKDIR /app

# Copy full workspace (simple + reliable for early project)
COPY . .

# Build real binary
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