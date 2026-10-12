import { existsSync, mkdtempSync, realpathSync, rmSync } from 'node:fs'
import { randomUUID } from 'node:crypto'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { prepare } from './bundle.mjs'
import { atomic, check, confined, object, save, sha } from './io.mjs'
import { loadRun, proposalFiles } from './proposal.mjs'
import { fresh } from './snapshot.mjs'
import { propose } from './provider.mjs'
import { validate } from './validation.mjs'
import { apply, revert } from './mutation.mjs'

function checkedRun(project, id) {
  check(typeof id === 'string' && /^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/.test(id), 'Invalid group run ID')
  const directory = join(realpathSync(project), 'explorer/proposals', id)
  const run = loadRun(project, directory)
  check(run.proposal.kind === 'group', 'Display actions require a group proposal')
  return { run, directory }
}

export function focusProposal(project, id) {
  const { run } = checkedRun(project, id)
  fresh(project, run.bundle)
  const config = JSON.parse(proposalFiles(run)[0].after)
  const oldViews = run.bundle.views?.views ?? []
  const groups = config.views.flatMap(view => view.regions.filter(region => {
    const before = oldViews.find(v => v.name === view.name)?.regions.find(r => r.name === region.name)
    return JSON.stringify(before) !== JSON.stringify(region)
  }).map(region => {
    const instances = run.bundle.model.instances.filter(i => region.members.includes(`path:${i.path}`) || (i.designator && region.members.includes(i.designator))).map(i => i.path)
    // Intent previews use exact instances; native aggregate rules remain available
    // through ordinary native view review rather than being silently omitted.
    check(!region.members.some(m => m.startsWith('agg:')), 'Temporary focus requires exact instance members; use native view review for aggregate rules')
    const selected = new Set(instances)
    const nets = run.bundle.model.nets.filter(n => n.members.some(m => selected.has(m.instance_path))).map(n => ({ name: n.name, inside: n.members.filter(m => selected.has(m.instance_path)), outside: n.members.filter(m => !selected.has(m.instance_path)) }))
    return { view: view.name, name: region.name, instances, nets }
  }))
  return { schema: 1, run: id, config, groups, model_sha256: sha(JSON.stringify(run.bundle.model)), rationale: run.proposal.rationale, uncertainty: run.proposal.uncertainty }
}

async function intentProposal(project, request, signal) {
  check(typeof request.intent === 'string' && request.intent.trim().length > 0 && request.intent.length <= 16000, 'Group intent must be nonempty and bounded')
  const temporary = realpathSync(mkdtempSync(join(tmpdir(), 'cohdl-group-intent-')))
  const id = `group-${randomUUID()}`
  try {
    save(temporary, 'selection.json', request.selection ?? { schema: 1, instances: [], groups: [] })
    await prepare(project, join(temporary, 'selection.json'), join(temporary, 'bundle.json'), signal)
    atomic(temporary, 'request.txt', `${request.intent}\nDisplay groups only. Use exact path: instance members. Explain selected original networks, unmatched intent and ambiguity. Preserve existing groups unless explicitly asked to edit/delete them. Optional navigation indexes reference existing canonical group names. No electrical source edits.`)
    return await propose(join(temporary, 'bundle.json'), { kind: 'group', engine: 'codex', requestFile: join(temporary, 'request.txt'), directory: join(realpathSync(project), 'explorer/proposals', id), signal })
  } finally { rmSync(temporary, { recursive: true, force: true }) }
}

export async function groupAction(project, request, signal) {
  check(object(request) && ['propose', 'focus', 'save', 'undo'].includes(request.action), 'Invalid display group action')
  if (request.action === 'propose') return intentProposal(project, request, signal)
  if (request.action === 'focus') return focusProposal(project, request.run)
  const { run, directory } = checkedRun(project, request.run)
  const receipt = join(directory, 'receipt.json')
  if (request.action === 'undo') return revert(project, receipt, signal)
  const validation = join(directory, 'validation.json')
  if (!existsSync(confined(run.root, `${run.path}/validation.json`, { missing: true }))) {
    const report = await validate(project, directory, validation, signal)
    check(report.decision === 'PASS', report.issues.join('; '))
  }
  return apply(project, directory, validation, receipt, signal)
}
