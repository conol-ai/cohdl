//! `--serve`: local HTTP + SSE live server (PLAN.md S3).
//!
//! Hand-rolled on std only (mirroring the compiler's zero-dependency
//! philosophy): a thread-per-connection HTTP listener, an mtime-polling
//! watcher thread, and Server-Sent Events for change notification.
//! Invalid edit states keep the last-known-good model; the error rides a
//! separate field so the frontend can overlay diagnostics.

use crate::http_files::{confined_file, content_type, percent_decode, view_file};
use crate::watch::{scan_mtime, scan_views};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct State {
    /// Last-known-good ExplorerModel JSON.
    model: String,
    /// Extraction error for the current (possibly invalid) source state.
    error: Option<String>,
}

impl State {
    fn update(&mut self, extraction: Result<crate::model::ExplorerModel, String>) {
        let model = match extraction {
            Ok(model) => model,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        let previous: serde_json::Value = serde_json::from_str(&self.model).unwrap();
        if model.verdict != "pass" && previous["verdict"] == "pass" {
            self.error = Some(live_diagnostics(&model));
            return;
        }
        self.model = serde_json::to_string(&model).unwrap();
        self.error = None;
    }
}

fn live_diagnostics(model: &crate::model::ExplorerModel) -> String {
    model
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let location = diagnostic.span.as_ref().map_or_else(String::new, |span| {
                format!("{}:{}:{}: ", span.file, span.line, span.col)
            });
            format!(
                "{}[{}] {location}{}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

const MAX_PERSISTENT_VIEWERS: i64 = 5;

/// Persistent SSE reservations also keep the app bundle alive. Overflow
/// clients reconnect with short responses until a reservation becomes free.
#[derive(Default)]
struct Viewers {
    active: AtomicI64,
    ever: AtomicBool,
    last_drop: Mutex<Option<Instant>>,
}

struct ViewerGuard(Arc<Viewers>);

impl ViewerGuard {
    fn new(v: &Arc<Viewers>) -> Option<Self> {
        v.active
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |active| {
                (active < MAX_PERSISTENT_VIEWERS).then(|| active + 1)
            })
            .ok()?;
        v.ever.store(true, Ordering::SeqCst);
        Some(Self(Arc::clone(v)))
    }
}

impl Drop for ViewerGuard {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::SeqCst);
        *self.0.last_drop.lock().unwrap() = Some(Instant::now());
    }
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
        if let Some(p) = confined_file(project, &format!("docs/photos/{mpn}.{ext}")) {
            return Some(p);
        }
    }
    None
}

pub fn serve(
    project: &Path,
    dist: &Path,
    port: u16,
    idle_exit: Option<Duration>,
) -> Result<(), String> {
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
        let dist = dist.to_path_buf();
        let mut last = scan_mtime(&project);
        let mut views = scan_views(&project, &dist);
        let mut documents = crate::document_api::watch_snapshot(&project);
        let mut proposals = crate::proposal_files::watch_snapshot(&project);
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(500));
            let now = scan_mtime(&project);
            let source_changed = now != last;
            if source_changed {
                last = now;
                let extraction = crate::project_model::extract(&project);
                state.lock().unwrap().update(extraction);
            }
            let current_views = scan_views(&project, &dist);
            let views_changed = current_views != views;
            views = current_views;
            let current_documents = crate::document_api::watch_snapshot(&project);
            let documents_changed = current_documents != documents;
            documents = current_documents;
            let current_proposals = crate::proposal_files::watch_snapshot(&project);
            let proposals_changed = current_proposals != proposals;
            proposals = current_proposals;
            if source_changed || views_changed || documents_changed || proposals_changed {
                version.fetch_add(1, Ordering::SeqCst);
                eprintln!(
                    "[watch] change detected -> version {}",
                    version.load(Ordering::SeqCst)
                );
            }
        });
    }

    let viewers = Arc::new(Viewers::default());
    // ---- idle exit (app bundle): retire once the last tab has been gone a
    // while, or if no tab ever arrived (the browser open failed silently).
    if let Some(idle) = idle_exit {
        let viewers = Arc::clone(&viewers);
        let started = Instant::now();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(15));
            if viewers.active.load(Ordering::SeqCst) > 0 {
                continue;
            }
            if viewers.ever.load(Ordering::SeqCst) {
                let quiet = viewers
                    .last_drop
                    .lock()
                    .unwrap()
                    .map(|t| t.elapsed())
                    .unwrap_or_default();
                if quiet > idle {
                    eprintln!("[idle] last viewer left {}s ago — exiting", quiet.as_secs());
                    std::process::exit(0);
                }
            } else if started.elapsed() > 3 * idle {
                eprintln!("[idle] no viewer ever connected — exiting");
                std::process::exit(0);
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
        let viewers = Arc::clone(&viewers);
        std::thread::spawn(move || {
            let _ = handle(conn, &state, &version, &dist, &roots, &project, &viewers);
        });
    }
    Ok(())
}

fn handle(
    mut conn: TcpStream,
    state: &Mutex<State>,
    version: &AtomicU64,
    dist: &Path,
    roots: &[std::path::PathBuf],
    project: &Path,
    viewers: &Arc<Viewers>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(conn.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let method = line.split_whitespace().next().unwrap_or("").to_owned();
    let path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
    // drain headers
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 || h == "\r\n" || h == "\n" {
            break;
        }
    }
    let full = path.clone();
    let Some(path) = percent_decode(path.split('?').next().unwrap_or("/"), false) else {
        return respond(&mut conn, 404, "text/plain", b"not found");
    };
    let path = path.as_str();

    if (path == "/api/proposals" || path.starts_with("/api/proposals/")) && method != "GET" {
        return respond(
            &mut conn,
            404,
            "application/json",
            b"{\"error\":\"Read-only proposal endpoint\"}",
        );
    }
    if path == "/api/proposals" || path.starts_with("/api/proposals/") {
        let model = state.lock().unwrap().model.clone();
        if let Some(response) = crate::proposal_api::handle(path, project, &model) {
            return respond(
                &mut conn,
                response.code,
                response.content_type,
                &response.body,
            );
        }
    }

    if path == "/api/source" || path == "/api/documents" || path.starts_with("/api/documents/") {
        let model = state.lock().unwrap().model.clone();
        if let Some(response) = crate::document_api::handle(path, &full, project, dist, &model) {
            return respond(
                &mut conn,
                response.code,
                response.content_type,
                &response.body,
            );
        }
    }

    // /api/photo?mpn=<mpn>: part photo from <project>/docs/photos/<mpn>.<ext>
    if path == "/api/photo" {
        let Some(mpn) = percent_decode(full.split_once("?mpn=").map_or("", |x| x.1), true) else {
            return respond(&mut conn, 404, "text/plain", b"no photo");
        };
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
        let Some(decoded) = percent_decode(&q, true) else {
            return respond(&mut conn, 404, "text/plain", b"not found");
        };
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
            Ok(bytes) => respond(&mut conn, 200, content_type(&f), &bytes),
            Err(_) => respond(&mut conn, 404, "text/plain", b"not found"),
        };
    }

    if path.starts_with("/views/") {
        return serve_view(&mut conn, project, dist, &path[1..]);
    }
    match path {
        "/api/project" if method == "GET" => {
            let Ok(root) = project.canonicalize() else {
                return respond(
                    &mut conn,
                    404,
                    "application/json",
                    b"{\"error\":\"Project identity unavailable\"}",
                );
            };
            let model: serde_json::Value =
                serde_json::from_str(&state.lock().unwrap().model).unwrap();
            let body = serde_json::json!({
                "schema": 1,
                "id": cohdl::hash::sha256_hex(root.as_os_str().as_encoded_bytes()),
                "design": model["design"],
            })
            .to_string();
            respond(&mut conn, 200, "application/json", body.as_bytes())
        }
        "/api/project" => respond(
            &mut conn,
            404,
            "application/json",
            b"{\"error\":\"Read-only project endpoint\"}",
        ),
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
            let Some(_viewer) = ViewerGuard::new(viewers) else {
                let version = version.load(Ordering::SeqCst);
                let body = format!("retry: 1000\ndata: {{\"version\":{version}}}\n\n");
                return respond(&mut conn, 200, "text/event-stream", body.as_bytes());
            };
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
            if path.starts_with("/api/") {
                return respond(
                    &mut conn,
                    404,
                    "application/json",
                    b"{\"error\":\"unknown API endpoint\"}",
                );
            }
            let rel = if path == "/" {
                "index.html"
            } else {
                path.strip_prefix('/').unwrap_or("")
            };
            let Some(f) = confined_file(dist, rel) else {
                return respond(&mut conn, 404, "text/plain", b"not found");
            };
            match std::fs::read(&f) {
                Ok(bytes) => respond(&mut conn, 200, content_type(&f), &bytes),
                Err(_) => respond(&mut conn, 404, "text/plain", b"not found"),
            }
        }
    }
}

fn serve_view(
    conn: &mut TcpStream,
    project: &Path,
    dist: &Path,
    relative: &str,
) -> std::io::Result<()> {
    let design = relative
        .strip_prefix("views/")
        .and_then(|p| p.strip_suffix(".view.json"))
        .filter(|d| !d.is_empty() && !d.contains(['/', '\\']));
    let Some((design, file)) =
        design.and_then(|d| view_file(project, dist, relative).map(|p| (d, p)))
    else {
        return respond(
            conn,
            404,
            "application/json",
            b"{\"error\":\"view config not found\"}",
        );
    };
    let Ok(bytes) = std::fs::read(file) else {
        return respond(
            conn,
            404,
            "application/json",
            b"{\"error\":\"view config not readable\"}",
        );
    };
    if let Err(error) = crate::view_config::validate(&bytes, design) {
        let body = serde_json::json!({"error": error}).to_string();
        return respond(conn, 422, "application/json", body.as_bytes());
    }
    respond(conn, 200, "application/json", &bytes)
}

fn respond(conn: &mut TcpStream, code: u16, ct: &str, body: &[u8]) -> std::io::Result<()> {
    let status = match code {
        200 => "200 OK",
        422 => "422 Unprocessable Content",
        _ => "404 Not Found",
    };
    conn.write_all(
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .as_bytes(),
    )?;
    conn.write_all(body)
}

#[cfg(test)]
#[path = "serve_tests.rs"]
mod tests;
