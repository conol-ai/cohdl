import { existsSync, rmSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { atomic, check, confined, JSON_LIMIT, load, now, read, save, sha } from './io.mjs'
import { locked } from './lock.mjs'
import { afterManifest, loadRun, proposalFiles, runPath } from './proposal.mjs'
import { fresh } from './snapshot.mjs'
import { inspectValidation } from './validation.mjs'

const same = (left, right) => JSON.stringify(left) === JSON.stringify(right)
const currentHash = (root, path) => { const bytes = read(root, path, undefined, true); return bytes === null ? null : sha(bytes) }
const RECEIPT_OPERATION_RESERVE = 65536

function receiptBudget(receipt) {
  check(Buffer.byteLength(JSON.stringify(receipt, null, 2) + '\n') <= JSON_LIMIT - RECEIPT_OPERATION_RESERVE, 'Receipt exceeds reversible JSON byte budget')
}

function manifestsMatch(report, run, files) {
  check(report.schema === 1 && report.id === run.id && report.decision === 'PASS' && report.proposal_sha256 === run.metadata.proposal_sha256 && report.bundle_sha256 === run.metadata.bundle_sha256, 'Saved validation identity or verdict mismatch')
  check(same(report.before, run.bundle.snapshot) && same(report.after, afterManifest(run.bundle.snapshot, files)), 'Saved validation manifests do not match exact proposal')
  check(report.compiler?.exit_code === 0 && report.compiler.result?.verdict === 'pass' && report.compiler.error === null && report.compiler.signal === null, 'Saved compiler validation is not passing')
}

function restoreFile(root, file) {
  check(currentHash(root, file.path) === file.after_sha256, `Rollback refused changed file: ${file.path}`)
  if (file.before === null) rmSync(confined(root, file.path))
  else atomic(root, file.path, file.before)
}

function rollback(root, completed) {
  const errors = []
  for (const file of [...completed].reverse()) {
    try { restoreFile(root, file) } catch (error) { errors.push(error.message) }
  }
  return errors
}

export async function apply(project, directory, validationFile, receiptFile, signal) {
  const location = runPath(project, directory)
  check(resolve(validationFile) === join(location.root, location.path, 'validation.json') && resolve(receiptFile) === join(location.root, location.path, 'receipt.json'), 'Apply requires canonical run validation and receipt paths')
  return locked(project, async () => {
    const run = loadRun(project, directory)
    check(!existsSync(confined(run.root, `${run.path}/receipt.json`, { missing: true })), 'An apply receipt already exists for this operation')
    check(run.proposal.kind !== 'plan', 'Planning proposals are read-only')
    const files = proposalFiles(run)
    const saved = load(run.root, `${run.path}/validation.json`)
    manifestsMatch(saved, run, files)
    fresh(project, run.bundle)
    const receipt = { schema: 1, id: run.id, proposal_sha256: run.metadata.proposal_sha256, bundle_sha256: run.metadata.bundle_sha256, validation_sha256: sha(read(run.root, `${run.path}/validation.json`)), apply_validation: null, before: saved.before, after: saved.after, files, apply: { started_at: now(), finished_at: null, outcome: 'applying', error: null }, revert: null }
    receiptBudget(receipt)
    save(run.root, `${run.path}/receipt.json`, receipt)
    const completed = []
    try {
      receipt.apply_validation = await inspectValidation(project, directory, 'apply-', signal)
      receiptBudget(receipt)
      save(run.root, `${run.path}/receipt.json`, receipt)
      check(receipt.apply_validation.decision === 'PASS', receipt.apply_validation.issues.join('; '))
      manifestsMatch(receipt.apply_validation, run, files)
      for (const file of files) {
        check(!signal?.aborted, 'Apply cancelled')
        fresh(project, run.bundle, afterManifest(receipt.before, completed))
        atomic(run.root, file.path, file.after)
        completed.push(file)
        save(run.root, `${run.path}/receipt.json`, receipt)
      }
      fresh(project, run.bundle, receipt.after)
      receipt.apply.outcome = 'applied'
      receipt.apply.finished_at = now()
      save(run.root, `${run.path}/receipt.json`, receipt)
      return receipt
    } catch (error) {
      const errors = rollback(run.root, completed)
      receipt.apply.outcome = errors.length ? 'interrupted' : 'rolled_back'
      receipt.apply.error = [error.message, ...errors].join('; ')
      receipt.apply.finished_at = now()
      save(run.root, `${run.path}/receipt.json`, receipt)
      throw new Error(receipt.apply.error)
    }
  })
}

function boundReceipt(run, receipt) {
  check(receipt.schema === 1 && receipt.id === run.id && receipt.proposal_sha256 === run.metadata.proposal_sha256 && receipt.bundle_sha256 === run.metadata.bundle_sha256, 'Receipt identity/hash mismatch')
  const files = proposalFiles(run)
  check(same(receipt.files, files) && same(receipt.before, run.bundle.snapshot) && same(receipt.after, afterManifest(receipt.before, files)), 'Receipt file/manifests differ from bound proposal')
  check(receipt.validation_sha256 === sha(read(run.root, `${run.path}/validation.json`)), 'Frozen validation bytes changed after apply')
  manifestsMatch(load(run.root, `${run.path}/validation.json`), run, files)
  check(['applied', 'applying', 'interrupted'].includes(receipt.apply?.outcome), 'Apply operation is not reversible')
  check(receipt.revert?.outcome !== 'reverted', 'Receipt was already reverted')
  return files
}

export async function revert(project, receiptFile, signal) {
  check(resolve(receiptFile).endsWith('/receipt.json'), 'Revert requires canonical receipt.json')
  const directory = resolve(receiptFile, '..')
  const location = runPath(project, directory)
  return locked(project, async () => {
    const run = loadRun(project, directory)
    const receipt = load(run.root, `${run.path}/receipt.json`)
    const files = boundReceipt(run, receipt)
    const outstanding = files.filter((file) => {
      const current = currentHash(run.root, file.path)
      check(current === file.before_sha256 || current === file.after_sha256, `Revert refused changed file hash: ${file.path}`)
      return current === file.after_sha256
    })
    let expected = afterManifest(receipt.before, outstanding)
    fresh(project, run.bundle, expected)
    receiptBudget(receipt)
    receipt.revert = { started_at: receipt.revert?.started_at ?? now(), finished_at: null, outcome: 'reverting', error: null }
    save(run.root, `${run.path}/receipt.json`, receipt)
    try {
      for (const file of [...outstanding].reverse()) {
        check(!signal?.aborted, 'Revert cancelled')
        fresh(project, run.bundle, expected)
        restoreFile(run.root, file)
        expected = expected.map((entry) => entry.path === file.path ? { path: file.path, sha256: file.before_sha256 } : entry)
        save(run.root, `${run.path}/receipt.json`, receipt)
      }
      fresh(project, run.bundle, receipt.before)
      receipt.revert.outcome = 'reverted'
      receipt.revert.finished_at = now()
      save(run.root, `${run.path}/receipt.json`, receipt)
      return receipt
    } catch (error) {
      receipt.revert.outcome = 'interrupted'
      receipt.revert.error = error.message
      receipt.revert.finished_at = now()
      save(run.root, `${run.path}/receipt.json`, receipt)
      throw error
    }
  })
}
