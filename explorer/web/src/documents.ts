import type { SrcSpan } from './model'

export interface DocumentGroup { view: string; region: string }
export interface Applicability {
  instances: string[]; groups: DocumentGroup[]; relation: 'exact' | 'related'
  page?: number | null; section?: string | null; facts?: Record<string, string> | null; uncertainty?: string | null
}
export interface DocumentRecord {
  id: string; title: string; kind: 'pdf' | 'markdown'; path: string | null
  source_url?: string | null; revision: string | null; sha256: string | null; pages: number | null
  review: 'unreviewed' | 'reviewed' | 'rejected'; uncertainty: string | null; applicability: Applicability[]
  status: 'ready' | 'invalid' | 'missing'; actual_sha256: string | null; issues: string[]; open_url: string | null
}
export interface Citation { document: string; page?: number | null; section?: string | null }
export interface DocumentConstraint {
  text: string; instances: string[]; groups: DocumentGroup[]; citations: Citation[]; uncertainty: string | null; issues: string[]
}
export interface DocumentRegistry { schema: 1; documents: DocumentRecord[]; constraints: DocumentConstraint[] }
export interface DocumentResource { registry: DocumentRegistry | null; loading: boolean; error: string; refresh: () => void }

export function applies(instances: string[], groups: DocumentGroup[], instance: string | undefined, view: string, selectedGroups: string[]): boolean {
  return instance ? instances.includes(instance) : groups.some(g => g.view === view && selectedGroups.includes(g.region))
}

export function documentHref(document: DocumentRecord, page?: number | null): string | null {
  if (document.status !== 'ready' || document.open_url !== `/api/documents/${document.id}` || !/^[a-zA-Z0-9_-]+$/.test(document.id)) return null
  if (page !== undefined && page !== null && (document.kind !== 'pdf' || !Number.isSafeInteger(page) || page < 1 || (document.pages !== null && page > document.pages))) return null
  return `${document.open_url}${page ? `#page=${page}` : ''}`
}

export function parseDocumentRegistry(value: unknown): DocumentRegistry {
  const object = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null && !Array.isArray(v)
  const strings = (v: unknown): v is string[] => Array.isArray(v) && v.every(x => typeof x === 'string')
  const nullableString = (v: unknown) => v == null || typeof v === 'string'
  const targets = (v: unknown): boolean => object(v) && strings(v.instances) && Array.isArray(v.groups) && v.groups.every(g => object(g) && typeof g.view === 'string' && typeof g.region === 'string')
  const citation = (v: unknown): boolean => object(v) && (v.page == null || (Number.isSafeInteger(v.page) && Number(v.page) >= 0)) && nullableString(v.section)
  if (!object(value) || value.schema !== 1 || !Array.isArray(value.documents) || !Array.isArray(value.constraints)) throw new Error('Invalid document registry response')
  const documentsValid = value.documents.every(d => object(d) && typeof d.id === 'string' && typeof d.title === 'string' && ['pdf', 'markdown'].includes(String(d.kind))
    && ['ready', 'invalid', 'missing'].includes(String(d.status)) && ['unreviewed', 'reviewed', 'rejected'].includes(String(d.review))
    && ['path', 'revision', 'sha256', 'uncertainty', 'actual_sha256', 'open_url', 'source_url'].every(k => nullableString(d[k]))
    && (d.pages == null || (Number.isSafeInteger(d.pages) && Number(d.pages) >= 0)) && strings(d.issues) && Array.isArray(d.applicability)
    && d.applicability.every(a => targets(a) && object(a) && ['exact', 'related'].includes(String(a.relation)) && citation(a) && nullableString(a.uncertainty)
      && (a.facts == null || (object(a.facts) && Object.values(a.facts).every(v => typeof v === 'string')))))
  const constraintsValid = value.constraints.every(c => targets(c) && object(c) && typeof c.text === 'string' && nullableString(c.uncertainty) && strings(c.issues)
    && Array.isArray(c.citations) && c.citations.every(v => citation(v) && object(v) && typeof v.document === 'string'))
  if (!documentsValid || !constraintsValid) throw new Error('Invalid document registry response')
  return value as unknown as DocumentRegistry
}

export function sourceUrl(span: SrcSpan): string {
  return `/api/source?${new URLSearchParams({ path: span.file, line: String(Math.max(1, span.line - 3)), count: '12' })}`
}
