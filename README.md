# Shipyard

> A thin, production-quality Rust platform (“harbour”) for building and operating services (“ships”) with clear contracts, golden paths, and observable-by-default behavior.
>
> **Stack:** Rust + Axum (monorepo)

---

## Why Shipyard exists

Shipyard is a compounding platform project designed to prove production-quality fundamentals:
- **Build** services with a consistent runtime contract
- **Operate** them with traces/metrics/log correlation from day one
- **Document** decisions and create proof artifacts (OSS + write-ups + devlogs)

The first reference service is **fulfilment API**.

### What “seaworthy” means here
Every request should produce **three signals**:
- **Trace** in Jaeger
- **Metrics** in Prometheus
- **JSON logs** with `trace_id`, `span_id`, `request_id`

---

## Principles (non-negotiable)

- **Thin platform:** no platform capability without an immediate consumer.
- **Golden paths:** one paved way to run, test, verify, and ship.
- **Portability:** stable runtime + observability contracts; cloud adapters later (no refactor).
- **Ship weekly:** every week ends with a shippable increment and proof.
- **No tech tourism:** no extra tech unless triggered by a concrete requirement.

---

## Repository layout

```
shipyard-platform/
  docs/
    api/                             # API contracts and conventions
    runbooks/                        # Operational guides (golden paths, drills)
    architecture/                    # ADRs and design notes
  ops/
    compose/                         # Local runtime stack (docker compose + configs)
  scripts/                           # Local helper scripts used by Make targets
  services/                          # “ships”
    fulfilment-api/
      src/
        main.rs                      # Runtime boot
        lib.rs                       # App assembly seam
        http/router.rs               # Routes + /api/v1
        http/v1/orders.rs            # POST /orders/validate
      tests/                         # Integration tests
      Dockerfile 
  Makefile                           # Golden path commands
```

---

## Quickstart

### Prerequisites
- Rust toolchain (stable)
- Docker + Docker Compose (for `make up` local runtime)

### Run the service directly
```zsh
make dev
```

Service listens on: `http://localhost:8080`

### Run the full local runtime (Compose)
```zsh
make up
```
### Verify endpoints
```zsh
make smoke
```

Or manually:
```zsh
curl -i http://localhost:8080/healthz
curl -i http://localhost:8080/readyz
```

### Run tests
```zsh
make test
```

### Formatting and linting
```zsh
make fmt
make lint
```

### Full local verification
```zsh
make check
```

---

## Docs
- API conventions: [docs/api/conventions.md](docs/api/conventions.md)
- Local runtime runbook: [docs/runbooks/local-runtime.md](docs/runbooks/local-runtime.md)
- Architecture decisions (ADRs): [docs/architecture/](docs/architecture/)

---

## Roadmap (single project, milestone chapters)

### Loop 1 — Foundations
- Service skeleton + health/readiness + validation endpoint
- Local observability runtime (OTel Collector + Jaeger + Prometheus)
- Correlated JSON logs (trace_id, span_id, request_id)

### Loop 2 — Data + correctness under failure
- Postgres + migrations
- Correctness patterns as required
    - Idempotency strategy
    - Outbox/worker pattern + retry semantics
    - Failure drills + runbooks

### Loop 3 — Platform capabilities + cloud adapter
- Add platform capabilities only when triggered by real consumers (auth/gateway/tenancy)
- Cloud deployment adapter
- Security baseline improvements as needed

### Loop 4 — Operations
- SLOs + alerting strategy
- Load testing + performance budgets
- Chaos/failure drills + postmortems
- Delivery pipeline maturity (release discipline, reliability gates)

---

## Contributing

This repo is built as a learning-in-public system. Issues/PRs are welcome:
- keep changes small,
- preserve golden paths,
- add verify steps and docs.

---

## License

MIT
