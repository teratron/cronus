# Technology Stack

**Version:** 1.2.0
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

### 4.4 Local LLM Provider Catalog

Six local inference providers are supported. Detection runs as parallel thread-scoped probes (timeout 800 ms); model name stems are normalised to lowercase for deduplication.

#### Provider table

```text
[REFERENCE]
Provider           | Protocol        | Default endpoint        | Notes
──────────────────────────────────────────────────────────────────────────────────────────
Ollama             | REST /api/*     | http://localhost:11434  | OLLAMA_HOST env override; IPv6→IPv4 fallback; wildcard bind (0.0.0.0/[::]) rejected
llama.cpp          | REST /v1/*      | http://localhost:8080   | GGUF format; Metal/CUDA/ROCm/Vulkan/SYCL/CPU backends
MLX                | REST /v1/*      | http://localhost:8080   | Apple Silicon only; mlx-4bit / mlx-8bit formats
vLLM               | REST /v1/*      | http://localhost:8000   | CUDA only; AWQ/GPTQ/SafeTensors; TurboQuant KV (research)
LM Studio          | REST /v1/*      | http://localhost:1234   | Desktop GUI; OpenAI-compat endpoint
Docker Model Runner| REST /engines/* | http://localhost:12434  | Container-based; GPU pass-through
```

`OllamaProvider` preferentially tries `localhost`; falls back to `127.0.0.1` for hosts where `localhost` resolves to `::1` while Ollama listens on IPv4 only.

#### GpuBackend taxonomy

```text
[REFERENCE]
Backend | Platform             | Notes
──────────────────────────────────────────────────────────────────────────
Cuda    | NVIDIA discrete      | Primary; TurboQuant, TensorParallel
Metal   | Apple unified memory | MLX-native; no separate CpuOffload path
Rocm    | AMD discrete         |
Vulkan  | Cross-platform       | Fallback when no first-class driver
Sycl    | Intel Arc            |
CpuArm  | ARM CPU              | Mobile / Apple CPU fallback
CpuX86  | x86 CPU              |
Ascend  | Huawei NPU           |
```

GPU detection cascade: nvidia-smi (extended addressing mode → standard fallback) → rocm-smi → sysfs → Windows WMI (PowerShell → wmic fallback) → AMD APU unified → NVIDIA ATS → Intel Arc → Apple Silicon → Ascend NPU → Vulkan fallback. Multi-vendor results are de-duplicated; discrete GPUs preferred over APU/integrated.

#### Special platform notes

```text
[REFERENCE]
Apple Silicon         — total_ram_gb is used as the VRAM pool (unified memory).
                        Available RAM via vm_stat: (free + inactive + purgeable) × page_size (16 KB on Apple Silicon).
                        CpuOffload is infeasible (skip, mark not available).

NVIDIA Grace/DGX Spark — addressing_mode=ATS → unified_memory=true; VRAM = /proc/meminfo total.

AMD Ryzen AI MAX      — unified_memory=true; VRAM = total DIMM sum via Win32_PhysicalMemory
                         (not AdapterRAM, which is capped ~4 GB by 32-bit BIOS field).

WMI AdapterRAM        — 32-bit field; falls back to GPU name heuristic for VRAM estimation
                         when reported value is ≤ 4 GB for known high-VRAM cards.
```

### 4.5 Quantization System

Quantization is orthogonal to provider: any GGUF-capable provider can serve any GGUF quant; MLX providers serve only MLX formats; vLLM serves AWQ/GPTQ/SafeTensors.

#### Weight quantization formats

```text
[REFERENCE]
GGUF standard hierarchy (best quality → most compressed):
  Q8_0 → Q6_K → Q5_K_M → Q4_K_M → Q3_K_M → Q2_K

MLX hierarchy:
  mlx-8bit → mlx-4bit

Pre-quantized formats (fixed bit-width, no dynamic re-quant):
  AWQ-4bit, AWQ-8bit, GPTQ-Int4, GPTQ-Int8, AutoRound-4bit, AutoRound-8bit

Unquantized:
  F32, F16, BF16

Unalike-distrib variants (UD-Q*_K_XL/L/M/S):
  Same bpp as base format; different importance weighting per layer.
```

#### Quantization parameters table

```text
[REFERENCE]
Format      | bytes/param | speed multiplier | quality penalty
─────────────────────────────────────────────────────────────
F16 / BF16  | 2.00        | 0.60             |  0
Q8_0        | 1.05        | 0.80             |  0
Q6_K        | 0.80        | 0.95             | -1
Q5_K_M      | 0.68        | 1.00             | -2
Q4_K_M      | 0.58        | 1.15             | -5
Q4_0        | 0.58        | 1.15             | -5
Q3_K_M      | 0.48        | 1.25             | -8
Q2_K        | 0.37        | 1.35             |-12
mlx-4bit    | 0.55        | 1.15             | -4
mlx-8bit    | 1.00        | 0.85             |  0
AWQ-4bit    | 0.50        | 1.20             | -3
AWQ-8bit    | 1.00        | 0.85             |  0
GPTQ-Int4   | 0.50        | 1.20             | -3
GPTQ-Int8   | 1.00        | 0.85             |  0

speed multiplier: applied to base tok/s constant; < 1.0 = slower than Q5_K_M baseline
quality penalty:  additive offset on 0–100 quality score
```

#### Best-quant-for-budget algorithm

```text
[REFERENCE]
best_quant_for_budget(params_B, budget_GB, context, hierarchy):
  for quant in hierarchy:                            // best quality first
    mem = params_B × bpp(quant) + kv_cache(context) + 0.5 GB overhead
    if mem ≤ budget_GB: return (quant, mem)          // first that fits
  // Nothing fits at full context: try half context
  for quant in hierarchy:
    mem = params_B × bpp(quant) + kv_cache(context/2) + 0.5 GB overhead
    if mem ≤ budget_GB: return (quant, mem)
  return TooTight                                    // no option fits
```

#### KV cache quantization

Orthogonal to weight quantization; trades context-window memory for quality.

```text
[REFERENCE]
KV format  | bytes/element | Runtime support
───────────────────────────────────────────────────────────────
fp16       | 2.0           | All runtimes (default)
fp8        | 1.0           | vLLM; llama.cpp fp8-enabled builds
q8_0       | 1.0           | llama.cpp --cache-type-k q8_0
q4_0       | 0.5           | llama.cpp --cache-type-k q4_0 (quality drop)
TurboQuant | ~0.34         | vLLM + CUDA only; compresses full-attention layers only
```

`TurboQuant` savings for hybrid architectures (models mixing self-attention with linear/state-space layers) are proportional to the full-attention fraction; the linear/Mamba layers remain at fp16.

### 4.6 Audio Processing Stack

Voice input features require a dedicated audio pipeline separate from the inference stack. The following components apply when Cronus implements voice/speech input.

#### Sample rate requirement

Speech recognition models require **16 000 Hz mono audio**. The pipeline must resample from any device-native rate to 16 kHz before passing audio to the model; never assume the microphone outputs at 16 kHz.

#### VAD pipeline

Voice Activity Detection gates the stream to avoid sending silence or noise to the model:

```text
[REFERENCE]
Pipeline:
  microphone → cpal stream
             → resampler (rubato → 16 000 Hz, mono)
             → VAD (ONNX model via ORT)
             → SmoothedVad (sliding-window temporal smoothing)
             → audio buffer
             → STT model

SmoothedVad averages VAD scores over a short window to prevent rapid on/off
switching at speech boundaries (hysteresis effect).
```

#### Audio I/O libraries

```text
[REFERENCE]
cpal   — cross-platform audio I/O (device enumeration, capture + playback streams)
rubato — high-quality async resampler; arbitrary ratio; converts device rate → 16 000 Hz
rodio  — audio playback for UI feedback sounds (confirmation tones, error sounds)
```

#### Platform-specific output mute during recording

Mute the system output while recording to prevent microphone feedback bleed:

```text
[REFERENCE]
Windows → Win32 COM: IMMDeviceEnumerator → IAudioEndpointVolume.SetMute()
          CoInitializeEx per thread; silent on failure (not all drivers expose the interface)

Linux   → try backends in order, stop on first success:
          1. wpctl set-mute @DEFAULT_AUDIO_SINK@ 1   (PipeWire)
          2. pactl set-sink-mute @DEFAULT_SINK@ 1    (PulseAudio)
          3. amixer set Master mute                  (ALSA)
          Silent on failure; does not block recording

macOS   → osascript: "set volume output muted true"
          Silent on failure
```

#### Stream lifecycle

```text
[REFERENCE]
On-demand mode:   open stream when recording starts; close immediately when done.
                  Releases the microphone between sessions (privacy default).

Lazy-close mode:  keep stream alive for a configurable idle timeout after recording stops.
                  Reduces stream re-open latency for high-frequency short-burst usage.

Default idle timeout: STREAM_IDLE_TIMEOUT = 30 s
```

### 4.7 SQLite Concurrency Policy

<!-- [ADDED] v1.1.0 -->

Nearly every subsystem persists through SQLite (state, memory scopes, knowledge base, code index, file store, inbox, board). One uniform access discipline applies to every database file the engine opens; per-subsystem deviation is a defect, not a choice:

- **WAL mode everywhere** — every database opens in WAL journal mode: readers never block the writer, the writer never blocks readers.
- **Single-writer discipline** — each database file has exactly one owning writer task; all mutations are submitted to that task's serialized queue (write-behind). Subsystems never open ad-hoc write connections to a file they do not own. This is the same per-resource serialization the engine already applies to file mutations, extended to databases.
- **Read pool** — reads go through a small pool of read-only connections per database; read concurrency is bounded by pool size, not by locking.
- **Timeouts, not spin-retries** — every connection sets `busy_timeout`; an `SQLITE_BUSY` that reaches a subsystem indicates a discipline violation to fix, never an error to retry in a loop.
- **Short transactions on the hot path** — long-running work (consolidation sweeps, reindexing, vacuum, backup snapshots) runs on the background tier in bounded batches; a hot-path transaction never spans a model call or network I/O.

### 4.8 Configuration Serialization Format

<!-- [ADDED] v1.2.0 -->

Config and small structured state serialize as **JSON by default**. JSON is the required format at every boundary; **RON** (Rusty Object Notation) is a narrow, opt-in choice for one specific class of file; **nodus never uses either as a core dependency**. A file's format is decided by where it lives and who edits it — not by preference:

| Format | Use for | Why |
| --- | --- | --- |
| **JSON** (default) | (a) files mirroring an external convention — `hooks.json`, `.mcp.json`, `plugin.json`/`extension.json`, `package.json`, `devcontainer.json`; (b) anything crossing the Rust↔TS/IPC boundary the desktop UI reads — `app.json`, `settings.json`, dashboard `layout.json`, `board.json`; (c) machine-generated data/index/export — codegraph graphs, analytics snapshots, migration `manifest.json`/`conversations.json` | The external contract or the non-Rust reader dictates it; RON has no mature TS parser and adds nothing for machine-written data no human edits. |
| **RON** (opt-in, justified dependency) | Rust-owned, human-hand-edited, **enum/variant-heavy** internal config that never crosses a language/interop boundary — e.g. routing rules (`routing.*`), model/gateway/channel bindings (`models.*`/`gateway.*`/`channels.*`), sandbox-policy presets (isolation tiers), automation node templates, quality-gate definitions | RON represents Rust tagged enums/tuples/structs faithfully and allows comments + trailing commas, so a hand-edited variant-config is legible and hard to mis-shape where JSON is error-prone. This — not general preference — is the "strictly necessary and justified" bar (project dependency policy) for adding the `ron` crate. |
| **TOML** | Build/tooling manifests only — `Cargo.toml`, `pyproject.toml` | Ecosystem standard; not a runtime-config choice, unchanged here. |

Discipline:

- **JSON is the fallback, not a defect.** A file that does not clearly meet the RON criterion stays JSON. Two formats are a real cost (tooling, cognitive load); RON must earn its place *per file*, never by default.
- **No big-bang migration.** Existing shipped JSON config (e.g. the Phase-4 model/router state) moves to RON only when a file is next substantially refactored *and* clears the criterion — never as a sweep. Changing a file's extension is a code migration, not a spec edit.
- **Serde-uniform.** Both formats deserialize through the same `serde` derives; the format is a call-site choice (`serde_json` vs `ron`), so a config type is format-agnostic and a file can move between formats with no model change.
- **nodus is exempt and stays format-neutral.** `crates/nodus` holds a zero-external-dependency contract (`l1-nodus-portability` LP-1) and MUST NOT take the `ron` crate. nodus core stores no config of its own — durable state is host-supplied through the `StorageProvider` seam (LP-15), and the host picks the format. RON is a `crates/core` (host) decision, never a nodus-core one.

## 5. Drawbacks & Alternatives

- **Two config formats (RON + JSON):** a split is cognitive and tooling overhead. Mitigated by the strict per-file criterion (§4.8) — RON is confined to hand-edited enum-heavy Rust config, JSON stays the default and the only boundary format; the split never reaches the TS side. Rejected alternatives: JSON-everywhere (loses tagged-enum legibility + comments for variant config), JSON5/JSONC (comments but still no faithful Rust enum shape and another parser), TOML-for-all (weak for deeply-nested tagged unions).
- **sqlite-vec is pre-1.0:** breaking changes possible; mitigate by pinning and isolating the vector access behind a core repository interface.
- **Tailwind v4 on system WebViews:** old Linux WebKitGTK / old Android WebView can fail to style; mitigate with a min-OS/WebView floor or a PostCSS downgrade, else fall back to Tailwind v3.4.
- **moon/Nx learning curve:** heavier than Turborepo; acceptable given Turborepo leaves the Rust half unmanaged. <!-- TBD: final monorepo tool decision (moon vs Nx) pending a thin spike -->

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ARCH]` | `.design/main/specifications/l1-architecture.md` | Architecture invariants this stack must satisfy |
| `[ENVEXAMPLE]` | `.env.example` | Declares required environment variables / secrets layout |
| `[GITIGNORE]` | `.gitignore` | Secret-exclusion contract (INV-7) |

## Document History

| Version | Date | Notes |
| --- | --- | --- |
| 1.1.0 | 2026-07-04 | Added §4.7 SQLite Concurrency Policy — uniform cross-subsystem discipline: WAL everywhere, single owning writer task per database file (write-behind queue), bounded read pool, busy_timeout instead of spin-retries, short hot-path transactions with heavy work on the background tier. History table added with this entry. |
| 1.2.0 | 2026-07-10 | Added §4.8 Configuration Serialization Format — JSON is the default and the only boundary format (external-convention files, Rust↔TS/IPC files the UI reads, machine-generated data); RON is a narrow opt-in for Rust-owned, hand-edited, enum/variant-heavy internal config where tagged-enum fidelity + comments beat error-prone JSON (the justified-dependency bar for the `ron` crate); TOML stays for build manifests only; serde-uniform so format is a call-site choice and no big-bang migration; nodus exempt and format-neutral (LP-1 zero-dep forbids the `ron` crate, its durable state is host-supplied via the StorageProvider seam). §5 gains the two-format-overhead drawback + rejected alternatives (JSON-everywhere, JSON5/JSONC, TOML-for-all). Additive policy section — stays Stable (Trust Mode, no contradiction with §1–4). |
