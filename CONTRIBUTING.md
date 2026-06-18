# Contributing

## Deployment Model

Cronus installs as two clearly separated tiers:

1. **Program tier (immutable).** Compiled executables, scripts, configs, bundled role catalog, templates, languages, and themes. Never written to at runtime; replaced wholesale on update.
2. **State tier (mutable).** Everything the runtime produces and rewrites: configuration, sessions, user-created projects (offices), hired roles, memory, and learning. Survives program updates and reinstalls.

By default the two tiers live in OS-native locations (not side by side). A portable mode placing both under one user-selected directory is also supported.

### OS-native locations

| Tier | Windows | macOS | Linux (XDG) |
| --- | --- | --- | --- |
| Program (immutable) | `%ProgramFiles%\Cronus\` | `/Applications/Cronus.app/Contents/Resources/` | `/opt/cronus/` or `/usr/local/lib/cronus/` |
| State (mutable) | `%APPDATA%\Cronus\` | `~/Library/Application Support/Cronus/` | `~/.local/share/cronus/` |
| Cache (regenerable) | `%LOCALAPPDATA%\Cronus\Cache\` | `~/Library/Caches/Cronus/` | `~/.cache/cronus/` |
| Logs / runtime state | `%LOCALAPPDATA%\Cronus\Logs\` | `~/Library/Logs/Cronus/` | `~/.local/state/cronus/` |

The runtime resolves these paths through a single path resolver. Cache and logs are kept out of the main state tier so the OS can manage them by its own conventions and they can be purged safely.

## Program Tier Layout (immutable)

```plaintext
<program>/
├── bin/            # Launchers and entry points: cronus (CLI), cronus-tui (TUI), cronusd (always-on service).
├── app/            # Compiled runtime: core engine library + desktop application shell.
├── templates/      # Blueprints copied into the state tier on init.
│   ├── employee/   # Skeleton for a hired employee (config, rules, memory, skills, skins).
│   └── workspace/  # Skeleton for an office/project workspace.
├── employees/      # Read-only role CATALOG (hireable specialists). See employees/CATALOG.md.
├── languages/      # Language packs: prompt locales and UI localization.
├── themes/         # UI themes and skins.
└── VERSION         # Program/engine version (update contract).
```

## State Tier Layout (mutable)

```plaintext
<state>/
├── .env                  # Local secrets. NEVER committed or backed up. (.env.example is the template.)
├── app.json              # User preferences (locale, theme).
├── config.json           # Global defaults.
├── auth.json             # Authentication provider registry (references, not raw secrets).
├── channels.json         # Connected communication channels.
├── models.json           # Available AI models and provider bindings.
├── routing.json          # Model-router policy: cost/quality tradeoff, fallback cascade, local-first.
├── gateway.json          # Gateway/always-on service runtime state.
├── AGENTS.md             # Global constitution inherited by every office and employee.
├── memory/               # GLOBAL memory level (about the human client; cross-project facts).
│   ├── global.db         #   SQLite + vector index (created at runtime).
│   ├── graph.db          #   Global knowledge graph (created at runtime).
│   └── notes/            #   Human-readable Markdown memory (editable).
├── skills/               # Globally learned, reusable skills (self-improvement).
├── employees/            # HIRED employees — EMPLOYEE (agent) memory level.
│   └── <role>/
│       ├── config.json   #   Runtime config: model, limits, budget.
│       ├── RULES.md      #   Persona and rules.
│       ├── memory/       #   Long-term role memory (db + notes).
│       ├── skills/       #   Assigned/learned skills.
│       └── skins/        #   Persona/presentation variants.
└── workspaces/           # OFFICES — one per project (isolated). WORKSPACE memory level.
    └── <ws>/
        ├── config.json   #   Office identity, manager, team, policies.
        ├── RULES.md      #   Office rules.
        ├── STATE.md      #   Live office state (read first on resume).
        ├── memory/       #   Shared office memory (db + notes).
        ├── graph/        #   Office knowledge graph: people, tasks, decisions, artifacts.
        ├── sessions/     #   SESSION (episodic) memory level — decays/prunes over time.
        ├── kanban/       #   Board: triage -> todo -> ready -> running -> blocked -> done -> archive.
        ├── office/       #   Spatial office model (agents mapped to tasks).
        ├── schedules/    #   Cron and heartbeat routines.
        ├── hooks/        #   Event automation.
        ├── sandboxes/    #   Isolated execution areas.
        ├── snapshots/    #   Restorable states and milestones.
        └── dashboard/    #   Dashboard state.
```

## Memory Levels

Memory is organized into four scopes; the router reads from most specific to least specific and writes to the scope a fact belongs to:

| Level | Location | Holds | Lifecycle |
| --- | --- | --- | --- |
| Global | `<state>/memory/` | Facts about the human client, cross-project preferences, shared skills | Long-lived; rarely pruned |
| Workspace | `<state>/workspaces/<ws>/memory/` + `graph/` | Office knowledge, project graph | Lives with the office |
| Employee | `<state>/employees/<role>/memory/` | Role expertise | Grows with the role |
| Session | `<state>/workspaces/<ws>/sessions/` | Dialogue and decision episodes | Decays by scope; auto-pruned |

Read order at retrieval: Employee → Workspace → Global (most specific wins). Deleting an office or a role removes its memory with it; only the global level outlives them.

## Commands

One command surface is exposed across the CLI and the TUI (and a matching library API). Grammar follows mainstream-CLI conventions: a top-level command `cronus <command>`, or a command group `cronus <group> <verb> [args] [--flags]`. The TUI mirrors each with a leading slash. This table is kept in sync as functionality grows.

### Core

| CLI | TUI/UI | Description |
| --- | --- | --- |
| `cronus help` | `/help` | Show usage and discover commands |
| `cronus init` | `/init` | Initialize Cronus (first run) or a project in the current location |
| `cronus idea` | `/idea` | Capture an idea or requirement for the office |
| `cronus plan` | `/plan` | Generate a plan from captured intent |
| `cronus task` | `/task` | Generate or list tasks |
| `cronus run` | `/run` | Execute queued work |
| `cronus status` | `/status` | Show current position, progress, and blockers |
| `cronus compact` | `/compact` | Compact the working context/state |
| `cronus analyze` | `/analyze` | Analyze the project/workspace |
| `cronus check [<gate>]` | `/check [<gate>]` | Run quality gates (tests, lint, types, format, bench, security) |
| `cronus memory` | `/memory` | Query and manage memory |
| `cronus goal` | `/goal` | Start an autonomous goal loop that runs until the goal is met |
| `cronus quit` | `/quit` | End the current session |
| `cronus exit` | `/exit` | Exit the application |

### Workspace

| CLI | TUI/UI | Description |
| --- | --- | --- |
| `cronus workspace list` | `/workspace list` | List workspaces |
| `cronus workspace create <name> [-d <desc>] [-p <path>]` | `/workspace create …` | Create a project office |
| `cronus workspace open <id>` | `/workspace open <id>` | Open / switch to a workspace |
| `cronus workspace info <id>` | `/workspace info <id>` | Show workspace details |
| `cronus workspace set <id> [--name <v>] [--description <v>] [--path <v>]` | `/workspace set <id> …` | Edit workspace metadata |
| `cronus workspace close <id>` | `/workspace close <id>` | Hide a workspace tab (state kept) |
| `cronus workspace delete <id>` | `/workspace delete <id>` | Delete a project workspace (home is protected) |
| `cronus workspace home` | `/workspace home` | Switch to the home workspace |

### Board

| CLI | TUI/UI | Description |
| --- | --- | --- |
| `cronus board show` | `/board show` | Show the office board |
| `cronus board list [--state <s>]` | `/board list …` | List cards, optionally by state |
| `cronus board move <card-id> <state>` | `/board move <card-id> <state>` | Move a card to a state |
| `cronus board block <card-id> --reason <r>` | `/board block …` | Block a card with a reason |
| `cronus board unblock <card-id>` | `/board unblock <card-id>` | Unblock a card |
| `cronus board archive <card-id>` | `/board archive <card-id>` | Archive a card (manual override; done cards auto-archive) |

### Schedule

| CLI | TUI/UI | Description |
| --- | --- | --- |
| `cronus schedule create <name> --action <a> --when <preset> [--time HH:MM] [--days mon,tue] [--every 15m]` | `/schedule create …` | Create a recurring schedule (friendly) |
| `cronus schedule create <name> --action <a> --once --at <datetime>` | `/schedule create … --once` | Create a one-shot schedule (auto-deletes after firing) |
| `cronus schedule create <name> --action <a> --cron "<expr>"` | `/schedule create … --cron …` | Create a schedule from a raw cron expression (advanced) |
| `cronus schedule list` | `/schedule list` | List schedules |
| `cronus schedule show <id>` | `/schedule show <id>` | Show a schedule |
| `cronus schedule set <id> [--when …] [--cron …] [--enabled true\|false]` | `/schedule set <id> …` | Edit a schedule |
| `cronus schedule enable <id>` | `/schedule enable <id>` | Enable a schedule |
| `cronus schedule disable <id>` | `/schedule disable <id>` | Disable a schedule (kept, not deleted) |
| `cronus schedule delete <id>` | `/schedule delete <id>` | Delete a schedule |
| `cronus schedule run <id>` | `/schedule run <id>` | Fire a schedule now |

> Schedule fire actions: `heartbeat` (wake the office, no card), `routine` (recurring work), `reminder` (notification).

## Visualization Stubs

The repository's `.release/` directory is a temporary visualization sandbox holding empty stubs of both tiers:

```plaintext
.release/
├── program/   # Stub of the immutable program tier.
└── state/     # Stub of the mutable state tier (example office: workspaces/default).
```

These stubs exist to discuss and visualize the layout; they are not a build artifact and do not ship.
