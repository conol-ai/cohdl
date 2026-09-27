//! Product-level checks for complete connectivity, explicit NCs, BOM and identity.

use std::collections::{BTreeMap, BTreeSet};

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in, BuildArtifacts, Checked};

const LIB: &str = r#"
pub trait Led { designator_prefix: "D" }
pub trait Resistor { designator_prefix: "R" }
pub device LedDev { pins { VDD: 1 [power_in], GND: 2 [power_in], DIN: 3 [input], DOUT: 4 [output] } }
impl Led for LedDev {}
pub device HostDev { pins { V5: 1 [power_out], GND: 2 [power_in], DATA: 3 [output] } }
pub device RDev { pins { A: 1 [passive], B: 2 [passive] } }
impl Resistor for RDev {}
pub device Source { pins { OUT: 1 [output] } }
pub device Sink { pins { IN: 1 [input] } }
pub footprint FP {}
pub part LED: LedDev { primary { mfr: "m", mpn: "led", footprint: FP } }
pub part HOST: HostDev { primary { mfr: "m", mpn: "host", footprint: FP } }
pub part RP: RDev { primary { mfr: "m", mpn: "r", footprint: FP } }
pub part SRCP: Source { primary { mfr: "m", mpn: "src", footprint: FP } }
pub part SNKP: Sink { primary { mfr: "m", mpn: "snk", footprint: FP } }
"#;

fn build(src: &str, prior: &LockState) -> (Checked, BuildArtifacts) {
    let mut checked =
        check_files_in("board", &[("src/main.cohdl".into(), src.into())], None).expect("selection");
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    let artifacts = build_artifacts(&mut checked, prior)
        .unwrap_or_else(|| panic!("{}", checked.diags.render(&checked.sm)));
    (checked, artifacts)
}

type Ends = BTreeSet<(String, String)>;

fn end(path: &str, pin: &str) -> (String, String) {
    (path.into(), pin.into())
}

fn chain(n: usize, loops: bool) -> String {
    let links = if loops {
        "for links: i in 0..(leds.len - 1) { net _: leds[i].DOUT, leds[i + 1].DIN }".into()
    } else {
        (0..n - 1)
            .map(|i| format!("net _: leds[{i}].DOUT, leds[{}].DIN\n", i + 1))
            .collect::<String>()
    };
    format!(
        "{LIB}\ndesign Chain {{
            inst host: HOST
            inst leds: [LED; {n}]
            net VCC [5V]: host.V5, leds[0..={}].VDD
            net GND [gnd]: host.GND, leds[0..={}].GND
            net DATA: host.DATA, leds[0].DIN
            {links}
            nc: leds[{}].DOUT
        }}",
        n - 1,
        n - 1,
        n - 1
    )
}

#[test]
fn led_chain_has_exact_nets_nc_parts_and_bom_for_one_two_and_ten() {
    for n in [1, 2, 10] {
        let (checked, art) = build(&chain(n, true), &LockState::default());
        let (manual, unrolled) = build(&chain(n, false), &LockState::default());
        let ir = checked.ir.as_ref().unwrap();
        let expected_nc = BTreeSet::from([end(&format!("Chain::leds_{}", n - 1), "DOUT")]);
        assert_eq!(ir.nc_pins, expected_nc);
        assert_eq!(manual.ir.as_ref().unwrap().nc_pins, expected_nc);

        let mut expected = BTreeMap::<String, Ends>::from([
            ("VCC".into(), BTreeSet::from([end("Chain::host", "V5")])),
            ("GND".into(), BTreeSet::from([end("Chain::host", "GND")])),
            (
                "DATA".into(),
                BTreeSet::from([end("Chain::host", "DATA"), end("Chain::leds_0", "DIN")]),
            ),
        ]);
        for i in 0..n {
            let path = format!("Chain::leds_{i}");
            expected.get_mut("VCC").unwrap().insert(end(&path, "VDD"));
            expected.get_mut("GND").unwrap().insert(end(&path, "GND"));
            if i + 1 < n {
                expected.insert(
                    format!("__for_links_{i}::__net0"),
                    BTreeSet::from([
                        end(&path, "DOUT"),
                        end(&format!("Chain::leds_{}", i + 1), "DIN"),
                    ]),
                );
            }
        }
        let actual: BTreeMap<_, _> = ir
            .nets
            .iter()
            .map(|net| (net.name.clone(), net.members.clone()))
            .collect();
        assert_eq!(actual.len(), ir.nets.len(), "net names must be unique");
        assert_eq!(actual, expected);
        let mut manual_partition: Vec<_> = manual
            .ir
            .as_ref()
            .unwrap()
            .nets
            .iter()
            .map(|net| net.members.clone())
            .collect();
        let mut expected_partition: Vec<_> = expected.values().cloned().collect();
        manual_partition.sort();
        expected_partition.sort();
        assert_eq!(manual_partition, expected_partition);
        for net in &ir.nets {
            assert_eq!(net.is_gnd, net.name == "GND");
            assert_eq!(
                net.voltage.as_ref().map(|v| v.text.as_str()),
                (net.name == "VCC").then_some("5V")
            );
        }

        let mut expected_parts =
            BTreeMap::from([("Chain::host".to_string(), ("board::HostDev", "board::HOST"))]);
        for i in 0..n {
            expected_parts.insert(format!("Chain::leds_{i}"), ("board::LedDev", "board::LED"));
        }
        let actual_parts: BTreeMap<_, _> = ir
            .instances
            .iter()
            .map(|(path, inst)| {
                assert_eq!(inst.variant, None);
                (
                    path.clone(),
                    (inst.device.as_str(), inst.part.as_deref().unwrap()),
                )
            })
            .collect();
        assert_eq!(actual_parts, expected_parts);
        let designators = (1..=n)
            .map(|i| format!("D{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let expected_bom = format!("Manufacturer,Comment,Designator,Footprint\n\"m\",\"host\",\"U1\",\"FP\"\n\"m\",\"led\",\"{designators}\",\"FP\"\n");
        assert_eq!(art.bom, expected_bom);
        assert_eq!(unrolled.bom, expected_bom);
        assert_eq!(art.lock, unrolled.lock);
    }
}

fn helper_source(n: usize, reversed: bool, extra: bool, label: &str) -> String {
    let filters = format!("for {label}: i in 0..{n} {{ series(inputs[i].OUT, outputs[i].IN) }}");
    let spare = "for spare: i in 0..1 { series(inputs[0].OUT, outputs[0].IN) }";
    let loops = if reversed {
        format!("{spare}\n{filters}")
    } else {
        format!("{filters}\n{spare}")
    };
    let prefix = if extra {
        "for unrelated: i in 0..1 { net _: inputs[0].OUT }"
    } else {
        ""
    };
    format!(
        "{LIB}\npub fn series(a: Pin, b: Pin) {{ inst r: RP net _: a, r.A net _: b, r.B }}
        design Identity {{ inst inputs: [SRCP; {n}] inst outputs: [SNKP; {n}] {prefix} {loops} }}"
    )
}

#[test]
fn helper_instances_keep_all_surviving_designators_across_edits() {
    let (_, base) = build(
        &helper_source(2, false, false, "filters"),
        &LockState::default(),
    );
    let mut expected_paths: BTreeSet<String> =
        ["Identity::__for_spare_0::__fn0_series::r".into()].into();
    for i in 0..2 {
        expected_paths.insert(format!("Identity::inputs_{i}"));
        expected_paths.insert(format!("Identity::outputs_{i}"));
        expected_paths.insert(format!("Identity::__for_filters_{i}::__fn0_series::r"));
    }
    assert_eq!(
        base.lock
            .designators
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected_paths
    );
    for (reversed, extra) in [(true, false), (false, true), (true, true)] {
        let (_, changed) = build(&helper_source(2, reversed, extra, "filters"), &base.lock);
        assert_eq!(
            changed.lock, base.lock,
            "reorder={reversed}, insert={extra}"
        );
        assert_eq!(changed.bom, base.bom);
    }
    let (_, grown) = build(&helper_source(3, false, false, "filters"), &base.lock);
    for (path, refdes) in &base.lock.designators {
        assert_eq!(grown.lock.designators.get(path), Some(refdes));
    }
    assert_eq!(
        grown.lock.designators.len(),
        base.lock.designators.len() + 3
    );
    let (_, renamed) = build(&helper_source(2, false, false, "renamed"), &base.lock);
    let old_refdes: BTreeSet<_> = base.lock.designators.values().collect();
    for (path, refdes) in &base.lock.designators {
        if path.contains("::__for_filters_") {
            let new_path = path.replace("::__for_filters_", "::__for_renamed_");
            assert!(!renamed.lock.designators.contains_key(path));
            assert_eq!(renamed.lock.tombstones.get(path), Some(refdes));
            assert!(!old_refdes.contains(&renamed.lock.designators[&new_path]));
        } else {
            assert_eq!(renamed.lock.designators.get(path), Some(refdes));
        }
    }
    assert_eq!(renamed.lock.designators.len(), base.lock.designators.len());
}
