import type { ViewDef } from './views'

export function groupNavigation(names: readonly string[], navigation: ViewDef['navigation'], query: string) {
  const term = query.trim().toLocaleLowerCase()
  const matches = (name: string) => name.toLocaleLowerCase().includes(term)
  const indexed = new Set(navigation?.flatMap(parent => parent.groups))
  const parents = (navigation ?? []).map(parent => ({ name: parent.name, parent: true,
    groups: parent.groups.filter(name => names.includes(name) && (matches(parent.name) || matches(name))) })).filter(parent => parent.groups.length)
  const leaves = names.filter(name => !indexed.has(name) && matches(name)).map(name => ({ name, groups: [name], parent: false }))
  return [...parents, ...leaves]
}

export function groupCheck(selected: readonly string[], groups: readonly string[]) {
  const count = groups.filter(name => selected.includes(name)).length
  return { checked: count > 0 && count === groups.length, mixed: count > 0 && count < groups.length }
}

export function toggleGroupSelection(selected: readonly string[], groups: readonly string[]) {
  return groupCheck(selected, groups).checked ? selected.filter(name => !groups.includes(name)) : [...new Set([...selected, ...groups])]
}

export function memberCount(groups: readonly string[], members: ReadonlyMap<string, ReadonlySet<string>>) {
  return new Set(groups.flatMap(name => [...members.get(name) ?? []])).size
}
