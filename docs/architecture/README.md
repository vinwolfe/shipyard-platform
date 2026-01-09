# Architecture Decision Records (ADRs)

ADRs capture decisions that shape the system and are are non-trivial to reverse.

## When to write an ADR
Write an ADR when you decide something that affects:
- repo structure and boundaries
- foundational patterns (error model, config approach)
- cross-cutting concerns (observability, deployment contracts, CI)
- a default that many modules/services will inherit

## Naming
`adr-0001-short-title.md`, `adr-0002-short-title.md`, ...

## Status
- Proposed: drafted but not applied broadly yet
- Accepted: the current decision
- Deprecated: no longer recommended for new work
- Superseded: replaced by a newer ADR

## Template
Use the standard template (see [ADR Template](./adr-000-template.md)).

Conventions used in this repo:
- Keep ADRs short; omit sections that would be fluff.
- In **Options considered**, add a short suffix in the title:
  - `(chosen)`
  - `(rejected: <reason>)`
  - `(deferred: <reason>)`

Example:
- `Option A: X (rejected: increases operational overhead)`
- `Option B: Y (chosen)`
- `Option C: Z (deferred: not required yet)`

## Validation mindset
Where applicable, include a lightweight validation plan:
- what we’ll measure
- how we’d roll back if it doesn’t