import assert from 'node:assert/strict'
import test from 'node:test'
import { cpSync, existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { resolve } from 'node:path'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import ts from 'typescript'

const out = mkdtempSync(resolve('test/.proposals-render-'))
process.once('exit', () => rmSync(out, { recursive: true, force: true }))
writeFileSync(resolve(out, 'package.json'), JSON.stringify({ type: 'commonjs' }))
const program = ts.createProgram(['src/proposals.ts', 'src/ProposalReview.tsx'], {
  target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS,
  jsx: ts.JsxEmit.ReactJSX, outDir: out, strict: true, skipLibCheck: true,
})
assert.deepEqual(ts.getPreEmitDiagnostics(program).map(d => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
for (const name of ['ProposalReview.css', 'ProvenanceInspector.css']) if (existsSync(`src/${name}`)) cpSync(`src/${name}`, resolve(out, name))
require.extensions['.css'] = () => {}
const { parseProposalList, parseProposalDetail, proposalUrl } = require(resolve(out, 'proposals.js'))
const { ProposalReview } = require(resolve(out, 'ProposalReview.js'))

const hash = 'a'.repeat(64)
const summary = { id: 'run-1', title: 'Persisted suggestion', kind: 'connect', status: 'reverted', freshness: 'fresh', issues: [] }
const childEvidence = { command: ['cohdl', 'check'], cwd: '/isolated/scratch', started_at: '2026-10-11T00:00:00Z', finished_at: '2026-10-11T00:00:01Z', exit_code: 0, signal: null, error: null }
const validation = { schema: 1, id: 'run-1', proposal_sha256: hash, bundle_sha256: hash, started_at: childEvidence.started_at, finished_at: childEvidence.finished_at, decision: 'PASS', issues: [], before: [], after: [], compiler: { ...childEvidence, result: { schema_version: 1, verdict: 'pass', diagnostics: [{ severity: 'warning', code: 'TEST', message: 'Retained compiler diagnostic' }] } } }
const operation = { started_at: childEvidence.started_at, finished_at: childEvidence.finished_at, outcome: 'applied', error: null }
const detail = { schema: 1, status: 'reverted', freshness: 'fresh', issues: [], artifacts: [{ name: 'provider.stdout.log', url: '/api/proposals/run-1/artifacts/provider.stdout.log' }], run: {
  schema: 1, id: 'run-1', kind: 'connect', created_at: childEvidence.started_at, bundle_sha256: hash, proposal_sha256: hash,
  provider: { ...childEvidence, engine: 'codex', heartbeat_at: null }, bundle: { design: 'Board', selection: { schema: 1, instances: ['Board::one'], groups: [] }, snapshot: [] },
  proposal: { schema: 1, kind: 'connect', title: 'Persisted suggestion', rationale: 'Rationale from provider', uncertainty: 'Review the exact suffix', citations: [{ document: null, page: null, section: null, source: { path: 'src/main.cohdl', line: 42, sha256: hash } }], changes: [{ path: 'src/main.cohdl', before_sha256: hash, old: 'Literal <old>', new: 'Literal & new' }] },
  validation, receipt: { schema: 1, id: 'run-1', bundle_sha256: hash, proposal_sha256: hash, validation_sha256: hash, apply_validation: validation, before: [], after: [], files: [], apply: operation, revert: { ...operation, outcome: 'reverted' } },
} }
const resource = extra => ({ runs: [summary], selected: 'run-1', detail, loading: false, error: '', detailLoading: false, detailError: '', select() {}, refresh() {}, ...extra })
const documents = { registry: null, loading: false, error: '', refresh() {} }
const render = extra => renderToStaticMarkup(React.createElement(ProposalReview, { resource: resource(extra), documents, onBack() {} }))

test('proposal parser preserves evidence and rejects unsafe IDs, links and response shapes', () => {
  assert.deepEqual(parseProposalList({ schema: 1, runs: [summary] }), [summary])
  assert.deepEqual(parseProposalDetail(detail), detail)
  assert.equal(proposalUrl('run-1'), '/api/proposals/run-1')
  for (const id of ['../run', '.hidden', 'a/b', 'a%2fb', 'a'.repeat(65)]) assert.throws(() => proposalUrl(id))
  for (const bad of [null, {}, { schema: 2, runs: [] }, { schema: 1, runs: [{ ...summary, status: 'success' }] }, { schema: 1, runs: [{ ...summary, id: '../run' }] }]) assert.throws(() => parseProposalList(bad))
  for (const bad of [{ ...detail, artifacts: [{ name: 'run.json', url: '/api/proposals/run-1/artifacts/run.json' }] }, { ...detail, artifacts: [{ name: 'provider.stdout.log', url: 'javascript:alert(1)' }] }, { ...detail, run: { ...detail.run, proposal: { ...detail.run.proposal, changes: [{ path: '../secret', old: '', new: '' }] } } }]) assert.throws(() => parseProposalDetail(bad))
})

test('proposal review keeps exact diff, citation, compiler result and both receipt operations', () => {
  const html = render()
  for (const text of ['Rationale from provider', 'Review the exact suffix', 'Retained compiler diagnostic', 'Compiler verdict: pass', 'reverted', 'applied', 'Literal &lt;old&gt;', 'Literal &amp; new', 'src/main.cohdl:42', 'Fresh apply validation']) assert.ok(html.includes(text), text)
  assert.match(html, /href="\/api\/proposals\/run-1\/artifacts\/provider.stdout.log"/)
  assert.match(html, /aria-label="Copy proposal JSON"/)
  assert.match(html, /Back to Circuit/)
})

test('empty, loading and error states always retain refresh recovery', () => {
  for (const [extra, text] of [[{ runs: [], selected: '', detail: null }, 'No proposals'], [{ loading: true }, 'Loading proposals'], [{ error: 'Network unavailable' }, 'Network unavailable'], [{ detailError: 'Detail unavailable', detail: null }, 'Detail unavailable']]) {
    const html = render(extra)
    assert.ok(html.includes(text)); assert.match(html, /aria-label="Refresh proposals"/)
  }
})

test('pending, failed, rejected, stale and interrupted remain separate visible states', () => {
  for (const status of ['pending', 'failed', 'rejected', 'interrupted', 'validated', 'applied', 'reverted']) {
    const html = render({ runs: [{ ...summary, status, freshness: 'stale' }], detail: { ...detail, status, freshness: 'stale', issues: ['Input bytes changed'] } })
    assert.ok(html.includes(status)); assert.ok(html.includes('stale')); assert.ok(html.includes('Input bytes changed'))
  }
})
