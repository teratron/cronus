---
phase: 13
name: "Declarative Configuration Surface"
status: Todo
subsystem: "crates/nodus"
requires: [7, 8]
provides: []
key_files:
  created: []
  modified: []
patterns_established: []
duration_minutes: ~
---

# Stage 13 Tasks — Declarative Configuration Surface

**Phase:** 13
**Status:** Todo
**Strategic Goal:** Realize the `§config` declarative-configuration surface (`l2-nodus-config.md`, NL-20) in `crates/nodus` — replace the `§config` parser stub with a field-declaration AST + parser, a pure pre-run shape check, a provenance- and secret-aware accepted-value model, a host-acceptance provider seam, and the `run_with_config` entry points. All-additive, zero new dependency (LP-1). Sequential tracks A→B→C→T.

## Atomic Checklist

- [ ] [T-13A01] Config AST — `ConfigDecl` / `ConfigField` / `FieldConstraint`
- [ ] [T-13A02] `Parser::parse_config` + rewire the `§config` deferral branch
- [ ] [T-13A03] Transpiler `§config` round-trip
- [ ] [T-13B01] `CONFIG_INVALID` error code + registry lockstep
- [ ] [T-13B02] Pure `check_config_values` shape check
- [ ] [T-13B03] `AcceptedConfig` value model — provenance + write-only secrets
- [ ] [T-13C01] `ConfigProvider` + `ExtensionRole::Config` + `builtin()`
- [ ] [T-13C02] `run_with_config` / `run_with_config_and_audit` + lib re-exports
- [ ] [T-13T01] Validation suite — shape check, secret neutrality, LP-8, zero-dep

## Detailed Tracking

### [T-13A01] Config AST types

- **Spec:** l2-nodus-config.md §4.2
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** `cargo test -p nodus` compiles; a unit test constructs a `ConfigDecl` whose `fields` preserve declaration order and cover every clause (default, `Range`, `OneOf`, required, secret, describe).
- **Handoff:** AST is referenced by every later track — land first.
- **Notes:** Add `ConfigDecl { header, fields }`, `ConfigField { name, ty, default, constraint, required, secret, describe }`, `FieldConstraint { Range { lo, hi }, OneOf(Vec<Value>) }` to `ast.rs`. `FileType::Config` already exists; reuse it. Field order preserved for deterministic rendering and stable diagnostics.

### [T-13A02] `parse_config` + rewire deferral branch

- **Spec:** l2-nodus-config.md §4.3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** unit test parses a `§config:` sample into the expected `ConfigField` set; asserts a `§config` header no longer returns the `"schema/config file parsing is not yet implemented"` error; asserts a `§schema` header still returns it (deferral preserved).
- **Handoff:** feeds T-13A03 (round-trip) and T-13B02 (shape check consumes the decl).
- **Notes:** Implement `Parser::parse_config(source) -> Result<ConfigDecl>` reusing `parse_header(FileType::Config)`; read field lines per the §4.1 grammar (`name : type` + optional `default:` / `range:` | `one-of:` / `required` / `secret` / `describe:`). Rewire the `§config` arm of both `parse` and `parse_with_schema` to delegate to `parse_config`; keep the `§schema` arm deferred. Field clauses lex as line-oriented markers within a field block (the `[DR]` in §4.3), mirroring `@in:` field lines.

### [T-13A03] Transpiler `§config` round-trip

- **Spec:** l2-nodus-config.md §4.3
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** round-trip test asserts `parse_config(to_nodus(&decl))` equals `decl` for a multi-field declaration that includes a `secret` field, a `one-of`, a `range`, and a `describe:` string.
- **Handoff:** enables a host editor to serialize an amended declaration back to source.
- **Notes:** Extend `transpiler.rs` so `to_nodus` re-emits a `ConfigDecl`; `secret`, `describe:`, and constraints must survive the trip. Follow the existing round-trip fidelity pattern used for test blocks.

### [T-13B01] `CONFIG_INVALID` error code + lockstep

- **Spec:** l2-nodus-config.md §4.8
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** the `error_registry_lockstep` test passes with the new row; `error_meta("NODUS:CONFIG_INVALID")` returns `(ErrorSeverity::Error, ErrorCategory::Validation)`.
- **Handoff:** the code T-13B02 emits and T-13C02 maps a host `Rejected` to.
- **Notes:** Add the `CONFIG_INVALID` `error_code` constant + `error_meta` row (severity `Error`, category `Validation`) in `vocab.rs`; extend the lockstep test so every canonical constant has metadata and vice-versa. One aggregate code carrying a structured violation list — not a code per reason (the `[DR]` in §4.8).

### [T-13B02] Pure `check_config_values` shape check

- **Spec:** l2-nodus-config.md §4.4
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** unit tests exercise each `ConfigReason` (`UnknownField`, `MissingRequired`, `TypeMismatch`, `OutOfRange`, `NotInEnum`, `BadDefault`); a valid proposed set yields an `AcceptedConfig`; calling the function twice on the same inputs returns identical results (purity / NL-6).
- **Handoff:** produces the `AcceptedConfig` T-13B03 models and T-13C02 boots with.
- **Notes:** `check_config_values(decl, proposed) -> Result<AcceptedConfig, Vec<ConfigViolation>>` in `validator.rs`. Validate in one pass: unknown field, required-present (unless a default exists), type match against the primitive-type registry, constraint satisfaction, and declared-default validity. Report ALL violations, apply NONE (never mutate a prior accepted set). Missing optionals resolve to their default. Pure and side-effect-free — the NL-1/NL-9 validation-category gate for configuration.

### [T-13B03] `AcceptedConfig` value model — provenance + write-only secrets

- **Spec:** l2-nodus-config.md §4.5
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** test asserts an accepted value carries provenance (host-external / unclassified resolves to `untrusted`, NL-11) and that a `secret` field exposes NO accessor rendering it into a model-facing prompt and NEVER appears in an emitted `ExecutionEvent` / `RunManifest` payload (only a `FieldDescriptor` reaches `observability.rs`).
- **Handoff:** the readable-configuration surface the executor binds in T-13C02.
- **Notes:** `AcceptedConfig` yields values by name as ordinary `Value`s with provenance. `secret` is write-only by construction (NL-11 neutralize-by-removing-the-capability), not a redaction filter — there is simply no rendering accessor and no trace path. Mirror the DG-7 descriptor-only exposure already used by dialog events.

### [T-13C01] `ConfigProvider` + `ExtensionRole::Config` + `builtin()`

- **Spec:** l2-nodus-config.md §4.6
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** LP-8 test — a config-consuming run requires `ExtensionRole::Config`; `HostCapabilities::builtin()` satisfies it via `DefaultConfigProvider`; a host whose capabilities omit `Config` is rejected fail-fast with `NODUS:CAPABILITY_UNMET` before executor boot.
- **Handoff:** the acceptance seam T-13C02's entry points call.
- **Notes:** `portability.rs`: `ConfigProvider` trait + `ConfigOutcome { Accepted, Rejected }` + `DefaultConfigProvider` (accepts the shape-checked set as-is, no I/O / no store / no UI). Add `ExtensionRole::Config`; `builtin()` provides it (like `Environment`'s stub, unlike `Dialog`). The `Config` requirement is entry-point-driven — asserted when a run is launched through `run_with_config` — and satisfied by `builtin()` by default; deriving it in `from_workflow` when a config surface is referenced is a secondary nicety, not required for this task.

### [T-13C02] `run_with_config[_and_audit]` + lib re-exports

- **Spec:** l2-nodus-config.md §4.7
- **Status:** Todo
- **Assignment:** Agent
- **Verify:** test asserts a `Rejected` outcome (or shape-check failure) returns `CONFIG_INVALID` BEFORE executor boot with ZERO audit events emitted and any prior accepted set left intact; a happy path binds config values readable by the workflow through the variable/context surface.
- **Handoff:** completes the public API; feeds the T-13T01 integration suite.
- **Notes:** `workflows.rs`: `run_with_config` + `run_with_config_and_audit` sequencing declaration → proposed → `check_config_values` → `provider.accept` → executor boot. A shape-check failure or `Rejected` returns before boot and emits no audit events (rejected runs never start — the observer-neutrality property the manifest gate already has). Re-export the config types and entry points from `lib.rs`.

### [T-13T01] Validation Task — config contract suite

- **Goal:** Verify the `crates/nodus` realization against `l2-nodus-config.md` (NL-20 + the secret/provenance boundary + LP-8), and confirm LP-1 zero-dep is preserved.
- **Method:** `tests/config.rs` integration suite covering: NL-20 shape-check coverage (each `ConfigReason`); the secret-neutrality gate (a `secret` field is never rendered into a model prompt nor present in any emitted event payload); the LP-8 fail-fast path (`Config` role required, `builtin()` satisfies, stripped host → `CAPABILITY_UNMET`); and the full declaration → proposed → acceptance → run happy path. Run `cargo test -p nodus --test config` (green) and confirm `cargo tree -p nodus -e no-dev` shows no new dependency versus the prior lock.
- **Status:** Todo
