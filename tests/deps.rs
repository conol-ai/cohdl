//! RFC-029: package dependency versioning — `[dependencies]`, `cohdl.lock`,
//! the E11xx family, `cohdl update`, and the fmt canonical form. Everything
//! at the binary boundary runs against hermetic temp projects; the project-
//! local `deps/` registry keeps the fake packages away from the real std.

use std::path::{Path, PathBuf};
use std::process::Command;

fn cohdl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
}

fn tmp_dir(tag: &str) -> PathBuf {
    let tmp = std::env::temp_dir().join(format!("cohdl-deps-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    tmp
}

const DEP_LIB: &str = "\
pub device D { pins { A: 1 [passive], B: 2 [passive] } }
pub footprint TFP {}
pub part P1: D { primary { mfr: \"m\", mpn: \"n\", footprint: TFP } }
";

const MAIN_SRC: &str = "\
design B {
    inst a: mypkg::P1
    inst b: mypkg::P1
    net X: a.A, b.A
    net Y: a.B, b.B
}
";

/// A project depending on `mypkg = "1.0.0"` served from its own `deps/`
/// registry (built std-less so the fixture is hermetic).
fn make_project_with_dep(root: &Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n\n[dependencies]\nmypkg = \"1.0.0\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), MAIN_SRC).unwrap();
    let pkg = root.join("deps/mypkg/1.0.0");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::write(
        pkg.join("cohdl.toml"),
        "[package]\nname = \"mypkg\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    std::fs::write(pkg.join("src/lib.cohdl"), DEP_LIB).unwrap();
}

/// A design that needs nothing from any library: std carries traits and the
/// devices the demo boards need, and passives live in their own package, so a
/// fixture that wants two terminals declares them itself.
const LOCAL_DESIGN: &str = "\
pub device D { pins { A: 1 [passive], B: 2 [passive] } }

design B {
    inst c1: D
    inst c2: D
    net N: c1.A, c2.A
    net GND [gnd]: c1.B, c2.B
}
";

fn run(args: &[&str]) -> (bool, String, String) {
    let out = cohdl().args(args).output().unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// ---------------------------------------------------------------------------
// The happy path: first resolution writes the lock; rebuilds verify it
// ---------------------------------------------------------------------------

#[test]
fn first_resolution_writes_lock_and_rebuild_is_byte_stable() {
    let tmp = tmp_dir("lock");
    make_project_with_dep(&tmp);

    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
    let lock_path = tmp.join("cohdl.lock");
    let first = std::fs::read_to_string(&lock_path).unwrap();
    assert!(first.contains("name = \"mypkg\""), "{first}");
    assert!(first.contains("version = \"1.0.0\""), "{first}");
    assert!(first.contains("hash = \"sha256:"), "{first}");

    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
    let second = std::fs::read_to_string(&lock_path).unwrap();
    assert_eq!(
        first, second,
        "cohdl.lock must be byte-stable across builds"
    );
}

// ---------------------------------------------------------------------------
// E1103: the load-bearing guarantee — changed content under a locked version
// ---------------------------------------------------------------------------

#[test]
fn tampered_locked_content_is_a_hard_error() {
    let tmp = tmp_dir("tamper");
    make_project_with_dep(&tmp);
    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");

    // Mutate the locked package's content without bumping its version.
    let lib = tmp.join("deps/mypkg/1.0.0/src/lib.cohdl");
    let mut text = std::fs::read_to_string(&lib).unwrap();
    text.push_str("// a silent edit under a published version\n");
    std::fs::write(&lib, text).unwrap();

    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(!ok);
    assert!(err.contains("E1103"), "{err}");
    assert!(err.contains("locked"), "{err}");
    assert!(err.contains("sha256:"), "{err}");

    // `cohdl update` is the sanctioned way to accept the change.
    let (ok, _, err) = run(&["update", tmp.to_str().unwrap()]);
    assert!(ok, "{err}");
    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "after update the build must pass again: {err}");
}

// ---------------------------------------------------------------------------
// E1101: exact versions only — ranges rejected at manifest parse
// ---------------------------------------------------------------------------

#[test]
fn version_ranges_are_rejected_with_suggestion() {
    let tmp = tmp_dir("range");
    make_project_with_dep(&tmp);
    std::fs::write(
        tmp.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n\n[dependencies]\nmypkg = \"^1.0\"\n",
    )
    .unwrap();
    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(!ok);
    assert!(err.contains("E1101"), "{err}");
    assert!(err.contains("exact"), "{err}");
    assert!(
        err.contains("mypkg = \"1.0.0\""),
        "suggests the nearest exact version: {err}"
    );
}

#[test]
fn json_mode_carries_package_diags_in_the_array() {
    let tmp = tmp_dir("json");
    make_project_with_dep(&tmp);
    std::fs::write(
        tmp.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n\n[dependencies]\nmypkg = \"~1.0.0\"\n",
    )
    .unwrap();
    let (ok, out, _) = run(&["check", tmp.to_str().unwrap(), "--no-std", "--json"]);
    assert!(!ok);
    assert!(out.contains("\"verdict\": \"fail\""), "{out}");
    assert!(out.contains("\"code\": \"E1101\""), "{out}");
    assert!(out.contains("cohdl.toml"), "{out}");
}

// ---------------------------------------------------------------------------
// E1102 / E1106: resolution failures
// ---------------------------------------------------------------------------

#[test]
fn unresolvable_version_lists_searched_locations() {
    let tmp = tmp_dir("missing");
    make_project_with_dep(&tmp);
    std::fs::write(
        tmp.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n\n[dependencies]\nmypkg = \"9.9.9\"\n",
    )
    .unwrap();
    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(!ok);
    assert!(err.contains("E1102"), "{err}");
    assert!(err.contains("9.9.9"), "{err}");
    assert!(err.contains("deps"), "lists the searched locations: {err}");
}

#[test]
fn manifest_version_is_the_authority_not_the_dirname() {
    // The version a package OFFERS comes from its manifest; the directory
    // name (even one that spells a different version) is pure convention.
    let tmp = tmp_dir("identity");
    make_project_with_dep(&tmp);
    std::fs::write(
        tmp.join("deps/mypkg/1.0.0/cohdl.toml"),
        "[package]\nname = \"mypkg\"\nversion = \"2.0.0\"\n",
    )
    .unwrap();
    // The pin (1.0.0) no longer exists anywhere on disk: E1102, with the
    // manifest-declared 2.0.0 listed as available.
    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(!ok);
    assert!(err.contains("E1102"), "{err}");
    assert!(err.contains("available: 2.0.0"), "{err}");

    // Re-pin to the declared version: resolves fine out of the same
    // "1.0.0"-named directory — the dirname is never consulted.
    std::fs::write(
        tmp.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n\n[dependencies]\nmypkg = \"2.0.0\"\n",
    )
    .unwrap();
    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
}

#[test]
fn arbitrary_version_dir_names_resolve_by_manifest() {
    let tmp = tmp_dir("dirname");
    make_project_with_dep(&tmp);
    std::fs::rename(tmp.join("deps/mypkg/1.0.0"), tmp.join("deps/mypkg/current")).unwrap();
    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
}

#[test]
fn misplaced_package_name_is_rejected() {
    let tmp = tmp_dir("misname");
    make_project_with_dep(&tmp);
    std::fs::write(
        tmp.join("deps/mypkg/1.0.0/cohdl.toml"),
        "[package]\nname = \"otherpkg\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(!ok);
    assert!(err.contains("E1106"), "{err}");
    assert!(err.contains("otherpkg"), "{err}");
}

#[test]
fn duplicate_package_identity_is_rejected() {
    let tmp = tmp_dir("dup");
    make_project_with_dep(&tmp);
    // A second directory declaring the same (name, version).
    let dup = tmp.join("deps/mypkg/copy");
    std::fs::create_dir_all(dup.join("src")).unwrap();
    std::fs::write(
        dup.join("cohdl.toml"),
        "[package]\nname = \"mypkg\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    std::fs::write(dup.join("src/lib.cohdl"), DEP_LIB).unwrap();
    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(!ok);
    assert!(err.contains("E1106"), "{err}");
    assert!(err.contains("one immutable identity"), "{err}");
}

// ---------------------------------------------------------------------------
// E1104 + `cohdl update` migration
// ---------------------------------------------------------------------------

#[test]
fn pre_rfc029_manifest_is_flagged_and_update_migrates_it() {
    let tmp = tmp_dir("migrate");
    std::fs::create_dir_all(tmp.join("src")).unwrap();
    std::fs::write(
        tmp.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n",
    )
    .unwrap();
    // Uses the real std (the compiler discovers the repo's versioned root).
    std::fs::write(tmp.join("src/main.cohdl"), LOCAL_DESIGN).unwrap();

    let (ok, _, err) = run(&["check", tmp.to_str().unwrap()]);
    assert!(!ok);
    assert!(err.contains("E1104"), "{err}");
    assert!(err.contains("cohdl update"), "{err}");

    let (ok, _, err) = run(&["update", tmp.to_str().unwrap()]);
    assert!(ok, "{err}");
    let manifest = std::fs::read_to_string(tmp.join("cohdl.toml")).unwrap();
    assert!(manifest.contains("[dependencies]"), "{manifest}");
    assert!(manifest.contains("std = \""), "{manifest}");
    assert!(tmp.join("cohdl.lock").is_file());

    let (ok, _, err) = run(&["check", tmp.to_str().unwrap()]);
    assert!(ok, "after migration the check must pass: {err}");
}

// ---------------------------------------------------------------------------
// E1105: the unsuppressable override warning
// ---------------------------------------------------------------------------

#[test]
fn std_override_always_warns() {
    let tmp = tmp_dir("override");
    std::fs::create_dir_all(tmp.join("src")).unwrap();
    std::fs::write(
        tmp.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n\n[dependencies]\nstd = \"0.2.0\"\n",
    )
    .unwrap();
    std::fs::write(tmp.join("src/main.cohdl"), LOCAL_DESIGN).unwrap();
    let repo_std = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std");
    let (ok, _, err) = run(&[
        "check",
        tmp.to_str().unwrap(),
        "--std",
        repo_std.to_str().unwrap(),
    ]);
    assert!(ok, "{err}");
    assert!(
        err.contains("E1105"),
        "the override warning is mandatory: {err}"
    );
    assert!(err.contains("not reproducible"), "{err}");
}

// ---------------------------------------------------------------------------
// E1107: corrupted lock
// ---------------------------------------------------------------------------

#[test]
fn corrupted_lock_is_flagged() {
    let tmp = tmp_dir("corrupt");
    make_project_with_dep(&tmp);
    std::fs::write(tmp.join("cohdl.lock"), "not a lock file\n").unwrap();
    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(!ok);
    assert!(err.contains("E1107"), "{err}");
}

// ---------------------------------------------------------------------------
// fmt: [dependencies] canonical form
// ---------------------------------------------------------------------------

#[test]
fn fmt_sorts_dependencies_by_name() {
    let tmp = tmp_dir("fmt");
    make_project_with_dep(&tmp);
    std::fs::write(
        tmp.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n\n[dependencies]\nzeta = \"1.0.0\"\nmypkg = \"1.0.0\"\n",
    )
    .unwrap();

    let (ok, _, err) = run(&["fmt", tmp.to_str().unwrap(), "--check"]);
    assert!(!ok, "unsorted [dependencies] must flag drift");
    assert!(err.contains("would reformat"), "{err}");

    let (ok, _, err) = run(&["fmt", tmp.to_str().unwrap()]);
    assert!(ok, "{err}");
    let manifest = std::fs::read_to_string(tmp.join("cohdl.toml")).unwrap();
    let m = manifest.find("mypkg").unwrap();
    let z = manifest.find("zeta").unwrap();
    assert!(m < z, "entries sorted by name:\n{manifest}");

    // Idempotent: a second fmt --check is clean.
    let (ok, _, err) = run(&["fmt", tmp.to_str().unwrap(), "--check"]);
    assert!(ok, "{err}");
}

// ---------------------------------------------------------------------------
// `[package]` display metadata: read by the compiler, published to the
// registry, and never rewritten by fmt
// ---------------------------------------------------------------------------

const METADATA_MANIFEST: &str = "\
[package]
name = \"t\"
version = \"0.2.0\"
description = \"A demo package = with an equals sign.\"
license = \"MIT\"
repository = \"https://example.com/t\"

[design]
top = \"B\"

[dependencies]
zeta = \"1.0.0\"
mypkg = \"1.0.0\"
";

#[test]
fn manifest_metadata_is_read_verbatim() {
    let tmp = tmp_dir("meta_read");
    make_project_with_dep(&tmp);
    std::fs::write(tmp.join("cohdl.toml"), METADATA_MANIFEST).unwrap();

    let (_, manifest) = cohdl::project::peek_manifest(&tmp).unwrap();
    assert_eq!(
        manifest.description.as_deref(),
        Some("A demo package = with an equals sign.")
    );
    assert_eq!(manifest.license.as_deref(), Some("MIT"));
    assert_eq!(
        manifest.repository.as_deref(),
        Some("https://example.com/t")
    );

    // Absent keys are absent — never an empty string standing in for one.
    let (_, bare) = cohdl::project::peek_manifest(&tmp.join("deps/mypkg/1.0.0")).unwrap();
    assert_eq!(bare.description, None);
    assert_eq!(bare.license, None);
    assert_eq!(bare.repository, None);
}

#[test]
fn fmt_leaves_package_metadata_untouched() {
    let tmp = tmp_dir("meta_fmt");
    make_project_with_dep(&tmp);
    std::fs::write(tmp.join("cohdl.toml"), METADATA_MANIFEST).unwrap();

    // fmt canonicalizes [dependencies] only (RFC-009); every other section
    // survives byte-for-byte, so display metadata is never dropped or
    // reordered by a format pass.
    let (ok, _, err) = run(&["fmt", tmp.to_str().unwrap()]);
    assert!(ok, "{err}");
    let after = std::fs::read_to_string(tmp.join("cohdl.toml")).unwrap();
    let expected = METADATA_MANIFEST.replace(
        "zeta = \"1.0.0\"\nmypkg = \"1.0.0\"",
        "mypkg = \"1.0.0\"\nzeta = \"1.0.0\"",
    );
    assert_eq!(after, expected, "only [dependencies] may move:\n{after}");

    let (ok, _, err) = run(&["fmt", tmp.to_str().unwrap(), "--check"]);
    assert!(ok, "{err}");
}

// ---------------------------------------------------------------------------
// Library-level: version parsing + lock round-trip
// ---------------------------------------------------------------------------

#[test]
fn version_parsing_is_exact_only() {
    use cohdl::deps::parse_exact_version;
    assert!(parse_exact_version("1.2.3").is_ok());
    assert!(parse_exact_version("0.0.0").is_ok());
    for bad in [
        "^1.2.3",
        "~1.2",
        ">=1.0, <2.0",
        "1.2",
        "1.2.3.4",
        "1.02.3",
        "1.2.x",
        "*",
    ] {
        assert!(
            parse_exact_version(bad).is_err(),
            "`{bad}` must be rejected"
        );
    }
}

#[test]
fn lock_render_parse_round_trip_is_stable() {
    use cohdl::deps::{LockEntry, LockFile};
    let mut lock = LockFile::default();
    lock.entries.insert(
        "std".to_string(),
        LockEntry {
            version: cohdl::deps::parse_exact_version("0.1.0").unwrap(),
            hash: "sha256:aaaa".to_string(),
        },
    );
    lock.entries.insert(
        "alpha".to_string(),
        LockEntry {
            version: cohdl::deps::parse_exact_version("2.4.1").unwrap(),
            hash: "sha256:bbbb".to_string(),
        },
    );
    let rendered = lock.render();
    let reparsed = LockFile::parse(&rendered).unwrap();
    assert_eq!(reparsed.render(), rendered, "round trip is byte-identical");
    // Sorted by name: alpha before std.
    assert!(rendered.find("alpha").unwrap() < rendered.find("std").unwrap());
}

// ---------------------------------------------------------------------------
// The library root (`lib/`): std is resolved by the same rule as every other
// package that ships beside it
// ---------------------------------------------------------------------------

#[test]
fn every_library_resolves_through_the_same_family_rule() {
    use cohdl::deps::Registry;
    let reg = Registry {
        lib_root: Some(PathBuf::from("/repo/lib")),
        project_deps: PathBuf::from("/proj/deps"),
        cache_root: Some(PathBuf::from("/home/.cohdl/registry")),
    };
    // std gets no privileged path: `lib/std` is where `lib/passives` is.
    for name in ["std", "passives", "@brand/whatever"] {
        assert_eq!(
            reg.families(name),
            vec![
                PathBuf::from("/proj/deps").join(name),
                PathBuf::from("/repo/lib").join(name),
                PathBuf::from("/home/.cohdl/registry").join(name),
            ],
            "`{name}` must search project deps, then lib/, then the cache"
        );
    }
}

#[test]
fn a_library_root_is_recognized_by_the_packages_it_offers() {
    use cohdl::deps::is_library_root;
    let tmp = tmp_dir("libroot");
    let lib = tmp.join("lib");
    std::fs::create_dir_all(&lib).unwrap();
    assert!(!is_library_root(&lib), "an empty dir offers no package");

    // A subdirectory that is not a package does not make a library root —
    // this is what keeps a binary's `/usr/lib` from being mistaken for one.
    std::fs::create_dir_all(lib.join("notapkg/src")).unwrap();
    assert!(!is_library_root(&lib));

    // A family dir whose package declares a different name is unreadable as
    // that family, so it still does not count.
    let mislabeled = lib.join("passives");
    std::fs::create_dir_all(mislabeled.join("src")).unwrap();
    std::fs::write(
        mislabeled.join("cohdl.toml"),
        "[package]\nname = \"resistors\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    assert!(!is_library_root(&lib));

    std::fs::write(
        mislabeled.join("cohdl.toml"),
        "[package]\nname = \"passives\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    assert!(
        is_library_root(&lib),
        "one readable family is enough — no name is privileged"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// Transitive resolution (RFC-029 amendment, 2026-08-25): the closure walk —
// a resolved package's own [dependencies] join the compile and the lock
// ---------------------------------------------------------------------------

/// One package in the project-local `deps/` registry, with its own
/// `[dependencies]` section when `deps` is non-empty.
fn write_pkg(root: &Path, name: &str, version: &str, deps: &str, lib: &str) {
    let pkg = root.join(format!("deps/{name}/{version}"));
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    let deps_section = if deps.is_empty() {
        String::new()
    } else {
        format!("\n[dependencies]\n{deps}")
    };
    std::fs::write(
        pkg.join("cohdl.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n{deps_section}"),
    )
    .unwrap();
    std::fs::write(pkg.join("src/lib.cohdl"), lib).unwrap();
}

fn write_root(root: &Path, deps: &str, main: &str) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        format!("[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n\n[dependencies]\n{deps}"),
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), main).unwrap();
}

const SUB_DEVICE: &str = "\
pub device S { pins { A: 1 [passive], B: 2 [passive] } }
pub footprint SFP {}
pub part SP: S { primary { mfr: \"m\", mpn: \"s\", footprint: SFP } }
";

/// A design that instantiates both the direct dependency's part and the
/// transitive dependency's part — the transitive package's files must
/// actually join the compile, not merely its lock row.
const TRANSITIVE_MAIN: &str = "\
design B {
    inst a: mypkg::P1
    inst s: subpkg::SP
    net X: a.A, s.A
    net Y: a.B, s.B
}
";

#[test]
fn transitive_dependency_resolves_and_locks_the_closure() {
    let tmp = tmp_dir("transitive");
    write_root(&tmp, "mypkg = \"1.0.0\"\n", TRANSITIVE_MAIN);
    write_pkg(&tmp, "mypkg", "1.0.0", "subpkg = \"1.0.0\"\n", DEP_LIB);
    write_pkg(&tmp, "subpkg", "1.0.0", "", SUB_DEVICE);

    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
    let lock_path = tmp.join("cohdl.lock");
    let first = std::fs::read_to_string(&lock_path).unwrap();
    assert!(
        first.contains("name = \"subpkg\""),
        "the lock records the transitive dependency: {first}"
    );
    assert!(first.contains("name = \"mypkg\""), "{first}");

    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
    let second = std::fs::read_to_string(&lock_path).unwrap();
    assert_eq!(first, second, "closure lock is byte-stable across builds");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn missing_transitive_dependency_names_its_requirer() {
    let tmp = tmp_dir("transitive-missing");
    write_root(&tmp, "mypkg = \"1.0.0\"\n", TRANSITIVE_MAIN);
    write_pkg(&tmp, "mypkg", "1.0.0", "subpkg = \"1.0.0\"\n", DEP_LIB);
    // subpkg exists nowhere on disk.

    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(!ok);
    assert!(err.contains("E1102"), "{err}");
    assert!(err.contains("required by `mypkg`"), "{err}");
    assert!(err.contains("cohdl install"), "{err}");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn divergent_transitive_pins_are_a_conflict_unless_the_project_chooses() {
    let tmp = tmp_dir("transitive-conflict");
    write_root(
        &tmp,
        "a = \"1.0.0\"\nb = \"1.0.0\"\n",
        "design B {\n    inst s1: subpkg::SP\n    inst s2: subpkg::SP\n    net X: s1.A, s2.A\n    net Y: s1.B, s2.B\n}\n",
    );
    write_pkg(&tmp, "a", "1.0.0", "subpkg = \"1.0.0\"\n", "// a\n");
    write_pkg(&tmp, "b", "1.0.0", "subpkg = \"1.0.1\"\n", "// b\n");
    write_pkg(&tmp, "subpkg", "1.0.0", "", SUB_DEVICE);
    write_pkg(&tmp, "subpkg", "1.0.1", "", SUB_DEVICE);

    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(!ok);
    assert!(err.contains("E1108"), "{err}");
    assert!(err.contains("1.0.0") && err.contains("1.0.1"), "{err}");
    assert!(err.contains("`a`") && err.contains("`b`"), "{err}");
    assert!(
        err.contains("pin `subpkg` in this project's [dependencies]"),
        "{err}"
    );

    // The project pin is the single authority: adding one resolves the
    // conflict without touching either dependency.
    write_root(
        &tmp,
        "a = \"1.0.0\"\nb = \"1.0.0\"\nsubpkg = \"1.0.1\"\n",
        "design B {\n    inst s1: subpkg::SP\n    inst s2: subpkg::SP\n    net X: s1.A, s2.A\n    net Y: s1.B, s2.B\n}\n",
    );
    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
    let lock = std::fs::read_to_string(tmp.join("cohdl.lock")).unwrap();
    assert!(lock.contains("version = \"1.0.1\""), "{lock}");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn the_project_pin_wins_over_a_dependencys_pin() {
    let tmp = tmp_dir("transitive-rootwins");
    // subpkg 1.0.1 declares SP101; 1.0.0 does not. The design compiles only
    // against the project's pin.
    write_root(
        &tmp,
        "mypkg = \"1.0.0\"\nsubpkg = \"1.0.1\"\n",
        "design B {\n    inst a: mypkg::P1\n    inst s: subpkg::SP101\n    net X: a.A, s.A\n    net Y: a.B, s.B\n}\n",
    );
    write_pkg(&tmp, "mypkg", "1.0.0", "subpkg = \"1.0.0\"\n", DEP_LIB);
    write_pkg(&tmp, "subpkg", "1.0.0", "", SUB_DEVICE);
    write_pkg(
        &tmp,
        "subpkg",
        "1.0.1",
        "",
        "pub device S101 { pins { A: 1 [passive], B: 2 [passive] } }\npub footprint SFP {}\npub part SP101: S101 { primary { mfr: \"m\", mpn: \"s101\", footprint: SFP } }\n",
    );

    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
    let lock = std::fs::read_to_string(tmp.join("cohdl.lock")).unwrap();
    assert!(
        lock.contains("name = \"subpkg\"\nversion = \"1.0.1\""),
        "one subpkg version in the build, and it is the project's: {lock}"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn tampered_transitive_content_is_a_hard_error() {
    let tmp = tmp_dir("transitive-tamper");
    write_root(&tmp, "mypkg = \"1.0.0\"\n", TRANSITIVE_MAIN);
    write_pkg(&tmp, "mypkg", "1.0.0", "subpkg = \"1.0.0\"\n", DEP_LIB);
    write_pkg(&tmp, "subpkg", "1.0.0", "", SUB_DEVICE);
    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");

    let lib = tmp.join("deps/subpkg/1.0.0/src/lib.cohdl");
    let mut text = std::fs::read_to_string(&lib).unwrap();
    text.push_str("// a silent edit under a published version\n");
    std::fs::write(&lib, text).unwrap();

    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(!ok);
    assert!(err.contains("E1103"), "{err}");
    assert!(err.contains("subpkg"), "{err}");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn removing_the_direct_dependency_prunes_its_transitive_lock_rows() {
    let tmp = tmp_dir("transitive-prune");
    write_root(&tmp, "mypkg = \"1.0.0\"\n", TRANSITIVE_MAIN);
    write_pkg(&tmp, "mypkg", "1.0.0", "subpkg = \"1.0.0\"\n", DEP_LIB);
    write_pkg(&tmp, "subpkg", "1.0.0", "", SUB_DEVICE);
    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");

    // Drop the source's use of the packages, then remove the direct
    // dependency; the next resolve prunes the now-unreachable subpkg row.
    std::fs::write(tmp.join("src/main.cohdl"), LOCAL_DESIGN).unwrap();
    let (ok, _, err) = run(&["remove", "mypkg", tmp.to_str().unwrap()]);
    assert!(ok, "{err}");
    let (ok, _, err) = run(&["check", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
    let lock = std::fs::read_to_string(tmp.join("cohdl.lock")).unwrap();
    assert!(!lock.contains("mypkg"), "{lock}");
    assert!(
        !lock.contains("subpkg"),
        "orphaned transitive rows are pruned: {lock}"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn no_std_skips_a_dependencys_std_pin() {
    let tmp = tmp_dir("transitive-nostd");
    write_root(&tmp, "mypkg = \"1.0.0\"\n", MAIN_SRC);
    // A std pin no registry can satisfy: under --no-std the walk must skip
    // it (std is settled outside the registry), or this build would E1102.
    write_pkg(&tmp, "mypkg", "1.0.0", "std = \"9.9.9\"\n", DEP_LIB);

    let (ok, _, err) = run(&["build", tmp.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
    let lock = std::fs::read_to_string(tmp.join("cohdl.lock")).unwrap();
    assert!(
        !lock.contains("\"std\""),
        "no std row under --no-std: {lock}"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn the_repos_std_is_an_ordinary_library_under_lib() {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib");
    assert!(
        cohdl::deps::is_library_root(&lib),
        "the repo's lib/ must be discoverable as a library root"
    );
    let (version, dir) = cohdl::deps::newest_available(&lib.join("std"), "std")
        .expect("lib/std offers the std package");
    assert_eq!(dir, lib.join("std"), "the family dir is itself the package");
    let declared = std::fs::read_to_string(lib.join("std/cohdl.toml"))
        .unwrap()
        .lines()
        .find_map(|l| {
            l.strip_prefix("version = ")
                .map(|v| v.trim_matches('"').to_string())
        })
        .expect("lib/std/cohdl.toml declares a version");
    assert_eq!(
        version.to_string(),
        declared,
        "std's version comes from lib/std/cohdl.toml, never from its path"
    );
}
