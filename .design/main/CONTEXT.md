# Project Context

**Generated:** 2026-07-10

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
│   ├── main/
│   ├── nodus/
│   ├── wiki/
│   └── workspace.json
├── .drafts/
│   ├── TODO.md
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
│   ├── cli/
│   ├── codegraph/
│   ├── core/
│   ├── nodus/
│   └── tui/
├── docs/
│   └── README.md
├── firebase-debug.log
├── installer/
├── package.json
├── packages/
│   └── ui/
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
└── rust-toolchain.toml
```

## Recent Changes



## Phase 9 — Operational Hardening — 2026-07-03

- T-9A01: sandbox network policy — deny-by-default named entries with per-binary allowlisting, three access tiers (restricted/balanced/open), typed access-failure classification (blocked-by-policy/missing-approval/unsupported/unknown)
- T-9A02: multi-user auth — bcrypt password hashing, 7-day `Instant`-based sessions with an orphan guard, RFC 6238 TOTP over HMAC-SHA1 with 8 single-use backup codes, admin promote/demote with privilege stashing + last-admin guard, reserved sentinel usernames rejected at creation
- T-9B01: doctor — six-category check catalog (store index, stuck cards, dangling sessions, config validity, disk pressure, crash recovery) with a conservative safe-repair-vs-escalate split, panic-isolated extension registration, `cronus doctor [--fix]`
- T-9B02: config hot-reload — flattened key-path diff, priority-ordered reload-rule table (first-match-wins, unmatched = safe restart default), skills-snapshot invalidation, bounded backoff -> polling -> disabled watcher recovery state machine
- T-9C01: backup & restore — plain-`std::fs` restore-by-copy excluding `.env`/cache always and logs by default (opt-in), symlink-safe, `cronus backup create/list` + `cronus restore`
- T-9C02: agent migration — schema-versioned manifest validation, two-layer split (archive/memory-candidates/skills/credentials-always-skipped), staged dry-run-first apply that backs up (via T-9C01) before any write, identity-based dedup
- T-9D01: GitHub issue reporting — fail-closed consent gate, BLAKE3 cross-machine/cross-episode error fingerprinting (hex-address + home-dir normalization), in-memory dedup table with the documented Lookup API, previewable sanitized payload
- T-9D02: self-improvement — calibration overconfidence/warning gate, mistake log with cross-project tagging, should-have-asked distinct-trigger recency lookup, at-most-one-pending-per-project ask-backs, upserted reasoning templates, a five-signal `build_brief` join
- T-9D03: telemetry — opt-in gate (default off, no-op recording while opted out), closed metric-name allowlist, a payload enum with no free-text variant, opt-out drops the queue
- T-9T01: cross-subsystem hardening integration — a seeded secret proven absent from a real backup archive, a real restored tier, and a scrubbed report preview; both consent gates (report, telemetry) proven to block by default; workspace-wide `cargo test`/`clippy -D warnings`/`fmt --check` all clean
- Verify: `cargo test --workspace` green across all 5 crates; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean

