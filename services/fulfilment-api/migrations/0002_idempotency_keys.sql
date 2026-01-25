CREATE TABLE IF NOT EXISTS idempotency_keys (
  endpoint TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  request_hash TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('IN_PROGRESS', 'COMPLETED')),
  response_status INT,
  response_body JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (endpoint, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_idempotency_keys_idempotency_key
  ON idempotency_keys (idempotency_key);