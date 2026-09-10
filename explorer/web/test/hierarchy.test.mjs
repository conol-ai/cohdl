import assert from 'node:assert/strict'
import { after, test } from 'node:test'
import { mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { createRequire } from 'node:module'
import ts from 'typescript'

// Compile the pure projection with the existing TypeScript dependency; no
// browser, compiler binary, or additional test framework is needed.
const outDir = mkdtempSync(join(tmpdir(), 'cohdl-explorer-test-'))
after(() => rmSync(outDir, { recursive: true, force: true }))
const program = ts.createProgram(['src/hierarchy.ts', 'src/transform.ts', 'src/boardGeometry.ts', 'src/handles.ts'], {
  target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS,
  outDir, strict: true, skipLibCheck: true, types: [],
})
assert.deepEqual(ts.getPreEmitDiagnostics(program).map((d) => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
const { projectScope, ownerScope, explorerNets } = require(resolve(outDir, 'hierarchy.js'))
const { buildGraph } = require(resolve(outDir, 'transform.js'))
const { passiveTerminals, wireHandles } = require(resolve(outDir, 'handles.js'))
const { transformPoint, placedBounds, footprintBounds, layoutScope, outlinePath, outlineBounds } = require(resolve(outDir, 'boardGeometry.js'))

const span = { file: 'src/main.cohdl', line: 1, col: 1 }
const port = (name, net, connected = true) => ({ name, net, connected, obligation: 'required' })
const sub = (path, parent, ports) => ({ path, parent, definition: 'demo::Cell', span, ports })
const part = (path) => ({
  path, device_fq: 'demo::Resistor', designator: path.endsWith('load') ? 'R1' : 'R2',
  impl_traits: [], specs: [], span, docs: [],
  pins: ['A', 'B'].map((logical, i) => ({ logical, numbers: [String(i + 1)], role: 'passive', obligation: 'required', connected: true, nc: false })),
})
const net = (name, members) => ({ name, is_gnd: false, span, members: members.map(([instance_path, logical_pin]) => ({ instance_path, logical_pin, numbers: ['1'] })) })
const model = () => ({
  schema_version: 1, design: 'Board', verdict: 'pass', diagnostics: [], nc: [], footprints: {},
  instances: [part('Board::load'), part('Board::banks_0::cell::r'), part('Board::banks_1::cell::r')],
  subdesigns: [
    sub('Board::banks_0', undefined, [port('IN', 'INPUT'), port('OUT', 'OUTPUT')]),
    sub('Board::banks_0::cell', 'Board::banks_0', [port('IN', 'INPUT'), port('OUT', 'OUTPUT'), { ...port('UNUSED', undefined, false), obligation: 'optional' }]),
    sub('Board::banks_1', undefined, [port('IN', 'INPUT'), port('OUT', 'OUTPUT')]),
    sub('Board::banks_1::cell', 'Board::banks_1', [port('IN', 'INPUT'), port('OUT', 'OUTPUT')]),
    sub('Board::empty', undefined, []),
    sub('Board::link_0', undefined, [port('A', 'ONLY_PORTS')]),
    sub('Board::link_1', undefined, [port('B', 'ONLY_PORTS')]),
  ],
  nets: [
    net('INPUT', [['Board::load', 'A'], ['Board::banks_0::cell::r', 'A'], ['Board::banks_1::cell::r', 'A']]),
    net('OUTPUT', [['Board::load', 'B'], ['Board::banks_0::cell::r', 'B'], ['Board::banks_1::cell::r', 'B']]),
  ],
  derived: { two_terminal: [], rails: [], fn_groups: [], bypasses: [] },
})

test('root shows array use sites and empty/port-only blocks, without physical impostors', () => {
  const m = model()
  const g = buildGraph(m)
  assert.deepEqual(g.nodes.filter((n) => n.inst).map((n) => n.id), ['Board::load'])
  assert.equal(g.nodes.filter((n) => n.kind === 'subdesign').length, 5)
  assert.ok(g.nodes.filter((n) => n.kind === 'subdesign').every((n) => !n.inst))
  const onlyPorts = g.edges.find((e) => e.net === 'ONLY_PORTS')
  assert.equal(onlyPorts.source, 'Board::link_0')
  assert.equal(onlyPorts.sourcePin, 'A')
  assert.equal(onlyPorts.target, 'Board::link_1')
  assert.equal(onlyPorts.targetPin, 'B')
  assert.equal(explorerNets(m).find((n) => n.name === 'ONLY_PORTS').members.length, 0)
})

test('nested drill-down joins named boundary ports to the next level only', () => {
  const m = model()
  const g = buildGraph(m, 'Board::banks_0')
  assert.deepEqual(g.nodes.map((n) => n.id), ['Board::banks_0::cell', 'port:Board::banks_0:IN', 'port:Board::banks_0:OUT'])
  assert.equal(g.edges.length, 2)
  assert.ok(g.edges.every((e) => e.source === 'Board::banks_0::cell' && e.target.startsWith('port:Board::banks_0:')))
  const leaf = buildGraph(m, 'Board::banks_0::cell')
  assert.equal(leaf.nodes.filter((n) => n.inst).length, 1)
  assert.equal(leaf.nodes.filter((n) => n.kind === 'port').length, 3)
  assert.equal(leaf.edges.length, 2)
  assert.equal(ownerScope(m, 'Board::banks_0::cell::__fn0_shunt::r'), 'Board::banks_0::cell')
  assert.equal(ownerScope(m, 'Board::banks_01::r'), '')
})

test('internal nets do not invent public ports or escape a collapsed boundary', () => {
  const m = model()
  m.nets.push(net('PRIVATE', [['Board::banks_0::cell::r', 'C']]))
  m.subdesigns[1].ports.push({ ...port('LOCAL', 'PRIVATE', false), obligation: 'optional' })
  const parent = projectScope(m, 'Board::banks_0')
  assert.ok(!parent.model.nets.some((n) => n.name === 'PRIVATE'))
  assert.equal(parent.boundaries[0].ports.find((p) => p.name === 'LOCAL').net, undefined)
  const leaf = projectScope(m, 'Board::banks_0::cell')
  assert.equal(leaf.model.nets.find((n) => n.name === 'PRIVATE').members.length, 2)
})

test('rail projection keeps boundary tags and aggregation stays within the open scope', () => {
  const m = model()
  m.derived.rails = ['INPUT', 'OUTPUT']
  m.derived.two_terminal = m.instances.map((i) => i.path)
  const root = buildGraph(m)
  assert.deepEqual(root.nodes.find((n) => n.kind === 'agg').aggMembers, ['Board::load'])
  assert.deepEqual(root.nodes.find((n) => n.id === 'Board::banks_0').railTags, ['INPUT', 'OUTPUT'])
  const leaf = buildGraph(m, 'Board::banks_0::cell')
  assert.deepEqual(leaf.nodes.find((n) => n.kind === 'agg').aggMembers, ['Board::banks_0::cell::r'])
})

test('flat view, old v1 snapshots, and repeated projections preserve original topology', () => {
  const m = model()
  const original = JSON.stringify(m)
  const flat = buildGraph(m, null)
  assert.equal(flat.nodes.filter((n) => n.inst).length, 3)
  assert.ok(!flat.nodes.some((n) => n.kind === 'subdesign'))
  assert.deepEqual(buildGraph(m).nodes, buildGraph(m).nodes)
  assert.equal(JSON.stringify(m), original)
  delete m.subdesigns
  assert.deepEqual(buildGraph(m).nodes, flat.nodes)
  assert.deepEqual(buildGraph(m).edges, flat.edges)
})

test('RF series resistor keeps branched antenna and feed nets on separate terminals in both modes', () => {
  const m = model()
  const series = 'Board::rf::rf_series'
  m.instances = ['Board::mcu', 'Board::rf::ant', series, 'Board::rf::rf_shunt_0', 'Board::rf::rf_shunt_1'].map(part)
  m.instances.find((i) => i.path === series).specs = [{ name: 'resistance', value: '0ohm' }]
  m.subdesigns = [sub('Board::rf', undefined, [port('FEED', 'BT_RF'), port('GND', 'GND')])]
  m.derived.two_terminal = [series]
  m.derived.rails = ['GND']
  // Member ordering reproduces OpenMicroDial: the resistor's B pin is both
  // an edge target and an edge source on the three-member antenna net.
  m.nets = [
    net('BT_RF', [['Board::mcu', 'A'], [series, 'A'], ['Board::rf::rf_shunt_0', 'A']]),
    net('rf::BT_ANT', [['Board::rf::ant', 'A'], [series, 'B'], ['Board::rf::rf_shunt_1', 'A']]),
    net('GND', [['Board::rf::rf_shunt_0', 'B'], ['Board::rf::rf_shunt_1', 'B']]),
  ]
  for (const scope of ['Board::rf', null]) {
    const g = buildGraph(m, scope)
    const nodes = new Map(g.nodes.map((n) => [n.id, n]))
    const resistor = nodes.get(series)
    assert.deepEqual(resistor.pinNets, { A: 'BT_RF', B: 'rf::BT_ANT' })
    assert.equal(resistor.kind, 'passive')
    const antenna = g.edges.filter((e) => e.net === 'rf::BT_ANT' && (e.source === series || e.target === series))
    assert.ok(antenna.some((e) => e.source === series))
    assert.ok(antenna.some((e) => e.target === series))
    for (const detailed of [false, true]) {
      const terminals = new Map(passiveTerminals(resistor).map((p) => [p.id, p]))
      const attached = new Map()
      for (const e of g.edges) {
        if (e.source !== series && e.target !== series) continue
        const wire = wireHandles(e, nodes, detailed)
        const handle = e.source === series ? wire.sourceHandle : wire.targetHandle
        const terminal = terminals.get(handle)
        assert.ok(terminal, 'every resistor wire must address a real terminal')
        assert.equal(resistor.pinNets[terminal.pin], e.net)
        const locations = attached.get(e.net) ?? new Set()
        locations.add(`${terminal.x},${terminal.y}`)
        attached.set(e.net, locations)
      }
      assert.equal(attached.get('BT_RF').size, 1)
      assert.equal(attached.get('rf::BT_ANT').size, 1)
      assert.notDeepEqual(attached.get('BT_RF'), attached.get('rf::BT_ANT'))
    }
  }
})

test('an unconnected passive terminal keeps its identity and side', () => {
  const m = model()
  m.subdesigns = []
  m.instances = [part('Board::diode')]
  m.instances[0].pins[1].logical = 'K'
  m.instances[0].pins[0].connected = false
  m.derived.two_terminal = ['Board::diode']
  m.nets = [net('CATHODE', [['Board::diode', 'K']])]
  const n = buildGraph(m).nodes[0]
  const pins = passiveTerminals(n)
  assert.deepEqual(pins.map((p) => [p.pin, p.side]), [['A', 'left'], ['K', 'right']])
  assert.equal(pins[0].label, 'A (1): unconnected')
  assert.equal(pins[1].label, 'K (2): CATHODE')
})

test('physical layout keeps board coordinates and mirrors before rotation', () => {
  assert.deepEqual(transformPoint([1, 2], [0, -1, 1, 0], [10, 20]), [12, 19])
  assert.deepEqual(transformPoint([1, 2], [0, 1, 1, 0], [10, 20]), [12, 21])
  const fp = { pads: [{ number: '1', shape: 'rect', x: 1, y: 2, size: [2, 1], rotate: 90, matrix: [0, -1, 1, 0] }], mount_holes: [] }
  assert.deepEqual(footprintBounds(fp), { x: 0.5, y: 1, width: 1, height: 2 })
  assert.deepEqual(placedBounds({ at: [10, 20], matrix: [0, -1, 1, 0] }, fp), { x: 11, y: 18.5, width: 2, height: 1 })
  const annulus = { pads: [{ shape: 'annulus', x: 0, y: 0, size: [4, 1] }], mount_holes: [] }
  assert.deepEqual(footprintBounds(annulus), { x: -2, y: -2, width: 4, height: 4 })
})

test('layout scope retains descendants, separates unplaced parts, and never stages them', () => {
  const m = model()
  m.layout = { placements: [{ instance: 'Board::banks_0::cell::r', at: [-3.25, 6.5], at_mm: ['-3.25', '6.5'], side: 'bottom', rotate: 270, matrix: [0, -1, -1, 0] }] }
  const before = JSON.stringify(m)
  assert.equal(layoutScope(m, '').unplaced.length, 2)
  const scope = layoutScope(m, 'Board::banks_0')
  assert.equal(scope.instances.length, 1)
  assert.equal(scope.unplaced.length, 0)
  assert.deepEqual(scope.placements[0].at, [-3.25, 6.5])
  assert.equal(layoutScope(m, 'Board::banks_1').placements.length, 0)
  assert.equal(JSON.stringify(m), before)
  delete m.layout
  assert.equal(layoutScope(m, null).unplaced.length, m.instances.length)
})

test('unanchored subdesign local layouts stay visible without becoming board placements', () => {
  const m = model()
  const path = 'Board::banks_0::cell::r'
  const local = { instance: path, at: [1, 2], at_mm: ['1', '2'], side: 'top', rotate: 90, matrix: [0, -1, 1, 0] }
  m.subdesigns[1].local_placements = [local]
  m.layout = null
  const before = JSON.stringify(m)
  const scoped = layoutScope(m, 'Board::banks_0::cell', 'local')
  assert.equal(scoped.frame, 'local')
  assert.deepEqual(scoped.placements, [local])
  assert.equal(scoped.unplaced.length, 0)
  assert.equal(scoped.outline, undefined)
  assert.equal(layoutScope(m, 'Board::banks_0::cell', 'board').placements.length, 0)
  assert.equal(layoutScope(m, '', 'local').placements.length, 0)
  assert.equal(layoutScope(m, null, 'local').frame, 'board')
  assert.equal(JSON.stringify(m), before)

  m.layout = { placements: [{ ...local, at: [100, 200], at_mm: ['100', '200'] }], board_outline: { start: [0, 0], segments: [] } }
  assert.deepEqual(layoutScope(m, 'Board::banks_0::cell', 'local').placements[0].at, [1, 2])
  assert.equal(layoutScope(m, 'Board::banks_0::cell', 'local').outline, undefined)
  assert.deepEqual(layoutScope(m, 'Board::banks_0::cell', 'board').placements[0].at, [100, 200])
  assert.equal(layoutScope(m, 'Board::banks_0::cell', 'board').outline, m.layout.board_outline)
  delete m.subdesigns[1].local_placements
  assert.equal(layoutScope(m, 'Board::banks_0::cell', 'local').frame, 'board', 'older snapshots remain usable')
})

test('DXF arc winding and major arcs survive the y-down board projection', () => {
  const outline = { source: 'outline.dxf', start: [10, 0], segments: [{ type: 'arc', to: [0, 10], center: [0, 0], clockwise: false }, { type: 'line', to: [10, 0] }] }
  assert.equal(outlinePath(outline), 'M 10 0 A 10 10 0 0 1 0 10 L 10 0 Z')
  outline.segments[0].clockwise = true
  assert.equal(outlinePath(outline), 'M 10 0 A 10 10 0 1 0 0 10 L 10 0 Z')
  assert.ok(outlineBounds(outline).some(([x, y]) => x === -10 && y === -10))
})
