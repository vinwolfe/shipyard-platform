# NOTE: This Dockerfile intentionally uses `COPY . .` for reliability in early loops.
# When Docker build times become painful, switch to a workspace-safe caching strategy
# using cargo-chef

# --- Build stage ---
FROM rust:bookworm AS builder
WORKDIR /app

# Copy full workspace (simple + reliable for early project)
COPY . .

# Build real binary
RUN cargo build --release -p fulfilment-api --bin fulfilment-api --bin outbox_worker

# --- Runtime stage ---
FROM debian:bookworm-slim
WORKDIR /app

# Minimal runtime deps
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fulfilment-api /usr/local/bin/fulfilment-api
COPY --from=builder /app/target/release/outbox_worker /usr/local/bin/outbox_worker

ENV SERVICE_PORT=8080
EXPOSE 8080

CMD ["fulfilment-api"]