# ADR-0009: Idempotency strategy for order creation (Idempotency-Key + Postgres)

- Status: Accepted
- Date: 2026-01-24
- Owners: Vin
- Tags: #correctness #persistence #api #architecture

## Context
Loop 2 focuses on **data + correctness under failure**. A core failure mode for an order-creation API is **duplicate writes** caused by client retries (timeouts, network failures, gateway retries, or user double-submits). Without an idempotency strategy, retries can create duplicate orders and downstream side effects.

Shipyard constraints and forces:
- Thin platform: no generic platform capability without a concrete consumer.
- Golden paths: one paved way to do retries safely.
- Portability: the approach should work locally and in future cloud adapters without refactors.

We already have:
- Structured logs with `request_id`, `trace_id`, `span_id`
- Standard JSON error envelope (`ApiError`)
- `409 Conflict` support (`ApiError::conflict`)

## Decision
- Chosen option: **Idempotency-Key header** for `POST /api/v1/orders` implemented using a **Postgres-backed idempotency table**.
- Summary:
  - Clients send `Idempotency-Key` for create-order requests; the server guarantees **at-most-once order creation effect per key**.
  - The service stores an idempotency record keyed by **(endpoint, idempotency_key)** and returns the **same result** on replay.
  - If the same key is reused with a different request payload, return **409 Conflict** to prevent ambiguous outcomes.
  - If a request is already in progress for a key, return **409 Conflict** (retry later) to keep behaviour explicit and simple.

### Client responsibilities (contract)
- Generate **one Idempotency-Key per user intent** (e.g., one “Create Order” action).
- Reuse the **same key** for retries of the same intent (automatic retry, refresh, timeout retry).
- Do not reuse keys across different intents or different payloads.

## Options considered
### Option A: No idempotency; rely on clients to “be careful” (rejected: duplicates are inevitable under retries)
- Pros:
  - Zero implementation effort
- Cons:
  - Duplicate orders under retry conditions
  - Hard to reason about correctness during failures
  - Forces downstream compensations and manual cleanup
- Risks:
  - Data integrity issues and operational burden increase quickly

### Option B: Idempotency-Key header + Postgres idempotency table (chosen)
- Pros:
  - Standard, widely understood API pattern for safe retries
  - Simple to implement with Postgres constraints (unique key) and small schema
  - Works in local Compose and future cloud deployments unchanged
  - Aligns with thin-platform: implemented where it’s needed (order create)
- Cons:
  - Requires schema and storage for idempotency records
  - Must decide how long to retain records (TTL/cleanup)
- Risks:
  - Table growth if cleanup is ignored (mitigate with retention policy later)
  - If an `IN_PROGRESS` row is left behind (crash between claim and completion), clients may receive repeated `409` until recovery (mitigate with runbook now; add staleness policy later)

### Option C: Natural idempotency via unique business key (e.g., external_id unique) (deferred: depends on domain guarantees we don’t have yet)
- Pros:
  - No extra idempotency table; simpler storage
- Cons:
  - Requires a reliable unique domain key and agreement on semantics
  - Harder to support multiple creates with same external_id if domain changes
- Risks:
  - Locks the API into assumptions that may not hold for real fulfilment flows

### Option D: Distributed lock / cache (Redis) or gateway-level idempotency (rejected: adds platform inventory too early)
- Pros:
  - Can reduce DB load in high-throughput scenarios
- Cons:
  - Additional infrastructure and failure modes
  - Not required for current scale and loop goals
- Risks:
  - Violates “no equipment without a ship that needs it” for Loop 2

## Consequences
- Positive:
  - Safe retries for `POST /api/v1/orders` without duplicate rows
  - Clear operational behaviour for replay, mismatch, and in-progress cases
  - Provides a correctness foundation for later patterns (outbox/worker) without premature abstraction
- Negative:
  - Adds a new table and retention concern (cleanup/TTL)
  - Adds extra code path to create-order flow
- Follow-ups:
  - Decide on retention window (e.g., 7–30 days) and cleanup strategy once usage patterns are clearer.
  - Add an `IN_PROGRESS` staleness policy (e.g., reclaim after N minutes) when needed.
  - Extend idempotency to other write endpoints only when required by concrete consumers.

## Validation plan
- Success metrics:
  - Same `Idempotency-Key` + same payload returns the **same order id** and does not create duplicates.
  - Same key + different payload returns **409 Conflict**.
  - Key marked `IN_PROGRESS` causes requests to return **409 Conflict** (explicit retry).
  - Logs/traces make the idempotency decision observable (hit/miss/replay/mismatch/in-progress).
- Rollback plan:
  - If idempotency causes unexpected friction, temporarily disable the behaviour while preserving the DB schema.

## References
- `docs/runbooks/idempotency.md`
- `docs/runbooks/persistence-local.md`