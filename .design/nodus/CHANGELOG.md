# Nodus Workspace Changelog

Internal phase journal. Each entry corresponds to a completed phase.

## Phase 1 — 2026-06-23

**Spec Completeness & Vocabulary Alignment**

- T-1A01: Defined `~PARALLEL` fail-fast error propagation semantics in l1-nodus-language.md §4.4 — first branch error bypasses `~JOIN`, forwards to `@err:`, `NODUS:RULE_VIOLATION` bypasses `@err:` per NL-2
- T-1A02: Defined `RUN(@macro_name)` macro invocation syntax in l1-nodus-language.md §4.3 — meta-command outside schema vocabulary, recognized before schema validation pass
- T-1B01: Cross-checked l2-nodus-runtime.md §4.6 against `vocab.rs::KNOWN_COMMANDS` — 50 commands, exact match; `BUILTIN_SCHEMA_VERSION = "0.4.5"` confirmed; documented `RUN` vocabulary gap (in `TRANSPILER_VERB_MAP` but not in `KNOWN_COMMANDS`)
- T-1B02: Verified all Canonical References in both specs resolve to existing source paths
- T-1C01: Added Document History table to l1-nodus-language.md; bumped spec to v1.0.1
- T-1C02: Added Document History table to l2-nodus-runtime.md; bumped spec to v1.0.1
- T-1T01: `cargo test -p nodus` — 126 passed, 0 failed (83 unit + 17 invariant + 26 parity); mapped all 10 NL invariants to tests; gaps filed: NL-8 (no validator test for reserved variable shadow) and NL-10 (no validator test for forward reference) → Phase 2 fixture corpus
