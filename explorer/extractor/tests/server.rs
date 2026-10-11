use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static NEXT_SERVER: AtomicU64 = AtomicU64::new(0);

struct Server {
    child: Child,
    root: PathBuf,
    port: u16,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

impl Server {
    fn start() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_SERVER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("explorer-http-{}-{stamp}-{id}", std::process::id()));
        for dir in ["project/src", "project/views", "dist/views", "dist/assets"] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        std::fs::write(
            root.join("project/cohdl.toml"),
            include_str!("fixtures/parameterized/cohdl.toml"),
        )
        .unwrap();
        std::fs::write(
            root.join("project/src/main.cohdl"),
            include_str!("fixtures/parameterized/src/main.cohdl"),
        )
        .unwrap();
        std::fs::write(root.join("dist/index.html"), "<html>Explorer</html>").unwrap();
        std::fs::write(root.join("dist/assets/中文.js"), "unicode asset").unwrap();
        std::fs::write(root.join("secret.json"), "private").unwrap();
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let child = Command::new(env!("CARGO_BIN_EXE_cohdl-explorer"))
            .arg(root.join("project"))
            .arg("--serve")
            .arg("--port")
            .arg(port.to_string())
            .arg("--dist")
            .arg(root.join("dist"))
            .stdout(Stdio::null())
            .stderr(std::fs::File::create(root.join("server.log")).unwrap())
            .spawn()
            .unwrap();
        let server = Self { child, root, port };
        let deadline = Instant::now() + Duration::from_secs(10);
        while TcpStream::connect(("127.0.0.1", port)).is_err() {
            assert!(Instant::now() < deadline, "HTTP server startup timed out");
            std::thread::sleep(Duration::from_millis(20));
        }
        server
    }

    fn request(&self, path: &str) -> (u16, String, String) {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        stream
            .write_all(
                format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                    .as_bytes(),
            )
            .unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let mut reader = BufReader::new(stream);
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap_or_else(|error| {
                panic!(
                    "HTTP {path}: {error}; server log: {}",
                    std::fs::read_to_string(self.root.join("server.log")).unwrap_or_default()
                )
            });
            if line == "\r\n" {
                break;
            }
            assert!(!line.is_empty(), "Incomplete response headers");
            headers.push_str(&line);
        }
        let length = headers
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .unwrap()
            .parse()
            .unwrap();
        let mut body = vec![0; length];
        reader.read_exact(&mut body).unwrap();
        let code = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
        (code, headers, String::from_utf8(body).unwrap())
    }

    fn config(&self, location: &str, value: &Value) {
        std::fs::write(
            self.root.join(location).join("views/Board.view.json"),
            value.to_string(),
        )
        .unwrap();
    }

    fn events(&self) -> BufReader<TcpStream> {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(4)))
            .unwrap();
        write!(
            stream,
            "GET /api/events HTTP/1.1\r\nHost: localhost\r\n\r\n"
        )
        .unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        while reader.read_line(&mut line).unwrap() != 0 {
            if line == "\r\n" {
                break;
            }
            line.clear();
        }
        reader
    }
}

fn config(name: &str) -> Value {
    json!({"schema_version":1,"design":"Board","views":[{"name":name,"regions":[{"name":"Core","members":["R1","path:Board::source","agg:GND + V3V3","agg:host:Board::source"]}]}]})
}

fn next_version(reader: &mut impl BufRead) -> u64 {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        let mut line = String::new();
        assert!(reader.read_line(&mut line).unwrap() > 0);
        if let Some(data) = line.strip_prefix("data: ") {
            return serde_json::from_str::<Value>(data).unwrap()["version"]
                .as_u64()
                .unwrap();
        }
        assert!(Instant::now() < deadline, "no SSE version event");
    }
}

#[test]
fn cli_help_version_and_errors_are_explicit() {
    for flag in ["--help", "-h", "--version"] {
        let out = Command::new(env!("CARGO_BIN_EXE_cohdl-explorer"))
            .arg(flag)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{flag}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(String::from_utf8_lossy(&out.stdout).contains("cohdl-explorer"));
    }
    for args in [
        vec!["--unknown"],
        vec!["project", "another"],
        vec!["project", "--port"],
        vec!["project", "--port", "oops"],
        vec!["project", "--port", "65536"],
        vec!["project", "--port", "0"],
        vec!["project", "--dist"],
        vec!["project", "-o"],
        vec!["project", "--out", "--serve"],
        vec!["project", "--port", "5211"],
        vec!["project", "--serve", "--out", "model.json"],
        vec!["project", "--serve", "--serve"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_cohdl-explorer"))
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("error:"),
            "{args:?}"
        );
    }
}

#[test]
fn project_views_override_bundle_and_invalid_configs_never_fall_back() {
    let server = Server::start();
    assert_eq!(server.request("/views/Board.view.json").0, 404);
    server.config("dist", &config("Bundled"));
    assert_eq!(
        serde_json::from_str::<Value>(&server.request("/views/Board.view.json").2).unwrap(),
        config("Bundled")
    );
    server.config("project", &config("Project"));
    assert_eq!(
        serde_json::from_str::<Value>(&server.request("/views/Board.view.json").2).unwrap(),
        config("Project")
    );
    let mut invalid = vec![json!({}), config(" "), config(" Padded ")];
    for (key, value) in [
        ("schema_version", json!(2)),
        ("design", json!("OtherBoard")),
        ("views", json!({})),
    ] {
        let mut v = config("Blocks");
        v[key] = value;
        invalid.push(v);
    }
    for name in ["", " Other ", "Other", "other", " Core "] {
        let mut v = config("Blocks");
        v["views"][0]["regions"][0]["name"] = json!(name);
        invalid.push(v);
    }
    for member in [
        json!(""),
        json!(" R1"),
        json!(3),
        json!("path:"),
        json!("agg:"),
        json!("agg:host:"),
        json!("unknown:R1"),
    ] {
        let mut v = config("Blocks");
        v["views"][0]["regions"][0]["members"] = json!([member]);
        invalid.push(v);
    }
    let mut v = config("Blocks");
    let view = v["views"][0].clone();
    v["views"].as_array_mut().unwrap().push(view);
    invalid.push(v);
    let mut v = config("Blocks");
    let r = v["views"][0]["regions"][0].clone();
    v["views"][0]["regions"].as_array_mut().unwrap().push(r);
    invalid.push(v);
    for value in invalid {
        server.config("project", &value);
        let response = server.request("/views/Board.view.json");
        assert_eq!(response.0, 422, "{value}: {response:?}");
        assert!(response.1.contains("application/json"));
        assert!(serde_json::from_str::<Value>(&response.2).unwrap()["error"].is_string());
    }
    std::fs::write(server.root.join("project/views/Board.view.json"), "{").unwrap();
    assert_eq!(server.request("/views/Board.view.json").0, 422);
    std::fs::remove_file(server.root.join("project/views/Board.view.json")).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&server.request("/views/Board.view.json").2).unwrap(),
        config("Bundled")
    );
}

#[test]
fn static_and_config_paths_are_confined_and_unicode_safe() {
    let server = Server::start();
    assert_eq!(server.request("/").0, 200);
    assert_eq!(
        server.request("/assets/%E4%B8%AD%E6%96%87.js").2,
        "unicode asset"
    );
    assert_eq!(server.request("/assets/中文.js").2, "unicode asset");
    for path in [
        "/api/missing",
        "/assets/missing.js",
        "/missing",
        "/../secret.json",
        "/%2e%2e/secret.json",
        "/views/%2e%2e/secret.json",
        "/views/%2f..%2fsecret.json",
        "/assets/%中文",
        "/assets/%FF",
        "/assets/%00",
        "/api/file?p=%中文",
        "/api/photo?mpn=%中文",
        "/%252e%252e/secret.json",
    ] {
        assert_eq!(server.request(path).0, 404, "{path}");
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            server.root.join("secret.json"),
            server.root.join("dist/assets/secret.json"),
        )
        .unwrap();
        std::os::unix::fs::symlink(
            server.root.join("secret.json"),
            server.root.join("project/views/Board.view.json"),
        )
        .unwrap();
        assert_eq!(server.request("/assets/secret.json").0, 404);
        assert_eq!(server.request("/views/Board.view.json").0, 404);
    }
}

#[test]
fn view_add_change_delete_emit_sse_without_changing_model() {
    let server = Server::start();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        server.root.join("project"),
        server.root.join("project/src/cycle"),
    )
    .unwrap();
    let model = server.request("/api/model").2;
    assert_eq!(
        serde_json::from_str::<Value>(&model).unwrap()["verdict"],
        "pass"
    );
    let mut events = server.events();
    let mut version = next_version(&mut events);
    // Wait for the watcher snapshot before the first edit.
    std::thread::sleep(Duration::from_millis(600));
    for name in [Some("Added"), Some("Changed"), None] {
        if let Some(name) = name {
            server.config("project", &config(name));
        } else {
            std::fs::remove_file(server.root.join("project/views/Board.view.json")).unwrap();
        }
        let next = next_version(&mut events);
        assert!(next > version);
        version = next;
        assert!(
            server.request("/api/model").2 == model,
            "view edit changed model bytes"
        );
        let response = server.request("/views/Board.view.json");
        if let Some(name) = name {
            assert_eq!(
                serde_json::from_str::<Value>(&response.2).unwrap(),
                config(name)
            );
        } else {
            assert_eq!(response.0, 404);
        }
    }
    server.config("dist", &config("Bundled"));
    assert!(next_version(&mut events) > version);
    assert!(
        server.request("/api/model").2 == model,
        "bundled edit changed model bytes"
    );
}

#[path = "server/documents.rs"]
mod document_tests;

#[path = "server/proposals.rs"]
mod proposal_tests;
