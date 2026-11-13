#!/usr/bin/env python3
"""Write tools/passive_data/uniroyal_dry_run.json: what the UNI-ROYAL resistors in the B1 snapshot would become.

``python3 -I tools/uniroyal_alternates.py [--check]``. Nothing is fetched.

DRY RUN. gen_passive.py does not read the file this writes, no part gains an
alternate, and nothing is bound. It answers one question ahead of plan W4:
which UNI-ROYAL part numbers JLCPCB stocks as Basic or Preferred are the same
resistor as a part the library already has, and which would be new values.

The B1 snapshot (tools/sourcing_data/jlc_catalog/2026-10-30/) holds the JLCPCB
Basic and Preferred parts. Its UNI-ROYAL part numbers are decoded with
``gen_passive.uniroyal_decode``. That rule is transcribed from UNI-ROYAL's
datasheet (tools/passive_data/uniroyal_chip_series.evidence.md), except the
zero-ohm jumper's spelling, which the datasheet does not give and which is
still inferred from the snapshot (see the comment above the rule in
gen_passive.py).

Each row lands in one list:

- ``alternates``: decodes to the resistance, tolerance and case size of an
  emitted library part, whose rated power is also the same.
- ``zero_ohm``: the zero-ohm jumper. The library part is ``R_0R_J_<size>``
  (declared 5%), the UNI-ROYAL number carries the letter F. The maintainer
  decided on 2026-11-13 that these may be alternates of that part, with a
  note; each row carries the note (``jumper_note``), and ``jumper_ratings``
  restates what the datasheet says about the jumper of each size. They stay
  in their own list because their basis differs from the others': a decision
  and a rating, not equal resistance and tolerance.
- ``new_values``: decodes, but the library emits no such part.
- ``not_decoded``: the rule refuses the part number.

``--validate-pages CACHE_DIR`` compares every decoded row with what its JLCPCB
page states (resistance, tolerance, rated power), reading the pages from the
fetch cache. It prints counts and mismatches and writes nothing: pages and
their descriptions are transient working data (tools/sourcing/http.py).
"""

from __future__ import annotations

import argparse
from decimal import Decimal, InvalidOperation
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import gen_passive  # noqa: E402

CATALOG = ROOT / "tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json"
OUT = ROOT / "tools/passive_data/uniroyal_dry_run.json"
ORIGIN = "https://jlcpcb.com"
# The snapshot's own spelling. It is a filter on snapshot rows, not an alias:
# mfr_aliases.json has no entry for it. The maintainer decided the library's name
# on 2026-11-13 (LIBRARY_MFR); the alias itself still waits for its review issue
# (alias_reviews/2026-11-11-uni-royal-proposal.md) and is not applied here.
CATALOG_MFR = "UNI-ROYAL(Uniroyal Elec)"
LIBRARY_MFR = "UNI-ROYAL"
DECIDED = "2026-11-13"
EVIDENCE = "tools/passive_data/uniroyal_chip_series.evidence.md"

# The jumper of each size as the datasheet rates it (p.5 §7 "Ratings": Resistance
# Value of Jumper, Rated Current of Jumper, Max. Overload Current of Jumper).
# The datasheet gives the jumper no tolerance and no part number.
UNIROYAL_JUMPER = {
    "0201": ("<50mΩ", "0.5A", "1A"),
    "0402": ("<50mΩ", "1A", "2A"),
    "0603": ("<50mΩ", "1A", "2A"),
    "0805": ("<50mΩ", "2A", "5A"),
    "1206": ("<50mΩ", "2A", "10A"),
    "1210": ("<50mΩ", "2A", "10A"),
    "2010": ("<50mΩ", "2A", "10A"),
    "2512": ("<50mΩ", "2A", "10A"),
}

# The OpenWalkieTalkie Rev B order package places these ten numbers on 43
# resistor positions (order-package inventory of 2026-11; that inventory is
# not in this repository). (LCSC number, part number on the order, placements).
OPENWALKIETALKIE_REV_B = (
    ("C25741", "0402WGF1003TCE", 14),
    ("C11702", "0402WGF1001TCE", 12),
    ("C26083", "0402WGF1004TCE", 5),
    ("C17168", "0402WGF0000TCE", 3),
    ("C25768", "0402WGF2202TCE", 2),
    ("C25905", "0402WGF5101TCE", 2),
    ("C25744", "0402WGF1002TCE", 2),
    ("C25092", "0402WGF220JTCE", 1),
    ("C25890", "0402WGF3301TCE", 1),
    ("C25760", "0402WGF1803TCE", 1),
)


def jumper_note(size: str) -> str:
    """The note a UNI-ROYAL jumper carries as an alternate of ``R_0R_J_<size>``: what
    the datasheet states for that size's jumper, and what it does not."""
    resistance, rated, overload = UNIROYAL_JUMPER[size]
    return (f"Alternate of R_0R_J_{size} by the maintainer's decision of {DECIDED}. The letter F states no 1% "
            f"tolerance here: UNI-ROYAL's datasheet SMD-SP-001 V.11 rates the {size} jumper at {resistance}, rated "
            f"current {rated}, maximum overload current {overload} (p.5 §7), and states no tolerance for a jumper, "
            f"no meaning for the tolerance letter on one, and no jumper part number ({EVIDENCE}, "
            f"\"The zero-ohm jumper\"). The library part's 5% is Yageo's ordering code for its jumper; Yageo "
            f"states the same resistance limit and the same two currents for this size.")


def library_part(decoded: dict, verified: dict) -> tuple[str | None, str]:
    """(emitted library part name or None, list name) for a decoded part number."""
    size, tol = decoded["size"], decoded["tolerance"]
    packing = gen_passive.R_PACKING[size]
    if decoded["mantissa"] is None:
        return f"R_0R_J_{size}", "zero_ohm"
    point = (decoded["mantissa"], decoded["decade"])
    if point not in gen_passive.resistor_values(tol):
        return None, "value not in the library's grid"
    token = gen_passive.ohm_token(*point)
    # gen_passive.py emits no part whose Yageo primary Yageo will not confirm.
    if verified.get(f"RC{size}{tol}{packing}-07{token}L") is False:
        return None, "no library part: its Yageo primary is unconfirmed"
    return f"R_{token}_{tol}_{size}", "alternates"


def ohms_text(ohms: Decimal) -> str:
    """`10000`, `49.9`, `0.1`: plain decimal, no exponent, no trailing zeros."""
    text = format(ohms, "f")
    return text.rstrip("0").rstrip(".") if "." in text else text


def classify(mpn: str, verified: dict) -> dict:
    """One part number's dry-run result: `list` plus the fields that list carries."""
    decoded = gen_passive.uniroyal_decode(mpn)
    if decoded is None:
        return {"list": "not_decoded", "reason": "not decoded"}
    if decoded["power"] != gen_passive.R_POWER[decoded["size"]]:
        return {"list": "not_decoded", "reason": "rated power differs from the library's for this size"}
    part, where = library_part(decoded, verified)
    fields = {"size": decoded["size"], "ohms": ohms_text(decoded["ohms"]),
              "tolerance": gen_passive.UNIROYAL_TOLERANCE[decoded["tolerance"]][0]}
    if part is None:
        return {"list": "new_values", **fields, "reason": where}
    if where == "zero_ohm":
        # A jumper has no tolerance: the row carries the letter as printed, and the note.
        fields = {"size": decoded["size"], "ohms": "0", "letter": decoded["tolerance"],
                  "alternate": True, "note": jumper_note(decoded["size"])}
    return {"list": where, "part": part, **fields}


def catalog_rows() -> list[dict]:
    pages = json.loads(CATALOG.read_text(encoding="utf-8"))
    return sorted((p for p in pages if p["mfr"] == CATALOG_MFR), key=lambda p: int(p["c"][1:]))


def build() -> dict:
    verified = gen_passive.load_verified()
    lists: dict[str, list] = {"alternates": [], "zero_ohm": [], "new_values": [], "not_decoded": []}
    by_pn, taken = {}, {}
    for page in catalog_rows():
        result = classify(page["mpn"], verified)
        name = result.pop("list")
        row = {"pn": page["c"], "mpn": page["mpn"], "category": page["category"], **result}
        if name != "not_decoded" and result["size"] != page["package"]:
            # The snapshot's package column is the one attribute the repository keeps.
            name, row = "not_decoded", {"pn": page["c"], "mpn": page["mpn"], "category": page["category"],
                                        "reason": f"decoded size {result['size']}, snapshot package {page['package']}"}
        if name in ("alternates", "zero_ohm"):
            if row["part"] in taken:
                raise SystemExit(f"{row['part']}: both {taken[row['part']]} and {page['mpn']}")
            taken[row["part"]] = page["mpn"]
        lists[name].append(row)
        by_pn[page["c"]] = (name, row)
    highlighted = []
    for pn, mpn, placements in OPENWALKIETALKIE_REV_B:
        result = classify(mpn, verified)
        entry = {"pn": pn, "mpn": mpn, "placements": placements, "list": result.pop("list"), **result}
        in_snapshot = by_pn.get(pn)
        entry["in_snapshot"] = in_snapshot is not None
        if in_snapshot is not None and (in_snapshot[1]["mpn"] != mpn or in_snapshot[0] != entry["list"]):
            raise SystemExit(f"{pn}: the order names {mpn}, the snapshot {in_snapshot[1]['mpn']}")
        highlighted.append(entry)
    counts = {
        "snapshot_rows": sum(map(len, lists.values())),
        **{name: len(rows) for name, rows in lists.items()},
        "by_category": {name: {cat: sum(r["category"] == cat for r in rows) for cat in ("basic", "preferred")}
                        for name, rows in lists.items()},
        "alternates_by_size": {size: sum(r["size"] == size for r in lists["alternates"])
                               for size in gen_passive.UNIROYAL_SIZE},
        # What a later PR would emit: the alternates, and the jumpers as alternates with their note.
        "alternates_with_jumpers": len(lists["alternates"]) + len(lists["zero_ohm"]),
        "openwalkietalkie_rev_b": {
            "numbers": len(highlighted),
            "placements": sum(e["placements"] for e in highlighted),
            "decoded": sum(e["list"] != "not_decoded" for e in highlighted),
            "in_snapshot": sum(e["in_snapshot"] for e in highlighted),
        },
    }
    return {
        "_provenance": {
            "status": "dry run: gen_passive.py does not read this file; no alternate is emitted and nothing is bound",
            "source": "B1 snapshot tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json (confirmed JLCPCB pages), "
                      f"rows whose manufacturer reads '{CATALOG_MFR}'",
            "rules": "transcribed from UNI-ROYAL's datasheet 'Thick Film Chip Resistors' SMD-SP-001 V.11 "
                     "(tools/passive_data/uniroyal_chip_series.evidence.md) and checked against JLCPCB's pages; "
                     "the zero-ohm jumper's part number is not in that datasheet and is still inferred from "
                     "the snapshot (tools/gen_passive.py, UNI-ROYAL section)",
            "meaning": "'alternates' decode to the resistance, tolerance, case size and rated power of the named "
                       "library part; 'zero_ohm' are jumpers lettered F, alternates of the library's 5% R_0R_J "
                       "part by the maintainer's decision (see 'decisions'), each with its note, and "
                       "'jumper_ratings' is the datasheet's p.5 rating of each size's jumper (it states no "
                       "tolerance for a jumper); "
                       "'new_values' decode but the library emits no such part; 'not_decoded' are refused by the rule",
            "generator": "tools/uniroyal_alternates.py",
        },
        "decisions": {
            "date": DECIDED,
            "by": "the maintainer",
            "library_mfr": LIBRARY_MFR,
            "alias": f"'{CATALOG_MFR}' -> '{LIBRARY_MFR}': decided, not applied; mfr_aliases.json is unchanged "
                     "until the review issue exists "
                     "(tools/sourcing_data/alias_reviews/2026-11-11-uni-royal-proposal.md)",
            "jumpers": "the four F-lettered zero-ohm jumpers may be alternates of R_0R_J_<size>, with a note; "
                       "each 'zero_ohm' row carries it",
        },
        "counts": counts,
        "openwalkietalkie_rev_b": highlighted,
        "jumper_ratings": {size: {"resistance": r, "rated_current": rated, "max_overload_current": overload}
                           for size, (r, rated, overload) in UNIROYAL_JUMPER.items()},
        **lists,
    }


# --------------------------------------------------------------------------
# --validate-pages: the decoded fields against JLCPCB's own page for the row.
# --------------------------------------------------------------------------

_LD_JSON = re.compile(r'<script[^>]*type="application/ld\+json"[^>]*>(.*?)</script>', re.S)
_RESISTANCE = re.compile(r"(?<![\w.])(\d+(?:\.\d+)?)(mΩ|Ω|kΩ|MΩ)(?!\w)")
_TOLERANCE = re.compile(r"±(\d+(?:\.\d+)?)%")
_POWER = re.compile(r"(?<![\w.])(\d+(?:\.\d+)?)(mW|W)(?![\w/])")
_OHM_SCALE = {"mΩ": -3, "Ω": 0, "kΩ": 3, "MΩ": 6}


def page_attributes(html: str, c: str, mpn: str) -> dict | None:
    """Resistance (ohms), tolerance (percent) and power (mW) the page states for its own part.

    They are read from the description of the page's schema.org ``Product``
    whose ``sku`` and ``mpn`` are the row's. A field the description states
    zero or several times is None; a page that is not this part's is None.
    """
    description = None
    for match in _LD_JSON.finditer(html):
        try:
            data = json.loads(match.group(1))
        except ValueError:
            continue
        if isinstance(data, dict) and data.get("@type") == "Product":
            if description is not None or data.get("sku") != c or data.get("mpn") != mpn:
                return None
            description = data.get("description")
    if not isinstance(description, str):
        return None

    def only(pattern):
        found = pattern.findall(description)
        return found[0] if len(found) == 1 else None

    try:
        resistance, tolerance, power = only(_RESISTANCE), only(_TOLERANCE), only(_POWER)
        return {
            "ohms": resistance and Decimal(resistance[0]).scaleb(_OHM_SCALE[resistance[1]]),
            "tolerance": tolerance and Decimal(tolerance),
            "milliwatts": power and Decimal(power[0]).scaleb(0 if power[1] == "mW" else 3),
        }
    except InvalidOperation:
        return None


def milliwatts(power: str) -> Decimal:
    """`62.5mW` or `1W`, the two spellings of gen_passive.R_POWER, in milliwatts."""
    if power.endswith("mW"):
        return Decimal(power.removesuffix("mW"))
    return Decimal(power.removesuffix("W")).scaleb(3)


def cached_page(cache_dir: Path, url: str) -> str | None:
    """The cached body for `url`, under the same checks as the fetch client's cache read."""
    digest = hashlib.sha256(url.encode("utf-8")).hexdigest()
    try:
        metadata = json.loads((cache_dir / f"{digest}.json").read_text(encoding="utf-8"))
        body = (cache_dir / f"{digest}.body").read_bytes()
        if metadata["url"] != url or metadata["sha256"] != hashlib.sha256(body).hexdigest():
            return None
        return body.decode("utf-8", errors="replace")
    except (OSError, KeyError, ValueError, TypeError):
        return None


def validate_pages(cache_dir: Path, rows: list[dict] | None = None) -> dict:
    """Decoded fields against each row's cached page. Nothing is fetched or written."""
    result = {"rows": 0, "not_decoded": [], "no_page": [], "unreadable": [], "match": 0, "mismatch": []}
    for page in catalog_rows() if rows is None else rows:
        result["rows"] += 1
        decoded = gen_passive.uniroyal_decode(page["mpn"])
        if decoded is None:
            result["not_decoded"].append({"pn": page["c"], "mpn": page["mpn"], "package": page["package"]})
            continue
        html = cached_page(cache_dir, ORIGIN + page["partdetail"])
        if html is None:
            result["no_page"].append(page["c"])
            continue
        stated = page_attributes(html, page["c"], page["mpn"])
        if stated is None or None in stated.values():
            result["unreadable"].append(page["c"])
            continue
        differs = [name for name, ours, theirs in (
            ("resistance", decoded["ohms"], stated["ohms"]),
            ("tolerance", Decimal(gen_passive.UNIROYAL_TOLERANCE[decoded["tolerance"]][0].rstrip("%")),
             stated["tolerance"]),
            ("power", milliwatts(decoded["power"]), stated["milliwatts"]),
        ) if ours != theirs]
        if decoded["size"] != page["package"]:
            differs.append("package")
        if differs:
            result["mismatch"].append({"pn": page["c"], "mpn": page["mpn"], "differs": differs})
        else:
            result["match"] += 1
    return result


def print_validation(result: dict) -> None:
    decoded = result["rows"] - len(result["not_decoded"])
    print(f"{result['rows']} snapshot rows; {decoded} decoded, {len(result['not_decoded'])} refused")
    print(f"decoded rows whose page agrees on resistance, tolerance, power and package: "
          f"{result['match']} of {decoded}")
    for name, label in (("mismatch", "disagree"), ("no_page", "have no cached page"),
                        ("unreadable", "have a page that does not state all three fields once")):
        print(f"{len(result[name])} {label}")
        for item in result[name]:
            print(f"  {item}")
    print("refused part numbers:")
    for item in result["not_decoded"]:
        print(f"  {item['pn']} {item['mpn']!r} (package {item['package']})")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true", help="fail if the committed report differs")
    parser.add_argument("--validate-pages", metavar="CACHE_DIR", type=Path,
                        help="compare decoded rows with their cached JLCPCB pages; writes nothing")
    args = parser.parse_args(argv)
    if args.validate_pages is not None:
        result = validate_pages(args.validate_pages)
        print_validation(result)
        # Anything short of every decoded row agreeing is a failed check, a missing page included.
        return 1 if result["mismatch"] or result["no_page"] or result["unreadable"] else 0
    text = json.dumps(build(), indent=1, ensure_ascii=False) + "\n"
    if args.check:
        if not OUT.is_file() or OUT.read_text(encoding="utf-8") != text:
            print(f"uniroyal_alternates.py: {OUT} is stale", file=sys.stderr)
            return 1
        return 0
    OUT.write_text(text, encoding="utf-8")
    counts = json.loads(text)["counts"]
    print(", ".join(f"{counts[name]} {name}" for name in ("alternates", "zero_ohm", "new_values", "not_decoded")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
