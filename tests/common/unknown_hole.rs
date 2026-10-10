pub const SOURCE: &str = r#"
pub pad SurfaceCopper { shape: annulus, size: (4mm, 2mm), layer: top_copper, plating: smd, paste: none }
pub footprint Screw {
    pad 1: SurfaceCopper at (0mm,0mm)
    mount_hole 1: source_unspecified at (0mm,0mm) diameter 2mm
}
pub device ScrewContact { pins { CONTACT: 1 [passive] } }
#[source_unspecified("mpn", "mfr")]
pub part ScrewPart: ScrewContact { primary { mpn: "", mfr: "", footprint: Screw } }
design Board { inst screw: ScrewPart nc: screw.CONTACT }
"#;
