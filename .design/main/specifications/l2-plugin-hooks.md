# Plugin Hook System

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-extensions.md

## Overview

The plugin hook system is the runtime complement to the extension registry. While the registry handles install-time lifecycle (discover → permit → activate), hooks are invoked at runtime at defined actor lifecycle and bus event boundaries. A plugin registers a `Hooks` object when it initializes; the runtime calls the relevant hook functions at each boundary, passing mutable output structs that plugins fill in to influence behavior.

## Related Specifications

- [l1-extensions.md](l1-extensions.md) - The unified extension model that hooks implement.
- [l2-extension-registry.md](l2-extension-registry.md) - Registry that activates plugins, making their Hooks available.
- [l2-agent-session.md](l2-agent-session.md) - Turn lifecycle; preStop/postStop hooks fire around actor completion.
- [l2-agent-registry.md](l2-agent-registry.md) - `ActorMatcher` may reference agent types defined in the registry.
- [l2-agent-autonomy.md](l2-agent-autonomy.md) - Subagent progress checker (built-in hook) uses autonomy context.

## 1. Motivation

Extension authors need to observe and influence agent behavior without forking the core. Hooks provide defined injection points — before an actor stops, after it delivers its result, on every bus event — with a fail-forward contract that protects the core loop from misbehaving plugins.

## 2. Constraints & Assumptions

- Hooks are registered in deterministic order (internal plugins first, then file hooks, then external plugins, each group in registration order). Execution order matches registration order.
- Hook execution is sequential within a group (not concurrent) to preserve deterministic behavior.
- Hooks must be fail-forward: an error in one hook is logged and a Session.Event.Error is published, but execution of the remaining hooks continues.
- Re-entry loops (preStop/postStop) are capped to prevent infinite continuation.
- File hooks are reloaded on config change; other hooks are loaded once at plugin initialization time.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| EXT-1 Unified model | All hook sources (built-in, file, external npm) register the same `Hooks` interface. |
| EXT-3 Default-deny | Hooks may not grant themselves additional tool permissions; they operate within the actor's existing Ruleset. |
| EXT-4 Sandboxed | External plugin hooks run inside the security sandbox; network access flows through the egress gate. |
| EXT-8 Provenance & audit | `HookEvent.Executed` records hook ID, plugin name, duration, and outcome for every invocation. |

## 4. Detailed Design

### 4.1 Hooks interface

```text
[REFERENCE]
Hooks {
  // Fires before the actor's completion is reported to the caller.
  // May trigger a re-entry turn (continue=true). Capped at MAX_PRE_REACT.
  "actor.preStop"?: HookFn<ActorPreStopInput> | HookEntry<ActorPreStopInput>

  // Fires after the actor's result is delivered to the caller.
  // May spawn a new actor turn (continue=true). Capped at MAX_POST_REACT.
  // Result of the new turn is NOT propagated to the original caller.
  "actor.postStop"?: HookFn<ActorPostStopInput> | HookEntry<ActorPostStopInput>

  // Fires for every bus event (fire-and-forget; no output struct).
  "event"?: (EventInput) => void

  // Fires when workspace config loads or reloads.
  "config"?: (Config) => Promise<void>

  // Fires during system-prompt construction; allows transforming the system array.
  "experimental.chat.system.transform"?:
    (SystemTransformInput, SystemTransformOutput) => Promise<void>
}

type HookFn<I> = (input: I, output: ActorStopOutput) => Promise<void>

type HookEntry<I> = {
  run:      HookFn<I>,
  matcher?: ActorMatcher,
}

ActorMatcher: { agent_type: String | Vec<String> }
  // Hook fires only for actors whose agent_type matches.
  // If absent, hook fires for all actors.

ActorStopOutput {
  continue: bool,       // set true to request re-entry
  reason:   Option<String>, // re-entry prompt; required when continue=true
}
```

### 4.2 ActorPreStopInput and ActorPostStopInput

```text
[REFERENCE]
ActorPreStopInput {
  actor_id:    String,
  session_id:  String,
  agent_type:  String,
  final_text:  Option<String>,
  structured:  Option<Value>,
}

ActorPostStopInput {
  actor_id:    String,
  session_id:  String,
  agent_type:  String,
  outcome:     AgentOutcome,   // success | failure | cancelled
}
```

### 4.3 preStop ReAct loop

The preStop loop runs before the actor's completion is reported to its caller:

```text
[REFERENCE]
MAX_PRE_REACT = 3

preStopLoop(input: ActorPreStopInput):
  for iteration in 0..MAX_PRE_REACT:
    decision = aggregateDecision(input, "actor.preStop")
    if !decision.continue:
      break
    if decision.reason is empty:
      log.warn("hook returned continue=true without reason; ignored")
      break
    publish(HookEvent.ReActReentered, { phase="pre", iteration, ... })
    runTurn(reason=decision.reason)   // injects reason as a new user turn
  else:
    publish(HookEvent.ReActMaxReached, { phase="pre", ... })
```

If the loop exhausts MAX_PRE_REACT iterations without a hook returning `continue=false`, the actor stops regardless (cap is a hard ceiling, never overridden by plugins).

### 4.4 postStop ReAct loop

The postStop loop runs after the actor's result is delivered:

```text
[REFERENCE]
MAX_POST_REACT = 3

postStopLoop(input: ActorPostStopInput):
  for iteration in 0..MAX_POST_REACT:
    decision = aggregateDecision(input, "actor.postStop")
    if !decision.continue:
      break
    publish(HookEvent.ReActReentered, { phase="post", iteration, ... })
    newTurn(reason=decision.reason)   // spawns a new actor turn; result NOT propagated
  else:
    publish(HookEvent.ReActMaxReached, { phase="post", ... })
```

The postStop loop can fire additional work (e.g., memory consolidation, status updates) without the original caller waiting for it.

### 4.5 Aggregation across plugins

Multiple plugins may each request re-entry in the same hook cycle. The decision is aggregated:

```text
[REFERENCE]
aggregateDecision(input, event_name) -> ActorStopAggregatedDecision:
  reasons = []
  plugin_names = []
  hook_ids = []
  any_continue = false

  for entry in all_hooks_with_meta:
    if entry.matcher is set AND !matchesActor(entry.matcher, input):
      publish(HookEvent.Executed, outcome="skipped")
      continue

    output = ActorStopOutput { continue: false }
    try:
      entry.hook[event_name](input, output)
    catch err:
      log.error(...)
      publish(Session.Event.Error, ...)
      continue   // fail-forward

    publish(HookEvent.Executed, outcome=..., continue_requested=output.continue, ...)

    if output.continue == true AND output.reason is non-empty:
      any_continue = true
      reasons.push(output.reason)
      plugin_names.push(entry.plugin_name)
      hook_ids.push(entry.hook_id)
    elif output.continue == true:
      log.warn("continue=true without reason; ignored")

  return ActorStopAggregatedDecision {
    continue: any_continue,
    reason: reasons.join("\n\n"),
    contributing_plugin_names: plugin_names,
    contributing_hook_ids: hook_ids,
  }
```

All contributing reasons are joined (separated by double newline) as the re-entry prompt. This lets multiple plugins each contribute context to the model's next turn.

### 4.6 File hooks (no install required)

File hooks are auto-discovered from directories listed in the workspace config:

```text
[REFERENCE]
Pattern: {hook,hooks}/*.{js,ts}
  in each directory from config.directories()

Loading:
  1. Glob match the pattern in each directory.
  2. Import each matched file (ESM import with cache-busting via ?v=timestamp).
  3. Treat module.default ?? module as the Hooks object if it is a non-null object.
  4. Register with plugin_name = "file:<basename_without_ext>".
  5. hook_id_for(event) = "file:<basename>#<event>".

Reloading:
  reloadFileHooks() invalidates the file-hook state and re-runs discovery.
  Called by extension-registry on config change (extension install/uninstall, hook file edit).
```

File hooks do not require npm install or an explicit `plugin_origins` entry. They are a lightweight escape hatch for workspace-local behavior.

### 4.7 External plugins (npm-installable)

```text
[REFERENCE]
Loading (from config.plugin_origins[]):
  1. config.waitForDependencies() — wait for npm install to complete.
  2. PluginLoader.loadExternal({ items: plugin_origins, kind: "server" }) resolves
     each specifier to a module and loads it.
  3. For each loaded module:
       a. Detect v1 plugin: { id, server: fn } → call server(input, options) → Hooks.
       b. Detect legacy plugin: function export → call fn(input) → Hooks.
  4. Register sequentially (deterministic order).

Error stages and behavior:
  install  → log error + publish session error; plugin skipped.
  compat   → log warning + publish session error; plugin skipped.
  entry    → log error + publish session error; plugin skipped.
  load     → log error + publish session error; plugin skipped.
  (Other plugins in the same load batch continue regardless.)
```

### 4.8 Built-in plugins

Built-in plugins are unconditionally loaded before file and external plugins:

| Plugin | Purpose |
| --- | --- |
| Auth plugins (per provider) | OAuth/token refresh, credential handoff via preStop/postStop hooks. |
| CheckpointSplitoverPlugin | Manages context-window boundaries; triggers checkpoint-writer at the right split point. |
| SubagentProgressCheckerPlugin | Monitors idle or stalled subagents; injects a nudge via postStop if a subagent has not made progress within a threshold. |

### 4.9 Observability events

```text
[REFERENCE]
HookEvent.Executed {
  event:               "actor.preStop" | "actor.postStop",
  hook_id:             String,     // "<plugin_name>#<event>"
  plugin_name:         String,
  actor_id:            String,
  agent_type:          String,
  duration_ms:         u64,
  outcome:             "success" | "error" | "skipped",
  continue_requested:  bool,
  reason_length:       usize,
}

HookEvent.ReActReentered {
  phase:                    "pre" | "post",
  actor_id:                 String,
  agent_type:               String,
  iteration:                u32,
  triggered_by_plugins:     Vec<String>,
  reason_preview:           String,    // first 200 chars of the concatenated reason
}

HookEvent.ReActMaxReached {
  phase:      "pre" | "post",
  actor_id:   String,
  agent_type: String,
}
```

All events are published on the workspace event bus; subscribers (TUI, telemetry) may react without coupling to hook execution.

## 5. Drawbacks & Alternatives

- **Sequential execution:** deterministic but not concurrent. A misbehaving plugin that hangs delays all subsequent hooks. Mitigated by per-hook timeouts (future work).
- **Fail-forward error contract:** a plugin error is logged but does not fail the actor. This is intentional — a buggy plugin should not take down the whole turn.
- **MAX_PRE_REACT/POST_REACT = 3:** arbitrary but prevents runaway loops. Configurable per workspace.
- **Alternative — event-driven hooks only:** simpler, but preStop/postStop re-entry capability is needed for automatic memory consolidation and subagent health checks without user involvement.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[EXT]` | `.design/main/specifications/l1-extensions.md` | Unified extension model |
| `[REGISTRY]` | `.design/main/specifications/l2-extension-registry.md` | Install-time lifecycle |
| `[SESSION]` | `.design/main/specifications/l2-agent-session.md` | Turn lifecycle; hook integration points |
| `[SPAWN]` | `.design/main/specifications/l2-orchestration.md` | Spawn machinery that invokes the preStop/postStop loops |
