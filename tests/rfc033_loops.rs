use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};
use std::collections::BTreeSet;

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

fn netlist(src: &str) -> String {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "clean build expected:\n{}",
        checked.diags.render(&checked.sm)
    );
    artifacts.expect("build").netlist
}

const LIB: &str = r#"
pub trait Led { designator_prefix: "D" }
pub device LedDev { pins { VDD: 1 [power_in], GND: 2 [power_in], DIN: 3 [input], DOUT: 4 [output] } }
impl Led for LedDev {}
pub device HostDev { pins { V5: 1 [power_out], GND: 2 [power_in], DATA: 3 [output] } }
pub device SinkDev { pins { S: 1 [passive] } }
pub footprint FP {}
pub part LED: LedDev { primary { mfr: "m", mpn: "led", footprint: FP } }
pub part HOST: HostDev { primary { mfr: "m", mpn: "host", footprint: FP } }
pub part SINK: SinkDev { primary { mfr: "m", mpn: "sink", footprint: FP } }
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
fn led_chain_n_1_2_10() {
    for (n, links) in [(1, 0), (2, 1), (10, 9)] {
        let (chk, r) = check(&chain(n));
        assert!(!chk.diags.has_errors(), "N={n}:\n{r}");
        let net = netlist(&chain(n));
        // every DOUT→DIN link is its own net: count nets with both a DOUT and a DIN endpoint
        assert_eq!(net.matches("__for_links_").count(), links, "N={n}\n{net}");
    }
}

#[test]
fn off_by_one_names_the_iteration() {
    let src = chain(10).replace("0..(leds.len - 1)", "0..leds.len");
    let r = check(&src).1;
    assert!(
        r.contains("E202")
            && r.contains("index 10 is out of bounds")
            && r.contains("__for_links_9")
            && r.contains("n = 9"),
        "{r}"
    );
    assert_eq!(
        r.matches("error[E202]").count(),
        1,
        "one diagnostic for the one failing iteration:\n{r}"
    );
}

#[test]
fn reversed_and_empty_ranges() {
    let src = chain(3).replace("0..(leds.len - 1)", "2..1");
    assert!(check(&src).1.contains("E1404"));
    let src = chain(1); // 0..0 — empty, still valid
    assert!(!check(&src).0.diags.has_errors());
}

#[test]
fn direct_inst_in_loop_is_e1406_even_when_empty() {
    let src = format!("{LIB}
design B {{ inst host: HOST  for x: i in 0..0 {{ inst extra: LED }}  net _: host.V5, host.GND, host.DATA }}");
    assert!(check(&src).1.contains("E1406"));
}

#[test]
fn sibling_frames_do_not_merge_same_named_nets() {
    // Named nets are not admitted directly in a circuit for body (E1406, see
    // rfc033_named_nets). The old frame-isolation property survives through a
    // helper fn: each sibling loop iteration gets its own private LINK.
    let src = format!(
        "{LIB}
fn link(x: Pin, y: Pin) {{ net LINK: x, y }}
design B {{
    inst host: HOST
    inst leds: [LED; 2]
    net P: host.V5, leds[0..=1].VDD
    net G [gnd]: host.GND, leds[0..=1].GND
    net D: host.DATA, leds[0].DIN
    for a: i in 0..1 {{ link(leds[0].DOUT, leds[1].DIN) }}
    for b: i in 0..1 {{ link(leds[1].DOUT, leds[1].DOUT) }}
}}"
    );
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "{r}");
    let ir = chk.ir.as_ref().unwrap();
    let links: Vec<_> = ir.nets.iter().filter(|n| n.name.contains("LINK")).collect();
    assert_eq!(links.len(), 2, "one private LINK per sibling frame:\n{r}");
    let names: BTreeSet<_> = links.iter().map(|n| n.name.as_str()).collect();
    assert!(
        names.contains("__for_a_0::__fn0_link::LINK")
            && names.contains("__for_b_0::__fn0_link::LINK"),
        "{:?}",
        names
    );
    let endpoints: BTreeSet<_> = links.iter().map(|n| n.members.clone()).collect();
    let expected: BTreeSet<_> = [
        [
            ("B::leds_0".to_string(), "DOUT".to_string()),
            ("B::leds_1".to_string(), "DIN".to_string()),
        ]
        .into_iter()
        .collect(),
        [("B::leds_1".to_string(), "DOUT".to_string())]
            .into_iter()
            .collect(),
    ]
    .into_iter()
    .collect();
    assert_eq!(
        endpoints, expected,
        "full endpoint sets, no cross-frame merge"
    );
}

#[test]
fn layout_loop_places_each_element() {
    let src = format!(
        "{LIB}
design B {{
    inst host: HOST
    inst leds: [LED; 6]
    net P: host.V5, leds[0..=5].VDD
    net G [gnd]: host.GND, leds[0..=5].GND
    net D: host.DATA, leds[0..=5].DIN
    for dead: n in 0..leds.len {{ net _: leds[n].DOUT }}
    layout {{
        for grid: n in 0..leds.len {{
            place leds[n] at (10mm + (n % 5) * 4mm, 10mm + (n / 5) * 4mm)
        }}
        place leds[2] at (0mm, 0mm)
    }}
}}"
    );
    let r = check(&src).1;
    assert!(
        r.contains("E1007") && r.contains("placed more than once"),
        "loop + explicit duplicate:\n{r}"
    );
    let src2 = src.replace("        place leds[2] at (0mm, 0mm)\n", "");
    let (mut chk, r) = check(&src2);
    assert!(!chk.diags.has_errors(), "{r}");
    let art =
        cohdl::pipeline::build_artifacts(&mut chk, &cohdl::lock::LockState::default()).unwrap();
    let layout = art.layout.unwrap();
    assert!(
        layout.contains("[26, 10]") && layout.contains("[10, 14]"),
        "{layout}"
    );
}
