# ADR-0003: Primary language is Rust

- Status: Accepted
- Date: 2026-01-09
- Owners: Vin
- Tags: #architecture

## Context
Shipyard is a compounding platform project: shared “harbour equipment” is expected to emerge as reusable crates, and multiple services (“ships”) should share consistent contracts and golden paths. The project also explicitly optimises for platform craftsmanship and learning outcomes, accepting some early velocity cost.

Multiple languages (e.g., Go, Rust, TS/Node) are viable choices for backend/platform work. The decision should reflect Shipyard’s constraints and intended style of reuse.

## Decision
- Chosen option: Rust
- Summary:
  - Rust is the primary language for services and shared crates.
  - We accept a steeper learning curve in exchange for stronger contract discipline and long-term maintainability of shared components.

## Options considered
### Option A: Go (rejected: optimises for simplicity/velocity over the desired crate-first contract discipline)
- Pros:
  - Fast iteration and simple operational footprint
  - Mature ecosystem for platform/backend systems
- Cons:
  - Relies more on conventions/tests for certain classes of invariants
  - Less aligned with a crate-first reuse strategy in this repo
- Risks:
  - The project may drift toward “ship fast” habits at the expense of the intended platform craftsmanship outcomes

### Option B: Rust (chosen)
- Pros:
  - Strong foundation for reusable crates with explicit contracts
  - Encourages correctness and clarity in shared components
  - Good long-term maintenance characteristics for platform libraries
- Cons:
  - Higher learning curve and potential early velocity drag
  - Compile times and ecosystem complexity can be friction if unmanaged
- Risks:
  - Over-abstracting early due to language power (mitigated by thin-platform, ship-first rule)

### Option C: TypeScript/Node (rejected: runtime type safety + ops trade-offs)
- Pros:
  - Very fast iteration; broad ecosystem
- Cons:
  - Runtime type safety; greater reliance on tests and conventions
- Risks:
  - Overhead maintaining strictness/robustness at scale without additional constraints

## Consequences
- Positive:
  - Shared “equipment” can be extracted as crates with strong API contracts
  - Cross-cutting patterns can be expressed consistently and safely
- Negative:
  - Requires discipline to keep iteration fast and avoid over-engineering
- Follow-ups:
  - Keep systems minimal and extract shared crates only when justified by immediate consumers.

## Validation plan
- Success metrics:
  - The reference ship (fulfilment API) can be built, tested, and iterated quickly with the golden path.
  - Shared components extracted as crates improve reuse without increasing friction.
- Rollback plan:
  - If Rust learning/velocity becomes a sustained blocker, reduce scope and keep the platform minimal rather than switching language midstream.