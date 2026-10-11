import { existsSync, mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { compiler } from './bundle.mjs'
import { atomic, check, confined, load, now, read, save, sha, text } from './io.mjs'
import { locked } from './lock.mjs'
import { afterManifest, loadRun, proposalFiles, runPath } from './proposal.mjs'
import { fresh } from './snapshot.mjs'
import { runProcess } from './process.mjs'

export async function compilerCheck(run, files, prefix, signal) {
  const directory = mkdtempSync(join(tmpdir(), 'cohdl-assist-check-'))
  try {
    for (const entry of run.bundle.snapshot) {
      if (entry.sha256 === null || entry.path.startsWith('explorer/')) continue
      const bytes = read(run.root, entry.path)
      check(sha(bytes) === entry.sha256, 'Source changed while copying compiler inputs')
      atomic(directory, entry.path, bytes)
    }
    for (const file of files) atomic(directory, file.path, file.after)
    const result = await runProcess([compiler(), 'check', directory, '--json'], { cwd: directory, signal })
    atomic(run.root, `${run.path}/${prefix}compiler.stdout.log`, result.stdout)
    atomic(run.root, `${run.path}/${prefix}compiler.stderr.log`, result.stderr)
    let parsed = null
    try {
      parsed = JSON.parse(text(result.stdout))
      check(parsed.schema_version === 1 && ['pass', 'fail'].includes(parsed.verdict) && Array.isArray(parsed.diagnostics), 'Unsupported compiler result schema')
    } catch (error) { result.evidence.error ??= `Invalid compiler JSON: ${error.message}`; parsed = null }
    return { ...result.evidence, result: parsed }
  } finally { rmSync(directory, { recursive: true, force: true }) }
}

export async function inspectValidation(project, directory, prefix = '', signal) {
  const location = runPath(project, directory)
  const metadata = load(location.root, `${location.path}/run.json`)
  const report = { schema: 1, id: location.id, proposal_sha256: metadata.proposal_sha256, bundle_sha256: metadata.bundle_sha256, started_at: now(), finished_at: '', decision: 'FAIL', issues: [], before: [], after: [], compiler: null }
  try {
    const run = loadRun(project, directory)
    report.before = fresh(project, run.bundle)
    const files = proposalFiles(run)
    report.after = afterManifest(report.before, files)
    report.compiler = await compilerCheck(run, files, prefix, signal)
    fresh(project, run.bundle)
    check(report.compiler.finished_at && report.compiler.exit_code === 0 && report.compiler.signal === null && report.compiler.error === null && report.compiler.result?.verdict === 'pass', 'Actual compiler validation failed')
    report.decision = 'PASS'
  } catch (error) { report.issues.push(error.message) }
  report.finished_at = now()
  return report
}

export async function validate(project, directory, destination, signal) {
  const location = runPath(project, directory)
  check(resolve(destination) === join(location.root, location.path, 'validation.json'), 'Validation output must be run validation.json')
  return locked(project, async () => {
    check(!existsSync(confined(location.root, `${location.path}/receipt.json`, { missing: true })), 'Validation is frozen after an apply receipt starts')
    const report = await inspectValidation(project, directory, '', signal)
    save(location.root, `${location.path}/validation.json`, report)
    check(report.decision === 'PASS', report.issues.join('; '))
    return report
  })
}
