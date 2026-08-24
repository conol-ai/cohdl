-- registry.cohdl.org D1 schema (RFC-030).
CREATE TABLE IF NOT EXISTS accounts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL, -- pbkdf2$<iters>$<salt hex>$<hash hex>
  is_official INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS brands (
  brand TEXT PRIMARY KEY,
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  verified INTEGER NOT NULL DEFAULT 0 -- human-gated (RFC-030): never self-service
);
CREATE TABLE IF NOT EXISTS tokens (
  token_hash TEXT PRIMARY KEY, -- sha256 hex of the bearer token
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS packages (
  name TEXT PRIMARY KEY,
  tier TEXT NOT NULL CHECK (tier IN ('official', 'brand', 'contrib')),
  owner_account INTEGER NOT NULL REFERENCES accounts(id),
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS versions (
  name TEXT NOT NULL REFERENCES packages(name),
  version TEXT NOT NULL,
  hash TEXT NOT NULL,   -- the server-computed RFC-029 content hash (authoritative)
  size INTEGER NOT NULL,
  r2_key TEXT NOT NULL,
  published_at TEXT NOT NULL,
  -- `[package]` metadata, read from the manifest inside this version's own
  -- archive at publish. Per-version because a manifest is a per-version
  -- fact; anything "package-level" derives from the newest version.
  description TEXT,
  license TEXT,
  repository TEXT,
  docs TEXT,            -- JSON array of the version's RFC-017 `#[doc]` paths
  -- Byte size of the version's api-docs sidecar in R2 (docs/apidocs.md).
  -- NULL = no docs uploaded; the sidecar is replaceable, so this tracks
  -- the latest upload.
  api_docs_size INTEGER,
  -- Content-addressed sidecar object selected atomically with its search
  -- projection. NULL means a pre-0003 upload at the legacy fixed R2 key.
  api_docs_r2_key TEXT,
  PRIMARY KEY (name, version)
);
CREATE INDEX IF NOT EXISTS versions_recent ON versions (published_at DESC);

-- Searchable public parts from the newest published version of each package.
-- The source of truth remains that version's API-doc sidecar in R2; this is a
-- replaceable discovery index. Trigrams make identifier/MPN substring search
-- efficient without a leading-wildcard scan.
CREATE VIRTUAL TABLE IF NOT EXISTS part_search USING fts5(
  package_name UNINDEXED,
  package_version UNINDEXED,
  fq,
  name,
  device,
  intent,
  searchable,
  avl_json UNINDEXED,
  tokenize = 'trigram'
);
