use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

fn netlist(src: &str) -> String {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "clean build expected:\n{}",
        checked.diags.render(&checked.sm)
    );
    artifacts.expect("build").netlist
}

const LIB: &str = r#"
pub trait Cap { designator_prefix: "C" }
pub device CapDev { pins { A: 1 [passive], B: 2 [passive] } }
impl Cap for CapDev {}
pub footprint FP {}
pub part C100N: CapDev { primary { mfr: "m", mpn: "c", footprint: FP } }
pub device Host { pins { P: 1 [passive], Q: 2 [passive] } }
pub part HOST: Host { primary { mfr: "m", mpn: "h", footprint: FP } }
"#;

#[test]
fn computed_length_index_and_len() {
    let src = format!(
        "{LIB}
design Board {{
    const N: Int = 2 * 2
    inst c: [C100N; N]
    inst h: HOST
    net A: h.P, c[0..=(c.len - 1)].A
    net B: h.Q, c[N - 4].B, c[1, 2, 3].B
}}"
    );
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "{r}");
    assert_eq!(netlist(&src).matches("(comp (ref \"C").count(), 4);
}

#[test]
fn out_of_bounds_computed_index_names_value() {
    let src = format!("{LIB}
design Board {{ const N: Int = 3  inst c: [C100N; N]  inst h: HOST  net A: h.P, c[N].A  nc: h.Q, c[0..=2].B, c[0..=1].A }}");
    let r = check(&src).1;
    assert!(
        r.contains("E202") && r.contains("index 3 is out of bounds for `c`"),
        "{r}"
    );
}

#[test]
fn const_cycle_and_bad_length() {
    let src = format!(
        "{LIB}
design Board {{ const N: Int = c.len  inst c: [C100N; N]  inst h: HOST  net _: h.P, h.Q }}"
    );
    let r = check(&src).1;
    assert!(
        r.contains("E1407") && r.contains("`N`") && r.contains("`c`"),
        "{r}"
    );
    let src = format!(
        "{LIB}
design Board {{ const N: Int = 1 - 1  inst c: [C100N; N]  inst h: HOST  net _: h.P, h.Q }}"
    );
    assert!(check(&src).1.contains("E211"));
}

#[test]
fn computed_placement_and_rotate() {
    let src = format!(
        "{LIB}
design Board {{
    const P: Length = 4mm
    inst c: [C100N; 3]
    inst h: HOST
    net A: h.P, c[0..=2].A
    net B: h.Q, c[0..=2].B
    layout {{
        place c[1] at (10mm + 1 * P, 2mm / 2) rotate 45 * 2
        place c[0] at (1.00mm + 0mm, -1.5mm)
    }}
}}"
    );
    let (mut chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "{r}");
    let art =
        cohdl::pipeline::build_artifacts(&mut chk, &cohdl::lock::LockState::default()).unwrap();
    let layout = art.layout.unwrap();
    assert!(
        layout.contains("\"at\": [14, 1]") || layout.contains("[14, 1]"),
        "{layout}"
    );
    assert!(layout.contains("\"rotate\": 90"), "{layout}");
    // canonical text for a computed value, literal text preserved for a literal
    assert!(
        layout.contains("1mm") || layout.contains("[1, -1.5]"),
        "{layout}"
    );
}

#[test]
fn placement_rotate_out_of_range_and_wrong_kind() {
    let src = format!("{LIB}
design Board {{ inst c: C100N  inst h: HOST  net A: h.P, c.A  net B: h.Q, c.B  layout {{ place c at (0mm, 0mm) rotate 360 }} }}");
    assert!(check(&src).1.contains("E1007"));
    let src = format!("{LIB}
design Board {{ inst c: C100N  inst h: HOST  net A: h.P, c.A  net B: h.Q, c.B  layout {{ place c at (0mm, 2) }} }}");
    assert!(check(&src).1.contains("E1401"), "Int where Length expected");
}

// Constants reject mixed dimensions, a wrong unit, division by zero, and overflow.
#[test]
fn invalid_constant_expressions_report_their_specific_errors() {
    let src = format!(
        "{LIB}
design Board {{
    const A: Int = 10mm + 2
    inst h: HOST
    net _: h.P, h.Q
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1401"), "10mm + 2 must be E1401:\n{r}");

    let src = format!(
        "{LIB}
design Board {{
    const B: Length = 3V
    inst h: HOST
    net _: h.P, h.Q
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1401"), "3V as Length must be E1401:\n{r}");

    let src = format!(
        "{LIB}
design Board {{
    const C: Int = 1 / 0
    inst h: HOST
    net _: h.P, h.Q
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1403"), "1 / 0 must be E1403:\n{r}");

    let src = format!(
        "{LIB}
design Board {{
    const E: Length = 1mm / 3
    inst h: HOST
    net _: h.P, h.Q
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1402"), "1mm / 3 must be E1402:\n{r}");
}
