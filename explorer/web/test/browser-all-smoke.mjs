import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { promisify } from 'node:util'

const run = promisify(execFile)
const stages = ['groups', 'union', 'navigation', 'locate', 'documents', 'proposals', 'readability', 'storage']

export async function runAllBrowserSmoke({ script, url, output, args }) {
  assert.ok(!args.includes('--layout-only') && !args.includes('--net-only'), 'All stages cannot omit navigation or locator checks')
  const report = { result: 'RUNNING', url, isolation: 'A fresh browser process and storage context for each sequential stage', stages: [], started: new Date().toISOString() }
  const save = () => writeFileSync(resolve(output, 'all-stages.json'), JSON.stringify(report, null, 2))
  save()
  for (const stage of stages) {
    const stageOutput = resolve(output, stage)
    mkdirSync(stageOutput, { recursive: true })
    const command = [script, '--url', url, '--stage', stage, '--output', stageOutput]
    const started = new Date().toISOString()
    let result
    try {
      result = { ...await run(process.execPath, command, { timeout: 300000, maxBuffer: 24 * 1024 * 1024 }), exit_code: 0 }
    } catch (error) {
      result = { stdout: error.stdout ?? '', stderr: error.stderr ?? String(error), exit_code: Number.isInteger(error.code) ? error.code : null, signal: error.signal ?? null }
    }
    const stdout = resolve(stageOutput, 'stdout.log'), stderr = resolve(stageOutput, 'stderr.log')
    writeFileSync(stdout, result.stdout)
    writeFileSync(stderr, result.stderr)
    const record = { stage, command: [process.execPath, ...command], started, finished: new Date().toISOString(), exit_code: result.exit_code, signal: result.signal ?? null, stdout, stderr }
    report.stages.push(record)
    console.log(JSON.stringify(record))
    if (result.exit_code !== 0) {
      report.result = 'FAIL'
      report.finished = record.finished
      save()
      throw new Error(`Browser stage ${stage} failed; see ${stderr}`)
    }
    save()
  }
  report.result = 'PASS'
  report.finished = new Date().toISOString()
  save()
  console.log(JSON.stringify({ result: report.result, stage: 'all', stages: report.stages.length, output }))
}
