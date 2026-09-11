//! Guards the SDD reference-containment boundary (Reference Rule §6): a
//! specification file name — the `l1-*.md` / `l2-*.md` naming convention the
//! design layer uses — must never appear in product source, comments
//! included. A simulation-qa pass found 25 such references scattered across
//! six crates (rationale citations that named their source file instead of
//! just stating the rationale); this test keeps them from creeping back in.
//!
//! Scans every `.rs` file under the workspace's `crates/` tree structurally
//! (byte classification, not a literal pattern string) so the check itself
//! never needs to embed the shape it is forbidding.

use std::fs;
use std::path::{Path, PathBuf};

/// Finds a `l1-...md` / `l2-...md` style reference in a line, if present.
/// Matches `l` + (`1` | `2`) + `-` + one or more lowercase-letter/digit/`-`
/// characters + literal `.md` — the exact shape the design layer's spec
/// files are named with.
fn spec_file_reference(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'l' && (bytes[i + 1] == b'1' || bytes[i + 1] == b'2') {
            let start = i;
            let mut j = i + 2;
            if bytes.get(j) == Some(&b'-') {
                j += 1;
                let name_start = j;
                while j < bytes.len()
                    && (bytes[j].is_ascii_lowercase()
                        || bytes[j] == b'-'
                        || bytes[j].is_ascii_digit())
                {
                    j += 1;
                }
                if j > name_start && line.as_bytes()[j..].starts_with(b".md") {
                    return Some(&line[start..j + 3]);
                }
            }
        }
        i += 1;
    }
    None
}

/// Recursively collects every `.rs` file under `dir`.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Defensive: an isolated per-crate build could leave a stray
            // `target/` under `crates/`; never scan build output.
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_product_source_file_names_a_specification_file() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/cli sits two levels under the workspace root")
        .to_path_buf();
    let crates_dir = workspace_root.join("crates");

    let mut files = Vec::new();
    rust_files(&crates_dir, &mut files);
    assert!(
        files.len() > 100,
        "the workspace tree walk found suspiciously few .rs files ({}) under {} — \
         the path resolution is probably wrong rather than the tree being small",
        files.len(),
        crates_dir.display()
    );

    let mut offenders = Vec::new();
    for path in &files {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for (line_no, line) in content.lines().enumerate() {
            if let Some(reference) = spec_file_reference(line) {
                offenders.push(format!(
                    "{}:{}: names {reference}",
                    path.display(),
                    line_no + 1
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "product source must never name a specification file (Reference Rule §6) — \
         restate the rationale in plain language instead:\n{}",
        offenders.join("\n")
    );
}
