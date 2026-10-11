import { realpathSync } from 'node:fs'
import { relative, resolve } from 'node:path'
import { check, hash, load, object, read, relativePath, sha, text } from './io.mjs'
import { validateCitations } from './citations.mjs'
import { validateSelection } from './bundle.mjs'

export function runPath(project, directory) {
  const root = realpathSync(project)
  const path = relative(root, resolve(directory))
  check(/^explorer\/proposals\/[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/.test(path), 'Run must be directly inside project explorer/proposals/ID')
  relativePath(path)
  return { root, path, id: path.split('/').at(-1) }
}

export function loadRun(project, directory) {
  const location = runPath(project, directory)
  const metadata = load(location.root, `${location.path}/run.json`)
  const bundleBytes = read(location.root, `${location.path}/bundle.json`)
  const proposalBytes = read(location.root, `${location.path}/proposal.json`)
  check(metadata.schema === 1 && metadata.id === location.id && hash(metadata.bundle_sha256) && hash(metadata.proposal_sha256), 'Invalid run identity or hashes')
  check(metadata.bundle_sha256 === sha(bundleBytes) && metadata.proposal_sha256 === sha(proposalBytes), 'Run artifact byte hash mismatch')
  check(metadata.provider?.engine === 'codex' && metadata.provider.finished_at && metadata.provider.exit_code === 0 && metadata.provider.error === null && metadata.provider.signal === null, 'Provider execution is not successful')
  const bundle = JSON.parse(text(bundleBytes))
  const proposal = JSON.parse(text(proposalBytes))
  check(bundle.schema === 1 && bundle.project === location.root && bundle.model?.verdict === 'pass' && bundle.model.design === bundle.design && Array.isArray(bundle.sources), 'Invalid bound bundle')
  validateSelection(bundle.selection, bundle.model, bundle.views)
  for (const source of bundle.sources) {
    check(typeof source.text === 'string' && sha(source.text) === source.sha256 && bundle.snapshot.some((s) => s.path === source.path && s.sha256 === source.sha256), 'Bundle source text/hash mismatch')
    relativePath(source.path)
  }
  return { ...location, metadata, bundle, proposal }
}

function aggregateRules(model) {
  const rails = new Set(model.derived.rails)
  const rules = new Set()
  for (const path of model.derived.two_terminal) {
    const nets = model.nets.filter((n) => n.members.some((m) => m.instance_path === path)).map((n) => n.name)
    if (!nets.length || nets.some((n) => !rails.has(n))) continue
    const bypass = model.derived.bypasses.find((b) => b.cap === path && model.instances.some((i) => i.path === b.target))
    rules.add(bypass ? `agg:host:${bypass.target}` : `agg:${nets.sort().join(' + ')}`)
  }
  return rules
}

function validateView(bytes, bundle) {
  const value = JSON.parse(bytes)
  check(value.schema_version === 1 && value.design === bundle.design && Array.isArray(value.views) && value.views.length >= 1 && value.views.length <= 128, 'Invalid native view schema/design')
  const names = new Set()
  const identities = new Set(bundle.model.instances.flatMap((i) => [i.designator, `path:${i.path}`]).filter(Boolean))
  for (const rule of aggregateRules(bundle.model)) identities.add(rule)
  const targets = new Set()
  for (const view of value.views) {
    check(typeof view.name === 'string' && view.name.trim() === view.name && view.name.length > 0 && !names.has(view.name), 'Invalid or duplicate view name')
    names.add(view.name)
    check(Array.isArray(view.regions) && view.regions.length <= 512, 'Invalid native group list')
    const regions = new Set()
    for (const region of view.regions) {
      check(typeof region.name === 'string' && region.name.trim() === region.name && region.name.length > 0 && region.name.toLowerCase() !== 'other' && !regions.has(region.name), 'Invalid or duplicate native group name')
      regions.add(region.name)
      targets.add(JSON.stringify([view.name, region.name]))
      check(Array.isArray(region.members) && region.members.length <= 2048 && region.members.every((m) => identities.has(m)), 'Unknown native group member')
    }
  }
  for (const document of bundle.registry?.documents ?? []) {
    for (const application of document.applicability ?? []) {
      for (const group of application.groups ?? []) check(targets.has(JSON.stringify([group.view, group.region])), 'Group edit removes document applicability target')
    }
  }
  for (const constraint of bundle.registry?.constraints ?? []) {
    for (const group of constraint.groups ?? []) check(targets.has(JSON.stringify([group.view, group.region])), 'Group edit removes constraint applicability target')
  }
}

export function proposalFiles(run) {
  const { bundle, proposal, metadata } = run
  check(object(proposal) && Object.keys(proposal).sort().join(',') === 'changes,citations,kind,rationale,schema,title,uncertainty', 'Invalid proposal fields')
  check(proposal.schema === 1 && ['group', 'plan', 'connect'].includes(proposal.kind) && metadata.kind === proposal.kind, 'Proposal kind mismatch')
  for (const name of ['title', 'rationale', 'uncertainty']) check(typeof proposal[name] === 'string' && proposal[name].trim().length > 0 && proposal[name].length <= 65536, `Invalid proposal ${name}`)
  validateCitations(proposal.citations, bundle)
  check(Array.isArray(proposal.changes) && proposal.changes.length <= 64, 'Proposal edit limit exceeded')
  check(proposal.kind === 'plan' ? proposal.changes.length === 0 : proposal.changes.length > 0, 'Plan is read-only; group/connect require edits')
  const seen = new Set()
  return proposal.changes.map((change) => {
    check(object(change) && Object.keys(change).sort().join(',') === 'before_sha256,new,old,path', 'Invalid change fields')
    relativePath(change.path)
    check(!seen.has(change.path), 'Duplicate edited path')
    seen.add(change.path)
    check(proposal.kind === 'group' ? change.path === `views/${bundle.design}.view.json` : /^src\/.+\.cohdl$/.test(change.path), 'Proposal edits outside permitted kind paths')
    check(typeof change.old === 'string' && typeof change.new === 'string' && Buffer.byteLength(change.new) <= 1024 * 1024, 'Invalid bounded replacement text')
    const baseline = bundle.snapshot.find((f) => f.path === change.path)
    check(baseline && baseline.sha256 === change.before_sha256, 'Change before hash mismatch')
    const before = bundle.sources.find((s) => s.path === change.path)?.text ?? null
    let after
    if (before === null) {
      check(proposal.kind === 'group' && change.before_sha256 === null && change.old === '', 'New files require absent native view destination')
      after = change.new
    } else {
      check(hash(change.before_sha256) && change.old.length > 0 && before.split(change.old).length === 2, 'Existing replacement must match exactly once')
      after = before.replace(change.old, change.new)
    }
    check(after !== before && Buffer.byteLength(after) <= 8 * 1024 * 1024, 'Change is empty or exceeds output budget')
    if (proposal.kind === 'group') validateView(after, bundle)
    return { path: change.path, before_sha256: change.before_sha256, after_sha256: sha(after), before, after }
  })
}

export function afterManifest(before, files) {
  return before.map((entry) => {
    const file = files.find((f) => f.path === entry.path)
    return file ? { path: entry.path, sha256: file.after_sha256 } : entry
  })
}
