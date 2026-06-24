# Automation Canvas

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The automation canvas is the visual representation of the automation pipeline model. It renders automation pipelines as interactive flow graphs, allowing users to compose, inspect, and debug cross-role automation without editing workflow files directly.

The canvas is a **projection** of the pipeline engine: it does not introduce new execution semantics. Every pipeline composed in the canvas runs through the same engine as pipelines defined implicitly in worker workflow files. The canvas adds visibility and interactive editing; it does not add a separate runtime.

The canvas serves three user needs:
1. **Composition** — create a new explicit pipeline by connecting trigger, logic, and action nodes visually.
2. **Inspection** — observe running and historical pipelines: trace per-node execution, view payloads, spot failures.
3. **Legibility** — make the office's automation graph readable to users who need to understand or modify it without reading DSL files.

## Related Specifications

- [l1-automation-pipeline.md](l1-automation-pipeline.md) — the pipeline model and engine this canvas renders; all node types, trigger types, and execution invariants are defined there
- [l1-office-visualization.md](l1-office-visualization.md) — the office spatial projection; the canvas may be embedded as a panel within it
- [l1-extensions.md](l1-extensions.md) — third-party action nodes backed by extensions
- [l2-app-ui.md](l2-app-ui.md) — desktop/web UI layer that hosts the canvas panel

## 1. Motivation

Implicit automation (per-worker `@ON:` rules) is adequate for isolated, per-role reactions. As office complexity grows — multiple roles handing off to each other, conditional branching across role boundaries, time-gated aggregations — implicit automation becomes difficult to audit or modify because it is scattered across many workflow files.

The canvas makes the whole-office automation graph visible in one place. It answers:
- What happens when event X arrives?
- Which roles are involved in this pipeline?
- Why did this pipeline fail on Tuesday?
- How do I add a delay between step 3 and step 4?

Without a canvas, the answers require reading and cross-referencing multiple nodus files. With the canvas, they are visual.

## 2. Constraints & Assumptions

- The canvas renders the pipeline model from `l1-automation-pipeline.md` — it does not define its own node types, trigger types, or execution semantics.
- The canvas operates in read-write mode for explicit pipelines and read-only mode for implicit pipelines (which are authored in workflow files, not in the canvas).
- The canvas must not require the user to understand nodus DSL syntax to create or modify pipelines.
- Canvas state (pipeline definitions, layout positions) persists in the office's local storage; no canvas state is sent to external services.
- The canvas is office-scoped: it shows only the automation graph of the current office.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **AC-1 Projection fidelity**: the canvas graph faithfully represents the running pipeline engine state. A pipeline that exists in the engine but is absent from the canvas, or vice versa, is a consistency violation. Stale canvas state must refresh automatically on engine events.

- **AC-2 Read-only implicit surfaces**: implicit pipelines (embedded in worker workflow files) appear in the canvas as read-only nodes. The canvas MUST NOT write changes back to workflow files. Users who want to modify an implicit pipeline must be directed to the workflow file or offered a conversion to an explicit pipeline (which creates a new definition file, not modifying the original).

- **AC-3 No runtime divergence**: the canvas does not execute pipelines independently. All execution is delegated to the pipeline engine. The canvas observes results via the AuditProvider trace stream; it MUST NOT have its own execution path for pipeline nodes.

- **AC-4 Data safety in inspection**: the canvas displays execution traces (per AP-6 in `l1-automation-pipeline.md`). These traces obey the data-safety boundary from `l1-nodus-observability.md` §4.4 — descriptors only, never verbatim user content. The canvas MUST NOT attempt to reconstruct or display raw event payloads beyond what the trace exposes.

- **AC-5 Local persistence**: canvas layout (node positions, zoom level, panel states) and explicit pipeline definitions are stored in the office's local state store. They MUST NOT be transmitted to any external service. The office's backup mechanism covers canvas state.

- **AC-6 Graceful degradation**: if the pipeline engine is offline or unreachable, the canvas displays last-known state with a staleness indicator. It MUST NOT block the user from viewing or editing pipeline definitions in offline mode — edits are queued for sync when the engine reconnects.

## 4. Detailed Design

### 4.1 Canvas Structure

The canvas presents three panels:

```text
[REFERENCE]
┌──────────────────────────────────────────────────────┐
│ Pipeline Selector (left rail)                        │
│  ├── Explicit pipelines (editable)                   │
│  └── Implicit pipelines per role (read-only)         │
├─────────────────────────────┬────────────────────────┤
│ Flow Graph (center)         │ Node Inspector (right) │
│  Nodes + edges + live state │  Config, trace, errors │
└─────────────────────────────┴────────────────────────┘
│ Execution Log (bottom, collapsible)                  │
│  Per-run trace events; filter by run_id / node / time│
└──────────────────────────────────────────────────────┘
```

### 4.2 Node Rendering

Each node type (§4.1 of `l1-automation-pipeline.md`) has a distinct visual form and a live execution state indicator:

| Node type | Visual indicator | Live execution state |
| --- | --- | --- |
| `trigger` | Entry arrow | last fired time + dedup window status |
| `filter` | Diamond (boolean) | pass / drop / not-yet-evaluated |
| `transform` | Gear | last transform input→output summary |
| `branch` | Diamond (multi-path) | which path was taken on last run |
| `action` | Filled rectangle | dispatched / pending / error |
| `delay` | Clock | remaining wait time; paused / resumed |
| `aggregate` | Funnel | events collected / window threshold |
| `loop` | Cycle arrow | current iteration / max_iterations |

Node edges carry the event payload descriptor (field names and types, not values). Clicking an edge in the inspector shows the last observed descriptor.

### 4.3 Explicit Pipeline Editing

When editing an explicit pipeline, the canvas provides:

1. **Node palette** — draggable node types organized by category (trigger, logic, action). Nodes from the extension registry appear as action nodes automatically.
2. **Connection rules** — validated live: trigger nodes can have only outbound edges; filter nodes require both pass and optionally a drop branch; action nodes must be leaf nodes unless followed by an error branch.
3. **Configuration panel** — each selected node exposes its configuration fields (trigger type, condition expression, action target role, delay duration, etc.) in the right-side inspector.
4. **Validation feedback** — the canvas reports AP-2, AP-3, AP-7 invariant violations inline before the pipeline is saved.

Editing is non-destructive: each save creates a new version of the pipeline definition (per LP-6 in `l1-automation-pipeline.md` §4.6). Prior versions are accessible from the pipeline selector.

### 4.4 Inspection and Debugging

For any pipeline run (explicit or implicit):

1. **Run list** — the execution log shows all runs within the selected time window, with status (ok / error / condition-halt) and duration.
2. **Run trace** — clicking a run expands it to a per-node sequence. Each node entry shows: emitted event type, elapsed time, outcome. Error nodes show error code and detail.
3. **Replay** — the canvas can re-display a historical run's trace against the current pipeline graph, even if the pipeline definition has changed. This enables post-mortem analysis without re-executing.
4. **Live mode** — while a pipeline is running, the canvas highlights the currently active node in real time, sourced from the AuditProvider event stream.

### 4.5 Implicit Pipeline Surfaces

Implicit pipelines (from `@ON:` blocks) appear in the canvas as a collapsed group per role. Expanding the group shows the individual trigger-action pairs as read-only nodes. The canvas renders a "Convert to explicit" action, which:
1. Extracts the `@ON:` blocks from the workflow file into a standalone pipeline definition
2. Marks the source `@ON:` blocks as superseded in the workflow file (without deleting them — the user confirms removal)
3. Registers the new explicit pipeline with the same semantics

This conversion does not change execution behavior — it only moves ownership from the workflow file to the pipeline registry.

## 5. Implementation Notes

1. Implement the canvas in inspection-only mode first — connect it to the AuditProvider stream and display run traces. This validates the data pipeline before editing is added.
2. Build the explicit pipeline editor as a serialization layer over the pipeline model: the canvas never stores its own representation; it reads and writes the canonical pipeline definition format.
3. The node palette is driven by the same registry as the pipeline engine — adding a new node type requires no canvas-specific code beyond a rendering entry.
4. Live mode uses the same AuditProvider event stream as the observability system; the canvas is a display consumer, not an additional observer.

## 6. Drawbacks & Alternatives

**Alternative: canvas-only (no implicit mode)** — require all automation to be authored in the canvas; eliminate embedded `@ON:` blocks. Rejected: violates the office autonomy promise for routine per-worker reactions, which should not require user interaction to configure.

**Alternative: embedded canvas in the office visualization** — render automation pipelines as an overlay on the spatial office floor plan. Compatible with this spec; the canvas can be embedded as a panel within the office visualization view. Deferred: implement as a layout option after the standalone canvas is stable.

**Alternative: text-based pipeline editor with syntax highlighting** — skip the graphical canvas; expose pipeline definitions as editable DSL files with IDE-like tooling. Viable as a power-user alternative but does not satisfy the legibility goal for non-technical users. The visual canvas and a text editor are not mutually exclusive; both can be provided as views of the same definition.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[PIPELINE-MODEL]` | `.design/main/specifications/l1-automation-pipeline.md` | Node taxonomy, trigger types, and execution semantics the canvas renders |
| `[NODUS-OBS]` | `.design/nodus/specifications/l1-nodus-observability.md` | AuditProvider event stream powering live mode and run trace (AC-3, AC-4) |
| `[OFFICE-VIZ]` | `.design/main/specifications/l1-office-visualization.md` | Spatial context; canvas may embed here as a panel |
| `[APP-UI]` | `.design/main/specifications/l2-app-ui.md` | Desktop/web UI host layer |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — AC-1…AC-6, three-panel layout, node rendering, explicit editing, implicit surfaces, inspection and debugging |
