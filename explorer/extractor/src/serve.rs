//! `--serve`: local HTTP + SSE live server (PLAN.md S3).
//!
//! Hand-rolled on std only (mirroring the compiler's zero-dependency
//! philosophy): a thread-per-connection HTTP listener, an mtime-polling
//! watcher thread, and Server-Sent Events for change notification.
//! Invalid edit states keep the last-known-good model; the error rides a
//! separate field so the frontend can overlay diagnostics.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

struct State {
    /// Last-known-good ExplorerModel JSON.
    model: String,
    /// Extraction error for the current (possibly invalid) source state.
    error: Option<String>,
}

fn allowed_roots(project: &Path) -> Vec<std::path::PathBuf> {
    let mut roots = vec![project.to_path_buf()];
    if let Ok(lib) = std::env::var("COHDL_LIB") {
        let p = std::path::PathBuf::from(lib);
        if let Some(parent) = p.parent() {
            roots.push(parent.to_path_buf()); // whole cohdl checkout (lib + examples)
        }
        roots.push(p);
    }
    roots
        .into_iter()
        .filter_map(|r| r.canonicalize().ok())
        .collect()
}

fn photo_for(project: &Path, mpn: &str) -> Option<std::path::PathBuf> {
    // sanitize: designators/MPNs are [A-Za-z0-9._-]
    if !mpn
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    {
        return None;
    }
    for ext in ["jpg", "png", "jpeg", "webp"] {
        let p = project.join("docs/photos").join(format!("{mpn}.{ext}"));
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

pub fn serve(project: &Path, dist: &Path, port: u16) -> Result<(), String> {
    let model = crate::project_model::extract(project)
        .map(|m| serde_json::to_string(&m).unwrap())
        .map_err(|e| format!("initial extract failed: {e}"))?;
    let state = Arc::new(Mutex::new(State { model, error: None }));
    let version = Arc::new(AtomicU64::new(1));

    // ---- watcher: poll source mtimes, re-extract on change
    {
        let state = Arc::clone(&state);
        let version = Arc::clone(&version);
        let project = project.to_path_buf();
        std::thread::spawn(move || {
            let mut last = scan_mtime(&project);
            loop {
                std::thread::sleep(Duration::from_millis(500));
                let now = scan_mtime(&project);
                if now != last {
                    last = now;
                    match crate::project_model::extract(&project) {
                        Ok(m) => {
                            let mut st = state.lock().unwrap();
                            st.model = serde_json::to_string(&m).unwrap();
                            st.error = None;
                        }
                        Err(e) => {
                            state.lock().unwrap().error = Some(e);
                        }
                    }
                    version.fetch_add(1, Ordering::SeqCst);
                    eprintln!(
                        "[watch] change detected -> version {}",
                        version.load(Ordering::SeqCst)
                    );
                }
            }
        });
    }

    let roots = allowed_roots(project);
    // Loopback only: /api/file serves datasheets/photos from the allow-listed
    // roots, which is fine to expose to the local user but not to the LAN.
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    eprintln!(
        "serving http://127.0.0.1:{port}/ (project: {})",
        project.display()
    );
    for conn in listener.incoming().flatten() {
        let state = Arc::clone(&state);
        let version = Arc::clone(&version);
        let dist = dist.to_path_buf();
        let roots = roots.clone();
        let project = project.to_path_buf();
        std::thread::spawn(move || {
            let _ = handle(conn, &state, &version, &dist, &roots, &project);
        });
    }
    Ok(())
}

fn scan_mtime(project: &Path) -> Vec<(PathBuf, SystemTime)> {
    let mut out = Vec::new();
    let mut stack = vec![project.to_path_buf()];
    while let Some(d) = stack.pop() {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if p.file_name().is_some_and(|n| n != "out" && n != "target") {
                        stack.push(p);
                    }
                } else if p.extension().is_some_and(|x| x == "cohdl")
                    || p.file_name().is_some_and(|n| n == "cohdl.toml")
                {
                    if let Ok(md) = p.metadata() {
                        out.push((p, md.modified().unwrap_or(SystemTime::UNIX_EPOCH)));
                    }
                }
            }
        }
    }
    out.sort();
    out
}

fn handle(
    mut conn: TcpStream,
    state: &Mutex<State>,
    version: &AtomicU64,
    dist: &Path,
    roots: &[std::path::PathBuf],
    project: &Path,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(conn.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
    // drain headers
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 || h == "\r\n" || h == "\n" {
            break;
        }
    }
    let full = path.clone();
    let path = path.split('?').next().unwrap_or("/");

    // /api/photo?mpn=<mpn>: part photo from <project>/docs/photos/<mpn>.<ext>
    if path == "/api/photo" {
        let mpn = percent_decode(full.split_once("?mpn=").map_or("", |x| x.1));
        return match photo_for(project, &mpn).and_then(|p| std::fs::read(&p).ok().map(|b| (p, b))) {
            Some((p, bytes)) => {
                let ct = match p.extension().and_then(|x| x.to_str()) {
                    Some("png") => "image/png",
                    Some("webp") => "image/webp",
                    _ => "image/jpeg",
                };
                respond(&mut conn, 200, ct, &bytes)
            }
            None => respond(&mut conn, 404, "text/plain", b"no photo"),
        };
    }

    // /api/file?p=<urlencoded abs path>: serve datasheets/photos from
    // allow-listed roots only (project dir + the cohdl checkout).
    if path == "/api/file" {
        let q = full.split_once("?p=").map_or("", |x| x.1).to_string();
        let decoded: String = percent_decode(&q);
        let ok_ext = decoded.rsplit('.').next().is_some_and(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "pdf" | "png" | "jpg" | "jpeg" | "webp" | "gif"
            )
        });
        let canon = std::path::Path::new(&decoded).canonicalize().ok();
        let allowed = canon
            .as_ref()
            .is_some_and(|c| roots.iter().any(|r| c.starts_with(r)));
        if !(ok_ext && allowed) {
            return respond(&mut conn, 404, "text/plain", b"forbidden");
        }
        let f = canon.unwrap();
        return match std::fs::read(&f) {
            Ok(bytes) => {
                let ct = match f
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|e| e.to_ascii_lowercase())
                {
                    Some(e) if e == "pdf" => "application/pdf",
                    Some(e) if e == "png" => "image/png",
                    Some(e) if e == "jpg" || e == "jpeg" => "image/jpeg",
                    Some(e) if e == "webp" => "image/webp",
                    Some(e) if e == "gif" => "image/gif",
                    _ => "application/octet-stream",
                };
                respond(&mut conn, 200, ct, &bytes)
            }
            Err(_) => respond(&mut conn, 404, "text/plain", b"not found"),
        };
    }

    match path {
        "/api/model" => {
            let st = state.lock().unwrap();
            let body = match &st.error {
                None => st.model.clone(),
                Some(e) => {
                    // last-known-good + error overlay field
                    let mut v: serde_json::Value = serde_json::from_str(&st.model).unwrap();
                    v["live_error"] = serde_json::Value::String(e.clone());
                    v.to_string()
                }
            };
            respond(&mut conn, 200, "application/json", body.as_bytes())
        }
        "/api/events" => {
            conn.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nAccess-Control-Allow-Origin: *\r\nConnection: keep-alive\r\n\r\n",
            )?;
            let mut seen = version.load(Ordering::SeqCst);
            conn.write_all(format!("data: {{\"version\":{seen}}}\n\n").as_bytes())?;
            loop {
                std::thread::sleep(Duration::from_millis(400));
                let now = version.load(Ordering::SeqCst);
                if now != seen {
                    seen = now;
                    if conn
                        .write_all(format!("data: {{\"version\":{now}}}\n\n").as_bytes())
                        .is_err()
                    {
                        return Ok(());
                    }
                } else if conn.write_all(b": keepalive\n\n").is_err() {
                    return Ok(());
                }
            }
        }
        _ => {
            // static files from dist/
            let rel = if path == "/" {
                "index.html"
            } else {
                &path[1..]
            };
            let f = dist.join(rel);
            let f = if f.is_file() {
                f
            } else {
                dist.join("index.html")
            };
            match std::fs::read(&f) {
                Ok(bytes) => {
                    let ct = match f.extension().and_then(|x| x.to_str()) {
                        Some("html") => "text/html; charset=utf-8",
                        Some("js") => "application/javascript",
                        Some("css") => "text/css",
                        Some("json") => "application/json",
                        Some("svg") => "image/svg+xml",
                        _ => "application/octet-stream",
                    };
                    respond(&mut conn, 200, ct, &bytes)
                }
                Err(_) => respond(&mut conn, 404, "text/plain", b"not found"),
            }
        }
    }
}

fn respond(conn: &mut TcpStream, code: u16, ct: &str, body: &[u8]) -> std::io::Result<()> {
    let status = if code == 200 {
        "200 OK"
    } else {
        "404 Not Found"
    };
    conn.write_all(
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .as_bytes(),
    )?;
    conn.write_all(body)
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(if b[i] == b'+' { b' ' } else { b[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

// keep Read import used on all platforms
#[allow(dead_code)]
fn _t(r: &mut dyn Read) {
    let _ = r;
}
