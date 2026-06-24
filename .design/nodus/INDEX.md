# Nodus Workspace Specifications Registry

**Version:** 1.0.5
**Status:** Active
**Engine Version:** 2.1.46

## Overview

Nodus is a purpose-built workflow DSL and its Rust runtime — vendored in the Cronus monorepo under `crates/nodus` and intended to become a standalone library. This workspace covers nodus as a self-contained tool: language design, grammar, type system, and public API. Integration with Cronus as a host (step binding, subsystem dispatch, platform constraints) is covered in `.design/main/`.

## Domain Specifications

| File | Description | Status | Layer | Version |
| --- | --- | --- | --- | --- |
| [l1-nodus-language.md](specifications/l1-nodus-language.md) | Nodus DSL language: file types (§wf/§schema/§config), section grammar (@ON/!!rules/!PREF/@in/@out/@ctx/@err/@steps/@test/@macro), step syntax (+modifier/^validator/~flag/→pipeline), control flow (?IF/~FOR/~UNTIL MAX:n/~PARALLEL), macro invocation (RUN(@macro_name)), error taxonomy (11 NODUS:* codes), 10 core invariants (NL-1…NL-10); ~PARALLEL fail-fast error propagation | Stable | 1 | 1.0.1 |
| [l2-nodus-runtime.md](specifications/l2-nodus-runtime.md) | Rust crate `crates/nodus`: module structure (lexer/parser/ast/vocab/validator/executor/transpiler/workflows), invariant compliance table (NL-1…NL-10, E013/E014 enforced), Value enum (Null/Bool/Int/Float/Text/List/Map), executor boot sequence (6 steps), public API (scaffold/validate/run/transpile/test/run_with_provider), ModelProvider trait + StubProvider, vocabulary schema v0.4.6 (51 commands: 50 domain + RUN meta-command), RUNTIME_OWNED_VARIABLES (9 read-only reserved vars) | Stable | 2 | 1.0.2 |
| [l1-nodus-portability.md](specifications/l1-nodus-portability.md) | Portability and extension contract: host neutrality (LP-1), extension via abstract interfaces (LP-2), two-host generalisation rule (LP-3), vocabulary isolation (LP-4), composable extension (LP-5), semantic versioning (LP-6), feedback loop lifecycle (LP-7); extension point taxonomy (ModelProvider + AuditProvider + future StorageProvider/PolicyProvider), feedback distillation protocol, universal pattern criteria, vocabulary layering model | Stable | 1 | 1.0.1 |
| [l1-nodus-observability.md](specifications/l1-nodus-observability.md) | Execution observability contract: trace-first output (HO-1), per-step attribution (HO-2), append-only immutability (HO-3), frozen evaluation boundary (HO-4), observer neutrality (HO-5), structured event taxonomy (HO-6); AuditProvider role, 10-type event taxonomy (step_start/step_end/step_error/constraint_hit/branch_taken/loop_iteration/macro_enter/macro_exit/model_call/model_response), run manifest, data-safety boundary (no raw user text in traces), frozen-vs-evolvable component table | Stable | 1 | 1.0.0 |

## Meta Information

- **Maintainer**: Core Team
- **Last Updated**: 2026-06-24
