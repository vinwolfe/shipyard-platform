# Local Runtime (Docker Compose)

This runbook describes how to run the local “harbour” runtime for Shipyard using Docker Compose.
It is intentionally short and focused on golden paths.

---

## Prerequisites

- Docker Desktop (or Docker Engine + Compose)
- Make
- (Optional) Rust toolchain if you also run the service directly via `make dev`

---

## Start the runtime stack

From the repository root:

```bash
make up
```

This will build and start the runtime services in the background.

Expected URLs
- Fulfilment API: http://localhost:8080
- Jaeger UI: http://localhost:16686
- Prometheus UI: http://localhost:9090

---

## Stop the runtime stack

```bash
make down
```

## Restart the runtime stack

```bash
make restart
```

If you need to rebuild images (after Dockerfile changes):
```bash
make restart-full
```

---

## Logs

Tail all logs:
```bash
make logs
```

Tail a single service:
```bash
make logs-service SVC=fulfilment-api
make logs-service SVC=otelcol
make logs-service SVC=jaeger
make logs-service SVC=prometheus
```

---

## Verify service endpoints

If the runtime stack is up:
```bash
make smoke
```

Manually:
```bash
curl -i http://localhost:8080/healthz
curl -i http://localhost:8080/readyz
```

---

## Troubleshooting

### Port already in use

If startup fails with a port binding error (e.g. 8080, 16686, 9090):
- Identify the process/container using the port and stop it, or
- Change the host port mappings in `ops/compose/docker-compose.yml`.

Quick check:
```bash
lsof -i :8080
lsof -i :16686
lsof -i :9090
```

### Compose starts, but the API is not reachable

Check the service logs:
```bash
make logs-service SVC=fulfilment-api
```

Ensure the container is running:
```bash
docker ps
```

### Jaeger UI loads but traces are missing

This is expected until tracing is instrumented and exported to the collector.

Confirm collector is running:
```bash
make logs-service SVC=otelcol
```

### Prometheus UI loads but targets show “DOWN”

This is expected until the API exposes /metrics and Prometheus is configured to scrape it.

Once metrics are implemented, targets should become healthy automatically.