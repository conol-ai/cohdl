use cohdl::pipeline::check_files_in;

fn parse_render(src: &str) -> String {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut c = check_files_in("board", &files, None).expect("selection");
    c.diags.sort(&c.sm);
    c.diags.render(&c.sm)
}

const LIB: &str = r#"
pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
"#;

#[test]
fn const_and_for_parse() {
    let src = format!(
        "{LIB}
design Board {{
    const N: Int = 2 + 1
    const PITCH: Length = 4mm
    inst d: [Dev; N]
    for links: n in 0..(N - 1) {{
        net _: d[n].B, d[n + 1].A
    }}
    net IN: d[0].A
    nc: d[N - 1].B
    layout {{
        for grid: n in 0..d.len {{
            place d[n] at (10mm + n * PITCH, -1.5mm)
        }}
    }}
}}"
    );
    let r = parse_render(&src);
    assert!(!r.contains("E010"), "no parse errors expected:\n{r}");
}

#[test]
fn minus_forms() {
    // n-1 and n - 1 are subtraction; -1.00mm is a literal; - 1.00mm is unary.
    let src = format!(
        "{LIB}
design Board {{
    const A: Int = 3
    const B: Int = A-1
    const C: Int = A - 1
    const D: Length = -1.00mm
    const E: Length = - 1.00mm
    const F: Int = -9223372036854775808
    inst d: Dev
    net _: d.A, d.B
}}"
    );
    let r = parse_render(&src);
    assert!(!r.contains("E010") && !r.contains("E001"), "{r}");
}

#[test]
fn negative_voltage_still_e105_and_tolerance_lexing() {
    let src = format!(
        "{LIB}
pub device V<X: Voltage = -5V> {{ pins {{ A: 1 [passive] }} spec {{ v: X }} }}
design Board {{ inst d: Dev  net _: d.A, d.B }}"
    );
    assert!(parse_render(&src).contains("E105"));
    let src2 = format!(
        "{LIB}
design Board {{ const T: Int = 10%3  inst d: Dev  net _: d.A, d.B }}"
    );
    assert!(parse_render(&src2).contains("E010"));
}

#[test]
fn int_generic_param_and_expr_args() {
    let src = format!(
        "{LIB}
fn bank<const N: Int = 2, L: Length>(p: Pin) {{ net _: p }}
design Board {{ inst d: Dev  bank::<1 + 1, 2mm * 2>(d.A)  net _: d.B }}"
    );
    let r = parse_render(&src);
    assert!(!r.contains("E010"), "{r}");
}

#[test]
fn every_loop_is_labelled() {
    let src = format!(
        "{LIB}
design Board {{
    inst d: Dev
    for n in 0..2 {{ net _: d.A }}
    net _: d.B
}}"
    );
    let r = parse_render(&src);
    assert!(
        r.contains("every loop is labelled"),
        "loop label hint missing:\n{r}"
    );
}

#[test]
fn signed_pin_number_still_e102() {
    // Legacy positions keep E102: a bare number may not carry a sign.
    let src = format!(
        "{LIB}
pub device Bad {{ pins {{ A: -1 [passive] }} }}"
    );
    assert!(parse_render(&src).contains("E102"));
}

#[test]
fn half_open_range_and_dotdot_eq() {
    // `..` in for headers; `..=` still parses for net fan-out.
    let src = format!(
        "{LIB}
design Board {{
    inst d: [Dev; 3]
    net _: d[0..=2].A
}}"
    );
    assert!(!parse_render(&src).contains("E010"));
}
