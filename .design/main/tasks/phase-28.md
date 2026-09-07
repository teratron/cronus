---
phase: 28
name: "Terminal Surface Derivation & Single Entry Point"
status: Todo
subsystem: "crates/tui · crates/cli"
requires: [27]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 28 Tasks — Terminal Surface Derivation & Single Entry Point

**Phase:** 28
**Status:** Todo
**Strategic Goal:** Carry the terminal surface onto the registry the prior phase minted — its slash catalog becomes a projection and the hand-copied verb mirror is **deleted and tombstoned**, not regenerated — and collapse the two executables into one, so the product is a single command with a default composition rather than two things to install.

## Phase Notes

**This surface is where the project learned what a mirrored catalog costs, so its migration is also a repayment.** Finding F-2 is the parity check that could not fail: a hand-copied constant of the sibling surface's verbs, held inside this crate specifically so the check would need no dependency on the crate it was checking. The reasoning was sound and the result was an oracle comparing a copy against itself. Deleting it — not regenerating it — is the point. A generated copy is still a copy; it fails as a stale artifact rather than as a disagreement between the things that actually run, and the second failure is the one the corpus must produce.

**The mirror is already visibly stale, and that is the concrete "before" state to preserve in the record.** `crates/tui/src/command.rs`'s `CATALOG` currently advertises 21 verbs plus `/help`. Five of them — `goal`, `trigger`, `mission`, `research`, `change` — name groups the prior phase **deleted from the product entirely**, and several more (`init`, `status`, `workspace`, `ext`, `registry`) are `Installation`-locus verbs this surface must not project at all. The catalog is not drifting toward being wrong; it is already wrong in both directions at once, which is exactly what a copy does while its own test stays green.

**The projected surface will visibly shrink, and that is correct rather than a regression.** After derivation this frontend projects `Semantic` + `ClientLocal`; the command line projects `Semantic` + `Installation`. So `/init`, `/status`, `/workspace`, `/ext`, and `/registry` leave the slash catalog because an installation verb has no meaning inside a live session, and the five deleted stubs leave because they no longer exist anywhere. Plan this as a **disclosed surface change**, stated up front — not as something an executor discovers mid-migration and has to decide about alone.

**Two dependency edges do not exist yet, and both must be added.** `crates/tui` depends on `cronus-domain`, `crossterm`, and `ratatui` — it has **no `cronus-core` dependency**, so it cannot reach `invocable_bootstrap::bootstrap()` today. And `crates/cli` has **no `cronus-tui` dependency**, so the single binary cannot bring up the terminal surface today. These are the phase's real structural work, and they land in different tracks (A needs core→tui; D needs tui→cli) while sharing one file, `crates/tui/Cargo.toml`. Tracks A and D are therefore **not** as independent as their file lists suggest — treat that manifest as a shared resource and expect one of the two to rebase on the other.

**This phase deliberately reverses something the prior phase deliberately shipped.** T-27D03 set `subcommand_required(true)` so a bare `cronus` exits 2 with a usage error, because at that moment there was no default composition to bring up. LH-4 requires exactly the opposite once one exists: bare `cronus` brings up this frontend, and `cronus tui` names it explicitly, both spellings required so a script never depends on the default not changing. T-28D02 is that reversal. It is a planned progression, not a regression — say so in the task's own record so the next reader does not "fix" it back.

**Behaviour-preservation oracle is weaker here than it was for the command line, and the plan must not pretend otherwise.** The sibling frontend had `cli_smoke.rs` — 41 end-to-end tests driving a real subprocess — which caught three real pre-ship defects across that phase. This crate has **38 in-file unit tests and no `tests/` directory at all**, and 10 of those 38 live in `command.rs`, the file whose mirror and catalog are being deleted. Several of them will be deleted *with* the thing they test, exactly as thirteen smoke tests were deleted with the stub groups they exercised. Track T therefore carries more weight here than it did there: the corpus registration (T-28T01) is not a formality, it is the replacement oracle.

**Sizing, stated against the prior phase's actual outcome rather than optimistically.** Track D of the prior phase was planned as one task and needed six `.N` splits before it closed. The comparable work here is Track A. It is genuinely smaller — ~2,155 lines in one crate versus ~4,000 across two surface halves and 29 groups — and every mechanism it consumes (registry, dispatcher, generated-parser pattern, renderer, corpus harness) already exists and is proven by a real consumer. So three tasks is an honest estimate, not a hopeful one. **The task to watch is T-28A02**: it replaces this crate's own `Dispatcher` trait and its `dispatch(&SlashCommand) -> String` contract, which every existing `app.rs` test constructs. If it grows past one pass, split it by the same `.N` convention the prior phase used, and split it by *what the dispatcher returns* (outcome plumbing first, rendering second) rather than by panel.

**Critical path and cascade risk.** Track A is the single point everything else depends on: B registers into the same catalog, C renders outcomes A produces, D launches the surface A defines, and T proves all of it. If the projection's shape is wrong, four tracks rework. Land A behind its own tests before opening B or C, and prefer discovering a shape problem in A02's unit tests over discovering it in the corpus run.

**Build environment.** Unchanged from the prior phase: anything triggering C compilation or a Tauri/`windres` step runs in **PowerShell**, not Git Bash. A clean check from Bash that suddenly fails at a C step is an environment artifact, not a code defect.

## Atomic Checklist

- [ ] [T-28A01] Slash catalog and discovery generated from the registry; the verb mirror deleted and tombstoned
- [ ] [T-28A02] Dispatch through the shared dispatcher; the local redaction call removed in favour of the boundary
- [ ] [T-28A03] An unresolved slash line is ordinary input, not a rendered failure
- [ ] [T-28B01] Pane and panel actions register as `ClientLocal` invocables through the shared door
- [ ] [T-28B02] The two surfaces' locus difference is declared, not asserted away
- [ ] [T-28C01] Panels render from core projections, with *unavailable* distinct from *empty*
- [ ] [T-28D01] `cronus tui` becomes a verb of the one binary; the standalone executable is retired
- [ ] [T-28D02] A bare invocation brings up the default composition
- [ ] [T-28T01] Corpus registration from this crate's own test target; finding F-2 repaid
- [ ] [T-28T02] Behaviour-preservation proof and full quality gates

## Detailed Tracking

### [T-28A01] Slash catalog and discovery generated from the registry; the verb mirror deleted and tombstoned

- **Spec:** l2-tui.md §2, §3 (INV-3, INV-9), §4.1, §4.3
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** Replace `crates/tui/src/command.rs`'s `const CATALOG` with a catalog built at startup from the registry's descriptors, filtered to this surface's own loci (`Semantic` + `ClientLocal`) and `Shipped` stability — the same filter shape `generated::semantic_shipped` already applies on the sibling surface, reused rather than restated if its signature allows. `/help`'s discovery listing renders from the same descriptors (name + summary), never from a second list. Delete `const EXPECTED_CLI_VERBS` and the parity test built on it outright; add its tombstone entry to `crates/conformance/src/findings.rs`'s append-only ledger in the same change, with the matching `ledger_baseline` edit that makes the addition a deliberate, reviewable act.
- **Verify:** `cargo test -p cronus-tui` green. `rg 'EXPECTED_CLI_VERBS|const CATALOG' crates/tui/` returns **no matches**. A new test asserts the built catalog's verb set equals the registry's own `Semantic`+`ClientLocal`+`Shipped` set for a registry the test populates itself (not a literal list) — a restated expected-verb list in the assertion is the exact defect this task deletes and fails review. `cargo test -p cronus-conformance` green with the new tombstone present in both `tombstones()` and `ledger_baseline::tombstones()`.
- **Handoff:** T-28A02 (dispatch consumes the same descriptors), T-28B01, T-28T01.
- **Notes:** `crates/tui/Cargo.toml` gains `cronus-core` here — the first of this phase's two new dependency edges. Expect the five deleted-stub verbs and the five installation-locus verbs to leave the catalog as a consequence; that is the disclosed surface change the Phase Notes name, and it belongs in this task's own `Changes` record, not discovered later. `command.rs` holds 10 of this crate's 38 tests; the ones that assert on `CATALOG` membership or the mirror die with it — delete them with their subject rather than rewriting them to assert the same thing about the generated list.

### [T-28A02] Dispatch through the shared dispatcher; the local redaction call removed in favour of the boundary

- **Spec:** l2-tui.md §3 (INV-2, INV-7), §4.2
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** Replace this crate's own `Dispatcher` trait (`app.rs`, `dispatch(&SlashCommand) -> String`) with the shared `cronus_core::invocable::Dispatcher`: parse a slash line into an `Invocation` by binding its declared binders from the descriptor, dispatch, and render the returned `Outcome` into the command-feedback view state. Delete `CoreDispatcher`'s `secrets: Vec<String>` field and its local masking pass — masking is the dispatch boundary's job and re-implementing it here is the asymmetry INV-7's amendment closes. The empty-secret-list half stays a recorded residual, unchanged and unfixed, exactly as it is on the sibling surface.
- **Verify:** `cargo test -p cronus-tui` green. `rg 'secrets|redact' crates/tui/src/` returns no local masking implementation (a comment naming the boundary is fine; a `Vec<String>` of secret values being matched against output is not). A test drives a real registered invocable end to end through the app's tick loop and asserts the rendered feedback matches the dispatched `Outcome`, including one `Rejected` case whose binder and mode reach the view state rather than being flattened into a bare string.
- **Handoff:** T-28A03, T-28C01, T-28T01.
- **Notes:** **This is the task to watch for a `.N` split** (see Phase Notes). If it grows past one pass, split by what the dispatcher returns — outcome plumbing first, feedback rendering second — never by panel. Every existing `app.rs` test constructs the current trait; changing its shape changes those constructions, which is expected and disclosed, not a weakened suite. The sibling surface's `Rendered`-value pattern (computation split from I/O so the result is assertable without a subprocess) is the precedent to follow if feedback rendering needs its own testable seam.

### [T-28A03] An unresolved slash line is ordinary input, not a rendered failure

- **Spec:** l2-tui.md §4.3 (v1.2.0 clause), l2-invocable-registry.md (SP-13 resolution split)
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** When resolution answers `Dispatched::Unknown`, this surface treats the line as **ordinary input** — it does not render an error, and it does not fabricate a failure `Outcome`. The sibling surface renders the identical answer as a usage error because a one-shot invocation has nothing to fall through to; both renderings are correct and the split exists precisely so one value can produce both.
- **Verify:** `cargo test -p cronus-tui` green, including a test that submits a slash-shaped line naming no invocable and asserts the app's command-feedback state is **not** an error rendering — and, in the same test, that a genuinely `Rejected` outcome still *is* rendered as a refusal, so the two are proven distinguishable rather than both silently swallowed.
- **Handoff:** T-28T01.
- **Notes:** Small by construction, and deliberately its own task rather than folded into A02: it is the one place where this surface and the command line are *supposed* to disagree, and burying that in a larger diff is how it gets "corrected" back to parity by a later reader who sees only an inconsistency.

### [T-28B01] Pane and panel actions register as `ClientLocal` invocables through the shared door

- **Spec:** l2-tui.md §4.4 (v1.2.0 clause), l2-invocable-registry.md §4.3 (EP-12)
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** Every action that exists only on this surface — pane focus, panel toggles, quit, view switching — registers as a `ClientLocal` invocable through the same `InvocableRegistry::register` a core or contributed verb uses, replacing whatever private key/action table currently drives them in `app.rs`. Keeping a local table beside the projection would rebuild, at smaller scale, exactly the hand-maintained catalog T-28A01 deletes.
- **Verify:** `cargo test -p cronus-tui` green. A test asserts every action this surface can perform is reachable through the registry (registered `ClientLocal`), and that no second dispatch path exists for them — concretely: the key handler resolves through the registry rather than a `match` on key codes bound directly to behaviour. `rg` shows no residual private action table.
- **Handoff:** T-28B02, T-28T01.
- **Notes:** Depends on T-28A01's registry access being in place. Key **bindings** stay local — which key maps to which action is terminal-surface presentation and belongs here (SP-8's do-not-unify record already names input mechanics as legitimately per-surface); what must not stay local is the *action's own declaration*.

### [T-28B02] The two surfaces' locus difference is declared, not asserted away

- **Spec:** l2-tui.md §4.3 (v1.2.0 clause), l2-surface-conformance.md §4.4, §4.6 (SP-8)
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** State the locus split where the corpus reads it: this surface projects `Semantic` + `ClientLocal`, the command line projects `Semantic` + `Installation`, the shared `Semantic` set is where parity binds, and each differing half is named with its reason. Express the difference as `DeclaredExclusion` entries in this surface's own corpus registration rather than as prose alone, so an undeclared omission fails the corpus instead of passing unnoticed.
- **Verify:** `cargo test -p cronus-tui` green with the corpus's surface-set family reporting **zero** divergence for this surface once exclusions are declared — and a test proving the declaration is load-bearing: removing one exclusion makes the surface-set family report the corresponding id as missing.
- **Handoff:** T-28T01.
- **Notes:** The negative test is the point. A declared exclusion nobody checks is the same shape as the mirror this phase deletes — a statement that cannot fail.

### [T-28C01] Panels render from core projections, with *unavailable* distinct from *empty*

- **Spec:** l2-tui.md §2, §3 (INV-2, INV-6), §4.1
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** Each panel (Board, Office, Status, Sessions/Log) becomes a pure function of a core-supplied projection plus terminal-local view state. A panel whose projection could not be obtained renders **unavailable with its reason**; a panel whose projection is legitimately empty renders empty. The two must be distinguishable in the view model, not only in the pixels.
- **Verify:** `cargo test -p cronus-tui` green, including one test per state — an unavailable projection renders a reason and an empty projection renders an empty view — asserting on the **view model**, so the distinction is a value the code carries rather than a formatting accident. `rg` shows no panel deriving a domain fact (a path, an ordering, a default) locally instead of receiving it.
- **Handoff:** T-28T01.
- **Notes:** This is the panel-level form of the residual the sibling surface still carries at its own list verbs (a store failure printing an empty listing with a success code). Here it is **fixed**, not preserved — this surface's panels are new work under the amendment, not a migration of shipped behaviour, so SP-10's preserve-then-correct rule does not apply to them. Say so explicitly in the task record; a reader who knows the residual will otherwise expect it to be carried.

### [T-28D01] `cronus tui` becomes a verb of the one binary; the standalone executable is retired

- **Spec:** l2-tui.md §4.4, l1-launch-handoff.md (LH-10)
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** Add `tui` to the command line's own installation-half declaration so `cronus tui` brings up this frontend in-process, and delete `crates/tui/Cargo.toml`'s `[[bin]] cronus-tui` target so the workspace ships exactly one executable. The standalone name is **retired under the declared-retirement rule**, not deleted silently: its departure is recorded where a user looking for it will find the replacement.
- **Verify:** `cargo build --workspace` produces exactly one binary — `cargo metadata --format-version 1` (or `ls target/debug`) shows no `cronus-tui` executable target. `cronus tui --help` exits 0 and `cronus --help` lists `tui` among its verbs. `cargo test --workspace --exclude cronus-desktop` green.
- **Handoff:** T-28D02, T-28T02.
- **Notes:** `crates/cli/Cargo.toml` gains `cronus-tui` here — the second of the phase's two new dependency edges, and the one that makes the command-line crate the composition root for both surfaces. Shares `crates/tui/Cargo.toml` with T-28A01; whichever lands second rebases. The retirement record belongs wherever this project already documents a removed name — check what the prior phase did for the five deleted groups before inventing a new mechanism for it.

### [T-28D02] A bare invocation brings up the default composition

- **Spec:** l2-tui.md §4.4 (v1.2.0 clause), l1-launch-handoff.md (LH-4), l2-cli.md §4.2
- **Status:** Todo
- **Assignment:** Agent
- **Scope:** `cronus` with no verb brings up this frontend; `cronus tui` names the same composition explicitly. Both spellings required — the default is what lets a user meet the product by typing its name, the explicit verb is what lets a script state what it wants so its meaning does not change on the day the default does.
- **Verify:** running the binary with no arguments enters the terminal surface (not a usage error) and exits 0 on a clean quit; `cronus tui` does the same; `cronus --help` and `cronus --version` still answer as themselves rather than launching anything. `cargo test -p cronus-cli --all-targets` green.
- **Handoff:** T-28T02.
- **Notes:** **This deliberately reverses `subcommand_required(true)`**, set by the prior phase precisely because no default composition existed then. Record it in the task's own `Changes` as a planned progression with its reason, or the next reader reads a shipped usage-error behaviour being undone as a regression. Verify the three surface-level requests (`--help`, `-h`, `--version`, `-V`, bare `help`) still route to the launcher rather than the composition — the existing entry point already distinguishes them and that logic must survive.

### [T-28T01] Corpus registration from this crate's own test target; finding F-2 repaid

- **Goal:** Make this surface the corpus's second registered consumer, and close F-2's repayment honestly.
- **Method:** Implement `SurfaceProjection` over this surface's **real** projection — its generated catalog, its advertised binders, and its actual dispatch path — and drive the shared corpus from `crates/tui`'s own test target, following the sibling surface's registration as the precedent for shape (isolated fixture registry, real handlers, schema read back from what the surface actually advertises rather than from the descriptor it was given). Then re-audit F-2's four SP-4 conditions against what is now true and update `seed_inventory()` accordingly.
- **Verify:** `cargo test -p cronus-tui` includes a corpus run whose reports are **exactly** the already-accepted residual set (the two redaction fixtures, for the same empty-secret-list reason every surface carries) and nothing else — anything further is a genuine new finding and must be recorded as one before it is fixed. F-2's repayment reflects reality: copies deleted (T-28A01), deletion pinned (its tombstone), fixture landed, consumer registered. If all four hold, F-2 reads repaid and the test that makes a repaid transition visible names it explicitly, as it already does for F-3.
- **Status:** Todo
- **Notes:** This is the phase's replacement oracle, not a formality — see the Phase Notes on why this crate's own unit tests are a weaker behaviour-preservation net than the sibling surface had. Expect F-1's `consumer_registered` to remain **false** even now: F-1 names three surfaces and the desktop shell has still not registered.

### [T-28T02] Behaviour-preservation proof and full quality gates

- **Goal:** Prove the migration changed structure and not behaviour, except where this phase's own tasks changed behaviour deliberately and said so.
- **Method:** Confirm every intentional behaviour change is recorded at its own task (the shrunk slash catalog, the reversed bare-invocation default, the retired executable name, panels distinguishing unavailable from empty) and that nothing else moved. Confirm the residuals this phase does **not** own still behave as before and remain recorded at their owning sites. Run the workspace gates.
- **Verify:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` (zero), `cargo test --workspace --exclude cronus-desktop` green across **two consecutive full runs**; no `unwrap()`/`panic!()` introduced on production paths (audit every file this phase created or substantially modified, `#[cfg(test)]` modules excluded); containment self-check clean across every touched file.
- **Status:** Todo
- **Notes:** The prior phase's closing task found that three of its four named residuals had never actually been recorded *inline at their owning sites*, only in planning prose — check this phase's own new residuals the same way rather than trusting the write-ups.
