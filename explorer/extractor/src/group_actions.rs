use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn execute(project: &Path, bytes: &[u8]) -> Result<Value, String> {
    let request: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if !matches!(
        request["action"].as_str(),
        Some("propose" | "focus" | "save" | "undo")
    ) {
        return Err("Invalid display group action".into());
    }
    let script = std::env::var_os("COHDL_ASSIST_CLI")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../assist/cli.mjs"));
    let mut child = Command::new(std::env::var_os("COHDL_NODE").unwrap_or_else(|| "node".into()))
        .arg(script)
        .arg("group")
        .arg(project)
        .args(["--request", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Group CLI unavailable: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("Group CLI input unavailable")?
        .write_all(bytes)
        .map_err(|e| e.to_string())?;
    let result = child.wait_with_output().map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&result.stdout)
        .map_err(|_| "Group CLI returned no valid JSON evidence".to_string())?;
    if !result.status.success() {
        return Err(value["error"].as_str().unwrap_or("Group CLI failed").into());
    }
    Ok(value)
}

pub fn response(project: &Path, bytes: &[u8]) -> (u16, Vec<u8>) {
    match execute(project, bytes) {
        Ok(value) => (200, value.to_string().into_bytes()),
        Err(error) => (
            422,
            json!({"schema":1,"error":error}).to_string().into_bytes(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn group_endpoint_rejects_unknown_and_electrical_actions_before_process() {
        for bytes in [
            b"{}".as_slice(),
            b"{\"action\":\"connect\"}",
            b"null",
            b"invalid",
        ] {
            assert!(execute(Path::new("/missing"), bytes).is_err());
        }
    }
}
