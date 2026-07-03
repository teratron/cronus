# Main Workspace Changelog

Internal phase journal. Each entry corresponds to a completed phase.

## Phase 7 — Leaf: TUI (2026-06-28)

- T-7A01: `terminal` module — `TerminalBackend` DI trait + `CrosstermBackend` production impl (Windows Press-only key filter) + `Tui<B>` RAII guard with panic-safe, idempotent restore; converted `cronus-tui` to lib+bin; added `crossterm` workspace dependency
- T-7A02: `app` module — `App::tick` pure step-function (input-first → non-blocking on slow snapshot → exactly one redraw), view-only `ViewModel`/`CoreSnapshot`, `SnapshotSource` poll seam + `CapabilitySource`, `Renderer` seam, `run`/`run_with` drivers. Event-seam resolved: core has no pub/sub → poll-snapshot fallback (INV-5 preserved)
- T-7B01: `view` module — `layout()` (2×2 panels above a one-row command bar, clamps at tiny sizes), `Focus` enum + `next`/`prev` tab-cycling, `RatatuiRenderer` (bordered/titled panels + focus highlight); adopted ratatui 0.30 (feature `crossterm_0_29`, single crossterm version); `Tab`/`Shift+Tab` focus routing; `Key::BackTab` added
- T-7B02: Board + Office panels — 7-column `BoardColumn` presentation projection (core models 6 states + archive store; transitions stay in core, INV-2), `BoardView`/`BoardCard`, `OfficeView`/`AgentActivity`, pure `render_board`/`render_office` into an off-screen ratatui `Buffer`
- T-7B03: Status + Sessions panels — `render_status` mirrors the `status` capability (version + status line); `SessionsView` bounded last-N scrollback projection (`MAX_SESSION_LINES` = 500) in `CoreSnapshot`, keeping the view a pure function of the snapshot (INV-5); all four panels render
- T-7C01: `command` module — slash parser, `CATALOG` (help + 21 CLI-mirrored verbs), `classify` → `Help`/`Run`/`Error` (unknown verb → inline error, never panic); interactive command bar (`command_input`/`command_feedback`, focus-aware key routing; Esc cancels the line in the bar, quits elsewhere)
- T-7C02: `Dispatcher` trait + `CapabilityDispatcher` (verb → core capability, then masks output via `cronus::redact::redact` — INV-7, no re-implementation) + `NoopDispatcher`; loop drains a submitted command and renders masked output, closing the input→core→redraw cycle
- T-7T01: `parity_matrix` validation — every slash command maps to a CLI verb (no TUI-only behavior), full CLI verb coverage, and a compile-time manifest check that the crate links `cronus` and never `cronus-cli` (structural INV-2); `cargo tree` confirms `cronus-tui → cronus`
- T-7T02: `render_state` + `mask_secrets` validation — extracted `render_view` (pure frame render); same view-model ⇒ byte-identical buffer (INV-5), a changed snapshot ⇒ a changed buffer, and a dispatched secret never reaches the rendered buffer (INV-7)
- Verify: `cargo test -p cronus-tui` — 38 passed, 0 failed; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean


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
