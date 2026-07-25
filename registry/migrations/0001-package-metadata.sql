-- Bring a registry DB created before `[package]` metadata existed up to the
-- current schema.sql. Additive and one-shot: SQLite has no
-- `ADD COLUMN IF NOT EXISTS`, so re-running this fails with "duplicate
-- column name" — that failure means the DB is already migrated.
--
--   npm run db:migrate            (production)
--   npm run db:migrate:local
--
-- Rows published before the migration read as NULL, which is exactly what a
-- manifest declaring none of these keys produces.
ALTER TABLE versions ADD COLUMN description TEXT;
ALTER TABLE versions ADD COLUMN license TEXT;
ALTER TABLE versions ADD COLUMN repository TEXT;
ALTER TABLE versions ADD COLUMN docs TEXT;
