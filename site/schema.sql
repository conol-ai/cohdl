-- cohdl.org D1 schema: the pre-launch waitlist.
--
-- One row per address. `email` is stored trimmed and lower-cased and carries a
-- UNIQUE constraint, so a repeat signup is idempotent rather than a duplicate.
--
-- No raw IP is kept: `ip_hash` is a salted SHA-256 used only to rate-limit and
-- to triage abuse, and it cannot be reversed to an address.
CREATE TABLE IF NOT EXISTS waitlist (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL,
  -- Where the signup came from (?ref=… on the landing page), for attribution.
  source TEXT,
  ip_hash TEXT,
  user_agent TEXT
);

CREATE INDEX IF NOT EXISTS waitlist_created_at ON waitlist (created_at);

-- Serves the per-IP rate limit window.
CREATE INDEX IF NOT EXISTS waitlist_ip_recent ON waitlist (ip_hash, created_at);
