import assert from 'node:assert/strict'
import { after, test } from 'node:test'
import { mkdtempSync, rmSync } from 'node:fs'
import { resolve } from 'node:path'
import { createRequire } from 'node:module'
import ts from 'typescript'

const outDir = mkdtempSync(resolve('node_modules/.connectivity-test-'))
after(() => rmSync(outDir, { recursive: true, force: true }))
const program = ts.createProgram(['src/connectivity.ts', 'src/layout.ts', 'src/edges.tsx'], {
  target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS, esModuleInterop: true,
  outDir, strict: true, skipLibCheck: true, types: [], jsx: ts.JsxEmit.ReactJSX,
})
assert.deepEqual(ts.getPreEmitDiagnostics(program).map((d) => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
const { bundleConnections, bundleLabelSize } = require(resolve(outDir, 'connectivity.js'))
const { layout } = require(resolve(outDir, 'layout.js'))
const { LaneEdge } = require(resolve(outDir, 'edges.js'))
const { createElement } = require('react')
const { renderToStaticMarkup } = require('react-dom/server')
const { ReactFlowProvider } = require('@xyflow/react')
const span = { file: 'main.cohdl', line: 1, col: 1 }
const node = (id, extras = {}) => ({ id, kind: 'ic', title: id, sub: '', railTags: [], netLabels: [], decors: [],
  pinsConnected: 0, pinsTotal: 0, pinNets: {}, width: 140, height: 60, ...extras })
const net = (name, paths) => ({ name, is_gnd: name === 'GND', span,
  members: paths.map((path, i) => ({ instance_path: path, logical_pin: `P${i}`, numbers: [String(i + 1)] })) })
const model = (nets, subdesigns = []) => ({ nets, subdesigns })
const graph = (ids, edges = []) => ({ nodes: ids.map((id) => node(id)), edges,
  location: new Map(ids.map((id) => [id, id])), railSet: new Set(['GND']) })

test('count labels reserve the same readable geometry used by their renderer', () => {
  assert.deepEqual(bundleLabelSize('1 net'), { width: 76, height: 32, fontSize: 20 })
})

test('neutral pairs count distinct nets, deduplicate pins and preserve full hidden membership', () => {
  const g = graph(['A', 'B', 'net:old'])
  g.nodes[2].kind = 'net'
  const first = net('X', ['A', 'A', 'B', 'HIDDEN'])
  const second = net('Y', ['B', 'A'])
  const before = JSON.stringify(g.nodes)
  const result = bundleConnections(g, model([first, second, first]))
  assert.deepEqual(result.graph.nodes.map((n) => n.id), ['A', 'B'])
  assert.equal(result.graph.edges.length, 1)
  const detail = result.bundles.get(result.graph.edges[0].id)
  assert.deepEqual(detail.objects, ['A', 'B'])
  assert.deepEqual(detail.nets, [first, second])
  assert.equal(detail.nets[0], first)
  assert.equal(detail.nets[0].members.length, 4)
  assert.equal(JSON.stringify(g.nodes), before)
  assert.equal(result.graph.location, g.location)
})

test('multi-object nets share only identical membership buses, never an inferred clique', () => {
  const g = graph(['A', 'B', 'C', 'D'])
  const result = bundleConnections(g, model([net('X', ['A', 'B', 'C']), net('Y', ['C', 'A', 'B']), net('Z', ['A', 'B', 'D'])]))
  const buses = result.graph.nodes.filter((n) => n.kind === 'net')
  assert.equal(buses.length, 2)
  assert.equal(result.graph.edges.length, 6)
  for (const bus of buses) {
    const size = bundleLabelSize(bus.title)
    assert.equal(bus.width, size.width)
    assert.equal(bus.height, size.height)
    const detail = result.bundles.get(bus.id)
    const spokes = result.graph.edges.filter((e) => e.target === bus.id)
    assert.equal(spokes.length, 3)
    for (const spoke of spokes) assert.equal(result.bundles.get(spoke.id), detail)
  }
  assert.equal(result.graph.edges.some((e) => ['A', 'B', 'C', 'D'].includes(e.target)), false)
})

test('checked visible boundary ports participate while hidden descendants and open child ports do not', () => {
  const g = graph(['A', 'child', 'parent-port', 'open-child'])
  g.nodes[1].ports = [{ name: 'BUS', net: 'X', connected: true }]
  g.nodes[2].ports = [{ name: 'LOCAL', net: 'LOCAL', connected: false }]
  g.nodes[3].ports = [{ name: 'OPEN', connected: false }]
  const physical = net('X', ['A', 'child::hidden', 'child::other'])
  const logical = { path: 'scope', span, ports: [{ name: 'LOCAL', net: 'LOCAL' }] }
  const result = bundleConnections(g, model([physical], [logical]))
  const pair = [...result.bundles.values()][0]
  assert.deepEqual(pair.objects, ['A', 'child'])
  assert.equal(pair.nets[0], physical)
  assert.equal(result.graph.edges.length, 1, 'one-object local port stays inspectable without a false edge')
  assert.equal(pair.objects.includes('open-child'), false)
})

test('rail stubs and one-object nets do not become bundles; bypass attachments retain no electrical count', () => {
  const attachment = { id: 'bypass', source: 'A', target: 'B', nets: [], net: 'bypass:A', label: '', dashed: true }
  const g = graph(['A', 'B'], [attachment])
  const result = bundleConnections(g, model([net('GND', ['A', 'B']), net('SINGLE', ['A', 'A'])]))
  assert.deepEqual(result.graph.edges, [attachment])
  assert.equal(result.bundles.size, 0)
})

function checkGeometry(g, positioned, regions) {
  const boxes = g.nodes.map((n) => {
    const p = positioned.positions.get(n.id)
    const parent = regions && positioned.regionBoxes.get(`region:${regions.byNode.get(n.id) ?? 'Other'}`)
    return { id: n.id, x: p.x + (parent?.x ?? 0), y: p.y + (parent?.y ?? 0), width: n.width, height: n.height }
  })
  for (const edge of g.edges) {
    const route = positioned.edgeRoutes.get(edge.id)
    assert.ok(route && route.points.length >= 2, `real sections for ${edge.id}`)
    for (let i = 1; i < route.points.length; i++) {
      const a = route.points[i - 1], b = route.points[i]
      assert.ok(a.x === b.x || a.y === b.y, 'orthogonal section')
      for (const box of boxes.filter((n) => n.id !== edge.source && n.id !== edge.target)) {
        const crossing = a.x === b.x
          ? a.x > box.x && a.x < box.x + box.width && Math.max(a.y, b.y) > box.y && Math.min(a.y, b.y) < box.y + box.height
          : a.y > box.y && a.y < box.y + box.height && Math.max(a.x, b.x) > box.x && Math.min(a.x, b.x) < box.x + box.width
        assert.equal(crossing, false, `${edge.id} crosses ${box.id}`)
      }
    }
    if (edge.label) for (const box of boxes) {
      const size = bundleLabelSize(edge.label), half = size.width / 2
      assert.equal(route.label.x + half > box.x && route.label.x - half < box.x + box.width
        && route.label.y + size.height / 2 > box.y && route.label.y - size.height / 2 < box.y + box.height, false, `label crosses ${box.id}`)
    }
    const source = boxes.find((n) => n.id === edge.source)
    const target = boxes.find((n) => n.id === edge.target)
    assert.deepEqual(route.points[0], { x: source.x + source.width, y: source.y + source.height / 2 })
    assert.deepEqual(route.points.at(-1), { x: target.x, y: target.y + target.height / 2 })
  }
}

test('real ELK retains orthogonal routes and label geometry for pair/bus topology', async () => {
  const { graph: g } = bundleConnections(graph(['group:A', 'group:B', 'group:C', 'group:D']), model([
    net('PAIR', ['group:A', 'group:D']), net('BUS', ['group:A', 'group:B', 'group:C']),
    net('PAIR2', ['group:B', 'group:D']),
  ]))
  checkGeometry(g, await layout(g))
})

test('compact exact-trace graphs retain ELK section routing instead of neutral bundle gutters', async () => {
  const g = graph(['group:A', 'group:B'], [{ id: 'exact', source: 'group:A', target: 'group:B', net: 'X', nets: ['X'], label: '' }])
  const p = await layout(g, undefined, { compact: true })
  assert.ok(p.positions.get('group:A').x < 48, 'actual ELK root padding, not fixed neutral cell margin')
  checkGeometry(g, p)
})

test('compound ELK sections translate their actual container offsets to absolute graph coordinates', async () => {
  const { graph: g } = bundleConnections(graph(['A', 'B', 'C', 'D']), model([
    net('LOCAL', ['A', 'B']), net('CROSS', ['B', 'C']), net('OTHER', ['C', 'D']),
  ]))
  const regions = { byNode: new Map([['A', 'Left'], ['B', 'Left'], ['C', 'Right'], ['D', 'Right']]), regions: ['Left', 'Right'] }
  checkGeometry(g, await layout(g, regions), regions)
})

test('complete compact overview bounds arbitrary pair chains without body or count-label overlap', async () => {
  const ids = Array.from({ length: 26 }, (_, i) => `group:${String(i).padStart(2, '0')}`)
  const nets = ids.slice(1).map((id, i) => net(`PAIR${i}`, [ids[i], id]))
  nets.push(net('SHARED', ids.slice(0, 5)))
  const { graph: g } = bundleConnections(graph(ids), model(nets))
  for (const aspectRatio of [1.6, 0.65]) {
    const p = await layout(g, undefined, { compact: true, aspectRatio, columns: aspectRatio < 1 ? 2 : undefined })
    checkGeometry(g, p)
    const bounds = g.nodes.map((n) => ({ ...p.positions.get(n.id), width: n.width, height: n.height }))
    assert.ok(Math.max(...bounds.map((n) => n.x + n.width)) < 2000, 'compact chain width')
    assert.ok(Math.max(...bounds.map((n) => n.y + n.height)) < 2500, 'bounded compact chain height')
    if (aspectRatio < 1) assert.equal(new Set(bounds.map((n) => n.x + n.width / 2)).size, 2, 'readable mobile two columns')
    const labels = g.edges.filter((e) => e.label).map((e) => ({ ...p.edgeRoutes.get(e.id).label, ...bundleLabelSize(e.label) }))
    for (let i = 0; i < labels.length; i++) for (const b of labels.slice(i + 1)) {
      const a = labels[i]
      assert.equal(Math.abs(a.x - b.x) < (a.width + b.width) / 2 && Math.abs(a.y - b.y) < (a.height + b.height) / 2, false, 'count-label overlap')
    }
  }
})

test('neutral edge renders actual routed coordinates with a wide hit path and no endpoint fallback', () => {
  const props = { id: 'bundle:test', sourceX: 1000, sourceY: 1000, targetX: 2000, targetY: 2000,
    data: { bundleId: 'bundle:test', route: { points: [{ x: 0, y: 0 }, { x: 30, y: 0 }, { x: 30, y: 50 }, { x: 80, y: 50 }], label: { x: 30, y: 25 } } } }
  const html = renderToStaticMarkup(createElement(ReactFlowProvider, null, createElement(LaneEdge, props)))
  assert.match(html, /M 0 0/)
  assert.match(html, /L 80 50/)
  assert.match(html, /stroke-width="24"/)
  assert.doesNotMatch(html, /M 1000 1000/)
  const missing = renderToStaticMarkup(createElement(ReactFlowProvider, null, createElement(LaneEdge, { ...props, data: { bundleId: 'bundle:test' } })))
  assert.doesNotMatch(missing, /<path/)
})
