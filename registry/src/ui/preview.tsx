// Client-side SVG previews for the API explorer: an auto-drawn IC-style
// schematic symbol for devices, and an exact footprint rendering from the
// docs payload. Every drawn string is publisher content and only ever
// becomes React text or an SVG attribute. Colors come from CSS variables
// (the `--fp-*` layer palette and the design tokens) — never hex in here.

import React from "react";
import type { DevicePinDoc, FootprintDoc, OutlineDoc, PadDoc, SilkDoc } from "./api";
import {
  arcPath,
  asArray,
  chamferPoints,
  footprintBounds,
  mm,
  scaleBarMm,
  shortName,
  symbolSides,
} from "./apidocs-model";

// --- schematic symbol -------------------------------------------------------

const PITCH = 26; // pin grid, viewBox units
const STUB = 30; // pin stub length
const CHAR_W = 6.2; // 10px mono advance estimate, for body sizing

function centers(count: number, extent: number): number[] {
  const start = (extent - (count - 1) * PITCH) / 2;
  return Array.from({ length: count }, (_, i) => start + i * PITCH);
}

function maxNameLen(pins: DevicePinDoc[]): number {
  return pins.reduce((max, pin) => Math.max(max, String(pin?.name ?? "").length), 0);
}

function pinClass(pin: DevicePinDoc): string {
  return pin.obligation === "optional" ? "sym-pin-g sym-optional" : "sym-pin-g";
}

function pinTitle(pin: DevicePinDoc): string {
  const numbers = asArray(pin.numbers).join(", ");
  return `${pin.name} — ${pin.role}, pin ${numbers}${
    pin.obligation === "optional" ? " (optional)" : ""
  }`;
}

/// IC-style auto-drawn symbol: rect body, pin stubs with physical numbers
/// outside and names inside, optional pins dashed. The caller passes the
/// chosen variant's pin set. Height scales with pin count — tall is fine.
export function SymbolPreview({ label, pins }: { label: string; pins: DevicePinDoc[] }) {
  const sides = symbolSides(pins);
  const lrRows = Math.max(sides.left.length, sides.right.length);
  const tbCols = Math.max(sides.top.length, sides.bottom.length);
  const bodyW = Math.max(
    110,
    (tbCols + 1) * PITCH,
    (maxNameLen(sides.left) + maxNameLen(sides.right)) * CHAR_W + 40,
  );
  const bodyH = Math.max(
    70,
    (lrRows + 1) * PITCH,
    Math.max(maxNameLen(sides.top), maxNameLen(sides.bottom)) * CHAR_W + 26,
  );
  const leftY = centers(sides.left.length, bodyH);
  const rightY = centers(sides.right.length, bodyH);
  const topX = centers(sides.top.length, bodyW);
  const bottomX = centers(sides.bottom.length, bodyW);
  const padLeft = sides.left.length > 0 ? STUB + 10 : 12;
  const padRight = sides.right.length > 0 ? STUB + 10 : 12;
  const padTop = sides.top.length > 0 ? STUB + 16 : 12;
  const padBottom = sides.bottom.length > 0 ? STUB + 16 : 12;
  const totalW = padLeft + bodyW + padRight;
  const totalH = padTop + bodyH + padBottom;

  return (
    <svg
      className="sym-svg"
      viewBox={`${-padLeft} ${-padTop} ${totalW} ${totalH}`}
      width={totalW}
      height={totalH}
      role="img"
      aria-label={label}
    >
      <rect className="sym-body" x={0} y={0} width={bodyW} height={bodyH} rx={3} />
      {sides.left.map((pin, i) => (
        <g key={pin.name} className={pinClass(pin)}>
          <title>{pinTitle(pin)}</title>
          <line className="sym-pin" x1={-STUB} y1={leftY[i]} x2={0} y2={leftY[i]} />
          <text className="sym-number" x={-STUB / 2} y={leftY[i] - 4} textAnchor="middle">
            {asArray(pin.numbers).join(",")}
          </text>
          <text className="sym-name" x={8} y={leftY[i]} dominantBaseline="central">
            {pin.name}
          </text>
        </g>
      ))}
      {sides.right.map((pin, i) => (
        <g key={pin.name} className={pinClass(pin)}>
          <title>{pinTitle(pin)}</title>
          <line className="sym-pin" x1={bodyW} y1={rightY[i]} x2={bodyW + STUB} y2={rightY[i]} />
          <text
            className="sym-number"
            x={bodyW + STUB / 2}
            y={rightY[i] - 4}
            textAnchor="middle"
          >
            {asArray(pin.numbers).join(",")}
          </text>
          <text
            className="sym-name"
            x={bodyW - 8}
            y={rightY[i]}
            textAnchor="end"
            dominantBaseline="central"
          >
            {pin.name}
          </text>
        </g>
      ))}
      {sides.top.map((pin, i) => (
        <g key={pin.name} className={pinClass(pin)}>
          <title>{pinTitle(pin)}</title>
          <line className="sym-pin" x1={topX[i]} y1={-STUB} x2={topX[i]} y2={0} />
          <text className="sym-number" x={topX[i]} y={-STUB - 5} textAnchor="middle">
            {asArray(pin.numbers).join(",")}
          </text>
          <text
            className="sym-name"
            x={topX[i]}
            y={8}
            transform={`rotate(90 ${topX[i]} 8)`}
            dominantBaseline="central"
          >
            {pin.name}
          </text>
        </g>
      ))}
      {sides.bottom.map((pin, i) => (
        <g key={pin.name} className={pinClass(pin)}>
          <title>{pinTitle(pin)}</title>
          <line className="sym-pin" x1={bottomX[i]} y1={bodyH} x2={bottomX[i]} y2={bodyH + STUB} />
          <text
            className="sym-number"
            x={bottomX[i]}
            y={bodyH + STUB + 12}
            textAnchor="middle"
          >
            {asArray(pin.numbers).join(",")}
          </text>
          <text
            className="sym-name"
            x={bottomX[i]}
            y={bodyH - 8}
            transform={`rotate(90 ${bottomX[i]} ${bodyH - 8})`}
            textAnchor="end"
            dominantBaseline="central"
          >
            {pin.name}
          </text>
        </g>
      ))}
    </svg>
  );
}

// --- footprint --------------------------------------------------------------

/// One pad's copper (plus its drill), drawn in the pad's own frame — the
/// caller translates/rotates. Resolved shapes: rect, circle, oval (stadium),
/// annulus (ring), roundrect via corner_radius, one-corner 45° chamfer.
function PadCopper({ pad, hair }: { pad: PadDoc; hair: number }) {
  const size = pad.size ?? [];
  const shape = pad.shape ?? "rect";
  let copper: React.ReactNode;
  if (shape === "circle") {
    copper = <circle className="fp-copper-fill" r={mm(size[0]) / 2} />;
  } else if (shape === "annulus") {
    const outer = mm(size[0]) / 2;
    const inner = mm(size[1]) / 2;
    copper = (
      <circle
        className="fp-copper-stroke"
        r={(outer + inner) / 2}
        strokeWidth={Math.max(outer - inner, hair)}
      />
    );
  } else if (pad.chamfer) {
    const points = chamferPoints(
      mm(size[0]) / 2,
      mm(size[1]) / 2,
      pad.chamfer.corner,
      mm(pad.chamfer.cut),
    );
    copper = (
      <polygon
        className="fp-copper-fill"
        points={points.map(([x, y]) => `${x},${y}`).join(" ")}
      />
    );
  } else {
    const w = mm(size[0]);
    const h = mm(size[1]);
    const rx =
      shape === "oval"
        ? Math.min(w, h) / 2
        : pad.corner_radius !== undefined
          ? Math.min(mm(pad.corner_radius), Math.min(w, h) / 2)
          : 0;
    copper = (
      <rect className="fp-copper-fill" x={-w / 2} y={-h / 2} width={w} height={h} rx={rx} ry={rx} />
    );
  }
  let drill: React.ReactNode = null;
  if (pad.drill?.round !== undefined) {
    drill = <circle className="fp-hole" r={mm(pad.drill.round) / 2} />;
  } else if (pad.drill?.slot) {
    const w = mm(pad.drill.slot[0]);
    const l = mm(pad.drill.slot[1]);
    const rx = Math.min(w, l) / 2;
    drill = (
      <rect className="fp-hole" x={-w / 2} y={-l / 2} width={w} height={l} rx={rx} ry={rx} />
    );
  }
  return (
    <>
      {copper}
      {drill}
    </>
  );
}

function OutlineShape({
  outline,
  className,
  hair,
  dash,
}: {
  outline: OutlineDoc;
  className: string;
  hair: number;
  dash: string;
}) {
  const x = mm(outline.at?.[0]);
  const y = mm(outline.at?.[1]);
  if (outline.shape === "circle") {
    return (
      <circle
        className={className}
        cx={x}
        cy={y}
        r={mm(outline.size?.[0]) / 2}
        strokeWidth={hair}
        strokeDasharray={dash}
      />
    );
  }
  const w = mm(outline.size?.[0]);
  const h = mm(outline.size?.[1]);
  return (
    <rect
      className={className}
      x={x - w / 2}
      y={y - h / 2}
      width={w}
      height={h}
      strokeWidth={hair}
      strokeDasharray={dash}
    />
  );
}

function SilkShape({ g, hair }: { g: SilkDoc; hair: number }) {
  if (!g) return null;
  switch (g.kind) {
    case "line":
      return (
        <line
          className="fp-silk"
          x1={mm(g.from?.[0])}
          y1={mm(g.from?.[1])}
          x2={mm(g.to?.[0])}
          y2={mm(g.to?.[1])}
          strokeWidth={mm(g.width)}
          strokeLinecap="round"
        />
      );
    case "circle": {
      if (g.fill) {
        // A filled circle with stroke width w is a filled circle of r + w/2.
        return <circle className="fp-silk-fill" cx={mm(g.at?.[0])} cy={mm(g.at?.[1])} r={mm(g.radius) + mm(g.width) / 2} />;
      }
      return (
        <circle
          className="fp-silk"
          cx={mm(g.at?.[0])}
          cy={mm(g.at?.[1])}
          r={mm(g.radius)}
          strokeWidth={mm(g.width) || hair}
        />
      );
    }
    case "arc": {
      // A degenerate zero-sweep arc has an empty path — draw nothing,
      // matching the compiler's degenerate point.
      const d = arcPath(mm(g.at?.[0]), mm(g.at?.[1]), mm(g.radius), g.start_angle, g.end_angle);
      if (!d) return null;
      return (
        <path className="fp-silk" d={d} strokeWidth={mm(g.width)} strokeLinecap="round" />
      );
    }
    case "polygon": {
      const points = asArray(g.points)
        .map((p) => `${mm(p?.[0])},${mm(p?.[1])}`)
        .join(" ");
      if (g.fill) return <polygon className="fp-silk-fill" points={points} />;
      return (
        <polygon
          className="fp-silk"
          points={points}
          strokeWidth={mm(g.width) || hair}
          strokeLinejoin="round"
        />
      );
    }
    default:
      return null;
  }
}

/// Exact footprint rendering from the docs payload. The authoring frame is
/// y-down — identical to SVG — so coordinates are used verbatim; pad
/// rotation is `rotate(-angle)` about the pad centre. Hovering a pad
/// highlights it and its `<title>` names the pad number.
export function FootprintPreview({
  footprint,
  pads,
  label,
}: {
  footprint: FootprintDoc;
  pads: ReadonlyMap<string, PadDoc>;
  label: string;
}) {
  if (footprint.placeholder) return null;
  const raw = footprintBounds(footprint, pads, 0);
  const rawW = raw.maxX - raw.minX;
  const rawH = raw.maxY - raw.minY;
  const dim = Math.max(rawW, rawH, 0.001);
  const margin = Math.max(0.7, dim * 0.09);
  const textMm = Math.max(0.55, dim / 26);
  const hair = Math.max(0.04, dim / 260);
  const minX = raw.minX - margin;
  const minY = raw.minY - margin;
  const width = rawW + margin * 2;
  const barZone = textMm * 2.4;
  const height = rawH + margin * 2 + barZone;
  const barMm = scaleBarMm(width);
  const barX = minX + width * 0.03;
  const barY = raw.maxY + margin + barZone / 2;
  const tick = textMm * 0.5;

  return (
    <svg
      className="fp-svg"
      viewBox={`${minX} ${minY} ${width} ${height}`}
      role="img"
      aria-label={label}
    >
      {footprint.courtyard && (
        <OutlineShape
          outline={footprint.courtyard}
          className="fp-courtyard"
          hair={hair}
          dash={`${hair * 8} ${hair * 5}`}
        />
      )}
      {footprint.window && (
        <OutlineShape
          outline={footprint.window}
          className="fp-window"
          hair={hair}
          dash={`${hair * 3} ${hair * 3}`}
        />
      )}
      {asArray(footprint.pads).map((placement, i) => {
        const x = mm(placement?.x);
        const y = mm(placement?.y);
        const rotate = placement?.rotate ?? 0;
        const def = pads.get(placement?.pad);
        const transform =
          rotate !== 0 ? `translate(${x} ${y}) rotate(${-rotate})` : `translate(${x} ${y})`;
        return (
          <g key={`${placement?.number}-${i}`} className="fp-pad" transform={transform}>
            <title>{`Pad ${placement?.number} · ${shortName(placement?.pad)}`}</title>
            {def ? (
              <PadCopper pad={def} hair={hair} />
            ) : (
              <circle className="fp-copper-stroke" r={0.4} strokeWidth={hair * 2} />
            )}
          </g>
        );
      })}
      {asArray(footprint.mount_holes).map((hole, i) => {
        const x = mm(hole?.x);
        const y = mm(hole?.y);
        const cls = hole?.plating === "plated" ? "fp-hole fp-plated" : "fp-hole fp-npth";
        return (
          <g key={`${hole?.number}-${i}`} className="fp-mount">
            <title>{`Mount hole ${hole?.number} (${hole?.plating})`}</title>
            {hole?.shape === "rect" || hole?.shape === "oval" ? (
              (() => {
                const w = mm(hole.size?.[0]);
                const h = mm(hole.size?.[1]);
                const rx = hole.shape === "oval" ? Math.min(w, h) / 2 : 0;
                return (
                  <rect
                    className={cls}
                    x={x - w / 2}
                    y={y - h / 2}
                    width={w}
                    height={h}
                    rx={rx}
                    ry={rx}
                    strokeWidth={hair * 2}
                  />
                );
              })()
            ) : (
              <circle
                className={cls}
                cx={x}
                cy={y}
                r={mm(hole?.diameter) / 2}
                strokeWidth={hair * 2}
              />
            )}
          </g>
        );
      })}
      {asArray(footprint.silk).map((g, i) => (
        <SilkShape key={`${g?.kind}-${i}`} g={g} hair={hair} />
      ))}
      {footprint.silkscreen_ref && (
        <text
          className="fp-ref"
          x={mm(footprint.silkscreen_ref.at?.[0])}
          y={mm(footprint.silkscreen_ref.at?.[1])}
          fontSize={textMm}
          textAnchor="middle"
          dominantBaseline="central"
        >
          REF**
        </text>
      )}
      <g className="fp-scalebar-g" aria-hidden="true">
        <line
          className="fp-scalebar"
          x1={barX}
          y1={barY}
          x2={barX + barMm}
          y2={barY}
          strokeWidth={hair * 1.5}
        />
        <line
          className="fp-scalebar"
          x1={barX}
          y1={barY - tick}
          x2={barX}
          y2={barY + tick}
          strokeWidth={hair * 1.5}
        />
        <line
          className="fp-scalebar"
          x1={barX + barMm}
          y1={barY - tick}
          x2={barX + barMm}
          y2={barY + tick}
          strokeWidth={hair * 1.5}
        />
        <text
          className="fp-scalebar-label"
          x={barX + barMm + textMm * 0.6}
          y={barY}
          fontSize={textMm * 0.9}
          dominantBaseline="central"
        >
          {`${barMm} mm`}
        </text>
      </g>
    </svg>
  );
}
