# Project Context

**Generated:** 2026-09-02

## Active Technologies

- Node.js
- Rust

## Core Project Structure

```plaintext
.
├── .agents/
│   ├── rules/
│   ├── skills/
│   └── workflows/
├── .cargo/
│   └── config.toml
├── .claude/
├── .codex/
├── .design/
│   ├── .version
│   ├── INDEX.md
│   ├── RULES.md
│   ├── graph-snapshot.json
│   ├── main/
│   ├── nodus/
│   └── workspace.json
├── .drafts/
│   ├── TODO.md
│   ├── UX-UI - разбор 3 кейсов.md
│   ├── UX-дизайн - 6 психологических принципов.md
│   ├── heartbeat.md
│   ├── project-names.md
│   ├── references.md
│   ├── reverse-derivation-mechanism.md
│   ├── technology-stack-research.md
│   └── ui-ux.md
├── .env.example
├── .github/
│   ├── dependabot.yml
│   └── workflows/
├── .gitignore
├── .magic/
├── .markdownlint.json
├── .qwen/
├── .release/
│   ├── program/
│   ├── project/
│   └── state/
├── AGENTS.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── apps/
│   └── desktop/
├── biome.json
├── crates/
│   ├── activation-os/
│   ├── auth-local/
│   ├── cli/
│   ├── codegraph/
│   ├── contract/
│   ├── core/
│   ├── domain/
│   ├── model-local/
│   ├── nodus/
│   ├── store-local/
│   └── tui/
├── docs/
│   └── README.md
├── installer/
├── package.json
├── packages/
│   └── ui/
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── rust-toolchain.toml
└── scripts/
    └── check-domain-boundary.mjs
```

## Recent Changes

- T-30B01: `w017_dialog_payload_inlining` + four free functions (`w017_collect_producers`/`_conditional`, `w017_scan_stmt`/`_conditional`) — a **dedicated** `target-root → producing command name` map (not a reuse of `collect_vars_stmt`, which tracks declared/used sets only), firing on a bare `$var` `ASK`/`CONFIRM` argument whose producer is in `MODEL_COMMANDS`.
- T-30C01: `l2-nodus-dialog.md` 1.2.0 → 1.2.1 — §4.8.3 corrected to the whole-arg reference model (nodus has no interpolation scanner) and the dedicated-walker mechanism; §4.8.4's scope-list bullet corrected to name exactly `?IF`/`?ELIF`/`?ELSE`/`~FOR`/`~UNTIL` plus the `~PARALLEL`/`?SWITCH`/`~MAP` exclusions found at T-30A02. `INDEX.md` row + top-level version synced (1.0.92 → 1.0.93).
- T-30T01/T-30T02: Added 11 unit tests in `validator.rs` (7 `W016` + 4 `W017`, built via direct `WorkflowFile`/`Step`/`Stmt` struct literals for precise nested-block control) plus two small helpers (`cmd_step`, `wf_with_steps`). Ran the full `tests/fixtures/` corpus (14 files) through a scratch example, deleted after use: zero fixtures newly emit either code — checked by hand why (an `@in`-sourced argument with no producer; a branch-action `ASK` that is never a candidate by construction). `cargo test -p nodus` — 482 passed (was 471; +11), 0 failed; clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 31 — Pinned-Generation Digest on `ResumeDescriptor` (l1-nodus-portability §4.11/LP-22(c) · l2-nodus-dialog §4.3) (2026-09-02)

- T-31A01: `ResumeDescriptor` (`executor.rs`) gained `pub workflow_digest: String`; the one construction site populated it with `digest_ast(ast)` — the identical call `ReproRecipe`'s construction a few lines below already makes on the same `ast`. `cargo check`/`clippy` clean on the first pass; no other call site required changes.
- T-31C01: `l2-nodus-dialog.md` 1.3.0 → 1.4.0 (§4.3's `workflow_digest` tags → `[IMPLEMENTED]`, new closing paragraph citing both tests) and `l2-nodus-portability.md` 1.9.0 → 1.10.0 (§3.2 LP-22 row (c) → Implemented, dropped from the "what is plannable" residue list; LP-22 overall stays Partially realized). `INDEX.md` version cells + top-level version synced.
- T-31T01: Added `resume_descriptor_workflow_digest_matches_digest_ast` (unit, `executor.rs`) and `resume_descriptor_digest_agrees_with_repro_recipe_digest` (integration, `tests/dialog.rs`, new `ManifestCapture` `AuditProvider`) — the latter proves `ResumeDescriptor.workflow_digest` agrees with the independently-built `ReproRecipe.workflow_digest` for the same paused run, since the two construction sites are never cross-checked in production code. `cargo test -p nodus` — 484 passed (was 482, +2), 0 failed; clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 32 — NE-12/HO-20 Digest Unification (l2-nodus-environment §4.4 · l2-nodus-observability §4.7) (2026-09-02)

- T-32A01: `digest_ast` (`executor.rs`) widened `fn` → `pub(crate) fn`; `EnvRunResult::candidate()` (`environment.rs`) now matches `Parser::parse(workflow_source)` — `Ok(ast)` hashes `digest_ast(&ast)`, `Err(_)` falls back to `digest_source(workflow_source)` exactly as before. Signature unchanged.
- T-32C01: `l2-nodus-environment.md` 1.2.0 → 1.3.0 (NE-12 row + §4.4 "Implemented [Phase 32]" paragraph) and `l2-nodus-portability.md` 1.11.0 → 1.11.1 (§3.2 cross-reference updated to the implemented version). `INDEX.md` version cells + top-level version synced.
- T-32T01: Added `candidate_digest_agrees_with_repro_recipe_digest` (integration, `tests/environment.rs`, new `ManifestCapture` `AuditProvider` mirroring Phase 31's `tests/dialog.rs` precedent) — proves `CandidateResult.workflow_digest` and `ReproRecipe.workflow_digest` agree for the same run — and `candidate_digest_falls_back_to_source_hash_when_unparseable` (unit, `environment.rs`), pinning the fallback value the pre-existing unparseable-input test never asserted on. `cargo test -p nodus` — 486 passed (was 484, +2), 0 failed; clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

