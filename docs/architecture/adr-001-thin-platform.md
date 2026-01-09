# ADR-0001: Thin platform, ship-first development

- Status: Accepted
- Date: 2026-01-09
- Owners: Vin
- Tags: #architecture

## Context
Shipyard is a platform (“harbour”) intended to accelerate building and operating services (“ships”). Platform work can easily drift into building abstractions without consumers, creating inventory, complexity, and wasted effort.

## Decision
- Chosen option: Ship-first, thin-platform development
- Summary:
  - Build platform capabilities only when a ship needs them.
  - Every platform capability must be used by the reference ship in the same loop before it is considered “done”.
  - Use the fulfilment API as the initial reference ship/consumer.

## Options considered
### Option A: Platform-first foundation buildout (rejected due to high risk of over-engineering).
- Pros:
  - Potentially cleaner “platform design” up front
  - Fewer early API changes if requirements are perfectly anticipated
- Cons:
  - High risk of unused abstractions and tech tourism
  - Slower feedback loop and delayed proof
- Risks:
  - Over-engineering and scope creep before any real consumer exists

### Option B: Ship-first, thin-platform development (chosen)
- Pros:
  - Fast feedback: decisions validated by real usage
  - Less waste: minimises wasted work and unused equipment
  - Keeps the platform honest and practical
- Cons:
  - Some early duplication may occur before extraction into crates is justified
- Risks:
  - If the reference ship is poorly chosen, it may bias platform direction

## Consequences
- Positive:
  - Lower inventory/WIP and faster iteration
  - Platform capabilities remain grounded in real needs
- Negative:
  - Refactoring may occur as common patterns emerge
- Follow-ups:
  - When duplication becomes painful, extract into crates with immediate consumption.

## Validation plan
- Success metrics:
  - Each platform addition is consumed by fulfilment API within the same loop.
  - No “orphan” crates/modules without a consumer.
- Rollback plan:
  - If a platform abstraction adds friction, revert to in-service implementation until a clearer shared contract emerges.
