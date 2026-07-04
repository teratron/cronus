# Doctor

**Version:** 1.1.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-doctor.md

## Overview

The concrete self-healing service: the health checks it runs, which problems it auto-repairs versus escalates, how it recovers after a crash, and the `doctor` command.

## Related Specifications

- [l1-doctor.md](l1-doctor.md) - The model this implements.
- [l2-scheduler.md](l2-scheduler.md) - Periodic checks run as a routine.
- [l2-github-issue.md](l2-github-issue.md) - Unrepairable issues may be reported with consent.
- [l2-cli.md](l2-cli.md) - Command grammar standard.

## 1. Motivation

The model needs concrete checks and a clear safe/risky split so the office self-heals without risk.

## 2. Constraints & Assumptions

- Checks are read-only; repairs are explicit and logged.
- A scheduled routine runs checks unattended; `cronus doctor` runs them on demand.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| HEAL-1 Continuous checks | A scheduled routine runs the check suite; `doctor` runs it on demand. |
| HEAL-2 Safe self-repair | Deterministic fixes (re-index, unstick obviously-finished cards, prune dangling sessions) run with `--fix`. |
| HEAL-3 Escalate risky | Ambiguous/destructive fixes are reported with a recommended action, not applied. |
| HEAL-4 Non-destructive | Checks read-only; repairs snapshot or are reversible. |
| HEAL-5 Traceable | Every check/repair writes to logs. |
| HEAL-6 Self-recovery | On startup, a recovery pass reconciles interrupted runs/sessions to a consistent state. |

## 4. Detailed Design

### 4.1 Check suite

| Check | Repair (safe) | Escalate (risky) |
| --- | --- | --- |
| store/index consistency | rebuild index from source text | corrupt source data |
| stuck `running` cards | re-queue clearly-abandoned cards | ambiguous in-progress work |
| dangling sessions | prune per session-routing | active-looking sessions |
| config validity | restore missing defaults | conflicting user config |
| disk/resource pressure | clear cache | low disk needing user action |
| crash recovery | resume from durable state | divergent partial writes |

### 4.2 Extensibility

Third-party packages (channel connectors, plugins, workspace extensions) can contribute their own health checks without modifying the core. Two registration paths are supported:

**Programmatic registration** (within the same process):

```text
[REFERENCE]
register_doctor_contribution(id: String, fn: DoctorCheckFn) -> void
// id should be namespaced, e.g. "myplugin.cron"
// fn: (ctx: DoctorRunContext) -> String[]
// Returns informational lines (empty list = nothing to report)
```

**Package entry-point** (third-party packages):

```text
[REFERENCE]
// In package metadata (equivalent of pyproject.toml or Cargo's inventory pattern):
[project.entry-points."cronus.doctor"]
my_plugin = "my_crate::doctor:doctor_notes"
```

`DoctorRunContext` is passed to each extension callable:

```text
[REFERENCE]
DoctorRunContext {
  cfg: Config,           // current workspace configuration
  cli_base_url: String,  // base URL for local service health checks
  timeout: f32,          // per-check timeout in seconds
  deep: bool             // true when --deep flag was passed
}
```

Extension execution order: manual registrations first (alphabetical by id), then entry-point discoveries (alphabetical by name). Each extension runs in isolation — a panicking or erroring extension logs a warning and is skipped; it does not abort the rest of the check suite.

<!-- [ADDED] v1.1.0 -->
**Concurrent probe execution.** Checks and runbook probes are read-only and independent, so the suite executes them concurrently under a bounded cap (default 4) with the existing per-check `timeout` applied individually; suite wall-clock approaches the slowest probe instead of the sum. The report is assembled after all probes settle and is always rendered in the canonical (alphabetical/runbook) order above — execution order is a scheduling detail, output order is deterministic. Probes that touch the same exclusive resource (e.g. a repair dry-run over one database) declare an exclusivity key and serialize against each other only. `--fix` repairs never run concurrently with probes: the suite settles first, then repairs apply one at a time (HEAL-2 logging preserved).

### 4.3 Extended check runbook

Beyond the check suite in §4.1, the doctor runs a structured runbook that surfaces environmental and installation-layer issues the abstract checks can't reach. Each runbook step is a named probe; output is a pass/warn/fail line with a remediation hint on non-pass.

```text
[REFERENCE]
Runbook probes (in execution order):

[prereqs]
  For each required tool (git, node-runtime, python):
    pass  — tool found in PATH, version meets minimum
    warn  — optional tool (e.g. web search CLI) absent; graceful degradation applies
    fail  — required tool missing; remediation = install command

[config]
  config.json (or equivalent workspace config) exists
    pass  — file found, required fields present (vault_path, port, host, install_id)
    warn  — optional fields absent (hermes_api_url, last_check_for_updates)
    fail  — file missing or unparseable; remediation = re-run install or create minimal config

[token]
  Ephemeral session token present (see l2-security.md §4.7 ephemeral mode)
    pass  — token file present, length ≥ 32 chars
    warn  — token missing; will be regenerated on next server start
    fail  — token directory not writable

[vault-zones]
  Knowledge vault contains all four expected zones:
    sources/   — raw materials; agent read-only
    wiki/      — synthesized knowledge; agent read+write
    journal/   — user-owned entries; agent read-only
    schema/    — rules files; agent read-only
  pass  — all four zone directories exist
  warn  — vault.db (FTS5+vector index) absent; keyword search falls back to linear scan
  fail  — vault path not configured or does not exist; remediation = re-run install wizard

[build-artifacts]
  UI build artifacts present (for the local console frontend):
    pass  — build output directory present
    warn  — no production build; dev server still works
    fail  — node_modules missing; run dependency install first

[server]
  Console process is listening on configured port:
    pass  — port is bound
    warn  — no process on port; server not started yet
    (not fail — server may be intentionally stopped)

[skills]
  Shipped skill packs installed and counted:
    pass  — skills directory exists, N skill files found (report N)
    warn  — skills directory absent or empty; remediation = re-run install

[bridge]
  Bridge extension installed (the mutation-only channel for agent→runtime writes):
    pass  — bridge skills directory present with ≥1 skill file
    fail  — bridge absent; agent mutations will fail silently

[search-tools]
  Free-tier web search CLI tools detected (optional; no paid APIs required):
    pass  — at least one of: ddgr, gogcli found in PATH
    warn  — neither found; web search skills will fall back to the agent's built-in browser tool
    (not fail — browser fallback is a supported degraded path)
```

Each runbook probe is registered as an extension check (same mechanism as §4.2); operators can add custom probes for workspace-specific dependencies.

### 4.4 Command surface

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| run checks | `cronus doctor` | `/doctor` | `doctor.check() -> Report` |
| run + safe repair | `cronus doctor --fix` | `/doctor --fix` | `doctor.repair() -> Report` |

## 5. Drawbacks & Alternatives

- **False positives unstick real work:** mitigated by conservative "clearly abandoned" criteria. <!-- TBD: abandonment criteria for stuck cards -->
- **Alternative — fix everything automatically:** rejected (HEAL-3).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[DOCTOR]` | `.design/main/specifications/l1-doctor.md` | Invariants this implements |
| `[CLI]` | `.design/main/specifications/l2-cli.md` | Command grammar standard |

## Document History

| Version | Date | Notes |
| --- | --- | --- |
| 1.1.0 | 2026-07-04 | Concurrent probe execution (§4.2): read-only checks/probes run under a bounded cap with per-check timeouts; deterministic report ordering after settle; exclusivity keys for same-resource probes; `--fix` repairs stay serialized after the suite settles. History table added with this entry. |
