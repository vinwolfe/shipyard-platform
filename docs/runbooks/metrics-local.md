# Metrics (local)

This runbook verifies the Prometheus scrape contract and basic HTTP server metrics.

## What we verify
- `fulfilment-api` exposes `/metrics` in Prometheus text format
- Prometheus can scrape the target (UP)
- HTTP request metrics exist and are queryable:
  - `http_requests_total` (counter)
  - `http_request_duration_seconds_bucket` (histogram)

## Start runtime
```bash
make up
```

## Generate traffic
```bash
make smoke
```

> Note: rate(...[1m]) may be empty until Prometheus has scraped a few times.
> Wait ~15–30s and refresh, or run make smoke twice.

## Verify metrics endpoint
```bash
curl -s http://localhost:8080/metrics | head
```

You should see lines like:
- `# HELP ...`
- `# TYPE ...`
- `http_requests_total{...} ...`
- `http_request_duration_seconds_bucket{...} ...`

## Verify Prometheus target

Open: `http://localhost:9090/targets`
Confirm:
- `fulfilment-api` target is UP
- target URL is `http://fulfilment-api:8080/metrics` (inside compose network)

## PromQL Checks (Run in Prometheus UI)

### Requests exist
```promql
http_requests_total
```

### Rate of requests (1m)

Total request rate:
```promql
sum(rate(http_requests_total[1m]))
```

Breakdown by route:
```promql
sum by (route) (rate(http_requests_total[1m]))
```

Breakdown by route + method + status:
```promql
sum by (route, method, status) (rate(http_requests_total[1m]))
```

### p95 latency (5m window)

All routes/status (overall service p95):
```promql
histogram_quantile(
  0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
)
```

Recommended: p95 by route:
```promql
histogram_quantile(
  0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, route)
)
```

p95 by route + method + status:
```promql
histogram_quantile(
  0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, route, method, status)
)
```

### Optional: query via HTTP API
```bash
curl -G 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=http_requests_total'
```

## Notes
- `/metrics` is excluded from HTTP metrics to avoid Prometheus scrape noise
- Labels use matched route patterns via Axum `MatchedPath` (e.g. `/api/v1/orders/validate`) to avoid high-cardinality labels from raw paths
- If `MatchedPath` is absent (e.g., for some fallbacks), the middleware falls back to `req.uri().path()`