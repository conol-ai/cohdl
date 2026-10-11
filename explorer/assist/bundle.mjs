import { mkdtempSync, realpathSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { check, hash, input, JSON_LIMIT, load, object, output, read, sha, text } from './io.mjs'
import { fresh, registry, snapshot } from './snapshot.mjs'
import { runProcess } from './process.mjs'

export const checkout = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
export const compiler = () => process.env.COHDL_COMPILER ?? join(checkout, 'target/debug/cohdl')
export const extractor = () => process.env.COHDL_EXTRACTOR ?? join(checkout, 'explorer/extractor/target/debug/cohdl-explorer')

export function validateSelection(selection, model, views) {
  check(object(selection) && selection.schema === 1 && Array.isArray(selection.instances) && Array.isArray(selection.groups), 'Invalid selection schema')
  check(selection.instances.length <= 2048 && selection.groups.length <= 512, 'Selection limit exceeded')
  check(new Set(selection.instances).size === selection.instances.length, 'Duplicate selected instance')
  for (const path of selection.instances) check(model.instances.some((i) => i.path === path), `Unknown selected instance: ${path}`)
  for (const group of selection.groups) {
    check(object(group) && views?.views.some((v) => v.name === group.view && v.regions.some((r) => r.name === group.region)), 'Unknown selected group')
  }
}

export function applicable(applications, selection) {
  if (!selection.instances.length && !selection.groups.length) return applications
  return applications.filter((a) => a.instances?.some((i) => selection.instances.includes(i)) || a.groups?.some((g) => selection.groups.some((s) => s.view === g.view && s.region === g.region)))
}

async function excerpts(project, document, applications, omissions, signal) {
  if (document.kind === 'markdown') return [{ page: null, text: text(read(project, document.path, 65536)) }]
  const pages = [...new Set(applications.flatMap((a) => a.page === null || a.page === undefined ? [] : [a.page]))].slice(0, 8)
  if (!pages.length) { omissions.push(`${document.id}: no known applicable PDF pages`); return [] }
  const script = 'import json,sys\nfrom pypdf import PdfReader\nr=PdfReader(sys.argv[1]); pages=json.loads(sys.argv[2]); out=[]\nfor p in pages:\n if not isinstance(p,int) or p<1 or p>len(r.pages): raise ValueError("PDF page outside actual bounds")\n t=r.pages[p-1].extract_text() or ""\n if len(t.encode("utf-8"))>65536: raise ValueError("PDF text page budget exceeded")\n out.append({"page":p,"text":t})\nprint(json.dumps(out))'
  const result = await runProcess([process.env.COHDL_PDF_PYTHON ?? 'python3', '-c', script, join(project, document.path), JSON.stringify(pages)], { cwd: project, timeout: 15000, limit: 600000, signal })
  if (result.evidence.exit_code !== 0 || result.evidence.error) { omissions.push(`${document.id}: PDF extraction unavailable: ${result.evidence.error ?? text(result.stderr).slice(0, 500)}`); return [] }
  return JSON.parse(text(result.stdout))
}

async function documentContext(project, raw, selection, manifest, omissions, signal) {
  const documents = []
  for (const source of raw?.documents ?? []) {
    const applications = applicable(source.applicability ?? [], selection)
    if (!applications.length) continue
    const actual = source.path === null ? null : manifest.find((s) => s.path === source.path)?.sha256 ?? null
    const issues = []
    if (source.review !== 'reviewed') issues.push(`Evidence is ${source.review}`)
    if (applications.some((a) => a.relation !== 'exact')) issues.push('Related applicability is not exact proof')
    if (!hash(source.sha256)) issues.push('Expected document hash is unknown')
    const status = actual === null ? 'missing' : !hash(source.sha256) || source.sha256 !== actual ? 'changed' : 'ready'
    const document = { id: source.id, title: source.title, kind: source.kind, path: source.path, revision: source.revision ?? null, sha256: source.sha256 ?? null, actual_sha256: actual, pages: source.pages ?? null, review: source.review, uncertainty: source.uncertainty ?? null, applicability: applications, status, issues, excerpts: [] }
    if (status === 'ready') document.excerpts = await excerpts(project, document, applications, omissions, signal)
    documents.push(document)
  }
  return documents
}

export async function prepare(project, selectionFile, outputFile, signal) {
  const root = realpathSync(project)
  const temporary = mkdtempSync(join(tmpdir(), 'cohdl-assist-extract-'))
  try {
    const initial = snapshot(root)
    const modelFile = join(temporary, 'model.json')
    const result = await runProcess([extractor(), root, '-o', modelFile], { cwd: temporary, signal })
    check(result.evidence.exit_code === 0 && !result.evidence.error, `Extractor failed: ${result.evidence.error ?? text(result.stderr)}`)
    const model = JSON.parse(text(input(modelFile)))
    check(model.schema_version === 1 && model.verdict === 'pass', 'Prepare requires a checked passing circuit')
    check(JSON.stringify(initial) === JSON.stringify(snapshot(root)), 'Sources changed during extraction')
    const baseline = snapshot(root, model.design)
    const views = load(root, `views/${model.design}.view.json`, true)
    const selection = JSON.parse(text(input(selectionFile, 65536)))
    validateSelection(selection, model, views)
    const raw = registry(root)
    const assets = new Set(raw?.documents.map((d) => d.path).filter(Boolean) ?? [])
    const sources = baseline.filter((s) => s.sha256 !== null && !assets.has(s.path)).map((s) => ({ path: s.path, sha256: s.sha256, text: text(read(root, s.path)) }))
    const omissions = []
    const documents = await documentContext(root, raw, selection, baseline, omissions, signal)
    const constraints = (raw?.constraints ?? []).filter((c) => applicable([{ instances: c.instances, groups: c.groups }], selection).length)
    const bundle = { schema: 1, project: root, design: model.design, selection, snapshot: baseline, sources, model, views, registry: raw, documents, constraints, omissions }
    fresh(root, bundle)
    const bytes = JSON.stringify(bundle, null, 2) + '\n'
    check(Buffer.byteLength(bytes) <= JSON_LIMIT, 'Bundle byte limit exceeded')
    output(outputFile, bytes)
    return { schema: 1, bundle: resolve(outputFile), sha256: sha(bytes), design: model.design, files: baseline.length, documents: documents.length, omissions }
  } finally { rmSync(temporary, { recursive: true, force: true }) }
}
