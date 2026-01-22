# Observability (local)

This runbook verifies the observability contract:
- traces visible in Jaeger (when OTLP export is enabled)
- JSON logs include request correlation fields
- responses include `x-request-id`

## Start runtime
```bash
make up
```

## Preconditions (traces)
Traces require OTLP export to be enabled for fulfilment-api:
- OTEL_EXPORTER_OTLP_ENDPOINT must be set (compose does this)

## Generate traffic
```bash
make smoke
```

## Verify request id header
```bash
curl -i http://localhost:8080/healthz | sed -n '1,20p'
```
Confirm the response includes x-request-id.

## Verify traces (Jaeger)
- Open: http://localhost:16686
- Service: fulfilment-api (or OTEL_SERVICE_NAME if overridden)
- Find traces for:
    - GET `/healthz`
    - GET `/readyz`
    - POST `/api/v1/orders/validate`

## Verify logs (correlation fields)

Tail service logs:
```bash
make logs-service SVC=fulfilment-api
```

Look for the `request.completed` log event and confirm fields exist:
- `request_id`
- `trace_id`
- `span_id`
- `method`
- `path`
- `status`
- `latency_us`

## Correlation scope (important)
Shipyard guarantees correlation for **request-scoped logs** (the "ship's voyage") via `shipyard-web`:
- `request.completed` log event includes `request_id`, `trace_id`, `span_id`
- request span fields include `method`, `path`
- responses always include `x-request-id`

Non-request logs (startup, background jobs) may not have trace context unless the code creates a span for that work.
This is intentional for Loop 1 (thin platform). When workers/jobs are introduced (Loop 2+), add `job_id` + spans around background work.

## Notes
- If `OTEL_EXPORTER_OTLP_ENDPOINT` is not set, logs still work but traces will not export
- You may see periodic GET `/metrics` requests from Prometheus scraping
- Log verbosity is controlled via `RUST_LOG`