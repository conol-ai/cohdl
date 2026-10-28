//! RFC-034 1a: supplier-clause syntax, AST retention and canonical formatting.
//! Cards 1a/1b: supplier-clause syntax, formatting and E813 semantics.

use cohdl::ast::{ItemKind, PartDef, SourceFile};
use cohdl::fmt::format_source;
use cohdl::lock::LockState;
use cohdl::pipeline::check_files_in;
use cohdl::span::SourceMap;
use cohdl::units::UnitType;

const LIB: &str = r#"
pub footprint FP {}
pub device D { pins { A: 1 [passive] } }
"#;

fn parse(src: &str) -> (SourceMap, cohdl::diag::Diagnostics, SourceFile) {
    let mut sm = SourceMap::new();
    let file = sm.add_file("src/main.cohdl", src);
    let mut diags = cohdl::diag::Diagnostics::new();
    let tokens = cohdl::lex::lex(file, sm.text(file), &mut diags);
    let ast = cohdl::parse::parse(tokens, &mut diags);
    diags.sort(&sm);
    (sm, diags, ast)
}

fn part<'a>(ast: &'a SourceFile, name: &str) -> &'a PartDef {
    ast.items
        .iter()
        .find_map(|item| match &item.kind {
            ItemKind::Part(p) if p.name.name == name => Some(p),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing part `{name}`"))
}

fn diagnostics(src: &str) -> String {
    let mut checked = check_files_in(
        "board",
        &[("src/main.cohdl".to_string(), src.to_string())],
        None,
    )
    .expect("design selection");
    checked.diags.sort(&checked.sm);
    checked.diags.render(&checked.sm)
}

fn checked(src: &str) -> cohdl::pipeline::Checked {
    let mut checked = check_files_in(
        "board",
        &[("src/main.cohdl".to_string(), src.to_string())],
        None,
    )
    .expect("design selection");
    checked.diags.sort(&checked.sm);
    checked
}

fn entry_source(clause: &str) -> String {
    format!(
        "{LIB}\npub part P: D {{ primary {{ mfr: \"M\", mpn: \"N\", footprint: FP, {clause} }} }}\n"
    )
}

fn e813s(clause: &str) -> Vec<cohdl::diag::Diagnostic> {
    let checked = checked(&entry_source(clause));
    let rendered = checked.diags.render(&checked.sm);
    let diagnostics: Vec<_> = checked
        .diags
        .iter()
        .filter(|diagnostic| diagnostic.code == "E813")
        .cloned()
        .collect();
    assert_eq!(diagnostics.len(), 1, "`{clause}`:\n{rendered}");
    diagnostics
}

#[test]
fn primary_and_alt_retain_bindings_defaults_and_source_order() {
    let src = format!(
        r#"{LIB}
pub part P: D {{
    primary {{
        mfr: "Maker", supplier acme "P-1" at (-1.25mm, 2mm) rotate 135,
        mpn: "MPN", footprint: FP
    }}
    alt {{
        mfr: "Other", mpn: "ALT", supplier bolt "B-2",
        supplier acme "A-2" rotate 359, footprint: FP
    }}
}}
"#
    );
    let (sm, diags, ast) = parse(&src);
    assert!(!diags.has_errors(), "{}", diags.render(&sm));
    let p = part(&ast, "P");

    assert_eq!(p.primary.suppliers.len(), 1);
    let primary = &p.primary.suppliers[0];
    assert_eq!(primary.supplier.name, "acme");
    assert_eq!(primary.part_number, "P-1");
    let (x, y) = primary.at.as_ref().unwrap();
    assert_eq!(x.value.as_ref().unwrap().unit, UnitType::Length);
    assert_eq!(x.text, "-1.25mm");
    assert_eq!(x.value.as_ref().unwrap().femto, -1_250_000_000_000_000);
    assert_eq!(y.text, "2mm");
    assert_eq!(primary.rotate.as_ref().unwrap().0, "135");
    assert!(primary.span.start <= primary.part_number_span.start);
    assert!(primary.part_number_span.end <= primary.span.end);

    assert_eq!(p.alts.len(), 1);
    let alt = &p.alts[0];
    assert_eq!(alt.suppliers.len(), 2);
    assert_eq!(alt.suppliers[0].supplier.name, "bolt");
    assert_eq!(alt.suppliers[1].supplier.name, "acme");
    assert!(alt.suppliers[0].at.is_none());
    assert!(alt.suppliers[0].rotate.is_none());
    assert_eq!(alt.suppliers[1].rotate.as_ref().unwrap().0, "359");
    assert!(alt.suppliers[1].at.is_none());
}

#[test]
fn supplier_is_contextual_and_unknown_avl_keys_remain_e802() {
    let src = format!(
        r#"{LIB}
pub part supplier: D {{
    primary {{ mfr: "M", mpn: "N", supplier: "ordinary field", footprint: FP }}
}}
design Board {{ inst supplier: supplier net N: supplier.A }}
"#
    );
    let (sm, diags, ast) = parse(&src);
    assert!(!diags.has_errors(), "{}", diags.render(&sm));
    let p = part(&ast, "supplier");
    assert!(p.primary.suppliers.is_empty());
    assert_eq!(p.primary.field("supplier").unwrap().value, "ordinary field");
    let rendered = diagnostics(&src);
    assert!(rendered.contains("E802"), "{rendered}");
    assert!(!rendered.contains("E010"), "{rendered}");
}

#[test]
fn malformed_clauses_are_syntax_errors() {
    for clause in [
        "supplier",
        "supplier acme",
        "supplier 123 \"PN\"",
        "supplier acme 123",
        "supplier acme \"PN\" at (0mm, )",
        "supplier acme \"PN\" at [0mm, 0mm]",
        "supplier acme \"PN\" rotate nope",
        "supplier acme \"PN\" stray",
    ] {
        let src = format!(
            "{LIB}\npub part P: D {{ primary {{ mfr: \"M\", mpn: \"N\", footprint: FP, {clause} }} }}\n"
        );
        let (sm, diags, _) = parse(&src);
        let rendered = diags.render(&sm);
        assert!(diags.has_errors(), "accepted `{clause}`");
        assert!(rendered.contains("E010"), "`{clause}`:\n{rendered}");
    }
}

#[test]
fn lexical_unit_diagnostics_reuse_existing_codes() {
    for (literal, code) in [("1zz", "E103"), ("1kC", "E104"), ("1mΩ", "E101")] {
        let src = format!(
            "{LIB}\npub part P: D {{ primary {{ mfr: \"M\", mpn: \"N\", footprint: FP, supplier acme \"PN\" at ({literal}, 0mm) }} }}\n"
        );
        let (sm, diags, _) = parse(&src);
        let rendered = diags.render(&sm);
        assert!(rendered.contains(code), "`{literal}`:\n{rendered}");
    }
}

#[test]
fn semantic_angle_values_survive_parsing_and_formatting() {
    for angle in ["-1", "360", "90.5", "65536"] {
        let src = format!(
            "{LIB}\npub part P: D {{ primary {{ mfr: \"M\", mpn: \"N\", footprint: FP, supplier acme \"PN\" rotate {angle} }} }}\n"
        );
        let (sm, diags, ast) = parse(&src);
        assert!(
            !diags.has_errors(),
            "angle `{angle}`: {}",
            diags.render(&sm)
        );
        assert_eq!(
            part(&ast, "P").primary.suppliers[0]
                .rotate
                .as_ref()
                .unwrap()
                .0,
            angle
        );
        let once = format_source("src/main.cohdl", &src).expect("format angle");
        assert!(once.contains(&format!("rotate {angle}")), "{once}");
        assert_eq!(format_source("src/main.cohdl", &once).unwrap(), once);
    }
}

#[test]
fn explicit_default_mapping_is_not_silently_dropped() {
    let src = format!(
        "{LIB}\npub part P: D {{ primary {{ mfr: \"M\", mpn: \"N\", footprint: FP, supplier acme \"PN\" at (0mm, 0mm) rotate 0 }} }}\n"
    );
    let once = format_source("src/main.cohdl", &src).unwrap();
    assert!(once.contains("at (0mm, 0mm) rotate 0"), "{once}");
    assert_eq!(format_source("src/main.cohdl", &once).unwrap(), once);
}

#[test]
fn fmt_moves_clauses_after_fields_and_round_trips() {
    let src = format!(
        r#"{LIB}
pub part P: D {{
    primary {{
        supplier acme "P-1" at (-1.25mm, 2mm) rotate 135, mpn: "N",
        supplier bolt "B-2", footprint: FP, mfr: "M"
    }}
}}
"#
    );
    let once = format_source("src/main.cohdl", &src).expect("format supplier clauses");
    let primary_start = once.find("primary {").unwrap();
    let primary_end = once[primary_start..].find(" }").unwrap() + primary_start;
    let primary = &once[primary_start..primary_end];
    let supplier = primary.find("supplier acme").unwrap();
    assert!(primary.find("mfr:").unwrap() < supplier);
    assert!(primary.find("mpn:").unwrap() < supplier);
    assert!(primary.find("footprint:").unwrap() < supplier);
    assert!(supplier < primary.find("supplier bolt").unwrap());
    assert!(once.contains("at (-1.25mm, 2mm) rotate 135"));
    assert!(!once.contains("supplier bolt \"B-2\" at"));
    assert!(!once.contains("supplier bolt \"B-2\" rotate"));

    let twice = format_source("src/main.cohdl", &once).expect("formatted source parses");
    assert_eq!(once, twice, "formatting is not idempotent:\n{once}");
    let (sm, diags, round_tripped) = parse(&once);
    assert!(!diags.has_errors(), "{}", diags.render(&sm));
    let p = part(&round_tripped, "P");
    assert_eq!(p.primary.suppliers.len(), 2);
    assert_eq!(p.primary.suppliers[0].part_number, "P-1");
    assert_eq!(p.primary.suppliers[1].part_number, "B-2");
}

#[test]
fn fmt_preserves_comments_around_moved_supplier_clauses() {
    let src = format!(
        r#"{LIB}
pub part P: D {{
    primary {{
        // supplier note
        supplier acme "A very long supplier part number that makes the entry wrap" at (0mm, -1mm) rotate 45, // mapping note
        mfr: "M", // maker note
        mpn: "N",
        // footprint note
        footprint: FP
    }} // entry note
}}
"#
    );
    let once = format_source("src/main.cohdl", &src).expect("format commented entry");
    for comment in [
        "// supplier note",
        "// mapping note",
        "// maker note",
        "// footprint note",
        "// entry note",
    ] {
        assert_eq!(
            once.matches(comment).count(),
            1,
            "missing {comment}:\n{once}"
        );
    }
    assert!(once.find("mfr:").unwrap() < once.find("supplier acme").unwrap());
    let twice = format_source("src/main.cohdl", &once).expect("formatted comments parse");
    assert_eq!(once, twice, "commented formatting is unstable:\n{once}");
}

#[test]
fn source_empty_primary_can_carry_a_binding() {
    let src = format!(
        r#"{LIB}
#[source_unspecified("mpn")]
pub part P: D {{
    primary {{ mfr: "M", mpn: "", footprint: FP, supplier acme "ORDER-1" }}
}}
design Board {{ inst p: P nc: p.A }}
"#
    );
    let rendered = diagnostics(&src);
    assert!(rendered.is_empty(), "{rendered}");
    let formatted = format_source("src/main.cohdl", &src).unwrap();
    assert!(diagnostics(&formatted).is_empty());
}

#[test]
fn entries_without_supplier_clauses_format_exactly_as_before() {
    let src =
        format!("{LIB}\npub part P: D {{ primary {{ mpn: \"N\", footprint: FP, mfr: \"M\" }} }}\n");
    assert_eq!(
        format_source("src/main.cohdl", &src).unwrap(),
        concat!(
            "pub footprint FP {}\n",
            "pub device D {\n",
            "    pins {\n",
            "        required A: 1 [passive]\n",
            "    }\n",
            "}\n",
            "\n",
            "pub part P: D {\n",
            "    primary { mpn: \"N\", footprint: FP, mfr: \"M\" }\n",
            "}\n",
        )
    );
}

#[test]
fn semantic_edge_literals_survive_parse_and_format_without_e813() {
    for (offset, angle) in [
        ("(1, -2)", "999999999999999999999999999"),
        ("(1V, -2mm)", "-90"),
        ("(0mm, 0mm)", "0"),
    ] {
        let src = format!(
            "{LIB}\npub part P: D {{ primary {{ mfr: \"M\", mpn: \"N\", footprint: FP, supplier acme \"PN\" at {offset} rotate {angle} }} }}\n"
        );
        let (sm, diags, _) = parse(&src);
        assert!(!diags.has_errors(), "{}", diags.render(&sm));
        let once =
            format_source("src/main.cohdl", &src).expect("semantic edges remain formattable");
        assert!(
            once.contains(&format!("at {offset} rotate {angle}")),
            "{once}"
        );
        assert_eq!(format_source("src/main.cohdl", &once).unwrap(), once);
    }
}

#[test]
fn supplier_ids_must_have_the_bounded_lowercase_shape() {
    for id in ["_acme", "Acme"] {
        let rendered = diagnostics(&entry_source(&format!("supplier {id} \"PN\"")));
        assert!(rendered.contains("E813"), "`{id}`:\n{rendered}");
        assert!(
            rendered.contains("invalid supplier identifier"),
            "{rendered}"
        );
    }
    let rendered = diagnostics(&entry_source("supplier acme-supply \"PN\""));
    assert!(
        rendered.contains("E010"),
        "malformed ID remains E010: {rendered}"
    );
    assert!(!rendered.contains("E813"), "{rendered}");
    let rendered = diagnostics(&entry_source("supplier 1acme \"PN\""));
    assert!(
        rendered.contains("E010") && rendered.contains("E103"),
        "{rendered}"
    );
    assert!(!rendered.contains("E813"), "{rendered}");
    let id_33 = format!("a{}", "b".repeat(32));
    let rendered = diagnostics(&entry_source(&format!("supplier {id_33} \"PN\"")));
    assert!(rendered.contains("at most 32 bytes"), "{rendered}");

    let id_32 = format!("a{}", "b".repeat(31));
    assert!(diagnostics(&entry_source(&format!("supplier {id_32} \"PN\""))).is_empty());
}

#[test]
fn supplier_part_numbers_check_trimmed_emptiness_utf8_bytes_and_controls() {
    for pn in ["", " \t "] {
        let errors = e813s(&format!("supplier acme \"{pn}\""));
        assert!(errors[0].message.contains("empty after trimming"));
        assert_eq!(
            checked(&entry_source(&format!("supplier acme \"{pn}\"")))
                .world
                .parts
                .values()
                .next()
                .unwrap()
                .primary
                .suppliers[0]
                .part_number,
            pn
        );
    }

    let pn_129 = "p".repeat(129);
    let errors = e813s(&format!("supplier acme \"{pn_129}\""));
    assert!(errors[0].message.contains("129 UTF-8 bytes"));
    let multibyte_130 = "é".repeat(65);
    assert!(
        diagnostics(&entry_source(&format!("supplier acme \"{multibyte_130}\"")))
            .contains("130 UTF-8 bytes")
    );

    for pn in ["P\u{7}N", "P\rN", "P\u{1b}N"] {
        let errors = e813s(&format!("supplier acme \"{pn}\""));
        assert!(errors[0].message.contains("control character"));
    }
    assert!(diagnostics(&entry_source(&format!(
        "supplier acme \"{}\"",
        "p".repeat(128)
    )))
    .is_empty());
}

#[test]
fn an_entry_allows_sixteen_suppliers_and_rejects_seventeen() {
    let clauses = |count| {
        (0..count)
            .map(|i| format!("supplier seller_{i} \"PN-{i}\""))
            .collect::<Vec<_>>()
            .join(", ")
    };
    assert!(diagnostics(&entry_source(&clauses(16))).is_empty());
    let errors = e813s(&clauses(17));
    assert!(errors[0].message.contains("at most 16"));
    assert!(errors[0].message.contains("17 supplier clauses"));
}

#[test]
fn duplicate_suppliers_are_rejected_per_entry_only() {
    let errors = e813s("supplier acme \"PN-1\", supplier acme \"PN-2\"");
    assert!(errors[0].message.contains("duplicate supplier `acme`"));
    assert_eq!(errors[0].secondary.len(), 1);

    let src = format!(
        r#"{LIB}
pub part P: D {{
    primary {{ mfr: "M", mpn: "N", footprint: FP, supplier acme "PN" }}
    alt {{ mfr: "M", mpn: "N", supplier acme "PN" }}
}}
"#
    );
    assert!(diagnostics(&src).is_empty());
}

#[test]
fn a_primary_alt_conflict_reports_both_exact_clause_spans() {
    let src = format!(
        r#"{LIB}
pub part P: D {{
    primary {{ mfr: "M", mpn: "N", footprint: FP, supplier acme "PN" }}
    alt {{ mfr: "M", mpn: "OTHER", supplier acme "PN" }}
}}
"#
    );
    let checked = checked(&src);
    let part = checked.world.parts.values().next().unwrap();
    let diagnostics: Vec<_> = checked
        .diags
        .iter()
        .filter(|diagnostic| diagnostic.code == "E813")
        .collect();
    assert_eq!(
        diagnostics.len(),
        1,
        "{}",
        checked.diags.render(&checked.sm)
    );
    assert_eq!(diagnostics[0].primary.span, part.alts[0].suppliers[0].span);
    assert_eq!(diagnostics[0].secondary.len(), 1);
    assert_eq!(
        diagnostics[0].secondary[0].span,
        part.primary.suppliers[0].span
    );
    assert_eq!(
        checked.sm.snippet(diagnostics[0].primary.span),
        "supplier acme \"PN\""
    );
    assert_eq!(
        checked.sm.snippet(diagnostics[0].secondary[0].span),
        "supplier acme \"PN\""
    );
}

#[test]
fn cross_part_conflicts_include_primary_and_alternate_entries() {
    let src = format!(
        r#"{LIB}
pub part A: D {{ primary {{ mfr: "M", mpn: "A", footprint: FP, supplier acme "PN" }} }}
pub part B: D {{
    primary {{ mfr: "M", mpn: "B", footprint: FP }}
    alt {{ mfr: "Other", mpn: "C", supplier acme "PN" }}
}}
"#
    );
    let rendered = diagnostics(&src);
    assert_eq!(rendered.matches("error[E813]").count(), 1, "{rendered}");
    assert!(
        rendered.contains("part `B` alt 1 entry conflicts with part `A` primary entry"),
        "{rendered}"
    );
    assert!(rendered.contains("first bound here"));
}

#[test]
fn empty_mpns_never_become_one_supplier_identity() {
    let src = format!(
        r#"{LIB}
#[source_unspecified("mpn")]
pub part A: D {{ primary {{ mfr: "M", mpn: "", footprint: FP, supplier acme "PN" }} }}
#[source_unspecified("mpn")]
pub part B: D {{ primary {{ mfr: "Other", mpn: "", footprint: FP, supplier acme "PN" }} }}
#[source_unspecified("mpn")]
pub part C: D {{ primary {{ mfr: "M", mpn: "", footprint: FP, supplier bolt "PN" }} }}
"#
    );
    let rendered = diagnostics(&src);
    assert_eq!(
        rendered.matches("error[E813]").count(),
        1,
        "world parts: {}\n{rendered}",
        checked(&src).world.parts.len()
    );
}

#[test]
fn supplier_pn_and_identity_comparison_is_exactly_as_written() {
    let src = format!(
        r#"{LIB}
pub part A: D {{ primary {{ mfr: "M", mpn: "A", footprint: FP, supplier acme "PN" }} }}
pub part B: D {{ primary {{ mfr: "M", mpn: "B", footprint: FP, supplier acme "pn" }} }}
pub part C: D {{ primary {{ mfr: "M", mpn: "C", footprint: FP, supplier acme " PN" }} }}
pub part E: D {{ primary {{ mfr: "M", mpn: "A", footprint: FP, supplier acme "PN" }} }}
"#
    );
    assert!(diagnostics(&src).is_empty(), "{}", diagnostics(&src));

    let case_conflict = src.replace(
        "mpn: \"B\", footprint: FP, supplier acme \"pn\"",
        "mpn: \"B\", footprint: FP, supplier acme \"PN\"",
    );
    assert!(diagnostics(&case_conflict).contains("E813"));
    let whitespace_conflict = src.replace(
        "mpn: \"C\", footprint: FP, supplier acme \" PN\"",
        "mpn: \"C\", footprint: FP, supplier acme \"PN\"",
    );
    assert!(diagnostics(&whitespace_conflict).contains("E813"));
}

#[test]
fn offsets_require_length_units_and_accept_negative_lengths() {
    for offset in ["1", "-1.5"] {
        let errors = e813s(&format!("supplier acme \"PN\" at ({offset}, 0mm)"));
        assert!(errors[0].message.contains("bare number"));
    }
    let errors = e813s("supplier acme \"PN\" at (0mm, 1V)");
    assert!(errors[0].message.contains("expected Length (`mm`)"));
    assert!(errors[0].message.contains("Voltage"));
    assert!(diagnostics(&entry_source(
        "supplier acme \"PN\" at (-1.5mm, 2mm) rotate 0"
    ))
    .is_empty());
}

#[test]
fn rotations_are_bounded_whole_degrees() {
    for angle in ["360", "-1", "90.5", "999999999999999999999999999"] {
        let errors = e813s(&format!("supplier acme \"PN\" rotate {angle}"));
        if angle == "90.5" || angle.len() > 10 {
            assert!(errors[0].message.contains("whole number"));
        } else {
            assert!(errors[0].message.contains("0..=359"));
        }
    }
    for angle in ["0", "1", "359"] {
        assert!(diagnostics(&entry_source(&format!(
            "supplier acme \"PN\" rotate {angle}"
        )))
        .is_empty());
    }
}

fn netlist(src: &str) -> String {
    let mut checked = checked(src);
    cohdl::pipeline::build_artifacts(&mut checked, &LockState::default())
        .unwrap_or_else(|| panic!("build failed:\n{}", checked.diags.render(&checked.sm)))
        .netlist
}

#[test]
fn kicad_net_emits_only_primary_suppliers_in_declaration_order() {
    let src = r#"
pub footprint FP {}
pub device D { pins { A: 1 [passive] } }
pub part P: D {
    primary { mfr: "M", mpn: "N", footprint: FP,
        supplier acme "A-1", supplier bolt "B-2" }
    alt { mfr: "Other", mpn: "ALT", supplier globex "NOT-PRIMARY" }
}
design Board { inst u: P nc: u.A }
"#;
    assert_eq!(
        netlist(src),
        r#"(export (version D)
  (components
    (comp (ref "U1") (value "D")
      (footprint "board::FP")
      (libsource (lib "cohdl") (part "D") (description ""))
      (fields
        (field (name "MPN") "N")
        (field (name "MFR") "M")
        (field (name "ACME") "A-1")
        (field (name "BOLT") "B-2")
      )
      (sheetpath (names "/") (tstamps "/"))
      (tstamp "00000000")
    )
  )
  (nets
  )
)
"#
    );
}

#[test]
fn kicad_net_supplier_only_entry_still_has_fields_and_old_bytes_are_stable() {
    let src = r#"
pub footprint FP {}
pub device D { pins { A: 1 [passive] } }
#[source_unspecified("mpn", "mfr")]
pub part P: D {
    primary { mfr: "", mpn: "", footprint: FP, supplier acme "A-1" }
}
design Board { inst u: P nc: u.A }
"#;
    let with_supplier = netlist(src);
    assert_eq!(
        with_supplier,
        r#"(export (version D)
  (components
    (comp (ref "U1") (value "D")
      (footprint "board::FP")
      (libsource (lib "cohdl") (part "D") (description ""))
      (fields
        (field (name "MPN") "")
        (field (name "MFR") "")
        (field (name "ACME") "A-1")
      )
      (sheetpath (names "/") (tstamps "/"))
      (tstamp "00000000")
    )
  )
  (nets
  )
)
"#
    );
    assert_eq!(
        with_supplier.replace("        (field (name \"ACME\") \"A-1\")\n", ""),
        netlist(&src.replace(", supplier acme \"A-1\"", "")),
        "the rest of a no-supplier netlist remains byte-identical"
    );
}

#[test]
fn supplier_conflicts_span_the_whole_compiled_package_set() {
    use cohdl::check::check_declarations_in;
    use cohdl::diag::Diagnostics;
    use cohdl::resolve::ModuleInfo;

    let mut sm = SourceMap::new();
    let mut diags = Diagnostics::new();
    let sources = [
        (
            "vendor_a/src/lib.cohdl",
            format!("{LIB}\npub part A: D {{ primary {{ mfr: \"M\", mpn: \"A\", footprint: FP, supplier acme \"PN\" }} }}"),
        ),
        (
            "vendor_b/src/lib.cohdl",
            format!("{LIB}\npub part B: D {{ primary {{ mfr: \"M\", mpn: \"B\", footprint: FP, supplier acme \"PN\" }} }}"),
        ),
    ];
    let parsed = sources
        .iter()
        .map(|(name, source)| {
            let file = sm.add_file(*name, source.clone());
            cohdl::parse::parse(cohdl::lex::lex(file, sm.text(file), &mut diags), &mut diags)
        })
        .collect();
    let modules = [
        ModuleInfo {
            package: "vendor_a".to_string(),
            module: "vendor_a".to_string(),
        },
        ModuleInfo {
            package: "vendor_b".to_string(),
            module: "vendor_b".to_string(),
        },
    ];
    let world = check_declarations_in(parsed, &modules, &mut diags);
    assert_eq!(world.parts.len(), 2);
    diags.sort(&sm);
    let rendered = diags.render(&sm);
    assert_eq!(rendered.matches("error[E813]").count(), 1, "{rendered}");
    assert!(rendered.contains("vendor_a/src/lib.cohdl"), "{rendered}");
    assert!(rendered.contains("vendor_b/src/lib.cohdl"), "{rendered}");
}
