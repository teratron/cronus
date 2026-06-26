# Agent Tool Ergonomics

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Agent tool ergonomics is the design discipline for making an agent-facing tool surface (MCP tools, extension tools, the in-session tool API) one that an agent *actually uses well* — rather than one that is technically correct but that the agent ignores, misuses, or abandons. The central, counter-intuitive finding: a tool's only channels for influencing an agent (its name, description, and the server's initialization instructions) are **low-salience** — they do not reliably change which tool the agent picks or how it phrases a call. So ergonomics is won not by *instructing the agent to behave differently* but by *shaping tool input/output/errors around the behavior the agent already has*.

The discipline turns on one measurable question: **did the tool's response let the agent stop?** An agent falls back to a manual workaround (re-reading files, shelling out, grepping) the instant a tool's answer is insufficient. Every ergonomic rule here exists to keep that from happening: make output sufficient, never train abandonment through errors, expose absence unmissably, scale budgets with the work, and validate the whole thing by ablation on the weakest model real users run.

This concept is tech-neutral and cross-cutting: it governs the code-intelligence tools ([l1-code-intelligence.md](l1-code-intelligence.md)), but equally the memory, knowledge-base, automation, and any future MCP surface.

## Related Specifications

- [l1-tool-composition.md](l1-tool-composition.md) — composition/grouping/deferred-resolution of tools (TC-1…TC-7); ergonomics is the orthogonal axis of *how each tool's I/O is shaped for an agent*. ATE-8 reuses the TC-7 surface-reduction model.
- [l2-agent-session.md](l2-agent-session.md) — `ToolSurfaceProfile` and the tool-call loop; the place these invariants are enforced at runtime.
- [l1-extensions.md](l1-extensions.md) / [l2-extension-registry.md](l2-extension-registry.md) — `ToolDefinition` (`promptSnippet`/`promptGuidelines`/`renderResult`) is where per-tool output shaping and the `isError` contract live.
- [l1-code-intelligence.md](l1-code-intelligence.md) — the worked subject: its tools must be sufficient-to-stop (CI-6/CI-7) and signal staleness (CI-3); ATE-6 is the general form of that signal.
- [l1-retrieval-evaluation.md](l1-retrieval-evaluation.md) — sibling measurement concept; ATE-9 (task-level A/B ablation) complements RE's ranking-quality metrics.
- [l1-output-contracts.md](l1-output-contracts.md) — validators on tool/task output; ATE-1 sufficiency is the agent-consumption counterpart of output validity.

## 1. Motivation

A tool can be correct and still fail in practice: the agent never calls it, calls it with the wrong query, or calls it once, gets a partial answer, and silently reverts to grep for the rest — paying full discovery cost anyway. Teams repeatedly try to fix this by *editing instructions* ("tell the agent to prefer the tool", "add better examples", "introduce a `trace` tool") and find it doesn't move behavior: those channels are low-salience, and new tools are under-picked. What *does* move behavior is changing what the tools the agent already reaches for return.

The failure modes are specific and recurring: a non-monotonic output budget that truncates exactly the large inputs the tool exists for; a hard error early in a session that makes the agent abandon the tool for the rest of it; a half-bridged flow that reveals a gap the agent then drills into manually; eight tools that all error on an unconfigured workspace instead of honestly exposing none. Each is an ergonomics defect, not a logic bug, and none is visible to a unit test — only to an end-to-end, with-vs-without measurement.

Capturing these as invariants gives every Cronus tool surface a shared contract for agent-facing quality, and a validation method (ATE-9) that doesn't fool itself by testing on a model strong enough to paper over the defect.

## 2. Constraints & Assumptions

- **Low-salience reality.** Tool descriptions and init instructions inform but do not control an agent; assume they cannot reliably change tool choice or query style.
- **Fallback is always available.** The agent can always re-read files / grep / shell out. The tool competes with that fallback on every call; "insufficient" loses.
- **Run-to-run variance is large.** Agent behavior is stochastic; any ergonomic claim needs medians over multiple runs, never n=1.
- **The floor model is the bar.** Real users attach the cheapest adequate model; an affordance must work there. Validating only on the strongest model hides salience/sufficiency defects.
- **Ergonomics ≠ requirements.** A tool supplies context/capability, not product intent; these rules shape delivery, not what the agent should decide.

## 3. Core Invariants

Layer 2 implementations MUST NOT violate these. They are technology-neutral.

- **ATE-1 — Sufficiency-to-stop.** A tool's output is designed so the agent does not fall back to a manual workaround to finish the task the tool claims to serve. The judging question for any change is "is the answer sufficient enough to *stop* the agent from reading?" A complete-but-larger response beats a smaller one that forces a fallback.
- **ATE-2 — Recoverable conditions return success-shaped guidance, not errors.** Expected, recoverable states (not initialized, symbol/record not found, out of index, empty result) return a success response carrying actionable next-step guidance. A hard/`isError` response is reserved for genuine "stop trying" cases: a security refusal, or a real malfunction (which carries a retry-once note). Rationale: a couple of hard errors early in a session teach the agent to abandon the tool entirely.
- **ATE-3 — Absence is the unmissable signal.** When a capability is unavailable or unconfigured, the surface exposes *nothing operational* — an empty tool list plus a short "inactive" notice — rather than tools that all fail. Absence is the one signal an agent cannot misread; activation stays the user's decision.
- **ATE-4 — Adapt the tool to the agent, not the agent to the tool.** Prefer making a tool the agent *already calls* do more with the input it *already gives* over (a) introducing a new tool the agent must learn to pick, or (b) relying on instructions/examples to change tool choice or query style. A change that needs the agent to behave differently hits the low-salience wall and will not land.
- **ATE-5 — Output budget monotonic in workload.** Per-call output budgets (total size, item count, per-item size) are monotonically non-decreasing in the size of the workload they serve: a larger tier never grants a smaller per-item budget than a smaller tier. A non-monotonic budget silently truncates the largest inputs — the exact case the tool exists for — and forces fallback.
- **ATE-6 — Explicit, per-item staleness signaling.** A tool that may serve best-effort/stale data signals staleness explicitly and per item — naming the stale item and directing the agent to verify it directly — rather than returning a possibly-wrong answer silently. (General form of [l1-code-intelligence.md](l1-code-intelligence.md) CI-3.)
- **ATE-7 — Never steer to a manual fallback.** Tool output never instructs the agent to "use the file reader / grep". When more is needed it steers to another call of the same tool family and frames already-returned data as authoritative for the turn. Steering to the fallback trains the tool's own abandonment.
- **ATE-8 — Lean surface by measured pick-rate.** Expose the tools agents actually pick; fold the information from rarely-picked capabilities inline into the primary tools and keep the rest reachable by opt-in. Surface size has a context cost paid every session. (Reuses the reduction model of [l1-tool-composition.md](l1-tool-composition.md) TC-7.)
- **ATE-9 — Validate by A/B ablation on the floor model.** Ergonomic claims are validated with-vs-without the capability, measuring *task-level* outcomes (tool-call count, manual-fallback count, wall-clock), as a median over multiple runs, on the deliberate **floor model** (the weakest model real users attach) — not the strongest. An affordance that lands on the floor model generalizes up; one that needs the strongest model does not generalize down. Both arms use the same model.

## 4. Concept Detail

### 4.1 The sufficiency loop

The agent's loop is: call tool → is the answer enough? → if yes, stop; if no, fall back to manual discovery. Ergonomics optimizes the "enough" branch. Practical levers (all ATE-1):

- Return the *answer*, sized to the answer, not the artifact: surface the relevant pieces in full even when buried in a large container, and collapse redundant/interchangeable detail to summaries — so the response tracks the question, not the file count.
- For an ambiguous request, return *all* viable resolutions in one call (e.g. every overload) so the agent never makes a second manual pass to disambiguate.
- Attach the adjacent context the agent would otherwise fetch next (callers/dependents/trail) inline, so "one more lookup" is unnecessary.

### 4.2 Error taxonomy for agents (ATE-2)

| Condition | Response shape | Why |
| --- | --- | --- |
| Not initialized / not found / empty / out-of-scope-but-valid | **Success** + guidance text | Recoverable; the agent should adjust, not abandon |
| Security refusal | **Hard error** | A genuine "do not retry" |
| Real malfunction | **Hard error** + retry-once note | Honest failure, bounded retry |
| Capability unconfigured | **No tools exposed** + inactive notice (ATE-3) | Absence is unmissable |

The asymmetry is deliberate: false "errors" are far more costly than verbose successes, because one or two early hard errors end the agent's use of the tool for the whole session.

### 4.3 Budget monotonicity (ATE-5)

Output budgets scale with the workload (e.g. indexed size): both the *number* of calls allowed and the *per-call/per-item* size grow with scale, and the per-item size never inverts across tiers. The canonical regression is a middle tier whose per-item cap sits below a smaller tier's, so a single large item (a giant file, a god-object) returns a uselessly small slice and forces a manual read. Treat per-item budget monotonicity as an invariant checked whenever tiers change.

### 4.4 Validation method (ATE-9)

- **Arms:** identical task, with the capability vs without; same model both arms.
- **Metrics:** task-level — tool-call count, manual-fallback (read/grep) count, wall-clock; optionally a forced-no-fallback "sufficiency proof" run.
- **Statistics:** median of N (≥2–4); report the range, never a single run.
- **Model:** the floor model on purpose. A stronger model's tool-use masks the salience/sufficiency defects a weaker one exposes; validating on the floor model is what makes an affordance generalize to every host.
- **Isolation:** to attribute a change, also run new-build vs baseline-build with the capability on in both arms.

## 5. Ideas to Adopt

| Mined mechanic | Adoption in Cronus |
| --- | --- |
| Sufficiency-to-stop as the design target | **[new]** ATE-1; the acceptance bar for every MCP/extension tool's output (`renderResult` in `l2-extension-registry.md`). |
| Recoverable-as-success vs reserve-hard-error | **[new]** ATE-2; tightens the `isError` contract in `l2-extension-registry.md` / `l2-agent-session.md` into a design rule, not just a field. |
| Absence (empty tool list + inactive notice) as the signal | **[new]** ATE-3; the unindexed/unconfigured posture for any capability surface. |
| Adapt-tool-to-agent over instruct-the-agent | **[new]** ATE-4; a standing rule for tool/skill authoring; explains why `promptGuidelines` edits under-deliver. |
| Monotonic output budget | **[new]** ATE-5; a checkable invariant for any tier-scaled tool output (directly applies to code-intelligence CI-6 budgeting). |
| Per-item staleness signaling | ATE-6; the general form of `l1-code-intelligence.md` CI-3, reusable by any best-effort cache surface (memory, knowledge base). |
| Never steer to manual fallback | **[new]** ATE-7; output-wording rule for all tools. |
| Lean surface by measured pick-rate | ATE-8; reuses `l1-tool-composition.md` TC-7 + `ToolSurfaceProfile`; adds the *measured* basis (fold rarely-picked tools inline). |
| A/B ablation on the floor model | **[new]** ATE-9; complements `l1-retrieval-evaluation.md` (ranking metrics) and `l1-evaluation-suites.md` (customization behavior) with task-level capability ablation + an explicit floor-model policy. |

## 6. Nodus Relevance

Nodus exposes a library + CLI/TUI surface and an error taxonomy (the `NODUS:*` codes) and an audit/observability contract — all agent-consumable, so the ergonomics transfer:

- **Error taxonomy ↔ ATE-2.** The `NODUS:*` codes should be classified into "recoverable → success-shaped diagnostic with a fix hint" vs "stop-trying → hard error" so an agent driving nodus (validate/run) adjusts rather than abandons. Validator diagnostics (E0xx/W0xx) are the natural success-shaped guidance channel.
- **Sufficiency ↔ ATE-1.** `validate` and `run` output should carry enough (the failing step, the rule hit, the expected vs actual) that an agent fixes the workflow without re-reading the whole `.nodus` file.
- **Absence ↔ ATE-3.** A nodus host with no workflow loaded should present nothing runnable rather than commands that all error.
- **Floor-model validation ↔ ATE-9.** Nodus-as-harness evaluation (its testing contract) should A/B on the floor model so a workflow affordance that helps a weak model is what ships.

The nodus workspace owns any realization; this records the relevance.

## 7. Drawbacks & Alternatives

- **Verbose successes cost tokens.** Returning guidance instead of errors and folding adjacent context inline enlarges responses. The trade is deliberate (ATE-1/ATE-2): a fallback round-trip costs more than a larger single response, and a false error costs a whole session's tool use.
- **Floor-model validation can under-shoot strong hosts.** An affordance tuned for the weakest model may leave value on the table for the strongest. Acceptable: the floor is where adoption breaks, and generalizing up is free; the inverse is not.
- **Measured pick-rate can entrench.** Folding rarely-picked tools away (ATE-8) risks hiding a genuinely useful but undiscovered tool. Mitigation: keep them opt-in reachable and re-measure as host models improve at tool use.
- **Alternative — instruct the agent harder.** The discarded path: richer prompts/examples to change tool choice. Repeatedly observed to under-deliver (ATE-4); kept only as a weak complement, never the mechanism.

## Document History

| Version | Change |
| --- | --- |
| 1.0.0 | Initial spec — ATE-1…ATE-9: agent-facing tool-surface ergonomics (sufficiency-to-stop, recoverable-as-success, absence-as-signal, adapt-tool-to-agent, monotonic budget, staleness signaling, no-manual-fallback steering, lean surface by pick-rate, A/B ablation on the floor model); ideas-to-adopt + nodus-relevance mapping |

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[COMPOSITION]` | `.design/main/specifications/l1-tool-composition.md` | Orthogonal tool-composition axis (TC-7 surface reduction) |
| `[SESSION]` | `.design/main/specifications/l2-agent-session.md` | Runtime tool-call loop + `ToolSurfaceProfile` enforcement point |
| `[REGISTRY]` | `.design/main/specifications/l2-extension-registry.md` | `ToolDefinition` / `isError` / `renderResult` contract |
