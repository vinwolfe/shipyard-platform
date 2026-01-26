-- Transactional outbox table for reliable side effects.

CREATE TABLE IF NOT EXISTS outbox (
  id            UUID PRIMARY KEY,
  event_type    TEXT NOT NULL,
  payload       JSONB NOT NULL,
  
  status        TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'PROCESSING', 'SENT', 'FAILED')),
  attempts      INT  NOT NULL DEFAULT 0,
  
  available_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  locked_at     TIMESTAMPTZ,

  last_error    TEXT,

  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Only index rows the worker actually scans.
CREATE INDEX IF NOT EXISTS outbox_pending_idx
  ON outbox (status, available_at);

CREATE INDEX IF NOT EXISTS outbox_created_at_idx
  ON outbox (created_at);