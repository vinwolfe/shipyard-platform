# ADR-0002: Dev golden path via Make targets

- Status: Accepted
- Date: 2026-01-09
- Owners: Vin
- Tags: #ci #architecture

## Context
We want a repeatable, low-friction developer experience to run, test, and verify locally. Without standard commands, workflows drift and verification becomes inconsistent. We also want to avoid introducing runtime orchestration complexity (Docker Compose) before it is required.

## Decision
- Chosen option: Root Makefile for dev workflow commands (non-runtime)
- Summary:
  - Provide a single golden path for dev commands: `dev`, `build`, `test`, `fmt`, `lint`, `check`, `smoke`, `ci`.
  - Reserve runtime orchestration commands (`up/down/logs/...`) for a later ticket.
  - Ensure local commands map cleanly to remote CI later.

## Options considered
### Option A: No Makefile; run cargo commands manually (rejected: increases friction and inconsistency)
- Pros:
  - Zero setup
  - No “extra tooling”
- Cons:
  - Inconsistent workflows over time
  - Harder onboarding and verification
- Risks:
  - Drift between “how I run it” and “how it should be run”

### Option B: Makefile for dev commands, keep runtime separate (chosen)
- Pros:
  - Clear golden path for all contributors (including future you)
  - Easy CI parity (`make ci`)
  - Keeps runtime orchestration isolated until needed
- Cons:
  - Small maintenance overhead (targets must stay accurate)
- Risks:
  - Target creep if we keep adding options instead of one paved route

### Option C: Include Docker Compose runtime targets immediately (deferred: not required yet)
- Pros:
  - One command could start everything sooner
- Cons:
  - Adds operational complexity before required
  - Couples dev workflow with runtime orchestration too early
- Risks:
  - Slows iteration during the first ship skeleton stage

## Consequences
- Positive:
  - Consistent developer workflow and verification steps
  - Easier to document and automate in CI
- Negative:
  - Must keep Makefile targets aligned with repo structure and tooling choices
- Follow-ups:
  - Add runtime targets (`up/down/logs/...`).
  - Add remote CI workflow calling `make ci`.