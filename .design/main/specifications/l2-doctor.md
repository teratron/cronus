# Doctor

**Version:** 1.0.0
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

### 4.3 Command surface

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
