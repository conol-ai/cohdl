//! Double-click flow for the macOS app bundle (CoHDL Explorer.app): choose a
//! project folder, serve it on a free loopback port, open the browser, and
//! exit once the last tab has been gone for a while (the bundle is
//! LSUIElement — no Dock icon — so the server must retire itself).
//!
//! Dev knobs, used by the release smoke test and honored nowhere else:
//! `COHDL_EXPLORER_PROJECT` skips the folder dialog, `COHDL_EXPLORER_NO_OPEN`
//! suppresses the browser.

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// How long the server outlives its last SSE viewer. A closed tab drops the
/// event stream within a keepalive tick, so this is "minutes after the last
/// tab closed", not "minutes of not clicking".
const IDLE_EXIT: Duration = Duration::from_secs(5 * 60);

/// App mode triggers on the bundle layout itself, not a flag: Finder launches
/// the plain binary at CoHDL Explorer.app/Contents/MacOS/ with no argv.
pub fn running_from_bundle() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.contains(".app/Contents/MacOS/")))
        .unwrap_or(false)
}

pub fn run() -> Result<(), String> {
    let project = match std::env::var_os("COHDL_EXPLORER_PROJECT") {
        Some(p) => PathBuf::from(p),
        None => match choose_project()? {
            Some(p) => p,
            None => return Ok(()), // dialog cancelled
        },
    };
    let dist = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|m| m.join("../Resources/web")))
        .ok_or("cannot locate Resources/web next to the executable")?;
    // Free loopback port: bind 0, read the assignment, release. The gap
    // before serve() rebinds is a race in principle, not in practice.
    let port = std::net::TcpListener::bind(("127.0.0.1", 0))
        .and_then(|l| l.local_addr())
        .map_err(|e| e.to_string())?
        .port();
    if std::env::var_os("COHDL_EXPLORER_NO_OPEN").is_none() {
        open_when_ready(port);
    }
    // stderr is invisible from Finder, so a failed serve (bad lock, missing
    // deps, port trouble) surfaces as an alert before the process ends.
    crate::serve::serve(&project, &dist, port, Some(IDLE_EXIT)).inspect_err(|e| alert(e))
}

fn choose_project() -> Result<Option<PathBuf>, String> {
    loop {
        let out = Command::new("osascript")
            .args([
                "-e",
                "POSIX path of (choose folder with prompt \
                 \"Choose a CoHDL project (the folder holding cohdl.toml)\")",
            ])
            .output()
            .map_err(|e| format!("osascript: {e}"))?;
        if !out.status.success() {
            return Ok(None); // cancel button
        }
        let path = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
        if path.join("cohdl.toml").is_file() {
            return Ok(Some(path));
        }
        alert(&format!(
            "{} has no cohdl.toml.\n\nChoose the project directory itself.",
            path.display()
        ));
    }
}

fn alert(msg: &str) {
    let script = format!(
        "display alert \"CoHDL Explorer\" message \"{}\" as critical",
        msg.replace('\\', "\\\\").replace('"', "\\\"")
    );
    let _ = Command::new("osascript").args(["-e", &script]).status();
}

/// The first extraction happens before the listener binds, so poll the port
/// (up to a minute, for big projects) and only then hand the URL to `open`.
fn open_when_ready(port: u16) {
    std::thread::spawn(move || {
        for _ in 0..600 {
            if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
                let _ = Command::new("open")
                    .arg(format!("http://127.0.0.1:{port}/"))
                    .status();
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
}
