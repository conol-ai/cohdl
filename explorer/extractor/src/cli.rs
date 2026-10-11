use std::path::PathBuf;

pub const HELP: &str = "cohdl-explorer - inspect a checked CoHDL project

Usage: cohdl-explorer <project-dir> [-o FILE]
       cohdl-explorer <project-dir> --serve [--port N] [--dist DIR]

Options:
  -h, --help       Show this help
  --version        Show version
  -o, --out FILE   Write extracted model JSON (default: stdout)
  --serve          Serve the model, project views and web assets on loopback
  --port N         HTTP port, 1..65535 (default: 5199)
  --dist DIR       Web assets (default: ../web/dist beside extractor)
  --               End options; the remaining argument is the project path
";

pub enum Command {
    Help,
    Version,
    Run(Options),
}

pub struct Options {
    pub project: PathBuf,
    pub out: Option<PathBuf>,
    pub serve: bool,
    pub port: u16,
    pub dist: Option<PathBuf>,
}

fn value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .filter(|v| !v.is_empty() && !v.starts_with('-'))
        .ok_or_else(|| format!("{flag} requires a value"))
}

pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let mut project = None;
    let mut out = None;
    let mut serve = false;
    let mut port = None;
    let mut dist = None;
    let mut positional = false;
    while let Some(arg) = args.next() {
        if positional || !arg.starts_with('-') {
            if project.replace(PathBuf::from(&arg)).is_some() {
                return Err("expected one project path".into());
            }
            continue;
        }
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "--version" => return Ok(Command::Version),
            "--" => positional = true,
            "--serve" if !serve => serve = true,
            "-o" | "--out" if out.is_none() => out = Some(PathBuf::from(value(&mut args, &arg)?)),
            "--dist" if dist.is_none() => dist = Some(PathBuf::from(value(&mut args, &arg)?)),
            "--port" if port.is_none() => {
                let parsed = value(&mut args, &arg)?
                    .parse::<u16>()
                    .ok()
                    .filter(|p| *p != 0)
                    .ok_or("--port must be an integer from 1 to 65535")?;
                port = Some(parsed);
            }
            _ => return Err(format!("unknown or repeated option: {arg}")),
        }
    }
    let project = project.ok_or("a project path is required")?;
    if !serve && (port.is_some() || dist.is_some()) {
        return Err("--port and --dist require --serve".into());
    }
    if serve && out.is_some() {
        return Err("--out cannot be combined with --serve".into());
    }
    Ok(Command::Run(Options {
        project,
        out,
        serve,
        port: port.unwrap_or(5199),
        dist,
    }))
}
