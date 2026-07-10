//! Trigger triage — classifies inbound signals before spawning agent sessions.

use std::collections::HashMap;

// ── TriggerEnvelope ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceType {
    ChatMessage,
    Webhook,
    Cron,
    Event,
    SubAgentSpawn,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::ChatMessage => "chat_message",
            SourceType::Webhook => "webhook",
            SourceType::Cron => "cron",
            SourceType::Event => "event",
            SourceType::SubAgentSpawn => "sub_agent_spawn",
        }
    }

    /// Default dedup window in seconds (0 = disabled).
    pub fn default_dedup_window_sec(&self) -> u32 {
        match self {
            SourceType::Event => 60,
            SourceType::Webhook => 30,
            SourceType::Cron => 0,
            SourceType::ChatMessage => 0,
            SourceType::SubAgentSpawn => 0,
        }
    }

    /// Default rate limit in requests per minute (0 = unlimited).
    pub fn default_rate_limit_rpm(&self) -> u32 {
        match self {
            SourceType::Webhook => 100,
            SourceType::Event => 500,
            _ => 0,
        }
    }
}

/// Classification inputs only — not the full payload content.
#[derive(Debug, Clone)]
pub struct TriggerPayload {
    /// Text excerpt (≤200 tokens / ~200 words).
    pub excerpt: String,
    pub event_kind: Option<String>,
    pub urgency_hint: Option<String>,
}

impl TriggerPayload {
    pub fn new(excerpt: impl Into<String>) -> Self {
        TriggerPayload {
            excerpt: excerpt.into(),
            event_kind: None,
            urgency_hint: None,
        }
    }

    pub fn with_event_kind(mut self, kind: impl Into<String>) -> Self {
        self.event_kind = Some(kind.into());
        self
    }
}

/// Normalized inbound signal.
#[derive(Debug, Clone)]
pub struct TriggerEnvelope {
    pub id: String,
    pub source_type: SourceType,
    pub source_id: Option<String>,
    pub payload: TriggerPayload,
    pub received_at: u64,
    pub workspace_id: String,
    pub metadata: HashMap<String, String>,
}

impl TriggerEnvelope {
    pub fn new(
        id: impl Into<String>,
        source_type: SourceType,
        payload: TriggerPayload,
        received_at: u64,
        workspace_id: impl Into<String>,
    ) -> Self {
        TriggerEnvelope {
            id: id.into(),
            source_type,
            source_id: None,
            payload,
            received_at,
            workspace_id: workspace_id.into(),
            metadata: HashMap::new(),
        }
    }
}

// ── TriageDecision ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriageDecision {
    Drop { reason: String },
    Notify { message: String, channel: String },
    SpawnReactor { agent_tier: String, task: String },
    SpawnOrchestrator { goal: String, priority: String },
}

impl TriageDecision {
    pub fn kind(&self) -> &'static str {
        match self {
            TriageDecision::Drop { .. } => "drop",
            TriageDecision::Notify { .. } => "notify",
            TriageDecision::SpawnReactor { .. } => "spawn_reactor",
            TriageDecision::SpawnOrchestrator { .. } => "spawn_orchestrator",
        }
    }
}

// ── Dedup cache ───────────────────────────────────────────────────────────────

/// A content hash key for dedup — lightweight hash of excerpt + event_kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash(Vec<u8>);

impl ContentHash {
    pub fn from_payload(payload: &TriggerPayload) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        payload.excerpt.hash(&mut hasher);
        if let Some(k) = &payload.event_kind {
            k.hash(&mut hasher);
        }
        ContentHash(hasher.finish().to_le_bytes().to_vec())
    }
}

/// In-memory dedup cache. Restart clears it — by design.
#[derive(Debug, Default)]
pub struct DeduplicateCache {
    store: HashMap<(SourceType, ContentHash), u64>,
    window_sec_by_source: HashMap<SourceType, u32>,
}

impl DeduplicateCache {
    pub fn new() -> Self {
        DeduplicateCache::default()
    }

    pub fn set_window(&mut self, source: SourceType, window_sec: u32) {
        self.window_sec_by_source.insert(source, window_sec);
    }

    fn window_for(&self, source: SourceType) -> u32 {
        self.window_sec_by_source
            .get(&source)
            .copied()
            .unwrap_or_else(|| source.default_dedup_window_sec())
    }

    /// Returns true if this envelope is a duplicate within the window.
    pub fn is_duplicate(&self, envelope: &TriggerEnvelope, now_sec: u64) -> bool {
        let window = self.window_for(envelope.source_type);
        if window == 0 {
            return false;
        }
        let hash = ContentHash::from_payload(&envelope.payload);
        let key = (envelope.source_type, hash);
        if let Some(&seen_at) = self.store.get(&key) {
            now_sec.saturating_sub(seen_at) < window as u64
        } else {
            false
        }
    }

    /// Record an envelope as seen.
    pub fn record(&mut self, envelope: &TriggerEnvelope, now_sec: u64) {
        let window = self.window_for(envelope.source_type);
        if window == 0 {
            return;
        }
        let hash = ContentHash::from_payload(&envelope.payload);
        self.store.insert((envelope.source_type, hash), now_sec);
    }
}

// ── Rate limiter ──────────────────────────────────────────────────────────────

#[derive(Debug)]
struct RateWindow {
    count: u32,
    window_start_sec: u64,
}

/// Per-source-type rate limiter.
#[derive(Debug, Default)]
pub struct RateLimiter {
    windows: HashMap<SourceType, RateWindow>,
    rpm_by_source: HashMap<SourceType, u32>,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter::default()
    }

    pub fn set_limit(&mut self, source: SourceType, rpm: u32) {
        self.rpm_by_source.insert(source, rpm);
    }

    fn limit_for(&self, source: SourceType) -> u32 {
        self.rpm_by_source
            .get(&source)
            .copied()
            .unwrap_or_else(|| source.default_rate_limit_rpm())
    }

    /// Returns true if the envelope exceeds the rate limit for its source type.
    pub fn is_rate_limited(&mut self, source_type: SourceType, now_sec: u64) -> bool {
        let limit = self.limit_for(source_type);
        if limit == 0 {
            return false;
        }
        let window = self.windows.entry(source_type).or_insert(RateWindow {
            count: 0,
            window_start_sec: now_sec,
        });
        // Reset window after 60 seconds
        if now_sec.saturating_sub(window.window_start_sec) >= 60 {
            window.count = 0;
            window.window_start_sec = now_sec;
        }
        window.count += 1;
        window.count > limit
    }
}

// ── Triage pipeline ───────────────────────────────────────────────────────────

/// Confidence threshold below which the cloud/rule fallback is triggered.
pub const CONFIDENCE_THRESHOLD: f32 = 0.8;

/// Max allowed excerpt length in bytes.
const MAX_EXCERPT_BYTES: usize = 2000;

/// Local classifier stub — always returns 0.0 confidence, forcing rule fallback.
/// Pluggable via the extension registry (`TriageClassifier` interface).
fn local_classify(envelope: &TriggerEnvelope) -> (TriageDecision, f32) {
    (rule_classify(envelope), 0.0)
}

/// Deterministic rule-based fallback classifier.
pub fn rule_classify(envelope: &TriggerEnvelope) -> TriageDecision {
    match envelope.source_type {
        SourceType::Cron
        | SourceType::Webhook
        | SourceType::ChatMessage
        | SourceType::SubAgentSpawn => TriageDecision::SpawnReactor {
            agent_tier: "chat".to_string(),
            task: envelope.payload.excerpt.clone(),
        },
        SourceType::Event => {
            let is_error = envelope
                .payload
                .event_kind
                .as_deref()
                .map(|k| k.contains("error") || k.contains("fail"))
                .unwrap_or(false);
            if is_error {
                TriageDecision::SpawnOrchestrator {
                    goal: envelope.payload.excerpt.clone(),
                    priority: "normal".to_string(),
                }
            } else {
                TriageDecision::Notify {
                    message: envelope.payload.excerpt.clone(),
                    channel: "default".to_string(),
                }
            }
        }
    }
}

/// Full triage pipeline.
pub fn triage(
    envelope: &TriggerEnvelope,
    cache: &mut DeduplicateCache,
    rate_limiter: &mut RateLimiter,
    now_sec: u64,
) -> TriageDecision {
    // 1. Rate limit check
    if rate_limiter.is_rate_limited(envelope.source_type, now_sec) {
        return TriageDecision::Drop {
            reason: "rate_limit".to_string(),
        };
    }
    // 2. Dedup check
    if cache.is_duplicate(envelope, now_sec) {
        return TriageDecision::Drop {
            reason: "dedup_suppressed".to_string(),
        };
    }
    // 3. Payload guard
    if envelope.payload.excerpt.is_empty() || envelope.payload.excerpt.len() > MAX_EXCERPT_BYTES {
        return TriageDecision::Drop {
            reason: "payload_invalid".to_string(),
        };
    }
    // 4. Classify: local → cloud seam → rule
    let (decision, confidence) = local_classify(envelope);
    let final_decision = if confidence >= CONFIDENCE_THRESHOLD {
        decision
    } else {
        // Cloud fallback seam — for now falls to rule classification
        rule_classify(envelope)
    };
    // 5. Record dedup
    cache.record(envelope, now_sec);
    final_decision
}

// ── Triage history record ─────────────────────────────────────────────────────

/// Stored record for `trigger history` command.
#[derive(Debug, Clone)]
pub struct TriageRecord {
    pub envelope_id: String,
    pub source_type: SourceType,
    pub decision: String,
    pub received_at: u64,
}
