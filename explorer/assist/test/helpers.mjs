import assert from 'node:assert/strict'
import { cpSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'

export const code = resolve(dirname(fileURLToPath(import.meta.url)), '../../..')
export const cli = join(code, 'explorer/assist/cli.mjs')
export const sha = (bytes) => createHash('sha256').update(bytes).digest('hex')
export const json = (path) => JSON.parse(readFileSync(path, 'utf8'))
export const put = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + '\n')
export const source = `pub trait TwoTerminal { designator_prefix: "R" }
pub device Res<R: Resistance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { resistance: R }
}
impl TwoTerminal for Res {}
pub footprint Land {}
pub part R10K: Res<10kohm> {
    primary { mfr: "Fixture", mpn: "R10K", footprint: Land }
}
design Board {
    inst a: R10K
    inst b: R10K
    net N: a.A, b.A
    net M: a.B, b.B
}
`

export function fixture() {
  const project = realpathSync(mkdtempSync(join(tmpdir(), 'cohdl-assist-test-')))
  for (const directory of ['src', 'views', 'explorer/documents']) mkdirSync(join(project, directory), { recursive: true })
  writeFileSync(join(project, 'src/main.cohdl'), source)
  writeFileSync(join(project, 'cohdl.toml'), '[package]\nname = "assist-fixture"\nversion = "0.1.0"\n\n[dependencies]\nstd = "0.3.0"\n')
  cpSync(join(code, 'lib/soic/cohdl.lock'), join(project, 'cohdl.lock'))
  put(join(project, 'views/Board.view.json'), { schema_version: 1, design: 'Board', views: [{ name: 'Functional', regions: [{ name: 'Pair', members: ['path:Board::a', 'path:Board::b'] }] }] })
  const text = '# Fixture evidence\nBoth passive resistors are connected in parallel.\n'
  writeFileSync(join(project, 'explorer/documents/evidence.md'), text)
  put(join(project, 'explorer/documents.json'), { schema: 1, documents: [{ id: 'evidence', title: 'Explicit fixture evidence', kind: 'markdown', path: 'explorer/documents/evidence.md', source_url: null, revision: 'fixture', sha256: sha(text), pages: null, review: 'reviewed', uncertainty: null, applicability: [{ instances: ['Board::a', 'Board::b'], groups: [{ view: 'Functional', region: 'Pair' }], relation: 'exact', page: null, section: 'Fixture evidence', facts: null, uncertainty: null }] }], constraints: [] })
  put(join(project, 'selection.json'), { schema: 1, instances: ['Board::a'], groups: [] })
  return project
}

export function run(args, expected = 0, options = {}) {
  const result = spawnSync(process.execPath, [cli, ...args], { cwd: code, encoding: 'utf8', timeout: 30000, ...options, env: { ...process.env, COHDL_LIB: join(code, 'lib'), ...options.env } })
  assert.equal(result.status, expected, result.stderr + result.stdout)
  return JSON.parse(result.stdout)
}

export function prepare(project) {
  const path = join(project, 'bundle.json')
  run(['prepare', project, '--selection', join(project, 'selection.json'), '--out', path])
  return { path, bundle: json(path) }
}

// Explicit persisted fixtures test validation; real provider acceptance runs separately.
export function seed(project, { kind = 'connect', change, citations } = {}) {
  const { path, bundle } = prepare(project)
  const id = 'fixture-proposal'
  const directory = join(project, 'explorer/proposals', id)
  mkdirSync(directory, { recursive: true })
  cpSync(path, join(directory, 'bundle.json'))
  const proposal = { schema: 1, kind, title: 'Explicit fixture proposal', rationale: 'Validation fixture, not an AI execution.', uncertainty: 'Test only; no electrical qualification.', citations: citations ?? [{ document: null, page: null, section: null, source: { path: 'src/main.cohdl', line: 14, sha256: sha(source) } }], changes: kind === 'plan' ? [] : [change ?? { path: 'src/main.cohdl', before_sha256: sha(source), old: 'net N: a.A, b.A', new: 'net N: b.A, a.A' }] }
  put(join(directory, 'proposal.json'), proposal)
  writeFileSync(join(directory, 'request.txt'), 'Explicit persisted test fixture')
  put(join(directory, 'run.json'), { schema: 1, id, kind, created_at: new Date().toISOString(), bundle_sha256: sha(readFileSync(path)), proposal_sha256: sha(readFileSync(join(directory, 'proposal.json'))), provider: { engine: 'codex', command: ['explicit-persisted-test-fixture'], cwd: project, heartbeat_at: null, started_at: new Date().toISOString(), finished_at: new Date().toISOString(), exit_code: 0, signal: null, error: null } })
  return { directory, bundle, proposal, validation: join(directory, 'validation.json'), receipt: join(directory, 'receipt.json') }
}
