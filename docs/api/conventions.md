# API Conventions

This document defines the baseline HTTP/API contracts for services in this repository.
It is intentionally small and opinionated to preserve golden paths.

---

## Versioning

- All business endpoints must be versioned under: `/api/v1`
- Runtime contract endpoints remain unversioned:
  - `GET /healthz`
  - `GET /readyz`

---

## Request correlation (`x-request-id`)

Every request is assigned a correlation identifier used for debugging and (later) tracing/log correlation.

### Behaviour
- If the request includes `x-request-id`, the service reuses it.
- Otherwise, the service generates a new UUIDv4 request id.
- The service must **always** return `x-request-id` on the response (success or error).

### Why
- Enables end-to-end correlation across proxies, gateways, and downstream services.
- Makes it trivial to join logs/traces/metrics later without refactoring.

---

## Error model (JSON)

All error responses must use a consistent JSON envelope.

### Shape
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "items must not be empty",
    "request_id": "3f2c0d7e-5e2a-4c8c-8e7c-2d5e0f4c9c8a",
    "details": { "field": "items" }
  }
}

### Rules
- `error.code` (string) is required and stable.
- `error.message` (string) is required and human-readable.
- `error.request_id` (string) is required and must match the response `x-request-id`.
- `error.details` is optional and should be a JSON object when present.

### Minimum status mapping
- 400 — validation errors and malformed requests
- 404 — route not found
- 500 — unexpected internal errors

---

## Notes
- This is a baseline contract. Do not expand it unless a concrete use-case requires it.
- If a future standard is adopted (e.g., RFC 7807), it should be documented and applied consistently.