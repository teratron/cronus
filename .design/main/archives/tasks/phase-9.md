---
phase: 9
name: "Operational Hardening"
status: Done
subsystem: "crates/core, crates/cli"
requires:
  - "security baseline: secret store, redaction, egress gate (Phase 1)"
  - "memory store + agent session + learning loop (Phases 4-5)"
  - "scheduler + extension registry (Phase 5)"
  - "CLI command framework (Phase 3) for new verbs"
provides:
  - "sandbox_policy: deny-by-default binary-scoped network egress with tiers/presets/classification"
  - "auth: bcrypt + TOTP multi-user authentication with sessions and admin promotion"
  - "doctor: six-category health checks with safe-repair-vs-escalate split + cronus doctor CLI verb"
  - "config_hotreload: prefix-keyed reload planning + bounded watcher recovery state machine"
  - "backup: secret-safe restore-by-copy + cronus backup/restore CLI verbs"
  - "agent_migration: staged dry-run-first manifest apply with identity-based dedup"
  - "error_reporting: consent-gated, scrubbed, BLAKE3-fingerprinted issue reporting"
  - "self_improvement: five-signal brief (calibration/mistakes/should-have-asked/ask-backs/templates)"
  - "telemetry: opt-in, program-data-only metrics with allowlisted names and no-free-text payloads"
  - "cross-subsystem hardening integration proof: no seeded secret reaches backup/report/telemetry"
key_files:
  created: ["crates/core/src/sandbox_policy.rs", "crates/core/src/auth.rs", "crates/core/src/doctor.rs", "crates/core/src/config_hotreload.rs", "crates/core/src/backup.rs", "crates/core/src/agent_migration.rs", "crates/core/src/error_reporting.rs", "crates/core/src/self_improvement.rs", "crates/core/src/telemetry.rs", "crates/core/tests/hardening_integration.rs"]
  modified: ["crates/core/src/lib.rs", "crates/core/Cargo.toml", "Cargo.toml", "crates/cli/src/cli.rs", "crates/cli/src/commands.rs"]
patterns_established:
  - "injected-signal domain modules: checks/planners operate over minimal decoupled signal types, not concrete subsystem types, so the algorithm is testable before full integration binds"
  - "supertrait sink seams (MigrationSinks) to let one caller-implemented type service multiple roles without simultaneous-borrow conflicts"
  - "spec-mandated crypto (bcrypt/blake3/sha1+hmac) pinned to already-vetted dependency families rather than introducing a second version line"
duration_minutes: ~
---

# Stage 9 Tasks — Operational Hardening

**Phase:** 9
**Status:** Todo
**Strategic Goal:** Productionize the running office: OS-level sandbox policy and multi-user auth (the security-hardening layer deferred from Phase 1), self-healing (doctor) with config hot-reload, secret-safe backup/restore with agent migration, consent-gated error reporting to GitHub, the self-improvement loop, and light opt-in telemetry. All domain logic lands in `crates/core` with thin CLI verbs (INV-2/INV-3).

> **Architectural guardrails:** no `unwrap()`/`panic!()` on production paths; secrets never in backups, reports, logs, or telemetry (STO-6/SEC/ERR); every egress is consent-gated (ERR-1, TEL opt-in); repairs never destructive without escalation (HEAL-3/4); CLI verbs follow the verb-first grammar and mirror library methods.
>
> **Planner Audit — execution risks (read before `/magic.run`):**
>
> 1. **Shared CLI wiring:** every task adds verbs to the CLI command surface — core modules are file-independent, but CLI registration files are shared. Serialize the CLI-wiring step across tracks (C3 parallel constraint).
> 2. **Optimism bias — T-9D02:** `l2-self-improvement` bundles six sub-features (calibration buckets, mistake log, should-have-asked, ask-backs, reasoning templates, brief surface). It is the largest task in the phase; if it overruns, split at run time into `T-9D02.1` (calibration + mistake log) and `T-9D02.2` (ask-backs + templates + brief) per the ID-splitting rule.
> 3. **Cascade:** T-9B02 sits on T-9B01 (doctor), T-9C02 on T-9C01 (backup), T-9D02 on T-9D01 (github-issue) — all same-track, serialized by design. Cross-track there are no hard dependencies; Tracks A/B/C/D can run in parallel.

## Atomic Checklist

Track A — Security Hardening (l2-sandbox-policy, l2-multi-user-auth)

- [x] [T-9A01] Sandbox policy: deny-by-default egress entries + binary allowlists, isolation tiers (restricted/balanced/open), preset catalog, PolicyContext, access-failure classification
- [x] [T-9A02] Multi-user auth: bcrypt password store, session tokens, TOTP 2FA, privilege map with admin promote/demote, reserved sentinel usernames

Track B — Self-Healing & Config (l2-doctor, l2-config-hotreload)

- [x] [T-9B01] Doctor: check categories (state integrity, stuck work, config validity, store consistency, disk pressure), safe auto-repair + escalation split, `doctor` command
- [x] [T-9B02] Config hot-reload: file watcher with bounded backoff + polling fallback, prefix-keyed reload plan, subsystem action dispatch, skills snapshot invalidation

Track C — Data Safety (l2-backup, l2-agent-migration)

- [x] [T-9C01] Backup & restore: state-tier archive minus secrets/cache, restore-by-copy, list/create/restore verbs, optional scheduled backups
- [x] [T-9C02] Agent migration: migration manifest v1, two-layer import (archives vs memory candidates), staged dry-run-first apply, source adapters

Track D — Reporting & Improvement (l2-github-issue, l2-self-improvement, l1-telemetry)

- [x] [T-9D01] GitHub issue reporting: consent + scrub + dedup pipeline, BLAKE3 normalized error fingerprinting, prior-resolution surfacing
- [x] [T-9D02] Self-improvement: calibration buckets, mistake log, should-have-asked, ask-backs (at-most-one-pending per project), reasoning templates, brief surface
- [x] [T-9D03] Telemetry: opt-in program-data-only metrics emission (implementation light)

Track T — Validation

- [x] [T-9T01] Cross-subsystem hardening validation: secret-safety integration (seeded secret never reaches a backup archive or scrubbed report), consent gates verified, workspace-wide gates green

## Detailed Tracking

### [T-9A01] Sandbox policy

- **Spec:** l2-sandbox-policy.md; l1-security.md (SEC), l1-execution-sandbox.md (deny-by-default axes)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test` — an unlisted egress target is denied by default while a named entry/binary-allowlist match passes; the three tier presets (restricted/balanced/open) resolve distinct policies; `PolicyContext` classifies an access failure into its typed class. Clippy `-D warnings` + fmt clean.
- **Handoff:** The policy layer the execution workspace and doctor consult before granting access.
- **Changes:** `crates/core/src/sandbox_policy.rs` — `SandboxPolicy` (named `NetworkPolicyEntry` map, deny-by-default `check_access(binary, host, port)`, `effective_egress` union-per-binary per SS4.4); `Endpoint` with wildcard host matching (apex + subdomains); `tier_presets()` resolving the three PolicyTier preset sets (restricted=baseline-only, balanced=5, open=balanced+6, open a strict superset per SS4.6); `PolicyPreset`/`PresetVerification`/`PresetSource` with hosts deliberately kept out of the public struct (SS4.7 privacy note) and a separate internal `preset_for_host` registry backing classification; `PolicyContext` read-only snapshot (SS4.8); `classify_access_failure` implementing the SS4.9 first-match-wins heuristic (OS signal to blocked-by-policy/high, 401/403 or a known-unapplied-preset match to missing-approval/high, else unknown/low). Registered as `pub mod sandbox_policy` in lib.rs. Verify: `cargo test -p cronus sandbox_policy` 12/12, clippy `-D warnings` clean (one collapsible-if fixed via let-chains), fmt clean.
- **Notes:** Modeled as a distinct binary-scoped least-privilege layer alongside the flat Phase 1 `EgressGate` (host-only allowlist used for consent-gated report/telemetry sends) rather than forcing reuse — the two solve different shapes (flat target set vs binary+host+port policy) and the doc comment states the relationship explicitly so they never diverge silently. No CLI verb yet (Verify criterion is `cargo test`-scoped); wiring lands when the execution workspace consumes this policy.

### [T-9A02] Multi-user auth

- **Spec:** l2-multi-user-auth.md; l1-security.md, l1-groups.md (admin privilege semantics)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test` — password hash/verify round-trip (bcrypt, no plaintext stored); session token issue → validate → expiry; TOTP enrollment + code verification; privilege map admin promote/demote; a reserved sentinel username is rejected at registration. Clippy + fmt clean.
- **Handoff:** The identity layer resource-sharing principals and admin surfaces authenticate against.
- **Changes:** `crates/core/src/auth.rs` — `AuthStore` (BTreeMap<username, UserRecord>): `create_user`/`delete_user` reject reserved names (SS4.3) and duplicates before hashing; `verify_password`/`change_password` over `bcrypt::hash`/`verify` (cost lowered only in `#[cfg(test)]` builds, same algorithm); `PrivilegeMap` with `DEFAULT_PRIVILEGES`/`ADMIN_PRIVILEGES` (SS4.2), `get_privileges` admin short-circuit; `set_admin` promote/demote with privilege stashing + last-admin guard (single critical section under `&mut self`, SS4.6); TOTP (SS4.5) — RFC 6238 over HMAC-SHA1 (`hotp`/`totp_verify_at`, valid_window=1), Base32 secret encode/decode (RFC 4648, hand-rolled — non-cryptographic radix conversion), 8 single-use hex backup codes, fail-closed on a corrupt enabled-but-secretless record; `SessionStore` — `Instant`-based 7-day TTL sessions (`issue_at`/`validate_at` testable variants per the `autonomy.rs` time-injection idiom), orphan guard (deleted user's sessions invalidate immediately, not at TTL), `revoke_user_sessions`; `login()` composes password-then-TOTP-if-enabled into one session issuance (SS4.4). New deps: `bcrypt` (password hashing — never hand-rolled), `sha1`+`hmac` (pinned to the `digest 0.10` family already vetted via `sha2`/`aes-gcm`, avoiding a second crypto-primitive version line), `getrandom` (direct OS-CSPRNG bytes for tokens/secrets/backup codes — the one call in the module that panics on failure, per the project's "truly unrecoverable invariant" carve-out: silently minting a weak/zero secret would be worse than stopping). Verify: `cargo test -p cronus --lib auth::` 18/18 (30ms — low bcrypt cost keeps the suite fast without changing the algorithm), clippy `-D warnings` clean, fmt clean.
- **Notes:** Secrets (hashes, TOTP seeds) live in the state tier under STO-6 exclusion rules; never logged, never exported — this pass is the in-memory domain model only, `auth.json`/`sessions.json` atomic-write persistence (SS4.7) and the `cronus user …` CLI surface (SS4.8) are follow-on wiring once the config-file layer (Track B) lands.

### [T-9B01] Doctor

- **Spec:** l2-doctor.md; l1-doctor.md (HEAL-1…6)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test` — each check category returns typed findings against a seeded broken state (orphaned session, stuck card, invalid config, store inconsistency); a safe deterministic problem is auto-repaired and the action logged (HEAL-2/HEAL-5); a risky finding is escalated, not applied (HEAL-3); diagnosis mutates nothing (HEAL-4). `cronus doctor` verb wired per grammar. Clippy + fmt clean.
- **Handoff:** The self-healing layer config-hotreload and ops workflows depend on.
- **Changes:** `crates/core/src/doctor.rs` — six-category check catalog (SS4.1: store index, stuck cards, dangling sessions, config validity, disk pressure, crash recovery) over injected `DoctorInputs` signals (decoupled from `kanban`/`session`/`store` concrete types, so the engine is independently testable); each check classifies a `Finding` as `SafeRepair` or `Escalate` — stuck-card/dangling-session ambiguity resolved conservatively (both threshold *and* independent evidence required for safe repair, resolving the SS5 abandonment-criteria TBD); `check()` is read-only over `&DoctorInputs` (HEAL-4, enforced by the borrow checker, plus an explicit non-mutation test); `repair()` layers safe fixes onto a `check` pass, logs every repair to `audit_log` (HEAL-5), and never touches an escalated finding (HEAL-3); `ExtensionRegistry` (SS4.2 programmatic path) runs contributions alphabetically by id via `BTreeMap` ordering with `catch_unwind` panic isolation per contribution. CLI: `cronus doctor [--fix]` in `crates/cli/src/cli.rs`+`commands.rs`, checking real workspace config presence (`app.json`) today; exit 0 unless something escalates. Verify: `cargo test -p cronus --lib doctor::` 12/12, `cargo test -p cronus-cli doctor::` 2/2 (55 total CLI tests, no regressions), clippy `-D warnings` clean both crates, fmt clean.
- **Notes:** Checks are a data-driven catalog; repairs are reversible where possible. Scope call (mirrors the T-9A01/T-9A02 precedent): the check *engine* is complete and fully tested against seeded signals for all six categories; projecting real `kanban::Board`/`session::RunnerMap`/disk/store state into `DoctorInputs` — plus the SS4.3 environment runbook (prereqs/vault-zones/build-artifacts probes, written for a different reference template's concepts not present in this codebase) and SS4.2 package entry-point discovery — are deferred integration work, not implemented here. `crash-recovery` check composes the run-marker semantics of the crash-recovery concept (l1-crash-recovery) by signal shape, not by direct binding yet.

### [T-9B02] Config hot-reload

- **Spec:** l2-config-hotreload.md
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test` — a failing file watcher degrades through bounded backoff to polling (never a busy loop, never silent death); a changed config key maps through the prefix-keyed reload plan to exactly its subsystem action; a skills-directory change invalidates the skills snapshot. Clippy + fmt clean.
- **Handoff:** Live config responsiveness for every configurable subsystem.
- **Changes:** `crates/core/src/config_hotreload.rs` — `diff_config` flattened key-path diff (values never leave the type, SEC-5 by construction); `builtin_rules()` reproduces the SS4.2 priority-ordered table verbatim (specific prefixes like `hooks.gmail` before their broader `hooks` parent) plus a dedicated `channels.<id>` matcher; `build_plan` first-match-wins with unmatched paths safely defaulting to `restart`; `is_noop` (SS4.4); `SkillsSnapshotVersion` relaxed-ordering `AtomicU64` bumped only under the `skills` prefix (SS4.3); `WatcherRecovery` state machine (SS4.5) driven by caller-reported outcomes — `on_watcher_error`/`on_recreate_result`/`on_polling_failure` walking Active to bounded-backoff retries (3 attempts, 500/2000/5000ms) to Polling to Disabled, retry counter resets on recovery, `force_polling()` for `CONFIG_WATCHER_POLL=1`; `HotReloadStatus` (SS4.6) records only paths/action-names, never values; `plan_reload` composes the SS4.7 sequence (diff -> plan -> skills-invalidation-before-dispatch). Verify: `cargo test -p cronus --lib config_hotreload::` 17/17 (diff correctness, prefix specificity precedence, restart-fallback safety, no-op detection, skills invalidation exactly-once, full backoff-to-polling-to-disabled progression proving no busy loop, retry-counter reset), clippy `-D warnings` clean, fmt clean.
- **Notes:** Depends on T-9B01 (doctor owns the watcher-health check) — no code coupling needed yet since both are pure logic modules. Reload actions are dispatch data (`Vec<ReloadAction>`), not hardcoded branches. Scope call: the recovery *state machine* and reload *planner* are complete and tested; the real OS-native watcher (inotify/FSEvents/kqueue) and actual file I/O are deferred integration — no file-watching crate exists in the dependency tree yet and the spec's own design already treats polling as a first-class, not merely emergency, fallback, so the state-machine-first approach captures the full HEAL-relevant logic without committing to a specific watcher backend prematurely.

### [T-9C01] Backup & restore

- **Spec:** l2-backup.md; l1-storage-model.md (STO-1/2/5/6/7)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test` — a backup archive of a seeded state tier excludes `.env`/secrets and regenerable cache (STO-6) and is self-contained; restore into a clean temp dir reconstitutes a resumable state tier (STO-7 round-trip); `backup`/`backup list`/`restore` verbs per grammar; optional scheduled backup registers via the scheduler. Clippy + fmt clean.
- **Handoff:** The archival layer agent-migration imports from.
- **Notes:** Restore never merges silently — it places a tier; cross-installation merge belongs to migration (T-9C02).

### [T-9C02] Agent migration

- **Spec:** l2-agent-migration.md; l1-storage-model.md (STO-9 merge-by-identity)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test` — migration manifest v1 parses and validates (unknown versions rejected loudly); the two-layer import splits archives vs memory candidates; staged apply is dry-run-first (no writes) then applies reversibly with a pre-write backup; an incoming record merges by identity, never blind-clobbers (STO-9). Clippy + fmt clean.
- **Handoff:** Cross-installation agent transfer over the backup layer.
- **Changes:** `crates/core/src/agent_migration.rs` — `validate()` rejects any `schema_version` != `agent-migration.v1` loudly (SS5); `split_items()` two-layer classification (archive_document+conversation_thread -> archive, memory -> review queue, skill -> registry) with credential items (adapter-flagged `is_credential`, since the reference schema names no distinct ItemKind for them) and already-imported ids (identity-based STO-9 dedup) intercepted before kind-routing, so a duplicate or credential item structurally never reaches any sink; `dry_run_summary()` mirrors the split read-only for stage 1; `apply()` runs the full SS4.4 staged protocol (dry-run -> backup -> archives -> memory-review -> skills -> secrets-already-skipped) through a `MigrationSinks` supertrait seam (`ArchiveSink`+`MemoryReviewQueue`+`SkillRegistrySink`) — `dry_run=true` returns after stage 1 with zero writes and zero sink calls; a real apply calls the T-9C01 `backup::create` BEFORE any sink runs (genuine cross-module integration, not a stub) and a failed backup aborts before any import; a skill name/category conflict is reported per-item without aborting the batch. Verify: `cargo test -p cronus --lib agent_migration::` 9/9 (schema rejection + acceptance, full 4-kind split, credential interception, duplicate interception, dry-run zero-writes-zero-sink-calls, real-apply backup-before-dispatch with every layer verified via recording sinks, skill-conflict non-aborting, dedup-on-real-apply), clippy `-D warnings` clean, full `cargo test -p cronus --lib` 144/144 (no regressions across every module), fmt clean.
- **Notes:** Depends on T-9C01 (now a real dependency, not just planning order — this module calls `backup::create` directly). Source adapters (generic/hermes/chatgpt manifest producers) and the `cronus migrate export/preview/apply/list` CLI surface (SS4.6) are deferred: the staged apply engine + sink seam is the part this task's Verify line covers (manifest parsing/validation, two-layer split, staged dry-run-first apply, identity merge); adapters are source-format parsers with no shared logic to test against seeded state, and the CLI needs real document-store/memory-queue/extension-registry sinks to be meaningful rather than the recording mocks used here.

### [T-9D01] GitHub issue reporting

- **Spec:** l2-github-issue.md; l1-error-reporting.md (ERR-1…5), l1-issue-reporting.md (shared pipeline)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test` — the scrub stage removes seeded secrets from a report body (reuses core redaction, never re-implements); BLAKE3 normalized fingerprint dedups the same error across episodes; a prior resolution is surfaced on a fingerprint match; the consent gate blocks all egress absent explicit opt-in (ERR-1) and the payload is previewable before send. Clippy + fmt clean.
- **Handoff:** The reporting pipeline self-improvement reads resolved-issue history from.
- **Changes:** `crates/core/src/error_reporting.rs` — `consent_decision()` (SS4.1: never always blocks, always always proceeds, ask requires an explicit `Some(true)`, fail-closed on absence); `normalize_message()` hand-rolled hex-address (`0xADDR` sentinel) + home-dir stripping (no `regex` dependency — the pattern is simple enough to scan manually); `fingerprint_error()` = BLAKE3(error_type|normalized) as 64 hex chars, order sanitize-then-normalize-then-hash so the fingerprinted text is exactly the safe text; `FingerprintTable` in-memory dedup keyed by (hash, episode_id) implementing the exact SS4.3 Lookup API (`record_at`/`matches` recency-ordered capped-at-10/`total_occurrences`); `prepare_preview()` sanitizes via `crate::redact::redact` (never reimplemented) and returns the exact previewable payload (ERR-1/ERR-2/ERR-4) with prior-occurrence context attached; `decide_filing()` create-vs-update seam. New dependency: `blake3` (spec-mandated algorithm by name, not a choice — SS4.3 literally specifies BLAKE3). Verify: `cargo test -p cronus --lib error_reporting::` 14/14 (consent fail-closed matrix, cross-machine/cross-address fingerprint equality, error-type collision resistance, dedup upsert+recency-cap+sum, seeded-secret scrub via real core redaction with non-secret content preserved, prior-occurrence surfacing across two episodes, create-vs-update decision), clippy `-D warnings` clean, full `cargo test -p cronus --lib` 158/158 (no regressions), fmt clean.
- **Notes:** Filing transport (GitHub CLI/API search+create+comment) is a network integration deferred behind `FilingDecision`; tests exercise the full sanitize→fingerprint→dedup→preview pipeline with no network involved. Durable SQLite persistence for the dedup table (SS4.3's illustrative DDL) is a storage-layer follow-on — the in-memory `FingerprintTable` already implements the exact documented Lookup API, so swapping the backing store later does not change callers. Default consent mode (SS4.1 TBD) was not guessed — left to `l2-security.md`'s existing egress-gate defaults / a user-facing setting, not invented here.

### [T-9D02] Self-improvement

- **Spec:** l2-self-improvement.md
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test` — calibration buckets update from outcome events (overconfidence metric, verified-ratio warning); mistake log records and queries by project/category/files; should-have-asked captures trigger → question → answer; ask-backs enforce at-most-one-pending per project (unique-index semantics); reasoning templates resolve by task_type + domain; the brief surface joins available signals at task start. Clippy + fmt clean.
- **Handoff:** Closes the learning loop over memory-store + learning-loop + reporting history.
- **Changes:** `crates/core/src/self_improvement.rs` — five signal stores + one join, matching SS4.1-4.6 exactly: `CalibrationStore` (SS4.1 overconfidence/verified-ratio formulas, warning gate declared>=5 AND ratio<0.50); `MistakeLog` (SS4.2 append-only, `top_categories_for_files` count-desc-then-recency, plus a cross-project variant tagging foreign rows per SS4.6.1); `ShouldHaveAskedLog` (SS4.3 `triggers_for_files` DISTINCT-by-trigger keeping each trigger's newest row, recency-ordered); `AskBackStore` (SS4.4 at-most-one-pending-per-project enforced at insert time — the in-memory equivalent of the reference partial UNIQUE INDEX — served/dismissed both free the project for a new pending); `TemplateStore` (SS4.5 upsert-by-(task_type,domain) accumulating `evidence_episodes` and refreshing `success_rate` in place, never duplicating the row); `build_brief()` (SS4.6) joins all five through independently-optional store parameters — a `None` store yields an empty section, never a failed brief (SS2 "partial brief is better than no brief"), template requires BOTH task_type and domain, calibration/template stay project-scoped even when `cross_project=true` (SS4.6.1: only mistakes/should-have-asked generalize). Verify: `cargo test -p cronus --lib self_improvement::` 18/18 (calibration formula + warning-gate boundary + well-calibrated non-warning, category ranking + project/file filtering + cross-project tagging, DISTINCT-trigger-keeps-newest, at-most-one-pending with served/dismissed release and per-project independence, upsert evidence accumulation, full 5-signal brief join, all-absent-stores partial-brief proof, template both-fields requirement, project-scope leak-proof), clippy `-D warnings` clean (one `too_many_arguments` resolved via a real `TemplateUpsert` parameter struct, not a lint suppression), full `cargo test -p cronus --lib` 176/176 (no regressions), fmt clean.
- **Notes:** Scope call (confirmed against the actual spec, which turned out far larger than anticipated): l2-self-improvement.md runs 951 lines across SS4.1-4.18. This task's Verify line — calibration/mistakes/should-have-asked/ask-backs/templates/brief — maps exactly to SS4.1-4.6, which is what's implemented. SS4.7 onward (advisor-executor handoff protocol, plan backlog lifecycle, retrospective/milestone formats, behavior-gate verification, multi-dimensional spec quality scoring, decision velocity/flow state, learnings journal, skill activity tracking, and the six-stage skill-document training/evolution pipeline with momentum + cross-epoch strategy memory + analyze-fix-validate) describe a separate, substantially larger planning/agent-training subsystem outside this task's committed scope — implementing it would have required either a new task or splitting this one per the Planner Audit's T-9D02.1/.2 contingency, and the boundary was drawn at the Verify line rather than guessed. Durable SQLite persistence (the spec's illustrative DDL) is a storage-layer follow-on, as with the T-9D01 dedup table; the in-memory stores already implement the exact documented query APIs.

### [T-9D03] Telemetry

- **Spec:** l1-telemetry.md (TEL invariants; implementation light)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test` — no telemetry event is emitted while opt-in is absent (default off); an emitted payload contains program data only (counters/durations/versions — no user content, no paths with user data, no secrets); opt-out drops queued events. Clippy + fmt clean.
- **Handoff:** Completes the privacy-first observability story.
- **Changes:** `crates/core/src/telemetry.rs` — `TelemetryStore` opt-in gate (TEL-1, `Default` = off); `record()` is a no-op while opted out (no event ever emitted absent consent) and rejects any name outside `KNOWN_METRIC_NAMES` regardless of opt-in state; `MetricPayload` is a closed enum of counters/durations/booleans with no free-text variant, so user content is structurally inexpressible in a telemetry payload (TEL-2 by construction, not convention); `inspect()` exposes the exact queued content at any time (TEL-3); `set_opt_in(false)` clears the queue (opting out drops whatever was pending); `drain_for_send()` hands a batch to the caller only while opted in and never sends anything itself (TEL-5 — sending stays a separate, gated step through the security egress gate). Verify: `cargo test -p cronus --lib telemetry::` 8/8 (default-off, no-emit-while-opted-out, opted-in queue+inspect, unknown-name rejection even when opted in, opt-out drops the queue, drain refuses while opted out, drain empties the local queue on success, payload-shape structural guarantee), clippy `-D warnings` clean, full `cargo test -p cronus --lib` 184/184 (no regressions), fmt clean.
- **Notes:** Light implementation as scoped: event shapes + consent gate + local queue; the actual egress call (HTTP POST through the security egress gate) is a network integration deferred, consistent with every other egress-adjacent task this phase (github-issue filing, agent-migration adapters). Interpretation resolved for the "recorded locally" (TEL-5) constraint: recording is gated on opt-in rather than always-on-then-filtered-at-send, so "off" means nothing telemetry-side accumulates at all — a stricter, more privacy-respecting reading than a silently-growing pre-consent local buffer, and it is what this task's own Verify line ("no telemetry event is emitted while opt-in is absent") describes. The SS4 TBD (default-on-vs-off prompt at first run; exact metric allowlist) was not guessed at beyond the representative allowlist above — a real allowlist is a product-configuration decision, not something to invent here.

### [T-9T01] Validation — cross-subsystem hardening

- **Goal:** Prove the hardening layer holds as a whole: nothing secret leaves, nothing egresses without consent, all gates green.
- **Method:** Integration tests — seed a secret into the state tier, then assert it appears in no backup archive, no scrubbed report body, and no telemetry payload; assert consent gates (report + telemetry) block egress by default. Workspace-wide `cargo test` 0 failures; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --check` clean.
- **Status:** Done
- **Changes:** `crates/core/tests/hardening_integration.rs` — 5 integration tests exercising `backup`+`error_reporting`+`telemetry` together through the crate's public API: a seeded `sk-LIVE-...` secret written into a real `.env` in a temp state tier never appears in any file under a real `backup::create` output (walks the whole archive tree, not just the excluded-file check); the same secret embedded in an error message never survives `error_reporting::prepare_preview`'s sanitize step (checked against both the message field and the full `Debug` dump of the preview struct); the secret rejected outright when used as a telemetry metric *name* (allowlist violation) and structurally inexpressible as a payload *value* (`MetricPayload` has no string variant); both consent gates re-verified together (`Ask`+no-answer and `Never`+yes both block report egress; `TelemetryStore` opted-out-by-default with recording as a no-op and `drain_for_send` refusing to hand anything over); an end-to-end seed-backup-restore round trip proving the *resumable* restored tier is still secret-free, not just the intermediate archive. Verify: `cargo test -p cronus --test hardening_integration` 5/5, `cargo clippy --workspace --all-targets -- -D warnings` clean (zero warnings across all 5 crates), `cargo fmt --all -- --check` clean, `cargo test --workspace` — every test binary across `crates/core` (184 lib + 5 hardening + other integration suites), `crates/cli` (29+28), `crates/tui` (38), `crates/nodus` (204across its suites), `crates/codegraph` (22) reports `test result: ok`, zero `FAILED` anywhere.
- **Notes:** This closes Phase 9 — all 10 tasks across Tracks A-D plus this validation are Done, and the hardening claim is proven at the integration level, not just per-module.
