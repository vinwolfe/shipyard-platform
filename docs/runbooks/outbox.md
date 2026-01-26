# Outbox runbook (local)

## Goal
Prove “DB write + side effect intent” correctness:
- Orders create enqueues an outbox row in the same DB transaction.
- Worker drains pending rows and marks them delivered.

## Prereqs
- Runtime up (includes migrations): `make up`
  - If you didn’t use `make up`, run: `make migrate`
- Service up: ensure `fulfilment-api` is running
- Worker up: ensure `fulfilment-outbox-worker` is running

## Sanity check: table exists
```sql
SELECT to_regclass('public.outbox');
```
Expected:
- Returns outbox

## Verify: enqueue
1) Create an order (use your existing request)
2) Inspect outbox rows:
```sql
SELECT id, event_type, status, attempts, created_at
FROM outbox
ORDER BY created_at DESC
LIMIT 10;
```

Expected:
- an `order.created` row appears
- status is `PENDING` briefly, then `SENT` once worker processes it

## Verify: observe worker behaviour

Tail worker logs:
```bash
make logs-service SVC=fulfilment-outbox-worker
```

Expected:
- Logs show delivery attempts (e.g., outbox.delivered)

## Verify: drain backlog
1. Stop worker container
2. Create 3–5 orders
3. Confirm rows are `PENDING`
4. Start worker container
5. Confirm rows transition to `SENT`

## Failure drill: retry
1. Temporarily force delivery to fail (e.g., make deliver() return error)
2. Confirm the row transitions:
   - PROCESSING -> PENDING
3. Confirm:
   - attempts increments
   - available_at moves forward 

## Notes
- Delivery is at-least-once. Downstream consumers should be idempotent for end-to-end exactly-once effects
- If you see relation "outbox" does not exist, migrations were not applied (run make migrate or make up)