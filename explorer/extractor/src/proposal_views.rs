use serde_json::Value;
use std::collections::BTreeSet;

fn array(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn identities(model: &Value) -> BTreeSet<String> {
    let mut identities = BTreeSet::new();
    for instance in array(&model["instances"]) {
        if let Some(designator) = instance["designator"].as_str() {
            identities.insert(designator.into());
        }
        if let Some(path) = instance["path"].as_str() {
            identities.insert(format!("path:{path}"));
        }
    }
    let rails: BTreeSet<_> = array(&model["derived"]["rails"])
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for path in array(&model["derived"]["two_terminal"]) {
        let mut nets: Vec<_> = array(&model["nets"])
            .iter()
            .filter(|n| {
                array(&n["members"])
                    .iter()
                    .any(|m| m["instance_path"] == *path)
            })
            .filter_map(|n| n["name"].as_str())
            .collect();
        if nets.is_empty() || nets.iter().any(|net| !rails.contains(net)) {
            continue;
        }
        let bypass = array(&model["derived"]["bypasses"]).iter().find(|b| {
            b["cap"] == *path
                && array(&model["instances"])
                    .iter()
                    .any(|i| i["path"] == b["target"])
        });
        if let Some(target) = bypass.and_then(|b| b["target"].as_str()) {
            identities.insert(format!("agg:host:{target}"));
        } else {
            nets.sort();
            identities.insert(format!("agg:{}", nets.join(" + ")));
        }
    }
    identities
}

pub fn validate(bytes: &[u8], bundle: &Value) -> Result<(), String> {
    let design = bundle["design"].as_str().ok_or("Design is missing")?;
    let config: Value = serde_json::from_slice(bytes).map_err(|_| "Invalid view JSON")?;
    let views = config["views"]
        .as_array()
        .filter(|a| a.len() <= 128)
        .ok_or("Too many views")?;
    let identities = identities(&bundle["model"]);
    let mut targets = BTreeSet::new();
    for view in views {
        let regions = view["regions"]
            .as_array()
            .filter(|a| a.len() <= 512)
            .ok_or("Too many view regions")?;
        for region in regions {
            let members = region["members"]
                .as_array()
                .filter(|a| a.len() <= 2048)
                .ok_or("Too many group members")?;
            if members
                .iter()
                .any(|m| !m.as_str().is_some_and(|m| identities.contains(m)))
            {
                return Err("Proposed view names an unknown member".into());
            }
            targets.insert((
                view["name"].as_str().unwrap_or(""),
                region["name"].as_str().unwrap_or(""),
            ));
        }
    }
    let documents = array(&bundle["registry"]["documents"]);
    let applications = documents.iter().flat_map(|d| array(&d["applicability"]));
    let constraints = array(&bundle["registry"]["constraints"]);
    for record in applications.chain(constraints) {
        for group in array(&record["groups"]) {
            let target = (
                group["view"].as_str().unwrap_or(""),
                group["region"].as_str().unwrap_or(""),
            );
            if !targets.contains(&target) {
                return Err(
                    "Group edit removes a document or constraint applicability target".into(),
                );
            }
        }
    }
    crate::view_config::validate(bytes, design)
}
