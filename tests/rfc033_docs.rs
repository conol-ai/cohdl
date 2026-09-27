//! RFC-033 Task 13 — package API docs schema v2: `schema_version` 2 iff any
//! emitted item uses M2 syntax; M2 items carry `body_source` (the fmt-canonical
//! body text) and omit the `insts`/`calls`/`nets` summary. v1 documents stay
//! byte-identical (tests/apidocs.rs pins those).

use cohdl::emit::docsjson::{render, PackageMeta, Rendered};
use cohdl::pipeline::{check_files_in_with_deps, Checked};

fn std_files() -> Vec<(String, String)> {
    let std_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std/src");
    let mut entries: Vec<_> = std::fs::read_dir(&std_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
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

use cohdl::emit::docsjson::DepMeta;

// Python's standard JSON parser validates the whole document before checking
// fields. Close stdin explicitly: wait_with_output must not wait for our EOF.
fn assert_json(docs: &Rendered, assertions: &str, args: &[&str]) {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let script = format!(
        "import json, sys\ndoc = json.load(sys.stdin)\nitems = {{i['name']: i for i in doc['items']}}\n{assertions}"
    );
    let mut child = Command::new("python3")
        .args(["-c", &script])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("python3 is required for docs JSON validation");
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(docs.json.as_bytes()).unwrap();
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "JSON assertions failed: {}\n{}",
        String::from_utf8_lossy(&output.stderr),
        docs.json
    );
}

const LIB: &str = "pub device Dev<L: Length = 1mm> { pins { optional A: 1 [passive] } }\n\
    pub subdesign Sub<L: Length = 1mm> { ports { required P: Pin } net _: P }\n\
    pub fn helper<L: Length = 1mm>(p: Pin) { net _: p }\n";

fn assert_m2(source: &str, body: &str) {
    let docs = docs_for("probe", "0.1.0", &[("src/main.cohdl", source)]);
    assert_json(
        &docs,
        r#"assert doc['schema_version'] == 2, doc['schema_version']
item = items['probe']
assert item['body_source'] == sys.argv[1], repr(item['body_source'])
assert item['file'] == 'src/main.cohdl'
payload = item[item['kind']]
for key in ('insts', 'calls', 'nets'):
    assert key not in payload, payload
    assert key not in item, item
"#,
        &[body],
    );
}

// Each document has only this one RFC-033 expression position: no const/for
// elsewhere can accidentally make a missed position look covered.
macro_rules! expression_case {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let body = $body;
            let src = format!("{LIB}pub subdesign probe {{\n    {body}\n}}\n");
            assert_m2(&src, body);
        }
    };
}

expression_case!(parenthesized_device_array, "inst a: [Dev; (3)]");
expression_case!(parenthesized_subdesign_array, "subdesign a: [Sub; (3)]");
expression_case!(computed_device_generic, "inst a: Dev<1mm + 2mm>");
expression_case!(computed_subdesign_generic, "subdesign a: Sub<1mm + 2mm>");
expression_case!(
    computed_call_generic,
    "inst a: Dev\n    helper::<1mm + 2mm>(a.A)"
);
expression_case!(computed_net_single, "inst a: [Dev; 3]\n    net _: a[(0)].A");
expression_case!(
    computed_net_range_start,
    "inst a: [Dev; 3]\n    net _: a[(0)..=2].A"
);
expression_case!(
    computed_net_range_end,
    "inst a: [Dev; 3]\n    net _: a[0..=(2)].A"
);
expression_case!(
    computed_net_range_step,
    "inst a: [Dev; 3]\n    net _: a[0..=2 step (1)].A"
);
expression_case!(
    computed_net_list,
    "inst a: [Dev; 3]\n    net _: a[0, (2)].A"
);
expression_case!(computed_nc_selector, "inst a: [Dev; 3]\n    nc: a[(0)].A");
expression_case!(
    computed_call_selector,
    "inst a: [Dev; 3]\n    helper(a[(0)].A)"
);
expression_case!(
    computed_port_connection,
    "inst a: [Dev; 3]\n    subdesign s: Sub {\n        P: a[(0)].A\n    }"
);
expression_case!(
    computed_placement_x,
    "inst a: Dev\n    layout {\n        place a at ((1mm), -2mm)\n    }"
);
expression_case!(
    computed_placement_y,
    "inst a: Dev\n    layout {\n        place a at (1mm, 2mm + 1mm)\n    }"
);
expression_case!(
    computed_placement_index,
    "inst a: [Dev; 3]\n    layout {\n        place a[(0)] at (1mm, -2mm)\n    }"
);
expression_case!(
    computed_placement_rotation,
    "inst a: Dev\n    layout {\n        place a at (1mm, -2mm) rotate 45 * 2\n    }"
);
expression_case!(
    array_len_property,
    "inst a: [Dev; 3]\n    inst b: [Dev; a.len]"
);
expression_case!(
    layout_const,
    "inst a: Dev\n    layout {\n        const P: Length = 1mm\n        place a at (P, 2mm)\n    }"
);
expression_case!(layout_loop, "inst a: [Dev; 3]\n    layout {\n        for positions: i in 0..3 {\n            place a[i] at (1mm, 2mm)\n        }\n    }");

#[test]
fn m2_package_gets_schema_v2_and_exact_body_source() {
    let src = "pub fn probe<const N: Int = 2>(p: Pin) { for x: i in 0..N { net _: p } }\n";
    let body = "for x: i in 0..N {\n        net _: p\n    }";
    assert_m2(src, body);
    let docs = docs_for("probe", "0.1.0", &[("src/main.cohdl", src)]);
    assert_json(
        &docs,
        r#"assert items['probe']['fn'] == {
    'generics': [{'name': 'N', 'bound': {'const': 'Int'}, 'default': 2}],
    'params': [{'name': 'p', 'type': {'kind': 'pin'}}]
}"#,
        &[],
    );
}

#[test]
fn unbound_generic_array_is_source_not_a_guessed_number() {
    assert_m2(
        &format!("{LIB}pub fn probe<const N: Int>() {{ inst a: [Dev; N + 1] }}\n"),
        "inst a: [Dev; N + 1]",
    );
}

#[test]
fn legacy_literals_keep_v1_and_parameter_array_summaries() {
    let src = format!("{LIB}pub fn plain<L: Length>(p: Pin) {{ inst a: [Dev<L>; 3] helper::<L>(p) net _: p, a[0..=2].A }}\n\
        pub subdesign placed {{ inst a: [Dev<1mm>; 3] subdesign b: [Sub; 3] layout {{ place a[0] at (1mm, -2mm) rotate 90 }} }}\n");
    let docs = docs_for("probe", "0.1.0", &[("src/main.cohdl", &src)]);
    assert_json(
        &docs,
        r#"assert doc['schema_version'] == 1
assert all('body_source' not in i for i in doc['items'])
assert items['plain']['fn'] == {
    'generics': [{'name': 'L', 'bound': {'unit': 'Length'}}],
    'params': [{'name': 'p', 'type': {'kind': 'pin'}}],
    'insts': [{'name': 'a', 'type': 'probe::Dev', 'array': 3, 'args': ['L']}],
    'calls': ['probe::helper'], 'nets': 1
}
assert items['placed']['subdesign'] == {
    'ports': [], 'insts': [
        {'name': 'a', 'type': 'probe::Dev', 'array': 3, 'args': ['1mm']},
        {'name': 'b', 'type': 'probe::Sub', 'kind': 'subdesign', 'array': 3}
    ], 'nets': 0
}"#,
        &[],
    );
}

#[test]
fn mixed_document_preserves_legacy_items_and_exact_m2_source() {
    let src = format!("{LIB}pub fn plain(p: Pin) {{ net _: p }}\n\
        pub fn probe(p: Pin) {{ const N: Int = 3 inst a: [Dev; (N)] for wiring: i in 0..N {{ net _: p, a[i].A }} }}\n");
    assert_m2(&src, "const N: Int = 3\n    inst a: [Dev; (N)]\n    for wiring: i in 0..N {\n        net _: p, a[i].A\n    }");
    let docs = docs_for("probe", "0.1.0", &[("src/main.cohdl", &src)]);
    assert_json(
        &docs,
        r#"assert items['plain']['fn'] == {
    'params': [{'name': 'p', 'type': {'kind': 'pin'}}], 'nets': 1
}
assert 'body_source' not in items['plain']
assert items['Sub']['subdesign'] == {
    'generics': [{'name': 'L', 'bound': {'unit': 'Length'}, 'default': '1mm'}],
    'ports': [{'name': 'P', 'obligation': 'required'}], 'nets': 1
}"#,
        &[],
    );
}

#[test]
fn empty_const_generic_body_has_exact_empty_source() {
    assert_m2("pub fn probe<const N: Int>() {}\n", "");
}

expression_case!(
    computed_bypass_index,
    "inst a: [Dev; 3]\n    #[bypass(a[(0)].A, 100nF)]\n    inst c: Dev"
);

#[test]
fn design_with_only_computed_array_has_exact_source() {
    assert_m2(
        &format!("{LIB}design probe {{ inst a: [Dev; (3)] nc: a[0].A, a[1].A, a[2].A }}\n"),
        "inst a: [Dev; (3)]\n    nc: a[0].A, a[1].A, a[2].A",
    );
}
