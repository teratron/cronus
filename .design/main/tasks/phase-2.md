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

- [ ] [T-2C01] Transpiler compact ↔ human, lossless round-trip

Track D — Executor (l2-workflow-runtime §4.2, §4.5 — proves WFL-7/8, WFL-3/6)

- [ ] [T-2D01] Minimal executor vertical slice (`log` + `generate`, boot sequence, result contract)
- [ ] [T-2D02] Full command set + control flow + bounded execution + hard-constraint enforcement

Track E — Validator (l2-workflow-runtime §4.1, §4.5 — proves WFL-2/5)

- [ ] [T-2E01] Validator + lint catalog (errors / warnings / info) + undefined-var + schema-vocabulary check

Track F — Library command surface (l2-workflow-runtime §4.4 — proves WFL-9)

- [ ] [T-2F01] `workflows::{scaffold,validate,run,transpile,test}` library API (validate-before-run)

Track T — Validation & parity

- [ ] [T-2T01] Golden parity corpus vs the reference implementation (resolves the spec's corpus TBD)
- [ ] [T-2T02] Per-invariant assertions WFL-1..9

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
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus transpiler::` — corpus round-trip: `compact → human → compact` equals the normalized original for every fixture, and `compact → human` is diff-clean against the golden human renders. Both directions go through the AST (T-2B02), proving the dual representation is lossless (WFL-1).
- **Handoff:** the human render is what the library `transpile(_, Human)` (T-2F01) and the client surface (later phases) display (WFL-9).
- **Notes:** "Normalized original" = canonical whitespace/ordering, so round-trip compares logic not formatting.

### [T-2D01] Minimal executor vertical slice

- **Spec:** l2-workflow-runtime.md §4.2, §4.5 (step 3 — proves WFL-7/8)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus executor::slice` — a 2-command workflow (`LOG` + `GEN` through a `StubProvider`) runs the boot sequence (load schema → read `!!` rules → read `!PREF` → register `@in`/`@ctx` → match `@ON` → execute `@steps`) and returns a structured `Result{ status: Success, out, log }`; the pipeline operator `→` threads each step's output to the next.
- **Handoff:** establishes the dispatch + result-contract skeleton that T-2D02 extends.
- **Notes:** Defines the **subsystem-dispatch seam** (WFL-7): a `ModelProvider` trait (stub impl here for `generate`/`analyze`) and local handlers for `log`/`fetch`. Real memory/HITL/orchestration/quality wiring is deferred to their owning phases (4–6) — Phase 2 ships only the seam + the result contract (`status`/`out`/`log`/`errors`/`flags`, WFL-8).

### [T-2D02] Full command set + control flow + bounded execution

- **Spec:** l2-workflow-runtime.md §4.2, §4.5 (step 5 — proves WFL-3/6/8)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus executor::` — each control construct has a passing test (`?if`/`?elif`/`?else`, `~for`, `~until` with `MAX:n`, `~parallel`/`~join`, `!break`/`!skip`); a `~until` missing `MAX` is rejected; a `!!NEVER`/`!!ALWAYS` violation aborts with a `RULE_VIOLATION` error and runs the declared `@err` handler (escalation); hitting the iteration cap returns a partial result flagged `MAX_REACHED`.
- **Handoff:** completes the executor; feeds the library `run`/`test` surface (T-2F01).
- **Notes:** Bounded execution (WFL-6) = explicit max-iteration/budget guard, no unbounded loops. Hard constraints (WFL-3) are inviolable even on orchestrator request — enforced at dispatch, not advisory. Bind the remaining command categories through the T-2D01 seams (memory/HITL/routing handlers may be local stubs until their phases land).

### [T-2E01] Validator + lint catalog + schema-vocabulary check

- **Spec:** l2-workflow-runtime.md §4.1, §4.5 (`validator` module — proves WFL-2/5)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus validator::` — golden lint cases assert the expected code across the full catalog (block-class errors, runnable-class warnings, advisory info); `PUBLISH` without a prior `VALIDATE` is reported; a `$var` used before assignment is reported (undefined-variable halt, WFL-5); an unknown command/vocabulary token fails against the loaded schema (WFL-2); running with no schema is an error. `run` invokes the validator first and halts on any block-class finding.
- **Handoff:** validate-before-run guarantee consumed by `run` (T-2F01).
- **Notes:** Port the reference lint catalog (errors block execution / warnings runnable / info advisory) plus the runtime error-code set. Three severities with distinct exit semantics: block-class halts, warning/info do not.

### [T-2F01] Library command surface `workflows::{scaffold,validate,run,transpile,test}`

- **Spec:** l2-workflow-runtime.md §4.4 (Library column — proves WFL-9)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus api::` — `scaffold(name) -> Workflow`, `validate(ref) -> Report`, `run(ref, input) -> Result`, `transpile(ref, mode) -> String`, `test(ref?) -> Report` each covered; `run` calls `validate` before `execute` and fails fast on a block-class finding; `transpile(_, Human)` returns the human render (WFL-9); `test` executes inline `@test:` blocks and reports pass/fail.
- **Handoff:** this library surface is the **source of truth**; the CLI `cronus workflow …` and TUI `/workflow …` bindings are thin wrappers added in Phase 3 (CLI) — **not** in this phase.
- **Notes:** No CLI/TUI code here — library methods only (per l2-cli.md: the library method is the source of truth, frontends are thin bindings).

### [T-2T01] Golden parity corpus vs the reference implementation

- **Spec:** l2-workflow-runtime.md §4.5 ("Parity testing" — resolves the corpus TBD)
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus parity::` — the reference golden corpus (sample workflows + lint cases) yields **equivalent** validation verdicts, execution results, and transpilation output; corpus fixtures live under `crates/nodus/tests/fixtures/`.
- **Handoff:** parity is the phase's definition-of-done gate.
- **Notes:** Extract the reference implementation's sample workflows + lint cases into shared in-repo fixtures (this resolves the spec's open corpus item). Reference materials are mined privately; **do not** record any external/local path in fixtures, specs, or source — restate provenance in plain language only.

### [T-2T02] Per-invariant assertions WFL-1..9

- **Spec:** l1-workflow-language.md §3 (WFL-1..9) via l2-workflow-runtime.md §3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus invariants::` — one named test per invariant: WFL-1 dual-render round-trip, WFL-2 schema-load-first, WFL-3 hard-constraint halt, WFL-4 preference-soft (never overrides a hard rule), WFL-5 validate-before-run, WFL-6 bounded execution, WFL-7 subsystem-dispatch seam present, WFL-8 result-contract shape, WFL-9 human-view render.
- **Handoff:** green invariant suite + green parity suite ⇒ Phase 2 Done.
- **Notes:** These guard the L1 contract directly, independent of the reference corpus — they stay valid even if the reference evolves.
