//! RFC-033 Task 8 — identity stability of loop-generated objects.
//!
//! A frame's path (`__for_{label}_{value}`) IS its identity: inserting an
//! unrelated loop above, growing the bound, or renaming the label changes
//! exactly what those rules say it changes — nothing else.

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

fn check(src: &str) -> cohdl::pipeline::Checked {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "clean check expected:\n{}",
        checked.diags.render(&checked.sm)
    );
    checked
}

fn build_with(src: &str, prior: &LockState) -> (String, cohdl::lock::LockState) {
    let mut checked = check(src);
    let art = build_artifacts(&mut checked, prior).expect("build");
    let lock = art.lock.clone();
    (art.netlist, lock)
}

const LIB: &str = r#"
pub trait Led { designator_prefix: "D" }
pub device LedDev { pins { VDD: 1 [power_in], GND: 2 [power_in], DIN: 3 [input], DOUT: 4 [output] } }
impl Led for LedDev {}
pub device HostDev { pins { V5: 1 [power_out], GND: 2 [power_in], DATA: 3 [output] } }
pub footprint FP {}
pub part LED: LedDev { primary { mfr: "m", mpn: "led", footprint: FP } }
pub part HOST: HostDev { primary { mfr: "m", mpn: "host", footprint: FP } }
"#;

fn chain(n: i64) -> String {
    format!(
        "{LIB}
design Chain {{
    const N: Int = {n}
    inst host: HOST
    inst leds: [LED; N]
    net VCC [5V]: host.V5, leds[0..=(leds.len - 1)].VDD
    net GND [gnd]: host.GND, leds[0..=(leds.len - 1)].GND
    net DATA: host.DATA, leds[0].DIN
    for links: n in 0..(leds.len - 1) {{
        net _: leds[n].DOUT, leds[n + 1].DIN
    }}
    nc: leds[leds.len - 1].DOUT
}}"
    )
}

#[test]
fn inserting_a_loop_above_keeps_link_identity() {
    // Baseline: the 10-LED chain.
    let (base_net, lock) = build_with(&chain(10), &LockState::default());
    let links: Vec<&str> = base_net
        .lines()
        .filter(|l| l.contains("__for_links_"))
        .collect();
    assert_eq!(links.len(), 9, "9 links:\n{base_net}");

    // Insert a second labelled loop ABOVE `links`; rebuild with the prior lock.
    let src = chain(10).replace(
        "    for links: n in 0..(leds.len - 1) {",
        "    for warm: w in 0..1 { net _: host.GND }\n    for links: n in 0..(leds.len - 1) {",
    );
    let (net2, _) = build_with(&src, &lock);
    for i in 0..9 {
        assert!(
            net2.contains(&format!("__for_links_{i}::")),
            "link {i} must survive:\n{net2}"
        );
    }
    // LED designators unchanged: D1..D10 all still present.
    for d in 1..=10 {
        assert!(
            net2.contains(&format!("\"D{d}\"")),
            "designator D{d}:\n{net2}"
        );
    }
}

#[test]
fn growing_n_extends_not_renumbers() {
    let (_base, lock) = build_with(&chain(10), &LockState::default());
    let (net12, _) = build_with(&chain(12), &lock);
    for i in 0..9 {
        assert!(
            net12.contains(&format!("__for_links_{i}::")),
            "old link {i} survives:\n{net12}"
        );
    }
    assert!(net12.contains("__for_links_9::"), "new link 9:\n{net12}");
    assert!(net12.contains("__for_links_10::"), "new link 10:\n{net12}");
}

#[test]
fn renaming_the_label_moves_the_paths() {
    let (_base, lock) = build_with(&chain(10), &LockState::default());
    let src = chain(10).replace("for links: n in", "for chain: n in");
    let (net, _) = build_with(&src, &lock);
    assert!(!net.contains("__for_links_"), "old paths gone:\n{net}");
    assert!(net.contains("__for_chain_0::"), "new paths appear:\n{net}");
    assert!(net.contains("__for_chain_8::"), "new paths appear:\n{net}");
}
