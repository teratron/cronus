# Technology Stack

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-architecture.md

## Overview

The validated technology stack for Cronus across all architectural layers, after platform research (2026-06). This spec is the source of truth for technology choices and the rationale behind each. It deliberately diverges from the original draft in four places (mobile topology, vector store, on-device LLM engine, monorepo tooling); those changes are recorded explicitly.

## Related Specifications

- [l1-architecture.md](l1-architecture.md) - The architecture this stack realizes.
- [l2-core-library.md](l2-core-library.md) - Core foundation built on this stack.
- [l2-cli.md](l2-cli.md) - CLI built on this stack.
- [l2-tui.md](l2-tui.md) - TUI built on this stack.
- [l2-app-ui.md](l2-app-ui.md) - Application UI/UX built on this stack.

## 1. Motivation

A cross-platform autonomous product must pick technologies that (a) let one core run on desktop, server, and mobile; (b) keep an always-on host viable; (c) support local-first storage with vector memory; and (d) integrate AI both locally and via cloud. Each choice below is tied to those needs and to the L1 invariants.

## 2. Constraints & Assumptions

- Versions target the latest stable lines as of 2026-06; pin exact versions in lockfiles.
- "Mobile as always-on server" from the original draft is **not** adopted — it is prohibited by iOS/Android OS lifecycle and app-store policy (see §5 and INV-4 compliance).
- Local LLM on mobile is foreground-only and runs through a C/C++ engine via FFI, not pure-Rust ML crates.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| INV-1 Embeddable core | Core is a **Rust** library crate with a C-ABI/FFI surface; linkable into Tauri, CLI/TUI binaries, and external host programs. |
| INV-2 Logic in core only | Frontends (CLI/TUI/React app) call the Rust core; no domain logic in TypeScript or terminal layers. |
| INV-3 Command parity | All frontends bind to the same core contract; commands map 1:1 to core calls. |
| INV-4 Hub-and-spoke autonomy | Always-on engine runs as a desktop OS service (systemd/launchd/Windows service) or headless server/remote node; **Tauri v2 mobile is a thin client**, woken by APNs/FCM push. |
| INV-5 Durable, restartable state | **SQLite** (file-based) with **sqlite-vec** for vectors; state is a copyable file; optional **libSQL/PostgreSQL** for remote sync. |
| INV-6 Graceful capability scaling | Mobile frontend exposes a subset (foreground + sync, optional 1–3B local model); never divergent behavior. |
| INV-7 Security of client data | Secrets in `.env`/OS keychain, excluded via `.gitignore`; only anonymized operational telemetry leaves the device, never user data. |

## 4. Detailed Design

### 4.1 Deployment topology

| Tier | Role | Technologies |
| --- | --- | --- |
| Hub (always-on) | Autonomy core: orchestrator, heartbeat, Kanban, cron, memory, model router | Rust headless core + OS service (systemd/launchd/Windows service) or remote/self-hosted/SSH |
| Spoke — Desktop | Full GUI over the core | Tauri v2 (Windows/macOS/Linux) |
| Spoke — Mobile | **Thin client**, not a server: foreground + push-driven sync | Tauri v2 (iOS/Android) + APNs/FCM |

### 4.2 Stack by layer

| Layer | Technology | Version (2026-06) | Verdict | Note / change vs draft |
| --- | --- | --- | --- | --- |
| Core engine | Rust | — | solid | One crate → desktop + mobile is real |
| Shell / packaging | Tauri v2 | 2.10.x | solid (desktop) / caveat (mobile) | Mobile = thin client only; no background autonomy |
| Frontend | React 19 + Vite + TypeScript | React 19.2, Vite 8 | solid | SPA (not Next.js) — correct for a local-served UI |
| Styling | Tailwind CSS v4 | 4.3.x | caveat | Needs Chrome 111 / Safari 16.4 → set min-WebView floor or PostCSS downgrade |
| Components | shadcn/ui **or** DaisyUI | — | solid | Pick ONE, not both |
| Rich-text | Lexical | 0.45.x | caveat (pre-1.0) | Pin version + adapter wrapper |
| Local DB | SQLite + sqlite-vec | sqlite-vec 0.1.x | solid (alpha caveat) | **Replaces libSQL-vector**; pure C, prebuilt iOS/Android |
| Remote / sync DB | libSQL / PostgreSQL | — | optional | Sync only, not the vector engine |
| Local LLM | llama.cpp via Rust FFI | — | caveat | **Replaces candle/mistral.rs** on mobile; 1–3B Q4; iOS Metal, Android CPU |
| Cloud LLM | OpenAI/Anthropic/OpenRouter + in-house router | — | solid | Difficulty-threshold + fallback cascade + semantic cache |
| Monorepo | moon **or** Nx + @monodon/rust | moon 2.x | change | **Replaces Turborepo** (JS-only, RFC #683). Fallback: Turbo(JS) + Cargo/sccache(Rust) |
| JS package manager | pnpm | — | solid | — |
| Frontend build | Vite | 8.0.x | solid | Node 20.19+/22.12+ |

### 4.3 Changes vs original draft

| Was (draft) | Now | Why |
| --- | --- | --- |
| Mobile = always-on server | Mobile = thin client (hub-and-spoke) | OS + store prohibition (iOS BGTasks / Guideline 2.5.4; Android 15 FGS 6h cap / Play policy) |
| LibSQL + vector | sqlite-vec (vectors) + libSQL/PG (sync) | Turso rewriting on Rust with no vector engine yet |
| candle/mistral.rs on mobile GPU | llama.cpp via FFI; iOS Metal / Android CPU | candle won't build on Android / no iOS GPU; mistral.rs has no mobile target |
| Turborepo | moon / Nx | Turborepo does not manage Rust (RFC #683) |

## 5. Drawbacks & Alternatives

- **sqlite-vec is pre-1.0:** breaking changes possible; mitigate by pinning and isolating the vector access behind a core repository interface.
- **Tailwind v4 on system WebViews:** old Linux WebKitGTK / old Android WebView can fail to style; mitigate with a min-OS/WebView floor or a PostCSS downgrade, else fall back to Tailwind v3.4.
- **moon/Nx learning curve:** heavier than Turborepo; acceptable given Turborepo leaves the Rust half unmanaged. <!-- TBD: final monorepo tool decision (moon vs Nx) pending a thin spike -->

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ARCH]` | `.design/main/specifications/l1-architecture.md` | Architecture invariants this stack must satisfy |
| `[ENVEXAMPLE]` | `.env.example` | Declares required environment variables / secrets layout |
| `[GITIGNORE]` | `.gitignore` | Secret-exclusion contract (INV-7) |
