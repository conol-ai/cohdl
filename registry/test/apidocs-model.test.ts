import { describe, expect, it } from "vitest";
import type { ApiDocsItem, DeviceDoc, DevicePinDoc, FootprintDoc, PadDoc, PartDoc } from "../src/ui/api";
import {
  arcPath,
  arcPoint,
  avlColumns,
  chamferPoints,
  classifyFq,
  filterItems,
  footprintBounds,
  isGroundName,
  itemSummary,
  mm,
  moduleGroups,
  padHalfSize,
  pinsForVariant,
  rotatedHalfExtents,
  scaleBarMm,
  symbolSides,
} from "../src/ui/apidocs-model";

function pin(
  name: string,
  role: DevicePinDoc["role"],
  obligation: DevicePinDoc["obligation"] = "required",
): DevicePinDoc {
  return { name, role, obligation, numbers: ["1"] };
}

function padItem(fq: string, pub = true): ApiDocsItem {
  const idx = fq.lastIndexOf("::");
  return {
    fq,
    name: idx === -1 ? fq : fq.slice(idx + 2),
    kind: "pad",
    pub,
    module: idx === -1 ? "pkg" : fq.slice(0, idx),
    file: "src/pads.cohdl",
    line: 1,
    pad: {},
  };
}

describe("mm parsing", () => {
  it("parses the emitter's canonical decimal strings", () => {
    expect(mm("1.2")).toBe(1.2);
    expect(mm("-0.95")).toBe(-0.95);
    expect(mm("0")).toBe(0);
    expect(mm("12")).toBe(12);
  });

  it("degrades missing or malformed values to 0 instead of NaN geometry", () => {
    expect(mm(undefined)).toBe(0);
    expect(mm("not-a-length")).toBe(0);
    expect(mm("")).toBe(0);
  });
});

describe("symbol side assignment", () => {
  it("puts input and passive pins on the left", () => {
    const sides = symbolSides([pin("IN", "input"), pin("A", "passive")]);
    expect(sides.left.map((p) => p.name)).toEqual(["IN", "A"]);
    expect(sides.right).toEqual([]);
    expect(sides.top).toEqual([]);
    expect(sides.bottom).toEqual([]);
  });

  it("puts output, bidirectional, and power_out pins on the right", () => {
    const sides = symbolSides([
      pin("OUT", "output"),
      pin("SDA", "bidirectional"),
      pin("VOUT", "power_out"),
    ]);
    expect(sides.right.map((p) => p.name)).toEqual(["OUT", "SDA", "VOUT"]);
    expect(sides.left).toEqual([]);
  });

  it("puts power_in on top except ground-looking names, which go bottom", () => {
    const sides = symbolSides([
      pin("VIN", "power_in"),
      pin("VDD", "power_in"),
      pin("GND", "power_in"),
      pin("AGND", "power_in"),
      pin("PGND", "power_in"),
      pin("VSS", "power_in"),
      pin("EP", "power_in"),
    ]);
    expect(sides.top.map((p) => p.name)).toEqual(["VIN", "VDD"]);
    expect(sides.bottom.map((p) => p.name)).toEqual(["GND", "AGND", "PGND", "VSS", "EP"]);
  });

  it("compares ground names uppercased", () => {
    expect(isGroundName("gnd")).toBe(true);
    expect(isGroundName("dgnd")).toBe(true);
    expect(isGroundName("vss")).toBe(true);
    expect(isGroundName("ep")).toBe(true);
    expect(isGroundName("VSSA")).toBe(false); // equals-only names stay exact
    expect(isGroundName("EPOCH")).toBe(false);
    expect(isGroundName("VDD")).toBe(false);
  });

  it("only applies the ground heuristic to power_in", () => {
    const sides = symbolSides([pin("GND_SENSE", "input"), pin("EP", "passive")]);
    expect(sides.left.map((p) => p.name)).toEqual(["GND_SENSE", "EP"]);
    expect(sides.bottom).toEqual([]);
  });

  it("keeps optional pins on their role's side, in order", () => {
    const sides = symbolSides([
      pin("OUT1", "output"),
      pin("OUT2", "output", "optional"),
      pin("NC", "passive", "optional"),
    ]);
    expect(sides.right.map((p) => p.name)).toEqual(["OUT1", "OUT2"]);
    expect(sides.right[1].obligation).toBe("optional");
    expect(sides.left.map((p) => p.name)).toEqual(["NC"]);
  });
});

describe("filtering", () => {
  const items = [
    padItem("pkg::pads::P_0402"),
    padItem("pkg::pads::P_0603"),
    padItem("pkg::internal::HIDDEN", false),
  ];

  it("matches a case-insensitive substring of the name", () => {
    expect(filterItems(items, { q: "p_04" }).map((i) => i.name)).toEqual(["P_0402"]);
    expect(filterItems(items, { q: "P_04" }).map((i) => i.name)).toEqual(["P_0402"]);
  });

  it("matches a case-insensitive substring of the fq", () => {
    expect(filterItems(items, { q: "PKG::PADS" }).map((i) => i.name)).toEqual([
      "P_0402",
      "P_0603",
    ]);
  });

  it("narrows by kind", () => {
    expect(filterItems(items, { kind: "pad" })).toHaveLength(2);
    expect(filterItems(items, { kind: "device" })).toEqual([]);
  });

  it("hides non-pub items unless showPrivate is set", () => {
    expect(filterItems(items, {}).map((i) => i.name)).toEqual(["P_0402", "P_0603"]);
    expect(filterItems(items, { showPrivate: true })).toHaveLength(3);
    expect(filterItems(items, { q: "hidden" })).toEqual([]);
    expect(filterItems(items, { q: "hidden", showPrivate: true }).map((i) => i.name)).toEqual([
      "HIDDEN",
    ]);
  });
});

describe("module grouping", () => {
  it("groups by module, sorted, preserving item order inside a group", () => {
    const items = [
      padItem("pkg::b::X"),
      padItem("pkg::a::Y"),
      padItem("pkg::b::Z"),
    ];
    const groups = moduleGroups(items);
    expect(groups.map((g) => g.module)).toEqual(["pkg::a", "pkg::b"]);
    expect(groups[1].items.map((i) => i.name)).toEqual(["X", "Z"]);
  });
});

describe("fq classification", () => {
  const doc = {
    schema_version: 1,
    generator: "cohdl 0.3.0",
    package: { name: "@st/stm32", version: "0.1.0", root: "st_stm32" },
    dependencies: [{ name: "std", version: "0.3.0", root: "std" }],
    items: [],
  };

  it("classifies local, dependency, and unknown roots", () => {
    expect(classifyFq("st_stm32::f0::STM32F072CB", doc)).toEqual({
      kind: "local",
      fq: "st_stm32::f0::STM32F072CB",
    });
    expect(classifyFq("std::Capacitor", doc)).toEqual({
      kind: "dependency",
      package: "std",
      fq: "std::Capacitor",
    });
    expect(classifyFq("mystery::Thing", doc)).toEqual({ kind: "plain", fq: "mystery::Thing" });
  });

  it("treats bare-named designs as local", () => {
    expect(classifyFq("MainBoard", doc)).toEqual({ kind: "local", fq: "MainBoard" });
  });
});

describe("footprint bounding box", () => {
  const rectPad: PadDoc = { shape: "rect", size: ["1", "0.5"] };
  const pads = new Map<string, PadDoc>([["pkg::pads::P", rectPad]]);

  it("uses circle and annulus leading diameters for half-sizes", () => {
    expect(padHalfSize({ shape: "circle", size: ["2"] })).toEqual({ hw: 1, hh: 1 });
    expect(padHalfSize({ shape: "annulus", size: ["3", "1"] })).toEqual({ hw: 1.5, hh: 1.5 });
    expect(padHalfSize(rectPad)).toEqual({ hw: 0.5, hh: 0.25 });
  });

  it("bounds an unrotated pad by its size", () => {
    const fp: FootprintDoc = {
      placeholder: false,
      pads: [{ number: "1", pad: "pkg::pads::P", x: "0", y: "0" }],
    };
    const b = footprintBounds(fp, pads, 0);
    expect(b).toEqual({ minX: -0.5, minY: -0.25, maxX: 0.5, maxY: 0.25 });
  });

  it("swaps the extents of a 90°-rotated pad", () => {
    const fp: FootprintDoc = {
      placeholder: false,
      pads: [{ number: "1", pad: "pkg::pads::P", x: "1", y: "0", rotate: 90 }],
    };
    const b = footprintBounds(fp, pads, 0);
    expect(b.minX).toBeCloseTo(0.75, 6);
    expect(b.maxX).toBeCloseTo(1.25, 6);
    expect(b.minY).toBeCloseTo(-0.5, 6);
    expect(b.maxY).toBeCloseTo(0.5, 6);
  });

  it("bounds an arbitrary rotation by the rotated box", () => {
    const { hw, hh } = rotatedHalfExtents(0.5, 0.25, 30);
    expect(hw).toBeCloseTo(0.5 * Math.cos(Math.PI / 6) + 0.25 * Math.sin(Math.PI / 6), 9);
    expect(hh).toBeCloseTo(0.5 * Math.sin(Math.PI / 6) + 0.25 * Math.cos(Math.PI / 6), 9);
    // the sign of the angle cannot matter for a bounding box
    expect(rotatedHalfExtents(0.5, 0.25, -30)).toEqual({ hw, hh });
  });

  it("includes silk stroke widths", () => {
    const fp: FootprintDoc = {
      placeholder: false,
      silk: [
        { kind: "line", from: ["-1", "0"], to: ["1", "0"], width: "0.2" },
        { kind: "circle", at: ["0", "2"], radius: "0.5", width: "0.2" },
      ],
    };
    const b = footprintBounds(fp, pads, 0);
    expect(b.minX).toBeCloseTo(-1.1, 9);
    expect(b.maxX).toBeCloseTo(1.1, 9);
    expect(b.minY).toBeCloseTo(-0.1, 9);
    expect(b.maxY).toBeCloseTo(2.6, 9);
  });

  it("includes polygon points grown by half the stroke width", () => {
    const fp: FootprintDoc = {
      placeholder: false,
      silk: [
        {
          kind: "polygon",
          points: [
            ["0", "0"],
            ["2", "0"],
            ["0", "1"],
          ],
          width: "0.4",
        },
      ],
    };
    const b = footprintBounds(fp, pads, 0);
    expect(b).toEqual({ minX: -0.2, minY: -0.2, maxX: 2.2, maxY: 1.2 });
  });

  it("includes courtyard, window, and mount holes", () => {
    const fp: FootprintDoc = {
      placeholder: false,
      courtyard: { shape: "rect", at: ["0", "0"], size: ["4", "2"] },
      window: { shape: "rect", at: ["0", "0"], size: ["1", "6"] },
      mount_holes: [
        {
          number: "1",
          plating: "non_plated",
          shape: "circle",
          x: "5",
          y: "0",
          diameter: "2",
        },
      ],
    };
    const b = footprintBounds(fp, pads, 0);
    expect(b).toEqual({ minX: -2, minY: -3, maxX: 6, maxY: 3 });
  });

  it("applies the margin on every side and defaults an empty footprint", () => {
    const fp: FootprintDoc = {
      placeholder: false,
      pads: [{ number: "1", pad: "pkg::pads::P", x: "0", y: "0" }],
    };
    const b = footprintBounds(fp, pads, 1);
    expect(b).toEqual({ minX: -1.5, minY: -1.25, maxX: 1.5, maxY: 1.25 });
    expect(footprintBounds({ placeholder: false }, pads, 0)).toEqual({
      minX: -1,
      minY: -1,
      maxX: 1,
      maxY: 1,
    });
  });
});

describe("chamfered pads", () => {
  it("produces the explicit 5-vertex polygon for each corner", () => {
    for (const corner of ["top_left", "top_right", "bottom_left", "bottom_right"]) {
      expect(chamferPoints(1, 0.5, corner, 0.2)).toHaveLength(5);
    }
    // y-down frame: top_left is (−hw, −hh); the cut replaces it with two
    // points 0.2 along each edge.
    const points = chamferPoints(1, 0.5, "top_left", 0.2);
    expect(points).toContainEqual([-1, -0.3]);
    expect(points).toContainEqual([-0.8, -0.5]);
    expect(points).not.toContainEqual([-1, -0.5]);
  });

  it("degrades an unknown corner to the plain rectangle", () => {
    expect(chamferPoints(1, 0.5, "middle", 0.2)).toHaveLength(4);
  });
});

describe("silkscreen arcs", () => {
  it("places cardinal angles on the y-down circle (visually clockwise)", () => {
    const [x0, y0] = arcPoint(0, 0, 2, 0);
    expect(x0).toBeCloseTo(2, 9);
    expect(y0).toBeCloseTo(0, 9);
    const [x90, y90] = arcPoint(0, 0, 2, 90);
    expect(x90).toBeCloseTo(0, 9);
    expect(y90).toBeCloseTo(2, 9); // +y: below the centre on screen
    const [x180, y180] = arcPoint(0, 0, 2, 180);
    expect(x180).toBeCloseTo(-2, 9);
    expect(y180).toBeCloseTo(0, 9);
    const [x270, y270] = arcPoint(0, 0, 2, 270);
    expect(x270).toBeCloseTo(0, 9);
    expect(y270).toBeCloseTo(-2, 9);
  });

  it("emits sweep-flag-1 arcs with exact cardinal endpoints", () => {
    expect(arcPath(0, 0, 1, 0, 90)).toBe("M 1 0 A 1 1 0 0 1 0 1");
    expect(arcPath(0, 0, 1, 0, 180)).toBe("M 1 0 A 1 1 0 0 1 -1 0");
    expect(arcPath(0, 0, 1, 0, 270)).toBe("M 1 0 A 1 1 0 1 1 0 -1");
    expect(arcPath(0, 0, 1, 90, 180)).toBe("M 0 1 A 1 1 0 0 1 -1 0");
  });

  it("sweeps 90→270 monotonically up through 180° with sweep flag 1", () => {
    expect(arcPath(0, 0, 1, 90, 270)).toBe("M 0 1 A 1 1 0 0 1 0 -1");
  });

  it("sweeps 270→90 monotonically DOWN through 180°, never through 0°", () => {
    // The compiler puts the arc midpoint at pt((start+end)/2) = pt(180°) —
    // the LEFT of centre — so this must be the left half-circle (sweep
    // flag 0), not its mirror-image complement. And never emit -0.
    const [midX] = arcPoint(0, 0, 1, (270 + 90) / 2);
    expect(midX).toBeLessThan(0);
    expect(arcPath(0, 0, 1, 270, 90)).toBe("M 0 -1 A 1 1 0 0 0 0 1");
  });

  it("renders a zero-length sweep as nothing — a degenerate point", () => {
    expect(arcPath(0, 0, 1, 45, 45)).toBe("");
  });

  it("draws a full 360° sweep as a circle of two half arcs", () => {
    expect(arcPath(0, 0, 1, 0, 360)).toBe("M 1 0 A 1 1 0 0 1 -1 0 A 1 1 0 0 1 1 0");
  });
});

describe("scale bar", () => {
  it("picks the largest 1-2-5 value at most a quarter of the width", () => {
    expect(scaleBarMm(4)).toBe(1);
    expect(scaleBarMm(10)).toBe(2);
    expect(scaleBarMm(100)).toBe(20);
    expect(scaleBarMm(1.2)).toBe(0.2);
    expect(scaleBarMm(0.02)).toBe(0.01); // floor
  });
});

describe("AVL columns", () => {
  it("unions field names in first-appearance order across primary and alts", () => {
    const columns = avlColumns({
      device: "passive::devices::MLCC",
      primary: {
        fields: [
          { name: "mfr", value: "Samsung" },
          { name: "mpn", value: "CL05B104KO5NNNC" },
        ],
      },
      alts: [{ fields: [{ name: "mpn", value: "GRM155R71C104KA88D" }, { name: "note", value: "x" }] }],
    });
    expect(columns).toEqual(["mfr", "mpn", "note"]);
  });
});

// The server intentionally does not deep-validate the owner-uploaded docs
// JSON (docs/apidocs.md), so the model must be total over hostile shapes:
// missing payload keys, non-array arrays, null strings.
describe("hostile document tolerance", () => {
  it("summarizes an item whose kind-named payload is missing", () => {
    for (const kind of ["design", "device", "part", "trait", "fn", "footprint", "pad"]) {
      const item = { ...padItem(`pkg::x::${kind.toUpperCase()}`), kind } as Record<
        string,
        unknown
      >;
      delete item.pad;
      expect(itemSummary(item as unknown as ApiDocsItem)).toBe("");
    }
  });

  it("returns no pins when the device pin blocks are not arrays", () => {
    const flat = { designator_prefix: "U", pins: "bogus" } as unknown as DeviceDoc;
    expect(pinsForVariant(flat, undefined)).toEqual([]);
    const nested = { designator_prefix: "U", pins: [{ pins: "bogus" }] } as unknown as DeviceDoc;
    expect(pinsForVariant(nested, undefined)).toEqual([]);
  });

  it("builds AVL columns over missing or non-iterable field lists", () => {
    expect(avlColumns({ device: "d", primary: {} } as unknown as PartDoc)).toEqual([]);
    expect(
      avlColumns({
        device: "d",
        primary: { fields: "bogus" },
        alts: [{ fields: [{ name: "mpn", value: "X" }] }, null],
      } as unknown as PartDoc),
    ).toEqual(["mpn"]);
  });

  it("filters items whose name and fq are null without throwing", () => {
    const broken = { ...padItem("pkg::pads::P"), name: null, fq: null } as unknown as ApiDocsItem;
    expect(filterItems([broken], { q: "p" })).toEqual([]);
    expect(filterItems([broken], {})).toHaveLength(1);
  });

  it("bounds a footprint with missing geometry fields instead of crashing", () => {
    const fp = {
      placeholder: false,
      pads: [null],
      mount_holes: "bogus",
      courtyard: { shape: "rect" },
      silk: [{ kind: "line", width: "0.2" }, { kind: "polygon" }, null],
      silkscreen_ref: {},
    } as unknown as FootprintDoc;
    const b = footprintBounds(fp, new Map(), 0);
    expect(Number.isFinite(b.minX)).toBe(true);
    expect(Number.isFinite(b.maxY)).toBe(true);
  });
});
