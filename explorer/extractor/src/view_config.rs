use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct ViewConfig {
    schema_version: u32,
    design: String,
    views: Vec<View>,
}

#[derive(Deserialize)]
struct View {
    name: String,
    regions: Vec<Region>,
}

#[derive(Deserialize)]
struct Region {
    name: String,
    members: Vec<String>,
}

fn name<'a>(value: &'a str, seen: &mut BTreeSet<&'a str>) -> Result<(), String> {
    if value.is_empty() || value.trim() != value || !seen.insert(value) {
        return Err("names must be unique, nonempty trimmed strings".into());
    }
    Ok(())
}

fn valid_member(member: &str) -> bool {
    if member.is_empty() || member.trim() != member {
        return false;
    }
    let suffix = if let Some(host) = member.strip_prefix("agg:host:") {
        host
    } else if let Some(suffix) = member
        .strip_prefix("path:")
        .or_else(|| member.strip_prefix("agg:"))
    {
        suffix
    } else {
        return !member.contains(':');
    };
    !suffix.is_empty() && !suffix.starts_with(char::is_whitespace)
}

pub fn validate(bytes: &[u8], design: &str) -> Result<(), String> {
    let config: ViewConfig =
        serde_json::from_slice(bytes).map_err(|e| format!("invalid view config: {e}"))?;
    if config.schema_version != 1 || config.design != design {
        return Err(format!(
            "view config requires schema_version 1 and design {design}"
        ));
    }
    if config.views.is_empty() {
        return Err("views must be a nonempty array".into());
    }
    let mut views = BTreeSet::new();
    for view in &config.views {
        name(&view.name, &mut views)?;
        let mut regions = BTreeSet::new();
        for region in &view.regions {
            name(&region.name, &mut regions)?;
            if region.name.eq_ignore_ascii_case("other") {
                return Err("Other is reserved for unmatched members".into());
            }
            if let Some(member) = region.members.iter().find(|m| !valid_member(m)) {
                return Err(format!("invalid member rule: {member}"));
            }
        }
    }
    Ok(())
}
