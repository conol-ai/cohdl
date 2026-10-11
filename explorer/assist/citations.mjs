import { applicable } from './bundle.mjs'
import { check, hash, object, relativePath } from './io.mjs'

function sourceCitation(source, bundle) {
  check(object(source) && typeof source.path === 'string' && source.path.endsWith('.cohdl'), 'Invalid source citation path')
  relativePath(source.path)
  const file = bundle.sources.find((s) => s.path === source.path)
  check(file && hash(source.sha256) && source.sha256 === file.sha256, 'Source citation hash does not match baseline')
  const lines = file.text.split('\n').length - (file.text.endsWith('\n') ? 1 : 0)
  check(Number.isSafeInteger(source.line) && source.line >= 1 && source.line <= lines, 'Source citation line outside file')
}

function documentCitation(citation, bundle) {
  const document = bundle.documents.find((d) => d.id === citation.document)
  check(document && document.status === 'ready' && document.review === 'reviewed' && document.actual_sha256 === document.sha256, 'Document citation is missing, changed or unreviewed')
  const targets = applicable(document.applicability, bundle.selection)
  check(targets.some((a) => a.relation === 'exact'), 'Document citation lacks exact applicability')
  check(citation.section === null || typeof citation.section === 'string' && citation.section.trim().length > 0 && citation.section.length <= 2048, 'Invalid citation section')
  if (document.kind === 'markdown') check(citation.page === null, 'Markdown citation cannot have a PDF page')
  else check(Number.isSafeInteger(document.pages) && Number.isSafeInteger(citation.page) && citation.page >= 1 && citation.page <= document.pages, 'Document citation requires a known bounded PDF page')
}

export function validateCitations(citations, bundle) {
  check(Array.isArray(citations) && citations.length >= 1 && citations.length <= 256, 'Proposal requires bounded citations')
  for (const citation of citations) {
    check(object(citation) && Object.keys(citation).sort().join(',') === 'document,page,section,source', 'Invalid citation fields')
    check(citation.document !== null || citation.source !== null, 'Citation requires document or source')
    if (citation.document === null) check(citation.page === null && citation.section === null, 'Citation page/section requires document')
    else documentCitation(citation, bundle)
    if (citation.source !== null) sourceCitation(citation.source, bundle)
  }
}
