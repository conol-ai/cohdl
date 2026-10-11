export const proposalStatuses = ['prepared', 'pending', 'failed', 'proposed', 'rejected', 'validated', 'applied', 'reverted', 'interrupted'] as const
export type ProposalStatus = typeof proposalStatuses[number]
export type ProposalKind = 'group' | 'plan' | 'connect'
export type Freshness = 'fresh' | 'stale' | 'unknown'
export interface ProposalSummary { id: string; title: string; kind: ProposalKind | null; status: ProposalStatus; freshness: Freshness; issues: string[] }
export interface SnapshotEntry { path: string; sha256: string | null }
export interface ProcessEvidence { command: string[]; cwd: string; started_at: string; finished_at: string | null; exit_code: number | null; signal: string | null; error: string | null }
export interface ProposalCitation { document: string | null; page: number | null; section: string | null; source: { path: string; line: number; sha256: string } | null }
export interface Proposal {
  schema: 1; kind: ProposalKind; title: string; rationale: string; uncertainty: string; citations: ProposalCitation[]
  changes: { path: string; before_sha256: string | null; old: string; new: string }[]
}
export interface Validation {
  schema: 1; id: string; proposal_sha256: string; bundle_sha256: string; started_at: string; finished_at: string; decision: 'PASS' | 'FAIL'; issues: string[]; before: SnapshotEntry[]; after: SnapshotEntry[]
  compiler: (ProcessEvidence & { result: { schema_version: 1; verdict: 'pass' | 'fail'; diagnostics: Record<string, unknown>[] } | null }) | null
}
export interface Operation { started_at: string; finished_at: string | null; outcome: string; error: string | null }
export interface ProposalDetail {
  schema: 1; status: ProposalStatus; freshness: Freshness; issues: string[]; artifacts: { name: string; url: string }[]
  run: {
    schema?: 1; id: string; kind?: ProposalKind; created_at?: string; bundle_sha256?: string; proposal_sha256?: string | null
    provider?: ProcessEvidence & { engine: 'codex'; heartbeat_at: string | null }
    proposal: Proposal | null
    bundle: { design: string; selection: { schema: 1; instances: string[]; groups: { view: string; region: string }[] }; snapshot: SnapshotEntry[] } | null
    validation: Validation | null
    receipt: { schema: 1; id: string; proposal_sha256: string; bundle_sha256: string; validation_sha256: string; apply_validation: Validation | null; before: SnapshotEntry[]; after: SnapshotEntry[]; files: { path: string; before_sha256: string | null; after_sha256: string; before: string | null; after: string }[]; apply: Operation; revert: Operation | null } | null
  }
}
export interface ProposalResource {
  runs: ProposalSummary[]; selected: string; detail: ProposalDetail | null; loading: boolean; error: string
  detailLoading: boolean; detailError: string; select: (id: string) => void; refresh: () => void
}

const object = (v: unknown): v is Record<string, unknown> => !!v && typeof v === 'object' && !Array.isArray(v)
const string = (v: unknown): v is string => typeof v === 'string'
const nullableString = (v: unknown) => v === null || string(v)
const strings = (v: unknown): v is string[] => Array.isArray(v) && v.length <= 10000 && v.every(string)
const id = (v: unknown): v is string => string(v) && /^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/.test(v)
const path = (v: unknown) => string(v) && v.length <= 1024 && /^[\x20-\x7e]+$/.test(v) && !/[\\:%]/.test(v) && v.split('/').every(p => !!p && !p.startsWith('.'))
const kind = (v: unknown) => ['group', 'plan', 'connect'].includes(String(v))
const hash = (v: unknown) => string(v) && /^[a-f0-9]{64}$/.test(v)
const nullableHash = (v: unknown) => v === null || hash(v)
const state = (v: Record<string, unknown>) => proposalStatuses.includes(v.status as ProposalStatus) && ['fresh', 'stale', 'unknown'].includes(String(v.freshness)) && strings(v.issues)
const array = (v: unknown, limit: number, check: (v: unknown) => boolean) => Array.isArray(v) && v.length <= limit && v.every(check)
const snapshot = (v: unknown) => array(v, 2048, e => object(e) && path(e.path) && nullableHash(e.sha256))
const child = (v: unknown) => object(v) && strings(v.command) && string(v.cwd) && string(v.started_at) && nullableString(v.finished_at) && (v.exit_code === null || Number.isSafeInteger(v.exit_code)) && nullableString(v.signal) && nullableString(v.error)
const operation = (v: unknown) => object(v) && string(v.started_at) && nullableString(v.finished_at) && string(v.outcome) && nullableString(v.error)
function validation(v: unknown): boolean {
  return object(v) && v.schema === 1 && id(v.id) && hash(v.proposal_sha256) && hash(v.bundle_sha256) && string(v.started_at) && string(v.finished_at) && ['PASS', 'FAIL'].includes(String(v.decision))
    && strings(v.issues) && snapshot(v.before) && snapshot(v.after) && (v.compiler === null || (child(v.compiler) && object(v.compiler) && (v.compiler.result === null || (object(v.compiler.result)
      && v.compiler.result.schema_version === 1 && ['pass', 'fail'].includes(String(v.compiler.result.verdict)) && array(v.compiler.result.diagnostics, 10000, object)))))
}
function proposal(v: unknown): boolean {
  return object(v) && v.schema === 1 && kind(v.kind) && string(v.title) && string(v.rationale) && string(v.uncertainty)
    && array(v.changes, 64, c => object(c) && path(c.path) && nullableHash(c.before_sha256) && string(c.old) && string(c.new))
    && array(v.citations, 256, c => object(c) && nullableString(c.document) && (c.page === null || (Number.isSafeInteger(c.page) && Number(c.page) > 0)) && nullableString(c.section)
      && (c.source === null || (object(c.source) && path(c.source.path) && Number.isSafeInteger(c.source.line) && Number(c.source.line) > 0 && hash(c.source.sha256))))
}

export function proposalUrl(value: string): string {
  if (!id(value)) throw new Error('Invalid proposal ID')
  return `/api/proposals/${value}`
}
export function parseProposalList(value: unknown): ProposalSummary[] {
  if (!object(value) || value.schema !== 1 || !array(value.runs, 128, r => object(r) && id(r.id) && string(r.title) && (r.kind === null || kind(r.kind)) && state(r))) throw new Error('Invalid proposal list response')
  const runs = value.runs as ProposalSummary[]
  if (new Set(runs.map(r => r.id)).size !== runs.length) throw new Error('Duplicate proposal IDs')
  return runs
}
export function parseProposalDetail(value: unknown): ProposalDetail {
  const fail = () => { throw new Error('Invalid proposal detail response') }
  if (!object(value) || value.schema !== 1 || !state(value) || !object(value.run) || !id(value.run.id)) return fail()
  const r = value.run
  const logs = ['provider.stdout.log', 'provider.stderr.log', 'compiler.stdout.log', 'compiler.stderr.log', 'apply-compiler.stdout.log', 'apply-compiler.stderr.log']
  if (!array(value.artifacts, 6, a => object(a) && string(a.name) && logs.includes(a.name) && a.url === `${proposalUrl(r.id as string)}/artifacts/${a.name}`)) return fail()
  if (r.schema !== undefined && (r.schema !== 1 || !kind(r.kind) || !string(r.created_at) || !hash(r.bundle_sha256) || !nullableHash(r.proposal_sha256)
    || !child(r.provider) || !object(r.provider) || r.provider.engine !== 'codex' || !nullableString(r.provider.heartbeat_at))) return fail()
  if (r.proposal !== null && !proposal(r.proposal)) return fail()
  if (r.bundle !== null && (!object(r.bundle) || !string(r.bundle.design) || !snapshot(r.bundle.snapshot) || !object(r.bundle.selection) || r.bundle.selection.schema !== 1
    || !strings(r.bundle.selection.instances) || !array(r.bundle.selection.groups, 512, g => object(g) && string(g.view) && string(g.region)))) return fail()
  if (r.validation !== null && !validation(r.validation)) return fail()
  if (r.receipt !== null && (!object(r.receipt) || r.receipt.schema !== 1 || r.receipt.id !== r.id || !hash(r.receipt.proposal_sha256) || !hash(r.receipt.bundle_sha256) || !hash(r.receipt.validation_sha256)
    || (r.receipt.apply_validation !== null && !validation(r.receipt.apply_validation)) || !snapshot(r.receipt.before) || !snapshot(r.receipt.after) || !operation(r.receipt.apply) || (r.receipt.revert !== null && !operation(r.receipt.revert))
    || !array(r.receipt.files, 64, f => object(f) && path(f.path) && nullableHash(f.before_sha256) && hash(f.after_sha256) && nullableString(f.before) && string(f.after)))) return fail()
  return value as unknown as ProposalDetail
}
