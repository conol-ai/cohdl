//! docs/apidocs.md: the package API documentation JSON (`cohdl docs`).
//!
//! Pins the load-bearing contract: byte-stable output (same source + same
//! dependency set → same bytes, double-run), the schema's exact shape and
//! key order (golden fixture), zero impact on every existing artifact
//! (netlist/BOM/lock bytes untouched by docs generation), per-variant
//! device views, foreign inlining of dependency-owned preview geometry,
//! and a scale run over lib/passive (~9k parts, incl. the legitimately
//! empty generated module).

use cohdl::emit::docsjson::{render, DepMeta, PackageMeta, Rendered};
use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in_with_deps, Checked};

fn std_files() -> Vec<(String, String)> {
    let std_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std/src");
    let mut entries: Vec<_> = std::fs::read_dir(&std_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "cohdl"))
        .collect();
    entries.sort();
    entries
        .into_iter()
        .map(|p| {
            (
                format!("std/{}", p.file_name().unwrap().to_string_lossy()),
                std::fs::read_to_string(&p).unwrap(),
            )
        })
        .collect()
}

fn std_version() -> String {
    let std_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std");
    let (_, manifest) = cohdl::project::peek_manifest(&std_dir).unwrap();
    manifest.version.expect("std manifest pins a version")
}

/// Check `files` (displayed exactly as given — project files must use the
/// `src/…` displays the CLI produces) as package `package` over std.
fn check_pkg(package: &str, files: &[(&str, &str)]) -> Checked {
    let mut all = std_files();
    all.extend(files.iter().map(|(n, c)| (n.to_string(), c.to_string())));
    let checked =
        check_files_in_with_deps(package, &["std".to_string()], &all, None).expect("pipeline runs");
    assert!(
        !checked.diags.has_errors(),
        "fixture must check cleanly:\n{}",
        checked.diags.render(&checked.sm)
    );
    checked
}

fn docs_for(package: &str, version: &str, files: &[(&str, &str)]) -> Rendered {
    let checked = check_pkg(package, files);
    render(
        &checked,
        &PackageMeta {
            name: package,
            version,
            description: None,
            license: None,
            repository: None,
        },
        &[DepMeta {
            name: "std".to_string(),
            version: std_version(),
            src_layout: true,
        }],
    )
}

// ---------------------------------------------------------------------------
// Golden fixture: exact bytes, exact key order.

const GOLDEN_SRC: &str = r#"#[doc("docs/acme.pdf")]
#[intent("a golden-fixture regulator")]
pub device REG<V: Voltage = 5V> {
    pins {
        required VIN: 1 [power_in]
        required GND: 2, 3 [power_in]
        optional NC: 4 [passive]
        required VOUT: 5 [power_out]
    }
    spec { voltage_rating: V }
}

impl IC for REG {}

pub trait Reg: IC {
    designator_prefix: "VR"
    spec { voltage_rating: Voltage }
}

impl Reg for REG {}

fn helper(vin: Pin, gnd: Pin) {
    inst r: REG<5V>
    net _: vin, r.VIN
    net _: gnd, r.GND
}

pub pad P_LEAD {
    shape: rect
    size: (0.6mm, 0.25mm)
    layer: top_copper
    plating: smd
}

pub footprint ACME5 {
    pad 1: P_LEAD at (-0.95mm, -0.65mm)
    pad 2: P_LEAD at (-0.95mm, 0mm)
    pad 3: P_LEAD at (-0.95mm, 0.65mm)
    pad 4: P_LEAD at (0.95mm, 0.65mm) rotate 180
    pad 5: P_LEAD at (0.95mm, -0.65mm)
    silkscreen { pin_1_marker near pad 1 shape dot }
    courtyard { shape: rect, at: (0mm, 0mm), size: (2.6mm, 1.8mm) }
    silkscreen_ref { at: (0mm, -1.4mm) }
}

pub part REG_5V0: REG<5V> {
    primary { mfr: "Acme", mpn: "ACME-REG5", footprint: ACME5 }
    alt     { mpn: "ACME-REG5B" }
}
"#;

const GOLDEN_JSON: &str = r#"{
  "schema_version": 1,
  "generator": "cohdl {VERSION}",
  "package": {
    "name": "acme",
    "version": "1.2.0",
    "root": "acme",
    "description": "Golden fixture",
    "license": "MIT"
  },
  "dependencies": [
    {
      "name": "std",
      "version": "{STD_VERSION}",
      "root": "std"
    }
  ],
  "items": [
    {
      "fq": "acme::ACME5",
      "name": "ACME5",
      "kind": "footprint",
      "pub": true,
      "module": "acme",
      "file": "src/lib.cohdl",
      "line": 35,
      "footprint": {
        "placeholder": false,
        "pads": [
          {
            "number": "1",
            "pad": "acme::P_LEAD",
            "x": "-0.95",
            "y": "-0.65"
          },
          {
            "number": "2",
            "pad": "acme::P_LEAD",
            "x": "-0.95",
            "y": "0"
          },
          {
            "number": "3",
            "pad": "acme::P_LEAD",
            "x": "-0.95",
            "y": "0.65"
          },
          {
            "number": "4",
            "pad": "acme::P_LEAD",
            "x": "0.95",
            "y": "0.65",
            "rotate": 180
          },
          {
            "number": "5",
            "pad": "acme::P_LEAD",
            "x": "0.95",
            "y": "-0.65"
          }
        ],
        "courtyard": {
          "shape": "rect",
          "at": [
            "0",
            "0"
          ],
          "size": [
            "2.6",
            "1.8"
          ]
        },
        "silkscreen_ref": {
          "at": [
            "0",
            "-1.4"
          ]
        },
        "markers": [
          {
            "kind": "pin_1_marker",
            "pad": "1",
            "shape": "dot"
          }
        ],
        "silk": [
          {
            "kind": "circle",
            "at": [
              "-1.75",
              "-0.65"
            ],
            "radius": "0.2",
            "width": "0.1",
            "fill": true
          }
        ]
      }
    },
    {
      "fq": "acme::P_LEAD",
      "name": "P_LEAD",
      "kind": "pad",
      "pub": true,
      "module": "acme",
      "file": "src/lib.cohdl",
      "line": 28,
      "pad": {
        "shape": "rect",
        "size": [
          "0.6",
          "0.25"
        ],
        "layer": "top_copper",
        "plating": "smd"
      }
    },
    {
      "fq": "acme::REG",
      "name": "REG",
      "kind": "device",
      "pub": true,
      "module": "acme",
      "file": "src/lib.cohdl",
      "line": 3,
      "intent": "a golden-fixture regulator",
      "docs": [
        "docs/acme.pdf"
      ],
      "device": {
        "generics": [
          {
            "name": "V",
            "bound": {
              "unit": "Voltage"
            },
            "default": "5V"
          }
        ],
        "designator_prefix": "U",
        "pins": [
          {
            "pins": [
              {
                "name": "VIN",
                "obligation": "required",
                "numbers": [
                  "1"
                ],
                "role": "power_in"
              },
              {
                "name": "GND",
                "obligation": "required",
                "numbers": [
                  "2",
                  "3"
                ],
                "role": "power_in"
              },
              {
                "name": "NC",
                "obligation": "optional",
                "numbers": [
                  "4"
                ],
                "role": "passive"
              },
              {
                "name": "VOUT",
                "obligation": "required",
                "numbers": [
                  "5"
                ],
                "role": "power_out"
              }
            ]
          }
        ],
        "specs": [
          {
            "fields": [
              {
                "name": "voltage_rating",
                "generic": "V"
              }
            ]
          }
        ]
      }
    },
    {
      "fq": "acme::REG_5V0",
      "name": "REG_5V0",
      "kind": "part",
      "pub": true,
      "module": "acme",
      "file": "src/lib.cohdl",
      "line": 46,
      "part": {
        "device": "acme::REG",
        "args": [
          "5V"
        ],
        "primary": {
          "fields": [
            {
              "name": "mfr",
              "value": "Acme"
            },
            {
              "name": "mpn",
              "value": "ACME-REG5"
            }
          ],
          "footprint": "acme::ACME5"
        },
        "alts": [
          {
            "fields": [
              {
                "name": "mpn",
                "value": "ACME-REG5B"
              }
            ]
          }
        ]
      }
    },
    {
      "fq": "acme::Reg",
      "name": "Reg",
      "kind": "trait",
      "pub": true,
      "module": "acme",
      "file": "src/lib.cohdl",
      "line": 15,
      "trait": {
        "super_traits": [
          "std::IC"
        ],
        "designator_prefix": "VR",
        "pins": [],
        "specs": [
          {
            "name": "voltage_rating",
            "type": "Voltage"
          }
        ]
      }
    },
    {
      "fq": "acme::helper",
      "name": "helper",
      "kind": "fn",
      "pub": false,
      "module": "acme",
      "file": "src/lib.cohdl",
      "line": 22,
      "fn": {
        "params": [
          {
            "name": "vin",
            "type": {
              "kind": "pin"
            }
          },
          {
            "name": "gnd",
            "type": {
              "kind": "pin"
            }
          }
        ],
        "insts": [
          {
            "name": "r",
            "type": "acme::REG",
            "args": [
              "5V"
            ]
          }
        ],
        "nets": 2
      }
    }
  ],
  "impls": [
    {
      "trait": "acme::Reg",
      "device": "acme::REG",
      "file": "src/lib.cohdl",
      "line": 20,
      "spec_map": [
        {
          "field": "voltage_rating",
          "spec": "voltage_rating"
        }
      ]
    },
    {
      "trait": "std::IC",
      "device": "acme::REG",
      "file": "src/lib.cohdl",
      "line": 13
    }
  ],
  "foreign": []
}
"#;

#[test]
fn golden_fixture_bytes_are_exact() {
    let checked = check_pkg("acme", &[("src/lib.cohdl", GOLDEN_SRC)]);
    let rendered = render(
        &checked,
        &PackageMeta {
            name: "acme",
            version: "1.2.0",
            description: Some("Golden fixture"),
            license: Some("MIT"),
            repository: None,
        },
        &[DepMeta {
            name: "std".to_string(),
            version: std_version(),
            src_layout: true,
        }],
    );
    let expected = GOLDEN_JSON
        .replace("{VERSION}", env!("CARGO_PKG_VERSION"))
        .replace("{STD_VERSION}", &std_version());
    assert_eq!(rendered.json, expected, "golden bytes must be exact");
    assert_eq!(rendered.items, 6);
}

#[test]
fn output_is_deterministic_across_fresh_pipelines() {
    let a = docs_for("acme", "1.2.0", &[("src/lib.cohdl", GOLDEN_SRC)]);
    let b = docs_for("acme", "1.2.0", &[("src/lib.cohdl", GOLDEN_SRC)]);
    assert_eq!(a.json, b.json, "same source → same bytes");
}

// ---------------------------------------------------------------------------
// Zero impact: docs generation never changes an existing artifact.

const DESIGN_SRC: &str = r#"
#[doc("docs/board.md")]
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
impl TwoTerminal for Res {}
pub footprint TFP {}
#[intent("the one part")]
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: TFP } }
design B {
    inst r1: R1
    inst r2: R1
    net X: r1.A, r2.A
    net Y: r1.B, r2.B
}
"#;

#[test]
fn docs_generation_is_zero_impact_on_build_artifacts() {
    let mut before = check_pkg("board", &[("src/main.cohdl", DESIGN_SRC)]);
    let baseline = build_artifacts(&mut before, &LockState::default()).expect("build succeeds");

    // A fresh pipeline that ALSO renders docs must produce identical
    // artifact bytes — the docs JSON is a new artifact, never an influence.
    let mut with_docs = check_pkg("board", &[("src/main.cohdl", DESIGN_SRC)]);
    let rendered = render(
        &with_docs,
        &PackageMeta {
            name: "board",
            version: "0.1.0",
            description: None,
            license: None,
            repository: None,
        },
        &[DepMeta {
            name: "std".to_string(),
            version: std_version(),
            src_layout: true,
        }],
    );
    assert!(rendered.items > 0);
    let after = build_artifacts(&mut with_docs, &LockState::default()).expect("build succeeds");
    assert_eq!(baseline.netlist, after.netlist, "netlist bytes untouched");
    assert_eq!(baseline.bom, after.bom, "BOM bytes untouched");
    assert_eq!(
        baseline.lock.render(),
        after.lock.render(),
        "lock bytes untouched"
    );

    // The design itself appears as an item with its body summary.
    assert!(rendered.json.contains("\"kind\": \"design\""));
    assert!(rendered.json.contains("\"nets\": 2"));
}

// ---------------------------------------------------------------------------
// Variants: per-variant render-ready views with RFC-008 merged specs.

const VARIANT_SRC: &str = r#"
pub device Buck {
    variants { QFN16, TSOT25 }
    pins[QFN16] { required VIN: 1 [power_in] required GND: 2 [power_in] }
    pins[TSOT25] { required VIN: 5 [power_in] required GND: 4 [power_in] }
    spec { voltage_rating: 6V }
    spec[TSOT25] { voltage_rating: 5V }
}
"#;

#[test]
fn varianted_devices_render_one_view_per_variant() {
    let out = docs_for("vpkg", "0.1.0", &[("src/lib.cohdl", VARIANT_SRC)]).json;
    // Both variants present, in declaration order, with merged specs — the
    // TSOT25 view carries the override, the QFN16 view the base value.
    let qfn = out.find("\"variant\": \"QFN16\"").expect("QFN16 view");
    let tsot = out.find("\"variant\": \"TSOT25\"").expect("TSOT25 view");
    assert!(qfn < tsot, "views follow declaration order");
    assert!(out.contains("\"variants\": [\n          \"QFN16\",\n          \"TSOT25\"\n        ]"));
    let specs_at = out.rfind("\"specs\"").unwrap();
    let specs = &out[specs_at..];
    assert!(specs.contains("\"value\": \"6V\""), "base spec view");
    assert!(specs.contains("\"value\": \"5V\""), "override spec view");
}

// ---------------------------------------------------------------------------
// Foreign inlining: dependency-owned pads/footprints/devices the previews
// need, with tar-relative file paths.

const DEP_PKG: &str = r#"
pub device Chip { pins { required IN: 1 [input] required OUT: 2 [output] } }
pub pad P_X { shape: circle size: (0.5mm) layer: top_copper plating: smd }
pub footprint FP_X {
    pad 1: P_X at (-1mm, 0mm)
    pad 2: P_X at (1mm, 0mm)
}
"#;

const CONSUMER_PKG: &str = r#"
pub part CHIP_A: lands::Chip {
    primary { mfr: "m", mpn: "CHIP-A", footprint: lands::FP_X }
}
"#;

#[test]
fn foreign_items_inline_dependency_geometry() {
    let mut files = std_files();
    files.push(("lands/lib.cohdl".to_string(), DEP_PKG.to_string()));
    files.push(("src/parts.cohdl".to_string(), CONSUMER_PKG.to_string()));
    let checked = check_files_in_with_deps(
        "consumer",
        &["std".to_string(), "lands".to_string()],
        &files,
        None,
    )
    .expect("pipeline runs");
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    let out = render(
        &checked,
        &PackageMeta {
            name: "consumer",
            version: "0.1.0",
            description: None,
            license: None,
            repository: None,
        },
        &[
            DepMeta {
                name: "std".to_string(),
                version: std_version(),
                src_layout: true,
            },
            DepMeta {
                name: "lands".to_string(),
                version: "1.0.0".to_string(),
                src_layout: true,
            },
        ],
    );
    // The part is the only local item; the referenced device, footprint, and
    // the footprint's pad are inlined under `foreign`, fq-sorted, with the
    // file path as it exists inside the dependency's own tar.
    assert_eq!(out.items, 1);
    let foreign_at = out.json.find("\"foreign\"").unwrap();
    let foreign = &out.json[foreign_at..];
    let chip = foreign
        .find("\"fq\": \"lands::Chip\"")
        .expect("device inlined");
    let fp = foreign
        .find("\"fq\": \"lands::FP_X\"")
        .expect("footprint inlined");
    let pad = foreign.find("\"fq\": \"lands::P_X\"").expect("pad inlined");
    assert!(chip < fp && fp < pad, "foreign is fq-sorted");
    assert!(foreign.contains("\"file\": \"src/lib.cohdl\""));
    // The dependency map lets the UI link foreign roots back to packages.
    assert!(out.json.contains("\"name\": \"lands\""));
    assert!(out.json.contains("\"root\": \"lands\""));
}

// ---------------------------------------------------------------------------
// Placeholder footprints and empty packages.

#[test]
fn placeholder_footprint_and_empty_package() {
    let out = docs_for(
        "ph",
        "0.1.0",
        &[("src/lib.cohdl", "pub footprint LATER {}\n")],
    );
    assert!(out
        .json
        .contains("\"footprint\": {\n        \"placeholder\": true\n      }"));

    let empty = docs_for("nothing", "0.1.0", &[("src/lib.cohdl", "// empty\n")]);
    assert_eq!(empty.items, 0);
    assert!(empty.json.contains("\"items\": [],"));
}

// ---------------------------------------------------------------------------
// Scale: lib/passive is the stress case (~9k parts, one empty generated
// module, generated #[doc] attrs, fns, real footprints).

#[test]
fn lib_passive_scales_and_stays_deterministic() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let deps = vec![("std".to_string(), root.join("lib/std"))];
    let proj = cohdl::project::load_project_with_deps(&root.join("lib/passive"), &deps)
        .expect("lib/passive loads");
    let names: Vec<String> = deps.iter().map(|(n, _)| n.clone()).collect();
    let render_once = || {
        let checked =
            check_files_in_with_deps(&proj.name, &names, &proj.files, None).expect("pipeline runs");
        assert!(
            !checked.diags.has_errors(),
            "{}",
            checked.diags.render(&checked.sm)
        );
        render(
            &checked,
            &PackageMeta {
                name: "passive",
                version: "0.0.0",
                description: None,
                license: None,
                repository: None,
            },
            &[DepMeta {
                name: "std".to_string(),
                version: std_version(),
                src_layout: true,
            }],
        )
    };
    let a = render_once();
    let b = render_once();
    assert_eq!(a.json, b.json, "9k-part output is byte-stable");
    assert!(
        a.items > 8900,
        "passive should document ~9k items, got {}",
        a.items
    );
    assert!(
        a.json.len() < 16 * 1024 * 1024,
        "must fit the registry's 16 MiB sidecar cap ({} bytes)",
        a.json.len()
    );
}

// ---------------------------------------------------------------------------
// Platform-independence: Windows display names use `\` separators; the
// emitter must normalize so impls/designs are never dropped and every
// `file` field is a `/`-separated tar path (same bytes on every platform).

#[test]
fn backslash_displays_normalize_to_tar_paths() {
    let src = r#"
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
impl TwoTerminal for Res {}
design B {
    inst r: Res
    net _: r.A, r.B
}
"#;
    let mut files = std_files();
    files.push(("src\\one.cohdl".to_string(), src.to_string()));
    let checked = check_files_in_with_deps("winpkg", &["std".to_string()], &files, None)
        .expect("pipeline runs");
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    let out = render(
        &checked,
        &PackageMeta {
            name: "winpkg",
            version: "0.1.0",
            description: None,
            license: None,
            repository: None,
        },
        &[DepMeta {
            name: "std".to_string(),
            version: std_version(),
            src_layout: true,
        }],
    );
    assert!(
        out.json.contains("\"kind\": \"design\""),
        "design must survive a backslash display"
    );
    assert!(
        out.json.contains("\"trait\": \"std::TwoTerminal\""),
        "impl must survive a backslash display"
    );
    assert!(
        out.json.contains("\"file\": \"src/one.cohdl\""),
        "file fields are /-separated tar paths"
    );
    assert!(!out.json.contains('\\'), "no backslash reaches the output");
}

// A dependency without a `src/` directory (the bare-directory layout the
// loader accepts) must not get a fabricated `src/` prefix on its foreign
// items' tar paths.
#[test]
fn srcless_dependency_foreign_files_have_no_src_prefix() {
    let mut files = std_files();
    files.push(("lands/lib.cohdl".to_string(), DEP_PKG.to_string()));
    files.push(("src/parts.cohdl".to_string(), CONSUMER_PKG.to_string()));
    let checked = check_files_in_with_deps(
        "consumer",
        &["std".to_string(), "lands".to_string()],
        &files,
        None,
    )
    .expect("pipeline runs");
    let out = render(
        &checked,
        &PackageMeta {
            name: "consumer",
            version: "0.1.0",
            description: None,
            license: None,
            repository: None,
        },
        &[
            DepMeta {
                name: "std".to_string(),
                version: std_version(),
                src_layout: true,
            },
            DepMeta {
                name: "lands".to_string(),
                version: "1.0.0".to_string(),
                src_layout: false,
            },
        ],
    );
    let foreign_at = out.json.find("\"foreign\"").unwrap();
    assert!(
        out.json[foreign_at..].contains("\"file\": \"lib.cohdl\""),
        "src-less dep keeps its bare tar path"
    );
    assert!(
        !out.json[foreign_at..].contains("src/lib.cohdl"),
        "no fabricated src/ prefix"
    );
}
