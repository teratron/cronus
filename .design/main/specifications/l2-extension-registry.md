# Extension Registry

**Version:** 1.0.6
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

### 4.5 Plugin manifest extended schema

Plugins that expose UI-configurable tools use an extended manifest format. The `meta.tools[*].config_fields` array declares the fields the desktop UI renders as a configuration form; values the user fills in are stored via the secret store (for `"password"` type) or the workspace config (for `"text"` and `"number"`).

```text
[REFERENCE]
plugin.json {
  id, name, version,
  type: "tool" | "bundle",
  description: String,
  description_i18n?: { "en-US": String, "zh-CN": String, ... },
  author: String,
  entry: { backend: "<filename>" },
  dependencies?: ["pkg>=version"],
  min_version?: String,           // minimum Cronus version required
  commands?:    String,          // commands directory; default: "./commands/"
  agents?:      String,          // agents directory; default: "./agents/"
  hooks?:       String | Object, // path to hooks.json or inline config; default: "./hooks/hooks.json"
  mcpServers?:  String | Object, // path to .mcp.json or inline MCP server config (see §4.12)
  meta?: {
    tools: [
      {
        name: String,             // tool function name
        description: String,
        icon?: String,            // emoji or icon identifier
        requires_config?: bool,   // true → block use until config_fields are filled
        config_fields?: [
          {
            name: String,         // identifier used in the plugin code
            label: String,        // displayed in the UI form
            type: "text" | "password" | "number",
            required: bool,
            placeholder?: String,
            help?: String,
            min?: number,         // for type: "number"
            max?: number
          }
        ]
      }
    ],
    api_key_url?: String,         // where to obtain credentials
    api_key_hint?: String
  }
}
```

**Storage:** `password` fields are stored in the OS keychain (same mechanism as `l2-security.md §4.1`); `text` and `number` fields are stored in `<ws>/extensions/plugins/<id>/config.json`. Neither type is written to tracked files.

**Localization:** `description_i18n` provides locale-keyed descriptions; the UI falls back to `description` when the current locale is absent.

**Compatibility:** `min_version` is checked at install time; incompatible plugins are blocked from activation with a clear version error.

### 4.6 Component auto-discovery

When a plugin activates, the registry discovers its components in two passes.

**Default directories** (always scanned; relative to `<plugin-root>/`):

```
<plugin-root>/
├── commands/          # .md files → slash commands
├── agents/            # .md files → agent definitions
├── skills/            # subdirectories each containing SKILL.md → skill triggers
└── hooks/
    └── hooks.json     # tool-event hook configuration (§4.10 in l2-plugin-hooks.md)
```

Subdirectory nesting creates namespaces: `commands/ci/lint.md` → command `lint` in namespace `ci`, listed as `/lint (plugin:<name>:ci)`.

**Manifest path overrides**: `plugin.json` `commands`/`agents`/`hooks`/`mcpServers` fields (§4.5) specify alternate paths. These are merged with default discoveries; an entry found in a default location is never overridden by a manifest path.

**Path rules**: all paths are relative, must start with `./`, must not contain `../`, use forward slashes. Absolute paths and traversal are rejected at activation.

#### `${PLUGIN_ROOT}` path variable

`${PLUGIN_ROOT}` expands to the plugin's absolute on-disk directory at session start. Use it in:

- Hook `command` fields: `"bash ${PLUGIN_ROOT}/scripts/validate.sh"`
- MCP server `command` and `args` fields
- Environment variable values in MCP server `env` blocks

Never use hardcoded absolute paths; `${PLUGIN_ROOT}` is the single portable reference across installs, platforms, and users.

### 4.7 Remote skill catalog

Skills may be published to and installed from a remote catalog. Each catalog entry carries trust metadata so the registry can enforce sandboxing proportional to the skill's capabilities.

#### CatalogSkill record

```text
[REFERENCE]
CatalogSkill {
  id,
  name,
  version,
  description,
  trustLevel: "markdown_only" | "assets" | "scripts_executables",
  compatibility: {
    minCronusVersion: String,
    platforms: ("windows" | "macos" | "linux")[],
    architectures?: ("x86_64" | "aarch64")[]
  },
  defaultInstall: bool,           // pre-installed for all workspaces
  recommendedForRoles: String[],  // preset role ids for which this skill is suggested
  contentHash: String,            // SHA-256 of the downloaded archive — verified before activation
  source: {
    kind: "github",
    hostname: String,             // default: "github.com"
    owner: String,
    repo: String,
    ref: String,                  // branch, tag, or "latest"
    commit: String                // pinned commit SHA — immutable reference
  }
}
```

#### Trust levels

| Level | What it contains | Sandbox behavior |
| --- | --- | --- |
| `markdown_only` | Pure text instructions, no executable content. | No sandbox needed; loaded as instruction text only. |
| `assets` | Instruction text + static data files (images, templates). | Sandboxed file access to the assets path; no code execution. |
| `scripts_executables` | Any executable code (scripts, binaries, WASM). | Full execution sandbox; same constraints as plugins (EXT-4). |

`markdown_only` skills are the safest — they carry no attack surface beyond prompt injection (caught by the skill scanner at §4.4). `scripts_executables` skills receive the most restrictive sandbox; they require an explicit user grant, not just an activation.

#### Catalog operations

The catalog is a read-only remote index. Local installs are a pull from the catalog followed by content hash verification.

Install flow:

1. Resolve `source.ref` to a commit SHA (immutable pin recorded in `source.commit`).
2. Download archive.
3. Verify `sha256(archive) == contentHash` — reject if mismatch.
4. Run skill scanner (static analysis, `trustLevel`-appropriate checks).
5. Register as `discovered`; activate only after explicit grant.

```text
[REFERENCE]
cronus skill catalog list [--role <role-id>] [--trust <level>]
cronus skill catalog install <catalog-id>
cronus skill catalog update <installed-id>
```

Remote catalog configuration (stored in workspace config, not tracked files):

```text
[REFERENCE]
CatalogConfig {
  url: String,           // catalog index endpoint
  signingKey?: String,   // public key to verify catalog index signature
  autoUpdate: bool       // default: false — catalog checks are always user-triggered
}
```

`autoUpdate: false` is the default; operators who want automatic catalog refreshes must opt in explicitly. Updates never auto-activate new skill versions — they go through the same `discovered → permitted → active` lifecycle.

### 4.8 Command surface

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

### 4.9 Skill definition format (SKILL.md)

A skill is a directory in the plugin's `skills/` subtree containing a `SKILL.md` file. The YAML frontmatter defines the trigger; the body is instruction content injected into the agent's context when the trigger fires.

```text
[REFERENCE]
SKILL.md YAML frontmatter:
  name:            String    // kebab-case skill identifier
  description:     String    // trigger description — concrete phrases, tasks, or file patterns
                             // that should activate this skill; include negative examples
                             // (when NOT to activate) to sharpen precision
  version:         String    // semver: "1.0.0"
  trigger:         String[]  // 5–8 natural-language phrases a user might say to invoke the skill;
                             // exhaustive is better — users won't remember skill names
  integrations:    String[]  // named integration slots the skill requires to run, e.g.:
                             //   "vault-search-fts5", "web-search-ddgr", "google-calendar-read"
  inputs:          Input[]   // typed input declarations the UI/CLI can render as a form
  output_artifact: String?   // relative path of the document the skill writes, e.g. "wiki/[slug].md"
  frequency:       Frequency // execution cadence (see below)
  pack:            String?   // optional function-pack grouping label, e.g. "ceo" | "engineering"
                             // used by the UI to group skills into pack browsers

Input {
  name:        String
  description: String
  required:    bool   // default false
}

Frequency: "on-demand"            // user-triggered only
         | "cron:<CRON_EXPR>"     // scheduled via the workspace cron system; e.g. "cron:0 8 * * *"
         | "on-demand-or-cron"    // supports both paths; scheduled-run context available via env
```

#### Skill body structure

Content-generating skills (blog posts, reports, research briefs, any output a human will read or publish) **must** embed voice and quality enforcement in the body's Steps section as numbered rules — not merely as style guidance. Voice rules that aren't enforced at the step level will be ignored by the agent.

The recommended body has five H2 sections:

```text
## When to run       – practitioner situation that earns the skill its keep
## What you'll get   – artifact shape + leverage ("what becomes possible")
## Steps             – numbered, ordered instructions; reference integrations explicitly
## Output format     – fenced example of the expected output document frontmatter + structure
## Example output    – 15–30 line realistic sample
```

#### Directory layout

```
skills/
└── my-skill/
    ├── SKILL.md          # frontmatter + instruction body
    └── references/       # optional reference files cited in SKILL.md
```

### 4.9.1 Skill pack override (shipped vs. user-owned)

Shipped skills are read-only — they live in the system extensions directory and are replaced on upgrade. Users can override any shipped skill by placing an edited copy in the workspace skill scope (`<ws>/skills/<pack>/<name>.md`) or the user-scope skill directory (`<state>/extensions/skills/<pack>/<name>/SKILL.md`).

```text
[REFERENCE]
Resolution order (first match wins):
  1. workspace skills     <ws>/skills/<pack>/<name>.md           (highest precedence)
  2. user-scope skills    <state>/extensions/skills/<pack>/<name>/SKILL.md
  3. shipped skills       <program>/extensions/skills/<pack>/<name>/SKILL.md

Override contract:
  - Updates to shipped skills NEVER overwrite user-owned copies.
  - Conflict is detected by name — if <pack>/<name> matches, user copy wins.
  - On activation, the registry records source: "shipped" | "workspace" | "user"
    for provenance tracking and doctor diagnostics.
  - A user override that is semantically identical to the shipped version produces a warning
    ("override is identical to shipped — consider deleting"); never an error.
```

This lets users customize any skill for their context without touching the ship directory. A `skill status` command shows whether each active skill is shipped or overridden.

### 4.10 Agent definition format

Agent definitions in the plugin's `agents/` directory are `.md` files. The YAML frontmatter declares identity and tool access; the body becomes the system prompt.

```text
[REFERENCE]
agents/<name>.md frontmatter:
  name:        String    // kebab-case, 3–50 chars, alphanumeric + hyphens,
                         //   starts and ends with alphanumeric
  description: String
    // Triggering conditions + 2–4 usage examples:
    // "Use this agent when <conditions>. Examples:
    //  <example>
    //  Context: <scenario>
    //  user: \"<user request>\"
    //  assistant: \"<how to respond and launch this agent>\"
    //  <commentary>why this agent fires</commentary>
    //  </example>"
    // Be specific about when NOT to use the agent.
  model:       inherit | sonnet | opus | haiku   // default: inherit
  color:       blue | cyan | green | yellow | magenta | red
  tools?:      String[]   // restrict to minimum required tools;
                          //   omit = all tools available
                          //   read-only:       ["Read","Grep","Glob"]
                          //   code generation: ["Read","Write","Grep"]
```

System prompt conventions (body text):

- Second person: "You are …", "You will …"
- Structure: Core Responsibilities → Analysis Process → Quality Standards → Output Format → Edge Cases
- Keep under 3 000 characters

### 4.11 Command definition format

Slash commands in the plugin's `commands/` directory are `.md` files. The body is an instruction **to the agent** (not a message to the user).

```text
[REFERENCE]
commands/<name>.md YAML frontmatter:
  description?:               String   // shown in /help; ≤ 60 chars
  allowed-tools?:             Array    // e.g. ["Read", "Bash(git:*)"]
  model?:                     inherit | sonnet | opus | haiku
  argument-hint?:             String   // e.g. "[pr-number] [priority]"
  disable-model-invocation?:  bool     // default: false

Dynamic tokens in the body:
  $ARGUMENTS           // all user-provided arguments as a single string
  $1, $2, …           // individual positional arguments
  @<path>              // include file contents inline at that point
  !`<bash command>`    // execute bash and inline the stdout (dynamic context)
  ${PLUGIN_ROOT}       // plugin-root-relative path for scripts and templates
```

Naming: verb-noun (`review-pr`, `generate-docs`); one responsibility per command; complex multi-step flows delegate to an agent definition (§4.10).

### 4.12 MCP server transport variants

The `connect` configuration in `.mcp.json` or the `mcpServers` manifest field accepts four transport variants:

```text
[REFERENCE]
// stdio — local process; communicates via stdin/stdout
{ "command": "${PLUGIN_ROOT}/servers/my-server",
  "args": ["--config", "${PLUGIN_ROOT}/config.json"],
  "env":  { "DB_URL": "${DB_URL}" } }

// sse — hosted endpoint with OAuth (auth flow handled by Cronus)
{ "type": "sse", "url": "https://mcp.example.com/sse" }

// http — REST endpoint with token-based auth
{ "type": "http", "url": "https://api.example.com/mcp",
  "headers": { "Authorization": "Bearer ${API_TOKEN}" } }

// ws — WebSocket endpoint for real-time / streaming
{ "type": "ws", "url": "wss://mcp.example.com/ws",
  "headers": { "Authorization": "Bearer ${TOKEN}" } }
```

**Tool naming**: registered MCP tools are prefixed as `mcp__<plugin-name>_<server-name>__<tool-name>`. Commands pre-allow specific tools via `allowed-tools`; prefer explicit names over wildcard `__*` grants.

**Security**: use HTTPS/WSS only; reference credentials via environment variables (`${MY_API_KEY}`); document required env vars in the plugin README.

### 4.13 Channel plugin kind

A **channel plugin** connects an external messaging platform (Telegram, Slack, DingTalk,
WeChat, or any third-party IM service) to a Cronus agent session. Channel plugins are a
distinct extension kind (`kind: "channel"`) in the unified registry alongside skills,
MCP servers, and plugins.

#### Channel interface

Every channel plugin implements three lifecycle methods:

```text
[REFERENCE]
ChannelPlugin {
  channel_type:           String,        // unique kind identifier, e.g. "telegram"
  display_name:           String,
  required_config_fields: Vec<String>,   // field names the user must provide (e.g. "token")
  create_channel(config: ChannelConfig) -> ChannelInstance
}

ChannelInstance methods:
  connect()                              // connect to platform; register message handlers
  send_message(chat_id: &str, text: &str) // format and deliver agent response
  disconnect()                            // clean up on shutdown

On receiving an inbound message, the plugin builds an Envelope and calls
handle_inbound(envelope). The base runtime handles routing, access control, session
management, slash commands, prompt serialization, and crash recovery — the plugin is
responsible only for the platform API translation.
```

#### Envelope (normalized inbound format)

All platforms convert inbound messages to a common Envelope before routing:

```text
[REFERENCE]
Envelope {
  // Identity
  sender_id:    String,          // stable unique identifier for this sender on this platform
  sender_name:  String,          // display name (may be empty)
  chat_id:      String,          // DM or group identifier; must distinguish DMs from groups
  channel_name: String,          // matches ChannelPlugin.channel_type

  // Content
  text:             String,      // message text; @mentions stripped by the plugin
  image_base64:     Option<String>,
  image_mime_type:  Option<String>,
  referenced_text:  Option<String>,   // quoted/replied-to text

  // Context flags
  is_group:       bool,
  is_mentioned:   bool,   // bot was @mentioned in a group; used by GroupGate
  is_reply_to_bot: bool,
  thread_id:      Option<String>,
}

Plugin contract:
  sender_id must be stable across messages (same user → same id).
  chat_id must distinguish DM channels from group channels.
  Boolean flags must be accurate — they directly gate access-control decisions.
  @mentions must be stripped from text before building the Envelope.
```

#### Session routing

One Cronus process serves multiple simultaneous users; each (channel, routing-key)
pair maps to an isolated agent session:

```text
[REFERENCE]
SessionScope: "user" | "thread" | "single"
  "user"   — one session per sender_id (default); conversations are fully isolated
  "thread" — one session per thread_id (or chat_id when thread_id is absent)
  "single" — all senders on this channel share one session (broadcast / group-bot use)

Routing key: "<channel_name>:<scope_key>"
  user   → scope_key = sender_id
  thread → scope_key = thread_id ?? chat_id
  single → scope_key = channel_name (constant)

Per-routing-key message chains are serialized (one promise chain per key) to prevent
concurrent prompt collisions on the same session.
```

#### Access control

Two gate layers are applied in order before a message reaches the agent:

```text
[REFERENCE]
SenderGate: "allowlist" | "pairing" | "open"
  "allowlist" — only sender_ids listed in allowed_users are forwarded; others dropped silently
  "pairing"   — new users must complete a pairing handshake (/pair <code>); codes are
                generated via CLI and approved by an operator
  "open"      — any sender may interact (suitable only for private/internal deployments)

GroupGate: "disabled" | "allowlist" | "open"
  "disabled"  — group messages are always dropped (default)
  "allowlist" — only group chat_ids listed in the groups config are accepted
  "open"      — any group chat may interact

GroupConfig {
  require_mention: bool,   // when true, bot must be @mentioned; bare group messages are ignored
}
```

#### Channel manifest

Channel plugins are packaged as extensions. The manifest (`extension.json`) declares
each channel type under a `"channels"` key:

```text
[REFERENCE]
extension.json {
  "name":    "my-channel-extension",
  "version": "1.0.0",
  "channels": {
    "my-platform": {
      "entry":        "dist/index.js",
      "display_name": "My Platform"
    }
  }
}
```

Channel instances are configured in the workspace settings (not tracked files):

```text
[REFERENCE]
settings.json "channels" block:
{
  "channels": {
    "<instance-name>": {
      "type":          "<channel_type>",        // matches ChannelPlugin.channel_type
      "token":         "$MY_BOT_TOKEN",         // env var reference — never hardcoded
      "sender_policy": "allowlist",             // SenderGate policy
      "allowed_users": ["user123"],
      "session_scope": "user",                  // SessionScope
      "cwd":           "/path/to/project",      // working directory for agent sessions
      "model":         "...",                   // optional model override for this channel
      "instructions":  "...",                   // per-channel system prompt supplement
      "group_policy":  "disabled",              // GroupGate policy
      "groups":        { "*": { "require_mention": true } }
    }
  }
}

Auth is plugin-specific. Credentials (tokens, app secrets) are stored via the secret
store (l2-security.md §4.1) — never written to tracked configuration files.
```

#### Built-in slash commands

The base channel runtime intercepts these slash commands before forwarding to the agent:

```text
[REFERENCE]
/clear  — end the current session; the next message starts a fresh session
/help   — show channel-specific help and available commands
/status — show session status, uptime, and current model
```

Third-party channel plugins may register additional slash commands via `register_command()`.

#### Channel CLI surface

| Action | CLI | TUI |
| start | `cronus channel start [name]` | `/channel start [name]` |
| stop | `cronus channel stop` | `/channel stop` |
| status | `cronus channel status` | `/channel status` |
| list pending pairings | `cronus channel pairing list <channel>` | — |
| approve pairing | `cronus channel pairing approve <channel> <code>` | — |

### 4.14 Installer target interface

[ADDED] The multi-agent installer exposes a uniform interface for wiring Cronus (or any
MCP-capable extension) into the configuration of every supported AI agent host. Adding
support for a new agent host requires exactly one new file implementing this interface and
one registry entry.

#### AgentTarget interface

```text
[REFERENCE]
AgentTarget {
  id:          TargetId,       // stable kebab-case identifier ("claude", "cursor", …)
  display_name: String,        // human-readable name for interactive prompts
  docs_url:    Option<String>, // where users can learn more about this agent

  supports_location(loc: Location) -> bool
    // Location: "global" | "local"
    // Some agents have no per-project config (only a global ~/.agent/ dir).
    // Return false for unsupported (target, location) pairs — installer skips cleanly.

  detect(loc: Location) -> DetectionResult
    // Best-effort heuristic: is this agent installed? Has Cronus already been wired?
    // False positives acceptable; false negatives require manual opt-in.

  install(loc: Location, opts: InstallOptions) -> WriteResult
    // Write the MCP-server config for this agent at the given location.
    // Surgical edit — never touch sibling MCP servers, sibling permissions, or
    // unrelated config sections. Idempotent: a byte-identical re-run returns `unchanged`.

  uninstall(loc: Location) -> WriteResult
    // Inverse of install. Removes only what install would have written.
    // Safe to call when nothing was ever installed (returns `not-found` actions).

  print_config(loc: Location) -> String
    // Print the MCP-server snippet a user would paste manually.
    // MUST NOT touch the filesystem — pure display function.

  describe_paths(loc: Location) -> Vec<String>
    // Filesystem paths this target would write to at this location.
    // Used for dry-run output and uninstall confirmation.
}

Location:  "global" | "local"
InstallOptions { auto_allow: bool }  // whether to write agent permission grants (if any)
```

#### DetectionResult and WriteResult

```text
[REFERENCE]
DetectionResult {
  installed:          bool,           // agent CLI / config dir detected on this machine
  already_configured: bool,           // Cronus MCP already wired at this location
  config_path:        Option<String>, // path inspected; shown in diagnostic / dry-run
}

WriteResult {
  files: Vec<{
    path:   String,
    action: FileAction,
  }>,
  notes: Vec<String>,   // optional one-line notes surfaced verbatim; keep short
}

FileAction:
  "created"   — new file written
  "updated"   — existing file modified
  "unchanged" — file touched but byte-identical; idempotent re-run
  "removed"   — file deleted by uninstall
  "not-found" — uninstall called on a file that was never written (safe no-op)
  "kept"      — file present but NOT touched (e.g. uninstall preserving sibling config)

`unchanged` is the critical state for installer stability: a re-run that writes the same
content must return `unchanged`, not `updated`, so callers can detect genuine changes.
```

#### Registry pattern

```text
[REFERENCE]
ALL_TARGETS: readonly Vec<AgentTarget>   // ordered; order is prompt order and --target=all order

get_target(id: &str) -> Option<&AgentTarget>
detect_all(loc: Location) -> Vec<(AgentTarget, DetectionResult)>

resolve_target_flag(value: &str, loc: Location) -> Vec<AgentTarget>:
  "auto" → detect_all, return those with installed=true; fallback to first target if none
  "all"  → ALL_TARGETS
  "none" → []
  csv    → split on ',', look up each id; error on unknown ids with the known list

CLI --target flag grammar:
  --target=auto       (default when flag omitted)
  --target=all
  --target=none
  --target=claude,cursor
```

#### Surgical config editing

Installer writes MUST use surgical JSON/TOML/JSONC editing rather than full rewrites.
This preserves user comments, formatting, sibling MCP servers, and unrelated config blocks
when Cronus is added or removed.

For JSONC: parse with a comment-preserving parser; locate or insert the
`mcpServers.cronus` key; write only that subtree. Adjacent keys and comments are untouched.

For TOML: a hand-rolled serializer scoped to `[mcp_servers.cronus]`; sibling `[section]`
blocks and `[[array]]` entries are preserved verbatim.

**Idempotency invariant**: install then uninstall then install must return the config file
to the same byte-exact state as after the first install.

#### Adding a new target

One new file `targets/<id>.ts` implementing `AgentTarget` + one entry in `ALL_TARGETS`.
No changes to the installer orchestrator or the MCP server itself.

## 5. Drawbacks & Alternatives

- **Sandbox overhead per MCP process:** real cost; justified — MCP servers are untrusted external code.
- **Manifest drift vs reality:** a manifest may under-declare what an extension does; mitigated by sandbox enforcement (grants bind regardless of manifest claims).
- **Alternative — install extensions globally unsandboxed:** rejected (EXT-3/4).

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[MODEL]` | `.design/main/specifications/l1-extensions.md` | Invariants this registry satisfies |
| `[SECURITY]` | `.design/main/specifications/l2-security.md` | Sandbox + egress enforcement |
| `[TOOLSEC]` | `.design/main/specifications/l2-tool-security.md` | Skill scanner runs at activation gate |
| `[CLI]` | `.design/main/specifications/l2-cli.md` | Command grammar standard |
