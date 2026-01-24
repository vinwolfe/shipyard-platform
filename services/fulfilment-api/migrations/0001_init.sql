-- Baseline schema for fulfilment-api
-- Note: This is intentionally minimal; storage usage will be introduced later

CREATE TABLE IF NOT EXISTS orders (
  id UUID PRIMARY KEY,
  external_id TEXT NOT NULL UNIQUE,
  item_count INTEGER NOT NULL,
  total_qty INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);