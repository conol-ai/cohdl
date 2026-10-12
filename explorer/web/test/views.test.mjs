import assert from 'node:assert/strict'
import { after, test } from 'node:test'
import { mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { createRequire } from 'node:module'
import ts from 'typescript'

const outDir = mkdtempSync(join(tmpdir(), 'cohdl-views-test-'))
after(() => rmSync(outDir, { recursive: true, force: true }))
const program = ts.createProgram(['src/views.ts', 'src/viewport.ts'], {
  target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS,
  outDir, strict: true, skipLibCheck: true, types: [],
})
assert.deepEqual(ts.getPreEmitDiagnostics(program).map((d) => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
const { parseViewConfig, assignRegions } = require(resolve(outDir, 'views.js'))
const { boundsOfRects } = require(resolve(outDir, 'viewport.js'))
const config = () => ({ schema_version: 1, design: 'Board', views: [
  { name: 'Blocks', regions: [{ name: 'Camera', members: ['U1', 'path:Board::camera', 'agg:host:Board::camera'] }] },
] })

test('valid project view config retains exact rule order and source values', () => {
  const c = config()
  const before = JSON.stringify(c)
  assert.deepEqual(parseViewConfig(c, 'Board'), c)
  assert.equal(JSON.stringify(c), before)
})

test('malformed view configs fail explicitly before graph assignment', () => {
  const invalid = [null, [], {}, { ...config(), schema_version: 2 }, { ...config(), design: 'Other' }]
  for (const mutate of [
    (c) => { c.views = [] },
    (c) => { c.views.push(c.views[0]) },
    (c) => { c.views[0].name = ' Blocks ' },
    (c) => { c.views[0].regions.push(c.views[0].regions[0]) },
    (c) => { c.views[0].regions[0].name = 'Other' },
    (c) => { c.views[0].regions[0].members = [null] },
    (c) => { c.views[0].regions[0].members = ['path:'] },
    (c) => { c.views[0].regions[0].members = ['agg:host:'] },
    (c) => { c.views[0].regions[0].members = ['unknown:U1'] },
  ]) { const c = config(); mutate(c); invalid.push(c) }
  for (const c of invalid) assert.throws(() => parseViewConfig(c, 'Board'), /Invalid view config:/)
})

test('valid overlapping rules keep first match and implicit Other without model mutation', () => {
  const c = config()
  c.views[0].regions.push({ name: 'Power', members: ['U1', 'U2'] })
  const g = { nodes: [{ id: 'Board::camera', inst: { designator: 'U1' } }, { id: 'Board::power', inst: { designator: 'U2' } }, { id: 'Board::open', inst: { designator: 'J1' } }] }
  const before = JSON.stringify(g)
  const a = assignRegions(g, parseViewConfig(c, 'Board').views[0])
  assert.deepEqual([...a.byNode], [['Board::camera', 'Camera'], ['Board::power', 'Power'], ['Board::open', 'Other']])
  assert.equal(JSON.stringify(g), before)
})

test('navigation indexes reference canonical groups and permit shared children', () => {
  const c = config()
  c.views[0].navigation = [{ name: 'Host', groups: ['Camera'] }, { name: 'Shared', groups: ['Camera'] }]
  assert.deepEqual(parseViewConfig(c, 'Board'), c)
  for (const navigation of [null, {}, [{ name: 'Host', groups: ['Unknown'] }],
    [{ name: 'Host', groups: [] }], [{ name: 'Host', groups: ['Camera', 'Camera'] }],
    [{ name: ' Host ', groups: ['Camera'] }], [{ name: 'Host', groups: ['Camera'] }, { name: 'Host', groups: ['Camera'] }]]) {
    c.views[0].navigation = navigation
    assert.throws(() => parseViewConfig(c, 'Board'), /Invalid view config:/)
  }
})

test('fit bounds include relative children and region extents, with empty/invalid guards', () => {
  assert.equal(boundsOfRects([]), null)
  assert.deepEqual(boundsOfRects([
    { x: 110, y: 220, width: 70, height: 80 },
    { x: -50, y: -10, width: 20, height: 30 },
    { x: 100, y: 200, width: 200, height: 200 },
  ]), { x: -50, y: -10, width: 350, height: 410 })
  for (const width of [NaN, Infinity, -1])
    assert.throws(() => boundsOfRects([{ x: 0, y: 0, width, height: 10 }]), /Invalid layout bounds/)
})
