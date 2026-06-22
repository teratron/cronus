use cronus::{Capabilities, Engine};

#[test]
fn engine_exposes_version_and_status() {
    let engine = Engine::new();
    assert!(!engine.version().is_empty());
    assert!(engine.status().contains("Cronus core"));
}
