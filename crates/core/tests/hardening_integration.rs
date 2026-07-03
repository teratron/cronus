//! Cross-subsystem hardening validation (Phase 9 §T-9T01): seeds a real
//! secret into a state tier and proves it reaches none of the three egress
//! surfaces this phase built — backup, error reporting, telemetry — and that
//! both consent gates (report + telemetry) block by default.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use cronus::backup::{self, BackupOptions};
use cronus::error_reporting::{
    self, ConsentDecision, ConsentMode, FingerprintTable, ReportRequest,
};
use cronus::telemetry::{MetricPayload, TelemetryStore};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp_dir(label: &str) -> PathBuf {
    let pid = std::process::id();
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("cronus-hardening-{pid}-{id}-{label}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

const SEEDED_SECRET: &str = "sk-LIVE-integration-test-9c8f2e";

#[test]
fn a_seeded_secret_never_reaches_a_backup_archive() {
    let state_root = tmp_dir("state-backup");
    let backups_dir = tmp_dir("backups");
    fs::write(state_root.join("config.json"), "{\"theme\":\"dark\"}").unwrap();
    fs::write(state_root.join(".env"), format!("API_KEY={SEEDED_SECRET}")).unwrap();

    let backup_ref =
        backup::create(&state_root, &backups_dir, None, BackupOptions::default()).unwrap();

    assert!(
        !backup_ref.path.join(".env").exists(),
        "the secret file itself is excluded"
    );
    for entry in walk(&backup_ref.path) {
        let content = fs::read_to_string(&entry).unwrap_or_default();
        assert!(
            !content.contains(SEEDED_SECRET),
            "seeded secret leaked into backup file {}",
            entry.display()
        );
    }

    let _ = fs::remove_dir_all(&state_root);
    let _ = fs::remove_dir_all(&backups_dir);
}

#[test]
fn a_seeded_secret_never_reaches_a_scrubbed_report_body() {
    let request = ReportRequest {
        error_type: "panic".to_string(),
        message: format!("auth failed with token={SEEDED_SECRET} while dialing upstream"),
        app_version: "0.1.0".to_string(),
        environment_summary: "windows 10".to_string(),
        repro_hints: vec!["cronus status".to_string()],
        episode_id: "ep-hardening".to_string(),
    };
    let mut fingerprints = FingerprintTable::new();
    let preview =
        error_reporting::prepare_preview(&request, "", &[SEEDED_SECRET], &mut fingerprints, 1_000);

    assert!(
        !preview.sanitized_message.contains(SEEDED_SECRET),
        "seeded secret leaked into the report preview"
    );
    assert!(
        !format!("{preview:?}").contains(SEEDED_SECRET),
        "seeded secret leaked into any field of the preview"
    );
    assert!(
        preview.sanitized_message.contains("while dialing upstream"),
        "non-secret content preserved"
    );
}

#[test]
fn a_seeded_secret_never_reaches_a_telemetry_payload() {
    let mut store = TelemetryStore::new();
    store.set_opt_in(true);

    // MetricPayload has no free-text variant, so there is no legitimate way
    // to carry the secret as a value. The only string on the wire is `name`,
    // which is allowlist-checked — attempting to smuggle the secret in
    // through the name must be rejected outright.
    let result = store.record(SEEDED_SECRET, MetricPayload::Count { value: 1 }, 1);
    assert!(
        result.is_err(),
        "an unknown (and here, secret-shaped) metric name is rejected"
    );
    assert!(
        store.inspect().is_empty(),
        "the rejected attempt left nothing queued"
    );

    // Legitimate allowlisted events also never carry the secret — proven by
    // construction (no string field exists to put it in).
    store
        .record("doctor_check", MetricPayload::Outcome { success: true }, 2)
        .unwrap();
    for event in store.inspect() {
        assert!(!format!("{event:?}").contains(SEEDED_SECRET));
    }
}

#[test]
fn report_and_telemetry_consent_gates_both_block_egress_by_default() {
    // Report: `Ask` mode with no explicit answer blocks (never assumes yes).
    assert_eq!(
        error_reporting::consent_decision(ConsentMode::Ask, None),
        ConsentDecision::Blocked
    );
    assert_eq!(
        error_reporting::consent_decision(ConsentMode::Never, Some(true)),
        ConsentDecision::Blocked
    );

    // Telemetry: opted out by default; recording is a no-op until opt-in.
    let mut store = TelemetryStore::new();
    assert!(!store.is_opted_in());
    store
        .record("startup", MetricPayload::Count { value: 1 }, 1)
        .unwrap();
    assert!(
        store.inspect().is_empty(),
        "telemetry emits nothing absent opt-in"
    );
    assert!(
        store.drain_for_send().is_empty(),
        "nothing is ever handed over for sending while opted out"
    );
}

#[test]
fn a_restored_backup_is_resumable_and_still_carries_no_secret() {
    // End-to-end: seed -> backup -> restore -> the resumable tier is clean.
    let state_root = tmp_dir("state-e2e");
    let backups_dir = tmp_dir("backups-e2e");
    let restore_target = tmp_dir("restored-e2e");
    fs::write(state_root.join("config.json"), "{}").unwrap();
    fs::write(state_root.join(".env"), format!("TOKEN={SEEDED_SECRET}")).unwrap();

    let backup_ref =
        backup::create(&state_root, &backups_dir, None, BackupOptions::default()).unwrap();
    backup::restore(&backup_ref, &restore_target).unwrap();

    assert!(
        restore_target.join("config.json").exists(),
        "restored tier is resumable"
    );
    assert!(!restore_target.join(".env").exists());
    for entry in walk(&restore_target) {
        let content = fs::read_to_string(&entry).unwrap_or_default();
        assert!(!content.contains(SEEDED_SECRET));
    }

    let _ = fs::remove_dir_all(&state_root);
    let _ = fs::remove_dir_all(&backups_dir);
    let _ = fs::remove_dir_all(&restore_target);
}

/// Recursively collect file paths under `root` (test helper — no symlink
/// following, mirroring the production copy's own safety stance).
fn walk(root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                out.extend(walk(&path));
            } else if file_type.is_file() {
                out.push(path);
            }
        }
    }
    out
}
