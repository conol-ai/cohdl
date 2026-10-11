import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { fixture, json, prepare, put, run, sha, source } from './helpers.mjs'

test('prepare binds actual checked sources and applicable document bytes', () => {
  const project = fixture()
  try {
    const { bundle } = prepare(project)
    assert.equal(bundle.schema, 1)
    assert.equal(bundle.design, 'Board')
    assert.equal(bundle.model.verdict, 'pass')
    assert.deepEqual(bundle.selection.instances, ['Board::a'])
    assert.equal(bundle.sources.find((s) => s.path === 'src/main.cohdl').text, source)
    assert.equal(bundle.snapshot.find((s) => s.path === 'src/main.cohdl').sha256, sha(source))
    assert.equal(bundle.snapshot.find((s) => s.path === 'design.lock').sha256, null)
    assert.match(bundle.documents.find((d) => d.id === 'evidence').excerpts[0].text, /parallel/)
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('prepare rejects unknown selected instances without creating a bundle', () => {
  const project = fixture()
  try {
    put(join(project, 'selection.json'), { schema: 1, instances: ['Board::missing'], groups: [] })
    const result = run(['prepare', project, '--selection', join(project, 'selection.json'), '--out', join(project, 'bundle.json')], 1)
    assert.match(result.error, /selection|instance/i)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('prepare rejects symlink source before following its bytes', () => {
  const project = fixture()
  try {
    writeFileSync(join(project, 'outside.txt'), 'dummy private fixture')
    symlinkSync('../outside.txt', join(project, 'src/alias.cohdl'))
    const result = run(['prepare', project, '--selection', join(project, 'selection.json'), '--out', join(project, 'bundle.json')], 1)
    assert.match(result.error, /symlink/i)
    assert.ok(!JSON.stringify(result).includes('dummy private fixture'))
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('prepare retains missing evidence as absence and uncertainty', () => {
  const project = fixture()
  try {
    rmSync(join(project, 'explorer/documents/evidence.md'))
    const { bundle } = prepare(project)
    assert.equal(bundle.snapshot.find((s) => s.path === 'explorer/documents/evidence.md').sha256, null)
    assert.equal(bundle.documents[0].status, 'missing')
    assert.deepEqual(bundle.documents[0].excerpts, [])
  } finally { rmSync(project, { recursive: true, force: true }) }
})
