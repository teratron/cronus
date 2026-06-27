# Master Task Index (Registry)

**Version:** 2.3.0
**Generated:** 2026-06-24
**Based on:** .design/nodus/PLAN.md v1.11.0
**Based on RULES:** .design/RULES.md v1.4.0
**Execution Mode:** Sequential
**Status:** Active

## Overview

Tactical registry of all phases. Atomic checklists live in `tasks/phase-{N}.md`.

## Active Phases

| Phase | Description | Status |
| --- | --- | --- |
| [Phase 1](archives/tasks/phase-1.md) | Spec Completeness & Vocabulary Alignment | `Done (Archived)` |
| [Phase 2](archives/tasks/phase-2.md) | Library Hardening: fixture corpus, NL-8/NL-10 enforcement, RUN vocabulary, extraction audit | `Done (Archived)` |

## Active Phases (continued)

| Phase | Description | Status |
| --- | --- | --- |
| [Phase 3](archives/tasks/phase-3.md) | Standalone Extraction: spec sync, Cargo hardening, public API docs, CI workflow, extraction procedure | `Done (Archived)` |
| [Phase 4](archives/tasks/phase-4.md) | Observability & Extension Framework: observability.rs (AuditProvider + 10-event enum), executor hook points (Tracks A/B/C/T), l2-nodus-runtime.md delta v1.0.3 | `Done (Archived)` |

## Active Phases (continued)

| Phase | Description | Status |
| --- | --- | --- |
| [Phase 5](archives/tasks/phase-5.md) | Portability Implementation: portability.rs (SchemaProvider + StorageProvider + PolicyProvider), vocab.rs delta (Schema::with_provider), lexer/parser schema-aware parsing, workflows.rs delta (run_with_schema), l2-nodus-runtime.md v1.0.4 | `Done (Archived)` |

| [Phase 6](archives/tasks/phase-6.md) | Testing Implementation: ast.rs (structured TestBlock), parser.rs (E015 + input/expected parsing), transpiler round-trip, workflows.rs (assertion evaluator + NT isolation), validator.rs (W001/W002), test() tag filtering, tests/testing.rs (6 integration tests), l2-nodus-testing.md spec | `Done (Archived)` |

## Active Phases (continued)

| Phase | Description | Status |
| --- | --- | --- |
| [Phase 7](archives/tasks/phase-7.md) | Capability Manifest (LP-8): portability.rs (CapabilityManifest + ExtensionRole + HostCapabilities), validate_manifest() fail-fast gate wired into executor boot, NODUS:* capability-rejection diagnostic, AST-derived role manifest, run_with_manifest API, LP-3 two-host + pre-run purity tests, l2-nodus-portability.md §4.7 + RFC → Stable | `Done (Archived)` |

## Active Phases (continued)

| Phase | Description | Status |
| --- | --- | --- |
| [Phase 8](archives/tasks/phase-8.md) | Error Taxonomy (l2-nodus-errors): vocab.rs ErrorSeverity/ErrorCategory enums + 14 new error_code constants + severity×category registry + error_meta() lookup; EXECUTION_FAILED deprecated + site reassignment; validator/executor emission; lockstep test | `Done (Archived)` |

## Active Phases (continued)

| Phase | Description | Status |
| --- | --- | --- |
| [Phase 9](archives/tasks/phase-9.md) | Closed Vocabulary Registries (l2-nodus-registries): vocab.rs KNOWN_FLAGS/KNOWN_VALIDATORS/PRIMITIVE_TYPES constants + Schema query surface; advisory validator warnings for unknown ~flag/^validator/@in type; strengthens NL-1/NL-7/NL-9 | `Done (Archived)` |

## Meta Information

- **Last Updated**: 2026-06-27
- **Phase 3 Planned**: 2026-06-24 (9 tasks, tracks A/B/C/D/T)
- **Phase 4 Complete**: 2026-06-24 (9 tasks, tracks A/B/C/T; all archived)
- **Phase 5 Complete**: 2026-06-24 (7 tasks, tracks A/B/C/T; all archived)
- **Phase 6 Complete**: 2026-06-24 (9 tasks, tracks A/B/C/T; all archived; 204 tests pass)
- **Phase 7 Complete**: 2026-06-27 (9 tasks, tracks A/B/C/T/D; all archived; l2-nodus-portability v1.1.0 Stable via LP-8; 217 tests pass)
- **Phase 8 Complete**: 2026-06-27 (7 tasks, tracks A/B/T; all archived; l2-nodus-errors — 24-code taxonomy; 222 tests pass)
- **Phase 9 Complete**: 2026-06-27 (tracks A/B/T; all archived; l2-nodus-registries — closed flag/validator/type registries + advisory W011/W012/W013; 228 tests pass)
- **Backlog**: l1-nodus-dialog (Draft, pending TBD review); remaining parity clusters (control-flow, operators [MATCHES/PCRE design fork], @needs, @ON priority, macro execution); Storage/Policy executor integration (LP-3)
- **Maintainer**: Core Team
