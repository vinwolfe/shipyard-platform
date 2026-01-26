# ADR-0010: Transactional outbox + worker polling (SKIP LOCKED) for reliable side effects

- Status: Accepted
- Date: 2026-01-26
- Owners: Vin
- Tags: #persistence #correctness #outbox #worker #architecture

## Context
We have Postgres-backed order creation and idempotency. The next failure mode is “DB write succeeds, but the side effect fails” (e.g., publishing an event, notifying another system). Without a reliability pattern, we risk losing side effects or duplicating them unpredictably under retries/crashes.

We want a minimal solution that:
- Preserves the “thin platform” principle (no queues yet).
- Avoids refactors when we later introduce DLQ and richer delivery.
- Proves correctness under failure in a way we can drill and runbook.

## Decision
- Chosen option: Transactional outbox table + polling worker using `FOR UPDATE SKIP LOCKED`.
- Summary:
  - Write domain change + outbox row in the same DB transaction.
  - A worker polls pending outbox rows and claims them using `FOR UPDATE SKIP LOCKED`.
  - Claiming is performed as a single atomic UPDATE (+ RETURNING) to avoid N+1 updates.
  - Failures retry with backoff using `available_at` + `attempts`.
  - Outbox status is constrained to a closed set via a DB CHECK constraint.
  - DLQ will be introduced later by changing delivery/marking rules, not by refactoring producers.

## Options considered
### Option A: Best-effort publish after DB commit (rejected: side effects can be lost)
- Pros:
  - Minimal code
- Cons:
  - Side effects can vanish on crash/timeout between DB commit and publish
- Risks:
  - Silent data loss under failure

### Option B: Transactional outbox + polling worker (chosen)
- Pros:
  - Reliable under crash/retry
  - Uses DB as the only moving part
  - Easy to evolve toward DLQ / external queue later
- Cons:
  - Adds worker process and a table
- Risks:
  - Polling intervals can add latency (acceptable for Loop 2)

### Option C: Introduce Kafka/SQS immediately (deferred: inventory + ops cost)
- Pros:
  - Better scaling characteristics
- Cons:
  - Adds infra complexity too early
- Risks:
  - “Tech tourism” and time sink during foundation loops

## Consequences
- Positive:
  - Orders create can be safely retried and still produce side effect intent reliably.
  - Delivery is at-least-once; downstream consumers should be idempotent for end-to-end exactly-once effects.
  - Worker can be independently operated and drilled.
- Negative:
  - Requires worker lifecycle management and backlog monitoring.
- Follow-ups:
  - Add DLQ semantics once we have a concrete failure policy requirement.
  - Add metrics for backlog and delivery outcomes.

## Validation plan
- Success metrics:
  - Creating an order writes an outbox row in the same transaction.
  - Worker delivers pending rows and marks them as delivered.
  - Worker crash/restart does not lose events (rows remain in DB and will be retried).
- Demo steps:
  - Stop worker, create orders, start worker, watch it drain backlog.
- Rollback plan:
  - Disable worker service; outbox rows accumulate but writes remain consistent.

## References
- `docs/runbooks/outbox.md`