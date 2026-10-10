use cohdl::emit::geom::mm_femto;

pub fn native_length(s: &str) -> i128 {
    let (whole, fraction) = s.split_once('.').unwrap_or((s, ""));
    assert!(fraction.len() <= 4);
    let scale = 10_i128.pow(fraction.len() as u32);
    let whole: i128 = whole.parse().unwrap();
    let fraction: i128 = if fraction.is_empty() {
        0
    } else {
        fraction.parse().unwrap()
    };
    (whole * scale + fraction) * 254_000_000_000_000 / scale
}

pub fn records() -> serde_json::Value {
    serde_json::from_str(include_str!("../fixtures/native_polygon_pads.json")).unwrap()
}

pub fn vertices(raw: &str) -> Vec<(i128, i128)> {
    let f: Vec<_> = raw.split('~').collect();
    let coords: Vec<_> = f[10].split_whitespace().map(native_length).collect();
    coords
        .chunks_exact(2)
        .map(|p| (p[0] - native_length(f[2]), p[1] - native_length(f[3])))
        .collect()
}

pub fn source() -> String {
    let mut out = String::new();
    for record in records().as_array().unwrap() {
        let name = record["ref"].as_str().unwrap();
        let pads = record["pads"].as_array().unwrap();
        let first: Vec<_> = pads[0].as_str().unwrap().split('~').collect();
        let (ox, oy) = (native_length(first[2]), native_length(first[3]));
        let mut placements = String::new();
        for (i, pad) in pads.iter().enumerate() {
            let raw = pad.as_str().unwrap();
            let f: Vec<_> = raw.split('~').collect();
            let polygon = f[1] == "POLYGON";
            out.push_str(&format!("pub pad {name}P{i} {{ shape: {}, size: ({}mm, {}mm), layer: top_copper, plating: smd", if polygon { "polygon" } else { "rect" }, mm_femto(native_length(f[4])), mm_femto(native_length(f[5]))));
            if polygon {
                let points = vertices(raw)
                    .iter()
                    .map(|(x, y)| format!("({}mm, {}mm)", mm_femto(*x), mm_femto(*y)))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!(", vertices: [{points}]"));
            }
            out.push_str(" }\n");
            placements.push_str(&format!(
                "pad {}: {name}P{i} at ({}mm, {}mm) rotate {}\n",
                f[8],
                mm_femto(native_length(f[2]) - ox),
                mm_femto(native_length(f[3]) - oy),
                f[11]
            ));
        }
        out.push_str(&format!("pub footprint {name}FP {{ {placements} }}\npub device {name}D {{ pins {{ P: {} [passive] }} }}\npub part {name}Part: {name}D {{ primary {{ mfr: \"native\", mpn: \"{}\", footprint: {name}FP }} }}\n", first[8], record["lcsc"].as_str().unwrap()));
    }
    out.push_str("design B { inst u1: U1Part inst u10: U10Part nc: u1.P, u10.P }\n");
    out
}
