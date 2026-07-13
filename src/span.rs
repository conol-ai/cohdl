//! Source files, byte spans, and line/column lookup.
//!
//! Every AST node and diagnostic carries a `Span`. The Constitution requires
//! every diagnostic to carry a precise source span, so spans are threaded
//! through the entire pipeline, including through fn expansion (an expanded
//! instance remembers the source span of the `inst` that produced it).

use std::fmt;

/// Identifies a file registered in the [`SourceMap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub u32);

/// A byte range within one source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(file: FileId, start: u32, end: u32) -> Self {
        Span { file, start, end }
    }

    /// A span covering both `self` and `other` (must be in the same file).
    pub fn to(self, other: Span) -> Span {
        debug_assert_eq!(self.file, other.file);
        Span {
            file: self.file,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// 1-based line/column position, for rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCol {
    pub line: u32,
    pub col: u32,
}

struct SourceFile {
    name: String,
    text: String,
    /// Byte offset of the start of each line.
    line_starts: Vec<u32>,
}

/// Owns all source text for a compilation.
#[derive(Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        SourceMap { files: Vec::new() }
    }

    pub fn add_file(&mut self, name: impl Into<String>, text: impl Into<String>) -> FileId {
        let text = text.into();
        let mut line_starts = vec![0u32];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        self.files.push(SourceFile {
            name: name.into(),
            text,
            line_starts,
        });
        FileId(self.files.len() as u32 - 1)
    }

    pub fn file_ids(&self) -> impl Iterator<Item = FileId> {
        (0..self.files.len() as u32).map(FileId)
    }

    pub fn name(&self, file: FileId) -> &str {
        &self.files[file.0 as usize].name
    }

    pub fn text(&self, file: FileId) -> &str {
        &self.files[file.0 as usize].text
    }

    pub fn snippet(&self, span: Span) -> &str {
        &self.files[span.file.0 as usize].text[span.start as usize..span.end as usize]
    }

    pub fn line_col(&self, file: FileId, offset: u32) -> LineCol {
        let f = &self.files[file.0 as usize];
        let line_idx = match f.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let line_start = f.line_starts[line_idx];
        // Column counts characters, not bytes, so multi-byte source (e.g. a stray
        // `Ω`) still points at the right visual column.
        let col = f.text[line_start as usize..offset as usize].chars().count() as u32;
        LineCol {
            line: line_idx as u32 + 1,
            col: col + 1,
        }
    }

    /// The full text of the (1-based) line, without its trailing newline.
    pub fn line_text(&self, file: FileId, line: u32) -> &str {
        let f = &self.files[file.0 as usize];
        let start = f.line_starts[(line - 1) as usize] as usize;
        let end = f
            .line_starts
            .get(line as usize)
            .map(|&s| s as usize)
            .unwrap_or(f.text.len());
        f.text[start..end].trim_end_matches(['\n', '\r'])
    }
}

impl fmt::Debug for SourceMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.files.iter().map(|sf| &sf.name))
            .finish()
    }
}
