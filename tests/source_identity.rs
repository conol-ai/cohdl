#[path = "common/source_identity.rs"]
mod fixture;

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

fn check(src: &str) -> cohdl::pipeline::Checked {
    check_files_in("board", &[("src/main.cohdl".into(), src.into())], None).unwrap()
}

#[test]
fn explicit_asymmetric_blanks_survive_without_collapsing_unknown_parts() {
    let mut c = check(fixture::source());
    assert!(
        build_artifacts(&mut c, &LockState::default()).is_some(),
        "{}",
        c.diags.render(&c.sm)
    );
    let ir = c.ir.as_ref().unwrap();
    let bom = cohdl::emit::bom::emit_bom_csv(&c.world, ir);
    assert_eq!(bom.lines().count(), 5, "{bom}");
    assert!(bom.contains("\"KnownMaker\",\"\""));
    assert!(bom.contains("\"\",\"KnownMPN\""));
    assert!(bom.contains("\"\",\"\",\"U3\",\"FP_A\""), "{bom}");
    assert!(bom.contains("\"\",\"\",\"U4\",\"FP_B\""), "{bom}");
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&c.world, ir, "board").unwrap();
    assert_eq!(xml.matches("<BomItem ").count(), 4);
    assert!(xml.contains("textualCharacteristicName=\"MPN\" textualCharacteristicValue=\"\""));
    let xml_path =
        std::env::temp_dir().join(format!("cohdl-source-identity-{}.xml", std::process::id()));
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
    let formatted = cohdl::fmt::format_source("src/main.cohdl", fixture::source()).unwrap();
    assert_eq!(formatted.matches("#[source_unspecified(").count(), 4);
    assert!(!check(&formatted).diags.has_errors());
    assert_eq!(
        formatted,
        cohdl::fmt::format_source("src/main.cohdl", &formatted).unwrap()
    );
}

#[test]
fn markers_cannot_guess_omit_or_relax_other_constraints() {
    let cases = [
        fixture::source().replace("#[source_unspecified(\"mpn\")]", ""),
        fixture::source().replace(
            "source_unspecified(\"mpn\")",
            "source_unspecified(\"value\")",
        ),
        fixture::source().replace(
            "source_unspecified(\"mpn\")",
            "source_unspecified(\"mpn\",\"mpn\")",
        ),
        fixture::source().replace("source_unspecified(\"mpn\")", "source_unspecified(\"mfr\")"),
        fixture::source().replace(
            "mfr: \"KnownMaker\", mpn: \"\"",
            "mfr: \"KnownMaker\", mpn: \"guessed\"",
        ),
        fixture::source().replace(
            "mfr: \"KnownMaker\", mpn: \"\"",
            "mfr: \"KnownMaker\", mpn: \" \"",
        ),
        fixture::source().replace(
            "mfr: \"KnownMaker\", mpn: \"\", footprint: FP_A",
            "mfr: \"KnownMaker\", mpn: \"\"",
        ),
        fixture::source().replace("mfr: \"KnownMaker\", mpn: \"\"", "mfr: \"KnownMaker\""),
        fixture::source().replace(
            "#[source_unspecified(\"mpn\")]",
            "#[source_unspecified(\"mpn\")]\n#[source_unspecified(\"mpn\")]",
        ),
        fixture::source().replace(
            "pub device D",
            "#[source_unspecified(\"mpn\")]\npub device D",
        ),
        fixture::source().replace(
            "footprint: FP_A } }",
            "footprint: FP_A } alt { mpn: \"\" } }",
        ),
    ];
    for source in cases {
        let c = check(&source);
        assert!(c.diags.has_errors(), "accepted invalid marker: {source}");
    }
}

#[test]
fn source_identity_marker_is_checked_by_actual_cli() {
    let dir = std::env::temp_dir().join(format!("cohdl-identity-cli-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        "[package]\nname = \"identity\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    for (source, success) in [
        (fixture::source().to_string(), true),
        (
            fixture::source().replace("#[source_unspecified(\"mpn\")]", ""),
            false,
        ),
    ] {
        std::fs::write(dir.join("src/main.cohdl"), source).unwrap();
        for cmd in ["check", "build"] {
            let out = std::process::Command::new(env!("CARGO_BIN_EXE_cohdl"))
                .arg(cmd)
                .arg(&dir)
                .args(["--no-std", "--json"])
                .output()
                .unwrap();
            assert_eq!(
                out.status.success(),
                success,
                "{}",
                String::from_utf8_lossy(&out.stdout)
            );
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}
