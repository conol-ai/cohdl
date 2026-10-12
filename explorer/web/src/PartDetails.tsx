import { ExternalLink, RefreshCw, Star } from 'lucide-react'
import type { ExplorerModel, Instance } from './model'
import type { DocumentResource } from './documents'
import { partPresentation, sourcePartValues } from './partPresentation'
import { pinConnection } from './pinConnection'
import { FootprintPreview } from './nodes'
import { ProvenanceInspector } from './ProvenanceInspector'
import { SourceExcerpt } from './SourceExcerpt'
import type { PinIdentity } from './PinTarget'
import type { InspectionContext } from './inspection'
import { documentHref } from './documents'
import { OutsideDestinations } from './OutsideDestinations'
import './PartDetails.css'

export function PartDetails({ model, instance, dark, resource, view, groups, visible, locations, selectedPin, selectedNet, marked, onMark, onPin, placement, connectionMode = 'individual', instanceRegions, onGroup, onEndpoint }: {
  model: ExplorerModel; instance: Instance; dark: boolean; resource: DocumentResource; view: string; groups: string[]
  visible: ReadonlySet<string>; locations?: ReadonlyMap<string, string>; selectedPin: string; selectedNet: string; marked: boolean; onMark: () => void
  onPin: (pin: PinIdentity) => void; placement?: React.ReactNode; connectionMode?: InspectionContext['connectionMode']
  instanceRegions?: ReadonlyMap<string, string>; onGroup?: (group: string) => void; onEndpoint?: (path: string, pin: string, net: string) => void
}) {
  const sourceValues = sourcePartValues(resource.registry)
  const identity = partPresentation(instance, sourceValues.get(instance.path))
  const footprint = instance.part?.footprint && model.footprints[instance.part.footprint]
  const pin = instance.pins.find(pin => pin.logical === selectedPin)
  const connection = pin && pinConnection(model, instance.path, pin, visible, selectedNet, locations, connectionMode)
  const network = connection?.net ?? (!pin ? model.nets.find(net => net.name === selectedNet) : undefined)
  const local = network?.members.filter(member => visible.has(member.instance_path)).length ?? 0
  const documents = resource.registry?.documents.filter(document => !['original-schematic', 'original-pcb'].includes(document.id))
    .map(document => ({ document, associations: document.applicability.filter(association => association.instances.includes(instance.path)) }))
    .filter(record => record.associations.length) ?? []
  const searchIdentity = identity.mpn || identity.value
  return <section className="part-details" aria-label="Component details" data-instance={instance.path}>
    <div className="part-heading"><h3>{identity.title}</h3><button className="icon-tool important-toggle" aria-label={marked ? 'Unmark important' : 'Mark important'} aria-pressed={marked} title={marked ? 'Unmark important' : 'Mark important'} onClick={onMark}><Star size={18} fill={marked ? 'currentColor' : 'none'} /></button></div>
    {footprint ? <div className="part-footprint"><FootprintPreview geo={footprint} dark={dark} /><div>{footprint.pads.length} pads · {instance.part?.footprint?.split('::').at(-1)}</div></div> : <div className="part-footprint-unavailable">Footprint unavailable</div>}
    {identity.value && identity.value !== identity.mpn && <div className="part-value">{identity.value}</div>}
    <dl className="part-facts"><div><dt>MPN</dt><dd>{identity.mpn || 'Unspecified'}</dd></div>{instance.part?.mfr && <div><dt>Manufacturer</dt><dd>{instance.part.mfr}</dd></div>}</dl>
    <section className="part-network" aria-label="Selected network" tabIndex={0} data-state={network ? 'connected' : connection?.state} data-net={network?.name ?? ''}>
      <h4>Selected network</h4>
      <strong>{network?.name ?? connection?.state ?? 'No network selected'}</strong>
      {pin && <div className="network-selected-pin">Selected pin: {identity.reference}.{pin.logical} ({pin.numbers.join(', ') || 'Unspecified'})</div>}
      {network ? <>
        <div>{network.members.length} endpoints · {local} local · {network.members.length - local} outside range</div>
        {instanceRegions && onGroup && onEndpoint && <OutsideDestinations model={model} net={network} instanceRegions={instanceRegions} visible={visible} sourceValues={sourceValues} onGroup={onGroup} onEndpoint={onEndpoint} />}
      </> : pin && <div>{pin.nc ? 'Explicitly not connected' : pin.connected ? 'Network membership unavailable' : 'No network connection'}</div>}
    </section>
    <h4>Connections</h4>
    <table className="pin-table"><thead><tr><th>Pin</th><th>#</th><th>Network</th><th>State</th></tr></thead><tbody>
      {instance.pins.map(pin => {
        const connection = pinConnection(model, instance.path, pin, visible, selectedNet, locations, connectionMode)
        const activate = () => onPin({ instance: instance.path, pin: pin.logical, net: connection.net?.name ?? '' })
        return <tr key={pin.logical} className="pin-target" role="button" tabIndex={0} data-pin={pin.logical} data-selected={selectedPin === pin.logical} data-traced={!!selectedNet && connection.net?.name === selectedNet}
          aria-label={`${pin.logical} · ${connection.net?.name ?? connection.state}`} aria-pressed={selectedPin === pin.logical}
          onClick={activate} onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); activate() } }}>
          <td>{pin.logical}</td><td>{pin.numbers.join(', ')}</td><td>{connection.net?.name ?? '—'}</td>
          <td title={connection.net ? `${connection.net.members.length} full endpoints · ${connection.hidden} outside range` : undefined}>{connection.state}</td>
        </tr>
      })}
    </tbody></table>
    <section className="part-documents" aria-label="Documents / Datasheet" aria-busy={resource.loading}>
      <div className="part-document-heading"><h4>Documents / Datasheet</h4><button className="icon-tool" aria-label="Refresh documents" title="Refresh documents" onClick={resource.refresh}><RefreshCw size={14} /></button></div>
      {resource.loading ? <p role="status">Checking documents...</p> : resource.error ? <p role="alert">Documents unavailable: {resource.error}</p> : documents.length ? <ul>
        {documents.map(({ document, associations }) => <li key={document.id} data-document-id={document.id}>
          {associations.map((association, index) => {
            const href = documentHref(document, association.page)
            return <div key={index}>{href ? <a href={href} target="_blank" rel="noreferrer">{document.title}{association.page ? ` · page ${association.page}` : ''}<ExternalLink size={12} /></a> : <span>{document.title} · {document.status === 'missing' ? 'Missing local document' : 'Unavailable'}</span>}
              <small>{association.relation === 'exact' ? 'Exact association' : 'Related evidence'} · {document.review}{association.uncertainty ? ` · ${association.uncertainty}` : ''}</small>
            </div>
          })}
        </li>)}
      </ul> : <p>No associated documents</p>}
      {searchIdentity && <a className="part-datasheets" href={`https://www.google.com/search?q=${encodeURIComponent(searchIdentity + ' datasheet')}`} target="_blank" rel="noreferrer">Datasheet search: {searchIdentity}<ExternalLink size={12} /></a>}
    </section>
    <ProvenanceInspector resource={resource} instance={instance.path} view={view} groups={groups} />
    <details className="part-technical"><summary>Technical details</summary><div>{instance.device_fq}</div><div>{instance.path}</div>
      <div>{instance.impl_traits.join(' · ')}</div><div>{instance.specs.map(s => `${s.name} = ${s.value}`).join(' · ')}</div>
      <SourceExcerpt key={instance.path} span={instance.span} />{placement}
    </details>
  </section>
}
