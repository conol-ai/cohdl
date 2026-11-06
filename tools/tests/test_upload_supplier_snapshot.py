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


def page(c, mpn="M", mfr="Acme", package="0603", category="basic", fetched_at="2026-10-30T04:00:00Z", stock=5):
    return {"c": c, "mpn": mpn, "mfr": mfr, "package": package, "category": category, "stock": stock,
            "partdetail": f"/partdetail/x/{c}/{c}", "fetched_at": fetched_at}


class UploadSupplierSnapshotTests(unittest.TestCase):
    def test_the_recorded_catalog_rebuilds_from_the_repository_alone(self):
        report = json.loads((RECORD / "report.json").read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory() as td, mock.patch.object(sys, "stdout", io.StringIO()):
            rebuilt = us.build(Path(td), None, RECORD / "report.json")
            # Bindings added since the upload grow the library's list; the
            # catalog itself must not change, and must still cover them all.
            current = rebuilt.pop("library_part_numbers")
            recorded = report.pop("library_part_numbers")
            self.assertEqual(rebuilt, report)
            self.assertLessEqual(set(recorded), set(current))
            body = json.loads((Path(td) / "snapshot.json").read_bytes())
            self.assertEqual(hashlib.sha256((Path(td) / "snapshot.json").read_bytes()).hexdigest(),
                             report["snapshot_sha256"])
        self.assertLessEqual(report["rows"], us.MAX_ROWS)
        self.assertLessEqual(report["bytes"], us.BODY_LIMIT)
        self.assertEqual(sorted(body), ["classes", "display_name", "parts", "product_url_template"])
        pns = {row["pn"] for row in body["parts"]}
        for pn in ("C117815", "C129276", "C1857444", "C529436", "C717701", "C730103"):
            self.assertIn(pn, pns)
        self.assertEqual(current, us.library_pns())
        self.assertLessEqual(set(current), pns)

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
            us.build(out, None, RECORD / "report.json")
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
            us.build(out, None, RECORD / "report.json")
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
