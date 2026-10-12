import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { code, fixture, seed, sha } from './helpers.mjs'
import { groupAction } from '../focus.mjs'

process.env.COHDL_LIB ??= join(code, 'lib')

function grouped(project) {
  const path = 'views/Board.view.json'
  const before = readFileSync(join(project, path), 'utf8')
  const config = JSON.parse(before)
  config.views[0].regions.push({ name: 'Focus A', members: ['path:Board::a'] })
  return seed(project, { kind: 'group', change: { path, before_sha256: sha(before), old: before, new: JSON.stringify(config) } })
}

test('temporary group focus resolves actual members and original cross-group nets without writes', async () => {
  const project = fixture()
  try {
    const run = grouped(project)
    const before = readFileSync(join(project, 'views/Board.view.json'))
    const result = await groupAction(project, { action: 'focus', run: 'fixture-proposal' })
    assert.deepEqual(result.groups.map(g => g.name), ['Focus A'])
    assert.deepEqual(result.groups[0].instances, ['Board::a'])
    assert.equal(result.groups[0].nets.length, 2)
    assert.equal(result.groups[0].nets[0].outside.length, 1)
    assert.equal(result.model_sha256, sha(JSON.stringify(run.bundle.model)))
    assert.deepEqual(readFileSync(join(project, 'views/Board.view.json')), before)
    writeFileSync(join(project, 'src/main.cohdl'), 'changed')
    await assert.rejects(groupAction(project, { action: 'focus', run: 'fixture-proposal' }), /changed|stale/i)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('explicit save and undo reuse validation receipts and preserve electrical sources', async () => {
  const project = fixture()
  try {
    grouped(project)
    const source = readFileSync(join(project, 'src/main.cohdl'))
    const view = readFileSync(join(project, 'views/Board.view.json'))
    const saved = await groupAction(project, { action: 'save', run: 'fixture-proposal' })
    assert.equal(saved.apply.outcome, 'applied')
    assert.notDeepEqual(readFileSync(join(project, 'views/Board.view.json')), view)
    const undone = await groupAction(project, { action: 'undo', run: 'fixture-proposal' })
    assert.equal(undone.revert.outcome, 'reverted')
    assert.deepEqual(readFileSync(join(project, 'views/Board.view.json')), view)
    assert.deepEqual(readFileSync(join(project, 'src/main.cohdl')), source)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('display group actions reject path traversal and electrical proposals', async () => {
  const project = fixture()
  try {
    seed(project)
    await assert.rejects(groupAction(project, { action: 'focus', run: '../escape' }), /run/i)
    await assert.rejects(groupAction(project, { action: 'focus', run: 'fixture-proposal' }), /group/i)
    await assert.rejects(groupAction(project, { action: 'connect' }), /action/i)
  } finally { rmSync(project, { recursive: true, force: true }) }
})
