use cohdl::diag::Severity;
use cohdl::pipeline::check_files_in;

const DEVICE: &str = "pub device D { pins { A: 1 [passive] } }\n";

fn check(body: &str) -> (cohdl::pipeline::Checked, String) {
    let source = format!("{DEVICE}{body}");
    let files = vec![("main.cohdl".to_string(), source.clone())];
    let mut checked = check_files_in("board", &files, None).unwrap();
    checked.diags.sort(&checked.sm);
    (checked, source)
}

#[test]
fn undefined_coordinate_name_reports_once_even_in_empty_or_large_loop() {
    for count in [0, 1, 50] {
        let (checked, source) = check(&format!(
            "design B {{ inst a:D nc:a.A layout {{
                for p:i in 0..{count} {{ place a at (i * PITCH, 0mm) }}
            }} }}"
        ));
        let errors: Vec<_> = checked
            .diags
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert_eq!(errors.len(), 1, "{}", checked.diags.render(&checked.sm));
        assert_eq!(errors[0].code, "E202");
        assert!(errors[0].message.contains("PITCH"));
        assert_eq!(
            errors[0].primary.span.start as usize,
            source.find("PITCH").unwrap()
        );
        assert_eq!(checked.sm.snippet(errors[0].primary.span), "PITCH");
    }
}

#[test]
fn undefined_helper_expression_is_checked_once_across_calls() {
    for calls in ["", "hook(h.A) hook(h.A)"] {
        let (checked, _) = check(&format!(
            "pub fn hook(p:Pin) {{ for w:i in 0..3 {{
                const X:Length = i * PITCH
                net _:p
            }} }}
            design B {{ inst h:D net _:h.A {calls} }}"
        ));
        let errors: Vec<_> = checked
            .diags
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert_eq!(errors.len(), 1, "{}", checked.diags.render(&checked.sm));
        assert_eq!(errors[0].code, "E202");
        assert_eq!(checked.sm.snippet(errors[0].primary.span), "PITCH");
    }
}

#[test]
fn separate_undefined_expression_sites_are_not_collapsed() {
    let (checked, source) = check(
        "design B { inst a:D nc:a.A layout {
            for p:i in 0..50 { place a at (i * PITCH, i * PITCH) }
        } }",
    );
    let errors: Vec<_> = checked
        .diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 2, "{}", checked.diags.render(&checked.sm));
    assert!(errors.iter().all(|d| d.code == "E202"));
    let starts: Vec<_> = errors
        .iter()
        .map(|d| d.primary.span.start as usize)
        .collect();
    assert_eq!(
        starts,
        source
            .match_indices("PITCH")
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
    );
}

#[test]
fn dynamic_index_failures_keep_each_iteration() {
    let (checked, _) = check(
        "design B { inst a:[D;1] nc:a[0].A
            for p:i in 1..3 { net _:a[i].A }
        }",
    );
    let errors: Vec<_> = checked
        .diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 2, "{}", checked.diags.render(&checked.sm));
    for (error, index) in errors.iter().zip([1, 2]) {
        assert_eq!(error.code, "E202");
        assert!(error.message.contains(&format!("index {index}")));
        assert!(error.message.contains(&format!("i = {index}")));
    }
}

#[test]
fn dynamic_index_failures_keep_separate_helper_activations() {
    let (checked, _) = check(
        "pub fn hook<const N:Int>(p:Pin) {
            inst a:[D;1]
            net _:p,a[0].A
            net _:p,a[N].A
        }
        design B { inst h:D hook::<1>(h.A) hook::<0>(h.A) hook::<2>(h.A) }",
    );
    let errors: Vec<_> = checked
        .diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 2, "{}", checked.diags.render(&checked.sm));
    assert!(errors.iter().all(|d| d.code == "E202"));
    assert!(errors.iter().any(|d| d.message.contains("index 1")));
    assert!(errors.iter().any(|d| d.message.contains("index 2")));
}

#[test]
fn forward_declared_coordinate_constant_remains_valid() {
    let (checked, _) = check(
        "design B { inst a:[D;3] nc:a[0].A,a[1].A,a[2].A layout {
            for p:i in 0..3 { place a[i] at (i * PITCH,0mm) }
            const PITCH:Length = 2mm
        } }",
    );
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    assert_eq!(checked.ir.unwrap().layout.placements.len(), 3);
}
