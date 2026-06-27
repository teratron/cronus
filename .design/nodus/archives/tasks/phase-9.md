---
phase: 9
name: "Closed Vocabulary Registries (l2-nodus-registries)"
status: Done
subsystem: "crates/nodus"
requires:
  - phase-8
provides:
  - KNOWN_FLAGS (12) + KNOWN_VALIDATORS (12) + PRIMITIVE_TYPES (10) registries in vocab.rs
  - Schema::is_known_flag / is_known_validator (pre-colon) / is_known_type query surface
  - validator W011/W012/W013 advisory diagnostics for unknown ~flag / ^validator / @in type
key_files:
  - crates/nodus/src/vocab.rs
  - crates/nodus/src/validator.rs
patterns_established:
  - "Advisory-first vocabulary checks: registry findings are warnings (never set has_errors), so host-specific vocabulary degrades gracefully until SchemaProvider extension lands"
duration_minutes: 0
---

# Phase 9 — Closed Vocabulary Registries (l2-nodus-registries)

Implement the closed vocabulary registries from `l1-nodus-language.md` §4.6
(contents per `l2-nodus-runtime.md` §4.7(f)) in `crates/nodus`, per
`l2-nodus-registries.md`. Adds `KNOWN_FLAGS`, `KNOWN_VALIDATORS`, and
`PRIMITIVE_TYPES` as `vocab` data, the `Schema` query surface, and advisory
(warning-severity) validator diagnostics for `~flag`/`^validator`/`@in`-type
tokens outside the registries. Strengthens NL-1/NL-7/NL-9; advisory-first so no
existing workflow hard-breaks.

**Specs covered**: `l2-nodus-registries.md` (Stable).

## Track A — Registries data & query surface (`vocab.rs`)

Pure additions mirroring the `KNOWN_COMMANDS` / `is_command` pattern.

- [x] **T-9A01** — Add the three registry constants
  - `KNOWN_FLAGS` (12: sentiment, intent, entities, topics, lang, toxicity, urgency, formality, clarity, relevance, pii, keywords)
  - `KNOWN_VALIDATORS` (12 names: len, min_len, no_pii, no_toxic, lang, format, required, sentiment, confidence, no_links, brand_voice, approved)
  - `PRIMITIVE_TYPES` (10: str, int, float, bool, list, obj, url, ts, null, any)
  - **Verify**: unit test `registries_have_expected_size` in `vocab.rs #[cfg(test)]` asserts lengths 12 / 12 / 10

- [x] **T-9A02** — Add `Schema` query methods
  - `is_known_flag(&self, name)`, `is_known_validator(&self, name)` (matches the segment before the first `:`), `is_known_type(&self, name)`
  - **Verify**: unit tests in `vocab.rs #[cfg(test)]` — `is_known_flag("sentiment")` true / `"bogus"` false; `is_known_validator("len:32")` true (pre-colon) / `"len"` true / `"nope"` false; `is_known_type("url")` true / `"widget"` false

## Track B — Validator diagnostics (`validator.rs`)

Advisory checks; warnings never set `has_errors` (NL-4 unaffected).

- [x] **T-9B01** — Emit advisory diagnostics for out-of-registry tokens
  - Warning for a `~flag` not in `KNOWN_FLAGS`, a `^validator` name not in `KNOWN_VALIDATORS`, and an `@in` field type not in `PRIMITIVE_TYPES`; assign the next free `W0xx` codes
  - **Verify**: unit tests in `validator.rs #[cfg(test)]` — a workflow with `~bogusflag` / `^nope` / an unknown `@in` type each yields a warning diagnostic and `ValidationReport::has_errors` stays `false`

## Track T — Validation & gates

- [x] **T-9T01** — Quality gates
  - `cargo test -p nodus` — full suite green (existing 222 + new); `cargo clippy -p nodus --all-targets -- -D warnings` — zero lints; `cargo fmt -p nodus` — clean; `cargo doc -p nodus --no-deps` — no new warnings beyond the pre-existing `test`-fn baseline
  - Confirm no existing fixture regresses to an *error* (registry checks are advisory)
  - **Verify**: all four commands exit 0; new-warning count is zero

## Status

**Status:** Done

## Notes

Execution order **A → B → T**: the registries and `Schema` queries (Track A) are
the data Track B consults. Risk is low — additions are data plus advisory
warnings, with no behavior change to execution. The advisory-first choice means
fixtures using host-specific flags/validators warn but do not fail; promotion to
hard errors (and host extension via `SchemaProvider`) is a future amendment.
