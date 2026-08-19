// The package API explorer (docs/apidocs.md): kind and module navigation
// over the schema_version 1 document, per-kind item pages, and the SVG
// previews. The document is publisher-derived content — every string in it
// renders as React text or an SVG attribute, extending the Markdown
// renderer's no-raw-HTML rule to this whole surface.

import React, { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { Link, useNavigate } from "@tanstack/react-router";
import type { UseQueryResult } from "@tanstack/react-query";
import {
  docUrl,
  useDocText,
  type ApiDocs,
  type ApiDocsImpl,
  type ApiDocsItem,
  type DeviceDoc,
  type FnDoc,
  type FootprintDoc,
  type PadDoc,
} from "./api";
import { Icon, StatePanel } from "./components";
import {
  asArray,
  avlColumns,
  classifyFq,
  filterItems,
  fnSignature,
  itemSummary,
  itemsByFq,
  kindCounts,
  moduleGroups,
  padFacts,
  padsByFq,
  partsForDevice,
  pinsForVariant,
  specsForVariant,
} from "./apidocs-model";
import { FootprintPreview, SymbolPreview } from "./preview";

const LIST_CAP = 200;
const RAIL_MODULE_CAP = 100;

export interface ApiSearchState {
  item?: string;
  q?: string;
  kind?: string;
}

type ApiNav = (
  next: { item?: string | null; q?: string | null; kind?: string | null },
  replace?: boolean,
) => void;

/// The API tab's content: loading/error/absent states around the explorer.
export function ApiExplorer({
  name,
  version,
  versionParam,
  query,
  search,
}: {
  name: string;
  version: string;
  versionParam?: string;
  query: UseQueryResult<ApiDocs | null>;
  search: ApiSearchState;
}) {
  if (query.isError) {
    return (
      <StatePanel
        tone="error"
        icon="search"
        title="Could not load the API documentation"
        action={
          <button className="button button-secondary" onClick={() => query.refetch()}>
            Try again
          </button>
        }
      >
        {(query.error as Error).message}
      </StatePanel>
    );
  }
  if (query.data === undefined && query.isLoading) {
    return (
      <div className="document-loading" role="status">
        <span className="skeleton skeleton-title" />
        <span className="skeleton skeleton-copy" />
        <span className="skeleton skeleton-copy" />
        <span className="sr-only">Loading API documentation…</span>
      </div>
    );
  }
  if (!query.data) {
    return (
      <StatePanel title="No API documentation for this release" icon="document">
        The publisher has not uploaded extracted API docs for {version}. They can backfill
        one with <code>cohdl docs --publish</code>.
      </StatePanel>
    );
  }
  return (
    <Explorer
      name={name}
      version={version}
      versionParam={versionParam}
      doc={query.data}
      search={search}
    />
  );
}

function Explorer({
  name,
  version,
  versionParam,
  doc,
  search,
}: {
  name: string;
  version: string;
  versionParam?: string;
  doc: ApiDocs;
  search: ApiSearchState;
}) {
  const navigate = useNavigate();
  const items = useMemo(() => asArray(doc.items), [doc]);
  const foreign = useMemo(() => asArray(doc.foreign), [doc]);
  const impls = useMemo(() => asArray(doc.impls), [doc]);
  const [showPrivate, setShowPrivate] = useState(false);
  const [filterText, setFilterText] = useState(search.q ?? "");

  useEffect(() => setFilterText(search.q ?? ""), [search.q]);

  const nav: ApiNav = (next, replace = false) => {
    const item = next.item === null ? undefined : (next.item ?? search.item);
    const q = next.q === null ? undefined : (next.q ?? search.q);
    const kind = next.kind === null ? undefined : (next.kind ?? search.kind);
    navigate({
      to: "/package/$",
      params: { _splat: name },
      search: {
        version: versionParam,
        // `item` alone already selects the API tab; keep the URL canonical.
        view: item ? undefined : "api",
        item,
        q,
        kind,
      },
      replace,
    });
  };

  const byFq = useMemo(() => itemsByFq(doc), [doc]);
  const pads = useMemo(() => padsByFq(doc), [doc]);
  const qFiltered = useMemo(
    () => filterItems(items, { q: search.q, showPrivate }),
    [items, search.q, showPrivate],
  );
  const visible = useMemo(
    () => (search.kind ? qFiltered.filter((item) => item.kind === search.kind) : qFiltered),
    [qFiltered, search.kind],
  );
  const counts = useMemo(() => kindCounts(qFiltered), [qFiltered]);
  const groups = useMemo(() => moduleGroups(visible), [visible]);

  const localSelected = search.item ? items.find((i) => i?.fq === search.item) : undefined;
  const selected = localSelected ?? (search.item ? foreign.find((i) => i?.fq === search.item) : undefined);

  return (
    <div className="api-explorer">
      <nav className="api-rail" aria-label={`API of ${name} ${version}`}>
        <div className="api-filter">
          <Icon name="search" size={15} />
          <input
            id="api-item-filter"
            name="api-item-filter"
            aria-label="Filter API items by name"
            placeholder="Filter items…"
            value={filterText}
            onChange={(event) => {
              const value = event.target.value;
              setFilterText(value);
              nav({ q: value.trim() ? value : null, item: null }, true);
            }}
          />
        </div>
        <div className="api-rail-section api-kinds">
          <div className="doc-nav-heading">
            <span>Kinds</span>
            <span>{qFiltered.length}</span>
          </div>
          <button
            type="button"
            className="api-kind-button"
            aria-pressed={!search.kind}
            onClick={() => nav({ kind: null, item: null })}
          >
            <span className="api-kind-label">all</span>
            <span>{qFiltered.length}</span>
          </button>
          {counts.map(({ kind, count }) => (
            <button
              type="button"
              key={kind}
              className="api-kind-button"
              aria-pressed={search.kind === kind}
              onClick={() => nav({ kind: search.kind === kind ? null : kind, item: null })}
            >
              <span className="api-kind-label">
                <span className={`kind-dot kind-${kind}`} aria-hidden="true" />
                {kind}
              </span>
              <span>{count}</span>
            </button>
          ))}
        </div>
        <div className="api-rail-section api-modules">
          <div className="doc-nav-heading">
            <span>Modules</span>
            <span>{groups.length}</span>
          </div>
          {groups.map((group) => (
            <ModuleGroupDetails
              key={group.module}
              group={group}
              defaultOpen={groups.length === 1}
              activeItem={search.item}
              nav={nav}
            />
          ))}
        </div>
        <label className="api-private-toggle">
          <input
            type="checkbox"
            checked={showPrivate}
            onChange={(event) => setShowPrivate(event.target.checked)}
          />
          <span>Show private items</span>
        </label>
        <p className="api-generator">{doc.generator}</p>
      </nav>

      <div className="api-content">
        {selected ? (
          <ItemDetail
            item={selected}
            isForeign={!localSelected}
            doc={doc}
            byFq={byFq}
            pads={pads}
            impls={impls}
            name={name}
            version={version}
            nav={nav}
          />
        ) : search.item ? (
          <StatePanel
            icon="search"
            title="This item is not in the API documentation"
            action={
              <button className="button button-secondary" onClick={() => nav({ item: null })}>
                Back to the item list
              </button>
            }
          >
            <code>{search.item}</code> is not a declaration of {name} {version}.
          </StatePanel>
        ) : (
          <ItemList items={visible} nav={nav} />
        )}
      </div>
    </div>
  );
}

/// One collapsible module group in the rail. Children only mount while the
/// group is open — a closed `<details>` would otherwise still put every
/// row in the DOM, and one real package has ~9k parts.
function ModuleGroupDetails({
  group,
  defaultOpen,
  activeItem,
  nav,
}: {
  group: { module: string; items: ApiDocsItem[] };
  defaultOpen: boolean;
  activeItem: string | undefined;
  nav: ApiNav;
}) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <details
      open={open}
      onToggle={(event) => setOpen((event.target as HTMLDetailsElement).open)}
    >
      <summary>
        <span>{group.module}</span>
        <span>{group.items.length}</span>
      </summary>
      {open && (
        <div className="api-module-items">
          {group.items.slice(0, RAIL_MODULE_CAP).map((item) => (
            <button
              type="button"
              key={item.fq}
              className={`doc-link${item.fq === activeItem ? " doc-link-active" : ""}${
                item.pub ? "" : " api-private"
              }`}
              aria-pressed={item.fq === activeItem}
              onClick={() => nav({ item: item.fq })}
            >
              <span className={`kind-dot kind-${item.kind}`} aria-hidden="true" />
              <span>{item.name}</span>
            </button>
          ))}
          {group.items.length > RAIL_MODULE_CAP && (
            <p className="api-rail-more">
              +{group.items.length - RAIL_MODULE_CAP} more — refine the filter
            </p>
          )}
        </div>
      )}
    </details>
  );
}

function ItemList({ items, nav }: { items: ApiDocsItem[]; nav: ApiNav }) {
  if (items.length === 0) {
    return (
      <StatePanel icon="search" title="No items match this filter">
        Try another name fragment, or clear the kind filter.
      </StatePanel>
    );
  }
  return (
    <div className="api-item-list">
      {items.slice(0, LIST_CAP).map((item) => (
        <button
          type="button"
          key={item.fq}
          className={`api-item-row${item.pub ? "" : " api-private"}`}
          onClick={() => nav({ item: item.fq })}
        >
          <span className={`kind-pill kind-${item.kind}`}>{item.kind}</span>
          <span className="api-item-main">
            <code>{item.name}</code>
            <small>{itemSummary(item)}</small>
          </span>
          <span className="api-item-module">{item.module}</span>
        </button>
      ))}
      {items.length > LIST_CAP && (
        <p className="results-note">
          Showing the first {LIST_CAP} of {items.length} items — refine the filter to see the
          rest.
        </p>
      )}
    </div>
  );
}

// --- shared detail pieces ---------------------------------------------------

/// An fq reference: a link into this explorer for local items, a link to the
/// owning package's page for dependency roots, plain text otherwise.
function FqRef({ fq, doc, nav }: { fq: string; doc: ApiDocs; nav: ApiNav }) {
  const target = classifyFq(fq, doc);
  if (target.kind === "local") {
    return (
      <button type="button" className="api-fq-link" onClick={() => nav({ item: fq })}>
        <code>{fq}</code>
      </button>
    );
  }
  if (target.kind === "dependency") {
    return (
      <Link className="api-fq-link" to="/package/$" params={{ _splat: target.package }}>
        <code>{fq}</code>
      </Link>
    );
  }
  return <code>{fq}</code>;
}

function Section({
  title,
  count,
  children,
}: {
  title: string;
  count?: number | string;
  children: ReactNode;
}) {
  return (
    <section className="api-section">
      <div className="api-section-heading">
        <h4>{title}</h4>
        {count !== undefined && <span className="panel-count">{count}</span>}
      </div>
      {children}
    </section>
  );
}

/// The declaration's source file, from the same immutable tar the document
/// index reads, with the item's line highlighted and scrolled into view.
function SourceView({
  pkg,
  version,
  file,
  line,
}: {
  pkg: string;
  version: string;
  file: string;
  line: number;
}) {
  const text = useDocText(pkg, version, file);
  const pre = useRef<HTMLPreElement>(null);
  const hit = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    const container = pre.current;
    const target = hit.current;
    if (container && target) {
      container.scrollTop = target.offsetTop - container.clientHeight / 2;
    }
  }, [text.data, line]);

  if (text.isPending) {
    return (
      <div className="document-loading" role="status">
        <span className="skeleton skeleton-copy" />
        <span className="skeleton skeleton-copy" />
        <span className="sr-only">Loading {file}…</span>
      </div>
    );
  }
  if (text.isError) {
    return (
      <StatePanel tone="error" title={`Could not load ${file}`}>
        {(text.error as Error).message}
      </StatePanel>
    );
  }
  const lines = (text.data ?? "").replace(/\r\n?/g, "\n").split("\n");
  return (
    <div className="source-view" role="region" aria-label={`Source of ${file}`}>
      <div className="source-view-heading">
        <Icon name="document" size={14} />
        <span>
          {file}:{line}
        </span>
      </div>
      <pre ref={pre}>
        <code>
          {lines.map((content, index) => (
            <span
              key={index}
              ref={index + 1 === line ? hit : undefined}
              className={index + 1 === line ? "src-line src-line-hit" : "src-line"}
            >
              <span className="src-no" aria-hidden="true">
                {index + 1}
              </span>
              {content}
              {"\n"}
            </span>
          ))}
        </code>
      </pre>
    </div>
  );
}

interface DetailContext {
  doc: ApiDocs;
  byFq: Map<string, ApiDocsItem>;
  pads: Map<string, PadDoc>;
  impls: ApiDocsImpl[];
  name: string;
  version: string;
  nav: ApiNav;
}

function ItemDetail({
  item,
  isForeign,
  ...ctx
}: DetailContext & { item: ApiDocsItem; isForeign: boolean }) {
  const [showSource, setShowSource] = useState(false);
  useEffect(() => setShowSource(false), [item.fq]);
  const docLinks = asArray(item.docs);

  return (
    <article className="api-detail">
      <header className="api-detail-heading">
        <button type="button" className="text-button" onClick={() => ctx.nav({ item: null })}>
          Back to the item list
        </button>
        <div className="api-detail-title">
          <span className={`kind-pill kind-${item.kind}`}>{item.kind}</span>
          {!item.pub && <span className="api-private-badge">private</span>}
          {isForeign && <span className="api-private-badge">from a dependency</span>}
        </div>
        <h3>
          <code>{item.fq}</code>
        </h3>
        {item.intent && <p className="api-intent">{item.intent}</p>}
        <p className="api-detail-meta">
          <span>
            {item.file}:{item.line}
          </span>
          {!isForeign && (
            <button
              type="button"
              className="text-button"
              onClick={() => setShowSource((open) => !open)}
            >
              {showSource ? "Hide source" : "View source"}
            </button>
          )}
        </p>
      </header>
      {!isForeign && docLinks.length > 0 && (
        <div className="api-doc-links">
          {docLinks.map((doc) => (
            <a
              key={doc}
              href={docUrl(ctx.name, ctx.version, doc)}
              rel="noopener noreferrer"
              target="_blank"
            >
              <Icon name="document" size={14} />
              <span>{doc}</span>
            </a>
          ))}
        </div>
      )}
      {showSource && !isForeign && (
        <SourceView pkg={ctx.name} version={ctx.version} file={item.file} line={item.line} />
      )}
      <KindBody item={item} ctx={ctx} />
    </article>
  );
}

function KindBody({ item, ctx }: { item: ApiDocsItem; ctx: DetailContext }) {
  // A hostile document may omit the kind-named payload entirely; the header
  // above still renders, the body simply stays empty.
  switch (item.kind) {
    case "trait":
      return item.trait ? <TraitBody item={item} ctx={ctx} /> : null;
    case "device":
      return item.device ? <DeviceBody fq={item.fq} device={item.device} ctx={ctx} /> : null;
    case "fn":
      return item.fn ? (
        <CircuitBody keyword="fn" name={item.name} body={item.fn} ctx={ctx} />
      ) : null;
    case "part":
      return item.part ? <PartBody item={item} ctx={ctx} /> : null;
    case "pad":
      return item.pad ? <PadBody fq={item.fq} pad={item.pad} ctx={ctx} /> : null;
    case "footprint":
      return item.footprint ? (
        <FootprintBody fq={item.fq} footprint={item.footprint} ctx={ctx} />
      ) : null;
    case "design":
      return item.design ? (
        <CircuitBody
          keyword="design"
          name={item.name}
          body={{ nets: item.design.nets, insts: item.design.insts, calls: item.design.calls }}
          ctx={ctx}
        />
      ) : null;
    default:
      return null;
  }
}

// --- per-kind bodies --------------------------------------------------------

function TraitBody({
  item,
  ctx,
}: {
  item: Extract<ApiDocsItem, { kind: "trait" }>;
  ctx: DetailContext;
}) {
  const t = item.trait;
  const superTraits = asArray(t.super_traits);
  const pins = asArray(t.pins);
  const specs = asArray(t.specs);
  const implementors = ctx.impls.filter((impl) => impl?.trait === item.fq);
  return (
    <>
      {superTraits.length > 0 && (
        <Section title="Super traits">
          <ul className="api-ref-list">
            {superTraits.map((st) => (
              <li key={st}>
                <FqRef fq={st} doc={ctx.doc} nav={ctx.nav} />
              </li>
            ))}
          </ul>
        </Section>
      )}
      {t.designator_prefix && (
        <p className="api-fact">
          Designator prefix <code>{t.designator_prefix}</code>
        </p>
      )}
      {pins.length > 0 && (
        <Section title="Pins" count={pins.length}>
          <div className="table-wrap">
            <table className="api-table">
              <thead>
                <tr>
                  <th scope="col">Pin</th>
                  <th scope="col">Obligation</th>
                </tr>
              </thead>
              <tbody>
                {pins.map((pin) => (
                  <tr key={pin?.name}>
                    <td>
                      <code>{pin?.name}</code>
                    </td>
                    <td>{pin?.obligation}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Section>
      )}
      {specs.length > 0 && (
        <Section title="Specs" count={specs.length}>
          <div className="table-wrap">
            <table className="api-table">
              <thead>
                <tr>
                  <th scope="col">Spec</th>
                  <th scope="col">Type</th>
                </tr>
              </thead>
              <tbody>
                {specs.map((spec) => (
                  <tr key={spec?.name}>
                    <td>
                      <code>{spec?.name}</code>
                    </td>
                    <td>{spec?.type}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Section>
      )}
      <Section title="Implementors" count={implementors.length}>
        {implementors.length === 0 ? (
          <p className="empty-line">No devices in this package implement this trait.</p>
        ) : (
          <ul className="api-ref-list">
            {implementors.map((impl) => (
              <li key={impl?.device}>
                <FqRef fq={impl?.device} doc={ctx.doc} nav={ctx.nav} />
              </li>
            ))}
          </ul>
        )}
      </Section>
    </>
  );
}

/// Device sections; also reused (headerless) on part pages via the shared
/// symbol/pin rendering. `fq` keys the impls/parts reverse joins.
function DeviceBody({ fq, device, ctx }: { fq: string; device: DeviceDoc; ctx: DetailContext }) {
  const variants = asArray(device.variants);
  const [variant, setVariant] = useState<string | undefined>(variants[0]);
  useEffect(() => setVariant(asArray(device.variants)[0]), [fq, device]);

  const pins = pinsForVariant(device, variant);
  const specs = specsForVariant(device, variant);
  const generics = asArray(device.generics);
  const traits = ctx.impls.filter((impl) => impl?.device === fq);
  const parts = partsForDevice(asArray(ctx.doc.items), fq);

  return (
    <>
      {variants.length > 0 && (
        <div className="filter-pills api-variants" aria-label="Package variant">
          {variants.map((candidate) => (
            <button
              type="button"
              key={candidate}
              aria-pressed={variant === candidate}
              onClick={() => setVariant(candidate)}
            >
              {candidate}
            </button>
          ))}
        </div>
      )}
      <div className="preview-panel">
        <SymbolPreview
          label={`Schematic symbol of ${fq}${variant ? `, variant ${variant}` : ""}`}
          pins={pins}
        />
      </div>
      <p className="api-fact">
        Designator prefix <code>{device.designator_prefix}</code>
      </p>
      {generics.length > 0 && (
        <Section title="Generics" count={generics.length}>
          <div className="table-wrap">
            <table className="api-table">
              <thead>
                <tr>
                  <th scope="col">Parameter</th>
                  <th scope="col">Bound</th>
                  <th scope="col">Default</th>
                </tr>
              </thead>
              <tbody>
                {generics.map((generic) => (
                  <tr key={generic?.name}>
                    <td>
                      <code>{generic?.name}</code>
                    </td>
                    <td>
                      {generic?.bound?.unit ??
                        asArray(generic?.bound?.traits).map((t, i) => (
                          <React.Fragment key={t}>
                            {i > 0 && " + "}
                            <FqRef fq={t} doc={ctx.doc} nav={ctx.nav} />
                          </React.Fragment>
                        ))}
                    </td>
                    <td>{generic?.default ?? "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Section>
      )}
      <Section title="Pins" count={pins.length}>
        <div className="table-wrap">
          <table className="api-table">
            <thead>
              <tr>
                <th scope="col">Pin</th>
                <th scope="col">Numbers</th>
                <th scope="col">Role</th>
                <th scope="col">Obligation</th>
              </tr>
            </thead>
            <tbody>
              {pins.map((pin) => (
                <tr key={pin?.name}>
                  <td>
                    <code>{pin?.name}</code>
                  </td>
                  <td>{asArray(pin?.numbers).join(", ")}</td>
                  <td>{pin?.role}</td>
                  <td>{pin?.obligation}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Section>
      {specs.length > 0 && (
        <Section title="Specs" count={specs.length}>
          <div className="table-wrap">
            <table className="api-table">
              <thead>
                <tr>
                  <th scope="col">Spec</th>
                  <th scope="col">Value</th>
                </tr>
              </thead>
              <tbody>
                {specs.map((spec) => (
                  <tr key={spec?.name}>
                    <td>
                      <code>{spec?.name}</code>
                    </td>
                    <td>
                      {spec?.generic !== undefined ? (
                        <code>{spec?.generic}</code>
                      ) : (
                        (spec?.value ?? "—")
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Section>
      )}
      {traits.length > 0 && (
        <Section title="Implemented traits" count={traits.length}>
          <ul className="api-ref-list">
            {traits.map((impl) => (
              <li key={impl?.trait}>
                <FqRef fq={impl?.trait} doc={ctx.doc} nav={ctx.nav} />
              </li>
            ))}
          </ul>
        </Section>
      )}
      {parts.length > 0 && (
        <Section title="Parts bound to this device" count={parts.length}>
          <ul className="api-ref-list">
            {parts.map((part) => (
              <li key={part.fq}>
                <FqRef fq={part.fq} doc={ctx.doc} nav={ctx.nav} />
              </li>
            ))}
          </ul>
        </Section>
      )}
    </>
  );
}

/// fn and design pages share the signature + insts/calls/nets summary; a
/// design body is an `FnDoc` with no generics or params.
function CircuitBody({
  keyword,
  name,
  body,
  ctx,
}: {
  keyword: "fn" | "design";
  name: string;
  body: FnDoc;
  ctx: DetailContext;
}) {
  const insts = asArray(body.insts);
  const calls = asArray(body.calls);
  return (
    <>
      <pre className="code-panel api-signature">
        <code>{fnSignature(keyword, name, body)}</code>
      </pre>
      <p className="api-fact">
        {body.nets} net statement{body.nets === 1 ? "" : "s"} in the body
      </p>
      {insts.length > 0 && (
        <Section title="Instances" count={insts.length}>
          <div className="table-wrap">
            <table className="api-table">
              <thead>
                <tr>
                  <th scope="col">Name</th>
                  <th scope="col">Device</th>
                  <th scope="col">Arguments</th>
                  <th scope="col">Variant</th>
                </tr>
              </thead>
              <tbody>
                {insts.map((inst, index) => (
                  <tr key={`${inst?.name}-${index}`}>
                    <td>
                      <code>{inst?.name}</code>
                    </td>
                    <td>
                      <FqRef fq={inst?.type} doc={ctx.doc} nav={ctx.nav} />
                    </td>
                    <td>{Array.isArray(inst?.args) ? inst?.args?.join(", ") : "—"}</td>
                    <td>{inst?.variant ?? "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Section>
      )}
      {calls.length > 0 && (
        <Section title="Calls" count={calls.length}>
          <ul className="api-ref-list">
            {calls.map((call) => (
              <li key={call}>
                <FqRef fq={call} doc={ctx.doc} nav={ctx.nav} />
              </li>
            ))}
          </ul>
        </Section>
      )}
    </>
  );
}

function PartBody({
  item,
  ctx,
}: {
  item: Extract<ApiDocsItem, { kind: "part" }>;
  ctx: DetailContext;
}) {
  const part = item.part;
  const deviceItem = ctx.byFq.get(part.device);
  const device = deviceItem?.kind === "device" ? deviceItem.device : undefined;
  const columns = avlColumns(part);
  const entries = [
    { label: "Primary", entry: part.primary },
    ...asArray(part.alts).map((entry, index) => ({ label: `Alt ${index + 1}`, entry })),
  ];
  const footprintFq = part.primary?.footprint;
  const footprintItem = footprintFq ? ctx.byFq.get(footprintFq) : undefined;
  const footprint = footprintItem?.kind === "footprint" ? footprintItem.footprint : undefined;

  return (
    <>
      <p className="api-fact">
        Device <FqRef fq={part.device} doc={ctx.doc} nav={ctx.nav} />
        {Array.isArray(part.args) && <code className="api-args">({part.args.join(", ")})</code>}
        {part.variant && (
          <>
            {" "}
            variant <code>{part.variant}</code>
          </>
        )}
      </p>
      {device && (
        <div className="preview-panel">
          <SymbolPreview
            label={`Schematic symbol of ${part.device}${part.variant ? `, variant ${part.variant}` : ""}`}
            pins={pinsForVariant(device, part.variant ?? device.variants?.[0])}
          />
        </div>
      )}
      <Section title="Approved vendors" count={entries.length}>
        <div className="table-wrap">
          <table className="api-table">
            <thead>
              <tr>
                <th scope="col">Entry</th>
                {columns.map((column) => (
                  <th scope="col" key={column}>
                    {column}
                  </th>
                ))}
                <th scope="col">Footprint</th>
              </tr>
            </thead>
            <tbody>
              {entries.map(({ label, entry }) => (
                <tr key={label}>
                  <td>{label}</td>
                  {columns.map((column) => (
                    <td key={column}>
                      {asArray(entry?.fields).find((field) => field?.name === column)?.value ??
                        "—"}
                    </td>
                  ))}
                  <td>
                    {entry?.footprint ? (
                      <FqRef fq={entry.footprint} doc={ctx.doc} nav={ctx.nav} />
                    ) : (
                      "—"
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Section>
      {footprintFq &&
        (footprint ? (
          footprint.placeholder ? (
            <StatePanel title="The footprint is a placeholder" icon="cube">
              <FqRef fq={footprintFq} doc={ctx.doc} nav={ctx.nav} /> has an empty stage-one
              body — no geometry to preview yet.
            </StatePanel>
          ) : (
            <Section title="Footprint">
              <p className="api-fact">
                <FqRef fq={footprintFq} doc={ctx.doc} nav={ctx.nav} />
              </p>
              <div className="preview-panel">
                <FootprintPreview
                  footprint={footprint}
                  pads={ctx.pads}
                  label={`Footprint ${footprintFq}`}
                />
              </div>
            </Section>
          )
        ) : (
          <p className="api-fact">
            Footprint <FqRef fq={footprintFq} doc={ctx.doc} nav={ctx.nav} />
          </p>
        ))}
    </>
  );
}

function FootprintBody({
  fq,
  footprint,
  ctx,
}: {
  fq: string;
  footprint: FootprintDoc;
  ctx: DetailContext;
}) {
  if (footprint.placeholder) {
    return (
      <StatePanel title="Stage-one placeholder" icon="cube">
        This footprint has an empty body — pad geometry has not been authored yet, so there
        is nothing to preview.
      </StatePanel>
    );
  }
  const placements = asArray(footprint.pads);
  const holes = asArray(footprint.mount_holes);
  const markers = asArray(footprint.markers);
  return (
    <>
      <div className="preview-panel">
        <FootprintPreview footprint={footprint} pads={ctx.pads} label={`Footprint ${fq}`} />
      </div>
      <Section title="Pad placements" count={placements.length}>
        <div className="table-wrap">
          <table className="api-table">
            <thead>
              <tr>
                <th scope="col">Pad</th>
                <th scope="col">Definition</th>
                <th scope="col">X (mm)</th>
                <th scope="col">Y (mm)</th>
                <th scope="col">Rotation</th>
              </tr>
            </thead>
            <tbody>
              {placements.map((placement, index) => (
                <tr key={`${placement?.number}-${index}`}>
                  <td>{placement?.number}</td>
                  <td>
                    <FqRef fq={placement?.pad} doc={ctx.doc} nav={ctx.nav} />
                  </td>
                  <td>{placement?.x}</td>
                  <td>{placement?.y}</td>
                  <td>{placement?.rotate !== undefined ? `${placement?.rotate}°` : "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Section>
      {holes.length > 0 && (
        <Section title="Mount holes" count={holes.length}>
          <div className="table-wrap">
            <table className="api-table">
              <thead>
                <tr>
                  <th scope="col">Hole</th>
                  <th scope="col">Plating</th>
                  <th scope="col">Shape</th>
                  <th scope="col">X (mm)</th>
                  <th scope="col">Y (mm)</th>
                  <th scope="col">Size (mm)</th>
                </tr>
              </thead>
              <tbody>
                {holes.map((hole, index) => (
                  <tr key={`${hole?.number}-${index}`}>
                    <td>{hole?.number}</td>
                    <td>{hole?.plating}</td>
                    <td>{hole?.shape}</td>
                    <td>{hole?.x}</td>
                    <td>{hole?.y}</td>
                    <td>
                      {hole?.diameter !== undefined
                        ? hole?.diameter
                        : asArray(hole?.size).join(" × ")}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Section>
      )}
      {markers.length > 0 && (
        <Section title="Semantic markers" count={markers.length}>
          <ul className="api-ref-list">
            {markers.map((marker, index) => (
              <li key={index}>
                <code>{marker?.kind}</code>
                {marker?.pad !== undefined && <> near pad {marker?.pad}</>}
                {marker?.cathode_pin !== undefined && <> cathode pin {marker?.cathode_pin}</>}
                {" · "}
                {marker?.shape}
              </li>
            ))}
          </ul>
        </Section>
      )}
    </>
  );
}

function PadBody({ fq, pad, ctx }: { fq: string; pad: PadDoc; ctx: DetailContext }) {
  // A synthetic single-pad footprint gives the mini preview for free.
  const solo: FootprintDoc = {
    placeholder: false,
    pads: [{ number: "1", pad: fq, x: "0", y: "0" }],
  };
  const facts = padFacts(pad);
  return (
    <>
      <div className="preview-panel">
        <FootprintPreview footprint={solo} pads={ctx.pads} label={`Pad ${fq}`} />
      </div>
      <Section title="Pad definition" count={facts.length}>
        <div className="table-wrap">
          <table className="api-table">
            <tbody>
              {facts.map((fact, index) => (
                <tr key={`${fact.name}-${index}`}>
                  <th scope="row">{fact.name}</th>
                  <td>{fact.value}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Section>
    </>
  );
}
