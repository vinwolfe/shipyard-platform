# Local persistence runbook (Postgres + migrations)

This runbook documents the golden path for running Postgres locally, applying service-owned migrations, and verifying DB integration via readiness + write/read endpoints.

---

## Contract

- Postgres runs locally via Docker Compose.
- Each service owns its schema changes under `services/<service>/migrations/`.
- Migrations are applied via a Rust binary (`services/<service>/src/bin/migrate.rs`) so the golden path remains Rust-only.
- `/readyz` is DB-backed: it returns 200 only if the DB is reachable.
- Persistence bootstrap includes minimal write + read routes:
  - `POST /api/v1/orders`
  - `GET /api/v1/orders/:id` (UUID)

---

## Prerequisites

- Docker + Docker Compose
- Rust toolchain (stable)
- (Optional) `jq` for pretty-printing JSON in examples

---

## Golden paths

### Option A: Full harbour (service + DB + observability)

```bash
make up
```

Expected URLs
- Fulfilment API: http://localhost:8080
- Postgres: localhost:5432
- Jaeger UI: http://localhost:16686
- Prometheus UI: http://localhost:9090

Verify Postgres is running:
```bash
docker ps | grep postgres
```

### Option B: Cargo dev + DB only

```bash
make dev-db-up
make dev
# when done
make dev-db-down
```

Notes
- `make dev` loads environment variables from .env (repo root) if present.
- `DATABASE_URL` must be set for `make dev` now that the service requires a DB.

---

## Apply migrations

If Postgres is running (via either option above):
```bash
make migrate
```

Override the DB URL if needed:
```bash
DATABASE_URL=postgres://shipyard:shipyard@localhost:5432/shipyard make migrate
```

---

## Verify readiness is DB-backed

Check readiness:
```bash
curl -i http://localhost:8080/readyz
```

Expected:
- 200 OK when DB is reachable
- 503 Service Unavailable when DB is down / unreachable

(Optional) tail service logs:
```bash
make logs-service SVC=fulfilment-api
```

---

## Verify write + read

Create an order:
```bash
curl -sS -X POST http://localhost:8080/api/v1/orders \
  -H 'Content-Type: application/json' \
  -d '{"external_id":"ord_runbook_1","items":[{"sku":"ABC","qty":1}]}' | jq
```

Copy the returned id, then fetch:
```bash
curl -sS http://localhost:8080/api/v1/orders/<id> | jq
```

Expected
- Create returns 201 Created + JSON body with id
- Get returns 200 OK with the same id
- Missing id returns 404 with standard error envelope + request_id

---

## Run DB integration tests

DB integration tests are marked #[ignore] and run via the Make target:
```bash
make test-db
```

---

## Minimal verification loop (fast)

Verification checklist
- Postgres container is running
- make migrate applies migrations successfully
- /readyz returns 200
- Create + get roundtrip works
- make test-db passes

```bash
make up
make migrate
curl -i http://localhost:8080/readyz
curl -sS -X POST http://localhost:8080/api/v1/orders \
  -H 'Content-Type: application/json' \
  -d '{"external_id":"ord_loop_1","items":[{"sku":"ABC","qty":1}]}' | jq
make test-db
make down
```

---

## Troubleshooting


### `/readyz` returns 503
1. Postgres may be down or DATABASE_URL is wrong.

2. Check compose logs:
```bash
make logs-service SVC=postgres
make logs-service SVC=fulfilment-api
```

### Postgres is running but migrations fail
1. Check your DATABASE_URL is correct.

2. Confirm Postgres is reachable:
```bash
nc -zv localhost 5432
```

### “relation does not exist” errors

Migrations may not have been applied. Run:
```bash
make migrate
```

### Resetting local state (destructive)

If local DB is in a bad state, and you want to clean reset:
```bash
make down
docker volume rm postgres_data
make up
make migrate
```

> ⚠️ This deletes all local Postgres data (local-only)

## Notes
- Migrations are not applied automatically on startup to keep lifecycle explicit and debuggable.
- A runtime migration job can be introduced later if deployment adapters require it.