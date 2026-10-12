import type { ExplorerModel, Net } from './model'
import { partPresentation } from './partPresentation'
import './OutsideDestinations.css'

export function destinationGroups(model: ExplorerModel, net: Net, instanceRegions: ReadonlyMap<string, string>, visible: ReadonlySet<string>, sourceValues: ReadonlyMap<string, string> = new Map()) {
  const instances = new Map(model.instances.map(instance => [instance.path, instance]))
  const groups = new Map<string, { group: string; local: boolean; navigable: boolean; endpoints: { member: Net['members'][number]; reference: string; identity: string }[] }>()
  for (const member of net.members) {
    const group = instanceRegions.get(member.instance_path) ?? 'Ungrouped'
    const local = visible.has(member.instance_path)
    const key = JSON.stringify([local, group])
    const entry = groups.get(key) ?? { group, local, navigable: instanceRegions.has(member.instance_path), endpoints: [] }
    const instance = instances.get(member.instance_path)
    const identity = instance && partPresentation(instance, sourceValues.get(member.instance_path))
    entry.endpoints.push({ member, reference: identity?.reference ?? 'Unreferenced component', identity: identity?.subtitle ?? '' })
    groups.set(key, entry)
  }
  return [...groups.values()].sort((a, b) => Number(b.local) - Number(a.local) || a.group.localeCompare(b.group))
}

export function OutsideDestinations({ model, net, instanceRegions, visible, sourceValues, onGroup, onEndpoint }: {
  model: ExplorerModel; net: Net; instanceRegions: ReadonlyMap<string, string>; visible: ReadonlySet<string>
  sourceValues?: ReadonlyMap<string, string>; onGroup: (group: string) => void
  onEndpoint: (path: string, pin: string, net: string) => void
}) {
  const groups = destinationGroups(model, net, instanceRegions, visible, sourceValues)
  const outside = groups.filter(group => !group.local).reduce((total, group) => total + group.endpoints.length, 0)
  return <section className="outside-destinations" data-net={net.name} aria-label={`${net.name} destinations`}>
    <h4>{net.name} destinations</h4>
    <div className="destination-counts">{net.members.length} endpoints · {outside} outside range</div>
    <div className="outside-destination-list">
      {groups.map(group => <details key={JSON.stringify([group.local, group.group])} className="destination-group" data-group={group.group} data-local={group.local}>
        <summary><strong>{group.group}</strong><span>{group.local ? 'Local' : 'Outside'} · {group.endpoints.length} endpoints</span></summary>
        {!group.local && group.navigable && <button className="destination-group-command" aria-label={`Open ${group.group} group`} onClick={() => onGroup(group.group)}>Open {group.group} group</button>}
        <ul>{group.endpoints.map((endpoint, index) => <li key={index} data-endpoint={endpoint.member.instance_path} data-visible={group.local} data-instance={endpoint.member.instance_path} data-pin={endpoint.member.logical_pin} data-physical={JSON.stringify(endpoint.member.numbers)}>
          <button className="destination-endpoint" title={`${endpoint.reference}.${endpoint.member.logical_pin} (${endpoint.member.numbers.join(', ')})`} onClick={() => onEndpoint(endpoint.member.instance_path, endpoint.member.logical_pin, net.name)}>
            <span>{endpoint.reference}.{endpoint.member.logical_pin}</span><span className="destination-physical">{endpoint.member.numbers.join(', ') || 'Unspecified'}</span>
          </button>
          {endpoint.identity && <div className="destination-identity">{endpoint.identity}</div>}
        </li>)}</ul>
      </details>)}
      {!groups.length && <p>No retained endpoints</p>}
    </div>
  </section>
}
