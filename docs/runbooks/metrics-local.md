# Metrics (local)

This runbook verifies the Prometheus scrape contract and basic HTTP server metrics.

## Start runtime
```bash
make up
```

## Generate traffic
```bash
make smoke
```

## Verify metrics endpoint
```bash
curl -s http://localhost:8080/metrics | head
```

## Verify Prometheus target

Open: `http://localhost:9090/targets`
Confirm fulfilment-api target is UP.

## PromQL Checks (Run in Prometheus UI)

Requests exist
```promql
http_requests_total
```

Rate of requests (1m)
```promql
rate(http_requests_total[1m])
```

p95 latency (5m window)
```promql
# All routes/status
histogram_quantile(
  0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
)

# Route
histogram_quantile(
  0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, route)
)

# Route + Method + Status
histogram_quantile(
  0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, route, method, status)
)
```

Optional: query via HTTP API
```bash
curl -G 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=http_requests_total'
```

## Notes
- `/metrics` is excluded from HTTP metrics to avoid scrape noise
- Labels use matched route patterns (e.g. `/api/v1/orders/validate`) to avoid high-cardinality labels