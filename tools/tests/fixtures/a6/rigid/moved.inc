// Independent hand-written p'=B(-90)*(p-(1mm,2mm)); angle'=angle-90.
pub footprint FP {
    pad 1: P at (0mm, 0mm)
    pad 2: P at (4mm, -2mm)
    courtyard { shape: rect, at: (2mm, -1mm), size: (6mm, 4mm) }
    silkscreen_ref { at: (5mm, -1mm) }
}
