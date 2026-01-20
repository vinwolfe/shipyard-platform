# ADR-0006: Runtime configuration is env-only, typed, and fail-fast (envy)

- Status: Accepted
- Date: 2026-01-16
- Owners: Vin
- Tags: #runtime #configuration

## Context
Shipyard aims to provide a repeatable “golden path” for building and operating services with clear runtime contracts. Configuration is a foundational contract: if it is inconsistent, implicit, or permissive, services become harder to deploy, debug, and operate.

## Decision
- Chosen option: Env-only configuration using `envy` + `serde`, with explicit validation and fail-fast startup.
- Summary:
  - Configuration comes from environment variables only (12-factor style).
  - Deserialize into a typed struct (serde), validate explicitly, and fail startup on invalid config.
  - Provide a test-friendly loader (`from_iter`) to avoid mutating process env in tests.

## Options considered
### Option A: Layered config system (`config` crate: files + env + overrides) (rejected: extra complexity and precedence rules early)
- Pros:
  - Supports multiple sources (files, env, overrides) and merging
  - Useful for complex multi-environment profiles
- Cons:
  - More knobs/precedence rules increase cognitive load and drift risk
  - Adds configuration “framework” inventory before the project needs it
- Risks:
  - Subtle precedence bugs and inconsistent developer/operator expectations

### Option B: Env-only typed config (`envy` + `serde`) (chosen)
- Pros:
  - Minimal, explicit, and aligns with 12-factor portability
  - Typed deserialization + explicit validation improves correctness
  - Easy to standardise across services as a platform contract
- Cons:
  - No first-class file layering (by design)
- Risks:
  - If future needs require layering, we may need to revisit strategy (mitigated by adapters and documentation)

### Option C: Manual `std::env` parsing (rejected: repetitive parsing and error handling)
- Pros:
  - No external dependency
  - Fully custom error messages
- Cons:
  - Boilerplate and inconsistency grow with each config field
  - Harder to keep parsing/validation uniform across services
- Risks:
  - Divergent config behaviour across services and increased maintenance burden

## Consequences
- Positive:
  - One clear runtime contract for configuration across services
  - Faster debugging: invalid config fails immediately with clear errors
  - Easier portability across local/cloud deployment adapters
- Negative:
  - Environment variables must be explicitly managed by each runtime adapter (compose now, cloud later)
  - File-based profiles are not supported unless introduced intentionally later
- Follow-ups:
  - Document the config contract (keys, defaults) in `docs/runbooks/configuration.md`.
  - Keep config additions minimal: add fields only when a concrete consumer requires them.

## Validation plan
- Success metrics:
  - Service fails fast on invalid configuration (e.g., invalid port)
  - Default configuration boots successfully with zero environment variables set
  - Tests can load configuration without modifying global env
- Rollback plan:
  - If env-only becomes insufficient, introduce a layered approach via the `config` crate
    while preserving the typed struct + validation contract as the stable interface.