import hashlib
import io
import json
import sys
import tempfile
import unittest
import urllib.parse
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import upload_supplier_snapshot as us

RECORD = us.DATA / "registry_uploads/2026-11-05-lcsc"
# The catalog the repository builds now, for the next upload.
CURRENT = {"rows": 13_604, "bytes": 3_583_901,
           "snapshot_sha256": "3307b6b8d811445eb6070a56c04534b9dc4eea5fdcb4d536e136d95dded84529",
           "aliases_sha256": "027d2a930f6016ad6a223fbbcc6fb4efce7e8c1458145a7dbb991edd014374e9"}


# The library's part numbers on 2026-11-06, when the counterexample was found. Parts
# bound later (C165948 in usb 0.1.6) are not in the old catalog, so the live library
# no longer shows it.
LIBRARY_2026_11_06 = set("""
    C1525 C8734 C9861 C23922 C93943 C105575 C106254 C107011 C107045 C107108 C107110 C114763
    C117815 C129276 C137046 C137203 C137422 C137837 C137927 C138161 C141703 C163421 C163429
    C274047 C309589 C477801 C482239 C482876 C482966 C483001 C483188 C488876 C488985 C527206
    C527232 C529436 C541229 C576985 C717701 C718246 C730103 C877226 C878096 C1338995 C1857444
    C3009257""".split())


def rebuild_recorded(out: Path) -> dict:
    """Rebuild the 2026-11-05 upload from the map and alias table it was built from."""
    return us.build(out, None, RECORD / "report.json",
                    map_path=RECORD / "lcsc_map.json", aliases_path=RECORD / "mfr_aliases.json")


def page(c, mpn="M", mfr="Acme", package="0603", category="basic", fetched_at="2026-10-30T04:00:00Z", stock=5):
    return {"c": c, "mpn": mpn, "mfr": mfr, "package": package, "category": category, "stock": stock,
            "partdetail": f"/partdetail/x/{c}/{c}", "fetched_at": fetched_at}


class UploadSupplierSnapshotTests(unittest.TestCase):
    def test_the_recorded_catalog_rebuilds_from_the_repository_alone(self):
        report = json.loads((RECORD / "report.json").read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory() as td, mock.patch.object(sys, "stdout", io.StringIO()):
            rebuilt = rebuild_recorded(Path(td))
            # Bindings added since the upload grow the library's list; the
            # catalog itself must not change, and must still cover them all.
            current = rebuilt.pop("library_part_numbers")
            recorded = report.pop("library_part_numbers")
            # The limits are the registry's at build time, not part of the catalog.
            self.assertEqual(report.pop("limits"), {"rows": 10_000, "bytes": us.BODY_LIMIT})
            self.assertEqual(rebuilt.pop("limits"), {"rows": us.MAX_ROWS, "bytes": us.BODY_LIMIT})
            self.assertEqual(rebuilt, report)
            self.assertLessEqual(set(recorded), set(current))
            body = json.loads((Path(td) / "snapshot.json").read_bytes())
            self.assertEqual(hashlib.sha256((Path(td) / "snapshot.json").read_bytes()).hexdigest(),
                             report["snapshot_sha256"])
        self.assertLessEqual(report["rows"], 10_000)
        self.assertLessEqual(report["bytes"], us.BODY_LIMIT)
        self.assertEqual(sorted(body), ["classes", "display_name", "parts", "product_url_template"])
        pns = {row["pn"] for row in body["parts"]}
        for pn in ("C117815", "C129276", "C1857444", "C529436", "C717701", "C730103"):
            self.assertIn(pn, pns)
        self.assertEqual(current, us.library_pns())
        self.assertLessEqual(set(current), pns)

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
        self.assertEqual(report["sources"], {"jlc_catalog/2026-10-30": 1334, "lcsc_map.json": 12_321, "overlap": 51})
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
            session = us.Credential("session", "ab" * 32)
            calls = []

            def request(method, url, body=None, credential=None, origin=None):
                calls.append((method, url, body, credential, origin))
                return 200, ({"rows": 1442} if url.endswith("/snapshot") else {"alias_key": "k"})
            with mock.patch.object(us, "request", side_effect=request):
                self.assertIn("is not the reported snapshot", us.upload(out, "0" * 64, "https://r.test", session)[0])
                missing = us.admin_credential({}, Path(td) / "no-token")
                self.assertIn("no admin credential", us.upload(out, digest, "https://r.test", missing)[0])
                self.assertEqual(calls, [])
                self.assertEqual(us.upload(out, digest, "https://r.test/x", session), [])
            self.assertEqual([c[1] for c in calls], [
                "https://r.test/api/admin/manufacturer-aliases/GigaDevice%20Semicon%20Beijing",
                "https://r.test/api/admin/manufacturer-aliases/Vishay%20Intertech",
                "https://r.test/api/admin/manufacturer-aliases/YAGEO",
                "https://r.test/api/admin/suppliers/lcsc/snapshot"])
            self.assertEqual(calls[-1][2], (out / "snapshot.json").read_bytes())
            self.assertTrue(all(c[0] == "PUT" and c[3] == session and c[4] == "https://r.test" for c in calls))
            self.assertNotIn(session.value, sys.stdout.getvalue())

            (out / "snapshot.json").write_bytes(b"{}")
            self.assertIn("no longer matches", us.upload(out, digest, "https://r.test", session)[0])

    def test_admin_credential_prefers_an_admin_token_and_refuses_the_wrong_kind(self):
        token = "cohdl_admin_" + "0f" * 24
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "admin-token"
            # The environment's token wins over the file and the session.
            got = us.admin_credential({"COHDL_ADMIN_TOKEN": token, "COHDL_ADMIN_SESSION": "ab" * 32}, path)
            self.assertEqual(got, us.Credential("token", token))
            # The `cohdl login` publish token is refused by name, before any request.
            publish = us.admin_credential({"COHDL_ADMIN_TOKEN": "cohdl_" + "0f" * 24}, path)
            self.assertIn("publish token", publish)
            self.assertIn("not an admin token", us.admin_credential({"COHDL_ADMIN_TOKEN": "nope"}, path))
            # A token file readable by others is refused; an owner-only one is used.
            path.write_text(token + "\n")
            path.chmod(0o644)
            self.assertIn("readable by you alone", us.admin_credential({}, path))
            path.chmod(0o600)
            self.assertEqual(us.admin_credential({}, path), us.Credential("token", token))
            # With no token anywhere, the web session still works.
            self.assertEqual(us.admin_credential({"COHDL_ADMIN_SESSION": "ab" * 32}, Path(td) / "none"),
                             us.Credential("session", "ab" * 32))
            self.assertIn("64-hex", us.admin_credential({"COHDL_ADMIN_SESSION": "x"}, Path(td) / "none"))
            self.assertIn("no admin credential", us.admin_credential({}, Path(td) / "none"))
            # The secret never appears in a repr, so never in a traceback.
            self.assertNotIn(token, repr(us.Credential("token", token)))

    def test_request_sends_a_token_as_bearer_and_a_session_as_same_origin_cookie(self):
        sent = []

        class Response:
            status = 200
            def __enter__(self): return self
            def __exit__(self, *exc): return False
            def read(self): return b"{}"

        def urlopen(req, timeout):
            sent.append({k.lower(): v for k, v in req.header_items()})
            return Response()
        token = "cohdl_admin_" + "0f" * 24
        with mock.patch.object(us.urllib.request, "urlopen", side_effect=urlopen):
            us.request("PUT", "https://r.test/api/admin/x", b"{}", us.Credential("token", token), "https://r.test")
            us.request("PUT", "https://r.test/api/admin/x", b"{}", us.Credential("session", "ab" * 32), "https://r.test")
        self.assertEqual(sent[0]["authorization"], f"Bearer {token}")
        self.assertNotIn("cookie", sent[0])
        self.assertNotIn("origin", sent[0])
        self.assertEqual(sent[1]["cookie"], "__Host-session=" + "ab" * 32)
        self.assertEqual(sent[1]["origin"], "https://r.test")
        self.assertNotIn("authorization", sent[1])

    def test_verify_passes_only_when_the_registry_serves_the_whole_catalog(self):
        with tempfile.TemporaryDirectory() as td, mock.patch.object(sys, "stdout", io.StringIO()) as stdout:
            out = Path(td)
            rebuild_recorded(out)
            with mock.patch.object(us, "request", side_effect=FakeRegistry(out)) as request:
                self.assertEqual(us.verify(out, "https://r.test"), [])
            # 1,442 rows in batches of 100, then the three aliases in one request.
            self.assertEqual(request.call_count, 15 + 1)
            self.assertIn("C1857444: STM32H743VIH6TR (STMicroelectronics, extended) known", stdout.getvalue())
            self.assertIn("rows: 1442 of 1442 known and equal", stdout.getvalue())

    def test_verify_refuses_an_older_catalog_that_holds_every_library_part_number(self):
        """The 2026-11-06 counterexample: the 2026-11-05 catalog holds all 46 library
        part numbers, so a check of those alone passed although the new catalog was
        never uploaded."""
        with tempfile.TemporaryDirectory() as td, mock.patch.object(sys, "stdout", io.StringIO()):
            old, new = Path(td) / "old", Path(td) / "new"
            rebuild_recorded(old)
            us.build(new, None, RECORD / "report.json")
            registry = FakeRegistry(old)
            self.assertLessEqual(LIBRARY_2026_11_06, set(registry.rows))
            with mock.patch.object(us, "request", side_effect=registry):
                problems = us.verify(new, "https://r.test")
        self.assertTrue(problems[0].startswith("12162 part numbers unknown to the registry: "), problems[0])
        self.assertIn("the registry's snapshot date is 2026-10-30T05:24:02Z, "
                      "not the snapshot's newest fetched_at 2026-11-06T01:45:04Z", problems)
        # 34 aliases went up after the old catalog. Five of them (PANASONIC, SEMTECH,
        # U-BLOX, XI'AN Aerosemi Tech, XINLUDA) differ from their canonical name only in
        # case, so the public lookup answers the same with or without them.
        self.assertEqual(sum(p.startswith("alias ") for p in problems), 29)

    def test_verify_reports_bad_answers(self):
        with tempfile.TemporaryDirectory() as td, mock.patch.object(sys, "stdout", io.StringIO()):
            out = Path(td)
            rebuild_recorded(out)
            pns = sorted(json.loads((out / "report.json").read_text())["library_part_numbers"])

            def check(edit, **kw):
                registry = FakeRegistry(out, **kw)
                registry.edit = edit
                with mock.patch.object(us, "request", side_effect=registry):
                    return us.verify(out, "https://r.test")

            def unknown(parts):
                for part in parts:
                    if part["pn"] == pns[0]:
                        part["catalog"] = "unknown"
            self.assertEqual(check(unknown), [f"1 part number unknown to the registry: {pns[0]}"])

            def other_mpn(parts):
                for part in parts:
                    if part["pn"] == pns[1]:
                        part["catalog"]["mpn"] = "OTHER"
            self.assertEqual(check(other_mpn), [f"1 part number differ from the snapshot: {pns[1]} (mpn)"])

            def drop_and_repeat(parts):
                if any(part["pn"] == pns[2] for part in parts):
                    kept = [part for part in parts if part["pn"] != pns[2]]
                    parts[:] = kept + [dict(kept[0]), {"pn": "C0", "catalog": "unknown"}]
            problems = check(drop_and_repeat)
            self.assertIn(f"1 part number missing from the registry's answers: {pns[2]}", problems)
            self.assertTrue(any(p.startswith("1 part number answered more than once: ") for p in problems))
            self.assertIn("1 part number answered but not asked: C0", problems)

            self.assertEqual(check(None, aliases={"GigaDevice Semicon Beijing": "GigaDevice", "YAGEO": "Yageo"}),
                             ["alias Vishay Intertech: the registry normalizes it to 'vishay intertech', not 'vishay'"])
            self.assertIn("HTTP 404", check(None, status=404)[0])

            (out / "aliases.json").write_bytes(b"{}")
            self.assertIn("no longer matches", check(None)[0])


class FakeRegistry:
    """The public lookups of registry/src/worker/supplier-parts.ts and manufacturers.ts,
    serving the catalog and aliases of a build directory."""

    def __init__(self, out: Path, aliases=None, status=200):
        self.rows = {row["pn"]: row for row in json.loads((out / "snapshot.json").read_bytes())["parts"]}
        aliases = json.loads((out / "aliases.json").read_bytes()) if aliases is None else aliases
        self.aliases = {us.manufacturer_key(k): v for k, v in aliases.items()}
        self.status, self.edit = status, None

    def __call__(self, method, url, body=None, credential=None, origin=None):
        assert method == "GET" and body is None and credential is None
        if self.status != 200:
            return self.status, {"error": "supplier not found"}
        parsed = urllib.parse.urlsplit(url)
        query = urllib.parse.parse_qs(parsed.query)
        if parsed.path == "/manufacturers/normalize":
            assert len(query["mfr"]) <= 100
            return 200, {"mfr": [{"value": v, "key": us.manufacturer_key(self.aliases.get(us.manufacturer_key(v), v))}
                                 for v in query["mfr"]], "mpn": []}
        assert parsed.path == "/suppliers/lcsc/parts"
        pns = query["pn"][0].split(",")
        assert len(pns) <= 100
        parts = [{"pn": pn, "catalog": json.loads(json.dumps(self.rows[pn])) if pn in self.rows else "unknown"}
                 for pn in pns]
        if self.edit:
            self.edit(parts)
        return 200, {"supplier": "lcsc", "snapshot_date": max(r["fetched_at"] for r in self.rows.values()),
                     "parts": parts}

if __name__ == "__main__":
    unittest.main()
