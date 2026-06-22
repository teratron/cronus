//! Symbol extraction — tree-sitter seam with a regex stub for Phase 4.

// ── Confidence ────────────────────────────────────────────────────────────────

/// Confidence of extracted symbol information.
#[derive(Debug, Clone, PartialEq)]
pub enum Confidence {
    /// Extracted from AST (highest confidence).
    Extracted,
    /// Inferred from naming patterns / heuristics.
    Inferred(f32),
    /// Ambiguous — multiple candidates.
    Ambiguous,
}

// ── Symbol ────────────────────────────────────────────────────────────────────

/// A code symbol extracted from a source file.
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: u32,
    pub confidence: Confidence,
    pub doc: Option<String>,
}

/// Coarse-grained symbol kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Constant,
    TypeAlias,
    Other,
}

// ── Extractor seam ────────────────────────────────────────────────────────────

/// Seam trait for language-specific symbol extraction.
///
/// At Phase 4, `RegexExtractor` is the default. Tree-sitter extractors
/// wire in Phase 5 when grammar binaries are bundled.
pub trait Extractor: Send + Sync {
    fn extract(&self, source: &str) -> Vec<Symbol>;
}

// ── RegexExtractor (Phase 4 stub) ─────────────────────────────────────────────

/// Extracts Rust-like function and struct declarations via simple regex patterns.
pub struct RegexExtractor;

impl Extractor for RegexExtractor {
    fn extract(&self, source: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        for (line_idx, line) in source.lines().enumerate() {
            let lineno = line_idx as u32 + 1;
            let trimmed = line.trim_start();

            if let Some(sym) = try_extract_fn(trimmed, lineno)
                .or_else(|| try_extract_keyword(trimmed, lineno, "struct ", "pub struct ", SymbolKind::Struct))
                .or_else(|| try_extract_keyword(trimmed, lineno, "enum ", "pub enum ", SymbolKind::Enum))
            {
                symbols.push(sym);
            }
        }
        symbols
    }
}

fn try_extract_fn(trimmed: &str, line: u32) -> Option<Symbol> {
    let rest = trimmed
        .strip_prefix("fn ")
        .or_else(|| trimmed.strip_prefix("pub fn "))
        .or_else(|| trimmed.strip_prefix("pub(crate) fn "))?;
    let name = rest.split('(').next().map(str::trim)?;
    if name.is_empty() {
        return None;
    }
    Some(Symbol {
        name: name.to_owned(),
        kind: SymbolKind::Function,
        line,
        confidence: Confidence::Extracted,
        doc: None,
    })
}

fn try_extract_keyword(trimmed: &str, line: u32, prefix: &str, pub_prefix: &str, kind: SymbolKind) -> Option<Symbol> {
    let rest = trimmed
        .strip_prefix(pub_prefix)
        .or_else(|| trimmed.strip_prefix(prefix))?;
    let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next()?;
    if name.is_empty() {
        return None;
    }
    Some(Symbol {
        name: name.to_owned(),
        kind,
        line,
        confidence: Confidence::Extracted,
        doc: None,
    })
}
