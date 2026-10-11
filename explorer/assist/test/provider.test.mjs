import test from 'node:test'
import assert from 'node:assert/strict'
import { existsSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { fixture, json, prepare, run } from './helpers.mjs'
import { runProcess } from '../process.mjs'
import { providerContext } from '../provider.mjs'

test('provider projection retains exact sources and topology while declaring redundant context omissions', () => {
  const project = fixture()
  try {
    const { bundle } = prepare(project)
    const context = providerContext(bundle)
    assert.deepEqual(context.sources, bundle.sources.filter(s => s.path !== 'explorer/documents.json'))
    assert.deepEqual(context.model.nets, bundle.model.nets)
    assert.deepEqual(context.model.nc, bundle.model.nc)
    assert.deepEqual(context.documents, bundle.documents)
    assert.ok(context.omissions.some(s => s.includes('geometry')))
    assert.ok(context.omissions.some(s => s.includes('registry')))
    assert.ok(context.registry_groups.some(g => g.view === 'Functional' && g.region === 'Pair'))
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('unavailable real provider persists honest failure and leaves source unchanged', () => {
  const project = fixture()
  try {
    const { path } = prepare(project)
    writeFileSync(join(project, 'request.txt'), 'Review the actual passive parallel connection.')
    const directory = join(project, 'explorer/proposals/unavailable-engine')
    const result = run(['propose', path, '--kind', 'plan', '--engine', 'codex', '--request', join(project, 'request.txt'), '--out', directory], 1, { env: { ...process.env, COHDL_CODEX: '/nonexistent/cohdl-assist-test-provider' } })
    assert.match(result.error, /ENOENT|provider|spawn/i)
    const metadata = json(join(directory, 'run.json'))
    assert.equal(metadata.provider.engine, 'codex')
    assert.ok(metadata.provider.started_at && metadata.provider.finished_at)
    assert.match(metadata.provider.error, /ENOENT|spawn/i)
    assert.equal(metadata.proposal_sha256, null)
    assert.equal(existsSync(join(directory, 'provider.stderr.log')), true)
    assert.equal(existsSync(join(directory, 'proposal.json')), false)
    assert.notEqual(metadata.provider.cwd, project)
    assert.ok(metadata.provider.command.includes('read-only'))
    assert.equal(existsSync(metadata.provider.cwd), false)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('unsupported engines are explicit errors before any proposal directory is made', () => {
  const project = fixture()
  try {
    const { path } = prepare(project)
    writeFileSync(join(project, 'request.txt'), 'Plan')
    const directory = join(project, 'explorer/proposals/unsupported')
    assert.match(run(['propose', path, '--kind', 'plan', '--engine', 'unknown', '--request', join(project, 'request.txt'), '--out', directory], 1).error, /engine/i)
    assert.equal(existsSync(directory), false)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('bounded subprocess captures actual timeout output and termination', async () => {
  const result = await runProcess([process.execPath, '-e', 'console.log("actual-before-timeout");setInterval(()=>{},1000)'], { cwd: process.cwd(), timeout: 100 })
  assert.match(result.evidence.error, /timeout/)
  assert.match(result.stdout.toString(), /actual-before-timeout/)
  assert.ok(result.evidence.finished_at)
  assert.notEqual(result.evidence.signal, null)
})

test('bounded subprocess stops oversized output without retaining excess bytes', async () => {
  const result = await runProcess([process.execPath, '-e', 'process.stdout.write("x".repeat(65536));setInterval(()=>{},1000)'], { cwd: process.cwd(), limit: 1024 })
  assert.match(result.evidence.error, /output limit/)
  assert.equal(result.stdout.length + result.stderr.length, 1024)
})

test('bounded subprocess cancellation leaves actual terminal evidence', async () => {
  const controller = new AbortController()
  const pending = runProcess([process.execPath, '-e', 'setInterval(()=>{},1000)'], { cwd: process.cwd(), signal: controller.signal })
  setTimeout(() => controller.abort(), 50)
  const result = await pending
  assert.match(result.evidence.error, /cancelled/)
  assert.ok(result.evidence.finished_at)
})
