//! RFC-033 §Scenarios — the three normative acceptance shapes, transcribed
//! onto synthetic devices/parts (the RFC's scenario devices are fixtures of
//! the proposal tree, not shipped parts; these synthetic twins carry the
//! same pins/kinds so every count and override assertion is the RFC's own).

use std::collections::{BTreeMap, BTreeSet};

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

fn build(src: &str, prior: &LockState) -> cohdl::pipeline::BuildArtifacts {
    let mut checked = check(src);
    build_artifacts(&mut checked, prior).expect("build")
}

const LIB: &str = r#"
pub trait LedPfx { designator_prefix: "D" }
pub device AddressableLED { pins { VDD: 1 [power_in], GND: 2 [power_in], DIN: 3 [input], DOUT: 4 [output] } }
impl LedPfx for AddressableLED {}
pub device Host { pins { V5: 1 [power_out], GND: 2 [power_in], DATA: 3 [output] } }
pub trait ResPfx { designator_prefix: "R" }
pub device SeriesR { pins { A: 1 [passive], B: 2 [passive] } }
impl ResPfx for SeriesR {}
pub trait CapPfx { designator_prefix: "C" }
pub device ShuntC { pins { A: 1 [passive], B: 2 [passive] } }
impl CapPfx for ShuntC {}
pub device SignalSource { pins { OUT: 1 [output] } }
pub device SignalSink { pins { IN: 1 [input] } }
pub device Ground { pins { GND: 1 [power_out] } }
pub footprint FP {}
pub part LEDP: AddressableLED { primary { mfr: "m", mpn: "led", footprint: FP } }
pub part HOSTP: Host { primary { mfr: "m", mpn: "host", footprint: FP } }
pub part RP: SeriesR { primary { mfr: "m", mpn: "r", footprint: FP } }
pub part CP: ShuntC { primary { mfr: "m", mpn: "c", footprint: FP } }
pub part SRCP: SignalSource { primary { mfr: "m", mpn: "src", footprint: FP } }
pub part SNKP: SignalSink { primary { mfr: "m", mpn: "snk", footprint: FP } }
pub part GNDP: Ground { primary { mfr: "m", mpn: "gnd", footprint: FP } }
"#;

fn led_chain(n: i64) -> String {
    format!(
        "{LIB}
design Chain {{
    const N: Int = {n}
    inst host: HOSTP
    inst leds: [LEDP; N]
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
fn led_chain_neighbor_nets_0_1_9() {
    for (n, links) in [(1, 0), (2, 1), (10, 9)] {
        let art = build(&led_chain(n), &LockState::default());
        assert_eq!(
            art.netlist.matches("__for_links_").count(),
            links,
            "N={n}: {links} neighbor nets"
        );
        // instances: host + N leds
        assert_eq!(
            art.netlist.matches("(comp (ref \"").count(),
            (n + 1) as usize
        );
    }
}

fn rc_board(n: i64, loops: bool) -> String {
    let wiring = if loops {
        "for wiring: i in 0..channels.len {
            net _: inputs[i].OUT, channels[i].IN
            net _: channels[i].OUT, outputs[i].IN
            net _: ground.GND, channels[i].GND
        }"
        .to_string()
    } else {
        (0..n).map(|i| format!(
            "net _: inputs[{i}].OUT, channels[{i}].IN\nnet _: channels[{i}].OUT, outputs[{i}].IN\nnet _: ground.GND, channels[{i}].GND\n"
        )).collect()
    };
    let placements = if loops {
        "for placement: i in 0..channels.len { place channels[i] at (10mm + i * 8mm, 15mm) }"
            .to_string()
    } else {
        (0..n)
            .map(|i| format!("place channels[{i}] at ({}mm, 15mm)\n", 10 + i * 8))
            .collect()
    };
    format!(
        "{LIB}
pub subdesign RcChannel {{
    ports {{
        required IN: Pin
        required OUT: Pin
        required GND: Pin
    }}
    inst r: RP
    inst c: CP
    net _: IN, r.A
    net _: OUT, r.B, c.A
    net _: GND, c.B
    layout {{
        place r at (0mm, 0mm)
        place c at (3mm, 0mm)
    }}
}}

design FilterBoard {{
    const N: Int = {n}
    inst inputs: [SRCP; N]
    inst outputs: [SNKP; N]
    inst ground: GNDP
    subdesign channels: [RcChannel; N]

    net GND [gnd]: ground.GND
    {wiring}

    layout {{
        {placements}
        place ground at (0mm, 15mm)
        place channels[6].c at (62mm, 17mm)
    }}
}}"
    )
}

type Endpoints = BTreeSet<(String, String)>;

fn endpoint(path: impl Into<String>, pin: &str) -> (String, String) {
    (path.into(), pin.into())
}

fn assert_net_partition(ir: &cohdl::ir::DesignIr, expected: &BTreeSet<Endpoints>) {
    let actual: BTreeSet<_> = ir.nets.iter().map(|n| n.members.clone()).collect();
    assert_eq!(actual.len(), ir.nets.len(), "no duplicate net partitions");
    assert_eq!(&actual, expected);
    assert!(ir.nc_pins.is_empty());
}

#[test]
fn rc_channels_have_exact_connectivity_placements_parts_and_stable_growth() {
    let mut prior = LockState::default();
    for n in [10, 12] {
        let mut checked = check(&rc_board(n, true));
        let art = build_artifacts(&mut checked, &prior).expect("build RC board");
        let ir = checked.ir.as_ref().unwrap();
        let mut manual = check(&rc_board(n, false));
        let unrolled = build_artifacts(&mut manual, &prior).expect("build manual RC board");
        let manual_ir = manual.ir.as_ref().unwrap();

        let mut ground = Endpoints::from([endpoint("FilterBoard::ground", "GND")]);
        let mut partitions = BTreeSet::new();
        let mut placements = vec![(
            "FilterBoard::ground".to_string(),
            mm(0),
            mm(15),
            0,
            cohdl::ast::PlacementSide::Top,
        )];
        let mut parts = BTreeMap::from([(
            "FilterBoard::ground".to_string(),
            ("board::Ground", "board::GNDP"),
        )]);
        for i in 0..n {
            let r = format!("FilterBoard::channels_{i}::r");
            let c = format!("FilterBoard::channels_{i}::c");
            let input = format!("FilterBoard::inputs_{i}");
            let output = format!("FilterBoard::outputs_{i}");
            ground.insert(endpoint(&c, "B"));
            partitions.insert(Endpoints::from([
                endpoint(&input, "OUT"),
                endpoint(&r, "A"),
            ]));
            partitions.insert(Endpoints::from([
                endpoint(&output, "IN"),
                endpoint(&r, "B"),
                endpoint(&c, "A"),
            ]));
            placements.push((
                r.clone(),
                mm(10 + i as i128 * 8),
                mm(15),
                0,
                cohdl::ast::PlacementSide::Top,
            ));
            let (x, y) = if i == 6 {
                (62, 17)
            } else {
                (13 + i as i128 * 8, 15)
            };
            placements.push((c.clone(), mm(x), mm(y), 0, cohdl::ast::PlacementSide::Top));
            parts.insert(r, ("board::SeriesR", "board::RP"));
            parts.insert(c, ("board::ShuntC", "board::CP"));
            parts.insert(input, ("board::SignalSource", "board::SRCP"));
            parts.insert(output, ("board::SignalSink", "board::SNKP"));
        }
        partitions.insert(ground.clone());
        placements.sort_by(|a, b| a.0.cmp(&b.0));
        for board in [ir, manual_ir] {
            assert_net_partition(board, &partitions);
            let gnd = board
                .nets
                .iter()
                .find(|net| net.name == "GND")
                .expect("exact manufacturing ground name");
            assert_eq!(gnd.members, ground);
            for net in &board.nets {
                assert_eq!(net.is_gnd, net.name == "GND");
                assert!(net.voltage.is_none());
            }
            let mut actual = placement_set(board);
            actual.sort_by(|a, b| a.0.cmp(&b.0));
            assert_eq!(actual, placements);
            let actual_parts: BTreeMap<_, _> = board
                .instances
                .iter()
                .map(|(path, inst)| {
                    assert!(inst.variant.is_none());
                    (
                        path.clone(),
                        (inst.device.as_str(), inst.part.as_deref().unwrap()),
                    )
                })
                .collect();
            assert_eq!(actual_parts, parts);
        }
        // This checks every populated row, rather than a component count or
        // an override coordinate found somewhere in the document.
        assert_eq!(art.bom, unrolled.bom);
        assert_eq!(art.layout, unrolled.layout);
        assert_eq!(art.lock, unrolled.lock);
        for (path, designator) in &prior.designators {
            assert_eq!(art.lock.designators.get(path), Some(designator));
        }
        assert_eq!(art.lock.designators.len(), parts.len());
        let repeated = build(&rc_board(n, true), &art.lock);
        assert_eq!(repeated.lock, art.lock);
        assert_eq!(repeated.netlist, art.netlist);
        assert_eq!(repeated.bom, art.bom);
        assert_eq!(repeated.layout, art.layout);
        prior = art.lock;
    }
}

fn bank_board(loops: bool) -> String {
    let wiring = if loops {
        "for wiring: i in 0..N { join(IN, channels[i].IN) join(channels[i].OUT, receivers[i].IN) net _: GND, channels[i].GND }".to_string()
    } else {
        (0..3).map(|i| format!("join(IN, channels[{i}].IN)\njoin(channels[{i}].OUT, receivers[{i}].IN)\nnet _: GND, channels[{i}].GND\n")).collect()
    };
    let placements = if loops {
        "for placement: i in 0..N { place channels[i] at (i * 8mm, 0mm) place receivers[i] at (i * 8mm, 10mm) }".to_string()
    } else {
        (0..3)
            .map(|i| {
                format!(
                    "place channels[{i}] at ({}mm, 0mm)\nplace receivers[{i}] at ({}mm, 10mm)\n",
                    i * 8,
                    i * 8
                )
            })
            .collect()
    };
    format!(
        "{LIB}
pub subdesign RcChannel {{
    ports {{
        required IN: Pin
        required OUT: Pin
        required GND: Pin
    }}
    inst r: RP
    inst c: CP
    net _: IN, r.A
    net _: OUT, r.B, c.A
    net _: GND, c.B
    layout {{
        place r at (0mm, 0mm)
        place c at (3mm, 0mm)
    }}
}}

pub fn join(src: Pin, dst: Pin) {{
    net _: src, dst
}}

pub subdesign FilterBank<const N: Int> {{
    ports {{
        required IN: Pin
        required GND: Pin
    }}
    inst receivers: [SNKP; N]
    subdesign channels: [RcChannel; N]
    {wiring}
    layout {{
        {placements}
    }}
}}

design Board {{
    inst source: SRCP
    inst ground: GNDP
    net INPUT: source.OUT
    net GND [gnd]: ground.GND
    subdesign bank: FilterBank<3> {{
        IN: source.OUT,
        GND: ground.GND,
    }}
    layout {{
        place bank at (10mm, 15mm)
        place bank.channels[1].c at (22mm, 17mm)
    }}
}}"
    )
}

#[test]
fn nested_filter_bank_has_exact_nets_parts_and_every_composed_placement() {
    let mut checked = check(&bank_board(true));
    let art = build_artifacts(&mut checked, &LockState::default()).expect("build bank");
    let mut manual = check(&bank_board(false));
    let unrolled = build_artifacts(&mut manual, &LockState::default()).expect("build manual bank");
    let mut input = Endpoints::from([endpoint("Board::source", "OUT")]);
    let mut ground = Endpoints::from([endpoint("Board::ground", "GND")]);
    let mut partitions = BTreeSet::new();
    let mut placements = Vec::new();
    let mut parts = BTreeMap::from([
        (
            "Board::source".to_string(),
            ("board::SignalSource", "board::SRCP"),
        ),
        (
            "Board::ground".to_string(),
            ("board::Ground", "board::GNDP"),
        ),
    ]);
    for i in 0..3 {
        let r = format!("Board::bank::channels_{i}::r");
        let c = format!("Board::bank::channels_{i}::c");
        let receiver = format!("Board::bank::receivers_{i}");
        input.insert(endpoint(&r, "A"));
        ground.insert(endpoint(&c, "B"));
        partitions.insert(Endpoints::from([
            endpoint(&r, "B"),
            endpoint(&c, "A"),
            endpoint(&receiver, "IN"),
        ]));
        placements.push((
            r.clone(),
            mm(10 + i * 8),
            mm(15),
            0,
            cohdl::ast::PlacementSide::Top,
        ));
        let (x, y) = if i == 1 { (22, 17) } else { (13 + i * 8, 15) };
        placements.push((c.clone(), mm(x), mm(y), 0, cohdl::ast::PlacementSide::Top));
        placements.push((
            receiver.clone(),
            mm(10 + i * 8),
            mm(25),
            0,
            cohdl::ast::PlacementSide::Top,
        ));
        parts.insert(r, ("board::SeriesR", "board::RP"));
        parts.insert(c, ("board::ShuntC", "board::CP"));
        parts.insert(receiver, ("board::SignalSink", "board::SNKP"));
    }
    partitions.insert(input.clone());
    partitions.insert(ground.clone());
    placements.sort_by(|a, b| a.0.cmp(&b.0));
    for ir in [checked.ir.as_ref().unwrap(), manual.ir.as_ref().unwrap()] {
        assert_net_partition(ir, &partitions);
        for (name, ends) in [("INPUT", &input), ("GND", &ground)] {
            assert_eq!(
                &ir.nets
                    .iter()
                    .find(|net| net.name == name)
                    .expect("named rail")
                    .members,
                ends
            );
        }
        for net in &ir.nets {
            assert_eq!(net.is_gnd, net.name == "GND");
            assert!(net.voltage.is_none());
        }
        let mut actual = placement_set(ir);
        actual.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(actual, placements);
        let actual_parts: BTreeMap<_, _> = ir
            .instances
            .iter()
            .map(|(path, inst)| {
                assert!(inst.variant.is_none());
                (
                    path.clone(),
                    (inst.device.as_str(), inst.part.as_deref().unwrap()),
                )
            })
            .collect();
        assert_eq!(actual_parts, parts);
    }
    assert_eq!(art.bom, unrolled.bom);
    assert_eq!(art.layout, unrolled.layout);
    assert_eq!(art.lock, unrolled.lock);
    let repeated = build(&bank_board(true), &art.lock);
    assert_eq!(repeated.netlist, art.netlist);
    assert_eq!(repeated.bom, art.bom);
    assert_eq!(repeated.layout, art.layout);
    assert_eq!(repeated.lock, art.lock);
}

// ---------------------------------------------------------------------------
// Subdesign layout `for` loops must keep their placements. The frame a
// default is recorded against is the subdesign NODE, not the loop-qualified
// `__for_{label}_{value}` path — otherwise `default_placements` finds no
// anchor for the owner and silently drops every loop-authored placement.
// ---------------------------------------------------------------------------

/// femto-mm value of a whole-mm coordinate.
fn mm(x: i128) -> i128 {
    x * 1_000_000_000_000_000
}

/// The complete board placement mapping, in collection order, compared for
/// exact equality — a dropped row or a phantom row both fail.
fn placement_set(
    ir: &cohdl::ir::DesignIr,
) -> Vec<(String, i128, i128, u16, cohdl::ast::PlacementSide)> {
    ir.layout
        .placements
        .iter()
        .map(|p| (p.path.clone(), p.at.0.femto, p.at.1.femto, p.rotate, p.side))
        .collect()
}

#[test]
fn subdesign_layout_loop_composes_the_full_expected_set() {
    // Bank<3>'s layout loop places its resistor array; the board anchors
    // source (0,0) and the whole bank (20,20). Expected: source (0,0),
    // b.rs_0 (20,20), b.rs_1 (25,20), b.rs_2 (30,20) — the complete set,
    // nothing dropped, nothing extra.
    let src = format!(
        "{LIB}
pub subdesign Bank<const N: Int> {{
    ports {{ required IN: Pin }}
    inst rs: [RP; N]
    for w: i in 0..N {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{ for p: i in 0..N {{ place rs[i] at (i * 5mm, 0mm) }} }}
}}
design Board {{
    inst source: SRCP
    subdesign b: Bank<3> {{ IN: source.OUT, }}
    layout {{ place source at (0mm, 0mm)  place b at (20mm, 20mm) }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    // Explicit placements keep declaration order; composed defaults append
    // after, in path order.
    let got = placement_set(ir);
    let want = vec![
        (
            "Board::source".to_string(),
            mm(0),
            mm(0),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::b::rs_0".to_string(),
            mm(20),
            mm(20),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::b::rs_1".to_string(),
            mm(25),
            mm(20),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::b::rs_2".to_string(),
            mm(30),
            mm(20),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
    ];
    assert_eq!(got, want, "the full placement mapping, exactly");
}

#[test]
fn subdesign_layout_loop_inside_two_instances_composes_independently() {
    // Two use sites of the same looping subdesign: each instance composes
    // through its own anchor, proving the owner is the real node per instance.
    let src = format!(
        "{LIB}
pub subdesign Pair {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 2]
    for w: i in 0..2 {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{ for p: i in 0..2 {{ place rs[i] at (i * 3mm, 1mm) }} }}
}}
design Board {{
    inst source: SRCP
    subdesign a: Pair {{ IN: source.OUT, }}
    subdesign b: Pair {{ IN: source.OUT, }}
    layout {{
        place source at (0mm, 0mm)
        place a at (10mm, 0mm)
        place b at (10mm, 10mm) rotate 90
    }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    // b rotate 90, top: inv=270 → (dx,dy)→(dy,−dx).
    // rs_0 local (0,1) → (1,0) → (11,10), rotate 0+90=90.
    // rs_1 local (3,1) → (1,−3) → (11,7), rotate 90.
    // Row order: the explicit `place source` first, then composed defaults in
    // path order (a::rs_0, a::rs_1, b::rs_0, b::rs_1).
    let got = placement_set(ir);
    let want = vec![
        (
            "Board::source".to_string(),
            mm(0),
            mm(0),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::a::rs_0".to_string(),
            mm(10),
            mm(1),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::a::rs_1".to_string(),
            mm(13),
            mm(1),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::b::rs_0".to_string(),
            mm(11),
            mm(10),
            90,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::b::rs_1".to_string(),
            mm(11),
            mm(7),
            90,
            cohdl::ast::PlacementSide::Top,
        ),
    ];
    assert_eq!(
        got, want,
        "each instance composes through its own anchor, complete mapping"
    );
}

#[test]
fn subdesign_layout_loop_survives_side_flip_and_outer_override() {
    // A back-side anchor mirrors x before rotating and flips the child side;
    // a board-level reach-in override replaces exactly one loop-authored row.
    let src = format!(
        "{LIB}
pub subdesign Pair {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 2]
    for w: i in 0..2 {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{ for p: i in 0..2 {{ place rs[i] at (i * 4mm, 2mm) }} }}
}}
design Board {{
    inst source: SRCP
    subdesign p: Pair {{ IN: source.OUT, }}
    layout {{
        place source at (0mm, 0mm)
        place p at (10mm, 10mm) rotate 180 side bottom
        place p.rs[1] at (1mm, 1mm) rotate 45
    }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    // rs_0 local (0,2), bottom: mirror → (0,2); inv(180)=180 → (0,−2);
    // absolute (10,8); rotate = 180+360−0 = 180 (reflection reverses);
    // side flips to bottom. rs_1 is the explicit override, verbatim.
    // Row order: source, the reach-in override p.rs[1], then the composed
    // default p.rs_0 (defaults append in path order after explicit rows).
    let got = placement_set(ir);
    let want = vec![
        (
            "Board::source".to_string(),
            mm(0),
            mm(0),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::p::rs_1".to_string(),
            mm(1),
            mm(1),
            45,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::p::rs_0".to_string(),
            mm(10),
            mm(8),
            180,
            cohdl::ast::PlacementSide::Bottom,
        ),
    ];
    assert_eq!(
        got, want,
        "reflection math + override, as the complete mapping"
    );
}

#[test]
fn subdesign_layout_loop_unanchored_stages_locals_but_not_board() {
    // Without a board anchor the loop-authored defaults stay staged (tooling
    // sees them via local_placements) and the manufacturing layout stays empty.
    let src = format!(
        "{LIB}
pub subdesign Pair {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 2]
    for w: i in 0..2 {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{ for p: i in 0..2 {{ place rs[i] at (i * 3mm, 0mm) }} }}
}}
design Board {{
    inst source: SRCP
    subdesign p: Pair {{ IN: source.OUT, }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    assert!(
        ir.layout.placements.is_empty(),
        "unanchored: no board placements, got {:?}",
        ir.layout
            .placements
            .iter()
            .map(|p| &p.path)
            .collect::<Vec<_>>()
    );
    let local = &ir.subdesigns["Board::p"].local_placements;
    assert_eq!(local.len(), 2, "staged authored defaults");
    let l0 = local.iter().find(|p| p.path == "Board::p::rs_0").unwrap();
    assert_eq!((l0.at.0.femto, l0.at.1.femto, l0.rotate), (mm(0), mm(0), 0));
    let l1 = local.iter().find(|p| p.path == "Board::p::rs_1").unwrap();
    assert_eq!((l1.at.0.femto, l1.at.1.femto), (mm(3), mm(0)));
}

#[test]
fn subdesign_layout_loop_duplicate_still_e1007() {
    // A loop-authored default for the same target as a sibling default in the
    // SAME subdesign layout is still a duplicate (E1007) — exactly one
    // diagnostic, pointing at the loop's placement site.
    let src = format!(
        "{LIB}
pub subdesign S {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 2]
    for w: i in 0..2 {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{
        place rs[0] at (0mm, 0mm)
        for p: i in 0..2 {{ place rs[i] at (i * 5mm, 0mm) }}
    }}
}}
design Board {{
    inst source: SRCP
    subdesign s: S {{ IN: source.OUT, }}
    layout {{ place source at (0mm, 0mm)  place s at (1mm, 1mm) }}
}}"
    );
    let files = vec![("src/main.cohdl".to_string(), src)];
    let mut checked = cohdl::pipeline::check_files_in("board", &files, None).expect("selection");
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &cohdl::lock::LockState::default());
    checked.diags.sort(&checked.sm);
    let e1007: Vec<&cohdl::diag::Diagnostic> =
        checked.diags.iter().filter(|d| d.code == "E1007").collect();
    assert_eq!(e1007.len(), 1, "exactly one E1007, got all: {e1007:?}");
    let d = e1007[0];
    assert_eq!(
        d.message,
        "`rs[i]` is placed more than once; target `Board::s::rs_0` — in Board::s::__for_p_0, i = 0"
    );
    assert!(matches!(d.severity, cohdl::diag::Severity::Error));
    // The primary span names the loop's placement path itself.
    assert_eq!(checked.sm.snippet(d.primary.span), "rs[i]");
}

#[test]
fn nested_subdesign_layout_loops_two_instances_compose() {
    // Nested subdesigns, EACH with a layout for loop, both instantiated at the
    // design level — the inner loop's owner is the inner node, inherited
    // through the middle layer's own loop-bearing layout.
    let src = format!(
        "{LIB}
pub subdesign Leaf {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 2]
    for w: i in 0..2 {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{ for p: i in 0..2 {{ place rs[i] at (i * 2mm, 0mm) }} }}
}}
pub subdesign Mid {{
    ports {{ required IN: Pin }}
    subdesign leaves: [Leaf; 2]
    for w: i in 0..2 {{ net _: IN, leaves[i].IN }}
    layout {{
        for p: i in 0..2 {{ place leaves[i] at (i * 10mm, 0mm) }}
    }}
}}
design Board {{
    inst source: SRCP
    subdesign m1: Mid {{ IN: source.OUT, }}
    subdesign m2: Mid {{ IN: source.OUT, }}
    layout {{
        place source at (0mm, 0mm)
        place m1 at (100mm, 100mm)
        place m2 at (0mm, 200mm)
    }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    // m1: leaves_0 at (100,100) → rs at (100,100),(102,100);
    //     leaves_1 at (110,100) → rs at (110,100),(112,100).
    // m2: leaves_0 at (0,200) → (0,200),(2,200);
    //     leaves_1 at (10,200) → (10,200),(12,200).
    // Row order: the three explicit rows in declaration order, then the
    // composed defaults in path order.
    let got = placement_set(ir);
    let want = vec![
        ("Board::source", 0, 0),
        ("Board::m1::leaves_0::rs_0", 100, 100),
        ("Board::m1::leaves_0::rs_1", 102, 100),
        ("Board::m1::leaves_1::rs_0", 110, 100),
        ("Board::m1::leaves_1::rs_1", 112, 100),
        ("Board::m2::leaves_0::rs_0", 0, 200),
        ("Board::m2::leaves_0::rs_1", 2, 200),
        ("Board::m2::leaves_1::rs_0", 10, 200),
        ("Board::m2::leaves_1::rs_1", 12, 200),
    ]
    .into_iter()
    .map(|(p, x, y)| {
        (
            p.to_string(),
            mm(x),
            mm(y),
            0,
            cohdl::ast::PlacementSide::Top,
        )
    })
    .collect::<Vec<_>>();
    assert_eq!(
        got, want,
        "nested loops, two instances, complete mapping incl. the source row"
    );
}

#[test]
fn subdesign_circuit_for_with_nested_layout_block_places_every_element() {
    // A circuit-body `for` whose body carries its own `layout {}` block: the
    // block's placements are defaults of the enclosing subdesign node, and
    // every iteration's rows must survive and compose through the node's
    // anchor. The layout loop inside a `layout {}` block is a different form
    // — this one nests the whole layout block inside the circuit loop.
    let src = format!(
        "{LIB}
pub subdesign Grid {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 4]
    for w: i in 0..4 {{
        net _: IN, rs[i].A, rs[i].B
        layout {{
            place rs[i] at (i * 5mm, 3mm)
        }}
    }}
}}
design Board {{
    inst source: SRCP
    subdesign g: Grid {{ IN: source.OUT, }}
    layout {{ place source at (0mm, 0mm)  place g at (10mm, 10mm) }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    // g anchored at (10,10): rs_i at (10+i*5, 13). Explicit source row first,
    // composed defaults after in path order.
    let got = placement_set(ir);
    let defaults: Vec<(String, i128, i128, u16, cohdl::ast::PlacementSide)> =
        [(0, 10, 13), (1, 15, 13), (2, 20, 13), (3, 25, 13)]
            .into_iter()
            .map(|(i, x, y)| {
                (
                    format!("Board::g::rs_{i}"),
                    mm(x),
                    mm(y),
                    0,
                    cohdl::ast::PlacementSide::Top,
                )
            })
            .collect();
    let mut want = vec![(
        "Board::source".to_string(),
        mm(0),
        mm(0),
        0,
        cohdl::ast::PlacementSide::Top,
    )];
    want.extend(defaults);
    assert_eq!(got, want, "circuit-for nested layout, complete mapping");
}
