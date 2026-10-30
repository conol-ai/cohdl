// Independent hand-written p'=B(-90)*(p-(1mm,2mm)); angle'=angle-90.
pub footprint FP_Molex_43045_0212 {
    pad 1: P_MicroFit3_PTH at (2mm, -1mm)
    pad 2: P_MicroFit3_PTH at (-1mm, -1mm)
    mount_hole 1: non_plated at (-1.94mm, -4mm) diameter 1.02mm
    mount_hole 2: non_plated at (-1.94mm, 2mm) diameter 1.02mm
    silkscreen {
        pin_1_marker near pad 1 shape dot
    }
    courtyard { shape: rect, at: (0.08mm, -1mm), size: (9.78mm, 8.16mm) }
    silkscreen_ref { at: (5.67mm, -1mm) }
}
