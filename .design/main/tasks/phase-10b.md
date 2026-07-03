---
phase: 10
name: "Advanced Office Features (L2) — Dependent Wave"
status: Todo
subsystem: "crates/core (voice, deliberation, version_control, inner_monologue, lookahead, global_orch) + apps/desktop + packages/ui (automation-canvas)"
requires:
  - "Phase 10 foundational wave: office-control, acp, navigation, automation-pipeline (tasks/phase-10.md)"
  - "Phase 4/5/6: agent-session, inbox, orchestration, scheduler, execution-workspace, quality-pipeline, kanban, role-catalog"
  - "Phase 8: app-ui (Tauri v2 + React 19)"
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 10 Tasks — Advanced Office Features (L2), Dependent Wave

**Phase:** 10 (workbook B)
**Status:** Todo
**Strategic Goal:** Implement the 7 dependent-wave Phase-10 L2 specs whose intra-phase dependencies are satisfied by the foundational wave (phase-10.md). Runs after the foundational wave. Backend logic in `crates/core`; automation-canvas is presentation-only over the automation engine (INV-2, AC-3).

## Atomic Checklist

- [ ] [T-10F01] Automation Canvas — three-panel projection + node rendering + live trace
- [ ] [T-10F02] Automation Canvas — explicit editing + dev-run request + observer scope view
- [ ] [T-10FT01] Validation — AC-1…AC-8 + presentation-only (fallow audit)
- [ ] [T-10G01] Voice Input — capture + VAD + transcription engine + model lifecycle
- [ ] [T-10G02] Voice Input — transform + review overlay + clipboard-safe injection + history
- [ ] [T-10GT01] Validation — VI-1…VI-10 (on-device residency, cancel-no-side-effect)
- [ ] [T-10H01] Deliberation — round runner + parallel arguments + synthesis + immutable log
- [ ] [T-10HT01] Validation — DL-1…DL-5
- [ ] [T-10I01] Version Control — VSA over worktrees + quality-gated commit + card boundary
- [ ] [T-10I02] Version Control — role authority table + branch strategies + Conventional Commits
- [ ] [T-10IT01] Validation — VC-1…VC-6
- [ ] [T-10J01] Inner Monologue — heartbeat cycle + snapshot + intentions + log-before-dispatch
- [ ] [T-10JT01] Validation — IM-1…IM-5
- [ ] [T-10K01] Lookahead — trigger detector + no-real-exec simulator + conclusion dispatcher + log
- [ ] [T-10KT01] Validation — LP-1…LP-6
- [ ] [T-10L01] Global Orchestration — aggregate view + ACP relay routing + phase-awareness
- [ ] [T-10L02] Global Orchestration — building escalation + cross-office deliberation
- [ ] [T-10LT01] Validation — GO-1…GO-6

## Detailed Tracking

### Track F — Automation Canvas (`apps/desktop` + `packages/ui`, presentation-only)

Depends on automation-pipeline (foundational). Projection only — no execution (AC-3).

#### [T-10F01] Three-panel projection + node rendering + live trace
- **Spec:** l2-automation-canvas.md §4.1, §4.2, §4.4 (AC-1, 3, 4, 6)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `pnpm -C apps/desktop test` green — graph renders from the engine registry; an injected `EngineEvent` auto-refreshes (AC-1); live node state + traces come only from a mocked AuditProvider stream showing descriptors (AC-4); engine-offline shows staleness badge (AC-6).
- **Notes:** Serialization layer over the canonical pipeline def; no private representation.

#### [T-10F02] Explicit editing + dev-run request + observer scope view
- **Spec:** l2-automation-canvas.md §4.3, §4.4, §4.5 (AC-2, 5, 7, 8)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `pnpm -C apps/desktop test` green — implicit pipelines read-only + convert-to-explicit creates a new def (AC-2); save persists to a mocked local store, never egresses (AC-5); pin+partial-run issues an engine dev-run request (asserted via a mock bridge, canvas never executes — AC-7); observer node shows kind badge + dashed scope overlay (AC-8).
- **Notes:** Node palette driven by the engine node registry.

#### [T-10FT01] Validation — Automation Canvas
- **Goal:** Verify AC-1…AC-8 + presentation-only. **Method:** `fallow audit --changed-since <base>` clean; component tests assert projection fidelity + no execution path. **Status:** Todo

### Track G — Voice Input (`crates/core` pipeline + shell overlay)

Deps technology-stack + security (Done).

#### [T-10G01] Capture + VAD + transcription engine + model lifecycle
- **Spec:** l2-voice-input.md §4.1, §4.2, §4.3 (VI-1, 3, 7, 8)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus voice::` green — cpal 16 kHz capture gated on explicit activation (VI-3); ONNX VAD segments; `TranscriptionEngine` trait swaps engines with no network path (VI-1, VI-7); model acquire/load/idle-unload reuses model-runtime + content-addressed store (VI-8).
- **Notes:** Bundled default model works offline first-run.

#### [T-10G02] Transform + review overlay + injection + history
- **Spec:** l2-voice-input.md §4.4 + §4.1 (VI-2, 4, 5, 6, 9, 10)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus voice::` + `pnpm -C apps/desktop test` green — pipeline halts at review before inject (VI-2); recording indicator renders full duration (VI-4); cancel discards audio+transcript, writes nothing incl. history (VI-5); transform off by default, remote LM consent-gated through egress (VI-6); clipboard-safe paste saves/restores prior clipboard (VI-10); confirmed-only history (VI-9).
- **Notes:** Review overlay is a system-level layer.

#### [T-10GT01] Validation — Voice Input
- **Goal:** Verify VI-1…VI-10. **Method:** `cargo test -p cronus voice` — assert no egress except opt-in transform; cancelled recordings leave zero trace. **Status:** Todo

### Track H — Deliberation (`crates/core`)

Deps orchestration + inbox (Done) + navigation (foundational).

#### [T-10H01] Round runner + parallel arguments + synthesis + immutable log
- **Spec:** l2-deliberation.md §4.1, §4.2, §4.3 (DL-1…DL-5)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus deliberation::` green — arguments dispatch as a parallel wave with no cross-read (DL-1); participant selection maximizes specialty diversity (DL-2); orchestrator synthesizes, no vote path (DL-3); log row is append-only, no update/delete (DL-4); over-budget argument truncated with marker (DL-5).
- **Notes:** Reuses orchestration wave execution + inbox store (`deliberation_round` type).

#### [T-10HT01] Validation — Deliberation
- **Goal:** Verify DL-1…DL-5. **Method:** `cargo test -p cronus deliberation`. **Status:** Todo

### Track I — Version Control (`crates/core`)

Deps execution-workspace + quality-pipeline + kanban-board (Done).

#### [T-10I01] VSA over worktrees + quality-gated commit + card boundary
- **Spec:** l2-version-control.md §4.1 (VC-2, 3, 4)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus version_control::` green — quality gate runs before commit; a fail discards the worktree with no partial flush (VC-4); commit binds exactly one card + mandatory footer (VC-3); a bypassed gate is rejected (VC-2).
- **Notes:** VSA is the execution-workspace worktree + commit policy — no new worktree code.

#### [T-10I02] Role authority + branch strategies + Conventional Commits
- **Spec:** l2-version-control.md §4.2, §4.3, §4.4 (VC-1, 5, 6)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus version_control::` green — the orchestration guard rejects an unauthorized commit/push/merge per the authority table (VC-1); no push targets main/trunk — branch+PR forced unconditionally (VC-5); commit-message generator emits a Conventional Commits string with the card footer.
- **Notes:** Authority check lives in the orchestration delegation guard.

#### [T-10IT01] Validation — Version Control
- **Goal:** Verify VC-1…VC-6. **Method:** `cargo test -p cronus version_control`. **Status:** Todo

### Track J — Inner Monologue (`crates/core`)

Deps scheduler + inbox + agent-session (Done) + navigation (foundational).

#### [T-10J01] Heartbeat cycle + snapshot + intentions + log-before-dispatch
- **Spec:** l2-inner-monologue.md §4.1, §4.2, §4.4, §4.5 (IM-1…IM-5)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus inner_monologue::` green — cycle fires only Active/Idle + foreground-idle (IM-1); every intention (incl. NoAction) logged before dispatch, dispatcher refuses an unlogged intention (IM-2); over-budget concludes with `truncated=true` (IM-3); intentions route through subsystem APIs, no direct store write (IM-4); Pulse-subsystem pause suppresses the cycle (IM-5).
- **Notes:** Pulse log reuses the inbox store (`pulse_monologue` type); prompt is a nodus step.

#### [T-10JT01] Validation — Inner Monologue
- **Goal:** Verify IM-1…IM-5. **Method:** `cargo test -p cronus inner_monologue`. **Status:** Todo

### Track K — Lookahead Planning (`crates/core`)

Deps orchestration + kanban-board + execution-workspace (Done).

#### [T-10K01] Trigger detector + no-real-exec simulator + conclusion dispatcher + log
- **Spec:** l2-lookahead-planning.md §4.1, §4.2, §4.3 (LP-1…LP-6)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus lookahead::` green — only catalog actions trigger (LP-1); the simulator issues zero real tool calls/writes (LP-2); depth+token budget hard-enforced (LP-3); a conclusion exists before commit (LP-4); BUDGET_EXHAUSTED routes to the ORC-9 gate, never silent proceed (LP-5); every conclusion appends to the decision log before commit (LP-6).
- **Notes:** Simulator reuses orchestrator context model + codegraph static analysis.

#### [T-10KT01] Validation — Lookahead
- **Goal:** Verify LP-1…LP-6. **Method:** `cargo test -p cronus lookahead`. **Status:** Todo

### Track L — Global Orchestration (`crates/core`, home manager)

Deps orchestration (Done) + acp + office-control (foundational) + deliberation (Track H).

#### [T-10L01] Aggregate view + ACP relay routing + phase-awareness
- **Spec:** l2-global-orchestration.md §4.1, §4.2, §4.3 (GO-1, 3, 4, 5)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus global_orch::` green — one coordinator; the aggregate view builds from subscribed OfficeState events, read-only toward offices (GO-4); routing decides target over the ACP relay inspecting envelope only (GO-5); a new-component card is annotated with mandatory phase concerns as non-optional acceptance criteria (GO-3).
- **Notes:** Phase structure read from machine-readable phase frontmatter.

#### [T-10L02] Building escalation + cross-office deliberation
- **Spec:** l2-global-orchestration.md §4.4 (GO-2, 6)
- **Status:** Todo · **Assignment:** Agent
- **Verify:** `cargo test -p cronus global_orch::` green — the coordinator cannot cancel/re-delegate an office's active work (GO-2, no such path); an escalation resolves directly, requests a cross-office deliberation round, or escalates to HITL (GO-6).
- **Notes:** Cross-office deliberation reuses Track H.

#### [T-10LT01] Validation — Global Orchestration
- **Goal:** Verify GO-1…GO-6. **Method:** `cargo test -p cronus global_orch`. **Status:** Todo

## Notes

- **Run order**: this workbook runs after the foundational wave (phase-10.md). Track L gates on Track H (deliberation) + the foundational acp/office-control. Tracks F/G/H/I/J/K are otherwise independent.
- **Execution mode**: Parallel (C3). Suggested: G/H/I/J/K in parallel (independent core), then F (canvas, needs the engine live) and L (needs deliberation + foundational).
- **Domain-logic-first**: follow the Phase-9 precedent — implement each algorithm against seeded/mock state; defer real OS/network integration (live audio devices, real cross-office transport) with a documented per-task note.
