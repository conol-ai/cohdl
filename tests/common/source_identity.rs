pub fn source() -> &'static str {
    r#"
pub pad Land { shape: rect, size: (1mm,1mm), layer: top_copper, plating: smd }
pub footprint FP_A { pad 1: Land at (0mm,0mm) }
pub footprint FP_B { pad 1: Land at (2mm,0mm) }
pub device D { pins { P: 1 [passive] } }
#[source_unspecified("mpn")]
pub part A: D { primary { mfr: "KnownMaker", mpn: "", footprint: FP_A } }
#[source_unspecified("mfr")]
pub part B: D { primary { mfr: "", mpn: "KnownMPN", footprint: FP_A } }
#[source_unspecified("mpn", "mfr")]
pub part C: D { primary { mfr: "", mpn: "", footprint: FP_A } }
#[source_unspecified("mpn", "mfr")]
pub part E: D { primary { mfr: "", mpn: "", footprint: FP_B } }
design Board { inst a: A inst b: B inst c: C inst e: E nc: a.P,b.P,c.P,e.P }
"#
}
