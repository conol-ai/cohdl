//! RFC-034 1a: supplier-clause syntax, AST retention and canonical formatting.
//! E813 semantic validation is intentionally outside this card.

use cohdl::ast::{ItemKind, PartDef, SourceFile};
use cohdl::fmt::format_source;
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
