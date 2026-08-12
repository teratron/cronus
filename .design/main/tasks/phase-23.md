---
phase: 23
name: "Tool Receipts"
status: Todo
subsystem: "crates/domain/src/tool_receipts"
requires: [5, 13]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 23 Tasks — Tool Receipts

**Phase:** 23
**Status:** Todo
**Strategic Goal:** Ship `l2-tool-receipts` — model-unforgeable per-action execution receipts (TR-1…TR-9) as a keyed-BLAKE3 MAC over a length-prefixed injective binding. Closes the project's one *verified* coverage gap: `l1-tool-receipts` has been Stable since 2026-07-02 with zero implementation, discovered mid-build by Phase 22 and disclosed rather than fabricated. Domain/facade split on the `*_bootstrap.rs` precedent; Track A gates B/C/D.

## Execution Character (Planning Audit)

**Small surface, exacting core.** This is not an orchestration phase — it builds a real primitive. The whole subsystem is perhaps 500 lines of domain logic, but the binding encoder is the kind of code that is *silently* wrong: a naive implementation passes every test of its own logic while being forgeable. Track A therefore carries concrete adversarial test vectors, not just round-trip tests.

**Zero new dependencies — verified, not assumed.** `blake3` is already a `cronus-domain` dependency (`crates/domain/Cargo.toml`) **and** already on the boundary-guard allowlist (`scripts/check-domain-boundary.mjs`, `ALLOWED`). No manifest edit, no guard edit. If any task finds itself adding a crate, that is a signal the design drifted — stop and re-read the spec's §4.1 rather than adding it.

**The integration surface is one call site — which is the phase's main honesty risk.** A survey found exactly **one** production caller of `ToolPolicy::is_permitted` (`crates/core/src/dev_office_workspace.rs:81`, `run_elevated_action`) and exactly one production caller of `append_audit_entry` (the same function). Everything else is tests. Two consequences:

1. **Track C is small.** `ReceiptedDispatch` has one real integration point today, and `run_elevated_action` is already shaped as gate → audit → outcome, so the receipt slots in naturally.
2. **The phase must not claim project-wide receipt coverage.** There is no general tool-dispatch surface in this codebase yet. Shipping `ReceiptedDispatch` as *the* seam is correct and future call sites adopt it by construction (`Receipted<T>` is the only way to obtain a result) — but at ship time exactly one path is receipted. INV-9 (shipped-surface honesty) and TR-8 (honest coverage boundary) both bite here: T-23T01 asserts what is actually covered, and no doc, help text, or status output may imply more.

**Load-bearing correctness property = the injective binding (§4.2).** The single most likely implementation error is treating `‖` in the spec's reference sketch as literal concatenation. That yields a MAC where `kind="ab", inputs="c"` and `kind="a", inputs="bc"` produce identical tags — one valid receipt simultaneously valid for a different action, forgeable without the key. The second most likely error is dropping `action_id` from the binding, which re-opens intra-session replay (an echoed receipt validating as a fresh invocation). Both are caught only by tests written *as adversaries*, which is why they are named explicitly in the Verify lines rather than left to "add unit tests".

**Cascade risk.** Track A's binding shape is consumed by every other track. If A02 lands with the wrong field order or a missing length prefix, B/C/D all re-do against the corrected tags. Mitigation: A01/A02 are deliberately small and fully testable against fixed key vectors with no I/O — prove them before writing a single line of facade code.

**Foundation-then-parallel:** A (binding + MAC) gates all. D (deferred lifecycle) needs B01's state types but *not* B02's ledger, so B02 and D01 can run in parallel once B01 lands. C (facade) needs A + B. T closes.

**Module layout — a deliberate refinement of the spec's §4.1 listing.** The spec shows `crates/domain/src/tool_receipts.rs` as a single file. Build it as a **directory module** (`crates/domain/src/tool_receipts/` with `mod.rs` re-exporting) instead: it carries five distinct concerns (key, binding, MAC, ledger, deferred lifecycle) and the house precedent for a domain module of that size is a directory — `loop_runner/`, `archetype/`, `session/` are all directories. §4.1's listing is a logical grouping of what the module contains, not a literal one-file constraint, and nothing in the spec's invariants depends on file count. This is a plan-level refinement, not a spec deviation — no `/magic.spec` amendment needed.

## Atomic Checklist

- [ ] [T-23A01] `ReceiptKey` + canonical length-prefixed `ActionBinding` encoder (TR-2/TR-5)
- [ ] [T-23A02] `mint`/`verify` + token format + constant-time compare (TR-2/TR-3)
- [ ] [T-23B01] `Receipted<T>` + `ReceiptState` — structural TR-1 enforcement
- [ ] [T-23B02] `ReceiptLedger` + `status()` default-deny + `CoverageReport` (TR-4/TR-8)
- [ ] [T-23C01] `ReceiptSession` — OS entropy, session lifetime, zeroing drop (TR-5)
- [ ] [T-23C02] `ReceiptedDispatch::invoke` + SEC-7 audit sink + wire the real call site (TR-1/TR-7/TR-9)
- [ ] [T-23D01] Deferred-action lifecycle: `Pending` → correlated completion → mint (TR-8)
- [ ] [T-23T01] Validation sweep: TR-1…TR-9 acceptance + leak paths + honest coverage

## Detailed Tracking

### [T-23A01] `ReceiptKey` + canonical binding encoder

- **Spec:** l2-tool-receipts.md §4.1 (tier split), §4.2 (canonical binding), §4.3 (the key); TR-2, TR-5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain tool_receipts::binding` — asserts (a) **boundary-shifting**: `encode(kind="ab", inputs="c", …)` ≠ `encode(kind="a", inputs="bc", …)` byte-for-byte, with every other field held equal; (b) each field's length prefix is present and little-endian; (c) the domain-separation tag leads the buffer; (d) `format!("{:?}", key)` contains no byte of the key material and renders the fixed redaction placeholder. `cargo clippy -p cronus-domain --all-targets -- -D warnings` clean.
- **Handoff:** Gates T-23A02 (which MACs this buffer) and therefore every other track.
- **Notes:** New module at `crates/domain/src/tool_receipts/` (directory — see the Execution Character note on layout) — pure, I/O-free, no new dependency (`blake3` already present and allowlisted). `ReceiptKey` is a newtype over `[u8; 32]` with the leak paths closed by construction: hand-written `Debug` printing `ReceiptKey(<redacted>)`, **no** `Display`, **no** `Serialize`, **no** `Clone`, no accessor returning raw bytes outside the module; `Drop` overwrites through a volatile write. State the volatile-write's residual limit in a plain-language comment at the site (§4.3 is honest that it does not defeat an earlier optimizer-spilled copy) — do not write a comment claiming full scrubbing. `ActionBinding` encodes `domain_tag ‖ u64_le(action_id) ‖ [u32_le(len) ‖ bytes]* ‖ u64_le(timestamp_ms)`; inputs and results enter as digests, never raw, so a secret-bearing argument never lands in the binding buffer. **`action_id` is not optional** — it is what makes the receipt witness an invocation rather than an action shape (§4.2).

### [T-23A02] `mint`/`verify` + token + constant-time compare

- **Spec:** l2-tool-receipts.md §4.2; TR-2, TR-3, TR-6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain tool_receipts::mac` — with a **fixed key and pinned clock**: (a) round-trip `verify(mint(x)) == Ok`; (b) **replay**: two bindings identical in every field except `action_id` produce different tags, and receipt #1 fails `verify` against binding #2; (c) altering the result digest alone flips `verify` to mismatch (TR-3); (d) a receipt minted under key K1 fails `verify` under K2 (post-rotation behaviour); (e) the token matches `^cronus-rcpt-[0-9a-f]+-[0-9a-f]{32}$`. The clock MUST be pinned in (b) — a test that lets the real millisecond advance passes for the wrong reason and would stay green with `action_id` removed. `cargo clippy -p cronus-domain --all-targets -- -D warnings` clean.
- **Handoff:** Completes Track A; unblocks B, C, D.
- **Notes:** `blake3::keyed_hash(key, binding)` — BLAKE3's keyed mode is a MAC by design; do **not** hand-roll HMAC over a plain hash, and do **not** reach for `hmac`/`sha1` from `crates/auth-local` (that would add a domain→adapter edge the boundary guard forbids, and SHA-1 is the wrong primitive to introduce into new code). Token = `"cronus-rcpt-" ‖ base16(timestamp_ms) ‖ "-" ‖ base16(tag[0..16])`; 128-bit truncation is deliberate and sufficient — the adversary is an in-process model with no key and no verification oracle. Comparison folds over the **full** tag length regardless of first mismatch; a short-circuiting `==` leaks tag bytes positionally under repeated probing. Write the constant-time compare once in this module and call it from `verify` only.

### [T-23B01] `Receipted<T>` + `ReceiptState`

- **Spec:** l2-tool-receipts.md §4.4 (dispatch seam), §4.6 (states); TR-1, TR-8
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain tool_receipts::receipted` — asserts the value cannot be reached without its receipt: `Receipted<T>` exposes no public constructor and no `From<T>`, its field is private, and the only way to build one is the crate-internal path used by dispatch. Prove it as a **compile-fail test** (`trybuild`-style or a documented `compile_fail` doctest) rather than a runtime assertion — a runtime test cannot demonstrate that an alternative construction does not exist. `cargo clippy -p cronus-domain --all-targets -- -D warnings` clean.
- **Handoff:** Gates T-23C02 (dispatch returns this type) and T-23D01 (`Pending` is one of its states). Once this lands, T-23B02 and T-23D01 are independent of each other.
- **Notes:** This task is the whole of TR-1's structural guarantee, and it must land **before** any caller exists — adding the wrapper after call sites accumulate turns a compile-time property into a refactor nobody finishes (§5 note 2). `ReceiptState` is `Minted(Receipt) | Pending { action_id }`; `Pending` deliberately carries **no** tag (see D01). Keep the type free of any `unwrap`-style escape hatch that would let a caller discard the receipt and keep the value.

### [T-23B02] `ReceiptLedger` + default-deny `status()` + `CoverageReport`

- **Spec:** l2-tool-receipts.md §4.5 (ledger, absence), §4.6 (coverage); TR-4, TR-8
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain tool_receipts::ledger` — asserts (a) `status()` on an `action_id` never dispatched returns `Unreceipted`, **not** an error and not an assumption of truth; (b) a minted action returns `Receipted(receipt)` and a deferred one returns `Pending`; (c) `CoverageReport` reports `receipted` and `pending` as separate counts; (d) **no** public API exists that converts `Unreceipted` into a recorded fact — assert by inspection in the test's doc comment naming the reviewed surface, since absence of a function cannot be asserted at runtime. `cargo clippy -p cronus-domain --all-targets -- -D warnings` clean.
- **Handoff:** Feeds T-23C02 (dispatch appends here) and T-23T01 (TR-4 acceptance).
- **Notes:** TR-4 is the invariant most likely to be built as something it is not. It does **not** mean parsing the model's prose for action claims — that is unbounded and fails open on the first unanticipated phrasing. The realization inverts the burden: the ledger is the sole authority on "did this happen", and any component that would record an action as fact consults `status()` and treats `Unreceipted` as fabricated. The guarantee is that **no upgrade path exists**, so the review that matters is of the API surface, not of a branch.

### [T-23C01] `ReceiptSession` — entropy, lifetime, zeroing drop

- **Spec:** l2-tool-receipts.md §4.1 (tier split), §4.3 (the key); TR-5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-core receipts_bootstrap` — asserts (a) two sessions constructed in one process hold different keys (fresh entropy, not a constant); (b) the key never appears in any `Debug`/`Display` rendering of `ReceiptSession`; (c) no code path writes the key to the state tier — assert by naming the reviewed surface, and keep the session out of every struct that is serialized. `cargo clippy -p cronus-core --all-targets -- -D warnings` clean.
- **Handoff:** Gates T-23C02 (which needs a live session to mint).
- **Notes:** New `crates/core/src/receipts_bootstrap.rs`, following the `activation_bootstrap` / `knowledge_bootstrap` / `loop_bootstrap` precedent exactly. This is the **only** non-deterministic act in the subsystem: reading OS entropy. `getrandom` is deliberately absent from the domain allowlist, which is why key generation lives here and the domain receives an opaque, already-random key — do not "simplify" by moving generation into the domain, that edge is what the boundary guard exists to reject. Rotation is implicit in process restart; there is no persistence path to write, and adding one would trade a secret with no theft window for one with a permanent on-disk surface (§6).

### [T-23C02] `ReceiptedDispatch::invoke` + audit sink + real call site

- **Spec:** l2-tool-receipts.md §4.4 (dispatch seam), §4.5 (audit record); TR-1, TR-3, TR-7, TR-9
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-core receipts_dispatch` — asserts (a) an **allowed** action yields a receipt binding `outcome_tag="ok"` and the real observed result; (b) a **blocked** action also yields a receipt, binding `outcome_tag="blocked"` and the block reason (TR-1 covers blocked calls, not just successes); (c) the receipt's result digest matches the *actual* returned value, so substituting a different result fails `verify` (TR-3); (d) every mint appends an `AuditEntry` carrying the token and never the key; (e) a `verify` mismatch appends its own `receipt_mismatch` entry rather than returning a bare `false`. Plus `cargo test -p cronus-core dev_office` still green — the existing `run_elevated_action` behaviour is unchanged apart from now carrying a receipt. `cargo clippy --workspace --all-targets -- -D warnings` clean.
- **Handoff:** Feeds T-23T01; the subsystem is functionally complete after this task.
- **Notes:** `invoke` must **execute** the action (take it as a closure or equivalent) rather than only gating it — TR-3 requires the *observed result* inside the MAC, and a gate-only signature cannot see one. Order is fixed: `ToolPolicy::is_permitted` runs **first and unchanged**, and its verdict is an *input* to the binding — receipts witness the authorization decision without participating in it (TR-7; this is also why no API here may accept or return a permission). The one real integration point is `crates/core/src/dev_office_workspace.rs::run_elevated_action` (already shaped gate → audit → outcome); route it through `invoke` and keep its existing fail-closed-on-audit-failure behaviour. Reuse the existing `append_audit_entry` — do **not** open a second log file, since TR-9 asks receipts to strengthen the existing SEC-7 trail rather than split the forensic record (§6). Note the shipped `AuditEntry` only serializes `ts`/`layer`/`category`/`severity`/`outcome`, so carry the token in a serialized field rather than one silently dropped.

### [T-23D01] Deferred-action lifecycle (TR-8)

- **Spec:** l2-tool-receipts.md §4.6; TR-8, and `l1-execution-graph` EG-12
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p cronus-domain tool_receipts::deferred` — asserts (a) a detached action registers `Pending { action_id }` and produces **no** tag at dispatch; (b) on correlated completion the same `action_id` mints a full receipt binding the real result; (c) `CoverageReport` reports `pending > 0` while outstanding and moves the count to `receipted` on resolution; (d) there is no constructor for a receipt over an unobserved result. `cargo clippy -p cronus-domain --all-targets -- -D warnings` clean.
- **Handoff:** Feeds T-23T01 (TR-8 acceptance).
- **Notes:** The failure to avoid is a placeholder tag: a receipt computed over a fabricated or empty result would be a *valid* receipt for a false claim, which inverts the entire subsystem. `Pending` is therefore a distinct state carrying no tag, never a `Receipt` with an empty result. Coverage is surfaced as a `{ receipted, pending }` pair specifically so a caller cannot round it down to a single "verified" number — TR-8's honesty requirement expressed as a shape.

### [T-23T01] Validation sweep — TR-1…TR-9 acceptance

- **Spec:** l2-tool-receipts.md (all sections); TR-1…TR-9
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** New `crates/core/tests/tool_receipts_invariants.rs` with one named test per invariant (TR-1…TR-9), driven through the real facade export chain (`cronus_core::…`), matching the `dev_office_invariants` / `knowledge_invariants` shape — no direct `cronus-domain` dependency from the test file. Plus two leak-path tests that must be written as tests rather than assumed by inspection (§5 note 5): `format!("{:?}", key)` contains no key byte, and a `redact::redact` pass over text containing a receipt token leaves the token **intact** (a scrubbed receipt is indistinguishable from a missing one and would make TR-4 fire on genuine actions). Closing gate, run in full and not sampled: `cargo test --workspace` green ×3 consecutive → `cargo clippy --workspace --all-targets -- -D warnings` clean → `cargo fmt --all -- --check` clean. Confirm no `unwrap()`/`panic!()` on production paths across every file the phase touched, file by file.
- **Handoff:** Closes Phase 23 — frontmatter `status: Todo → Done` with `provides`/`key_files`/`patterns_established` filled; PLAN/TASKS milestone update; `.design/main/CHANGELOG.md` phase entry; auto-archive via `finalize --workflow=run`.
- **Notes:** **The honesty requirement is part of the acceptance bar, not a footnote.** At ship time exactly one production path (`run_elevated_action`) is receipted, because the codebase has no general tool-dispatch surface yet. The TR-8 test must assert the *real* coverage rather than an aspirational one, and no help text, status output, or doc line may imply project-wide receipt coverage (INV-9). If a later task is tempted to soften this, that is the exact failure `l1-tool-receipts` TR-4 and TR-8 were written to prevent: a surface implying coverage it does not have is worse than an honest partial one. Record the residual — every other call site that acquires a tool-dispatch path in future adopts receipts by construction — as a disclosed boundary, not as a gap needing a follow-up phase.
