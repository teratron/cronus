---
phase: 9
name: "Operational Hardening"
status: Todo
subsystem: "crates/core, crates/cli"
requires:
  - "security baseline: secret store, redaction, egress gate (Phase 1)"
  - "memory store + agent session + learning loop (Phases 4-5)"
  - "scheduler + extension registry (Phase 5)"
  - "CLI command framework (Phase 3) for new verbs"
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
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

- [ ] [T-9A01] Sandbox policy: deny-by-default egress entries + binary allowlists, isolation tiers (restricted/balanced/open), preset catalog, PolicyContext, access-failure classification
- [ ] [T-9A02] Multi-user auth: bcrypt password store, session tokens, TOTP 2FA, privilege map with admin promote/demote, reserved sentinel usernames

Track B — Self-Healing & Config (l2-doctor, l2-config-hotreload)

- [ ] [T-9B01] Doctor: check categories (state integrity, stuck work, config validity, store consistency, disk pressure), safe auto-repair + escalation split, `doctor` command
- [ ] [T-9B02] Config hot-reload: file watcher with bounded backoff + polling fallback, prefix-keyed reload plan, subsystem action dispatch, skills snapshot invalidation

Track C — Data Safety (l2-backup, l2-agent-migration)

- [ ] [T-9C01] Backup & restore: state-tier archive minus secrets/cache, restore-by-copy, list/create/restore verbs, optional scheduled backups
- [ ] [T-9C02] Agent migration: migration manifest v1, two-layer import (archives vs memory candidates), staged dry-run-first apply, source adapters

Track D — Reporting & Improvement (l2-github-issue, l2-self-improvement, l1-telemetry)

- [ ] [T-9D01] GitHub issue reporting: consent + scrub + dedup pipeline, BLAKE3 normalized error fingerprinting, prior-resolution surfacing
- [ ] [T-9D02] Self-improvement: calibration buckets, mistake log, should-have-asked, ask-backs (at-most-one-pending per project), reasoning templates, brief surface
- [ ] [T-9D03] Telemetry: opt-in program-data-only metrics emission (implementation light)

Track T — Validation

- [ ] [T-9T01] Cross-subsystem hardening validation: secret-safety integration (seeded secret never reaches a backup archive or scrubbed report), consent gates verified, workspace-wide gates green

## Detailed Tracking

### [T-9A01] Sandbox policy

- **Spec:** l2-sandbox-policy.md; l1-security.md (SEC), l1-execution-sandbox.md (deny-by-default axes)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — an unlisted egress target is denied by default while a named entry/binary-allowlist match passes; the three tier presets (restricted/balanced/open) resolve distinct policies; `PolicyContext` classifies an access failure into its typed class. Clippy `-D warnings` + fmt clean.
- **Handoff:** The policy layer the execution workspace and doctor consult before granting access.
- **Notes:** Extends the Phase 1 egress gate — reuse it, never a parallel deny list. Policy is data (config-driven), not code branches.

### [T-9A02] Multi-user auth

- **Spec:** l2-multi-user-auth.md; l1-security.md, l1-groups.md (admin privilege semantics)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — password hash/verify round-trip (bcrypt, no plaintext stored); session token issue → validate → expiry; TOTP enrollment + code verification; privilege map admin promote/demote; a reserved sentinel username is rejected at registration. Clippy + fmt clean.
- **Handoff:** The identity layer resource-sharing principals and admin surfaces authenticate against.
- **Notes:** Secrets (hashes, TOTP seeds) live in the state tier under STO-6 exclusion rules; never logged, never exported.

### [T-9B01] Doctor

- **Spec:** l2-doctor.md; l1-doctor.md (HEAL-1…6)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — each check category returns typed findings against a seeded broken state (orphaned session, stuck card, invalid config, store inconsistency); a safe deterministic problem is auto-repaired and the action logged (HEAL-2/HEAL-5); a risky finding is escalated, not applied (HEAL-3); diagnosis mutates nothing (HEAL-4). `cronus doctor` verb wired per grammar. Clippy + fmt clean.
- **Handoff:** The self-healing layer config-hotreload and ops workflows depend on.
- **Notes:** Checks are a data-driven catalog; repairs are reversible where possible; crash-recovery consistency check composes the run-marker semantics of the crash-recovery concept.

### [T-9B02] Config hot-reload

- **Spec:** l2-config-hotreload.md
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — a failing file watcher degrades through bounded backoff to polling (never a busy loop, never silent death); a changed config key maps through the prefix-keyed reload plan to exactly its subsystem action; a skills-directory change invalidates the skills snapshot. Clippy + fmt clean.
- **Handoff:** Live config responsiveness for every configurable subsystem.
- **Notes:** Depends on T-9B01 (doctor owns the watcher-health check). Reload actions are dispatch data, not hardcoded branches.

### [T-9C01] Backup & restore

- **Spec:** l2-backup.md; l1-storage-model.md (STO-1/2/5/6/7)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — a backup archive of a seeded state tier excludes `.env`/secrets and regenerable cache (STO-6) and is self-contained; restore into a clean temp dir reconstitutes a resumable state tier (STO-7 round-trip); `backup`/`backup list`/`restore` verbs per grammar; optional scheduled backup registers via the scheduler. Clippy + fmt clean.
- **Handoff:** The archival layer agent-migration imports from.
- **Notes:** Restore never merges silently — it places a tier; cross-installation merge belongs to migration (T-9C02).

### [T-9C02] Agent migration

- **Spec:** l2-agent-migration.md; l1-storage-model.md (STO-9 merge-by-identity)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — migration manifest v1 parses and validates (unknown versions rejected loudly); the two-layer import splits archives vs memory candidates; staged apply is dry-run-first (no writes) then applies reversibly with a pre-write backup; an incoming record merges by identity, never blind-clobbers (STO-9). Clippy + fmt clean.
- **Handoff:** Cross-installation agent transfer over the backup layer.
- **Notes:** Depends on T-9C01. Source adapters are a seam (trait), one built-in adapter suffices for the phase.

### [T-9D01] GitHub issue reporting

- **Spec:** l2-github-issue.md; l1-error-reporting.md (ERR-1…5), l1-issue-reporting.md (shared pipeline)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — the scrub stage removes seeded secrets from a report body (reuses core redaction, never re-implements); BLAKE3 normalized fingerprint dedups the same error across episodes; a prior resolution is surfaced on a fingerprint match; the consent gate blocks all egress absent explicit opt-in (ERR-1) and the payload is previewable before send. Clippy + fmt clean.
- **Handoff:** The reporting pipeline self-improvement reads resolved-issue history from.
- **Notes:** Filing transport is a seam; tests run against a mock transport — no network in tests.

### [T-9D02] Self-improvement

- **Spec:** l2-self-improvement.md
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — calibration buckets update from outcome events (overconfidence metric, verified-ratio warning); mistake log records and queries by project/category/files; should-have-asked captures trigger → question → answer; ask-backs enforce at-most-one-pending per project (unique-index semantics); reasoning templates resolve by task_type + domain; the brief surface joins available signals at task start. Clippy + fmt clean.
- **Handoff:** Closes the learning loop over memory-store + learning-loop + reporting history.
- **Notes:** Largest task of the phase — split into T-9D02.1/.2 at run time if it overruns (see Planner Audit). Depends on T-9D01 for resolved-issue signals.

### [T-9D03] Telemetry

- **Spec:** l1-telemetry.md (TEL invariants; implementation light)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — no telemetry event is emitted while opt-in is absent (default off); an emitted payload contains program data only (counters/durations/versions — no user content, no paths with user data, no secrets); opt-out drops queued events. Clippy + fmt clean.
- **Handoff:** Completes the privacy-first observability story.
- **Notes:** Light implementation: event shapes + consent gate + local queue; transport is a seam.

### [T-9T01] Validation — cross-subsystem hardening

- **Goal:** Prove the hardening layer holds as a whole: nothing secret leaves, nothing egresses without consent, all gates green.
- **Method:** Integration tests — seed a secret into the state tier, then assert it appears in no backup archive, no scrubbed report body, and no telemetry payload; assert consent gates (report + telemetry) block egress by default. Workspace-wide `cargo test` 0 failures; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --check` clean.
- **Status:** Todo
