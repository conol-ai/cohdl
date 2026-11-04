-- Official supplier metadata and catalog snapshots (RFC-034).
CREATE TABLE suppliers (
  id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  product_url_template TEXT NOT NULL,
  class_vocabulary TEXT NOT NULL
);
CREATE TABLE supplier_catalog (
  supplier TEXT NOT NULL REFERENCES suppliers(id) ON DELETE CASCADE,
  pn TEXT NOT NULL,
  mpn TEXT NOT NULL,
  mpn_key TEXT NOT NULL,
  manufacturer_key TEXT NOT NULL,
  manufacturer TEXT NOT NULL,
  package TEXT NOT NULL,
  "class" TEXT NOT NULL,
  stock INTEGER NOT NULL CHECK (stock >= 0),
  links TEXT NOT NULL,
  fetched_at TEXT NOT NULL,
  PRIMARY KEY (supplier, pn)
);
CREATE INDEX supplier_catalog_mpn
  ON supplier_catalog (supplier, mpn_key, "class", stock DESC);
CREATE TABLE manufacturer_aliases (
  alias_key TEXT PRIMARY KEY,
  alias TEXT NOT NULL,
  canonical TEXT NOT NULL
);
