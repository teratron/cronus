# Project Context

**Generated:** 2026-07-31

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
│   ├── commands/
│   ├── rules/
│   ├── scheduled_tasks.lock
│   └── skills/
├── .codex/
│   ├── prompts/
│   ├── rules/
│   └── skills/
├── .design/
│   ├── .cache/
│   ├── .graph-cache/
│   ├── .version
│   ├── INDEX.md
│   ├── RULES.md
│   ├── graph-before.json
│   ├── main/
│   ├── nodus/
│   ├── wiki/
│   └── workspace.json
├── .drafts/
│   ├── TODO.md
│   ├── UX-UI - разбор 3 кейсов.md
│   ├── UX-дизайн - 6 психологических принципов.md
│   ├── desktop.drawio.svg
│   ├── heartbeat.md
│   ├── project-names.md
│   ├── references.md
│   ├── release.drawio.svg
│   ├── technology-stack-research.md
│   └── ui-ux.md
├── .env
├── .env.example
├── .github/
│   ├── dependabot.yml
│   └── workflows/
├── .gitignore
├── .markdownlint.json
├── .release/
│   ├── program/
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

- T-22A01: Added `Transpiler::nodus_braced_raw_body` (re-quotes every `raw_lines` token except literal `{`/`}`, which must stay unquoted since `collect_braced_raw_lines`'s depth counter matches token *type*, not value). Inverted the `@test:` emission branch in `to_nodus` to prefer `raw_lines` when non-empty; structured-field emission is now reached only for a programmatically-constructed `TestBlock`
- T-22A02: Added `Validator::w015_test_pair_separator`, mirroring `parse_test_body`'s own section-tracking control flow so a legitimately-consumed `key : value` triple's value token is never re-examined as a would-be key. Warning-severity; co-fires with `W009` by design when all of a block's `expected:` pairs are non-conforming
- T-22B01: Added `@macro:` emission, reusing `nodus_braced_raw_body` directly. Found the real corpus fixture (`macro_expand.nodus`) uses the non-braced `@macro: name` form whose body `parse_macro_block` never captures (macro body expansion is a separately deferred feature) — `MacroBlock.raw_lines` is empty in the corpus today, so the braced-body path is implemented but untested by any fixture (recorded in PLAN.md Backlog)
- T-22B02: Added `human_mode` emission, deliberately placed **last** in `to_nodus` — `collect_comment_block` greedily consumes every following Comment token, so emitting it earlier would silently absorb any free-standing comment into it on re-parse
- T-22T01: Widened `full_corpus_ast_equal_after_compact_round_trip` (`tests/parity.rs`) from `ast1.steps == ast2.steps` to `ast1 == ast2` — green across the whole 11-fixture normative corpus. Per-task sensitivity confirmed empirically: temporarily disabling `@macro:` emission failed the widened harness with the exact predicted `macros` diff, then restored
- T-22T02: `cargo test -p nodus` — 429 passed (was 420; +9), 0 failed; clippy `-D warnings` clean; fmt clean (1 finding fixed); no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]`; `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

## Phase 23 — Built-in Durable-State Conformance (l1-nodus-portability §4.1/§4.11 · LP-2/LP-15) (2026-07-31)

- T-23A01: Added `InMemoryStorageProvider` to `portability.rs` — `Mutex<Vec<(String, Value)>>`, insert-or-overwrite `store`, clone-on-`load`, poisoned-lock recovery via `unwrap_or_else(|poisoned| poisoned.into_inner())` — replacing `NoopStorageProvider`, whose discarding `store`/always-`None` `load` satisfied neither L1 §4.1's in-memory built-in mandate nor LP-2's in-process-sufficiency requirement
- T-23A02: Updated `lib.rs`'s re-export to `InMemoryStorageProvider`; scoped `portability.rs`'s module doc's "pending LP-3 graduation" note to wiring only, since the built-in itself is now conformant; removed a stale `NoopStorageProvider` citation from `DefaultConfigProvider`'s doc comment
- T-23C01: `l2-nodus-portability` 1.3.0 → 1.3.1 — Overview, §4.2 registry row, §4.3 heading/body, §3.1's LP-15 row, and §5 item 3 (plus its closing sentence) reconciled to the as-built fix; self-review caught two live references the first pass missed (§3's LP-2 row, §4.5's module map), both corrected in the same bump; `INDEX.md` row + top-level version synced
- T-23C02: **Cancelled (superseded)** — its target (correcting §5 item 2's LP-11 framing) was already reached by the prior spec pass, from the opposite direction (opening the gate rather than demoting the claim); nothing left for this task to do
- T-23T01: Split the Phase-5 contract test's storage half out of `noop_storage_and_policy_compile` (renamed `noop_policy_and_schema_compile`, storage assertions removed) into 4 dedicated integration tests — round-trip, overwrite, absent-key, instance-isolation — plus 3 equivalent in-crate unit tests in `portability.rs`
- T-23T02: `cargo test -p nodus` — 435 passed (was 429; +6), 0 failed (every suite `ok`, verified against a precise `test result: FAILED` pattern rather than a naive `passed`/`failed` substring sum, which several `NODUS:*_FAILED` error-code strings would false-positive); clippy `-D warnings` clean; fmt clean; no `.unwrap()`/`panic!()`/`unreachable!()`/`.expect(` outside `#[cfg(test)]` (the one new `lock()` call recovers a poisoned guard); `git diff --stat` on `Cargo.toml`/`Cargo.lock` empty (LP-1 preserved)

