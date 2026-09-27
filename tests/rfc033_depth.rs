//! Crash regressions run in bounded subprocesses, including the parser's
//! ordinary test-thread stack. Never enlarge the stack to make a probe pass.
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};

const LIMIT: usize = 96;
const SHAPES: [&str; 10] = [
    "paren", "unary", "negative", "add", "mul", "mixed", "circuit", "layout", "both", "combined",
];
const TIMEOUT: Duration = Duration::from_secs(15);

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "cohdl-depth-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn command(binary: &Path) -> Command {
    #[cfg(unix)]
    {
        let mut c = Command::new("sh");
        c.args(["-c", "ulimit -c 0; exec \"$@\"", "depth-probe"])
            .arg(binary);
        c
    }
    #[cfg(not(unix))]
    {
        Command::new(binary)
    }
}

fn run(mut cmd: Command) -> Output {
    let tmp = Temp::new();
    let out = tmp.0.join("stdout");
    let err = tmp.0.join("stderr");
    cmd.stdout(std::fs::File::create(&out).unwrap())
        .stderr(std::fs::File::create(&err).unwrap());
    let mut child = Process(cmd.spawn().unwrap());
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(start.elapsed() < TIMEOUT, "subprocess timed out: {cmd:?}");
        std::thread::sleep(Duration::from_millis(10));
    };
    let output = Output {
        status,
        stdout: std::fs::read(out).unwrap(),
        stderr: std::fs::read(err).unwrap(),
    };
    assert!(
        status.code().is_some(),
        "subprocess died by signal: {cmd:?}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn source(shape: &str, depth: usize) -> String {
    let expr = match shape {
        "paren" => format!("{}1{}", "(".repeat(depth - 1), ")".repeat(depth - 1)),
        "unary" => format!("{}1", "+".repeat(depth - 1)),
        "negative" => format!("{}1", "- ".repeat(depth - 1)),
        "add" => vec!["1"; depth].join("+"),
        "mul" => vec!["1"; depth].join("*"),
        "mixed" => {
            let p = (depth - 1) / 3;
            let u = (depth - 1) / 3;
            let chain = depth - p - u;
            format!(
                "{}{}{}{}",
                "(".repeat(p),
                "+".repeat(u),
                vec!["1"; chain].join("+"),
                ")".repeat(p)
            )
        }
        "circuit" | "layout" | "both" | "combined" => {
            // Loop headers contain a leaf expression, so N loops have depth N+1.
            let loops = if shape == "combined" {
                (depth - 1) / 2
            } else {
                depth - 1
            };
            let mut body = String::new();
            for i in 0..loops {
                if shape == "both" && i == loops / 2 {
                    body.push_str("layout { ");
                }
                body.push_str(&format!("for l{i}: i{i} in 0..0 {{ "));
            }
            if shape == "combined" {
                body.push_str(&format!(
                    "const N: Int = {}1{}",
                    "(".repeat(depth - loops - 1),
                    ")".repeat(depth - loops - 1)
                ));
            }
            body.push_str(&"}".repeat(loops));
            if shape == "both" {
                body.push('}');
            }
            return if shape == "layout" {
                format!("design B {{ layout {{ {body} }} }}")
            } else {
                format!("design B {{ {body} }}")
            };
        }
        _ => panic!("unknown shape"),
    };
    format!("design B {{ const N: Int = {expr} }}")
}

// Invoked only as a subprocess by parser_boundaries. A crash cannot abort the
// parent suite; this also exercises bounded clone/drop on the default stack.
#[test]
fn parser_probe() {
    let Ok(path) = std::env::var("COHDL_DEPTH_PROBE") else {
        return;
    };
    let text = std::fs::read_to_string(path).unwrap();
    let mut diags = cohdl::diag::Diagnostics::new();
    let tokens = cohdl::lex::lex(cohdl::span::FileId(0), &text, &mut diags);
    let ast = cohdl::parse::parse(tokens, &mut diags);
    if std::env::var("COHDL_DEPTH_REJECT").unwrap() == "yes" {
        let ds: Vec<_> = diags.iter().collect();
        assert_eq!(ds.len(), 1, "{diags:?}");
        assert_eq!(ds[0].code, "E102");
        assert_eq!(
            ds[0].message,
            format!("syntax/AST depth limit of {LIMIT} exceeded")
        );
        let span = ds[0].primary.span;
        assert!(span.end > span.start && span.end as usize <= text.len());
        if let Ok(offset) = std::env::var("COHDL_DEPTH_OFFSET") {
            let offset: u32 = offset.parse().unwrap();
            assert_eq!(
                span,
                cohdl::span::Span::new(cohdl::span::FileId(0), offset, offset + 1)
            );
        }
        assert!(
            ast.items.is_empty(),
            "no partial failing file reaches consumers"
        );
    } else {
        assert!(diags.is_empty(), "{diags:?}");
        assert_eq!(ast.items.len(), text.matches("design B").count());
        drop(ast.clone());
        if ast.items.len() == 1 {
            let checked = cohdl::pipeline::check_files_in(
                "depth",
                &[("src/main.cohdl".to_string(), text)],
                None,
            )
            .unwrap();
            assert!(
                !checked.diags.has_errors(),
                "{}",
                checked.diags.render(&checked.sm)
            );
        }
    }
}

#[test]
fn parser_boundaries() {
    let tmp = Temp::new();
    let path = tmp.0.join("probe.cohdl");
    for shape in SHAPES {
        for depth in [LIMIT - 1, LIMIT, LIMIT + 1] {
            eprintln!("parser boundary: {shape} depth {depth}");
            std::fs::write(&path, source(shape, depth)).unwrap();
            let mut cmd = command(&std::env::current_exe().unwrap());
            cmd.args(["--exact", "parser_probe", "--nocapture"])
                .env("COHDL_DEPTH_PROBE", &path)
                .env(
                    "COHDL_DEPTH_REJECT",
                    if depth > LIMIT { "yes" } else { "no" },
                );
            if depth > LIMIT {
                let text = source(shape, depth);
                let expr_start = "design B { const N: Int = ".len();
                let offset = match shape {
                    "paren" | "unary" => expr_start + LIMIT,
                    "negative" => expr_start + 2 * LIMIT,
                    "add" | "mul" => expr_start + 2 * LIMIT - 1,
                    "circuit" | "layout" | "both" => text.rfind("in ").unwrap() + 3,
                    "combined" => text.rfind("= ").unwrap() + 2 + (depth - 1) / 2,
                    "mixed" => text.rfind("+1").unwrap(),
                    _ => unreachable!(),
                };
                cmd.env("COHDL_DEPTH_OFFSET", offset.to_string());
            }
            let out = run(cmd);
            assert!(
                out.status.success(),
                "{shape} depth {depth}:\n{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}

#[test]
fn recovery_and_sibling_budgets_are_bounded() {
    let tmp = Temp::new();
    let path = tmp.0.join("probe.cohdl");
    let balanced = (0..6).fold("1".to_string(), |e, _| format!("({e}+{e})"));
    for (text, reject) in [
        (source("paren", 7001).replace(')', ""), true),
        (source("circuit", 7001).replace('}', ""), true),
        (source("layout", 7001).replace('}', ""), true),
        (
            format!(
                "{} {}",
                source("combined", LIMIT),
                source("combined", LIMIT)
            ),
            false,
        ),
        (format!("design B {{ const N: Int = {balanced} }}"), false),
    ] {
        std::fs::write(&path, text).unwrap();
        let mut cmd = command(&std::env::current_exe().unwrap());
        cmd.args(["--exact", "parser_probe", "--nocapture"])
            .env("COHDL_DEPTH_PROBE", &path)
            .env("COHDL_DEPTH_REJECT", if reject { "yes" } else { "no" });
        let out = run(cmd);
        assert!(
            out.status.success(),
            "{} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

fn project(path: &Path, text: &str) {
    std::fs::create_dir_all(path.join("src")).unwrap();
    let repo_std = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std");
    let (_, manifest) = cohdl::project::peek_manifest(&repo_std).unwrap();
    let version = manifest.version.unwrap();
    let dep = path.join("deps/std").join(&version);
    std::fs::create_dir_all(dep.join("src")).unwrap();
    std::fs::copy(repo_std.join("cohdl.toml"), dep.join("cohdl.toml")).unwrap();
    for entry in std::fs::read_dir(repo_std.join("src")).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_some_and(|e| e == "cohdl") {
            std::fs::copy(&p, dep.join("src").join(p.file_name().unwrap())).unwrap();
        }
    }
    std::fs::write(path.join("cohdl.toml"), format!("[package]\nname = \"depth\"\nversion = \"0.1.0\"\n[dependencies]\nstd = \"{version}\"\n")).unwrap();
    std::fs::write(path.join("src/main.cohdl"), text).unwrap();
}

#[test]
fn cli_deep_inputs_are_diagnostics_not_crashes() {
    let tmp = Temp::new();
    for shape in SHAPES {
        let depth = if shape == "add" { 13000 } else { 7001 };
        let text = source(shape, depth);
        project(&tmp.0, &text);
        for verb in ["check", "fmt", "docs"] {
            let mut cmd = command(Path::new(env!("CARGO_BIN_EXE_cohdl")));
            cmd.arg(verb).arg(&tmp.0);
            if verb == "check" {
                cmd.arg("--no-std");
            }
            let artifact = tmp.0.join("api.json");
            if verb == "docs" {
                cmd.arg("--out").arg(&artifact);
            }
            let out = run(cmd);
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert_eq!(out.status.code(), Some(1), "{verb}/{shape}: {stderr}");
            assert_eq!(
                stderr.matches("error[E102]").count(),
                1,
                "{verb}/{shape}: {stderr}"
            );
            assert_eq!(stderr.matches("error[").count(), 1, "{stderr}");
            assert!(
                stderr.contains(&format!("depth limit of {LIMIT}")),
                "{stderr}"
            );
            assert!(
                !stderr.contains("panic") && !stderr.contains("overflow"),
                "{stderr}"
            );
            assert!(!artifact.exists() && !tmp.0.join("out").exists());
            assert_eq!(
                std::fs::read_to_string(tmp.0.join("src/main.cohdl")).unwrap(),
                text
            );
        }
    }
}

#[test]
fn cli_at_limit_still_checks_formats_and_documents() {
    let tmp = Temp::new();
    for shape in SHAPES {
        for verb in ["check", "fmt", "docs"] {
            project(&tmp.0, &source(shape, LIMIT));
            let mut cmd = command(Path::new(env!("CARGO_BIN_EXE_cohdl")));
            cmd.arg(verb).arg(&tmp.0);
            if verb == "check" {
                cmd.arg("--no-std");
            }
            let out = run(cmd);
            assert!(
                out.status.success(),
                "{verb}/{shape}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            if verb == "docs" {
                serde_json::from_slice::<Value>(&out.stdout).unwrap();
            }
        }
    }
}

fn send(input: &mut impl Write, message: Value) {
    let body = message.to_string();
    write!(input, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
    input.flush().unwrap();
}

#[test]
fn lsp_survives_deep_open_and_normal_change() {
    if std::env::var_os("COHDL_LSP_DEPTH_PROBE").is_none() {
        let mut cmd = command(&std::env::current_exe().unwrap());
        cmd.args([
            "--exact",
            "lsp_survives_deep_open_and_normal_change",
            "--nocapture",
        ])
        .env("COHDL_LSP_DEPTH_PROBE", "1");
        let out = run(cmd);
        assert!(
            out.status.success(),
            "{} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        return;
    }
    let tmp = Temp::new();
    let file = tmp.0.join("main.cohdl");
    std::fs::write(&file, "design B {}").unwrap();
    let uri = format!("file://{}", file.canonicalize().unwrap().display());
    let mut cmd = command(Path::new(env!("CARGO_BIN_EXE_cohdl")));
    cmd.arg("lsp")
        .env(
            "COHDL_STD",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std"),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = Process(cmd.spawn().unwrap());
    let mut input = child.0.stdin.take().unwrap();
    let mut output = BufReader::new(child.0.stdout.take().unwrap());
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || loop {
        let mut len = 0;
        loop {
            let mut line = String::new();
            if output.read_line(&mut line).unwrap_or(0) == 0 {
                return;
            }
            if line.trim().is_empty() {
                break;
            }
            if let Some(n) = line.strip_prefix("Content-Length:") {
                len = n.trim().parse().unwrap();
            }
        }
        let mut bytes = vec![0; len];
        if output.read_exact(&mut bytes).is_err() {
            return;
        }
        if tx
            .send(serde_json::from_slice::<Value>(&bytes).unwrap())
            .is_err()
        {
            return;
        }
    });
    send(
        &mut input,
        json!({"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{"capabilities":{}}}),
    );
    assert_eq!(rx.recv_timeout(TIMEOUT).unwrap()["id"], 1);
    send(
        &mut input,
        json!({"jsonrpc":"2.0", "method":"initialized", "params":{}}),
    );
    for (i, shape) in SHAPES.iter().enumerate() {
        let version = i * 2 + 1;
        let text = source(shape, if *shape == "add" { 13000 } else { 7001 });
        if i == 0 {
            send(
                &mut input,
                json!({"jsonrpc":"2.0", "method":"textDocument/didOpen", "params":{"textDocument":{"uri":uri,"languageId":"cohdl","version":version,"text":text}}}),
            );
        } else {
            send(
                &mut input,
                json!({"jsonrpc":"2.0", "method":"textDocument/didChange", "params":{"textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":text}]}}),
            );
        }
        let msg = rx
            .recv_timeout(TIMEOUT)
            .expect("deep document must get a bounded response");
        assert_eq!(msg["method"], "textDocument/publishDiagnostics");
        assert_eq!(msg["params"]["uri"], uri);
        let ds = msg["params"]["diagnostics"].as_array().unwrap();
        let errors: Vec<_> = ds.iter().filter(|d| d["severity"] == 1).collect();
        assert_eq!(errors.len(), 1, "{msg}");
        assert_eq!(errors[0]["code"], "E102");
        let range = &errors[0]["range"];
        assert_eq!(range["start"]["line"], 0);
        assert_eq!(range["end"]["line"], 0);
        assert!(
            range["end"]["character"].as_u64().unwrap()
                > range["start"]["character"].as_u64().unwrap()
        );
        assert!(errors[0]["message"]
            .as_str()
            .unwrap()
            .contains(&format!("depth limit of {LIMIT}")));
        send(
            &mut input,
            json!({"jsonrpc":"2.0", "method":"textDocument/didChange", "params":{"textDocument":{"uri":uri,"version":version+1},"contentChanges":[{"text":source(shape, LIMIT)}]}}),
        );
        let msg = rx
            .recv_timeout(TIMEOUT)
            .expect("server must survive and check the next version");
        assert_eq!(msg["params"]["uri"], uri);

        assert!(
            msg["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .all(|d| d["severity"] != 1),
            "{msg}"
        );
        // The existing server omits the optional diagnostic `version`. Keep
        // exactly one change outstanding and round-trip a request tagged with
        // that version before sending another change (no stale clear can pass).
        let id = 100 + version + 1;
        send(
            &mut input,
            json!({"jsonrpc":"2.0", "id":id, "method":"depth/barrier", "params":{}}),
        );
        assert_eq!(rx.recv_timeout(TIMEOUT).unwrap()["id"], id);
    }
    send(
        &mut input,
        json!({"jsonrpc":"2.0", "id":2, "method":"shutdown", "params":null}),
    );
    assert_eq!(rx.recv_timeout(TIMEOUT).unwrap()["id"], 2);
    send(
        &mut input,
        json!({"jsonrpc":"2.0", "method":"exit", "params":null}),
    );
    let start = Instant::now();
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        assert!(start.elapsed() < TIMEOUT, "LSP shutdown timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
    reader.join().unwrap();
}
