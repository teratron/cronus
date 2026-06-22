use cronus::checkpoint::{
    AUTO_MEMORY_THRESHOLD_BYTES, CheckpointPaths, CheckpointWriter, FileCheckpointWriter,
    MAX_SNAPSHOTS, NoOpCheckpointWriter, build_resume_reminder, list_snapshots, needs_auto_memory,
    prune_snapshots, read_section, write_atomic,
};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp_dir() -> std::path::PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let d = std::env::temp_dir().join(format!("cronus-cp-test-{pid}-{id}"));
    fs::create_dir_all(&d).unwrap();
    d
}

// ── CheckpointPaths ───────────────────────────────────────────────────────────

#[test]
fn checkpoint_paths_layout_under_state_dir() {
    let state = std::path::PathBuf::from("/state");
    let p = CheckpointPaths::new(&state);
    assert_eq!(p.context, std::path::PathBuf::from("/state/checkpoint"));
    assert_eq!(
        p.memory,
        std::path::PathBuf::from("/state/checkpoint/memory")
    );
    assert_eq!(
        p.notes,
        std::path::PathBuf::from("/state/checkpoint/notes.md")
    );
}

#[test]
fn fork_paths_include_fork_id() {
    let state = std::path::PathBuf::from("/state");
    let p = CheckpointPaths::fork(&state, "abc123");
    assert!(
        p.context
            .to_string_lossy()
            .contains("checkpoint-fork-abc123")
    );
    assert!(
        p.memory
            .to_string_lossy()
            .contains("checkpoint-fork-abc123")
    );
    assert!(p.notes.to_string_lossy().contains("checkpoint-fork-abc123"));
}

// ── write_atomic ──────────────────────────────────────────────────────────────

#[test]
fn write_atomic_creates_file_with_content() {
    let dir = tmp_dir();
    let path = dir.join("ctx");
    write_atomic(&path, "hello world").unwrap();
    let got = fs::read_to_string(&path).unwrap();
    assert_eq!(got, "hello world");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn write_atomic_no_tmp_left_on_success() {
    let dir = tmp_dir();
    let path = dir.join("ctx");
    write_atomic(&path, "data").unwrap();
    let tmp = path.with_extension("tmp");
    assert!(!tmp.exists(), "temp file must be renamed away on success");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn write_atomic_creates_parent_dirs() {
    let dir = tmp_dir();
    let deep = dir.join("a").join("b").join("ctx");
    write_atomic(&deep, "nested").unwrap();
    assert!(deep.exists());
    fs::remove_dir_all(&dir).ok();
}

// ── FileCheckpointWriter ──────────────────────────────────────────────────────

#[test]
fn file_writer_writes_context_file() {
    let dir = tmp_dir();
    let paths = CheckpointPaths::new(&dir);
    FileCheckpointWriter.write(&paths, "session body").unwrap();
    let got = fs::read_to_string(&paths.context).unwrap();
    assert_eq!(got, "session body");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn noop_writer_does_nothing() {
    let dir = tmp_dir();
    let paths = CheckpointPaths::new(&dir);
    NoOpCheckpointWriter.write(&paths, "anything").unwrap();
    assert!(!paths.context.exists(), "noop writer must not create files");
    fs::remove_dir_all(&dir).ok();
}

// ── read_section ──────────────────────────────────────────────────────────────

#[test]
fn read_section_returns_full_content_when_under_limit() {
    let dir = tmp_dir();
    let path = dir.join("sec");
    fs::write(&path, "short").unwrap();
    let got = read_section(&path, 1000).unwrap();
    assert_eq!(got, "short");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn read_section_truncates_at_max_bytes() {
    let dir = tmp_dir();
    let path = dir.join("sec");
    let content = "a".repeat(200);
    fs::write(&path, &content).unwrap();
    let got = read_section(&path, 100).unwrap();
    assert!(got.len() <= 100, "must not exceed max_bytes");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn read_section_error_on_missing_file() {
    let result = read_section(&std::path::PathBuf::from("/nonexistent/path/to/file"), 100);
    assert!(result.is_err());
}

// ── System reminder + auto-memory ────────────────────────────────────────────

#[test]
fn resume_reminder_contains_timestamp() {
    let r = build_resume_reminder("2026-06-22T10:00:00Z");
    assert!(r.contains("2026-06-22T10:00:00Z"));
}

#[test]
fn needs_auto_memory_false_below_threshold() {
    assert!(!needs_auto_memory(AUTO_MEMORY_THRESHOLD_BYTES));
}

#[test]
fn needs_auto_memory_true_above_threshold() {
    assert!(needs_auto_memory(AUTO_MEMORY_THRESHOLD_BYTES + 1));
}

// ── Snapshot retention ────────────────────────────────────────────────────────

#[test]
fn list_snapshots_empty_dir() {
    let dir = tmp_dir();
    let snaps = list_snapshots(&dir);
    assert!(snaps.is_empty());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn list_snapshots_returns_sorted_paths() {
    let dir = tmp_dir();
    for name in &["2026-01", "2026-03", "2026-02"] {
        fs::write(dir.join(name), "").unwrap();
    }
    let snaps = list_snapshots(&dir);
    assert_eq!(snaps.len(), 3);
    let names: Vec<_> = snaps
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert_eq!(names, vec!["2026-01", "2026-02", "2026-03"]);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn prune_snapshots_removes_oldest_when_over_cap() {
    let dir = tmp_dir();
    // Create MAX_SNAPSHOTS + 5 snapshot files
    for i in 0..MAX_SNAPSHOTS + 5 {
        fs::write(dir.join(format!("snap-{i:04}")), "").unwrap();
    }
    let removed = prune_snapshots(&dir).unwrap();
    assert_eq!(removed, 5);
    let remaining = list_snapshots(&dir);
    assert_eq!(remaining.len(), MAX_SNAPSHOTS);
    // Verify we kept the NEWEST (highest index)
    let names: Vec<_> = remaining
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(names.contains(&"snap-0049"), "must keep newest entries");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn prune_snapshots_noop_when_under_cap() {
    let dir = tmp_dir();
    for i in 0..5 {
        fs::write(dir.join(format!("snap-{i}")), "").unwrap();
    }
    let removed = prune_snapshots(&dir).unwrap();
    assert_eq!(removed, 0);
    fs::remove_dir_all(&dir).ok();
}

// ── Constants ─────────────────────────────────────────────────────────────────

#[test]
fn constants_have_expected_values() {
    const { assert!(AUTO_MEMORY_THRESHOLD_BYTES == 50_000) }
    const { assert!(MAX_SNAPSHOTS == 50) }
}
