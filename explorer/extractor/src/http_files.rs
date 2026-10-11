use std::path::{Component, Path, PathBuf};

pub fn percent_decode(value: &str, query: bool) -> Option<String> {
    let mut bytes = value.bytes();
    let mut decoded = Vec::with_capacity(value.len());
    while let Some(byte) = bytes.next() {
        let byte = match byte {
            b'%' => {
                let hi = char::from(bytes.next()?).to_digit(16)?;
                let lo = char::from(bytes.next()?).to_digit(16)?;
                u8::try_from(hi * 16 + lo).ok()?
            }
            b'+' if query => b' ',
            other => other,
        };
        if byte == 0 {
            return None;
        }
        decoded.push(byte);
    }
    String::from_utf8(decoded).ok()
}

pub fn confined_file(root: &Path, relative: &str) -> Option<PathBuf> {
    let path = Path::new(relative);
    if path
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
        || relative.contains('\\')
    {
        return None;
    }
    let root = root.canonicalize().ok()?;
    let file = root.join(path).canonicalize().ok()?;
    (file.starts_with(root) && file.is_file()).then_some(file)
}

pub fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "js" => "application/javascript",
        "css" => "text/css",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

pub fn view_file(project: &Path, dist: &Path, relative: &str) -> Option<PathBuf> {
    let requested = project.join(relative);
    // An invalid project override must never expose a bundled replacement.
    if std::fs::symlink_metadata(&requested).is_ok() {
        confined_file(project, relative)
    } else {
        confined_file(dist, relative)
    }
}
