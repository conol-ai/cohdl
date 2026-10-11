#!/usr/bin/env node
import { prepare } from './bundle.mjs'
import { check } from './io.mjs'
import { validate } from './validation.mjs'
import { apply, revert } from './mutation.mjs'
import { propose } from './provider.mjs'

const help = { schema: 1, command: 'node explorer/assist/cli.mjs', operations: ['prepare PROJECT --selection FILE --out BUNDLE', 'propose BUNDLE --kind group|plan|connect --engine codex --request FILE --out RUN_DIR', 'validate PROJECT RUN_DIR --out RUN_DIR/validation.json', 'apply PROJECT RUN_DIR --validation RUN_DIR/validation.json --out RUN_DIR/receipt.json', 'revert PROJECT RUN_DIR/receipt.json'] }
const abort = new AbortController()
process.once('SIGINT', () => abort.abort())
process.once('SIGTERM', () => abort.abort())

function argumentsFor(args) {
  const positional = []
  const flags = {}
  for (let index = 0; index < args.length; index++) {
    const value = args[index]
    if (!value.startsWith('--')) { positional.push(value); continue }
    check(['--selection', '--out', '--kind', '--engine', '--request', '--validation', '--timeout-ms'].includes(value), `Unknown argument: ${value}`)
    check(!Object.hasOwn(flags, value) && index + 1 < args.length && !args[index + 1].startsWith('--'), `Missing or duplicate flag: ${value}`)
    flags[value] = args[++index]
  }
  return { positional, flags }
}

try {
  const [operation, ...args] = process.argv.slice(2)
  if (operation === '--help' || operation === 'help') console.log(JSON.stringify(help))
  else {
    const { positional, flags } = argumentsFor(args)
    if (operation === 'prepare') {
      check(positional.length === 1 && flags['--selection'] && flags['--out'] && Object.keys(flags).length === 2, 'prepare requires PROJECT --selection FILE --out BUNDLE')
      console.log(JSON.stringify(await prepare(positional[0], flags['--selection'], flags['--out'], abort.signal)))
    } else if (operation === 'propose') {
      check(positional.length === 1 && flags['--kind'] && flags['--engine'] && flags['--request'] && flags['--out'] && Object.keys(flags).length === (flags['--timeout-ms'] ? 5 : 4), 'propose requires BUNDLE --kind KIND --engine ENGINE --request FILE --out RUN_DIR')
      console.log(JSON.stringify(await propose(positional[0], { kind: flags['--kind'], engine: flags['--engine'], requestFile: flags['--request'], directory: flags['--out'], timeout: flags['--timeout-ms'] ? Number(flags['--timeout-ms']) : 180000, signal: abort.signal })))
    } else if (operation === 'validate') {
      check(positional.length === 2 && flags['--out'] && Object.keys(flags).length === 1, 'validate requires PROJECT RUN_DIR --out REPORT')
      console.log(JSON.stringify(await validate(positional[0], positional[1], flags['--out'], abort.signal)))
    } else if (operation === 'apply') {
      check(positional.length === 2 && flags['--validation'] && flags['--out'] && Object.keys(flags).length === 2, 'apply requires PROJECT RUN_DIR --validation REPORT --out RECEIPT')
      console.log(JSON.stringify(await apply(positional[0], positional[1], flags['--validation'], flags['--out'], abort.signal)))
    } else if (operation === 'revert') {
      check(positional.length === 2 && Object.keys(flags).length === 0, 'revert requires PROJECT RECEIPT')
      console.log(JSON.stringify(await revert(positional[0], positional[1], abort.signal)))
    } else throw new Error(`Unknown operation: ${operation ?? '(missing)'}`)
  }
} catch (error) {
  console.log(JSON.stringify({ schema: 1, error: error.message }))
  process.exitCode = 1
}
