# Master Task Index (Registry)

**Version:** 2.0.0
**Generated:** 2026-06-24
**Based on:** .design/nodus/PLAN.md v1.8.0
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

## Meta Information

- **Last Updated**: 2026-06-24
- **Phase 3 Planned**: 2026-06-24 (9 tasks, tracks A/B/C/D/T)
- **Phase 4 Complete**: 2026-06-24 (9 tasks, tracks A/B/C/T; all archived)
- **Phase 5 Complete**: 2026-06-24 (7 tasks, tracks A/B/C/T; all archived)
- **Phase 6 Complete**: 2026-06-24 (9 tasks, tracks A/B/C/T; all archived; 204 tests pass)
- **Maintainer**: Core Team
