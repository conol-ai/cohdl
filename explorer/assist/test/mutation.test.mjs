import test from 'node:test'
import assert from 'node:assert/strict'
import { chmodSync, existsSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'
import { fixture, json, put, run, seed, sha, source } from './helpers.mjs'

function validated(project, options) {
  const proposal = seed(project, options)
  run(['validate', project, proposal.directory, '--out', proposal.validation])
  return proposal
}
const apply = (project, p, status = 0, options) => run(['apply', project, p.directory, '--validation', p.validation, '--out', p.receipt], status, options)

test('apply rechecks compiler and reverses exact bytes with frozen validation evidence', () => {
  const project = fixture()
  try {
    const p = validated(project)
    const originalValidation = readFileSync(p.validation)
    assert.equal(apply(project, p).apply.outcome, 'applied')
    const receipt = json(p.receipt)
    assert.equal(receipt.apply_validation.compiler.exit_code, 0)
    assert.deepEqual(readFileSync(p.validation), originalValidation)
    assert.match(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), /net N: b.A, a.A/)
    assert.match(run(['validate', project, p.directory, '--out', p.validation], 1).error, /frozen|receipt/i)
    assert.deepEqual(readFileSync(p.validation), originalValidation)
    assert.equal(run(['revert', project, p.receipt]).revert.outcome, 'reverted')
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
    assert.match(apply(project, p, 1).error, /receipt|applied|operation/i)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('apply refuses stale document even with a saved PASS', () => {
  const project = fixture()
  try {
    const p = validated(project)
    writeFileSync(join(project, 'explorer/documents/evidence.md'), 'changed after validation')
    assert.match(apply(project, p, 1).error, /stale/i)
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('apply never trusts forged after manifests in a saved validation', () => {
  const project = fixture()
  try {
    const p = validated(project)
    const report = json(p.validation)
    report.after.find((f) => f.path === 'src/main.cohdl').sha256 = '0'.repeat(64)
    put(p.validation, report)
    assert.match(apply(project, p, 1).error, /manifest|validation/i)
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('apply preserves real failed fresh compiler evidence without source writes', () => {
  const project = fixture()
  try {
    const p = validated(project)
    apply(project, p, 1, { env: { ...process.env, COHDL_COMPILER: '/nonexistent/cohdl-assist-test-compiler' } })
    const receipt = json(p.receipt)
    assert.equal(receipt.apply.outcome, 'rolled_back')
    assert.equal(receipt.apply_validation.decision, 'FAIL')
    assert.match(receipt.apply_validation.compiler.error, /ENOENT|spawn/i)
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('revert refuses an unrelated subsequent edit', () => {
  const project = fixture()
  try {
    const p = validated(project)
    apply(project, p)
    const editorText = source + '// unrelated editor change\n'
    writeFileSync(join(project, 'src/main.cohdl'), editorText)
    assert.match(run(['revert', project, p.receipt], 1).error, /changed|hash|stale/i)
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), editorText)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('revert rejects a symlink replacement without touching its target', () => {
  const project = fixture()
  try {
    const p = validated(project)
    apply(project, p)
    writeFileSync(join(project, 'dummy.cohdl'), 'dummy protected target')
    rmSync(join(project, 'src/main.cohdl'))
    symlinkSync('../dummy.cohdl', join(project, 'src/main.cohdl'))
    assert.match(run(['revert', project, p.receipt], 1).error, /symlink/i)
    assert.equal(readFileSync(join(project, 'dummy.cohdl'), 'utf8'), 'dummy protected target')
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('multi-file write failure rolls back completed own writes and retains evidence', () => {
  const project = fixture()
  const directory = join(project, 'src/locked')
  try {
    mkdirSync(directory)
    const notes = '// explicit rollback fixture before\n'
    writeFileSync(join(directory, 'notes.cohdl'), notes)
    const p = seed(project)
    p.proposal.changes.push({ path: 'src/locked/notes.cohdl', before_sha256: sha(notes), old: 'fixture before', new: 'fixture after' })
    put(join(p.directory, 'proposal.json'), p.proposal)
    const metadata = json(join(p.directory, 'run.json'))
    metadata.proposal_sha256 = sha(readFileSync(join(p.directory, 'proposal.json')))
    put(join(p.directory, 'run.json'), metadata)
    run(['validate', project, p.directory, '--out', p.validation])
    chmodSync(directory, 0o500)
    apply(project, p, 1)
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
    assert.equal(readFileSync(join(directory, 'notes.cohdl'), 'utf8'), notes)
    assert.equal(json(p.receipt).apply.outcome, 'rolled_back')
  } finally { chmodSync(directory, 0o700); rmSync(project, { recursive: true, force: true }) }
})

test('new native view apply and revert restores explicit absence', () => {
  const project = fixture()
  try {
    const path = 'views/Board.view.json'
    const original = readFileSync(join(project, path), 'utf8')
    // Registry group targets stay valid even when the view file is absent at baseline.
    rmSync(join(project, path))
    const p = validated(project, { kind: 'group', change: { path, before_sha256: null, old: '', new: original } })
    apply(project, p)
    assert.equal(existsSync(join(project, path)), true)
    run(['revert', project, p.receipt])
    assert.equal(existsSync(join(project, path)), false)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('concurrent live CLI lock refuses mutation and an actually exited owner can be recovered', () => {
  const project = fixture()
  try {
    const p = seed(project)
    const lock = join(project, 'explorer/proposals/.mutation.lock')
    put(lock, { pid: process.pid, token: 'live-fixture' })
    assert.match(run(['validate', project, p.directory, '--out', p.validation], 1).error, /live|lock/i)
    assert.equal(json(lock).token, 'live-fixture')
    const exited = spawnSync(process.execPath, ['-p', 'process.pid'], { encoding: 'utf8' })
    assert.equal(exited.status, 0)
    put(lock, { pid: Number(exited.stdout.trim()), token: 'exited-fixture' })
    assert.equal(run(['validate', project, p.directory, '--out', p.validation]).decision, 'PASS')
    assert.equal(existsSync(lock), false)
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('interrupted write-ahead receipt recovers exact before bytes without claiming completed apply', () => {
  const project = fixture()
  try {
    const p = validated(project)
    apply(project, p)
    const receipt = json(p.receipt)
    receipt.apply.outcome = 'applying'
    receipt.apply.finished_at = null
    put(p.receipt, receipt)
    assert.equal(run(['revert', project, p.receipt]).revert.outcome, 'reverted')
    assert.equal(json(p.receipt).apply.finished_at, null)
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
  } finally { rmSync(project, { recursive: true, force: true }) }
})
