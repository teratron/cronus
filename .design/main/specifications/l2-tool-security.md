# Tool Security

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-security.md

## Overview

Two-layer defense against malicious or accidental tool misuse: a static skill scanner that inspects content at install/load time, and a runtime tool guard that evaluates every tool call before execution. Together they form a defense-in-depth that catches threats both when extensions enter the system and when they attempt to act.

## Related Specifications

- [l1-security.md](l1-security.md) - The model this implements (SEC-4 sandbox, SEC-6 audit).
- [l2-security.md](l2-security.md) - Secret isolation, egress gate, sandbox backend.
- [l2-extension-registry.md](l2-extension-registry.md) - Extension activation; skill scanner runs at the activation gate.
- [l2-orchestration.md](l2-orchestration.md) - Approval gate that tool guard can escalate into.
- [l2-scheduler.md](l2-scheduler.md) - CronPromptInjectionBlocked — fire-time injection scan uses the skill scanner.

## 1. Motivation

Skills and plugins are untrusted content; tool calls are untrusted actions. A static scan prevents clearly-malicious content from entering the system; a runtime guard prevents an activated extension from exceeding its intended scope or being subverted through prompt injection at execution time. Neither layer is sufficient alone: static scanning misses runtime context; runtime-only checks let malicious content reside undetected until it fires.

## 2. Constraints & Assumptions

- The skill scanner runs synchronously at install/load time; it does not make LLM calls.
- The tool guard runs synchronously before each tool call; latency must be sub-millisecond for the common (safe) path.
- Both layers use pattern matching (regex); they are heuristics, not proofs.
- Hard-blocked actions are denied immediately with no escalation path — they are unconditional.
- Escalation (`SuspendedPermission`) routes through the same approval gate as other high-impact actions.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| SEC-4 Data vs telemetry | Skill scanner ensures skills do not contain user-data-exfiltration patterns before activation. |
| SEC-6 Auditable | Every guard finding and approval/denial appends to the audit log. |
| SEC-3 No exfiltration | Tool guard's `data_exfiltration` and `network_abuse` categories enforce the egress gate at call time. |
| SEC-2 Safe defaults | `ToolExecutionLevel` defaults to SMART (balanced); OFF requires explicit config change. |

## 4. Detailed Design

### 4.1 Layer 1 — Skill Scanner (static, at install/load)

The scanner inspects skill content (Markdown, YAML, any text files in the skill package) against a library of named signature rules. It runs when a skill is added (`ext add`) and optionally on demand (`ext scan <id>`).

#### Attack categories

| Category | What it detects |
| --- | --- |
| `command_injection` | Shell commands embedded in skill instructions; backtick/`$()` patterns |
| `data_exfiltration` | Instructions to send user data to external endpoints |
| `hardcoded_secrets` | Literal API keys, tokens, private keys inside skill files |
| `obfuscation` | Base64 blobs, unusual encoding suggesting hidden payloads |
| `prompt_injection` | Instructions that attempt to override or suppress the agent's system behavior |
| `social_engineering` | Manipulative authority/urgency framing inside skill instructions |
| `supply_chain` | Install-script tricks, dependency substitution patterns |
| `unauthorized_tool_use` | Instructions to use tools outside declared capabilities |

#### Signature rule format

```text
[REFERENCE]
- id: CATEGORY_DESCRIPTOR
  category: command_injection          # one of the 8 categories
  severity: HIGH                       # CRITICAL | HIGH | MEDIUM | LOW | INFO
  patterns:
    - "(?i)regex_pattern_here"         # regex; list = any match fires
  file_types: [markdown]               # markdown | yaml | text | any
  description: "Human-readable intent"
  remediation: "How to fix"
```

Patterns are tested case-insensitively; multilingual variants are listed within the same rule.

#### Scan result

```text
[REFERENCE]
ScanFinding {
  id, rule_id, category, severity,
  file, line, snippet,
  description, remediation
}

ScanResult {
  findings: ScanFinding[],
  is_safe: bool,         // true when no CRITICAL or HIGH findings
  max_severity: Severity
}
```

Skills with CRITICAL or HIGH findings are blocked from activation. MEDIUM and below surface as warnings; the user can override with an explicit `--force` flag and the override is logged to the audit trail.

### 4.2 Layer 2 — Tool Guard (runtime, at each tool call)

The guard intercepts every tool call before execution and evaluates its parameters through a set of guardians. The result determines whether the call proceeds, is escalated for approval, or is hard-blocked.

#### Threat categories

| Category | Guardian | Examples |
| --- | --- | --- |
| `command_injection` | shell_evasion_guardian | semicolons, pipes, `&&`, `||` in arguments |
| `data_exfiltration` | rule_guardian | HTTP POST bodies containing user message content |
| `path_traversal` | file_guardian | `../../etc/passwd`-style paths |
| `sensitive_file_access` | file_guardian | `.env`, `.ssh/`, keychain files |
| `network_abuse` | rule_guardian | Unexpected outbound calls |
| `credential_exposure` | rule_guardian | Passing secret values as plain-text arguments |
| `resource_abuse` | rule_guardian | Unbounded loops, mass file operations |
| `prompt_injection` | rule_guardian | Tool parameters containing override instructions |
| `code_execution` | shell_evasion_guardian | Eval patterns, dynamic code construction |
| `privilege_escalation` | rule_guardian | `sudo`, `chmod +s`, service manipulation |

#### Severity model

`CRITICAL > HIGH > MEDIUM > LOW > INFO > SAFE`

`is_safe = true` when no CRITICAL or HIGH findings are present.

#### Guard finding

```text
[REFERENCE]
GuardFinding {
  id, rule_id,
  category: GuardThreatCategory,
  severity: GuardSeverity,
  title, description,
  tool_name,
  param_name?,        // which parameter triggered
  matched_value?,     // truncated value that matched
  matched_pattern?,   // the pattern that fired
  snippet?,           // context around the match
  remediation?,
  guardian            // which guardian produced this finding
}
```

#### Tool guard result

```text
[REFERENCE]
ToolGuardResult {
  tool_name, params,
  findings: GuardFinding[],
  is_safe: bool,
  max_severity: GuardSeverity,
  guard_duration_seconds: f64,
  guardians_used: String[]
}
```

#### Execution level

The `ToolExecutionLevel` setting controls when approval is required:

| Level | Behavior | When to use |
| --- | --- | --- |
| `strict` | All tools require approval before execution | High-security production |
| `smart` | INFO/LOW → auto-allow; MEDIUM+/HIGH/CRITICAL → require approval | **Default recommended** |
| `auto` | Only explicitly listed `guarded_tools` are checked | Legacy / backward compat |
| `off` | Guard completely disabled | Development / fully trusted env only |

The default is `smart`. `off` must be set explicitly; it is never the default for any deployment profile.

#### Hard-blocked patterns

The following are unconditionally denied with no escalation — no user approval can override them at runtime:

- Destructive filesystem commands (`rm -rf /`, `sudo rm -rf`, `mkfs`, `dd if=`)
- Any file path that resolves outside the agent's declared working directory

Hard blocks are logged to the audit trail as `HARD_BLOCKED` entries.

#### Path containment

For any tool call that references file paths: all paths must resolve to within the agent's cwd boundary. Absolute paths are checked after resolution; symlinks are followed before the check. A path that escapes the boundary is a hard block.

#### Suspended permission (approval escalation)

When the guard triggers an approval flow (non-hard-block, severity ≥ threshold), it wraps the call in a `SuspendedPermission` payload:

```text
[REFERENCE]
SuspendedPermission {
  payload: { toolCall, options },
  agent: String,
  tool_name: String,
  tool_kind: String,      // "file" | "shell" | "network" | "other"
  target?: String,        // primary affected resource (path, command, URL)
  action?: String,
  summary?: String,       // human-readable description
  command?: String,       // extracted shell command, if applicable
  paths: String[],        // up to 5 affected paths
  requires_user_confirmation: true
}
```

The approval gate presents this to the user with the proposed options (allow/deny variants). The outcome is recorded in the audit log whether approved or denied.

### 4.3 Interaction with the scheduler

The scheduler's `CronPromptInjectionBlocked` exception (see `l2-scheduler.md §4.6`) is produced by running the skill scanner on the fully-assembled fire-time prompt — this is the runtime application of the skill scanner outside of install time. This catches injection payloads loaded from skill content after job creation.

### 4.4 Audit log entries

Both layers append findings to the workspace audit log (`<ws>/audit.log`):

```text
[REFERENCE]
{ timestamp, layer: "scanner"|"guard", tool_name?, finding_id, category, severity, outcome: "allowed"|"denied"|"escalated"|"hard_blocked" }
```

## 5. Drawbacks & Alternatives

- **False positives:** regex heuristics will block some legitimate patterns. The MEDIUM/LOW/INFO range allows escalation (user decides) rather than hard-blocking. Custom rule overrides can suppress specific rule IDs per workspace.
- **Latency on every tool call:** guard runs inline. Benchmarks must confirm the fast path (no findings) is under 1 ms; guardian failures are logged but don't block the call to avoid guard availability becoming a single point of failure.
- **Evasion via obfuscation:** a determined attacker can encode payloads to evade regex. The scanner's `obfuscation` category catches common encodings; detection is best-effort. Defense-in-depth (sandbox + egress gate) limits the blast radius of any evasion.
- **Alternative — LLM-based scanning:** semantically richer but expensive and slower. LLM scanning can be added as an optional deep-scan pass (`ext scan --deep`); regex is the always-on fast path.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | Invariants this implements |
| `[L2SEC]` | `.design/main/specifications/l2-security.md` | Secret store, egress gate, sandbox |
| `[EXT]` | `.design/main/specifications/l2-extension-registry.md` | Extension activation gate |
| `[SCHED]` | `.design/main/specifications/l2-scheduler.md` | Fire-time prompt injection scan |
