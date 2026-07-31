import { Link } from "@tanstack/react-router";
import { useEffect, useRef, useState, type ReactNode } from "react";
import type { SearchRow } from "./api";

export type IconName =
  | "arrow"
  | "check"
  | "copy"
  | "cube"
  | "document"
  | "external"
  | "hash"
  | "search"
  | "shield"
  | "terminal";

export function Icon({
  name,
  size = 18,
  className,
}: {
  name: IconName;
  size?: number;
  className?: string;
}) {
  const paths: Record<IconName, ReactNode> = {
    arrow: (
      <>
        <path d="M5 12h14" />
        <path d="m13 6 6 6-6 6" />
      </>
    ),
    check: <path d="m5 12 4 4L19 6" />,
    copy: (
      <>
        <rect width="13" height="13" x="9" y="9" rx="2" />
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
      </>
    ),
    cube: (
      <>
        <path d="m21 8-9 5-9-5" />
        <path d="m3 8 9-5 9 5v8l-9 5-9-5Z" />
        <path d="M12 13v8" />
      </>
    ),
    document: (
      <>
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z" />
        <path d="M14 2v6h6M8 13h8M8 17h6" />
      </>
    ),
    external: (
      <>
        <path d="M15 3h6v6" />
        <path d="m10 14 11-11" />
        <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
      </>
    ),
    hash: (
      <>
        <path d="M4 9h16M3 15h16M10 3 8 21M16 3l-2 18" />
      </>
    ),
    search: (
      <>
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-4-4" />
      </>
    ),
    shield: (
      <>
        <path d="M20 13c0 5-3.5 7.5-8 9-4.5-1.5-8-4-8-9V5l8-3 8 3Z" />
        <path d="m9 12 2 2 4-4" />
      </>
    ),
    terminal: (
      <>
        <path d="m4 17 6-6-6-6M12 19h8" />
      </>
    ),
  };

  return (
    <svg
      aria-hidden="true"
      className={className}
      fill="none"
      height={size}
      viewBox="0 0 24 24"
      width={size}
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.8"
    >
      {paths[name]}
    </svg>
  );
}

export function BrandMark({ size = 34 }: { size?: number }) {
  return (
    <svg
      aria-hidden="true"
      className="brand-mark"
      height={size}
      viewBox="0 0 40 40"
      width={size}
      fill="none"
    >
      <rect x="7.5" y="7.5" width="25" height="25" rx="7" />
      <path d="M14 16.5h4.5l3-4.5M14 23.5h4.5l3 4.5M21.5 12v16M21.5 16.5H27M21.5 23.5H27" />
      <circle cx="14" cy="16.5" r="1.3" />
      <circle cx="14" cy="23.5" r="1.3" />
      <circle cx="27" cy="16.5" r="1.3" />
      <circle cx="27" cy="23.5" r="1.3" />
    </svg>
  );
}

const TIER_LABELS: Record<string, string> = {
  official: "Official",
  brand: "Verified manufacturer",
  contrib: "Community",
};

export function TierBadge({ tier }: { tier: string }) {
  return (
    <span className={`tier-badge tier-${tier}`}>
      <span className="tier-dot" aria-hidden="true" />
      {TIER_LABELS[tier] ?? tier}
    </span>
  );
}

export function formatDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(date);
}

export function formatSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return `${bytes} B`;
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unit = units[0];
  for (let index = 1; value >= 1024 && index < units.length; index++) {
    value /= 1024;
    unit = units[index];
  }
  return `${value >= 10 ? value.toFixed(0) : value.toFixed(1)} ${unit}`;
}

export function CopyButton({
  value,
  label = "Copy",
  compact = false,
}: {
  value: string;
  label?: string;
  compact?: boolean;
}) {
  const [status, setStatus] = useState<"idle" | "copied" | "error">("idle");
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    [],
  );

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setStatus("copied");
    } catch {
      setStatus("error");
    }
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => setStatus("idle"), 1800);
  };

  const text = status === "copied" ? "Copied" : status === "error" ? "Copy failed" : label;
  return (
    <button
      type="button"
      className={`copy-button${compact ? " copy-button-compact" : ""}`}
      onClick={copy}
      aria-label={label}
    >
      <Icon name={status === "copied" ? "check" : "copy"} size={compact ? 15 : 17} />
      <span>{text}</span>
      <span className="sr-only" aria-live="polite">
        {status === "copied" ? "Copied to clipboard" : status === "error" ? "Copy failed" : ""}
      </span>
    </button>
  );
}

export function CommandBox({
  command,
  label,
  compact = false,
}: {
  command: string;
  label?: string;
  compact?: boolean;
}) {
  return (
    <div className={`command-block${compact ? " command-block-compact" : ""}`}>
      {label && <span className="command-label">{label}</span>}
      <div className="command-row">
        <Icon name="terminal" size={17} />
        <code>{command}</code>
        <CopyButton value={command} compact label="Copy" />
      </div>
    </div>
  );
}

export function StatePanel({
  tone = "neutral",
  icon = "cube",
  title,
  children,
  action,
}: {
  tone?: "neutral" | "error" | "success";
  icon?: IconName;
  title: string;
  children?: ReactNode;
  action?: ReactNode;
}) {
  return (
    <div
      className={`state-panel state-${tone}`}
      role={tone === "error" ? "alert" : tone === "success" ? "status" : undefined}
    >
      <div className="state-icon">
        <Icon name={icon} size={20} />
      </div>
      <div>
        <strong>{title}</strong>
        {children && <div className="state-copy">{children}</div>}
        {action && <div className="state-action">{action}</div>}
      </div>
    </div>
  );
}

export function LoadingRows({ count = 3 }: { count?: number }) {
  return (
    <div className="loading-stack" role="status" aria-label="Loading packages">
      <span className="sr-only">Loading packages…</span>
      {Array.from({ length: count }, (_, index) => (
        <div className="package-card skeleton-card" key={index} aria-hidden="true">
          <span className="skeleton skeleton-icon" />
          <span className="skeleton-lines">
            <span className="skeleton skeleton-title" />
            <span className="skeleton skeleton-copy" />
            <span className="skeleton skeleton-meta" />
          </span>
        </div>
      ))}
    </div>
  );
}

export function PackageCard({ pkg }: { pkg: SearchRow }) {
  return (
    <Link className="package-card" to="/package/$" params={{ _splat: pkg.name }}>
      <span className="package-glyph">
        <Icon name="cube" size={21} />
      </span>
      <span className="package-card-body">
        <span className="package-card-heading">
          <code>{pkg.name}</code>
          <TierBadge tier={pkg.tier} />
        </span>
        <span className="package-card-description">
          {pkg.description ?? "No package description has been published yet."}
        </span>
        <span className="package-card-meta">
          <span>v{pkg.latest}</span>
          <span aria-hidden="true">·</span>
          <time dateTime={pkg.updated}>Updated {formatDate(pkg.updated)}</time>
        </span>
      </span>
      <Icon name="arrow" className="package-card-arrow" size={19} />
    </Link>
  );
}
