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
    #[serde(default)]
    navigation: Vec<Navigation>,
}

#[derive(Deserialize)]
struct Navigation {
    name: String,
    groups: Vec<String>,
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
        let mut parents = BTreeSet::new();
        for parent in &view.navigation {
            name(&parent.name, &mut parents)?;
            if parent.groups.is_empty() {
                return Err("navigation groups must be a nonempty array".into());
            }
            let mut children = BTreeSet::new();
            for child in &parent.groups {
                name(child, &mut children)?;
                if !regions.contains(child.as_str()) {
                    return Err(format!("navigation references unknown group: {child}"));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate;
    use serde_json::json;

    #[test]
    fn navigation_references_canonical_regions_and_permits_shared_paths() {
        let value = json!({"schema_version":1,"design":"Board","views":[{"name":"Functional",
            "regions":[{"name":"USB","members":["U1"]}],
            "navigation":[{"name":"Host","groups":["USB"]},{"name":"Shared","groups":["USB"]}]}]});
        assert!(validate(value.to_string().as_bytes(), "Board").is_ok());
    }

    #[test]
    fn navigation_rejects_unknown_duplicate_empty_and_malformed_paths() {
        for navigation in [
            json!(null),
            json!({}),
            json!([{"name":"Host","groups":[]}]),
            json!([{"name":"Host","groups":["Unknown"]}]),
            json!([{"name":"Host","groups":["USB","USB"]}]),
            json!([{"name":"Host","groups":["USB"]},{"name":"Host","groups":["USB"]}]),
        ] {
            let value = json!({"schema_version":1,"design":"Board","views":[{"name":"Functional",
                "regions":[{"name":"USB","members":["U1"]}],"navigation":navigation}]});
            assert!(
                validate(value.to_string().as_bytes(), "Board").is_err(),
                "{value}"
            );
        }
    }
}
