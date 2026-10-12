import assert from 'node:assert/strict'
import { after, test } from 'node:test'
import { mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { createRequire } from 'node:module'
import ts from 'typescript'

const outDir = mkdtempSync(join(tmpdir(), 'cohdl-group-tree-'))
after(() => rmSync(outDir, { recursive: true, force: true }))
const program = ts.createProgram(['src/groupTree.ts'], { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS, outDir, strict: true, skipLibCheck: true, types: [] })
program.emit()
const require = createRequire(import.meta.url)
const { groupNavigation, groupCheck, toggleGroupSelection, memberCount } = require(resolve(outDir, 'groupTree.js'))

const names = ['Core', 'Camera', 'USB', 'Mechanical']
const parents = [{ name: 'Host', groups: ['Core', 'Camera', 'USB'] }, { name: 'Shared', groups: ['USB'] }]
test('tree indexes share canonical children and retain unmatched flat groups', () => {
  assert.deepEqual(groupNavigation(names, parents, ''), [...parents.map(p => ({ ...p, parent: true })), { name: 'Mechanical', groups: ['Mechanical'], parent: false }])
  assert.deepEqual(groupNavigation(names, parents, 'usb').map(p => [p.name, p.groups]), [['Host', ['USB']], ['Shared', ['USB']]])
  assert.deepEqual(groupNavigation(names, parents, 'Host').map(p => p.groups), [['Core', 'Camera', 'USB']])
  assert.deepEqual(groupNavigation(names, undefined, '').map(p => p.name), names)
})
test('parent and child toggles preserve siblings and expose indeterminate without duplicate selections', () => {
  assert.deepEqual(groupCheck(['USB'], parents[0].groups), { checked: false, mixed: true })
  assert.deepEqual(groupCheck(['USB'], ['USB']), { checked: true, mixed: false })
  const selected = toggleGroupSelection(['USB', 'Mechanical'], parents[0].groups)
  assert.deepEqual(selected, ['USB', 'Mechanical', 'Core', 'Camera'])
  assert.deepEqual(toggleGroupSelection(selected, ['USB']), ['Mechanical', 'Core', 'Camera'])
  assert.deepEqual(toggleGroupSelection(selected, parents[0].groups), ['Mechanical'])
})
test('overlapping region and parent counts use physical identity unions', () => {
  const members = new Map([['Core', new Set(['U1', 'C1'])], ['USB', new Set(['U1', 'D1'])]])
  assert.equal(memberCount(['Core', 'USB', 'USB'], members), 3)
  assert.equal(memberCount(['Core'], members), 2)
  assert.equal(memberCount(['Missing'], members), 0)
})
