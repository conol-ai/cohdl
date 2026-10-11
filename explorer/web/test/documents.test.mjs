import assert from 'node:assert/strict'
import test from 'node:test'
import { cpSync, existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { resolve } from 'node:path'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import ts from 'typescript'

const out = mkdtempSync(resolve('test/.documents-render-'))
process.once('exit', () => rmSync(out, { recursive: true, force: true }))
writeFileSync(resolve(out, 'package.json'), JSON.stringify({ type: 'commonjs' }))
const program = ts.createProgram(['src/documents.ts', 'src/ProvenanceInspector.tsx'], {
  target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS,
  jsx: ts.JsxEmit.ReactJSX, outDir: out, strict: true, skipLibCheck: true,
})
assert.deepEqual(ts.getPreEmitDiagnostics(program).map((d) => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
if (existsSync('src/ProvenanceInspector.css')) cpSync('src/ProvenanceInspector.css', resolve(out, 'ProvenanceInspector.css'))
require.extensions['.css'] = () => {}
const { applies, documentHref, parseDocumentRegistry, sourceUrl } = require(resolve(out, 'documents.js'))
const { ProvenanceInspector } = require(resolve(out, 'ProvenanceInspector.js'))

const target = { instances: ['Board::one'], groups: [{ view: 'Functional', region: 'Camera' }], relation: 'exact', page: 76, facts: { mpn: '', value: 'Authored source value' } }
const document = {
  id: 'ordering', title: 'Ordering evidence', kind: 'pdf', path: 'docs/ordering.pdf',
  revision: 'G', sha256: 'a'.repeat(64), pages: 78, review: 'reviewed', uncertainty: null,
  applicability: [target], status: 'ready', actual_sha256: 'a'.repeat(64), issues: [], open_url: '/api/documents/ordering',
}
const registry = { schema: 1, documents: [document], constraints: [] }
const resource = (value = registry) => ({ registry: value, loading: false, error: '', refresh() {} })
const render = (props = {}) => renderToStaticMarkup(React.createElement(ProvenanceInspector, { resource: resource(), instance: 'Board::one', view: 'Functional', groups: [], ...props }))

test('part and qualified multi-group applicability never invents membership', () => {
  assert.equal(applies(target.instances, target.groups, 'Board::one', 'Other view', []), true)
  assert.equal(applies(target.instances, target.groups, 'Board::two', 'Functional', ['Camera']), false)
  assert.equal(applies(target.instances, target.groups, undefined, 'Functional', ['Power', 'Camera']), true)
  assert.equal(applies(target.instances, target.groups, undefined, 'Source pages', ['Camera']), false)
  assert.equal(applies(target.instances, target.groups, undefined, 'Functional', []), false)
  assert.equal(applies([], [], undefined, 'Functional', ['Camera']), false)
})

test('document links use real PDF page index and reject unavailable or unsafe targets', () => {
  assert.equal(documentHref(document, 76), '/api/documents/ordering#page=76')
  assert.equal(documentHref(document), '/api/documents/ordering')
  for (const page of [0, -1, 79, 1.5, NaN, Infinity, Number.MAX_SAFE_INTEGER + 1]) assert.equal(documentHref(document, page), null)
  assert.equal(documentHref({ ...document, pages: null }, 76), '/api/documents/ordering#page=76')
  assert.equal(documentHref({ ...document, kind: 'markdown' }, 1), null)
  for (const status of ['invalid', 'missing']) assert.equal(documentHref({ ...document, status }, 76), null)
  for (const open_url of ['https://outside.example/file', '/api/file?p=secret', 'javascript:alert(1)']) assert.equal(documentHref({ ...document, open_url }, 76), null)
  assert.equal(documentHref({ ...document, id: '../ordering' }, 76), null)
})

test('typed response parsing retains invalid record diagnostics beside good documents', () => {
  const invalid = { ...document, id: 'invalid', pages: 0, applicability: [{ ...target, page: 0 }], status: 'invalid', issues: ['Citation page is outside the declared PDF pages'], open_url: null }
  const response = { ...registry, documents: [document, invalid] }
  assert.deepEqual(parseDocumentRegistry(response), response)
  assert.deepEqual(parseDocumentRegistry({ schema: 1, documents: [], constraints: [] }), { schema: 1, documents: [], constraints: [] })
  for (const bad of [null, {}, { ...registry, schema: 2 }, { ...registry, documents: [{ ...document, status: 'invented' }] }, { ...registry, documents: [{ ...document, applicability: [{ ...target, facts: { mpn: 1 } }] }] }, { ...registry, constraints: [{ text: 'Bad target', instances: 'Board::one', groups: [], citations: [], issues: [] }] }]) assert.throws(() => parseDocumentRegistry(bad), /Invalid document registry/)
})

test('source excerpt URLs preserve structured paths and bounded context', () => {
  const url = new URL(sourceUrl({ file: 'src/a b#c.cohdl', line: 42, col: 1 }), 'http://localhost')
  assert.equal(url.pathname, '/api/source')
  assert.equal(url.searchParams.get('path'), 'src/a b#c.cohdl')
  assert.equal(url.searchParams.get('line'), '39')
  assert.equal(url.searchParams.get('count'), '12')
  assert.equal(new URL(sourceUrl({ file: 'src/main.cohdl', line: 1, col: 1 }), 'http://localhost').searchParams.get('line'), '1')
})

test('native inspector preserves authored empty facts, page citation and source identity', () => {
  const html = render()
  assert.match(html, /Authored empty/)
  assert.match(html, /Authored source value/)
  assert.match(html, /Local hash verified/)
  assert.match(html, /PDF page 76/)
  assert.match(html, /href="\/api\/documents\/ordering#page=76"/)
  assert.match(html, /Exact association/)
  assert.match(html, /Revision: G/)
  assert.match(html, /Actual a{64}/)
  assert.doesNotMatch(render({ instance: 'Board::two' }), /Ordering evidence/)
  assert.match(render({ instance: undefined, groups: ['Camera'] }), /Ordering evidence/)
})

test('related and missing original evidence stays distinct from local support', () => {
  const related = { ...document, id: 'family', title: '<script>Family document</script>', open_url: '/api/documents/family', applicability: [{ ...target, relation: 'related' }], uncertainty: 'Exact suffix is not established', review: 'unreviewed' }
  const missing = { ...document, id: 'original', title: 'Missing original', path: null, status: 'missing', actual_sha256: null, open_url: null, pages: null, uncertainty: 'Historical metadata only', issues: ['Local original is unavailable'] }
  const support = { ...document, id: 'support', title: 'Supporting library text', kind: 'markdown', pages: null, applicability: [{ ...target, page: null, section: 'Facts used' }], open_url: '/api/documents/support' }
  const html = render({ resource: resource({ ...registry, documents: [related, missing, support] }) })
  assert.match(html, /Related part evidence/)
  assert.match(html, /Exact suffix is not established/)
  assert.match(html, /No local original/)
  assert.match(html, /Not locally verified/)
  assert.match(html, /Page count unknown; citation range not verified/)
  assert.match(html, /Local supporting Markdown/)
  assert.match(html, /href="\/api\/documents\/support"/)
  assert.doesNotMatch(html, /href="\/api\/documents\/original/)
  assert.doesNotMatch(html, /<script>/)
  assert.match(html, /&lt;script&gt;/)
})

test('constraints retain missing citation limitations and qualified group selection', () => {
  const constraints = [{ text: 'Recorded supply limit', instances: ['Board::one'], groups: target.groups, citations: [{ document: 'ordering', page: 1 }, { document: 'absent', page: 2 }], uncertainty: 'Does not establish board compliance', issues: ['Constraint cites an unknown document'] }]
  const html = render({ resource: resource({ ...registry, constraints }) })
  assert.match(html, /Recorded supply limit/)
  assert.match(html, /Does not establish board compliance/)
  assert.match(html, /Constraint cites an unknown document/)
  assert.match(html, /href="\/api\/documents\/ordering#page=1"/)
  assert.match(html, /absent.*Unavailable/)
  assert.doesNotMatch(render({ instance: undefined, view: 'Source pages', groups: ['Camera'], resource: resource({ ...registry, constraints }) }), /Recorded supply limit/)
})

test('loading error empty and accessible refresh remain usable', () => {
  assert.match(render({ resource: { ...resource(), loading: true } }), /role="status"/)
  assert.match(render({ resource: { ...resource(), error: 'Malformed document registry' } }), /role="alert">Malformed document registry/)
  assert.match(render({ resource: resource({ schema: 1, documents: [], constraints: [] }) }), /No applicable document records/)
  const calls = []
  const element = ProvenanceInspector({ resource: { ...resource(), refresh: () => calls.push('refresh') }, instance: 'Board::one', view: 'Functional', groups: [] })
  const heading = React.Children.toArray(element.props.children)[0]
  const refresh = React.Children.toArray(heading.props.children).find((child) => child.type === 'button')
  assert.equal(refresh.props['aria-label'], 'Refresh source evidence')
  assert.equal(refresh.props.title, 'Refresh source evidence')
  refresh.props.onClick()
  assert.deepEqual(calls, ['refresh'])
})
