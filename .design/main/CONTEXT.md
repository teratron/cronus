# Project Context

**Generated:** 2026-07-03

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
│   └── technology-stack-research.md
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
├── installer/
├── package.json
├── packages/
│   └── ui/
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
└── rust-toolchain.toml
```

## Recent Changes


## Phase 8 — Flower: Desktop App — 2026-07-02

- T-8A01: toolchain + scaffold (pnpm workspace, `packages/ui` React 19/Vite/TS, `apps/desktop` + Tauri v2 crate at `apps/desktop/tauri`)
- T-8A02: shell <-> core IPC bridge — `Bridge<Capabilities>` over the embedded engine, output masked via `cronus::redact` (INV-7); `capability_version`/`capability_status` commands; shell-agnostic typed TS client wired in `main.tsx`
- T-8B01: settings persistence — merge-safe JSON store, dual log-level deserializer (string + legacy int), additive migration preserving unknown fields, per-OS defaults, `AtomicU8` hot log level, atomic temp+rename saves, fail-soft startup load
- T-8B02: shell systems — tray 9-variant State x Theme icon matrix (no I/O on transitions) + state-dependent menu + copy-last-result fallback; shortcut bindings with conflict auto-rollback, backend switching with re-validation, suspend/resume + dynamic cancel; per-OS overlay geometry with saturating math + GTK escape hatch; single-instance acquire-or-forward + unified trigger dispatch
- T-8C01: five-surface workbench (office/board/chat/editor/dashboard) rendering from injected state; system/light/dark themes over design tokens; en/ru i18n with typed keys and English fallback
- T-8C02: Office View — one `OfficeProjection`, two renders (graph: nodes/reporting/assignment edges/activity; floor: rooms/seats); inspect-as-intent
- T-8C03: Dashboard — per-office stats + core-computed building aggregate, read-only, re-renders per projection
- T-8D01: provider-keyed system-prompt dispatch (single decision point, model families contained per provider); byte-stable `<env>` XML context + `<available_references>` with escaping
- T-8D02: MCP client model — Stdio/SSE/StreamableHTTP with `DEFAULT_TIMEOUT_MS=30_000` (stdio exempt), five-state connection status, roots-only capabilities, OAuth pending-transport map (consume-always, resume-on-success)
- T-8T01: structural validation — `fallow audit` clean; 0 `any`; dependency direction UI -> core only
- T-8T02: store-compliance — same state => identical render; masked secrets rendered verbatim (INV-7); token-only theming; locale swap leaves no stale text
- Verify: Tauri crate `cargo test` 34/34 + clippy `-D warnings` + fmt; `packages/ui` vitest 27/27 + biome + tsc; `pnpm -r build` green; root workspace tests 0 failures

