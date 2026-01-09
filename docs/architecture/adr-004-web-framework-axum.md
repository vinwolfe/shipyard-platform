# ADR-0004: Web framework is Axum

- Status: Accepted
- Date: 2026-01-09
- Owners: Vin
- Tags: #architecture

## Context

Shipyard aims to standardise cross-cutting concerns (request identification, tracing, metrics, error shaping) as reusable “harbour equipment” that can be applied consistently across services. The repo therefore benefits from a framework that aligns with composable middleware and common ecosystem conventions.

Multiple Rust web frameworks are viable (e.g., Axum, actix-web). The decision should prioritise consistency with a layered middleware approach and future extraction into shared crates.

## Decision
- Chosen option: Axum
- Summary:
  - Services use Axum for HTTP routing and request handling.
  - Prefer Tower-friendly middleware patterns to keep cross-cutting concerns consistent.

## Options considered
### Option A: actix-web (rejected: different ecosystem conventions vs Tower-first patterns)
- Pros:
  - Mature and fast
  - Good production track record
- Cons:
  - Different middleware/composition model than Tower-first stacks
- Risks:
  - Cross-cutting concerns (observability, request IDs) may not align as cleanly with the planned stack

### Option B: Axum (chosen)
- Pros:
  - Tower ecosystem alignment (middleware, layers)
  - Good ergonomics for building typed APIs
  - Common integration patterns for tracing/metrics
- Cons:
  - Requires comfort with Tower concepts
- Risks:
  - Misuse of layers/extractors can add complexity (mitigated by golden path + examples)

## Consequences
- Positive:
  - Clear conventions for middleware and observability integration
  - Easier to standardise `shipyard-web` patterns later
- Negative:
  - Need to keep examples/docs tight so Tower patterns don’t sprawl
- Follow-ups:
  - Introduce shared web crate only when repeated patterns justify extraction (thin platform rule).

## Validation plan
- Success metrics:
  - `/healthz` and `/readyz` implemented with tests
  - Middleware integration is straightforward when added (request_id, tracing)
- Rollback plan:
  - If Axum creates sustained friction, reassess with a small spike before committing to a framework swap.