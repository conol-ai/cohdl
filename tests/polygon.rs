#[path = "common/polygon_fixture.rs"]
mod fixture;

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

fn checked(src: &str) -> cohdl::pipeline::Checked {
    check_files_in("board", &[("src/main.cohdl".into(), src.into())], None).unwrap()
}

fn declaration(vertices: &str, extra: &str) -> String {
    format!("pub pad P {{ shape: polygon, size: (2mm, 2mm), layer: top_copper, plating: smd, vertices: [{vertices}] {extra} }} design B {{}}")
}

#[test]
fn actual_native_polygons_build_losslessly_in_both_exporters() {
    let mut c = checked(&fixture::source());
    let artifacts = build_artifacts(&mut c, &LockState::default());
    assert!(artifacts.is_some(), "{}", c.diags.render(&c.sm));
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&c.world, c.ir.as_ref().unwrap()).unwrap();
    let all = mods
        .iter()
        .map(|m| m.2.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(all.matches("(pad \"61\"").count(), 9);
    assert_eq!(all.matches("(size 0.000001 0.000001)").count(), 2);
    for record in fixture::records().as_array().unwrap() {
        for raw in record["pads"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|p| p.starts_with("PAD~POLYGON"))
        {
            let points = fixture::vertices(raw);
            let exact = points
                .iter()
                .map(|(x, y)| {
                    format!(
                        "(xy {} {})",
                        cohdl::emit::geom::mm_femto(*x),
                        cohdl::emit::geom::mm_femto(*y)
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            assert!(all.contains(&format!("(gr_poly (pts {exact}) (width 0) (fill yes))")));
            let def = c
                .world
                .pads
                .values()
                .find(|p| {
                    p.vertices.as_ref().is_some_and(|(v, _)| {
                        v.iter()
                            .map(|(x, y)| (x.femto, y.femto))
                            .collect::<Vec<_>>()
                            == points
                    })
                })
                .unwrap();
            assert_eq!(def.shape.unwrap().0, cohdl::ast::PadShape::Polygon);
        }
    }
    let xml =
        cohdl::emit::ipc2581::emit_ipc2581(&c.world, c.ir.as_ref().unwrap(), "board").unwrap();
    assert!(xml.contains("<Contour><Polygon><PolyBegin"));
    assert!(
        xml.contains("<PolyBegin x=\"0.5998718\" y=\"-0.6001004\"/>"),
        "{xml}"
    );
    let xml_path =
        std::env::temp_dir().join(format!("cohdl-native-polygon-{}.xml", std::process::id()));
    std::fs::write(&xml_path, &xml).unwrap();
    let output = std::process::Command::new("xmllint")
        .args(["--noout", "--schema"])
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/schema/IPC-2581B1.xsd"
        ))
        .arg(&xml_path)
        .output()
        .unwrap();
    std::fs::remove_file(xml_path).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn malformed_polygons_fail_with_specific_diagnostics() {
    let cases = [
        ("", "", "three distinct"),
        ("(0mm,0mm),(1mm,0mm),(2mm,0mm)", "", "zero area"),
        (
            "(0mm,0mm),(2mm,2mm),(0mm,2mm),(2mm,0mm),(3mm,1mm)",
            "",
            "self-intersecting",
        ),
        (
            "(0mm,0mm),(2mm,0mm),(-1mm,0mm),(0mm,2mm)",
            "",
            "overlapping adjacent",
        ),
        (
            "(0mm,0mm),(2mm,0mm),(2mm,2mm),(1mm,0mm),(0mm,2mm)",
            "",
            "self-intersecting",
        ),
        (
            "(0mm,0mm),(2mm,0mm),(2mm,2mm),(2mm,0mm),(0mm,2mm)",
            "",
            "repeated vertex",
        ),
        ("(0mm,0mm),(101mm,0mm),(0mm,1mm)", "", "100mm"),
        ("(0mm,0mm),(1V,0mm),(0mm,1mm)", "", "Length"),
        (
            "(0mm,0mm),(1mm,0mm),(0mm,1mm)",
            ", chamfer: (top_left,0.1mm)",
            "chamfer",
        ),
        (
            "(0mm,0mm),(1mm,0mm),(0mm,1mm)",
            ", mask_expansion: 0.1mm",
            "exact polygon offsets",
        ),
    ];
    for (vertices, extra, diagnostic) in cases {
        let c = checked(&declaration(vertices, extra));
        assert!(c.diags.has_errors(), "accepted {vertices}");
        assert!(
            c.diags.render(&c.sm).contains(diagnostic),
            "{}",
            c.diags.render(&c.sm)
        );
    }
}

#[test]
fn native_polygons_share_exact_board_geometry_on_both_sides() {
    for bottom in [false, true] {
        let source = fixture::source().replace("nc: u1.P, u10.P", &format!("nc: u1.P, u10.P layout {{ place u1 at (0mm,0mm) side {} place u10 at (10mm,0mm) side {} }}", if bottom { "bottom" } else { "top" }, if bottom { "bottom" } else { "top" }));
        let mut c = checked(&source);
        assert!(
            build_artifacts(&mut c, &LockState::default()).is_some(),
            "{}",
            c.diags.render(&c.sm)
        );
        let board =
            cohdl::emit::kicad_pcb::emit_kicad_pcb(&c.world, c.ir.as_ref().unwrap(), "board")
                .unwrap();
        assert_eq!(board.matches("(size 0.000001 0.000001)").count(), 2);
        for record in fixture::records().as_array().unwrap() {
            for raw in record["pads"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str())
                .filter(|p| p.starts_with("PAD~POLYGON"))
            {
                let exact = fixture::vertices(raw)
                    .iter()
                    .map(|(x, y)| {
                        format!(
                            "(xy {} {})",
                            cohdl::emit::geom::mm_femto(*x),
                            cohdl::emit::geom::mm_femto(if bottom { -*y } else { *y })
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                assert!(
                    board.contains(&exact),
                    "missing exact polygon on bottom={bottom}"
                );
            }
        }
    }
}

#[test]
fn valid_collinear_edges_and_repeated_points_are_preserved() {
    let c = checked(&declaration(
        "(-1mm,-1mm),(0mm,-1mm),(1mm,-1mm),(1mm,-1mm),(1mm,1mm),(-1mm,1mm),(-1mm,-1mm)",
        "",
    ));
    assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
    assert_eq!(
        c.world
            .pads
            .values()
            .next()
            .unwrap()
            .vertices
            .as_ref()
            .unwrap()
            .0
            .len(),
        7
    );
}

#[test]
fn geometry_bounds_and_shape_field_contract_fail_closed() {
    let triangle = "(-100mm,-100mm),(100mm,-100mm),(0mm,100mm)";
    let c = checked(&declaration(triangle, ""));
    assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
    for src in [
        declaration(triangle, "").replace("shape: polygon", "shape: rect"),
        declaration(triangle, "").replace(&format!("vertices: [{triangle}]"), ""),
        declaration(&vec!["(0mm,0mm)"; 4097].join(","), ""),
        declaration("(1mm,1mm),(2mm,1mm),(1mm,2mm)", ""),
        declaration(
            "(-0.0000001mm,-1mm),(1mm,-1mm),(1mm,1mm),(-0.0000001mm,1mm)",
            "",
        ),
        declaration(triangle, ", vertices: [(0mm,0mm),(1mm,0mm),(0mm,1mm)]"),
    ] {
        let c = checked(&src);
        assert!(c.diags.has_errors(), "accepted {src}");
    }
}

#[test]
fn physical_export_apis_return_errors_for_unrepresentable_anchor_geometry() {
    let mut c = checked(&fixture::source());
    assert!(build_artifacts(&mut c, &LockState::default()).is_some());
    let pad = c
        .world
        .pads
        .values_mut()
        .find(|p| p.vertices.is_some())
        .unwrap();
    for (x, _) in &mut pad.vertices.as_mut().unwrap().0 {
        x.femto += 10_000_000_000_000_000;
    }
    let ir = c.ir.as_ref().unwrap();
    let error = cohdl::emit::kicad_mod::emit_kicad_mods(&c.world, ir).unwrap_err();
    assert!(error
        .to_string()
        .contains("exact custom-pad export is unsupported"));
    assert!(cohdl::emit::ipc2581::emit_ipc2581(&c.world, ir, "board").is_err());
    assert!(cohdl::emit::kicad_pcb::emit_kicad_pcb(&c.world, ir, "board").is_err());
}

#[test]
fn polygon_format_roundtrip_retains_every_native_point() {
    let source = fixture::source();
    let formatted = cohdl::fmt::format_source("src/main.cohdl", &source).unwrap();
    let c = checked(&formatted);
    assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
    assert_eq!(
        formatted,
        cohdl::fmt::format_source("src/main.cohdl", &formatted).unwrap()
    );
    let before = checked(&source);
    for (name, pad) in &before.world.pads {
        assert_eq!(
            pad.vertices.as_ref().map(|(v, _)| v
                .iter()
                .map(|(x, y)| (x.femto, y.femto))
                .collect::<Vec<_>>()),
            c.world.pads[name].vertices.as_ref().map(|(v, _)| v
                .iter()
                .map(|(x, y)| (x.femto, y.femto))
                .collect::<Vec<_>>())
        );
    }
}

#[test]
fn native_polygon_fixtures_pass_actual_cli_check_and_build() {
    let dir = std::env::temp_dir().join(format!("cohdl-native-polygon-cli-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        "[package]\nname = \"native-polygon\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/main.cohdl"), fixture::source()).unwrap();
    for cmd in ["check", "build"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_cohdl"))
            .arg(cmd)
            .arg(&dir)
            .args(["--no-std", "--json"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::write(
        dir.join("src/main.cohdl"),
        declaration("(0mm,0mm),(1mm,1mm),(0mm,1mm),(1mm,0mm)", ""),
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .arg("check")
        .arg(&dir)
        .args(["--no-std", "--json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    std::fs::remove_dir_all(dir).unwrap();
}
