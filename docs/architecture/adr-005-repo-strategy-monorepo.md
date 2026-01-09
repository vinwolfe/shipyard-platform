# ADR-0005: Repo strategy is a monorepo

- Status: Accepted
- Date: 2026-01-09
- Owners: Vin
- Tags: #architecture

## Context
Shipyard is a compounding system: shared crates (“harbour equipment”) and multiple services (“ships”) should evolve together with a single golden path. We want consistent tooling, consistent verification, and low friction for cross-cutting improvements.

The project also values incremental extraction: start minimal and extract shared crates only when justified by immediate consumers.

## Decision
- Chosen option: Monorepo
- Summary:
  - Keep platform crates and services in one repository.
  - Standardise commands and structure via golden paths.
  - Prefer clear boundaries (crates vs services) over separate repos.

## Options considered
### Option A: Multi-repo (rejected: higher coordination and versioning overhead early)
- Pros:
  - Strong isolation boundaries
  - Independent versioning/release cadence
- Cons:
  - Higher coordination cost for cross-cutting changes
  - Dependency/version management overhead early
- Risks:
  - Slower iteration and fractured “golden path” experience

### Option B: Monorepo (chosen)
- Pros:
  - Single workspace and consistent tooling
  - Easier refactors and cross-cutting changes
  - One documented way to run/test/verify
- Cons:
  - Requires discipline to keep structure clean
- Risks:
  - Repo can become messy if boundaries aren’t enforced (mitigated by structure + ADRs)

### Option C: Hybrid (deferred: not required until scale demands it)
- Pros:
  - Can split later if needed
- Cons:
  - Adds complexity before it is needed
- Risks:
  - Premature optimisation of repo topology

## Consequences
- Positive:
  - Faster compounding across crates/services
  - Clear, consistent developer experience
- Negative:
  - Must actively maintain structure and avoid dumping ground directories
- Follow-ups:
  - Keep a strict rule: any new shared capability must have an immediate consumer.

## Validation plan
- Success metrics:
  - A fresh clone can run and verify the reference service via one paved workflow
  - Cross-cutting changes (e.g., middleware) can be applied consistently
- Rollback plan:
  - If monorepo overhead becomes significant, split services into separate repos while keeping crates versioned independently.