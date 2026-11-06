import hashlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import upload_supplier_snapshot as us

RECORD = us.DATA / "registry_uploads/2026-11-05-lcsc"
# The catalog the repository builds now, for the next upload.
CURRENT = {"rows": 13_604, "bytes": 3_583_901,
           "snapshot_sha256": "3307b6b8d811445eb6070a56c04534b9dc4eea5fdcb4d536e136d95dded84529",
           "aliases_sha256": "027d2a930f6016ad6a223fbbcc6fb4efce7e8c1458145a7dbb991edd014374e9"}


def rebuild_recorded(out: Path) -> dict:
    """Rebuild the 2026-11-05 upload from the map and alias table it was built from."""
    recorded = json.loads((RECORD / "report.json").read_text(encoding="utf-8"))["library_part_numbers"]
    return us.build(out, None, RECORD / "report.json", map_path=RECORD / "lcsc_map.json",
                    aliases_path=RECORD / "mfr_aliases.json", library=recorded)


def page(c, mpn="M", mfr="Acme", package="0603", category="basic", fetched_at="2026-10-30T04:00:00Z", stock=5):
    return {"c": c, "mpn": mpn, "mfr": mfr, "package": package, "category": category, "stock": stock,
            "partdetail": f"/partdetail/x/{c}/{c}", "fetched_at": fetched_at}


class UploadSupplierSnapshotTests(unittest.TestCase):
    def test_the_recorded_catalog_rebuilds_from_the_repository_alone(self):
        report = json.loads((RECORD / "report.json").read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory() as td, mock.patch.object(sys, "stdout", io.StringIO()):
            rebuilt = rebuild_recorded(Path(td))
            # The limits are the registry's at build time, not part of the catalog.
            self.assertEqual(report.pop("limits"), {"rows": 10_000, "bytes": us.BODY_LIMIT})
            self.assertEqual(rebuilt.pop("limits"), {"rows": us.MAX_ROWS, "bytes": us.BODY_LIMIT})
            self.assertEqual(rebuilt, report)
            body = json.loads((Path(td) / "snapshot.json").read_bytes())
            self.assertEqual(hashlib.sha256((Path(td) / "snapshot.json").read_bytes()).hexdigest(),
                             report["snapshot_sha256"])
        self.assertLessEqual(report["rows"], 10_000)
        self.assertLessEqual(report["bytes"], us.BODY_LIMIT)
        self.assertEqual(sorted(body), ["classes", "display_name", "parts", "product_url_template"])
        pns = {row["pn"] for row in body["parts"]}
        for pn in ("C117815", "C129276", "C1857444", "C529436", "C717701", "C730103"):
            self.assertIn(pn, pns)
        # The library has since bound part numbers this catalog lacks (C165948 in usb 0.1.6),
        # so the record rebuilds against its own list, and the current library refuses it.
        self.assertTrue(set(us.library_pns()) - pns)
        with tempfile.TemporaryDirectory() as td, mock.patch.object(sys, "stdout", io.StringIO()):
            with self.assertRaisesRegex(us.SnapshotError, "library part numbers missing from the catalog"):
                us.build(Path(td), None, RECORD / "report.json", map_path=RECORD / "lcsc_map.json",
                         aliases_path=RECORD / "mfr_aliases.json")

    def synthetic(self, root: Path, mpns: list[str]) -> dict:
        """Build a catalog of one B1 page per MPN and nothing else."""
        snapshot, runs, lib = root / "snapshot", root / "runs", root / "lib"
        for d in (snapshot, runs, lib):
            d.mkdir()
        pages = [{"c": f"C{i + 1}", "category": "extended", "mfr": "Acme", "mpn": mpn, "package": "0603",
                  "partdetail": f"/partdetail/x/C{i + 1}/C{i + 1}", "stock": 0} for i, mpn in enumerate(mpns)]
        (snapshot / "catalog.json").write_text(json.dumps(pages), encoding="utf-8")
        (snapshot / "requests.json").write_text(json.dumps(
            [{"snapshot_date": "2026-10-30T04:00:00Z", "url": us.ORIGIN + p["partdetail"]} for p in pages]),
            encoding="utf-8")
        (snapshot / "provenance.json").write_text(json.dumps({"sha256": {"catalog.json": hashlib.sha256(
            (snapshot / "catalog.json").read_bytes()).hexdigest()}}), encoding="utf-8")
        (root / "map.json").write_text('{"rows": []}', encoding="utf-8")
        with mock.patch.object(sys, "stdout", io.StringIO()):
            return us.build(root / "out", None, snapshot=snapshot, map_path=root / "map.json", runs=runs, lib=lib)

    def test_the_build_holds_the_registrys_row_and_byte_limits(self):
        with tempfile.TemporaryDirectory() as td:
            self.assertEqual(self.synthetic(Path(td), ["M"] * us.MAX_ROWS)["rows"], us.MAX_ROWS)
        with tempfile.TemporaryDirectory() as td, self.assertRaisesRegex(us.SnapshotError, "15001 rows"):
            self.synthetic(Path(td), ["M"] * (us.MAX_ROWS + 1))
        # Fewer rows, longer MPNs: the body alone sets the limit, to the byte.
        with tempfile.TemporaryDirectory() as td:
            base = self.synthetic(Path(td), ["M"] * 10_000)["bytes"]
        extra = us.BODY_LIMIT - base
        mpns = ["M" * (1 + extra // 10_000 + (i < extra % 10_000)) for i in range(10_000)]
        with tempfile.TemporaryDirectory() as td:
            self.assertEqual(self.synthetic(Path(td), mpns)["bytes"], us.BODY_LIMIT)
        mpns[0] += "M"
        with tempfile.TemporaryDirectory() as td, self.assertRaisesRegex(
                us.SnapshotError, f"10000 rows, {us.BODY_LIMIT + 1} bytes"):
            self.synthetic(Path(td), mpns)

    def test_the_current_catalog_builds_from_the_repository_alone(self):
        with tempfile.TemporaryDirectory() as td, mock.patch.object(sys, "stdout", io.StringIO()):
            report = us.build(Path(td), None, RECORD / "report.json")
            pns = {row["pn"] for row in json.loads((Path(td) / "snapshot.json").read_bytes())["parts"]}
        self.assertEqual({k: report[k] for k in CURRENT}, CURRENT)
        self.assertEqual(report["sources"], {"jlc_catalog/2026-10-30": 1334, "lcsc_map.json": 12_284, "overlap": 14})
        self.assertEqual(report["source_sha256"]["lcsc_map.json"],
                         hashlib.sha256(us.MAP.read_bytes()).hexdigest())
        self.assertEqual(report["aliases"], json.loads(us.ALIASES.read_text(encoding="utf-8"))["aliases"])
        self.assertEqual(report["library_part_numbers"], us.library_pns())
        self.assertLessEqual(set(report["library_part_numbers"]), pns)
        # Every part number the map names is in the catalog.
        self.assertLessEqual({row["pn"] for row in json.loads(us.MAP.read_text())["rows"]}, pns)

    def test_the_limits_are_the_registrys(self):
        source = (us.ROOT / "registry/src/worker/suppliers.ts").read_text(encoding="utf-8")
        self.assertIn("export const MAX_CATALOG_ROWS = 15_000;", source)
        self.assertIn("export const SUPPLIER_BODY_LIMIT = 4 * 1024 * 1024;", source)
        self.assertEqual((us.MAX_ROWS, us.BODY_LIMIT), (15_000, 4 * 1024 * 1024))

    def test_a_page_without_a_fetch_time_fails_the_build(self):
        with tempfile.TemporaryDirectory() as td:
            with self.assertRaisesRegex(us.SnapshotError, "request log has no fetch of .*C1525"):
                us.build(Path(td), None)

    def test_one_part_number_is_one_row(self):
        older, newer = page("C2", stock=1), page("C2", stock=9, fetched_at="2026-10-30T05:00:00Z")
        rows = us.merge([page("C10"), older, newer])
        self.assertEqual([row["pn"] for row in rows], ["C2", "C10"])
        self.assertEqual(rows[0]["stock"], 9)
        self.assertEqual(rows[0]["links"], [{"label": "JLCPCB", "url": "https://jlcpcb.com/partdetail/x/C2/C2"}])
        for field, value in (("mpn", "N"), ("mfr", "Other"), ("package", "0805"), ("category", "extended")):
            with self.subTest(field=field), self.assertRaisesRegex(us.SnapshotError, "C2: two different records"):
                us.merge([page("C2"), page("C2", **{field: value})])

    def test_rows_the_registry_would_refuse_are_reported(self):
        good = us.catalog_row(page("C1"))
        self.assertEqual(us.check_rows([good]), [])
        cases = [(dict(good, mpn=" M"), "mpn"), (dict(good, manufacturer="A​B"), "manufacturer"),
                 (dict(good, package=""), "package"), (dict(good, **{"class": "gold"}), "class"),
                 (dict(good, stock=-1), "stock"), (dict(good, fetched_at="2026-10-30"), "fetched_at")]
        for row, field in cases:
            with self.subTest(field=field):
                self.assertEqual(us.check_rows([row]), [f"C1: invalid {field}"])

    def test_upload_sends_only_the_signed_off_body_with_the_session_unprinted(self):
        with tempfile.TemporaryDirectory() as td, mock.patch.object(sys, "stdout", io.StringIO()):
            out = Path(td)
            rebuild_recorded(out)
            digest = json.loads((out / "report.json").read_text())["snapshot_sha256"]
            session = "ab" * 32
            calls = []

            def request(method, url, body=None, session=None, origin=None):
                calls.append((method, url, body, session, origin))
                return 200, ({"rows": 1442} if url.endswith("/snapshot") else {"alias_key": "k"})
            with mock.patch.object(us, "request", side_effect=request):
                self.assertIn("is not the reported snapshot", us.upload(out, "0" * 64, "https://r.test", session)[0])
                self.assertIn("COHDL_ADMIN_SESSION", us.upload(out, digest, "https://r.test", "")[0])
                self.assertEqual(calls, [])
                self.assertEqual(us.upload(out, digest, "https://r.test/x", session), [])
            self.assertEqual([c[1] for c in calls], [
                "https://r.test/api/admin/manufacturer-aliases/GigaDevice%20Semicon%20Beijing",
                "https://r.test/api/admin/manufacturer-aliases/Vishay%20Intertech",
                "https://r.test/api/admin/manufacturer-aliases/YAGEO",
                "https://r.test/api/admin/suppliers/lcsc/snapshot"])
            self.assertEqual(calls[-1][2], (out / "snapshot.json").read_bytes())
            self.assertTrue(all(c[0] == "PUT" and c[3] == session and c[4] == "https://r.test" for c in calls))
            self.assertNotIn(session, sys.stdout.getvalue())

            (out / "snapshot.json").write_bytes(b"{}")
            self.assertIn("no longer matches", us.upload(out, digest, "https://r.test", session)[0])

    def test_verify_reports_unknown_and_differing_rows(self):
        with tempfile.TemporaryDirectory() as td, mock.patch.object(sys, "stdout", io.StringIO()):
            out = Path(td)
            rebuild_recorded(out)
            rows = {row["pn"]: row for row in json.loads((out / "snapshot.json").read_bytes())["parts"]}
            pns = json.loads((out / "report.json").read_text())["library_part_numbers"]
            parts = [{"pn": pn, "catalog": dict(rows[pn])} for pn in pns]
            with mock.patch.object(us, "request", return_value=(200, {"parts": parts, "snapshot_date": "x"})):
                self.assertEqual(us.verify(out, "https://r.test"), [])
            parts[0]["catalog"] = "unknown"
            parts[1]["catalog"]["mpn"] = "OTHER"
            with mock.patch.object(us, "request", return_value=(200, {"parts": parts})):
                self.assertEqual(us.verify(out, "https://r.test"), [
                    f"{pns[0]}: unknown to the registry", f"{pns[1]}: the registry's mpn differ from the snapshot"])
            with mock.patch.object(us, "request", return_value=(404, {"error": "supplier not found"})):
                self.assertIn("HTTP 404", us.verify(out, "https://r.test")[0])


if __name__ == "__main__":
    unittest.main()
