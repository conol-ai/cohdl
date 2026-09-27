//! PR-43 re-review (2026-09-27) regression B: a repeated placement of the
//! SAME resolved target from one loop must report ONE E1007, not one per
//! iteration. The dedup key is the coordinate owner + resolved target —
//! never code/message — so two independent targets still report each, and
//! rotation/unit/geometry E1007 errors per iteration are untouched.

use cohdl::pipeline::check_files_in;

const LIB: &str = r#"
pub device D { pins { A: 1 [passive], B: 2 [passive] } }
pub footprint FP {}
pub part P: D { primary { mfr: "m", mpn: "p", footprint: FP } }
"#;

fn render(body: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".into(), format!("{LIB}\n{body}"))];
    let mut c = check_files_in("board", &files, None).unwrap();
    c.diags.sort(&c.sm);
    let r = c.diags.render(&c.sm);
    (c, r)
}

#[test]
fn loop_placing_same_target_reports_single_e1007() {
    // explicit placement + 50 loop iterations all targeting leds[2]
    let (c, r) = render(
        r#"design B {
    inst leds: [P; 4]
    net _: leds[0..=3].A, leds[0..=3].B
    layout {
        place leds[2] at (0mm, 0mm)
        for grid: i in 0..50 { place leds[2] at (1mm * i, 0mm) }
    }
}"#,
    );
    let e1007: Vec<_> = c.diags.iter().filter(|d| d.code == "E1007").collect();
    assert_eq!(e1007.len(), 1, "one E1007 for one resolved target:\n{r}");
    assert!(r.contains("placed more than once"), "{r}");
    // first conflict keeps the actual target + frame/binder provenance
    let m = &e1007[0].message;
    assert!(
        m.contains("target `B::leds_2`"),
        "resolved target in message:\n{m}"
    );
    assert!(
        m.contains("grid") && m.contains("i = "),
        "frame/binder context:\n{m}"
    );
}

#[test]
fn loop_placing_two_distinct_targets_reports_each() {
    let (c, r) = render(
        r#"design B {
    inst leds: [P; 4]
    net _: leds[0..=3].A, leds[0..=3].B
    layout {
        place leds[1] at (0mm, 0mm)
        place leds[2] at (5mm, 0mm)
        for grid: i in 0..10 {
            place leds[1] at (1mm * i, 0mm)
            place leds[2] at (1mm * i, 5mm)
        }
    }
}"#,
    );
    let targets: Vec<_> = c
        .diags
        .iter()
        .filter(|d| d.code == "E1007")
        .map(|d| d.message.clone())
        .collect();
    assert_eq!(targets.len(), 2, "one E1007 per distinct target:\n{r}");
    assert!(targets.iter().any(|m| m.contains("B::leds_1")), "{r}");
    assert!(targets.iter().any(|m| m.contains("B::leds_2")), "{r}");
}

#[test]
fn rotation_errors_per_iteration_are_not_deduplicated() {
    // out-of-range rotation is a different E1007 class: every iteration's
    // computed angle matters and must NOT be collapsed by the placement dedup
    let (c, r) = render(
        r#"design B {
    inst leds: [P; 2]
    net _: leds[0..=1].A, leds[0..=1].B
    layout {
        for grid: i in 0..3 { place leds[0] at (0mm, 0mm) rotate 360 + i }
    }
}"#,
    );
    let n = c
        .diags
        .iter()
        .filter(|d| d.code == "E1007" && d.message.contains("rotate"))
        .count();
    assert_eq!(n, 3, "each iteration's rotation error survives:\n{r}");
}

#[test]
fn subdesign_default_plus_single_outer_override_is_legal() {
    // a subdesign's internal default and ONE outer reach-in override are a
    // legal pair (override wins for that instantiation) — no E1007 at all
    let (c, r) = render(
        r#"pub subdesign S {
    inst a: P
    net _: a.A, a.B
    layout { place a at (1mm, 1mm) }
}
design B {
    subdesign s: S {}
    layout {
        place s at (10mm, 0mm)
        for grid: i in 0..1 { place s.a at (2mm, 2mm) }
    }
}"#,
    );
    assert!(
        !c.diags
            .iter()
            .any(|d| d.severity == cohdl::diag::Severity::Error),
        "default + single outer override must be error-free:\n{r}"
    );
    let ir = c.ir.as_ref().unwrap();
    let layout = ir
        .layout
        .placements
        .iter()
        .map(|p| (p.path.clone(), p.at.0.femto, p.at.1.femto))
        .collect::<Vec<_>>();
    assert_eq!(
        layout,
        vec![(
            "B::s::a".to_string(),
            2_000_000_000_000_000i128,
            2_000_000_000_000_000i128
        )],
        "complete placement set is exactly the override:\n{layout:?}"
    );
}

#[test]
fn conflicts_in_different_owners_each_report() {
    // the same resolved target spelling placed twice in the SUBDESIGN's
    // default layout and twice in the DESIGN's layout are two distinct
    // owner contexts — both conflicts must survive the dedup key
    let (c, r) = render(
        r#"pub subdesign S {
    inst a: P
    net _: a.A, a.B
    layout {
        for grid: i in 0..3 { place a at (1mm * i, 1mm) }
    }
}
design B {
    subdesign s: S {}
    layout {
        for grid: i in 0..3 { place s.a at (2mm * i, 2mm) }
    }
}"#,
    );
    let e1007: Vec<_> = c.diags.iter().filter(|d| d.code == "E1007").collect();
    assert_eq!(e1007.len(), 2, "one conflict per owner context:\n{r}");
    // same resolved target B::s::a in both owner contexts: the SUBDESIGN's
    // default-layout conflict reports inside the node's own frame, the
    // DESIGN's layout conflict in the design frame — distinct frames prove
    // the owner key separated them rather than double-reporting one owner
    let sub = e1007
        .iter()
        .find(|d| d.message.contains("B::s::__for_grid_1"))
        .unwrap_or_else(|| panic!("subdesign-owner conflict in its own frame:\n{r}"));
    let design = e1007
        .iter()
        .find(|d| d.message.contains("in B::__for_grid_1"))
        .unwrap_or_else(|| panic!("design-owner conflict in the design frame:\n{r}"));
    assert!(
        sub.message.contains("i = 1"),
        "first subdesign conflict iteration:\n{}",
        sub.message
    );
    assert!(
        design.message.contains("i = 1"),
        "first design conflict iteration:\n{}",
        design.message
    );
    assert_eq!(
        c.sm.snippet(sub.primary.span),
        "a",
        "subdesign-layout target spelling"
    );
    assert_eq!(
        c.sm.snippet(design.primary.span),
        "s.a",
        "design-layout reach-in spelling"
    );
}

#[test]
fn five_thousand_iterations_single_report_first_conflict_exact() {
    let body = r#"design B {
    inst leds: [P; 2]
    net _: leds[0..=1].A, leds[0..=1].B
    layout {
        place leds[0] at (0mm, 0mm)
        for grid: i in 0..5000 { place leds[0] at (1mm, 0mm) }
    }
}"#;
    let (c, r) = render(body);
    let e1007: Vec<_> = c.diags.iter().filter(|d| d.code == "E1007").collect();
    assert_eq!(e1007.len(), 1, "5000 iterations, one report:\n{r}");
    let d = e1007[0];
    assert!(d.message.contains("target `B::leds_0`"), "{}", d.message);
    assert!(
        d.message.contains("i = 0"),
        "first conflict is the first iteration:
{}",
        d.message
    );
    // primary span is the placement TARGET path of the first offending
    // iteration (the loop body's `leds[0]`), byte-exact in the source
    let snippet = c.sm.snippet(d.primary.span);
    assert_eq!(snippet, "leds[0]", "span at the placed target: {snippet}");
    let full_source = format!("{LIB}\n{body}");
    let expected_start = full_source.rfind("place leds[0]").unwrap() + "place ".len();
    assert_eq!(
        d.primary.span.start as usize, expected_start,
        "exact span start offset"
    );
    assert_eq!(
        d.primary.span.end as usize,
        expected_start + "leds[0]".len(),
        "exact span end offset"
    );
}

#[test]
fn explicit_duplicate_still_reports_once_per_target() {
    let (c, r) = render(
        r#"design B {
    inst a: P
    inst b: P
    net _: a.A, a.B, b.A, b.B
    layout {
        place a at (0mm, 0mm)
        place a at (1mm, 0mm)
        place b at (0mm, 5mm)
        place b at (1mm, 5mm)
    }
}"#,
    );
    let n = c.diags.iter().filter(|d| d.code == "E1007").count();
    assert_eq!(n, 2, "one report per duplicated target:\n{r}");
}
