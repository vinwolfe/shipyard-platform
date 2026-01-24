# ADR-008: Persistence bootstrap for Loop 2 (Postgres + service-owned migrations)

- Status: Accepted
- Date: 2026-01-24
- Owners: Vin
- Tags: #db #persistence #architecture

## Context
Loop 2 introduces persistence and correctness under failure. We need a repeatable baseline for local development and later CI/production evolution.

For Shipyard, services are treated as independently deployable units (“ships”) inside a monorepo (“harbour”). Database schema changes are part of a service’s internal implementation and should remain owned by the service to avoid cross-service coupling and coordination overhead.

Constraints and forces:
- Thin platform: introduce only what the service needs now.
- Golden path: one paved way to run local DB + apply migrations.
- Portability: keep deployment and migration approach adaptable later.

## Decision
- Chosen option: Local Postgres for development + service-owned migrations living inside each service.
- Summary:
  - Run Postgres via docker compose for local runtime.
  - Store migrations under `services/<service>/migrations/` to make schema ownership explicit.
  - Provide a single golden-path command to apply migrations.

## Options considered
### Option A: Central migrations directory at repo root (rejected: unclear ownership + cross-service coupling)
- Pros:
  - One place to find all migrations
- Cons:
  - Ownership becomes ambiguous once multiple services exist
  - Increases coordination costs and release coupling
- Risks:
  - “Shared DB schema” becomes an accidental platform dependency

### Option B: Migrations owned by each service (chosen)
- Pros:
  - Clear ownership and encapsulation
  - Migrations evolve with service code in the same PR
  - Works for DB-per-service later without restructuring
- Cons:
  - Requires per-service tooling/commands (mitigated by golden path)
- Risks:
  - Teams must avoid sharing tables across services (documented as a boundary rule)

## Consequences
- Positive:
  - Strong ownership boundaries: schema belongs to the service.
  - Easy to onboard: find DB changes next to the service code.
- Negative:
  - If we later choose a shared schema across services, it requires explicit governance (discouraged).
- Follow-ups:
  - Add a minimal migration runner for `fulfilment-api`.
  - Add a runbook section for local DB + migrations.

## Validation plan
- Success metrics:
  - Postgres runs locally with docker compose.
  - Migrations apply successfully with a single command.
- Demo steps:
  - `make up`
  - `make migrate`
- Rollback plan:
  - If migration approach becomes burdensome, switch to a CLI-based approach (sqlx-cli) without changing migration locations.

## References
- `ops/compose/docker-compose.yml`
- `services/fulfilment-api/migrations/`