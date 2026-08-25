-- Anonymous requests for missing component libraries, reviewed by official
-- registry accounts. Manufacturer + part number is one durable queue item;
-- repeated demand increments request_count without overwriting the original
-- source information or reopening resolved work.
CREATE TABLE component_requests (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  manufacturer TEXT NOT NULL,
  manufacturer_key TEXT NOT NULL,
  part_number TEXT NOT NULL,
  part_number_key TEXT NOT NULL,
  datasheet_url TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved')),
  request_count INTEGER NOT NULL DEFAULT 1 CHECK (request_count >= 1),
  created_at TEXT NOT NULL,
  last_requested_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  resolved_at TEXT,
  resolved_by_account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
  CHECK (
    (status = 'open' AND resolved_at IS NULL) OR
    (status = 'resolved' AND resolved_at IS NOT NULL)
  ),
  UNIQUE (manufacturer_key, part_number_key)
);
CREATE INDEX component_requests_queue
  ON component_requests (status, request_count DESC, last_requested_at DESC);
CREATE INDEX component_requests_newest
  ON component_requests (status, last_requested_at DESC, id DESC);
