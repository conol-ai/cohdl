-- Token scopes. A CLI token either publishes packages (`publish`, every token
-- minted before this migration) or maintains official supplier data
-- (`admin`), never both. Admin tokens carry an expiry; publish tokens keep
-- none. Both columns are additive, so the Worker running before this
-- migration keeps working after it: apply this BEFORE the Worker that reads
-- `scope` takes traffic.
ALTER TABLE tokens ADD COLUMN scope TEXT NOT NULL DEFAULT 'publish' CHECK (scope IN ('publish', 'admin'));
ALTER TABLE tokens ADD COLUMN expires_at TEXT;
