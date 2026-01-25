# Idempotency (orders)

This runbook verifies Shipyard’s idempotency behaviour for order creation and provides operator recovery steps.

## Contract

For `POST /api/v1/orders`:
- If `Idempotency-Key` is provided:
  - First request claims the key and processes the request.
  - Replays with the same key + same payload return the stored response.
  - Reuse with the same key + different payload returns `409 Conflict`.
  - If the key is `IN_PROGRESS`, requests return `409 Conflict` (retry later).
- If `Idempotency-Key` is NOT provided:
  - The request executes normally.
  - Duplicate creates are possible under retries (by design; thin default).

---

## Verify (curl)

Set:
```bash
BASE_URL=http://localhost:8080
```


### Create order (first claim)

```bash
curl -i -X POST "$BASE_URL/api/v1/orders" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: idem-demo-1" \
  -d '{"external_id":"ord_demo_1","items":[{"sku":"ABC","qty":1}]}'
```

### Replay (same key + same payload)

Expect: 200 OK and the same order id as the first response.
```bash
curl -i -X POST "$BASE_URL/api/v1/orders" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: idem-demo-1" \
  -d '{"external_id":"ord_demo_1","items":[{"sku":"ABC","qty":1}]}'
```

### Mismatch (same key + different payload)

Expect: 409 Conflict.
```bash
curl -i -X POST "$BASE_URL/api/v1/orders" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: idem-demo-1" \
  -d '{"external_id":"ord_demo_2","items":[{"sku":"XYZ","qty":2}]}'
```

### No key behaviour (thin default)

Expect: two different creates may produce two different orders.
```bash
curl -i -X POST "$BASE_URL/api/v1/orders" \
  -H "Content-Type: application/json" \
  -d '{"external_id":"ord_demo_no_key","items":[{"sku":"ABC","qty":1}]}'
```

---

## Verify (DB inspection)

Connect to Postgres (example if running in Compose):
```bash
psql "$DATABASE_URL"
```

Inspect a key:
```sql
SELECT endpoint, idempotency_key, status, request_hash, response_status, created_at, updated_at
FROM idempotency_keys
WHERE endpoint = 'POST:/api/v1/orders'
  AND idempotency_key = 'idem-demo-1';
```

Look for recent keys:
```sql
SELECT endpoint, idempotency_key, status, updated_at
FROM idempotency_keys
ORDER BY updated_at DESC
LIMIT 20;
```

---

## Verify (logs/traces)

Expected:
- Response header includes x-request-id.
- Logs include request_id, trace_id, span_id.

---

## Failure drill: stuck `IN_PROGRESS`

If a request crashes after claiming a key but before completion, subsequent retries may return 409 Conflict repeatedly.

### Manual recovery (dev/local only)

If safe, delete the stuck row and retry the request.
```sql
DELETE FROM idempotency_keys
WHERE endpoint = 'POST:/api/v1/orders'
  AND idempotency_key = 'idem-demo-1'
  AND status = 'IN_PROGRESS';
```

> TODO: Introduce a staleness policy for IN_PROGRESS rows (reclaim after N minutes) when needed.