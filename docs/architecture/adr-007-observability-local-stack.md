# ADR-0007: Local observability stack + request correlation contract (OTel Collector + Jaeger + Prometheus)

- Status: Accepted
- Date: 2026-01-20
- Owners: Vin
- Tags: #observability #runtime #architecture

## Context
Shipyard is a compounding platform (“harbour”) for building and operating services (“ships”) with a repeatable golden path. Operability must be baked in from day one to avoid “it works locally” systems that are opaque in failure.

We want a Loop 1 setup that provides fast feedback and proves the observability contract without introducing unnecessary platform inventory or cloud-specific coupling.

## Decision
- Chosen option: OpenTelemetry Collector for OTLP ingest + export, Jaeger for local trace UI, Prometheus for metrics scraping, and structured JSON logs to stdout with correlation fields.
- Summary:
  - Traces: services export OTLP spans to an OTel Collector; Jaeger provides immediate local UI payoff.
  - Metrics: Prometheus scrapes `/metrics` (exporter implementation can evolve; scrape contract is stable).
  - Logs: JSON structured logs to stdout include `request_id`, `trace_id`, and `span_id` for cross-signal correlation and future cloud compatibility.

## Options considered
### Option A: Direct export to Jaeger + Prometheus without a Collector (rejected: reduces flexibility and future portability)
- Pros:
  - Fewer moving parts locally
  - Faster initial setup if using vendor-specific exporters
- Cons:
  - Harder to swap backends or add processing (sampling, filtering, attribute enrichment)
  - Less aligned with cloud portability where a Collector is a common boundary
- Risks:
  - Early lock-in to specific backends/exporters increases migration cost later

### Option B: OTel Collector as the OTLP boundary + Jaeger + Prometheus + JSON stdout logs (chosen)
- Pros:
  - Standard OTLP boundary enables backend swaps with config changes (not code refactors)
  - Collector supports future processing/enrichment without changing services
  - Jaeger provides fast trace UI payoff; Prometheus is the de facto scrape standard
  - JSON stdout logging is cloud-friendly and works with container runtimes by default
- Cons:
  - Adds one extra local component (Collector) to run and understand
- Risks:
  - Misconfiguration can lead to “no traces exported” confusion (mitigated by runbook + defaults)

### Option C: Adopt an LGTM-style stack locally now (Grafana + Loki + Tempo + Mimir) (deferred: higher ops cost than Loop 1 needs)
- Pros:
  - Closer to a “full” production-style observability platform
  - Unified Grafana experience across signals
- Cons:
  - Higher setup and maintenance overhead
  - More knobs and operational complexity during foundational loops
- Risks:
  - Spending time operating the observability stack instead of building/operating the service

## Consequences
- Positive:
  - Clear Loop 1 observability contract: traces + metrics + correlated logs for every request path
  - High-feedback local verification via Jaeger and Prometheus UIs
  - Portable architecture: future backend changes are primarily configuration changes (Collector/exporters)
- Negative:
  - Local runtime includes multiple services (Collector, Jaeger, Prometheus) that must be managed
  - `/metrics` scraping can produce background request noise in logs (mitigated via log filters)
- Follow-ups:
  - Document and maintain a single verification runbook: `docs/runbooks/observability-local.md`.
  - Keep the contract stable; extend only when a concrete consumer requires more (e.g., SLOs, richer error taxonomy).
  - Defer choosing long-term vendor backends (GCP Logging/Monitoring or LGTM) until later loops.

## Validation plan
- Success metrics:
  - A request produces a trace visible in Jaeger for `fulfilment-api`.
  - Logs for a request include `request_id`, `trace_id`, and `span_id`.
  - Prometheus can reach the service scrape target (`/metrics`) (even if metrics are minimal initially).
- Rollback plan:
  - If trace export causes friction, disable OTLP export by unsetting `OTEL_EXPORTER_OTLP_ENDPOINT`.
    Logs remain available and the service remains operable; traces can be re-enabled later.

## References
- `docs/runbooks/observability-local.md`