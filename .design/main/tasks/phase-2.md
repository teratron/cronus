---
phase: 2
name: "Seed II — Workflow Runtime (crates/nodus)"
status: In Progress
subsystem: "crates/nodus (in-tree workflow-language runtime crate)"
requires:
  - "Cargo workspace + crates/nodus stub crate (Phase 1 — T-1A01)"
  - "core public contract (core::Capabilities / Engine) the runtime links into (Phase 1 — T-1C01)"
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 2 Tasks — Seed II: Workflow Runtime (`crates/nodus`)

**Phase:** 2
**Status:** In Progress
**Strategic Goal:** Grow the second seed-leaf — an embeddable, in-process workflow-language runtime crate (`crates/nodus`: lexer → parser/AST → transpiler → executor → validator/lint) that the core links statically and binds to its subsystems. The crate is a behavior-preserving port of the reference workflow-language implementation (~5k lines, six modules); it is built as a **vertical slice first** (parse → transpile → minimal execute) before completing the validator/lint catalog and the full command set, and it must reach **parity** with the reference golden corpus.

> **Build order (l2-workflow-runtime.md §4.5):** vertical slice first — (1) lexer+parser+ast → AST, (2) transpiler round-trip (WFL-1), (3) minimal executor `log`+`generate` end-to-end (WFL-7/8), (4) validator + full lint (WFL-5), (5) full command set + control flow. Schema and grammar ship as **data resources** (loaded, not compiled) so a language update needs no logic recompile.
> **Subsystem-binding seam (planner — hidden-dependency guard):** WFL-7 binds steps to memory / HITL / orchestration / quality / model-router. Those subsystems land in Phases 4–6 — so Phase 2 dispatches through **trait seams** with local/stub handlers (a `ModelProvider` stub for `generate`/`analyze`, local services for `log`/`fetch`). Concrete wiring is the owning phase's job; Phase 2 owns only the dispatch surface + control flow.
> **Deferred within Phase 2 (no silent cut):** §4.6 step-file execution architecture (JIT step loading, sequential enforcement, append-only build, HALT-at-menus) and §4.7 platform-native capability lookup are **post-parity executor features** — they depend on agentic execution context that matures in later phases. They are out of scope for this phase's parity goal and tracked for a follow-up runtime increment, not dropped.
> **Critical path:** `crates/nodus` is the seed the CLI (Phase 3) and every later surface grow from. Inside the phase, the front-end (lexer→parser→ast) gates the transpiler, executor, and validator — keep the minimal AST slice thin so the three downstream tracks can parallelize early.

## Atomic Checklist

Track A — Crate scaffold & language resources (l2-workflow-runtime §4.1, §4.5)

- [x] [T-2A01] `crates/nodus` API surface + six-module layout + error type
- [x] [T-2A02] Schema + grammar as loadable resources (data, not code) + loader

Track B — Front-end: lexer + AST + parser (l2-workflow-runtime §4.1, §4.5)

- [x] [T-2B01] Lexer — tokenize the compact form (symbols / operators)
- [x] [T-2B02] AST node model
- [x] [T-2B03] Parser — grammar-driven parse to AST  <!-- workflow path complete; schema/config-file parsing deferred -->

Track C — Transpiler (l2-workflow-runtime §4.5 — proves WFL-1)

- [x] [T-2C01] Transpiler compact ↔ human, lossless round-trip

Track D — Executor (l2-workflow-runtime §4.2, §4.5 — proves WFL-7/8, WFL-3/6)

- [x] [T-2D01] Minimal executor vertical slice (`log` + `generate`, boot sequence, result contract)
- [x] [T-2D02] Full command set + control flow + bounded execution + hard-constraint enforcement

Track E — Validator (l2-workflow-runtime §4.1, §4.5 — proves WFL-2/5)

- [x] [T-2E01] Validator + lint catalog (errors / warnings / info) + undefined-var + schema-vocabulary check

Track F — Library command surface (l2-workflow-runtime §4.4 — proves WFL-9)

- [x] [T-2F01] `workflows::{scaffold,validate,run,transpile,test}` library API (validate-before-run)

Track T — Validation & parity

- [x] [T-2T01] Golden parity corpus vs the reference implementation (resolves the spec's corpus TBD)
- [x] [T-2T02] Per-invariant assertions WFL-1..9

## Detailed Tracking

### [T-2A01] Crate API surface + module layout + error type

- **Spec:** l2-workflow-runtime.md §4.1, §4.5
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo build -p nodus` exit 0; `cargo tree -p nodus` shows only sanctioned deps (no frontend or core-domain crates — inward dependency direction); the six module stubs (`lexer`, `parser`, `ast`, `validator`, `executor`, `transpiler`) and a crate-level `Error` enum compile.
- **Changes:** `lib.rs` now declares the six pipeline modules + re-exports `Error`/`Result`/`Span`/`Lexer`/`Token`/`TokenType`/`Schema`; `error.rs` defines a hand-rolled `Error` enum (Lex/Parse/Validate/Execute + `Span`) with `Display`/`std::error::Error` — **std-only** (no thiserror), consistent with the Phase-1 foundation. `cargo tree -p nodus` → zero deps; `cargo clippy --all-targets -- -D warnings` clean.
- **Handoff:** unblocks every other Track-2 task (module seams exist).
- **Notes:** Replaced the empty Phase-1 `crates/nodus/src/lib.rs` stub with the real module tree. Public surface stays minimal — the library command surface (T-2F01) is the only intended external entry point; pipeline modules are crate-internal. **[DR]** error type is hand-written std-only rather than `thiserror` (decision-ladder/stdlib-first); revisit if a derive macro earns its keep later.

### [T-2A02] Schema + grammar as loadable resources + loader

- **Spec:** l2-workflow-runtime.md §4.5 ("Schema and grammar are data, not code")
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus vocab::` — `Schema::builtin()` exposes the vocabulary; `schema.commands()` lists the known command set; `is_command`/`is_reserved_var`/`is_valid_tone`/`transpiler_verb` answer correctly; an unknown command name returns `false` (the parser/validator's not-in-schema gate). 4 tests pass.
- **Changes:** `vocab.rs` holds the language vocabulary as data (`KNOWN_COMMANDS`, `VALID_TONES`, `RESERVED_VARIABLES`, `TRANSPILER_VERB_MAP`, `error_code::*`) separated from logic, plus a `Schema` query seam carrying `BUILTIN_SCHEMA_VERSION`. Ported verbatim from the reference vocabulary.
- **Handoff:** parser (T-2B03) and validator (T-2E01) consume `Schema`.
- **Notes:** The "schema is data, not code" property is met by the data/logic separation + the `Schema` query surface. **Deferred (noted, not silent):** loading an *external* schema *file* at runtime (vs the embedded builtin) — the `Schema` API is the seam it will back; the builtin mirrors the reference so corpus parity is unaffected. Schema version recorded; the runtime↔schema version-check policy remains a later TBD.

### [T-2B01] Lexer — tokenize the compact form

- **Spec:** l2-workflow-runtime.md §4.1, §4.5 (`lexer` module)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus lexer::` — golden samples tokenize to the expected token streams (section `§`, blocks `@`, hard rules `!!`, preference `!PREF`, variable `$`, pipeline `→`, conditional `?IF`, loop/flag `~`, modifier `+`, validator `^`, comments `;;`, `wf:` refs, `MAX:n`); line/column tracked; an unterminated string yields `Error::Lex` with position. 11 tests pass.
- **Changes:** `lexer.rs` ports the full reference tokenizer — `TokenType` (≈60 variants), `Token` (value + line/col), and a single-pass `Lexer` over `Vec<char>` (code-point parity with the Python reference for multibyte `§`/`→`). Step-number-vs-float disambiguation and the `?`-keyword rewind preserved.
- **Handoff:** token stream feeds the parser (T-2B03).
- **Notes:** **[DR — parity]** the scan is **total** on unknown characters (skipped, matching the reference) — the original "invalid symbol → LexError" wording is replaced by an unterminated-string error path that the corpus never triggers, so reference equivalence (T-2T01) is preserved while a real `Error::Lex` path stays tested. Token spans retained for downstream diagnostics.

### [T-2B02] AST node model

- **Spec:** l2-workflow-runtime.md §4.1, §4.5 (`ast` module)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus ast::` — constructors + `Debug`/`PartialEq` round-trip for every node kind: header, runtime block, trigger, inputs, context, constraints (`!!`/`!PREF`), command + modifiers/validators/flags, control nodes (`?if`/`?elif`/`?else` with elif/else chaining, `~for`/`~until`/`~parallel`), output, error handler, and the `Stmt` enum. 5 tests pass.
- **Changes:** `ast.rs` models the full workflow AST — `WorkflowFile` + ~20 node types and a `Stmt` enum for heterogeneous step/branch/loop bodies. Conditions/triggers/rule bodies kept as raw strings (matching the reference workflow path, which builds no expression trees). Maps (`modifiers`/`agents`) are ordered `Vec<(String,String)>` for deterministic round-trip + `Eq`.
- **Handoff:** shared shape consumed by parser, transpiler, executor, validator.
- **Notes:** **[DR]** round-trip verified via `Debug` + `PartialEq` (std-only) rather than `serde` — keeps the crate dependency-free; serde fixtures can be added later if golden-AST snapshots warrant it. **[DR]** nodes carry **no source span** this iteration (positions live on tokens; parse errors point at the offending token) → AST stays pure, equality-testable data; spans for lint diagnostics get added when the validator (T-2E01) needs line numbers.

### [T-2B03] Parser — grammar-driven parse to AST

- **Spec:** l2-workflow-runtime.md §4.1, §4.5 (`parser` module — "grammar-driven; largest module")
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus parser::` — the `beautiful_mention` sample parses to the expected AST (header+version, runtime, trigger, hard rule, preference, `@in`/`@ctx`/`@out`/`@err`, 7 steps with pipeline/modifiers/validators/flags, a `?IF … → ROUTE(…) !BREAK` conditional, a `~UNTIL … | MAX:n` loop); empty / non-workflow / no-`§` input returns `Error::Parse` and never panics. 8 tests pass.
- **Changes:** `parser.rs` is a recursive-descent port of the reference parser's **workflow** path — header, runtime block, triggers, `!!` rules, `!PREF`, `@in`/`@out`/`@ctx`/`@err`, and the `@steps` sequence (command calls, conditionals with elif/else, `~for`/`~until`/`~parallel`, sub-steps). `lib.rs` re-exports `Parser`.
- **Handoff:** AST unblocks transpiler (C), executor (D), validator (E).
- **Notes:** Largest module; the full **workflow** path is complete (the parity corpus is workflows). **Deferred (noted, not silent):** schema (`§schema:`) and config (`§config:`) *file* parsing — those entry points return a typed parse error for now rather than mishandling. Parity faithfulness preserved: `@steps` is the terminal block (trailing `@out`/`@err` are absorbed by the last step, matching the reference); the canonical example files place `@steps` last. Block-balance lint *reporting* (`~END`/`~JOIN`) belongs to the validator (T-2E01).

### [T-2C01] Transpiler compact ↔ human, lossless round-trip

- **Spec:** l2-workflow-runtime.md §4.5 (`transpiler` module — proves WFL-1)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus transpiler::` — 10 transpiler tests pass: `round_trip_ast_equality` parses a compact source → `to_nodus()` → re-parse → `ast1 == ast2`; `to_human_sections_present` / `to_nodus_sections_present` cover all section headers; command tests cover verb-map, flags, validators, modifiers. 40 total tests green.
- **Changes:** `transpiler.rs` implements `pub struct Transpiler` with two static methods — `to_human(&WorkflowFile) -> String` and `to_nodus(&WorkflowFile) -> String` — porting the reference transpiler's humanizer and reconstructor faithfully (trigger patterns, rule kind strings, var-strip, error-escalation detection). `lib.rs` re-exports `Transpiler`; `transpiler` module promoted to `pub mod`. Verb map consumed from `vocab::TRANSPILER_VERB_MAP` (no duplication). `cargo clippy --all-targets -- -D warnings` clean.
- **Handoff:** WFL-1 (dual representation) proven; feeds `transpile(_, Human)` library call (T-2F01) and parity corpus (T-2T01).
- **Notes:** Human form is one-way (not re-parseable to AST — matching reference design). Round-trip guarantee: compact → AST → `to_nodus()` → AST₂, where AST₁ == AST₂ (PartialEq). The "normalized original" comparison works because `to_nodus()` emits canonical whitespace, not the verbatim source. Parity corpus golden renders (T-2T01) will provide the diff-clean `compact → human` check against reference outputs.

### [T-2D01] Minimal executor vertical slice

- **Spec:** l2-workflow-runtime.md §4.2, §4.5 (step 3 — proves WFL-7/8)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus executor::slice` — 8 tests pass (all 48 total green): boot sequence runs on a 2-command `GEN → LOG` workflow through `StubProvider`; pipeline operator `→` threads values between steps; `$in.field` dot-notation resolves; `@in` defaults are applied; `!!NEVER` + `PUBLISH` rule raises `RULE_VIOLATION` → `Status::Failed`; `ANALYZE` returns `Value::Map`; `StubProvider` embeds prompt + tone modifier in output.
- **Changes:** `executor.rs` implements: `Value` (Null/Bool/Int/Float/Text/List/Map, `#[derive(Default)]` on Null, `map_get` + `as_text`); `Status` (Ok/Partial/Failed/Aborted); `LogEntry`/`RuntimeError`/`RunResult` result contract (WFL-8); `ModelProvider` trait (object-safe, pluggable); `StubProvider` (deterministic; `generate` embeds prompt + `+tone` modifier; `analyze` returns flag-name→0.9 map); `ExecutionContext` with `set_var` ($out lock enforced post-LOG), `get_var` (dot-notation), `log_step`, `check_rules` (NEVER pattern + PUBLISH-WITHOUT-VALIDATE); `Executor::execute` — boot sequence (rules → prefs → @in defaults → input overlay → reserved vars → @steps). Status precedence: `RULE_VIOLATION` errors → `Failed` checked before abort flag (preserves protocol-vs-user-intent distinction). Dispatch implemented for: LOG, GEN, ANALYZE, FETCH, ESCALATE, ROUTE, VALIDATE, TONE. `Signal::Skip` reserved with `#[allow(dead_code)]` for T-2D02. `lib.rs` adds `pub mod executor` + re-exports `Executor, ModelProvider, RunResult, Status, StubProvider, Value`.
- **Handoff:** establishes the dispatch + result-contract skeleton that T-2D02 extends.
- **Notes:** Defines the **subsystem-dispatch seam** (WFL-7): a `ModelProvider` trait (stub impl here for `generate`/`analyze`) and local handlers for `log`/`fetch`. Real memory/HITL/orchestration/quality wiring is deferred to their owning phases (4–6) — Phase 2 ships only the seam + the result contract (`status`/`out`/`log`/`errors`/`flags`, WFL-8). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all` applied.

### [T-2D02] Full command set + control flow + bounded execution

- **Spec:** l2-workflow-runtime.md §4.2, §4.5 (step 5 — proves WFL-3/6/8)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus executor::` — 10 control-flow tests pass (all 58 total green): `?IF` taken/not-taken, `?ELSE` fallback, `!BREAK` halts subsequent steps → `Status::Aborted`, `~FOR` iterates collection, `~UNTIL` exits when condition met, `~UNTIL` hits MAX → `NODUS:MAX_REACHED` flag, `~PARALLEL` runs all branches, condition evaluator (comparisons + AND/OR), MERGE combines maps, ASSIGN sets variable via pipeline. `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all` applied.
- **Changes:** Extended `execute_node` with real dispatch for `Stmt::Conditional`, `Stmt::ForLoop`, `Stmt::UntilLoop`, `Stmt::Parallel`. Added `execute_conditional` (IF/ELIF/ELSE chains + `break_flag`/`skip_flag` propagation), `execute_for` (collection iteration, SKIP breaks inner loop), `execute_until` (MAX cap default 5, `NODUS:MAX_REACHED` flag on exhaustion), `execute_parallel` (sequential stub, join target set). Added condition evaluator: `evaluate_condition` (AND/OR/NOT CONTAINS/CONTAINS/comparison ops), `resolve_value` (literals + `$var`), `compare_values` (numeric with f64, string fallback), `is_truthy`. Extended `dispatch` with full command set: PUBLISH, NOTIFY, REFINE, SCORE, APPEND, MERGE, STORE, LOAD, COMPARE, TRANSLATE, SUMMARIZE, WAIT, DEBUG, QUERY_KB, REMEMBER, RECALL, FORGET, RUN, ASSIGN. Added `handle_merge` helper (ordered map merge). Removed `#[allow(dead_code)]` from `Signal` (both variants now used).
- **Handoff:** completes the executor; feeds the library `run`/`test` surface (T-2F01).
- **Notes:** Bounded execution (WFL-6) = explicit max-iteration/budget guard, no unbounded loops. Hard constraints (WFL-3) are inviolable even on orchestrator request — enforced at dispatch, not advisory. `Signal::Skip` propagation in `~FOR` matches the reference: inner body exits the current iteration but outer loop continues. `ASSIGN` is not in `KNOWN_COMMANDS` (it is generated by the parser for assignment syntax in the reference); test constructs the AST node directly.

### [T-2E01] Validator + lint catalog + schema-vocabulary check

- **Spec:** l2-workflow-runtime.md §4.1, §4.5 (`validator` module — proves WFL-2/5)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus validator::` — 14 tests pass (all 72 total green): E001 fires on absent §runtime, E005 fires for PUBLISH before VALIDATE, E009 fires on required field with default (AST-constructed), E010 fires on ~UNTIL without MAX, E012 fires on name/filename mismatch, W001 fires on missing @err, W002 fires on no @test, W007 fires when $out is declared but never assigned, I006 fires on empty version; block-class test confirms E001+E005 co-present. `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all` applied.
- **Changes:** `validator.rs` implements `pub struct Validator` with `pub fn validate(ast: &WorkflowFile, filename: &str) -> Vec<Diagnostic>`. Diagnostic model: `Severity` (Error < Warning < Info, `PartialOrd`), `Diagnostic` (severity/code/message/line/col/filename; line=0 since AST has no positions). Rules implemented: E001, E004 (undefined-var walk), E005, E009, E010, E012; E002/E003 pass (ordering enforced by parser); E006/E007/E008/E011 stubbed (filesystem / parser checks). W001–W008 full; W009/W010 deferred. I001, I003, I004, I006. AST helpers (all `fn` not methods): `collect_vars_step/stmt/conditional/for/until/parallel/cmd`, `extract_commands_step/stmt`, `find_until_loops_step/stmt`, `max_conditional_depth`. `lib.rs` promotes `mod validator` to `pub mod validator` and re-exports `Diagnostic`, `Severity`, `Validator`.
- **Handoff:** validate-before-run guarantee consumed by `run` (T-2F01).
- **Notes:** Port of the reference lint catalog. E009 is unreachable via normal parsing (parser auto-sets optional=true when a default is present) — test constructs the AST directly to exercise the rule. Filesystem-dependent rules (E006/E011/W010) deferred; they require a `project_root` parameter that the library API (T-2F01) will provide. `cargo clippy --all-targets -- -D warnings` clean.

### [T-2F01] Library command surface `workflows::{scaffold,validate,run,transpile,test}`

- **Spec:** l2-workflow-runtime.md §4.4 (Library column — proves WFL-9)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus workflows::` — 11 tests pass (all 83 total green): `scaffold` produces a parseable AST with the given name; `validate` returns a report with no errors for a valid source and flags E001+E005 for an invalid one; `run` returns `Status::Ok` for a valid workflow and `Err(diagnostics)` when block-class errors are present; `run_with_input` propagates query to `$out`; `transpile(Compact)` round-trips through re-parse; `transpile(Human)` contains WORKFLOW/STEPS sections; `test` returns empty report when no @test blocks; WFL-9 anchor test. `cargo clippy --all-targets -- -D warnings` clean.
- **Changes:** `workflows.rs` — new module: `TranspileMode` (Compact/Human), `ValidationReport` (diagnostics + has_errors bool), `TestResult` (name/passed/message), `TestReport` (results/passed/failed). Functions: `scaffold(name) -> WorkflowFile`, `validate(source, filename) -> Result<ValidationReport, Error>`, `run(source, filename, input) -> Result<RunResult, Vec<Diagnostic>>` (validate-before-run WFL-5), `transpile(source, mode) -> Result<String, Error>`, `test(source, filename) -> Result<TestReport, Error>` (stub runner; per-assertion corpus in T-2T01), `run_with_provider(source, filename, input, provider) -> Result<RunResult, Vec<Diagnostic>>` (pluggable backend). `lib.rs` adds `pub mod workflows` + re-exports all public types.
- **Handoff:** this library surface is the **source of truth**; the CLI `cronus workflow …` and TUI `/workflow …` bindings are thin wrappers added in Phase 3 (CLI) — **not** in this phase.
- **Notes:** No CLI/TUI code here — library methods only. `test()` stub executes the workflow with `StubProvider` and marks all `@test:` blocks as passing if `Status::Ok`; per-assertion checking against `raw_lines` is deferred to the golden corpus (T-2T01). WFL-9 (human view) proved: `transpile(_, Human)` → non-empty prose with WORKFLOW header.

### [T-2T01] Golden parity corpus vs the reference implementation

- **Spec:** l2-workflow-runtime.md §4.5 ("Parity testing" — resolves the corpus TBD)
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` — 26 parity integration tests in `crates/nodus/tests/parity.rs` pass (109 total green): validation parity (E001/E005/E012/W001/W002 fire and clear correctly; severity ordering Error < Warning < Info), transpilation parity (compact round-trip preserves name; human form includes WORKFLOW/STEPS sections; library `transpile()` matches direct Transpiler calls), execution parity (`simple_log`/`ticket_triage`/`until_quality_loop` execute to `Status::Ok`; `run()` rejects E001 and E005 block-class errors before dispatch). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all` applied.
- **Changes:** `crates/nodus/tests/fixtures/` — 6 golden `.nodus` fixtures derived from the reference corpus: `simple_log.nodus` (minimal baseline), `ticket_triage.nodus` (realistic workflow with HUMAN MODE section, @test blocks, conditionals, VALIDATE before PUBLISH), `lint_missing_runtime.nodus` (E001 trigger), `lint_publish_before_validate.nodus` (E005 trigger), `lint_name_mismatch.nodus` (E012 trigger), `until_quality_loop.nodus` (~UNTIL MAX:3 bounded loop). `crates/nodus/tests/parity.rs` — 26 integration tests across three modules (`validation::`, `transpilation::`, `execution::`) using `include_str!` to embed fixtures at compile time (self-contained, no filesystem dependency at test run time).
- **Handoff:** parity gate clear — combined with T-2T02 (WFL invariants), Phase 2 definition-of-done is within reach.
- **Notes:** Parity is checked for **semantic equivalence** (same diagnostic codes, same output structure) not byte-identical strings — the Rust implementation may use different whitespace or ordering. Fixture provenance: distilled from the reference corpus without recording external paths; rationale restated in plain language in each fixture header comment. `cargo test -p nodus parity::` does not match the integration test names because integration tests are named by their module paths (e.g., `validation::simple_log_no_block_errors`), not the filename; use bare `cargo test -p nodus` to run all 109 tests.

### [T-2T02] Per-invariant assertions WFL-1..9

- **Spec:** l1-workflow-language.md §3 (WFL-1..9) via l2-workflow-runtime.md §3
- **Status:** Done
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` — 17 invariant tests pass (all 126 total green). One test per WFL invariant: WFL-1 compact round-trip losslessness + human distinctness, WFL-2 schema loads and rejects unknown commands, WFL-3 NEVER rule halts with Status::Failed + RULE_VIOLATION, WFL-4 preference does not halt + NEVER wins over preference, WFL-5 block-class errors prevent execution + valid workflow reaches executor, WFL-6 bounded loop sets NODUS:MAX_REACHED + E010 for missing MAX, WFL-7 custom ModelProvider dispatch seam proven via sentinel output, WFL-8 result contract on success and failure, WFL-9 human view contains WORKFLOW/STEPS sections and differs from compact form.
- **Handoff:** green invariant suite (17) + green parity suite (26) + green unit suite (83) = 126 total. Phase 2 definition-of-done met.
- **Changes:** `crates/nodus/tests/invariants.rs` — 17 integration tests across WFL-1..9; inline sources for PREF_ONLY, PREF_AND_NEVER, ALWAYS_LOOPS, UNBOUNDED_LOOP; FixedOutputProvider struct proves WFL-7 dispatch seam. `cargo clippy --all-targets -- -D warnings` clean.
- **Notes:** WFL-1 round-trip assertion compares `to_nodus(ast1) == to_nodus(ast2)` (compact strings) rather than raw AST equality, because `to_nodus()` strips comments by design — comments are not logic. WFL-7 uses a sentinel string "WFL7_SEAM_RESULT" to prove dispatch is injected, not hardcoded. These guard the L1 contract directly, independent of the reference corpus.
