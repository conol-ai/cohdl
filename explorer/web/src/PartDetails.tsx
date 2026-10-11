import { Star } from 'lucide-react'
import type { ExplorerModel, Instance } from './model'
import type { DocumentResource } from './documents'
import { partPresentation, sourcePartValues } from './partPresentation'
import { pinConnection } from './pinConnection'
import { FootprintPreview } from './nodes'
import { ProvenanceInspector } from './ProvenanceInspector'
import { SourceExcerpt } from './SourceExcerpt'
import type { PinIdentity } from './PinTarget'

export function PartDetails({ model, instance, dark, resource, view, groups, visible, selectedPin, selectedNet, marked, onMark, onPin, placement }: {
  model: ExplorerModel; instance: Instance; dark: boolean; resource: DocumentResource; view: string; groups: string[]
  visible: ReadonlySet<string>; selectedPin: string; selectedNet: string; marked: boolean; onMark: () => void
  onPin: (pin: PinIdentity) => void; placement?: React.ReactNode
}) {
  const identity = partPresentation(instance, sourcePartValues(resource.registry).get(instance.path))
  const footprint = instance.part?.footprint && model.footprints[instance.part.footprint]
  return <section className="part-details" aria-label="Component details" data-instance={instance.path}>
    <div className="part-heading"><h3>{identity.title}</h3><button className="icon-tool important-toggle" aria-label={marked ? 'Unmark important' : 'Mark important'} aria-pressed={marked} title={marked ? 'Unmark important' : 'Mark important'} onClick={onMark}><Star size={18} fill={marked ? 'currentColor' : 'none'} /></button></div>
    {footprint ? <div className="part-footprint"><FootprintPreview geo={footprint} dark={dark} /><div>{footprint.pads.length} pads · {instance.part?.footprint?.split('::').at(-1)}</div></div> : <div className="part-footprint-unavailable">Footprint unavailable</div>}
    {identity.value && identity.value !== identity.mpn && <div className="part-value">{identity.value}</div>}
    <dl className="part-facts"><div><dt>MPN</dt><dd>{identity.mpn || 'Unspecified'}</dd></div>{instance.part?.mfr && <div><dt>Manufacturer</dt><dd>{instance.part.mfr}</dd></div>}</dl>
    <h4>Connections</h4>
    <table className="pin-table"><thead><tr><th>Pin</th><th>#</th><th>Network</th><th>State</th></tr></thead><tbody>
      {instance.pins.map(pin => {
        const connection = pinConnection(model, instance.path, pin, visible, selectedNet)
        const activate = () => onPin({ instance: instance.path, pin: pin.logical, net: connection.net?.name ?? '' })
        return <tr key={pin.logical} className="pin-target" role="button" tabIndex={0} data-pin={pin.logical} data-selected={selectedPin === pin.logical}
          aria-label={`${pin.logical} · ${connection.net?.name ?? connection.state}`} aria-pressed={selectedPin === pin.logical}
          onClick={activate} onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); activate() } }}>
          <td>{pin.logical}</td><td>{pin.numbers.join(', ')}</td><td>{connection.net?.name ?? '—'}</td>
          <td title={connection.net ? `${connection.net.members.length} full endpoints · ${connection.hidden} outside range` : undefined}>{connection.state}</td>
        </tr>
      })}
    </tbody></table>
    <ProvenanceInspector resource={resource} instance={instance.path} view={view} groups={groups} />
    {!!instance.docs.length && <details><summary>Local documents</summary>{instance.docs.filter(d => /\.pdf$/i.test(d.name)).map(d => <div key={d.name}><a href={`/api/file?p=${encodeURIComponent(d.abs)}`} target="_blank" rel="noreferrer">{d.name.split('/').at(-1)}</a></div>)}</details>}
    {identity.mpn && <div className="part-datasheets"><a href={`https://www.google.com/search?q=${encodeURIComponent(identity.mpn + ' datasheet')}`} target="_blank" rel="noreferrer">Datasheet search</a></div>}
    <details className="part-technical"><summary>Technical details</summary><div>{instance.device_fq}</div><div>{instance.path}</div>
      <div>{instance.impl_traits.join(' · ')}</div><div>{instance.specs.map(s => `${s.name} = ${s.value}`).join(' · ')}</div>
      <SourceExcerpt key={instance.path} span={instance.span} />{placement}
    </details>
  </section>
}
