//! Model error recovery — failover taxonomy, classification pipeline,
//! credential pool, and multi-hop health probe.

use std::time::{Duration, Instant};

// ── FailoverKind ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverKind {
    RateLimit,
    AuthFailure,
    ContextOverflow,
    NetworkTimeout,
    ModelNotFound,
    QuotaExhausted,
    InternalServerError,
    Unknown,
}

impl FailoverKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FailoverKind::RateLimit => "RateLimit",
            FailoverKind::AuthFailure => "AuthFailure",
            FailoverKind::ContextOverflow => "ContextOverflow",
            FailoverKind::NetworkTimeout => "NetworkTimeout",
            FailoverKind::ModelNotFound => "ModelNotFound",
            FailoverKind::QuotaExhausted => "QuotaExhausted",
            FailoverKind::InternalServerError => "InternalServerError",
            FailoverKind::Unknown => "Unknown",
        }
    }
}

// ── RecoveryAction ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    Retry,
    Compress,
    RotateCredential,
    Fallback,
    Abort,
}

// ── Classification pipeline ───────────────────────────────────────────────────

/// Classify an HTTP error response into a `FailoverKind`.
///
/// Matching order: HTTP status (specific) → body patterns → Unknown.
pub fn classify(http_status: u16, body: &str) -> FailoverKind {
    match http_status {
        401 | 403 => FailoverKind::AuthFailure,
        404 => FailoverKind::ModelNotFound,
        429 => FailoverKind::RateLimit,
        500..=599 => classify_5xx(body),
        _ => classify_body(body),
    }
}

fn classify_5xx(body: &str) -> FailoverKind {
    let b = body.to_ascii_lowercase();
    if b.contains("timeout") || b.contains("timed out") {
        FailoverKind::NetworkTimeout
    } else {
        FailoverKind::InternalServerError
    }
}

fn classify_body(body: &str) -> FailoverKind {
    let b = body.to_ascii_lowercase();
    if b.contains("rate limit") || b.contains("too many requests") {
        FailoverKind::RateLimit
    } else if b.contains("unauthorized") || b.contains("invalid api key") {
        FailoverKind::AuthFailure
    } else if b.contains("context") && (b.contains("length") || b.contains("overflow")) {
        FailoverKind::ContextOverflow
    } else if b.contains("quota") || b.contains("billing") {
        FailoverKind::QuotaExhausted
    } else if b.contains("timeout") || b.contains("timed out") {
        FailoverKind::NetworkTimeout
    } else {
        FailoverKind::Unknown
    }
}

/// Map a `FailoverKind` to its default recovery action.
pub fn recovery_action(kind: FailoverKind) -> RecoveryAction {
    match kind {
        FailoverKind::RateLimit => RecoveryAction::Retry,
        FailoverKind::AuthFailure => RecoveryAction::RotateCredential,
        FailoverKind::ContextOverflow => RecoveryAction::Compress,
        FailoverKind::NetworkTimeout => RecoveryAction::Retry,
        FailoverKind::ModelNotFound => RecoveryAction::Fallback,
        FailoverKind::QuotaExhausted => RecoveryAction::Fallback,
        FailoverKind::InternalServerError => RecoveryAction::Retry,
        FailoverKind::Unknown => RecoveryAction::Abort,
    }
}

// ── Credential pool ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Credential {
    pub value: String,
    failure_count: u32,
}

impl Credential {
    pub fn new(value: String) -> Self {
        Credential {
            value,
            failure_count: 0,
        }
    }
}

/// Round-robin credential pool with per-credential failure tracking.
pub struct CredentialPool {
    credentials: Vec<Credential>,
    index: usize,
    evict_threshold: u32,
}

impl CredentialPool {
    pub fn new(credentials: Vec<String>, evict_threshold: u32) -> Self {
        CredentialPool {
            credentials: credentials.into_iter().map(Credential::new).collect(),
            index: 0,
            evict_threshold,
        }
    }

    /// Return the current credential without advancing the index.
    pub fn current(&self) -> Option<&Credential> {
        self.credentials.get(self.index)
    }

    /// Rotate to the next credential and return it.
    ///
    /// Returns `None` if the pool is exhausted.
    pub fn rotate(&mut self) -> Option<&Credential> {
        if self.credentials.is_empty() {
            return None;
        }
        self.index = (self.index + 1) % self.credentials.len();
        self.credentials.get(self.index)
    }

    /// Record a failure on the current credential. Evicts if threshold is reached.
    pub fn record_failure(&mut self) {
        if let Some(cred) = self.credentials.get_mut(self.index) {
            cred.failure_count += 1;
            if cred.failure_count >= self.evict_threshold {
                self.credentials.remove(self.index);
                if !self.credentials.is_empty() {
                    self.index %= self.credentials.len();
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.credentials.is_empty()
    }

    pub fn len(&self) -> usize {
        self.credentials.len()
    }
}

// ── ProviderHealthStatus ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ProviderHealthStatus {
    pub healthy: bool,
    pub last_probe: Instant,
    pub latency_p50_ms: u64,
    pub context_window: Option<u32>,
}

impl ProviderHealthStatus {
    pub fn age(&self) -> Duration {
        self.last_probe.elapsed()
    }
}

// ── Health probe seam ─────────────────────────────────────────────────────────

/// Probe result from a single hop of the multi-hop health probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeOutcome {
    Healthy,
    Unhealthy,
}

/// Multi-hop health probe seam.
///
/// Primary probe: lightweight HTTP GET to `/health` (if supported).
/// Fallback probe: minimal model call with a fixed 1-token prompt.
/// A provider is marked unhealthy only when both probes fail.
pub trait HealthProbe: Send + Sync {
    fn probe_primary(&self) -> ProbeOutcome;
    fn probe_fallback(&self) -> ProbeOutcome;

    fn probe(&self) -> ProviderHealthStatus {
        let primary = self.probe_primary();
        let fallback = if primary == ProbeOutcome::Healthy {
            ProbeOutcome::Healthy
        } else {
            self.probe_fallback()
        };
        ProviderHealthStatus {
            healthy: primary == ProbeOutcome::Healthy || fallback == ProbeOutcome::Healthy,
            last_probe: Instant::now(),
            latency_p50_ms: 0,
            context_window: None,
        }
    }
}

/// Always-healthy stub for use in tests.
pub struct NoOpProbe;

impl HealthProbe for NoOpProbe {
    fn probe_primary(&self) -> ProbeOutcome {
        ProbeOutcome::Healthy
    }
    fn probe_fallback(&self) -> ProbeOutcome {
        ProbeOutcome::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_429_is_rate_limit() {
        assert_eq!(classify(429, ""), FailoverKind::RateLimit);
    }

    #[test]
    fn classify_401_is_auth_failure() {
        assert_eq!(classify(401, ""), FailoverKind::AuthFailure);
    }

    #[test]
    fn classify_body_rate_limit() {
        assert_eq!(classify(0, "rate limit exceeded"), FailoverKind::RateLimit);
    }

    #[test]
    fn classify_body_auth() {
        assert_eq!(classify(0, "invalid api key"), FailoverKind::AuthFailure);
    }

    #[test]
    fn classify_500_timeout_body() {
        assert_eq!(
            classify(503, "request timed out"),
            FailoverKind::NetworkTimeout
        );
    }

    #[test]
    fn recovery_action_mapping() {
        assert_eq!(
            recovery_action(FailoverKind::RateLimit),
            RecoveryAction::Retry
        );
        assert_eq!(
            recovery_action(FailoverKind::AuthFailure),
            RecoveryAction::RotateCredential
        );
        assert_eq!(
            recovery_action(FailoverKind::ContextOverflow),
            RecoveryAction::Compress
        );
        assert_eq!(
            recovery_action(FailoverKind::Unknown),
            RecoveryAction::Abort
        );
    }

    #[test]
    fn credential_pool_rotate() {
        let mut pool = CredentialPool::new(vec!["key-a".into(), "key-b".into()], 5);
        assert_eq!(pool.current().unwrap().value, "key-a");
        pool.rotate();
        assert_eq!(pool.current().unwrap().value, "key-b");
        pool.rotate();
        assert_eq!(pool.current().unwrap().value, "key-a");
    }

    #[test]
    fn credential_pool_evicts_on_threshold() {
        let mut pool = CredentialPool::new(vec!["only-key".into()], 2);
        pool.record_failure();
        assert!(!pool.is_empty());
        pool.record_failure();
        assert!(pool.is_empty());
    }

    #[test]
    fn noop_probe_is_healthy() {
        let probe = NoOpProbe;
        let status = probe.probe();
        assert!(status.healthy);
    }
}
