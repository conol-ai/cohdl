-- Public part discovery for `cohdl search`. The table is a derived index of
-- the newest release's API-doc sidecar, never package identity. Existing
-- packages are backfilled by re-uploading their newest docs (`cohdl docs
-- --publish`) after this migration is applied.
--
-- FTS5's trigram tokenizer supports efficient identifier and MPN substring
-- matches. UNINDEXED columns are stored return metadata; the remaining text
-- columns participate in MATCH and bm25 ranking.
--
-- The R2 pointer makes replaceable API-doc uploads concurrency-safe: bytes
-- use immutable content-addressed keys, while this pointer and the derived
-- part rows change in one D1 transaction. Existing rows remain NULL and are
-- served from the legacy fixed key until their documented backfill upload.
ALTER TABLE versions ADD COLUMN api_docs_r2_key TEXT;

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
