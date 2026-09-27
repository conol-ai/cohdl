use cohdl::pipeline::check_files_in;

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
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
fn uncalled_fn_duplicate_local_now_fails() {
    let src = format!(
        "{LIB}
pub fn unused(p: Pin) {{ inst c: C100N  inst c: C100N  net _: p, c.A  nc: c.B }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}"
    );
    assert!(check(&src).1.contains("E201"));
}

#[test]
fn empty_loop_hides_nothing_decidable() {
    let src = format!(
        "{LIB}
pub fn f<const N: Int>(p: Pin) {{ for empty: i in 0..0 {{ const BAD: Int = i / 0  net _: p }} }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}"
    );
    assert!(
        check(&src).1.contains("E1403"),
        "known zero divisor in an uncalled fn's empty loop"
    );
    let src = format!(
        "{LIB}
pub fn g(p: Pin) {{ for empty: i in 0..0 {{ net _: missing.P }} }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}"
    );
    assert!(check(&src).1.contains("E202"));
    let src = format!(
        "{LIB}
pub fn h(p: Pin) {{ for empty: i in 0..0 {{ const X: Int = 1 / i  net _: p }} }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}"
    );
    assert!(!check(&src).1.contains("E1403"), "1 / i waits for a real i");
}

#[test]
fn const_type_and_bounds_kinds() {
    let src = format!(
        "{LIB}
design Board {{ const A: Int = 2mm  inst h: HOST  net _: h.P, h.Q }}"
    );
    assert!(check(&src).1.contains("E1401"));
    let src = format!("{LIB}
design Board {{ const L: Length = 2mm  inst h: HOST  for x: i in 0..L {{ net _: h.P }}  net _: h.Q }}");
    assert!(check(&src).1.contains("E1401"));
}

#[test]
fn skipped_activation_is_not_value_specialized() {
    let src = format!(
        "{LIB}
pub fn f<const N: Int>(p: Pin) {{ for e: i in 0..0 {{ const BAD: Int = 1 / N  net _: p }} }}
design Board {{ inst h: HOST  for none: i in 0..0 {{ f::<0>(h.P) }}  net _: h.P, h.Q }}"
    );
    let r = check(&src).1;
    assert!(
        !r.contains("E1403"),
        "no callee specialization inside a skipped loop:\n{r}"
    );
    let src2 = src.replace("f::<0>(h.P)", "f::<1 / 0>(h.P)");
    assert!(
        check(&src2).1.contains("E1403"),
        "the argument expression itself is checked"
    );
    let src3 = format!(
        "{LIB}
pub fn f<const N: Int>(p: Pin) {{ for e: i in 0..0 {{ const BAD: Int = 1 / N  net _: p }} }}
design Board {{ inst h: HOST  f::<0>(h.P)  net _: h.Q }}"
    );
    assert!(
        check(&src3).1.contains("E1403"),
        "actual activation binds N=0 and fails"
    );
}
