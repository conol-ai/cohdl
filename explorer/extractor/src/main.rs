//! cohdl-explorer — project the CoHDL compiler's `Checked{ir, world}` into
//! the versioned ExplorerModel JSON (PLAN.md S1).
//!
//! Usage: cohdl-explorer <project-dir> [-o out.json]
//!
//! Dependency resolution mirrors the CLI's RFC-029 path (validate → resolve
//! against the ../cohdl lib root → verify lock hashes) but never writes the
//! project's cohdl.lock — the extractor is strictly read-only.

mod model;
mod project_model;
mod serve;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut dir: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut do_serve = false;
    let mut port: u16 = 5199;
    let mut dist: Option<PathBuf> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "-o" | "--out" => out = args.next().map(PathBuf::from),
            "--serve" => do_serve = true,
            "--port" => port = args.next().and_then(|p| p.parse().ok()).unwrap_or(5199),
            "--dist" => dist = args.next().map(PathBuf::from),
            _ => dir = Some(PathBuf::from(a)),
        }
    }
    let Some(dir) = dir else {
        eprintln!("usage: cohdl-explorer <project-dir> [-o out.json] [--serve [--port N] [--dist DIR]]");
        return ExitCode::from(2);
    };
    if do_serve {
        let dist = dist.unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../web/dist")
        });
        return match serve::serve(&dir, &dist, port) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => { eprintln!("error: {e}"); ExitCode::from(1) }
        };
    }
    match project_model::extract(&dir) {
        Ok(model) => {
            let json = serde_json::to_string_pretty(&model).expect("serialize");
            match out {
                Some(p) => {
                    if let Err(e) = std::fs::write(&p, json) {
                        eprintln!("write {}: {e}", p.display());
                        return ExitCode::from(2);
                    }
                    eprintln!("wrote {}", p.display());
                }
                None => println!("{json}"),
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
