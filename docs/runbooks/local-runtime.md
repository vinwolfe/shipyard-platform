# Local Runtime (Docker Compose)

This runbook describes how to run the local “harbour” runtime for Shipyard using Docker Compose.
It is intentionally short and focused on golden paths.

---

## Prerequisites

- Docker Desktop (or Docker Engine + Compose)
- Make
- (Optional) Rust toolchain if you also run the service directly via `make dev`

---

## Start the runtime stack (full harbour)

From the repository root:

```bash
make up
```

This will build and start the runtime services in the background.

Expected URLs
- Fulfilment API: http://localhost:8080
- Jaeger UI: http://localhost:16686
- Prometheus UI: http://localhost:9090
- Postgres: localhost:5432

---

## Start Postgres only (for local cargo dev)

```bash
make dev-db-up
make dev
# when done
make dev-db-down
```

Notes
- `make dev` loads environment variables from `.env` (repo root) if present.
- `DATABASE_URL` must be set for `make dev` now that the service requires a DB.

---

## Stop the runtime stack

```bash
make down
```

## Restart the runtime stack

```bash
make restart
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
make logs-service SVC=postgres
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

Readiness semantics
- `/healthz` should return 200 when the service is up.
- `/readyz` should return:
    - 200 when dependencies (DB) are reachable
    - 503 when the DB is not configured / not reachable

---

## Run DB integration tests

DB tests are marked #[ignore] by default and run through the Make target.
```bash
make test-db
```

This target will:
- ensure Postgres is up and healthy
- run ignored DB tests (name-prefixed with db_)
- shut down the DB stack afterwards

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
lsof -i :5432
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

### /readyz returns 503

This usually means Postgres is not reachable.

Check Postgres logs:
```bash
make logs-service SVC=postgres
```

Confirm the service has a correct DATABASE_URL.
- Compose runtime typically uses host postgres (container name / service name).
- Local cargo dev typically uses host localhost.

### DATABASE_URL must be set when running make dev

- Ensure .env exists in the repo root and includes DATABASE_URL=..., or export it in your shell.
- If you want Postgres locally via compose profile:
```bash
make dev-db-up
make dev
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