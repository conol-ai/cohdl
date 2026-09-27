use cohdl::pipeline::check_files_in;

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

const LIB: &str = r#"
pub trait Led { designator_prefix: "D" }
pub device LedDev { pins { A: 1 [passive], B: 2 [passive] } }
impl Led for LedDev {}
pub device Host { pins { P: 1 [passive], Q: 2 [passive] } }
pub footprint FP {}
pub part LED: LedDev { primary { mfr: "m", mpn: "led", footprint: FP } }
pub part HOST: Host { primary { mfr: "m", mpn: "host", footprint: FP } }
"#;

// RFC §9's worked example: 2 iterations × 8 work items — passes.
#[test]
fn rfc_worked_example_2x8() {
    let src = format!(
        "{LIB}
design B {{
    inst led: LED
    for links: n in 0..2 {{
        nc: led.B
    }}
    net _: led.A
}}"
    );
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "{r}");
}

#[test]
fn iteration_budget_exact_boundaries() {
    // Every ENTERED iteration counts cumulatively (outer + inner): 1000
    // outer + 1000×99 inner = exactly 100,000 — passes at the limit.
    let src = format!(
        "{LIB}
design B {{
    inst host: HOST
    inst led: LED
    for a: i in 0..1000 {{
        for b: j in 0..99 {{
            nc: led.B
        }}
    }}
    net _: host.P, host.Q, led.A
}}"
    );
    let (chk, r) = check(&src);
    assert!(
        !chk.diags.has_errors(),
        "100,000 iterations is the limit:\n{r}"
    );

    // 1001 × 100 = 100,100 — the 100,001st iteration trips E1405, once.
    let src = format!(
        "{LIB}
design B {{
    inst host: HOST
    inst led: LED
    for a: i in 0..1001 {{
        for b: j in 0..100 {{
            nc: led.B
        }}
    }}
    net _: host.P, host.Q, led.A
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1405"), "{r}");
    assert!(
        r.contains("100,001") || r.contains("100001"),
        "names the failing iteration count:\n{r}"
    );
    assert_eq!(
        r.matches("error[E1405]").count(),
        1,
        "E1405 reports exactly once:\n{r}"
    );
}

#[test]
fn frame_depth_limit_is_64() {
    // 64 nested loops, each iterating once — the frame stack peaks at 64.
    let mut inner = String::new();
    for d in 0..64 {
        inner.push_str(&format!("for d{d}: i{d} in 0..1 {{"));
    }
    inner.push_str("net _: led.A");
    for _ in 0..64 {
        inner.push('}');
    }
    let src = format!(
        "{LIB}
design B {{
    inst led: LED
    {inner}
    nc: led.B
}}"
    );
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "64 frames is the limit:\n{r}");

    // 65 nested loops trip the depth check.
    let mut inner = String::new();
    for d in 0..65 {
        inner.push_str(&format!("for d{d}: i{d} in 0..1 {{"));
    }
    inner.push_str("net _: led.A");
    for _ in 0..65 {
        inner.push('}');
    }
    let src = format!(
        "{LIB}
design B {{
    inst led: LED
    {inner}
    net _: led.A
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1405") && r.contains("frames"), "{r}");
    assert_eq!(r.matches("error[E1405]").count(), 1, "once:\n{r}");
}

#[test]
fn legacy_graph_is_not_metered() {
    // No M2 syntax anywhere in the reachable graph: no metering, hence no
    // E1405 even at silly literal scales (10 literal instances here).
    let src = format!("{LIB}
design B {{
    inst a0: HOST
    inst a1: HOST
    inst a2: HOST
    inst a3: HOST
    inst a4: HOST
    inst a5: HOST
    inst a6: HOST
    inst a7: HOST
    inst a8: HOST
    inst a9: HOST
    net _: a0.P, a0.Q, a1.P, a1.Q, a2.P, a2.Q, a3.P, a3.Q, a4.P, a4.Q, a5.P, a5.Q, a6.P, a6.Q, a7.P, a7.Q, a8.P, a8.Q, a9.P, a9.Q
}}");
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "{r}");
    assert!(!r.contains("E1405"));
}

#[test]
fn uncalled_m2_helper_does_not_activate_metering() {
    // The helper has M2 syntax but is never referenced: no metering.
    let src = format!(
        "{LIB}
pub fn helper(p: Pin) {{ for looped: n in 0..1000001 {{ nc: p }} }}
design B {{
    inst host: HOST
    net _: host.P, host.Q
}}"
    );
    let (chk, r) = check(&src);
    assert!(
        !chk.diags.has_errors(),
        "uncalled M2 fn must not meter:\n{r}"
    );
    assert!(!r.contains("E1405"));
}

#[test]
fn work_item_budget_trips_once() {
    // Referencing the M2 helper ACTIVATES metering; the cheap overflow is a
    // literal array of 1,000,001 elements charged 1 each before allocation.
    let src = format!(
        "{LIB}
pub fn helper<const K: Int>(p: Pin) {{ nc: p }}
design B {{
    inst host: HOST
    inst huge: [LED; 1000001]
    for act: n in 0..1 {{ helper::<3>(host.P) }}
    nc: host.Q
    nc: huge[0].A
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1405"), "1,000,001 work items trips:\n{r}");
    assert_eq!(r.matches("error[E1405]").count(), 1, "exactly once:\n{r}");
}

#[test]
fn literal_placements_do_not_activate_metering() {
    let src = format!("{LIB} design B {{ inst a: HOST nc: a.P, a.Q layout {{ place a at (1mm, -2mm) rotate 90 }} }}");
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "{r}");
    let design = chk.world.designs.values().next().unwrap();
    assert!(!cohdl::check::meter::metering_needed(&chk.world, design));
}

#[test]
fn empty_layout_loops_activate_and_obey_iteration_budget() {
    let (chk, r) = check("design B { layout { for empty: i in 0..0 {} } }");
    assert!(!chk.diags.has_errors(), "{r}");
    let design = chk.world.designs.values().next().unwrap();
    assert!(cohdl::check::meter::metering_needed(&chk.world, design));
    let r = check("design B { layout { for too_many: i in 0..100001 {} } }").1;
    assert_eq!(r.matches("error[E1405]").count(), 1, "{r}");
}

#[test]
fn expanded_net_members_count_before_deduplication() {
    // 20 instances + 21 work items for the initial net + 50,000 * 21
    // for the repeated net. All nets merge, but all authored fanout counts.
    let src = format!(
        "{LIB} design B {{
        inst a: [HOST; 20]
        net _: a[0..=19].P
        for repeat: i in 0..50000 {{ net _: a[0..=19].Q }}
    }}"
    );
    let (chk, r) = check(&src);
    assert_eq!(r.matches("error[E1405]").count(), 1, "{r}");
    assert!(chk.ir.as_ref().is_none_or(|ir| ir.instances.is_empty()));
}

#[test]
fn max_integer_selector_reports_bounds_without_overflowing() {
    for index in [
        "9223372036854775807..=9223372036854775807",
        "(9223372036854775807)..=(9223372036854775807)",
    ] {
        let src = format!("{LIB} design B {{ inst a: [HOST; 1] net _: a[{index}].P nc: a[0].Q }}");
        let r = check(&src).1;
        assert!(r.contains("error[E202]"), "{r}");
    }
}

#[test]
fn computed_range_stride_must_be_positive() {
    for stride in [0, -1] {
        let src = format!("{LIB} design B {{ const S: Int = {stride} inst a: [HOST; 2] net _: a[0..=1 step S].P nc: a[0].Q, a[1].Q }}");
        let r = check(&src).1;
        assert!(r.contains("error[E211]"), "{r}");
    }
}

#[test]
fn largest_array_lengths_fail_before_bulk_allocation() {
    for declaration in [
        "inst a: [HOST; 9223372036854775807]",
        "subdesign a: [Empty; 9223372036854775807]",
    ] {
        let src = format!(
            "{LIB} pub subdesign Empty {{}} design B {{ const ACTIVE: Int = 0 {declaration} }}"
        );
        let (chk, r) = check(&src);
        assert_eq!(r.matches("error[E1405]").count(), 1, "{r}");
        assert!(chk
            .ir
            .as_ref()
            .is_none_or(|ir| ir.instances.is_empty() && ir.subdesigns.is_empty()));
    }
}

#[test]
fn reference_expressions_activate_metering_at_every_call_site() {
    for operation in [
        "helper(a[0 + 0].P)",
        "subdesign s: S { P: a[0 + 0].P }",
        "#[bypass(a[0 + 0].P, 100nF)] inst b: HOST nc: b.P, b.Q",
    ] {
        let src = format!(
            "{LIB} pub fn helper(p: Pin) {{ net _: p }}
            pub subdesign S {{ ports {{ optional P: Pin }} }}
            design B {{ inst a: [HOST; 1] net _: a[0].P nc: a[0].Q {operation} }}"
        );
        let (chk, r) = check(&src);
        assert!(!chk.diags.has_errors(), "{operation}: {r}");
        let design = chk.world.designs.values().next().unwrap();
        assert!(
            cohdl::check::meter::metering_needed(&chk.world, design),
            "{operation}"
        );
    }
}

#[test]
fn budget_failure_cannot_produce_build_artifacts() {
    for declaration in [
        "inst huge: [LED; 1000001]",
        "subdesign huge: [Empty; 1000001]",
    ] {
        let (mut checked, rendered) = check(&format!(
            "{LIB} pub subdesign Empty {{}} design B {{ const ACTIVE: Int = 1 {declaration} }}"
        ));
        assert_eq!(
            checked.diags.iter().map(|d| d.code).collect::<Vec<_>>(),
            ["E1405"],
            "{rendered}"
        );
        assert!(
            cohdl::pipeline::build_artifacts(&mut checked, &cohdl::lock::LockState::default())
                .is_none()
        );
        assert!(checked.ir.as_ref().is_none_or(|ir| ir.instances.is_empty()
            && ir.subdesigns.is_empty()
            && ir.layout.placements.is_empty()));
    }
}

#[test]
fn empty_layout_iteration_limits_are_exact() {
    for count in [100000, 100001] {
        let (c, r) = check(&format!(
            "design B {{ layout {{ for empty: i in 0..{count} {{}} }} }}"
        ));
        let expected = if count == 100000 {
            vec![]
        } else {
            vec!["E1405"]
        };
        assert_eq!(
            c.diags.iter().map(|d| d.code).collect::<Vec<_>>(),
            expected,
            "{r}"
        );
    }
}
