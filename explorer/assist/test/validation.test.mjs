import test from 'node:test'
import assert from 'node:assert/strict'
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { fixture, json, put, run, seed, sha, source } from './helpers.mjs'

function validate(project, proposal, expected = 0) {
  return run(['validate', project, proposal.directory, '--out', proposal.validation], expected)
}

test('validate compiles exact proposed CoHDL in isolated copy without source mutation', () => {
  const project = fixture()
  try {
    const proposal = seed(project)
    const result = validate(project, proposal)
    assert.equal(result.decision, 'PASS')
    const saved = json(proposal.validation)
    assert.equal(saved.compiler.exit_code, 0)
    assert.equal(saved.compiler.result.verdict, 'pass')
    assert.notEqual(saved.compiler.cwd, project)
    assert.equal(saved.before.find((f) => f.path === 'src/main.cohdl').sha256, sha(source))
    assert.equal(saved.after.find((f) => f.path === 'src/main.cohdl').sha256, sha(source.replace('net N: a.A, b.A', 'net N: b.A, a.A')))
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
    assert.equal(existsSync(saved.compiler.cwd), false, 'compiler scratch removed')
  } finally { rmSync(project, { recursive: true, force: true }) }
})

for (const mutation of ['source', 'document', 'new-source']) test(`validate rejects stale ${mutation} with persisted failure evidence`, () => {
  const project = fixture()
  try {
    const proposal = seed(project)
    const file = mutation === 'source' ? 'src/main.cohdl' : mutation === 'document' ? 'explorer/documents/evidence.md' : 'src/added.cohdl'
    writeFileSync(join(project, file), mutation === 'source' ? source + '\n' : 'changed fixture')
    const result = validate(project, proposal, 1)
    assert.match(result.error, /stale/i)
    assert.equal(json(proposal.validation).decision, 'FAIL')
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('validate retains real compiler failure diagnostics', () => {
  const project = fixture()
  try {
    const proposal = seed(project, { change: { path: 'src/main.cohdl', before_sha256: sha(source), old: 'net N: a.A, b.A', new: 'net N: a.UNKNOWN, b.A' } })
    validate(project, proposal, 1)
    const result = json(proposal.validation)
    assert.equal(result.decision, 'FAIL')
    assert.equal(result.compiler.result.verdict, 'fail')
    assert.ok(result.compiler.result.diagnostics.some((d) => d.severity === 'error'))
    assert.ok(readFileSync(join(proposal.directory, 'compiler.stdout.log'), 'utf8').includes('UNKNOWN'))
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('validate rejects changed provider bytes even when JSON remains valid', () => {
  const project = fixture()
  try {
    const proposal = seed(project)
    writeFileSync(join(proposal.directory, 'proposal.json'), readFileSync(join(proposal.directory, 'proposal.json'), 'utf8') + ' ')
    assert.match(validate(project, proposal, 1).error, /hash|bytes/i)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

for (const invalid of ['path', 'line', 'hash', 'document', 'page']) test(`validate rejects unsupported citation ${invalid}`, () => {
  const project = fixture()
  try {
    let citation = { document: null, page: null, section: null, source: { path: 'src/main.cohdl', line: 14, sha256: sha(source) } }
    if (invalid === 'path') citation.source.path = '../outside.cohdl'
    if (invalid === 'line') citation.source.line = 0
    if (invalid === 'hash') citation.source.sha256 = '0'.repeat(64)
    if (invalid === 'document') citation = { document: 'missing', page: null, section: null, source: null }
    if (invalid === 'page') citation.page = 1
    const proposal = seed(project, { citations: [citation] })
    assert.match(validate(project, proposal, 1).error, /citation|source|document|page|path/i)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('native group proposal validates actual member rules and preserves registry targets', () => {
  const project = fixture()
  try {
    const path = 'views/Board.view.json'
    const before = readFileSync(join(project, path), 'utf8')
    const next = json(join(project, path))
    next.views.push({ name: 'Resistors', regions: [{ name: 'Pair', members: ['path:Board::a', 'path:Board::b'] }] })
    const proposal = seed(project, { kind: 'group', change: { path, before_sha256: sha(before), old: before, new: JSON.stringify(next) } })
    assert.equal(validate(project, proposal).decision, 'PASS')
    next.views[0].regions[0].name = 'Removed old identity'
    proposal.proposal.changes[0].new = JSON.stringify(next)
    put(join(proposal.directory, 'proposal.json'), proposal.proposal)
    const metadata = json(join(proposal.directory, 'run.json'))
    metadata.proposal_sha256 = sha(readFileSync(join(proposal.directory, 'proposal.json')))
    put(join(proposal.directory, 'run.json'), metadata)
    assert.match(validate(project, proposal, 1).error, /group|applicability/i)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('document citation metadata cannot be promoted by tampering with the archived bundle', () => {
  const project = fixture()
  try {
    const registryPath = join(project, 'explorer/documents.json')
    const registry = json(registryPath)
    registry.documents[0].review = 'unreviewed'
    put(registryPath, registry)
    const proposal = seed(project, { kind: 'plan', citations: [{ document: 'evidence', page: null, section: 'Fixture evidence', source: null }] })
    const bundlePath = join(proposal.directory, 'bundle.json')
    const bundle = json(bundlePath)
    bundle.documents[0].review = 'reviewed'
    put(bundlePath, bundle)
    const metadata = json(join(proposal.directory, 'run.json'))
    metadata.bundle_sha256 = sha(readFileSync(bundlePath))
    put(join(proposal.directory, 'run.json'), metadata)
    assert.match(validate(project, proposal, 1).error, /document|registry|bound|metadata/i)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('actual extracted source authority rejects nonexistent members in a forged archived model', () => {
  const project = fixture()
  try {
    const path = 'views/Board.view.json'
    const before = readFileSync(join(project, path), 'utf8')
    const view = JSON.parse(before)
    view.views[0].regions.push({ name: 'Ghost', members: ['path:Board::ghost'] })
    const p = seed(project, { kind: 'group', change: { path, before_sha256: sha(before), old: before, new: JSON.stringify(view) } })
    const bundlePath = join(p.directory, 'bundle.json')
    const bundle = json(bundlePath)
    bundle.model.instances.push({ ...bundle.model.instances[0], path: 'Board::ghost', designator: 'GHOST' })
    put(bundlePath, bundle)
    const metadata = json(join(p.directory, 'run.json'))
    metadata.bundle_sha256 = sha(readFileSync(bundlePath))
    put(join(p.directory, 'run.json'), metadata)
    assert.match(validate(project, p, 1).error, /model|authorit|extract/i)
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('literal dollar replacement tokens remain exact provider bytes in compiled output', () => {
  const project = fixture()
  try {
    const replacement = 'net N: b.A, a.A // literal $& $` $\' $$'
    const p = seed(project, { change: { path: 'src/main.cohdl', before_sha256: sha(source), old: 'net N: a.A, b.A', new: replacement } })
    const result = validate(project, p)
    assert.equal(result.after.find(f => f.path === 'src/main.cohdl').sha256, sha(source.replace('net N: a.A, b.A', () => replacement)))
  } finally { rmSync(project, { recursive: true, force: true }) }
})

test('registered documents outside explorer stay out of isolated compiler input regardless of valid document size', () => {
  const project = fixture()
  try {
    mkdirSync(join(project, 'docs'))
    const bytes = Buffer.alloc(9 * 1024 * 1024, 0x20)
    writeFileSync(join(project, 'docs/large.pdf'), bytes)
    const registryPath = join(project, 'explorer/documents.json')
    const registry = json(registryPath)
    registry.documents.push({ ...registry.documents[0], id: 'large', kind: 'pdf', path: 'docs/large.pdf', sha256: sha(bytes), applicability: [] })
    put(registryPath, registry)
    const p = seed(project, { kind: 'plan' })
    assert.equal(validate(project, p).decision, 'PASS')
  } finally { rmSync(project, { recursive: true, force: true }) }
})

for (const target of ['view', 'document']) test(`source edits cannot leave dangling ${target} identities after actual compilation`, () => {
  const project = fixture()
  try {
    if (target === 'document') {
      const path = join(project, 'views/Board.view.json')
      const view = json(path)
      view.views[0].regions[0].members = ['path:Board::a']
      put(path, view)
    }
    const next = source.replace('    inst b: R10K\n', '').replace(', b.A', '').replace(', b.B', '')
    const p = seed(project, { change: { path: 'src/main.cohdl', before_sha256: sha(source), old: source, new: next } })
    assert.match(validate(project, p, 1).error, /member|instance|applicability|target/i)
    assert.equal(readFileSync(join(project, 'src/main.cohdl'), 'utf8'), source)
  } finally { rmSync(project, { recursive: true, force: true }) }
})
