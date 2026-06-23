# Nodus Workspace Specifications Registry

**Version:** 1.0.2
**Status:** Active
**Engine Version:** 2.1.46

## Overview

Nodus is a purpose-built workflow DSL and its Rust runtime — vendored in the Cronus monorepo under `crates/nodus` and intended to become a standalone library. This workspace covers nodus as a self-contained tool: language design, grammar, type system, and public API. Integration with Cronus as a host (step binding, subsystem dispatch, platform constraints) is covered in `.design/main/`.

## Domain Specifications

| File | Description | Status | Layer | Version |
| --- | --- | --- | --- | --- |
| [l1-nodus-language.md](specifications/l1-nodus-language.md) | Nodus DSL language: file types (§wf/§schema/§config), section grammar (@ON/!!rules/!PREF/@in/@out/@ctx/@err/@steps/@test/@macro), step syntax (+modifier/^validator/~flag/→pipeline), control flow (?IF/~FOR/~UNTIL MAX:n/~PARALLEL), macro invocation (RUN(@macro_name)), error taxonomy (11 NODUS:* codes), 10 core invariants (NL-1…NL-10); ~PARALLEL fail-fast error propagation | Stable | 1 | 1.0.1 |
| [l2-nodus-runtime.md](specifications/l2-nodus-runtime.md) | Rust crate `crates/nodus`: module structure (lexer/parser/ast/vocab/validator/executor/transpiler/workflows), invariant compliance table (NL-1…NL-10), Value enum (Null/Bool/Int/Float/Text/List/Map), executor boot sequence (6 steps), public API (scaffold/validate/run/transpile/test/run_with_provider), ModelProvider trait + StubProvider, vocabulary schema v0.4.5 (50 commands verified); RUN meta-command gap documented | Stable | 2 | 1.0.1 |

## Meta Information

- **Maintainer**: Core Team
- **Last Updated**: 2026-06-23
