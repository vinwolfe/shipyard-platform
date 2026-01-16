# Configuration

This repository follows a single golden path for runtime configuration:
**environment variables → typed config → fail fast**.

---

## Variables

### `ENV`
- Values: `dev` | `test` | `prod`
- Default: `dev`

### `SERVICE_PORT`
- Type: TCP port number
- Default: `8080`
- Notes: `0` is invalid and will fail fast at startup.

### `OTEL_EXPORTER_OTLP_ENDPOINT`
- Type: string (endpoint URL)
- Default: unset
- Notes: when set, must not be empty.

---

## Example

```bash
export ENV=dev
export SERVICE_PORT=8080
# export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```