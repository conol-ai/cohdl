#!/usr/bin/env python3
"""Group the mapped chip resistors and MLCCs for the W3 sampled-inference proposal.

``python3 -I tools/w3_groups.py -o DIR [--check]`` writes DIR/groups.json and
DIR/groups.md. A committed report is a dated snapshot whose ``sources`` record
what it was built from. Nothing is fetched, and no binding or frame row is written:
this is the grouping report the proposal
(``docs/library-supplier-w3-sampling.md``) asks for, not evidence.

Scope: the ``ChipResistor`` and ``MLCC`` parts of ``lib/passive`` whose
(manufacturer, MPN) ``lcsc_map.json`` maps. Every AVL entry is one member.

A group is (footprint package, its version, footprint, part type, manufacturer,
series), where the series is the MPN's leading letters (Yageo ``RC``/``CC``,
Vishay ``CRCW``, Samsung ``CL``). Different series never share a group, and
the page's package text (``0402``) is not part of the key.

For each group the report gives its members, the members' frame rows with
their own evidence (``jlc_frames.json``), the sample the rule asks for (at least
MIN_SAMPLE or FRACTION of the members, whichever is larger), how many more
direct checks that needs, and a deterministic choice of them among members with
at least MIN_STOCK on their confirmed page. Members with no stock can belong to
a group but are never chosen as samples.

The rule is ``sourcing/w3_rule.py``. A member counts as a sample only when a
preview record judges it: a ``jlc_previews/*/w3.json`` names the record's BOM,
checks and measurements (with the body offset as a vector), and the land files
the preview was made with. Its readings hold only while those files still hash
the same. A judged member passes when every copy's body is drawn and within the
rule's sample limits: its angle, its raw offset, and its offset once its view's
median is taken off (board_bias).

The rule names the method a chip body is read by (``body_method``,
``frame_boards/measure_preview.py``). A record's ``w3.json`` names its method 1
readings, the only kind there was when the first records were made; readings by
a later method are a new dated file beside them, named by the record's
``w3-readings.json``. A record with no readings by the rule's method is not
judged, and a reading that names another method stops the build. Under a method
other than 1 a drawn chip body's status is the reading's own ``body_ok``, not the
run's ``checks.json``, which was made with method 1.

A record that is not judged is never dropped without a word. The report lists it
with the reason (``records_not_judged``: no readings by the rule's method, a land
file changed since its preview, or another package), and each group lists its
members placed on it (``record_not_judged``). Such a member is neither a passed
sample nor a failure, like a direct row the rule cannot judge
(``direct_not_judged``); the build does not stop, because a record lapsing is how
a land change and a new reading method are meant to take effect. ``w3_record.py``
does stop: it writes no group record while a member of the group sits on a
record that only lacks readings by the rule's method.

A member whose model a preview did not draw (and that no copy fails otherwise)
is ``not_drawn``: it is not a passed sample and is never inferred, and by itself
it does not pause its group. A group is ``paused`` when any member fails
otherwise (left for manual review, outside the limits, or unverified for another
reason), when a direct reading is not (0, 0, 0), or when its part numbers not
drawn are more than the rule's share of its judged members.

A view is one side of one preview record; its median is taken over every drawn
chip body on it. A reading within the limits counts only when enough of those
bodies belong to other groups (``w3_rule.bias_supported``); otherwise its member
is ``bias_unsupported``: neither a passed sample nor a failure. The report lists
what each view's median is made of.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import re
import sys

# `python3 -I` does not put the script's directory on sys.path.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from lcsc_backfill import _balanced_block, _string_fields, _strip_comments
from sourcing.bindings import package_metadata
from sourcing import w3_rule

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "tools/sourcing_data"
PASSIVE = ROOT / "lib/passive"
PREVIEWS = DATA / "jlc_previews"
BOARDS = ROOT / "tools/frame_boards"
TYPES = tuple(w3_rule.RULE["types"])
MIN_SAMPLE = w3_rule.RULE["min_sample"]
FRACTION = w3_rule.RULE["fraction"]
# The assembler builds at least five boards and adds attrition. On 2026-11-07 the
# W3 pilot's BOM step left unconfirmed, with a shortage warning, exactly the 11
# samples whose confirmed page read fewer than 20 pieces (5 to 19), and confirmed
# every one with 21 or more. 30 leaves room for stock sold since the page was read.
MIN_STOCK = w3_rule.RULE["min_stock"]
# The detail ``frame_boards/jlc_preview.py`` gives a part whose model is not drawn.
NOT_DRAWN = "no model drawn"


def preview_inputs() -> dict[Path, dict]:
    """Every preview record's ``w3.json``, by record directory."""
    return {path.parent: json.loads(path.read_text(encoding="utf-8"))
            for path in sorted(PREVIEWS.glob("*/w3.json"))}


def board_inputs() -> dict[Path, dict]:
    """Each frame board's own ``w3.json``, by path: the samples a run of that board
    found short at the BOM step before any preview of it was recorded. Such a run
    took no placement preview, so it has no record under ``jlc_previews`` and gives
    no reading; the board's directory keeps the dated fact until a record does."""
    return {path: json.loads(path.read_text(encoding="utf-8"))
            for path in sorted(BOARDS.glob("*/w3.json"))}


READINGS = "w3-readings.json"


def readings_file(record: Path, meta: dict, method: int | None = None) -> list[str]:
    """The files of RECORD that give its chip body readings by METHOD (the rule's by
    default), as names within the record; empty when it has none. Method 1 is
    ``w3.json``'s own ``measurements``; a later one is listed in READINGS."""
    method = w3_rule.RULE["body_method"] if method is None else method
    if method == 1:
        return [meta["measurements"]]
    index = record / READINGS
    if not index.is_file():
        return []
    name = json.loads(index.read_text(encoding="utf-8"))["readings"].get(str(method))
    return [READINGS, name] if name else []


def short_at_preview() -> dict[str, str]:
    """Samples a preview's BOM step reported short although their page read MIN_STOCK
    or more: the stock sold after the page was read. Each record keeps its own list,
    and so does a board whose run stopped there with no record (``board_inputs``).
    Each is replaced by the nearest eligible member, so the other samples, which that
    preview confirmed in stock, stay as they are."""
    short = {}
    for meta in (*preview_inputs().values(), *board_inputs().values()):
        if set(short) & set(meta["short_at_preview"]):
            raise SystemExit("w3_groups.py: a sample is listed short at preview twice")
        short.update(meta["short_at_preview"])
    return short


SHORT_AT_PREVIEW = short_at_preview()


def not_judged_reason(record: Path, meta: dict) -> tuple[str, str] | None:
    """Why RECORD is not judged under the rule, as (kind, sentence), or None when it
    is. ``package``: it previewed another package. ``land``: a land file no longer
    hashes as it did at the preview, so its readings have lapsed. ``method``: it
    has no readings by the rule's method yet."""
    if meta["package"] != w3_rule.RULE["package"]:
        return "package", f"its package is {meta['package']}, not {w3_rule.RULE['package']}"
    changed = sorted(path for path, digest in meta["land"].items() if _sha256(ROOT / path) != digest)
    if changed:
        return "land", "changed since its preview: " + ", ".join(changed)
    if not readings_file(record, meta):
        return "method", f"no readings by method {w3_rule.RULE['body_method']}"
    return None


def _record_name(record: Path) -> str:
    return str((record / "README.md").relative_to(ROOT))


def _refs(record: Path, meta: dict) -> dict[str, tuple[str, str]]:
    """{ref: (footprint, pn)} of RECORD's BOM."""
    with (record / meta["bom"]).open(encoding="utf-8-sig", newline="") as handle:
        rows = list(csv.DictReader(handle))
    return {ref.strip(): (row["Footprint"], row["LCSC Part #"])
            for row in rows for ref in row["Designator"].split(",")}


def records_not_judged() -> list[dict]:
    """Each preview record with W3 inputs that the rule does not judge:
    {record, kind, reason, parts: {(footprint, pn): [refs]}}."""
    found = []
    for record, meta in preview_inputs().items():
        why = not_judged_reason(record, meta)
        if why is None:
            continue
        parts: dict[tuple[str, str], list[str]] = {}
        for ref, part in sorted(_refs(record, meta).items()):
            parts.setdefault(part, []).append(ref)
        found.append({"record": _record_name(record), "kind": why[0], "reason": why[1], "parts": parts})
    return found


def _records():
    """Each judged preview record: (README path, {ref: (footprint, pn)}, checks, readings).
    Only records that have readings by the rule's method, and whose land files still
    hash as recorded, count; the others are ``records_not_judged``."""
    method = w3_rule.RULE["body_method"]
    for record, meta in preview_inputs().items():
        if not_judged_reason(record, meta) is not None:
            continue
        files = readings_file(record, meta)
        refs = _refs(record, meta)
        checks = json.loads((record / meta["checks"]).read_text(encoding="utf-8"))
        readings = {m["ref"]: m for m in json.loads((record / files[-1]).read_text(encoding="utf-8"))}
        other = sorted(m["ref"] for m in readings.values() if "body" in m and m["body"].get("method", 1) != method)
        if other:
            raise SystemExit(f"w3_groups.py: {record.name}/{files[-1]} is listed as method {method} readings, "
                             f"but {', '.join(other[:5])} {'was' if len(other) == 1 else 'were'} read another way")
        yield _record_name(record), refs, checks, readings


def view_id(record: str, side: str) -> str:
    return f"{record}#{side}"


def view_bodies() -> dict[str, list[tuple[str, str]]]:
    """For each view, the (footprint, pn) of every drawn body its median is taken
    over, read by the rule's method (a body with no BOM line is ``("", "")``)."""
    views: dict[str, list[tuple[str, str]]] = {}
    for name, refs, _, readings in _records():
        for ref, m in sorted(readings.items()):
            if "offset_xy_mm" in m.get("body", {}):
                views.setdefault(view_id(name, m["side"]), []).append(refs.get(ref, ("", "")))
    return views


def judged_members() -> dict[tuple[str, str], list[dict]]:
    """Each (footprint, pn) a preview record judges, with one outcome per copy.

    Each outcome is {record, ref, view, status, not_drawn, reading, ok}: ``status``
    is the preview check's (the reading's own ``body_ok`` under a method other than
    1), ``not_drawn`` whether it found bare pads where the model should be, and
    ``reading`` the rule's (method; angle; raw offset, the view's bias and the
    residual; limit) for a drawn body. ``build`` adds to the reading what the
    view's median is made of, which depends on the member's group.
    """
    judged: dict[tuple[str, str], list[dict]] = {}
    method = w3_rule.RULE["body_method"]
    for name, refs, checks, readings in _records():
        bias = {}
        for side in {m["side"] for m in readings.values()}:
            offsets = [m["body"]["offset_xy_mm"] for m in readings.values()
                       if m["side"] == side and "offset_xy_mm" in m.get("body", {})]
            if offsets:
                bias[side] = w3_rule.board_bias([tuple(o) for o in offsets])
        for ref, (footprint, pn) in sorted(refs.items()):
            check, m = checks.get(ref), readings.get(ref)
            if check is None or m is None:
                continue
            status = check["status"]
            # The run's check of a chip body was made with method 1. Under another method
            # the reading's own body_ok says whether the body lies along its pads, centred.
            if method != 1 and status in ("body", "manual review") and "body" in m:
                status = "body" if m["body_ok"] else "manual review"
            outcome = {"record": name, "ref": ref, "view": view_id(name, m["side"]), "status": status,
                       "not_drawn": status == "unverified" and check["detail"].startswith(NOT_DRAWN),
                       "reading": None, "ok": False}
            if outcome["not_drawn"] != (m.get("drawn") is False):
                raise SystemExit(f"w3_groups.py: {name}: the check and the reading of {ref} "
                                 "disagree on whether its model is drawn")
            body = m.get("body", {})
            if status in ("body", "manual review") and "offset_xy_mm" in body:
                outcome["reading"] = w3_rule.sample_reading(
                    body["delta_mod_180"], body["offset_xy_mm"], body["offset_limit_mm"], bias[m["side"]],
                    body_method=body.get("method", 1))
                outcome["ok"] = status == "body" and w3_rule.sample_ok(outcome["reading"])
            judged.setdefault((footprint, pn), []).append(outcome)
    return judged


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def passive_entries(lib: Path = PASSIVE) -> list[dict]:
    """Every AVL entry of a chip resistor or MLCC part, with its effective footprint."""
    entries = []
    for path in sorted((lib / "src").glob("*.cohdl")):
        text = _strip_comments(path.read_text(encoding="utf-8"))
        for match in re.finditer(r"\bpub\s+part\s+(\w+)\s*:\s*(\w+)[^{]*\{", text):
            if match.group(2) not in TYPES:
                continue
            block, _ = _balanced_block(text, match.end() - 1)
            footprint = None
            for kind in re.finditer(r"\b(primary|alt)\s*\{", block):
                fields, _ = _balanced_block(block, kind.end() - 1)
                found = re.search(r"\bfootprint\s*:\s*(\w+)", fields)
                if kind.group(1) == "primary":
                    footprint = found.group(1) if found else None
                values = dict(_string_fields(fields))
                entries.append({"part": match.group(1), "type": match.group(2), "entry": kind.group(1),
                                "mfr": values["mfr"], "mpn": values["mpn"],
                                "footprint": found.group(1) if found else footprint})
    return entries


def series(mpn: str) -> str:
    found = re.match(r"[A-Z]+", mpn)
    if not found:
        raise ValueError(f"{mpn}: no series prefix")
    return found.group(0)


def required(members: int) -> int:
    return w3_rule.required(members)


def spread(items: list, count: int) -> list:
    """COUNT items evenly spaced through ITEMS, first and last included when possible."""
    if count >= len(items):
        return list(items)
    if count == 1:
        return [items[0]]
    return [items[round(i * (len(items) - 1) / (count - 1))] for i in range(count)]


def choose(items: list, count: int, short) -> list:
    """spread(ITEMS, COUNT), with each item in SHORT replaced by the nearest item, by
    position, that is neither chosen nor short (the lower one first on a tie)."""
    chosen = spread(items, count)
    taken = {id(item) for item in chosen}
    for n, item in enumerate(chosen):
        if not short(item):
            continue
        at = next(i for i, other in enumerate(items) if other is item)
        nearest = sorted(range(len(items)), key=lambda i: (abs(i - at), i))
        free = [i for i in nearest if id(items[i]) not in taken and not short(items[i])]
        if free:
            chosen[n] = items[free[0]]
            taken.add(id(chosen[n]))
    return [item for item in chosen if not short(item)]


def confirmed_pages(snapshot: str) -> tuple[Path, dict[str, dict]]:
    """The backfill run's confirmed pages whose SHA-256 the map rows cite."""
    for path in sorted((DATA / "lcsc_backfill").glob("*/confirmed.json")):
        if _sha256(path) == snapshot:
            return path, {page["c"]: page for page in json.loads(path.read_text(encoding="utf-8"))}
    raise SystemExit(f"w3_groups.py: no lcsc_backfill run has confirmed pages {snapshot}")


def build() -> dict:
    name, version = package_metadata(PASSIVE / "cohdl.toml")
    map_rows = json.loads((DATA / "lcsc_map.json").read_text(encoding="utf-8"))["rows"]
    by_identity = {(row["mfr"], row["mpn"]): row for row in map_rows}
    snapshots = {row["snapshot_sha256"] for row in map_rows}
    if len(snapshots) != 1:
        raise SystemExit("w3_groups.py: the map cites more than one confirmed snapshot")
    pages_path, pages = confirmed_pages(snapshots.pop())
    frames = json.loads((DATA / "jlc_frames.json").read_text(encoding="utf-8"))["rows"]
    direct = {(row["package"], row["package_version"], row["footprint"], row["pn"]): row
              for row in frames if row["supplier"] == "lcsc" and row["evidence"] is not None}
    judged = judged_members()
    views = view_bodies()
    unjudged = records_not_judged()

    groups: dict[tuple, dict] = {}
    unmapped = 0
    for entry in passive_entries():
        row = by_identity.get((entry["mfr"], entry["mpn"]))
        if row is None:
            unmapped += 1
            continue
        page = pages[row["pn"]]
        key = (name, version, entry["footprint"], entry["type"], entry["mfr"], series(entry["mpn"]))
        group = groups.setdefault(key, {"members": [], "direct": []})
        group["members"].append([row["pn"], entry["mpn"], entry["part"], entry["entry"],
                                 row["category"], page["stock"], page["package"]])
        frame = direct.get((name, version, entry["footprint"], row["pn"]))
        if frame is not None:
            group["direct"].append({"pn": row["pn"], "row": frame["id"], "mapping": frame["mapping"],
                                    "record": frame["evidence"]["record"]})

    report = []
    for key in sorted(groups):
        group = groups[key]
        members = sorted(group["members"], key=lambda m: m[1])
        anomalies = [d["pn"] for d in group["direct"] if any(group_value != 0 for group_value in d["mapping"].values())]
        own = {(key[2], m[0]) for m in members}
        # Copied, so that what is added to a reading here stays this group's.
        outcomes = {m[0]: [dict(c, reading=c["reading"] and dict(c["reading"])) for c in judged[(key[2], m[0])]]
                    for m in members if (key[2], m[0]) in judged}
        composition = {view: (len(bodies), sum(body not in own for body in bodies))
                       for view, bodies in views.items()}
        for copies in outcomes.values():
            for c in copies:
                if c["reading"] is not None:
                    bodies, other = composition[c["view"]]
                    c["reading"].update(bias_view=c["view"], bias_bodies=bodies, bias_other_bodies=other)
                c["supported"] = c["reading"] is not None and w3_rule.bias_supported(c["reading"])
        within = sorted(pn for pn, copies in outcomes.items() if all(c["ok"] for c in copies))
        passed = [pn for pn in within if all(c["supported"] for c in outcomes[pn])]
        # Within the limits, but against a median that is the group's own: not a
        # passed sample and not a failure.
        unsupported = [pn for pn in within if pn not in passed]
        # Not drawn on some copy, and no copy failing otherwise: neither passed nor a failure.
        not_drawn = sorted(pn for pn, copies in outcomes.items()
                           if any(c["not_drawn"] for c in copies) and all(c["ok"] or c["not_drawn"] for c in copies))
        failed = sorted(set(outcomes) - set(within) - set(not_drawn))
        tried = set(outcomes) | {d["pn"] for d in group["direct"]}
        need = max(0, required(len(members)) - len(passed))
        eligible = [m for m in members if m[0] not in tried and m[5] >= MIN_STOCK]
        excess = w3_rule.not_drawn_excess(len(not_drawn), len(outcomes), len(members))
        paused = bool(anomalies or failed or excess)
        report.append({
            "package": key[0], "package_version": key[1], "footprint": key[2], "type": key[3],
            "mfr": key[4], "series": key[5],
            "members": len(members),
            "zero_stock": sum(m[5] <= 0 for m in members),
            "below_min_stock": sum(0 < m[5] < MIN_STOCK for m in members),
            "page_package_text": sorted({m[6] for m in members}),
            "direct": sorted(group["direct"], key=lambda d: d["pn"]),
            "required": required(len(members)),
            "more_needed": need,
            "status": "paused" if paused else ("sampled" if need == 0 else "needs_samples"),
            "anomalies": anomalies,
            "passed": [{"pn": pn, **outcomes[pn][0]["reading"],
                        "records": sorted({c["record"] for c in outcomes[pn]})} for pn in passed],
            "failed": [{"pn": pn, "outcomes": outcomes[pn]} for pn in failed],
            "not_drawn": [{"pn": pn, "outcomes": outcomes[pn]} for pn in not_drawn],
            "bias_unsupported": [{"pn": pn, "outcomes": outcomes[pn]} for pn in unsupported],
            "judged": len(outcomes),
            # Of max(judged, required): see w3_rule.not_drawn_excess.
            "not_drawn_share": round(len(not_drawn) / max(len(outcomes), required(len(members))), 4),
            "not_drawn_excess": excess,
            "views": {view: {"bodies": bodies, "other_bodies": other}
                      for view, (bodies, other) in sorted(composition.items())
                      if any(c["view"] == view for copies in outcomes.values() for c in copies)},
            "direct_not_judged": sorted({d["pn"] for d in group["direct"]} - set(outcomes)),
            # Members placed on a record the rule does not judge: not samples, not failures.
            "record_not_judged": [{"pn": m[0], "record": r["record"], "kind": r["kind"],
                                   "refs": r["parts"][(key[2], m[0])]}
                                  for r in unjudged for m in sorted(members, key=lambda m: m[0])
                                  if (key[2], m[0]) in r["parts"]],
            "sample": [m[0] for m in choose(eligible, need, lambda m: m[0] in SHORT_AT_PREVIEW)],
            "member_rows": [m[:6] for m in members],
        })
    member_group = {(g["footprint"], row[0]): " ".join((g["footprint"], g["type"], g["mfr"], g["series"]))
                    for g in report for row in g["member_rows"]}
    made_of = {}
    for view, bodies in sorted(views.items()):
        counts: dict[str, int] = {}
        for body in bodies:
            name_ = member_group.get(body, "no group")
            counts[name_] = counts.get(name_, 0) + 1
        made_of[view] = {"bodies": len(bodies), "groups": dict(sorted(counts.items()))}
    in_groups = {(g["footprint"], row[0]) for g in report for row in g["member_rows"]}
    return {
        "schema_version": 5,
        "rule": {**w3_rule.RULE, "short_at_preview": SHORT_AT_PREVIEW},
        "sources": {
            "lib/passive": version,
            "lcsc_map.json": _sha256(DATA / "lcsc_map.json"),
            "jlc_frames.json": _sha256(DATA / "jlc_frames.json"),
            str(pages_path.relative_to(DATA)): _sha256(pages_path),
            **{str((record / name).relative_to(DATA)): _sha256(record / name)
               for record, meta in preview_inputs().items()
               for name in ("w3.json", meta["bom"], meta["checks"], *readings_file(record, meta))},
            **{str(path.relative_to(ROOT / "tools")): _sha256(path) for path in board_inputs()},
        },
        "unmapped_entries": unmapped,
        "member_columns": ["pn", "mpn", "part", "entry", "category", "stock"],
        # Per view (preview record and side): the drawn bodies its median is taken over, by group.
        "views": made_of,
        # Preview records with W3 inputs that the rule does not judge, each with why and how
        # many of its part numbers are group members (listed per group, ``record_not_judged``).
        "records_not_judged": [{"record": r["record"], "kind": r["kind"], "reason": r["reason"],
                                "part_numbers": len(r["parts"]),
                                "members": sum(part in in_groups for part in r["parts"])} for r in unjudged],
        "groups": report,
    }


def render(report: dict) -> str:
    if report["schema_version"] == 1:
        return _render_v1(report)
    # Schema 3 is the 2026-11-10 report (rule w3-2026-11-10b), rendered as it was; schema 4
    # the reports of 2026-11-11 to 2026-11-12b, which did not list the records not judged.
    v4 = report["schema_version"] >= 4
    v5 = report["schema_version"] >= 5
    groups = report["groups"]
    rule = report["rule"]
    total = sum(g["members"] for g in groups)
    lines = [
        "# W3 sampling groups: chip resistors and MLCCs",
        "",
        "Generated by `tools/w3_groups.py`; do not edit. This is a grouping report, not evidence: "
        "a binding inferred from a group cites a group record, never this report.",
        "",
        f"- `lib/passive` {report['sources']['lib/passive']}; {total:,} mapped entries in {len(groups)} groups "
        f"({report['unmapped_entries']:,} entries are not mapped).",
        f"- Rule `{rule['version']}` (`tools/sourcing/w3_rule.py`): at least {rule['min_sample']} or "
        f"{rule['fraction']:.0%} of a group's members, whichever is larger, judged on a preview. A sample "
        f"passes when its body is drawn, lies along its pads within {rule['angle_limit_deg']:g} deg mod 180, "
        f"is centred within {rule['offset_raw_fraction']:g} of its per-part limit as measured (raw), and "
        f"within {rule['offset_residual_fraction']:g} of it once its view's median offset is taken off "
        f"(residual). Samples are chosen only among members with at least {rule['min_stock']} in "
        f"stock. Members with no stock are {'inferred' if rule['infer_zero_stock'] else 'not inferred'}.",
        *([f"- Chip bodies are read by `measure_preview.py` method {rule['body_method']}"
           + (": the drawn model alone, without the silkscreen beside it." if rule["body_method"] == 2 else ".")
           + " Readings made by another method do not count, and each view's median is over readings of "
           "this method."] if "body_method" in rule else []),
        f"- A sample whose model is not drawn is not a passed sample and does not {'by itself ' if v4 else ''}"
        f"pause its group; that part "
        f"number is {'inferred' if rule['infer_not_drawn'] else 'not inferred'}. A sample left for manual "
        "review, outside the limits, or unverified for another reason pauses its group.",
        *([f"- A reading counts only when the median taken off it is not its group's own: at least "
           f"{rule['bias_min_other_bodies']} of the view's drawn bodies, and more than "
           f"{rule['bias_other_share_above']:.0%} of them, must belong to other groups. Otherwise the sample "
           "is unsupported: not a passed sample, and not a failure.",
           f"- A group is paused when its part numbers not drawn are more than "
           f"{rule['not_drawn_max_share']:.0%} of its judged members (of its required sample, while fewer "
           "have been judged)."] if v4 else []),
        f"- Groups: {sum(g['status'] == 'sampled' for g in groups)} sampled, "
        f"{sum(g['status'] == 'needs_samples' for g in groups)} need samples, "
        f"{sum(g['status'] == 'paused' for g in groups)} paused.",
        f"- Samples passed: {sum(len(g['passed']) for g in groups)}. Still needed outside paused groups: "
        f"{sum(g['more_needed'] for g in groups if g['status'] != 'paused')}. "
        f"Chosen: {sum(len(g['sample']) for g in groups if g['status'] != 'paused')}.",
        f"- Members with no stock: {sum(g['zero_stock'] for g in groups):,}.",
        "- A group whose members with stock cannot make up its sample (Chosen below More needed) "
        "cannot be inferred: its members need their own evidence.",
        "- Direct rows not judged: frame rows with their own evidence on a preview record that has no "
        "`w3.json`, so the rule cannot judge them. They do not count as samples.",
        *([f"- Records not judged: {len(report['records_not_judged'])} of "
           f"{sum(name.endswith('/w3.json') and name.startswith('jlc_previews/') for name in report['sources'])} "
           "preview records with a `w3.json`. A record is not judged when it has no readings by the rule's "
           "method, when a land file has changed since its preview, or when it previewed another package. "
           "Its members are not samples and not failures; they are listed below."] if v5 else []),
        "",
        "| Footprint | Type | Manufacturer | Series | Members | No stock | Passed | Failed | Not drawn | "
        + ("Unsupported | " if v4 else "") + "Required | More needed | Chosen | Status |",
        "|---|---|---|---|---:|---:|---:|---:|---:|" + ("---:|" if v4 else "") + "---:|---:|---:|---|",
    ]
    for g in groups:
        lines.append(f"| `{g['footprint']}` | {g['type']} | {g['mfr']} | {g['series']} | {g['members']:,} | "
                     f"{g['zero_stock']:,} | {len(g['passed'])} | {len(g['failed'])} | {len(g['not_drawn'])} | "
                     + (f"{len(g['bias_unsupported'])} | " if v4 else "")
                     + f"{g['required']} | {g['more_needed']} | {len(g['sample'])} | {g['status']} |")
    lines += ["", "## Why groups are paused", ""]
    for g in groups:
        if g["status"] != "paused":
            continue
        reasons = [f"direct reading off zero: {', '.join(g['anomalies'])}"] if g["anomalies"] else []
        for f in g["failed"]:
            why = "; ".join(f"{c['ref']} {c['status']}" + (f" ({_reading(c['reading'], rule)})" if c["reading"] else "")
                            for c in f["outcomes"])
            reasons.append(f"{f['pn']}: {why}")
        if v4 and g["not_drawn_excess"]:
            of = max(g["judged"], g["required"])
            reasons.append(f"not drawn: {len(g['not_drawn'])} of {of} "
                           f"{'judged' if of == g['judged'] else 'required'} ({g['not_drawn_share']:.1%}), "
                           f"above {rule['not_drawn_max_share']:.0%}")
        lines.append(f"- `{g['footprint']}` {g['type']} {g['mfr']} {g['series']}: " + " | ".join(reasons))
    lines += ["", "## Not drawn", "",
              "Not passed samples and not inferred; they do not pause their group by themselves.", ""]
    for g in groups:
        if g["not_drawn"]:
            lines.append(f"- `{g['footprint']}` {g['type']} {g['mfr']} {g['series']} ({g['status']}): "
                         + ", ".join(f"{n['pn']} ({', '.join(c['ref'] for c in n['outcomes'])})"
                                     for n in g["not_drawn"]))
    if v4:
        lines += ["", "## What each view's median is made of", "",
                  "A view is one side of one preview record. For each group with bodies in it: the group's "
                  "own drawn bodies, and the bodies of other groups, which must number at least "
                  f"{rule['bias_min_other_bodies']} and be more than {rule['bias_other_share_above']:.0%} "
                  "of the view for the group's readings on it to count.", ""]
        for view, made in report["views"].items():
            lines += [f"- `{view}`: {made['bodies']} drawn bodies"]
            for name, count in made["groups"].items():
                other = made["bodies"] - count
                if name == "no group":
                    lines.append(f"  - in no group: {count}")
                    continue
                supported = (other >= rule["bias_min_other_bodies"]
                             and other > rule["bias_other_share_above"] * made["bodies"])
                lines.append(f"  - {name}: {count} own, {other} of other groups ({other / made['bodies']:.1%}): "
                             f"{'counts' if supported else 'does not count'}")
        lines += ["", "## Samples whose median is their group's own", "",
                  "Within the limits, but not passed samples: their view does not hold enough bodies of "
                  "other groups.", ""]
        for g in groups:
            if g["bias_unsupported"]:
                lines.append(f"- `{g['footprint']}` {g['type']} {g['mfr']} {g['series']} ({g['status']}): "
                             + ", ".join(f"{n['pn']} ({', '.join(c['ref'] for c in n['outcomes'])})"
                                         for n in g["bias_unsupported"]))
        if not any(g["bias_unsupported"] for g in groups):
            lines.append("- None.")
    if v5:
        lines += ["", "## Records not judged", "",
                  "Preview records the rule reads nothing from, and the group members placed on them. "
                  "None of these members is a passed sample or a failure on that record.", ""]
        for r in report["records_not_judged"]:
            lines.append(f"- `{r['record']}`: {r['reason']}; {r['members']} of its {r['part_numbers']} "
                         "part numbers are group members")
            for g in groups:
                mine = [n for n in g["record_not_judged"] if n["record"] == r["record"]]
                if mine:
                    lines.append(f"  - `{g['footprint']}` {g['type']} {g['mfr']} {g['series']} ({g['status']}): "
                                 + ", ".join(f"{n['pn']} ({', '.join(n['refs'])})" for n in mine))
        if not report["records_not_judged"]:
            lines.append("- None: every record with a `w3.json` is judged.")
    lines += ["", "## Offsets of passed samples", "",
              "Per group: the bias taken off in each view, and the largest raw offset and residual among its "
              "passed samples, each as a share of that sample's per-part limit.", ""]
    for g in groups:
        if not g["passed"]:
            continue
        biases = sorted({tuple(p["bias_xy_mm"]) for p in g["passed"]})
        raw = max(g["passed"], key=lambda p: (p["raw_offset_mm"] / p["offset_limit_mm"], p["pn"]))
        residual = max(g["passed"], key=lambda p: (p["residual_offset_mm"] / p["offset_limit_mm"], p["pn"]))
        lines.append(
            f"- `{g['footprint']}` {g['type']} {g['mfr']} {g['series']}: {len(g['passed'])} passed; bias "
            + ", ".join(f"({x:+.4f}, {y:+.4f}) mm" for x, y in biases)
            + f"; largest raw {raw['pn']} {raw['raw_offset_mm']:.3f} of {raw['offset_limit_mm']:.3f} mm "
            f"({raw['raw_offset_mm'] / raw['offset_limit_mm']:.0%}); largest residual {residual['pn']} "
            f"{residual['residual_offset_mm']:.3f} of {residual['offset_limit_mm']:.3f} mm "
            f"({residual['residual_offset_mm'] / residual['offset_limit_mm']:.0%})")
    lines += ["", "## Samples chosen", ""]
    for g in groups:
        if g["sample"] and g["status"] != "paused":
            lines.append(f"- `{g['footprint']}` {g['type']} {g['mfr']} {g['series']}: {', '.join(g['sample'])}")
    return "\n".join(lines) + "\n"


def _reading(reading: dict, rule: dict) -> str:
    limit = reading["offset_limit_mm"]
    return (f"{reading['angle_deg']:+.1f} deg; raw {reading['raw_offset_mm']:.3f} of "
            f"{rule['offset_raw_fraction'] * limit:.4f} mm; residual {reading['residual_offset_mm']:.3f} of "
            f"{rule['offset_residual_fraction'] * limit:.4f} mm")


def _render_v1(report: dict) -> str:
    """The 2026-11-07 report, as it was rendered."""
    groups = report["groups"]
    rule = report["rule"]
    total = sum(g["members"] for g in groups)
    lines = [
        "# W3 sampling groups: chip resistors and MLCCs",
        "",
        "Generated by `tools/w3_groups.py`; do not edit. This is a grouping report for the "
        "sampled-inference proposal, not evidence. No binding depends on it.",
        "",
        f"- `lib/passive` {report['sources']['lib/passive']}; {total:,} mapped entries in {len(groups)} groups "
        f"({report['unmapped_entries']:,} entries are not mapped).",
        f"- Draft rule under review: at least {rule['min_sample']} or {rule['fraction']:.0%} of a group's "
        f"members, whichever is larger, checked directly. Samples are chosen only among members "
        f"with at least {rule['min_stock']} in stock.",
        f"- Groups: {sum(g['status'] == 'sampled' for g in groups)} already sampled, "
        f"{sum(g['status'] == 'needs_samples' for g in groups)} need samples, "
        f"{sum(g['status'] == 'paused' for g in groups)} paused.",
        f"- Direct checks still needed: {sum(g['more_needed'] for g in groups)}. "
        f"Chosen: {sum(len(g['sample']) for g in groups)}.",
        f"- Members with no stock: {sum(g['zero_stock'] for g in groups):,}. They can be inferred only "
        "once the rule is accepted, and are never recorded as previewed.",
        "- A group whose members with stock cannot make up its sample (Chosen below More needed) "
        "cannot be inferred: its members need their own evidence.",
        "",
        "| Footprint | Type | Manufacturer | Series | Members | No stock | Direct | Required | More needed | Chosen | Status |",
        "|---|---|---|---|---:|---:|---:|---:|---:|---:|---|",
    ]
    for g in groups:
        lines.append(f"| `{g['footprint']}` | {g['type']} | {g['mfr']} | {g['series']} | {g['members']:,} | "
                     f"{g['zero_stock']:,} | {len(g['direct'])} | {g['required']} | {g['more_needed']} | "
                     f"{len(g['sample'])} | {g['status']} |")
    lines += ["", "## Samples chosen", ""]
    for g in groups:
        if g["sample"]:
            lines.append(f"- `{g['footprint']}` {g['type']} {g['mfr']} {g['series']}: {', '.join(g['sample'])}")
    return "\n".join(lines) + "\n"


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("-o", "--out", type=Path, required=True)
    parser.add_argument("--check", action="store_true", help="fail if DIR differs from a fresh build")
    args = parser.parse_args(argv)
    report = build()
    files = {"groups.json": json.dumps(report, indent=1, ensure_ascii=False) + "\n", "groups.md": render(report)}
    if args.check:
        stale = [name for name, text in files.items()
                 if not (args.out / name).is_file() or (args.out / name).read_text(encoding="utf-8") != text]
        for name in stale:
            print(f"w3_groups.py: {args.out / name} is not what the repository builds", file=sys.stderr)
        return 1 if stale else 0
    args.out.mkdir(parents=True, exist_ok=True)
    for name, text in files.items():
        (args.out / name).write_text(text, encoding="utf-8")
    print(render(report).split("\n| Footprint")[0])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
