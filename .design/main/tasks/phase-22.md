---
phase: 22
name: "Developer Office"
status: Todo
subsystem: "crates/domain/src/dev_office"
requires: [4, 9, 11]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 22 Tasks — Developer Office

**Phase:** 22
**Status:** Todo
**Strategic Goal:** Ship `l2-dev-office` — the self-hosting developer office (DVO-1…DVO-8): a conditional `WorkspaceKind::Developer` that materializes only in a genuine checkout of the canonical Cronus repo, only for a human-admitted developer, and is otherwise entirely absent. An orchestration layer over already-shipped subsystems; the phase builds only the gate, the tier resolution, the workspace-kind extension, and the composing wiring. Foundation-then-parallel; Track A gates B/C/D.

## Atomic Checklist

- [ ] [T-22A01] Domain gate + tier + `WorkspaceKind::Developer` + read-only `AdmissionReader` port
- [ ] [T-22B01] `repo_authenticity` facade — network-free canonical-repo check (DVO-2)
- [ ] [T-22C01] Admission on the human-write-only auth plane + reader impl (DVO-3)
- [ ] [T-22C02] Trigger-loaded module: load on `Elevated`, clean unload (DVO-4)
- [ ] [T-22D01] Workspace-kind registration + confinement/audit wiring (DVO-1/6/7)
- [ ] [T-22D02] `cronus dev status|admit|revoke` CLI surface
- [ ] [T-22T01] Validation sweep: DVO-1…DVO-8 acceptance

## Detailed Tracking

### [T-22A01] Domain gate + tier + workspace kind + reader port

- **Spec:** l2-dev-office.md §4.1 (tier split), §4.2 (activation gate), §4.4 (AdmissionReader); DVO-1, DVO-3, DVO-5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain dev_office` — the resolution table proven against fabricated inputs: `(Genuine, admitted)` → `Elevated`; a credential in a `NotCanonical`/`NotARepo` tree → `Absent`; `feedback_tier_enabled=false` (default) + not admitted → `Absent`; `feedback_tier_enabled=true` + not admitted → `Feedback`; and `WorkspaceKind::Developer` classifies as system-owned/non-deletable/not-project-creatable. `cargo clippy -p cronus-domain --all-targets -- -D warnings` clean.
- **Handoff:** Gates T-22B01/C01/C02/D01 — the gate + tier + kind types are the shared foundation.
- **Notes:** New `crates/domain/src/dev_office.rs`, pure/I/O-free: `RepoAuthenticity` (Genuine{upstream}/NotCanonical/NotARepo), `AdmissionTier` (Absent/Feedback/Elevated), `GateInputs`, `DevOfficeGate::resolve` (the exact match from §4.2 — `Elevated` reachable **only** via the `(Genuine, admitted)` arm, so a credential outside the canonical repo grants nothing). `WorkspaceKind::Developer` added to the workspace-kind enum (0..1, system-owned). **Load-bearing move (DVO-3):** the domain gate depends only on a read-only `trait AdmissionReader { fn is_admitted(&self) -> bool; }` — no mint/write method exists on it, so agent self-escalation is *unrepresentable* at the type level (the BA-4/OA-4 structural-enforcement pattern), not a bypassable runtime check. Tested against a scriptable fake reader.

### [T-22B01] `repo_authenticity` facade (DVO-2)

- **Spec:** l2-dev-office.md §4.3; DVO-2
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-core dev_office_gate` — a `.git/config` bound to the canonical upstream → `Genuine`; a non-canonical upstream → `NotCanonical`; no `.git` marker → `NotARepo`; an ambiguous/multi-remote config with no unambiguous canonical match → `NotCanonical` (fail-closed, AT-6). Exercised against real temporary `.git` fixtures; **no network call**.
- **Handoff:** Feeds the gate (A01) at the facade; independent of C once A01 lands.
- **Notes:** New `crates/core/src/dev_office_gate.rs`: `repo_authenticity(cwd) -> RepoAuthenticity` — `find_worktree_marker` (local `.git`), `read_bound_upstream` (parse `.git/config` remotes), `upstream_matches_canonical` against a compiled-in `CANONICAL_UPSTREAM` **build constant** (not user config — un-retargetable by an end user, §5 note 1). Conservative on ambiguity: `Genuine` only on an unambiguous match. Local filesystem read only.

### [T-22C01] Admission on the human-write-only plane (DVO-3)

- **Spec:** l2-dev-office.md §4.4; DVO-3, SEC-10
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — a `DeveloperAdmission` is minted only through the human-principal path and read back by the `AdmissionReader` impl; `is_admitted()` reflects mint then revoke; there is **no agent-reachable write path** — the domain gate holds only `&dyn AdmissionReader`, structurally unable to mint (asserted by the fact that the reader trait exposes no write method; the write API lives only on the human-principal auth surface).
- **Handoff:** Supplies the `admitted` input to the gate; C02 keys the module trigger off it.
- **Notes:** `DeveloperAdmission` on the `crates/auth-local` SEC-10 human-write-only plane (the background-activation-consent precedent — the same plane the agent has no write path to). **Disclosed execution question (surfaces here, not a design gap):** whether `auth-local` already exposes a generic human-write-only grant store the record can ride, or whether a small dedicated `DeveloperAdmission` record is added — an implementation reuse detail resolved at execution, flagged not guessed.

### [T-22C02] Trigger-loaded module: load/unload (DVO-4)

- **Spec:** l2-dev-office.md §4.5; DVO-4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — a transition into `Elevated` loads the dev-office module (and marks the floor present); a transition out (admission revoked, or the cwd is no longer the canonical repo) unloads it cleanly with **no stale elevated surface**; the gate is re-evaluated on its input-changing events and never cached as a remembered "was elevated" value.
- **Handoff:** Completes the DVO-4 lifecycle; D01 registers what the loaded module surfaces.
- **Notes:** Compose the extension/module loader (`l1-extensions`); the trigger predicate is `DevOfficeGate::resolve(...) == Elevated`. Clean unload leaves no capability/registration/rendered surface behind. **Observe-not-remember** (the `l2-service-activation` BA-8 precedent): a revoked admission must never leave a stale elevated surface, so the gate result is recomputed, not stored.

### [T-22D01] Workspace registration + confinement/audit wiring (DVO-1/6/7)

- **Spec:** l2-dev-office.md §4.1, §4.7; DVO-1, DVO-6, DVO-7
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` — `WorkspaceKind::Developer` registers as `0..1`, system-owned, non-deletable, and **not creatable through the project-creation flow**; the dev workspace's scope is its bound repository directory only (no handle into another workspace's store — the isolation assertion); an elevated action passes the `l2-tool-security` gate and produces an `l1-tool-receipts` receipt; a fault in a user office cannot reach dev authority (INV-8 boundary assertion).
- **Handoff:** Surfaces the office the module (C02) loads; T01 sweeps DVO-1/6/7.
- **Notes:** Compose the `crates/store-local` workspace registry (register the conditional Developer kind) + `l2-sandbox-policy` (process boundary) + `l2-tool-security` (authority gate) + `l1-tool-receipts` (audit) + the DW-8 human checkpoint. **Mostly composition + assertion tests, not new logic** — the elevated capabilities ARE the existing guarded subsystems, scoped to the bound repo. The DVO-1 conditional-floor surfacing is a thin binding over the already-built `l2-navigation` (no new GUI).

### [T-22D02] `cronus dev` CLI surface

- **Spec:** l2-dev-office.md §4 (composed surfaces); DVO-3, l1-architecture INV-9
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test` + 1 real CLI smoke via the compiled binary — `cronus dev status` prints the resolved tier (`Absent` in a non-canonical tree or with no admission); `cronus dev admit`/`revoke` are human-principal acts at the CLI (the operator is the principal, so this is a legitimate DVO-3 admission path — distinct from an *agent* self-granting); the binary handles each without panic and reports honestly.
- **Handoff:** Completes the shipped surface; T01 validates end to end.
- **Notes:** `cronus dev status|admit|revoke`, verb-first per `l2-cli`, INV-9 shipped-surface honesty (no unbound verbs). `status` reads `DevOfficeGate::resolve`; `admit`/`revoke` write the `DeveloperAdmission` on the human plane (the CLI human operator is the DVO-3 principal — the agent still has no such path). Reuses the `cronus activation`/`cronus knowledge` CLI-module precedent.

### [T-22T01] Validation sweep: DVO-1…DVO-8 acceptance

- **Goal:** Verify the assembled developer office against `l2-dev-office` — every DVO invariant covered by a named test through the real gate + facade.
- **Method:** New `crates/core/tests/dev_office_invariants.rs` — one named test per invariant: DVO-1 conditional floor (present only while `Elevated`), DVO-2 repo-authenticity fail-closed (non-canonical/ambiguous → not `Genuine`), DVO-3 escalation unrepresentable (the domain gate cannot mint an admission), DVO-4 clean unload (no stale surface after revoke), DVO-5 tier resolution + feedback default-off (+ opt-in), DVO-6 repo-scope isolation (no cross-workspace handle), DVO-7 confinement + audit (tool-security gate + receipt on every elevated action), DVO-8 unchanged dev-workflow (the standard pipeline, no exception lane). Final gate: `cargo test -p cronus-core dev_office_invariants` green + `cargo test --workspace` exit 0 ×3 + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check`.
- **Status:** Todo
