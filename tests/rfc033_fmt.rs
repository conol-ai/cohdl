use cohdl::fmt::format_source;

const SRC: &str = "pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
design Board {
    const N: Int = (2+1)*2
    inst d: [Dev; N]
    for links: n in 0..N-1 {
        net _: d[n].B, d[n+1].A
    }
    net IN: d[0].A
    nc: d[N - 1].B
    layout {
        for grid: n in 0..d.len {
            place d[n] at (10mm+n*4mm, -1.5mm) rotate 90*n
        }
    }
}
";

#[test]
fn canonical_and_idempotent() {
    let once = format_source("main.cohdl", SRC).unwrap();
    assert!(once.contains("const N: Int = (2 + 1) * 2"));
    assert!(once.contains("for links: n in 0..N - 1 {"));
    assert!(once.contains("net _: d[n].B, d[n + 1].A"));
    assert!(once.contains("place d[n] at (10mm + n * 4mm, -1.5mm) rotate 90 * n"));
    let twice = format_source("main.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}

#[test]
fn parentheses_are_never_dropped() {
    let src = "design Board { const A: Int = (1 + 2) * 3 }\n";
    let out = format_source("main.cohdl", src).unwrap();
    assert!(out.contains("(1 + 2) * 3"));
}

#[test]
fn rotate_zero_is_never_printed() {
    let src = "pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
design Board {
    inst d: Dev
    net _: d.A, d.B
    layout { place d at (0mm, 0mm) rotate 0 }
}
";
    let out = format_source("main.cohdl", src).unwrap();
    assert!(out.contains("place d at (0mm, 0mm)"), "{}", out);
    assert!(
        !out.contains("rotate"),
        "`rotate 0` must not print:\n{}",
        out
    );
}

#[test]
fn nested_layout_loop_and_const_round_trip() {
    let src = "pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
design Board {
    inst d: [Dev; 2]
    layout {
        const P: Length = 2mm
        for grid: n in 0..2 {
            const Q: Int = n + 1
            place d[n] at (P * Q, -1.5mm) rotate 90 * Q
            for inner: m in 0..Q {
                place d[m] at (1mm, 1mm)
            }
        }
    }
}
";
    let once = format_source("main.cohdl", src).unwrap();
    assert!(once.contains("const P: Length = 2mm"), "{}", once);
    assert!(once.contains("for grid: n in 0..2 {"), "{}", once);
    assert!(once.contains("const Q: Int = n + 1"), "{}", once);
    assert!(
        once.contains("place d[n] at (P * Q, -1.5mm) rotate 90 * Q"),
        "{}",
        once
    );
    assert!(once.contains("for inner: m in 0..Q {"), "{}", once);
    let twice = format_source("main.cohdl", &once).unwrap();
    assert_eq!(once, twice, "fmt must be idempotent");
}

#[test]
fn const_int_generic_round_trips() {
    let src = "pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
fn bank<const N: Int = 2, L: Length>(p: Pin) { net _: p }
design Board {
    inst d: Dev
    net _: d.A, d.B
}
";
    let once = format_source("main.cohdl", src).unwrap();
    assert!(once.contains("<const N: Int = 2, L: Length>"), "{}", once);
    let twice = format_source("main.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}

#[test]
fn unary_minus_is_tight_and_literals_keep_spelling() {
    let src = "design Board { const A: Length = - 1.00mm  const B: Int = - (2 + 1) }\n";
    let out = format_source("main.cohdl", src).unwrap();
    // `- 1.00mm` (space) is unary-neg on a literal: unary is tight, the
    // literal's own text keeps its spelling.
    assert!(out.contains("-1.00mm"), "{}", out);
    assert!(out.contains("-(2 + 1)"), "{}", out);
    let twice = format_source("main.cohdl", &out).unwrap();
    assert_eq!(out, twice);
}

#[test]
fn circuit_loop_comments_stay_with_their_statements() {
    let expected = "fn wire(p: Pin, q: Pin) {
    for links: i in 0..2 { // loop header
        // connection
        net _: p, q // wire it
        for nested: j in 0..1 {
            net _: p, q // nested wire
        } // nested end
    } // loop end
}
";
    assert_eq!(format_source("main.cohdl", expected).unwrap(), expected);
}

const ORDERED_LAYOUT: &str = r#"design B {
    inst a: [P; 3]
    net G: a[0..=2].A, a[0..=2].B
    layout {
        // loop placement
        for positions: i in 0..1 { // outer
            // inner placement
            for inner: j in 0..1 {
                place a[j] at (0mm, 0mm) // first
            } // inner end
            // const after loop
            const X: Length = 5mm
            place a[1] at (X, 0mm) // second
        } // outer end
        // constraint after loop
        net_class signal { G }
        // direct placement
        place a[2] at (10mm, 0mm) // third
        const Y: Length = 1mm // last member
    }
}
"#;

#[test]
fn layout_members_and_comments_keep_source_order_in_nested_loops() {
    let once = format_source("main.cohdl", ORDERED_LAYOUT).unwrap();
    assert_eq!(once, ORDERED_LAYOUT);
    assert_eq!(format_source("main.cohdl", &once).unwrap(), once);
}

#[test]
fn same_line_trailing_comments_belong_to_the_last_member() {
    let src = "design B {\n    const A: Int = 1 const B: Int = 2 // second const\n    for links: i in 0..1 { net _: a.A net _: a.B } net _: a.C // final net\n    layout {\n        const X: Length = 1mm place a at (X, 0mm) // place\n        for row: i in 0..1 { place b at (0mm, 0mm) } place c at (1mm, 0mm) // last place\n    }\n}\n";
    let expected = "design B {\n    const A: Int = 1\n    const B: Int = 2 // second const\n    for links: i in 0..1 {\n        net _: a.A\n        net _: a.B\n    }\n    net _: a.C // final net\n    layout {\n        const X: Length = 1mm\n        place a at (X, 0mm) // place\n        for row: i in 0..1 {\n            place b at (0mm, 0mm)\n        }\n        place c at (1mm, 0mm) // last place\n    }\n}\n";
    let once = format_source("main.cohdl", src).unwrap();
    assert_eq!(once, expected);
    assert_eq!(format_source("main.cohdl", &once).unwrap(), once);
}

#[test]
fn ordered_layout_formatting_preserves_complete_build_outputs() {
    use cohdl::lock::LockState;
    use cohdl::pipeline::{build_artifacts, check_files_in};
    let src = format!(
        "pub device D {{ pins {{ A: 1 [passive], B: 2 [passive] }} }}\n\
         pub footprint FP {{}}\n\
         pub part P: D {{ primary {{ mfr: \"m\", mpn: \"p\", footprint: FP }} }}\n\
         {ORDERED_LAYOUT}"
    );
    let formatted = format_source("main.cohdl", &src).unwrap();
    let build = |text: String| {
        let mut checked = check_files_in("board", &[("main.cohdl".into(), text)], None).unwrap();
        assert!(
            !checked.diags.has_errors(),
            "{}",
            checked.diags.render(&checked.sm)
        );
        build_artifacts(&mut checked, &LockState::default())
            .unwrap_or_else(|| panic!("{}", checked.diags.render(&checked.sm)))
    };
    let before = build(src);
    let after = build(formatted);
    assert_eq!(before.netlist, after.netlist);
    assert_eq!(before.bom, after.bom);
    assert!(before.layout.is_some());
    assert_eq!(before.layout, after.layout);
}

#[test]
fn board_outline_keeps_its_position_in_mixed_layout() {
    let src = ORDERED_LAYOUT.replace(
        "        const Y: Length = 1mm // last member",
        "        // external outline\n        board_outline: \"board.dxf\" // outline\n        const Y: Length = 1mm // last member",
    );
    let once = format_source("main.cohdl", &src).unwrap();
    assert_eq!(once, src);
    assert_eq!(format_source("main.cohdl", &once).unwrap(), once);
}
