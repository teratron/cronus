use cronus_core::quality::{
    CardTag, DoneGateStatus, GateKind, GateResult, GateResultStore, GateStatus, Language,
    check_done_gate, detect_language,
};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp_dir(label: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("cronus-quality-{pid}-{id}-{label}"));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn make_result(gate: GateKind, status: GateStatus) -> GateResult {
    GateResult {
        gate,
        status,
        output: String::new(),
        duration_ms: 0,
    }
}

// ── detect_language ─────────────────────────────────────────────────────────

#[test]
fn detect_language_rust_from_cargo_toml() {
    let dir = tmp_dir("rust");
    std::fs::write(dir.join("Cargo.toml"), "").unwrap();
    assert_eq!(detect_language(&dir), Language::Rust);
}

#[test]
fn detect_language_typescript_from_package_json() {
    let dir = tmp_dir("ts");
    std::fs::write(dir.join("package.json"), "").unwrap();
    assert_eq!(detect_language(&dir), Language::TypeScript);
}

#[test]
fn detect_language_python_from_pyproject() {
    let dir = tmp_dir("py-pyproject");
    std::fs::write(dir.join("pyproject.toml"), "").unwrap();
    assert_eq!(detect_language(&dir), Language::Python);
}

#[test]
fn detect_language_python_from_setup_py() {
    let dir = tmp_dir("py-setup");
    std::fs::write(dir.join("setup.py"), "").unwrap();
    assert_eq!(detect_language(&dir), Language::Python);
}

#[test]
fn detect_language_go_from_go_mod() {
    let dir = tmp_dir("go");
    std::fs::write(dir.join("go.mod"), "").unwrap();
    assert_eq!(detect_language(&dir), Language::Go);
}

#[test]
fn detect_language_unknown_for_empty_dir() {
    let dir = tmp_dir("unknown");
    assert_eq!(detect_language(&dir), Language::Unknown);
}

// ── GateResultStore ─────────────────────────────────────────────────────────

#[test]
fn gate_result_store_records_and_retrieves_for_card() {
    let mut store = GateResultStore::new();
    store.record("card-1", make_result(GateKind::Tests, GateStatus::Pass));
    store.record("card-1", make_result(GateKind::Lint, GateStatus::Fail));

    let results = store.results_for("card-1");
    assert_eq!(results.len(), 2);
}

#[test]
fn gate_result_store_returns_empty_for_missing_card() {
    let store = GateResultStore::new();
    assert!(store.results_for("nobody").is_empty());
}

#[test]
fn gate_result_store_filters_by_card_id() {
    let mut store = GateResultStore::new();
    store.record("card-a", make_result(GateKind::Tests, GateStatus::Pass));
    store.record("card-b", make_result(GateKind::Tests, GateStatus::Fail));

    assert_eq!(store.results_for("card-a").len(), 1);
    assert_eq!(store.results_for("card-b").len(), 1);
}

// ── check_done_gate ─────────────────────────────────────────────────────────

#[test]
fn done_gate_passes_when_all_required_pass() {
    let results = vec![
        make_result(GateKind::Tests, GateStatus::Pass),
        make_result(GateKind::Lint, GateStatus::Pass),
        make_result(GateKind::TypeFormat, GateStatus::Pass),
    ];
    assert_eq!(check_done_gate(&results), DoneGateStatus::Pass);
}

#[test]
fn done_gate_fails_when_tests_missing() {
    let results = vec![
        make_result(GateKind::Lint, GateStatus::Pass),
        make_result(GateKind::TypeFormat, GateStatus::Pass),
    ];
    assert_eq!(check_done_gate(&results), DoneGateStatus::Fail);
}

#[test]
fn done_gate_fails_when_lint_fails() {
    let results = vec![
        make_result(GateKind::Tests, GateStatus::Pass),
        make_result(GateKind::Lint, GateStatus::Fail),
        make_result(GateKind::TypeFormat, GateStatus::Pass),
    ];
    assert_eq!(check_done_gate(&results), DoneGateStatus::Fail);
}

#[test]
fn done_gate_fails_when_type_format_missing() {
    let results = vec![
        make_result(GateKind::Tests, GateStatus::Pass),
        make_result(GateKind::Lint, GateStatus::Pass),
    ];
    assert_eq!(check_done_gate(&results), DoneGateStatus::Fail);
}

#[test]
fn done_gate_passes_with_extra_skipped_gates() {
    let results = vec![
        make_result(GateKind::Tests, GateStatus::Pass),
        make_result(GateKind::Lint, GateStatus::Pass),
        make_result(GateKind::TypeFormat, GateStatus::Pass),
        make_result(GateKind::Benchmarks, GateStatus::Skipped),
        make_result(GateKind::Security, GateStatus::Skipped),
    ];
    assert_eq!(check_done_gate(&results), DoneGateStatus::Pass);
}

// ── GateKind ────────────────────────────────────────────────────────────────

#[test]
fn gate_kind_always_on_set() {
    assert!(GateKind::Tests.is_always_on());
    assert!(GateKind::Lint.is_always_on());
    assert!(GateKind::TypeFormat.is_always_on());
    assert!(!GateKind::Benchmarks.is_always_on());
    assert!(!GateKind::Security.is_always_on());
}

#[test]
fn gate_kind_conditional_set() {
    assert!(GateKind::Benchmarks.is_conditional());
    assert!(GateKind::Security.is_conditional());
    assert!(!GateKind::Tests.is_conditional());
}

// ── CardTag ─────────────────────────────────────────────────────────────────

#[test]
fn card_tag_as_str_roundtrip() {
    assert_eq!(CardTag::Performance.as_str(), "performance");
    assert_eq!(CardTag::Security.as_str(), "security");
}
