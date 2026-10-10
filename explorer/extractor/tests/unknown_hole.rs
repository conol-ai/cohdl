#[path = "../../../tests/common/unknown_hole.rs"]
mod fixture;

#[test]
fn hole_plating_remains_nullable_through_actual_extractor_cli() {
    let dir =
        std::env::temp_dir().join(format!("cohdl-extract-unknown-hole-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        "[package]\nname = \"unknown-hole\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    for (plating, expected) in [
        ("source_unspecified", serde_json::Value::Null),
        ("plated", true.into()),
        ("non_plated", false.into()),
    ] {
        std::fs::write(
            dir.join("src/main.cohdl"),
            fixture::SOURCE.replace("source_unspecified at", &format!("{plating} at")),
        )
        .unwrap();
        let model =
            serde_json::to_value(cohdl_explorer::project_model::extract(&dir).unwrap()).unwrap();
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_cohdl-explorer"))
            .arg(&dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            model,
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()
        );
        assert_eq!(model["verdict"], "pass");
        let fp = model["footprints"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap();
        assert_eq!(fp["mount_holes"][0]["plated"], expected);
        assert_eq!(fp["mount_holes"][0]["size"], serde_json::json!([2.0]));
        assert_eq!(fp["pads"][0]["shape"], "annulus");
        assert_eq!(fp["pads"][0]["layer"], "top_copper");
        assert_eq!(fp["pads"][0]["size"], serde_json::json!([4.0, 2.0]));
        assert_eq!(fp["pads"][0]["number"], "1");
        let pin = &model["instances"][0]["pins"][0];
        assert_eq!(pin["logical"], "CONTACT");
        assert_eq!(pin["nc"], true);
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn actual_copper_layers_survive_extraction() {
    let dir = std::env::temp_dir().join(format!("cohdl-extract-layers-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        "[package]\nname = \"layers\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    for layer in ["top_copper", "bottom_copper", "through_all"] {
        let source = fixture::SOURCE.replace("top_copper", layer);
        let source = if layer == "through_all" {
            source
                .replace(
                    "shape: annulus, size: (4mm, 2mm)",
                    "shape: circle, size: (4mm)",
                )
                .replace(
                    "plating: smd, paste: none",
                    "plating: plated_through_hole, drill: 2mm",
                )
        } else {
            source
        };
        std::fs::write(dir.join("src/main.cohdl"), source).unwrap();
        let model =
            serde_json::to_value(cohdl_explorer::project_model::extract(&dir).unwrap()).unwrap();
        assert_eq!(model["verdict"], "pass", "{model}");
        let fp = model["footprints"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap();
        assert_eq!(fp["pads"][0]["layer"], layer);
        assert_eq!(fp["pads"][0]["pth"], layer == "through_all");
    }
    std::fs::remove_dir_all(dir).unwrap();
}
