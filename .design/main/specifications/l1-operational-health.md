# Operational Health & Insights

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

A continuous observability layer that turns locally-recorded runtime traces into an explainable picture of how well the system is operating: a quantified health score per monitored unit, severity-graded alerts when signals cross thresholds, trend and anomaly detection over rolling windows, and cost/usage accounting. It answers "is the system healthy, and where is it drifting?" without leaving the device.

This is distinct from three neighbours. Telemetry governs the optional, opt-in sharing of anonymized metrics *outward* to improve the product. The self-healing subsystem *repairs* integrity and consistency faults. Practice analytics *coaches the user* on prompting and session hygiene. Operational health only *measures and warns* about the runtime itself — it never shares, never repairs, never coaches.

## Related Specifications

- [l1-telemetry.md](l1-telemetry.md) - Telemetry shares anonymized metrics outward; operational health observes the local runtime and never egresses.
- [l1-doctor.md](l1-doctor.md) - The self-healing subsystem repairs faults; operational health surfaces signals and defers remediation to it (OH-7).
- [l1-practice-analytics.md](l1-practice-analytics.md) - Practice analytics judges the user's working practice; operational health judges the runtime's operating condition.
- [l2-budget-engine.md](l2-budget-engine.md) - Source of cost and token accounting consumed as operational signals (OH-6).
- [l1-storage-model.md](l1-storage-model.md) - Runtime traces (sessions, errors, resource usage) are read from durable state.
- [l1-dashboard.md](l1-dashboard.md) - Primary surface that renders health scores, alerts, and trends.

## 1. Motivation

A local, long-running agent system accumulates failure modes that no single correctness check catches: gateway memory creeping toward an OOM kill, error rates climbing, a workspace going quiet for days, sessions ballooning in length until context management degrades, model spend drifting past intent. Each is invisible until it becomes an incident.

Correctness checks ("is the state consistent?") and repair ("fix the broken thing") do not answer "is this trending toward trouble?". That requires a measurement layer: aggregate the traces the runtime already writes, score them against named thresholds, detect directional anomalies, and raise graded alerts early — while the operator can still act. Because the data is sensitive operational ground truth, the whole layer stays on-device by construction.

## 2. Constraints & Assumptions

- Health is derived from traces the runtime already records (sessions, messages, errors, resource samples, cost/token counts); this spec does not mandate new instrumentation beyond what those traces expose.
- Observation is strictly read-only and on-device; no signal computation mutates observed state and no data leaves the machine.
- Thresholds and signal weights are configurable; the defaults are a sensible baseline, not a contract.
- A "monitored unit" is the system as a whole and each isolated office/workspace independently — scores and alerts are always attributed to a unit.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **OH-1 (On-device, read-only observation):** operational health is computed entirely from locally-recorded traces; computation never mutates observed state and never egresses data (consistent with the local-first, no-exfiltration posture).
- **OH-2 (Explainable health score):** each monitored unit carries a bounded composite health score (a fixed numeric range). Every deduction from full health traces to a named signal and the threshold it crossed; the score is never an opaque number — its breakdown is always reconstructable.
- **OH-3 (Severity-graded alerts):** a signal crossing a declared threshold raises an alert classified `critical | warning | info`, carrying the metric name, current value, threshold, and a human-readable detail. Alerts are deduplicated per unit, per signal, per period so a standing condition does not flood the surface.
- **OH-4 (Trend & anomaly detection):** beyond point thresholds, the layer compares a recent rolling window against a prior window to detect directional anomalies — sharp activity decline, or runaway growth in cost, tokens, or session length — and reports them as anomalies distinct from static threshold breaches.
- **OH-5 (Context-pressure signal):** average interaction length per session is tracked as a leading indicator of context bloat; sustained excess raises a context-management risk before failures occur (consistent with the context-budget tiers).
- **OH-6 (Cost & usage accounting):** token consumption, per-model usage breakdown, and cost are aggregated per unit and per time window as first-class operational signals, feeding both alerts (budget overrun) and the health score.
- **OH-7 (Measure, don't act):** the layer surfaces signals, scores, and risks only. It MUST NOT auto-remediate — repair is owned by the self-healing subsystem, and behavioural coaching by practice analytics. Operational health hands findings to those subsystems; it does not perform their jobs.
- **OH-8 (Inspectable & exportable):** the full health snapshot — scores with their signal breakdowns, active alerts, and trend series — is inspectable by the user and exportable as a self-contained report. No signal is hidden from the operator.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 Signal Taxonomy

A *signal* is a named, measurable operational quantity with an optional threshold and weight. Signals fall into families:

| Family | Example signals | Source |
| --- | --- | --- |
| Reliability | error-rate, crash/restart count, stuck-work count | error log, doctor checks |
| Resource | memory pressure, queue depth | runtime resource samples |
| Engagement | sessions/day, messages/session, idle-since | session traces |
| Economy | cost/window, tokens/window, per-model spend | budget engine |
| Trend | activity-window delta, cost-window delta | derived (§4.3) |

### 4.2 Health Score Composition

```text
[REFERENCE]
score := MAX_SCORE
for each signal with a configured penalty:
    if signal.value crosses signal.threshold(tier):
        score := score − signal.weight(tier)
score := clamp(score, MIN_SCORE, MAX_SCORE)
breakdown := list of { signal, observed, threshold, deduction }   // OH-2
```

The breakdown is retained alongside the score so any deduction is explainable. Tiers (e.g. warning vs critical thresholds for the same signal) carry different weights.

### 4.3 Trend & Anomaly Windows

```text
[REFERENCE]
recent := aggregate(metric, window = last N units of time)
prior  := aggregate(metric, window = preceding N units of time)
if prior > 0 and recent / prior < decline_ratio:   anomaly("activity-drop", pct = 1 − recent/prior)
if prior > 0 and recent / prior > growth_ratio:     anomaly("runaway-growth", metric)
```

Anomalies are reported separately from threshold alerts: a metric can be within its absolute bounds yet still be anomalous in its trajectory.

### 4.4 Alert Model

```text
Alert {
  unit_id   : UnitId
  signal    : string
  severity  : "critical" | "warning" | "info"
  metric    : string
  current   : number
  threshold : number
  detail    : string        // human-readable
  period    : DateBucket    // dedup key with unit_id + signal (OH-3)
}
```

Alerts are sorted by severity, then recency, for presentation. Risk flags (e.g. `critical_health`, `high_errors`, `resource_pressure`, `low_engagement`) are coarse boolean rollups derived from the same signals for quick scanning.

### 4.5 Snapshot & Export

A health snapshot bundles, per monitored unit: the score with its breakdown, active alerts, risk flags, and trend series for the standard metrics, plus system-wide totals. The snapshot is the single inspectable artifact (OH-8) and the basis for the dashboard surface and any exported report.

## 5. Drawbacks & Alternatives

- **Threshold tuning burden:** static thresholds produce false positives until tuned; mitigated by sensible defaults, severity grading, and per-unit dedup (OH-3). A future refinement may learn baselines per unit.
- **Alternative — fold into the self-healing subsystem:** rejected; conflates measurement with repair and violates OH-7's separation. Repair acts on discrete faults; health scores a continuous condition.
- **Alternative — fold into telemetry:** rejected; telemetry is outward-sharing and opt-in, health is local and always-on. Different data ownership and lifecycle.
- **Alternative — fold into practice analytics:** rejected; that subsystem judges the user's practice, this judges the runtime. Different subject, different audience.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[BUDGET]` | `.design/main/specifications/l2-budget-engine.md` | Authoritative source of cost/token accounting consumed by OH-6. |
| `[DASHBOARD]` | `.design/main/specifications/l1-dashboard.md` | Surface contract that renders the health snapshot (OH-8). |
| `[TELEMETRY]` | `.design/main/specifications/l1-telemetry.md` | Boundary spec — defines what stays local vs. what may be shared, which OH-1 must not cross. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-26 | Core Team | Initial spec — operational health layer: explainable health score, severity-graded alerts, trend/anomaly detection, cost/usage accounting, measure-don't-act boundary (OH-1…OH-8). |
