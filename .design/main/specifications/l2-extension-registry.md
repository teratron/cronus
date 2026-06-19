# Extension Registry

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-extensions.md

## Overview

The concrete extension system: the manifest format, where extensions live (program presets vs state custom), how each kind connects (skills, MCP servers as a client, sandboxed plugins), how permissions and the sandbox are enforced, the skill-generation pipeline, and the command surface.

## Related Specifications

- [l1-extensions.md](l1-extensions.md) - The model this registry implements.
- [l2-security.md](l2-security.md) - Sandbox backend, egress gate, secret grants.
- [l2-filesystem-layout.md](l2-filesystem-layout.md) - Preset (program) and custom (state) locations.
- [l2-workflow-runtime.md](l2-workflow-runtime.md) - Skills authored as workflows run here.
- [l2-cli.md](l2-cli.md) - Command grammar standard.

## 1. Motivation

The model needs a concrete registry with a uniform manifest so all three kinds connect through one permissioned, sandboxed path, and a place for the office to deposit generated skills.

## 2. Constraints & Assumptions

- Manifests are validated before activation; unknown/over-broad permission requests are surfaced at the grant gate.
- MCP is consumed as a client (stdio or remote); Cronus may also expose its own capabilities as an MCP server (optional).
- Plugins run in the security sandbox; network access flows through the egress gate.

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| EXT-1 Unified model | One registry indexes all kinds; entries differ only by `kind` + interface adapter. |
| EXT-2 Lifecycle | Registry states: discovered → permitted → active → inactive; use requires `active`. |
| EXT-3 Default-deny | New/external extensions register as `discovered`; activation requires an explicit grant. |
| EXT-4 Sandboxed | Plugins and MCP processes run via the security sandbox; tools cannot exceed grants. |
| EXT-5 Preset + custom | Presets under `<program>/extensions/`; custom/generated under `<state>/extensions/`. |
| EXT-6 Scoped grants | Manifest `permissions` (fs/network/secrets) are minimal; network passes the egress gate. |
| EXT-7 Skill generation | The curator distills patterns into candidate skills written to `<state>/extensions/skills/`. |
| EXT-8 Provenance & audit | Each entry records `source`; activations and tool calls append to the audit log. |
| EXT-9 Manifest contract | `extension.json` validated against a schema before activation. |

## 4. Detailed Design

### 4.1 Manifest (conceptual)

```text
[REFERENCE]
{
  id, kind,                 // "skill" | "mcp-server" | "plugin"
  name, version,
  source,                   // "preset" | "custom" | "generated"
  capabilities: [...],
  permissions: {            // explicit + minimal (EXT-6)
    fs?: ["<scoped paths>"],
    network?: ["<allowed hosts>"],
    secrets?: ["<secret keys>"]
  },
  entry?,                   // plugin code entry
  connect?                  // mcp: { transport: "stdio"|"http", command|url }
}
```

### 4.2 Locations

```plaintext
<program>/extensions/        # read-only preset extensions (skills, bundled MCP configs)
<state>/extensions/
├── skills/                  # custom + generated skills
├── mcp/                     # connected MCP server configs
└── plugins/                 # installed plugins
```

Skills also attach to roles (`<state>/employees/<role>/skills/`) and offices (`<ws>/skills/`); the registry indexes all scopes.

### 4.3 Connecting each kind

- **skill:** load instructions (and an optional workflow run by the workflow runtime); no separate process.
- **mcp-server:** start/connect via stdio or remote URL as a client; import its declared tools after the grant; the process runs under the sandbox and the egress gate.
- **plugin:** load the code entry into the sandbox with its scoped grants.

### 4.4 Skill generation

The curator/archivist role detects recurring successful patterns and distills a candidate `SKILL` into `<state>/extensions/skills/`, marked `source: generated`, status `discovered` (inactive) pending review. <!-- TBD: distillation trigger threshold + whether generated skills auto-activate after N successful reuses -->

### 4.5 Command surface

Conforms to the CLI grammar standard (see `l2-cli.md` §4.4).

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| list | `cronus ext list [--kind skill\|mcp\|plugin]` | `/ext list …` | `ext.list({kind?}) -> Extension[]` |
| add | `cronus ext add <source>` | `/ext add <source>` | `ext.add(source) -> Extension` (prompts grant) |
| info | `cronus ext info <id>` | `/ext info <id>` | `ext.get(id) -> Extension` |
| enable | `cronus ext enable <id>` | `/ext enable <id>` | `ext.enable(id) -> void` |
| disable | `cronus ext disable <id>` | `/ext disable <id>` | `ext.disable(id) -> void` |
| remove | `cronus ext remove <id>` | `/ext remove <id>` | `ext.remove(id) -> void` |
| add MCP server | `cronus mcp add <name> (--stdio "<cmd>" \| --url <url>)` | `/mcp add …` | `ext.addMcp(name, connect) -> Extension` |
| generate skill | `cronus skill generate` | `/skill generate` | `ext.generateSkills() -> Extension[]` |

`mcp`/`skill` are convenience sub-commands over the same registry (`ext`).

## 5. Drawbacks & Alternatives

- **Sandbox overhead per MCP process:** real cost; justified — MCP servers are untrusted external code.
- **Manifest drift vs reality:** a manifest may under-declare what an extension does; mitigated by sandbox enforcement (grants bind regardless of manifest claims).
- **Alternative — install extensions globally unsandboxed:** rejected (EXT-3/4).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[MODEL]` | `.design/main/specifications/l1-extensions.md` | Invariants this registry satisfies |
| `[SECURITY]` | `.design/main/specifications/l2-security.md` | Sandbox + egress enforcement |
| `[CLI]` | `.design/main/specifications/l2-cli.md` | Command grammar standard |
