//! cohdl-explorer — project the CoHDL compiler's `Checked{ir, world}` into
//! the versioned ExplorerModel JSON (PLAN.md S1).
//!
//! Usage: cohdl-explorer <project-dir> [-o out.json]
//!
//! Dependency resolution mirrors the CLI's RFC-029 path (validate → resolve
//! against the ../cohdl lib root → verify lock hashes) but never writes the
//! project's cohdl.lock — the extractor is strictly read-only.

#[cfg(target_os = "macos")]
mod app;
mod cli;
mod document_api;
mod document_registry;
mod document_types;
mod http_files;
mod model;
mod project_model;
mod serve;
mod view_config;
mod watch;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Finder launches the bundle's binary with no argv: that IS the app.
    #[cfg(target_os = "macos")]
    if args.is_empty() && app::running_from_bundle() {
        return match app::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(1)
            }
        };
    }
    let options = match cli::parse(args) {
        Ok(cli::Command::Help) => {
            print!("{}", cli::HELP);
            return ExitCode::SUCCESS;
        }
        Ok(cli::Command::Version) => {
            println!("cohdl-explorer {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Ok(cli::Command::Run(options)) => options,
        Err(e) => {
            eprintln!("error: {e}\n\n{}", cli::HELP);
            return ExitCode::from(2);
        }
    };
    let dir = options.project;
    if options.serve {
        let dist = options
            .dist
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../web/dist"));
        return match serve::serve(&dir, &dist, options.port, None) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(1)
            }
        };
    }
    match project_model::extract(&dir) {
        Ok(model) => {
            let json = serde_json::to_string_pretty(&model).expect("serialize");
            match options.out {
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
