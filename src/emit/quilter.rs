//! RFC-027: the Quilter physics-constraint CSV set.
//!
//! Eight files, headers and column order matching the real supplied templates
//! exactly. Emitted as a SET whenever the design carries at least one physics
//! attribute or an annotated `diff_pair` — kinds a design does not use still
//! emit their header row (matching the supplied header-only files), so a
//! consumer always sees the full, fixed file set.
//!
//! Number scales are the templates' own: max_current in mA, capacitance in
//! nF, impedances in ohm, frequency in GHz. Booleans render lowercase.
//! Component references are final designators; pins are PAD numbers — a
//! multi-pad pin flattens into one row per pad (the supplied
//! `bypass_capacitors.csv` shows exactly this shape).

use crate::emit::geom::scaled;
use crate::ir::DesignIr;

/// The eight (file name, content) pairs, or `None` when the design carries no
/// physics facts at all (pre-RFC-027 outputs stay byte-identical: no files).
pub fn emit_quilter_csvs(ir: &DesignIr) -> Option<Vec<(String, String)>> {
    let l = &ir.layout;
    let annotated_pair = |d: &crate::ir::LayoutDiffPair| {
        d.differential_impedance.is_some()
            || d.single_ended_impedance.is_some()
            || d.frequency.is_some()
    };
    let any = !l.grounds.is_empty()
        || !l.high_currents.is_empty()
        || !l.impedances.is_empty()
        || !l.bypasses.is_empty()
        || !l.crystals.is_empty()
        || !l.converters.is_empty()
        || !l.bga_fanouts.is_empty()
        || l.diff_pairs.iter().any(annotated_pair);
    if !any {
        return None;
    }
    // Final designator per instance path (assigned before emission at build).
    let des = |path: &str| -> String {
        ir.instances
            .get(path)
            .and_then(|i| i.designator.clone())
            .unwrap_or_else(|| "?".to_string())
    };
    let b = |v: bool| if v { "true" } else { "false" };

    let mut out = Vec::new();

    let mut s = String::from("component,generate_fanout\n");
    for path in &l.bga_fanouts {
        s.push_str(&format!("{},true\n", des(path)));
    }
    out.push(("bga_components.csv".to_string(), s));

    let mut s = String::from("capacitor,bypassed_component,bypassed_pin,capacitance\n");
    for bp in &l.bypasses {
        for pad in &bp.pads {
            s.push_str(&format!(
                "{},{},{},{}\n",
                des(&bp.cap_path),
                des(&bp.target_path),
                pad,
                scaled(bp.capacitance.femto, 6), // nF
            ));
        }
    }
    out.push(("bypass_capacitors.csv".to_string(), s));

    let mut s = String::from("crystal,crystal_parent,parent_signal_pin_1,parent_signal_pin_2\n");
    for c in &l.crystals {
        s.push_str(&format!(
            "{},{},{},{}\n",
            des(&c.crystal_path),
            des(&c.parent_path),
            c.pad1,
            c.pad2
        ));
    }
    out.push(("crystal_oscillators.csv".to_string(), s));

    let mut s = String::from(
        "positive_net_name,negative_net_name,differential_impedance,single_ended_impedance,frequency\n",
    );
    for d in l.diff_pairs.iter().filter(|d| annotated_pair(d)) {
        let cell = |v: &Option<crate::units::UnitValue>, scale: u32| {
            v.as_ref()
                .map(|u| scaled(u.femto, scale))
                .unwrap_or_default()
        };
        s.push_str(&format!(
            "{},{},{},{},{}\n",
            d.p,
            d.n,
            cell(&d.differential_impedance, 15), // ohm
            cell(&d.single_ended_impedance, 15), // ohm
            cell(&d.frequency, 24),              // GHz
        ));
    }
    out.push(("differential_pairs.csv".to_string(), s));

    let mut s = String::from("net_name,is_primary,use_region_pour\n");
    for g in &l.grounds {
        s.push_str(&format!(
            "{},{},{}\n",
            g.net,
            b(g.primary),
            b(g.region_pour)
        ));
    }
    out.push(("ground_nets.csv".to_string(), s));

    let mut s = String::from("net_name,max_current,use_power_pour\n");
    for h in &l.high_currents {
        s.push_str(&format!(
            "{},{},{}\n",
            h.net,
            scaled(h.current.femto, 12), // mA
            b(h.power_pour)
        ));
    }
    out.push(("high_current_nets.csv".to_string(), s));

    let mut s = String::from("net_name,single_ended_impedance,frequency\n");
    for i in &l.impedances {
        s.push_str(&format!(
            "{},{},{}\n",
            i.net,
            scaled(i.impedance.femto, 15), // ohm
            scaled(i.frequency.femto, 24), // GHz
        ));
    }
    out.push(("single_ended_impedance_signals.csv".to_string(), s));

    let mut s = String::from("switching_converter,inductor,input_capacitor,output_capacitor\n");
    for c in &l.converters {
        s.push_str(&format!(
            "{},{},{},{}\n",
            des(&c.conv_path),
            des(&c.inductor_path),
            c.input_cap_path.as_deref().map(des).unwrap_or_default(),
            c.output_cap_path.as_deref().map(des).unwrap_or_default(),
        ));
    }
    out.push(("switching_converters.csv".to_string(), s));

    Some(out)
}
