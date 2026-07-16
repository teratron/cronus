---
phase: 17
name: "Model Transport & Provider Connectivity (L2)"
status: Todo
subsystem: "crates/contract (NEW InferenceBackend trait + request/stream-event/cancel/descriptor types; routing-metadata ModelProvider untouched) · crates/model-local (NEW infra crate: blocking HTTP+TLS client, endpoint-profile model consuming router api_base, /v1-compatible streaming generate with worker-thread pull-iterator + cancellation, embed/describe/pull per capability flag, wire-failure→error-recovery-taxonomy mapping with no internal retry) · crates/core (facade wiring + host-side nodus bridge that collapses the stream to satisfy nodus::ModelProvider) · workspace+CI (model-local registered as peer infra crate; boundary guard forbids domain→model-local): realizes l2-model-runtime — the streaming inference surface neither existing trait provided, closing the audit gap where selection was implemented but no seam could perform the call"
requires: [2, 4, 13]
provides: []
key_files:
  created: ["crates/model-local/Cargo.toml", "crates/model-local/src/lib.rs"]
  modified: ["crates/contract/src/lib.rs", "Cargo.toml", "Cargo.lock"]
patterns_established: []
duration_minutes: ~
---

# Stage 17 Tasks — Model Transport & Provider Connectivity (L2)

**Phase:** 17
**Status:** Todo
**Strategic Goal:** Realize `l2-model-runtime` — the transport layer that turns a routing decision into a real inference call, closing the concept-conformance audit's keystone gap (model *selection* is fully implemented while no seam can *reach* a model). Define the missing streaming call surface as a new `InferenceBackend` trait in the contract crate; realize it in a new `model-local` infra crate as a synchronous, thread-scoped, streaming, cancellable REST transport over the six federated local providers (technology-stack §4.4); wire it into the facade and bridge it to the nodus `generate`/`analyze` seam by collapsing the stream to a `String`. The load-bearing property is **honest seam separation**: the routing-metadata `ModelProvider` (score) and the new `InferenceBackend` (call) are two facets of one provider object, and the transport owns neither selection (router) nor retry/rotation (error-recovery) — it performs the call and maps failures upward.

> **Real-integration phase, not domain-logic-first.** Unlike Phases 9–16, this builds the external seam prior phases deferred — a real HTTP+TLS client (spec-sanctioned dependency, `l2-model-runtime` §2). Tests stay hermetic via a mock HTTP server (a `std::net::TcpListener` fixture emitting scripted SSE/NDJSON), never a live provider — so `cargo test` needs no Ollama/llama.cpp running. **No async runtime** is the load-bearing constraint: concurrency is a bounded worker-thread pool, streaming is a pull-iterator over a bounded channel, cancellation is an atomic flag that drops the socket. Acceptance = the streaming contract (ordered tokens → Done, mid-stream cancel → Error(Cancelled)) proven against the mock server + a previously-inert model-consuming path (triage classification / compaction) produces a real result through the full seam + graceful degrade with no backend bound.

## Atomic Checklist

- [x] [T-17A01] `InferenceBackend` trait + types in `crates/contract`: `generate_stream` (→ pull-iterator of `StreamEvent { Token | ToolCall | Usage | Done | Error }`), `embed`, `describe` (→ `ModelDescriptor`: name/digest/size/params), `pull`, residency hints, and a `CancelHandle` (atomic flag). The routing-metadata `ModelProvider` trait stays exactly as-is — the two are distinct facets. **Verify:** `cargo test -p cronus-contract inference` — trait + types compile; a hand-written mock impl drives a scripted stream to `Done` and, on a mid-stream `CancelHandle` set, yields `Error(Cancelled)` and stops.
- [x] [T-17B01] New `crates/model-local` + blocking HTTP+TLS client dependency (ureq-class, rustls); endpoint-profile model (protocol family `/v1`-compatible vs provider-native, capability flags, probe rules) that consumes the router policy's `api_base` (not a parallel address registry); loopback reachability probe reusing the stack §4.4 discipline (thread-scoped, 800 ms timeout, loopback-only default). **Verify:** `cargo test -p cronus-model-local profile` — profiles resolve their address from a supplied `api_base`; a probe against a stub `TcpListener` classifies reachable vs connection-refused vs timeout within the 800 ms budget.
- [x] [T-17B02] `/v1`-compatible streaming generate: a worker thread owns the HTTP connection and parses SSE/NDJSON into `StreamEvent`s pushed to a **bounded** channel; the caller pulls (backpressure = the bounded channel); the `CancelHandle` drops the connection mid-stream. **Verify:** `cargo test -p cronus-model-local generate_stream` — against the mock HTTP server emitting SSE chunks, the iterator yields ordered `Token`s then `Done`; a cancel mid-stream yields `Error(Cancelled)` and the server observes the socket close; a slow consumer does not grow memory unbounded (bounded-channel assertion).
- [ ] [T-17B03] `embed` + `describe` + `pull` per capability flag; wire-failure mapping onto the error-recovery taxonomy (connect-refused / timeout / 4xx / 5xx / malformed-stream / cancelled) with **no internal retry** (retry/rotate/fallback stay upstream); remote profiles constructible only behind the security egress grant, credential attached from the secret store at call time (never cached). **Verify:** `cargo test -p cronus-model-local failure_map` — each simulated wire failure maps to the correct taxonomy variant; exactly one request is attempted per call (no retry observed); a remote profile refuses to construct without an egress grant.
- [ ] [T-17C01] Facade wiring + nodus bridge in `crates/core`: expose a wired `contract::InferenceBackend` to the engine; implement the host-side adapter satisfying `nodus::ModelProvider` by calling `generate_stream` and concatenating `Token` events into the `String` that `generate` returns, and mapping `analyze`. nodus gains **no** dependency (LP-1). **Verify:** `cargo test -p cronus-core nodus_bridge` — a nodus `GEN` step run with the bridge over a mock backend returns the concatenated stream as its output (no longer the `[STUB …]` label); `analyze` returns a real flag→score map.
- [ ] [T-17D01] Register `model-local` as a workspace member and peer infra crate; extend the CI boundary guard so a `domain → model-local` edge is forbidden (domain depends on the contract trait only, never the transport crate). **Verify:** the boundary-guard test FAILS on an injected `domain → model-local` dependency and PASSES on the real tree (the failure path is tested, not only the happy path). **Note:** the matching `l2-crate-topology` *spec* amendment (add `model-local` as a peer infra crate row, same tier/rules as `store-local`/`auth-local`) rides via `/magic.spec` — mechanical, not a run-task spec edit.
- [ ] [T-17T01] Validation sweep: a previously-inert model-consuming path (trigger-triage classification and/or context compaction) runs end to end through the full seam (facade → `InferenceBackend` → mock HTTP backend) and produces a real result; the no-backend path degrades without panic (documented behavior). **Verify:** `cargo test -p cronus-core --test transport_e2e` — the classifier/compactor produces a non-stub result with a backend wired and degrades gracefully with none; plus `cargo test --workspace` green + clippy `-D warnings` + `cargo fmt --all --check` clean.

## Detailed Tracking

### [T-17A01] InferenceBackend trait + types

- **Spec:** l2-model-runtime.md §4.1, §4.2, MR-2/MR-8
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-contract inference` — 4/4 passed (`generate_stream_runs_to_done_uncancelled`, `cancel_mid_stream_yields_single_cancelled_error_then_stops`, `unsupported_capability_reported_honestly_not_emulated`, `cancel_handle_clone_shares_the_same_flag`). `cargo check --workspace` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean.
- **Handoff:** Gates B (transport implements the trait) and C (facade/bridge consume it).
- **Notes:** Critical path — the trait shape is the single point every other track builds on; keep it small and stable. Routing-metadata `ModelProvider` is untouched (zero changes to its section). **Pre-existing unrelated defect found during regression check (not caused by this task, not fixed here — out of scope):** `cargo test -p cronus-core --test model_router fit_level_routing_exclusion` fails when run in isolation, on the pre-change baseline too (confirmed via `git stash`/`git stash pop` — same failure with or without this task's diff). Root cause: `BANDIT_COUNTER` (`crates/domain/src/router/mod.rs:55`) is a process-wide `static AtomicU64`, not scoped per `RouterPool` — its "every 20th call" trigger fires on the very first `route()` call of an isolated test run (counter starts at 0, `0 % 20 == 0`), and the bandit branch (lines 171–183) picks the first healthy provider **without any context-window fit check**, violating the test's own asserted invariant ("tiny context window must be excluded"). This is a real correctness gap in `l2-model-router`'s bandit path (MR-7-adjacent) plus a test-isolation smell (shared global mutable state across the test binary) — reported to the user, not silently fixed; out of this task's crate/scope.

### [T-17B01] model-local crate + endpoint profiles + probe

- **Spec:** l2-model-runtime.md §4.3, MR-1/MR-10
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-model-local` — 7/7 passed (accepts the real §4.4 `localhost` defaults for all six providers; accepts numeric loopback `127.0.0.1`/`[::1]`; rejects `0.0.0.0`/`[::]` wildcard binds; rejects non-loopback hosts by default; probe classifies `Reachable` against a real bound listener, a closed port as refused-or-honestly-timed-out, and `Timeout` against the RFC 5737 reserved test-net address within budget). Repeated 3x — zero flakiness. `cargo check --workspace` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean.
- **Handoff:** Gates B02/B03; D01 needs the crate to exist.
- **Notes:** New crate registered in root `Cargo.toml` workspace members + `workspace.dependencies` (`cronus-model-local`, `ureq = { version = "3.3.0", default-features = false, features = ["rustls"] }` — resolved cleanly, rustls only, no native-tls, matching §2's exact requirement). `EndpointProfile::new` is loopback-only by construction (MR-1): accepts the literal hostname `"localhost"` (the real §4.4 defaults use it, not a numeric IP) plus `127.0.0.1`/`::1`; rejects wildcard (`0.0.0.0`/`::`) and any other host as `ProfileError` variants — a remote profile is a distinct, egress-gated constructor for T-17B03. **Empirical environment finding (not a defect):** in this execution sandbox, connecting to a definitively-closed local port does not yield an immediate OS-level `ConnectionRefused` — it silently times out instead (confirmed via a standalone experiment, consistently reproducible), unlike typical bare-metal behavior. The probe code's `ConnectionRefused` branch is correct and exercised normally on a real user machine; the test was adjusted to accept either `ConnectionRefused` or `Timeout` for a closed port, with the reasoning recorded inline, rather than asserting a sandbox-specific behavior as if it were universal.

**Second pre-existing, unrelated defect found during full-workspace regression verification (not fixed here — out of scope, same discipline as T-17A01's finding):** `cargo test --workspace` intermittently fails `cargo test -p cronus-core --test mission_mode resolve_mode_env_overrides_config` ("env must win over config"). Root cause, confirmed by reading `crates/core/tests/mission_mode.rs:124-144`: three tests (`resolve_mode_env_overrides_config`, `resolve_mode_config_used_when_no_env`, `resolve_mode_defaults_to_full`) mutate the same process-wide `CRONUS_MISSION_MODE` environment variable with no synchronization; the inline comment claiming "test runs single-threaded or with unique key" is incorrect — `cargo test`'s default runner is multi-threaded, so this is an unsynchronized global-mutable-state race, not caused by this task's diff (confirmed unrelated: this session's changes touch only `crates/contract`, `crates/model-local`, and `Cargo.toml`/`Cargo.lock`). Combined with T-17A01's `BANDIT_COUNTER` finding, this is the SECOND instance this session of a shared-global-state test-isolation bug in `crates/core`'s test suite — worth a dedicated hardening pass (recommend `/magic.task main "harden test isolation: BANDIT_COUNTER + CRONUS_MISSION_MODE env races"` as its own task) rather than folding fixes into this phase's unrelated diff.

### [T-17B02] Streaming generate + cancellation

- **Spec:** l2-model-runtime.md §4.2, MR-8
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-model-local` — 11/11 passed (7 B01 + 4 new streaming: ordered `Token`s → `Done` against the mock SSE server; clean EOF without a `[DONE]` sentinel treated as `Done`; mid-stream cancel → exactly one `Error(Cancelled)` then `None`, with the server observing the client's socket close via a bounded read that deliberately does NOT count its own read-timeout as a close; bounded-channel backpressure — with capacity 4 and a 50-event server, the undrained worker buffers ≤ capacity+1, then the test drains fully so no thread wedges on a small OS socket buffer). Repeated 3x — zero flakiness. `cargo test --workspace` fully green this run; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean.
- **Handoff:** Feeds C01 (bridge collapses this stream) and T01. `MockSseServer` fixture established (bind/accept_and_run + drain_request/write_sse_* helpers) — reuse in B03/T01 as planned.
- **Notes:** Implemented as `EndpointProfile::generate_stream[_with_capacity]` → `StreamReceiver` (pull-iterator, terminal-event latch, worker-death surfaced as `MalformedStream` never a silent clean end) over `run_generate_worker`: raw `TcpStream` + hand-rolled HTTP/1.1 request, status/header parse, SSE `data:` framing, `[DONE]` sentinel + clean-EOF-as-Done, `sync_channel` backpressure, cooperative cancel via 50 ms read-timeout polling (`FrameReader`), socket closed by dropping the reader. **Dependency correction (frugality):** `ureq` (added in B01) was removed before ever being used — the loopback catalog needs no TLS, and ureq's whole-body timeout model doesn't map onto the poll-and-check-cancel contract; `serde_json` (already a workspace dep) covers SSE payload parsing. The HTTP+TLS client the spec sanctions (§2) now explicitly rides with the remote/egress-gated profile path (B03+), where TLS is genuinely required — recorded in the crate doc header. `GenerateRequest.parameters` deliberately not folded into the wire body yet (typed-JSON coercion deferred; noted inline). Two test defects caught by manual review before first compile (during the tool outage): a potential test hang (backpressure test never drained → server write could wedge on a small socket buffer) and a false-positive (server counting its own read-timeout as "client closed").

### [T-17B03] embed / describe / pull + failure mapping

- **Spec:** l2-model-runtime.md §4.4, §4.5, MR-3/4/5/9
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-model-local failure_map` — each wire failure → correct taxonomy variant; exactly one request per call; remote profile refuses without egress grant.
- **Handoff:** Completes the transport surface.
- **Notes:** No internal retry — rotation/pool stays in router/error-recovery. MR-3/4/5 are delegated-with-disclosure (federated server owns storage).

### [T-17C01] Facade wiring + nodus bridge

- **Spec:** l2-model-runtime.md §4.1, MR-2/MR-11
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-core nodus_bridge` — nodus `GEN` returns the concatenated real stream (not the stub); `analyze` returns a real map.
- **Handoff:** Makes the whole model-consuming surface live; feeds T01.
- **Notes:** The bridge is the stream→String projection; nodus stays dependency-free.

### [T-17D01] Topology registration + CI boundary guard

- **Spec:** l2-model-runtime.md §4.1; l2-crate-topology (peer infra crate pattern)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** boundary-guard test FAILS on injected `domain → model-local`, PASSES on the real tree.
- **Handoff:** Keeps INV-8 compiler/CI-enforced with the new crate.
- **Notes:** Code + CI only. The `l2-crate-topology` spec amendment rides via `/magic.spec` (mechanical peer-crate row) — do NOT edit the Stable spec inside run.

### [T-17T01] Validation sweep

- **Goal:** Prove a previously-inert model-consuming path now produces a real result through the full seam, and degrades gracefully with no backend.
- **Method:** `cargo test -p cronus-core --test transport_e2e` (classifier/compactor real result + no-backend degrade) + `cargo test --workspace` + clippy `-D warnings` + `cargo fmt --all --check`.
- **Status:** Todo

## Tracks

- **A — Contract inference trait (foundation):** T-17A01. Gates B and C.
- **B — Transport crate:** T-17B01 → T-17B02 → T-17B03 (share the crate + mock-server fixture → serialized).
- **C — Facade + nodus bridge:** T-17C01 (depends on A; uses a mock backend, real backend from B).
- **D — Topology + CI guard:** T-17D01 (depends on B01 — crate must exist).
- **T — Validation:** T-17T01 (last; end-to-end through the full seam).

Foundation-then-parallel: Track A gates B/C; B is serialized on the shared crate + mock-HTTP fixture; C proceeds against a mock trait impl once A lands; D follows B01; T closes the phase.
