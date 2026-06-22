use cronus::router::recovery::{
    classify, recovery_action, CredentialPool, FailoverKind, NoOpProbe, RecoveryAction,
    HealthProbe,
};

// ── Classification ────────────────────────────────────────────────────────────

#[test]
fn http_429_classifies_as_rate_limit() {
    assert_eq!(classify(429, ""), FailoverKind::RateLimit);
}

#[test]
fn http_401_classifies_as_auth_failure() {
    assert_eq!(classify(401, ""), FailoverKind::AuthFailure);
}

#[test]
fn http_403_classifies_as_auth_failure() {
    assert_eq!(classify(403, ""), FailoverKind::AuthFailure);
}

#[test]
fn http_404_classifies_as_model_not_found() {
    assert_eq!(classify(404, ""), FailoverKind::ModelNotFound);
}

#[test]
fn http_500_timeout_body_classifies_network_timeout() {
    assert_eq!(classify(500, "request timed out"), FailoverKind::NetworkTimeout);
}

#[test]
fn http_500_generic_classifies_internal_server_error() {
    assert_eq!(classify(500, "something went wrong"), FailoverKind::InternalServerError);
}

#[test]
fn body_rate_limit_classifies_correctly() {
    assert_eq!(classify(0, "rate limit exceeded, try again"), FailoverKind::RateLimit);
}

#[test]
fn body_invalid_api_key_classifies_auth() {
    assert_eq!(classify(0, "invalid api key"), FailoverKind::AuthFailure);
}

#[test]
fn body_context_length_overflow() {
    assert_eq!(classify(0, "context length overflow"), FailoverKind::ContextOverflow);
}

#[test]
fn body_quota_exceeded() {
    assert_eq!(classify(0, "quota exceeded for billing period"), FailoverKind::QuotaExhausted);
}

#[test]
fn body_unknown_falls_through() {
    assert_eq!(classify(0, "some totally new error"), FailoverKind::Unknown);
}

// ── Recovery actions ──────────────────────────────────────────────────────────

#[test]
fn rate_limit_action_is_retry() {
    assert_eq!(recovery_action(FailoverKind::RateLimit), RecoveryAction::Retry);
}

#[test]
fn auth_failure_action_is_rotate_credential() {
    assert_eq!(recovery_action(FailoverKind::AuthFailure), RecoveryAction::RotateCredential);
}

#[test]
fn context_overflow_action_is_compress() {
    assert_eq!(recovery_action(FailoverKind::ContextOverflow), RecoveryAction::Compress);
}

#[test]
fn unknown_action_is_abort() {
    assert_eq!(recovery_action(FailoverKind::Unknown), RecoveryAction::Abort);
}

// ── Credential pool ───────────────────────────────────────────────────────────

#[test]
fn credential_pool_rotates_round_robin() {
    let mut pool = CredentialPool::new(vec!["a".into(), "b".into(), "c".into()], 10);
    assert_eq!(pool.current().unwrap().value, "a");
    pool.rotate();
    assert_eq!(pool.current().unwrap().value, "b");
    pool.rotate();
    assert_eq!(pool.current().unwrap().value, "c");
    pool.rotate();
    assert_eq!(pool.current().unwrap().value, "a");
}

#[test]
fn credential_pool_evicts_after_threshold() {
    let mut pool = CredentialPool::new(vec!["x".into(), "y".into()], 2);
    pool.record_failure();
    assert_eq!(pool.len(), 2);
    pool.record_failure();
    assert_eq!(pool.len(), 1, "credential must be evicted after threshold failures");
    assert!(!pool.is_empty());
}

#[test]
fn credential_pool_exhausted_when_all_evicted() {
    let mut pool = CredentialPool::new(vec!["only".into()], 1);
    pool.record_failure();
    assert!(pool.is_empty());
    assert!(pool.current().is_none());
}

// ── Health probe ──────────────────────────────────────────────────────────────

#[test]
fn noop_probe_reports_healthy() {
    let status = NoOpProbe.probe();
    assert!(status.healthy);
}

#[test]
fn unhealthy_primary_falls_back_to_fallback() {
    struct AlwaysUnhealthyProbe;
    impl HealthProbe for AlwaysUnhealthyProbe {
        fn probe_primary(&self) -> cronus::router::recovery::ProbeOutcome {
            cronus::router::recovery::ProbeOutcome::Unhealthy
        }
        fn probe_fallback(&self) -> cronus::router::recovery::ProbeOutcome {
            cronus::router::recovery::ProbeOutcome::Healthy
        }
    }
    let status = AlwaysUnhealthyProbe.probe();
    assert!(status.healthy, "fallback probe success must yield healthy status");
}

#[test]
fn both_probes_fail_yields_unhealthy() {
    struct BothFailProbe;
    impl HealthProbe for BothFailProbe {
        fn probe_primary(&self) -> cronus::router::recovery::ProbeOutcome {
            cronus::router::recovery::ProbeOutcome::Unhealthy
        }
        fn probe_fallback(&self) -> cronus::router::recovery::ProbeOutcome {
            cronus::router::recovery::ProbeOutcome::Unhealthy
        }
    }
    let status = BothFailProbe.probe();
    assert!(!status.healthy, "both probes failing must yield unhealthy status");
}
