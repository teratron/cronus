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

## Visualization Stubs

The repository's `.release/` directory is a temporary visualization sandbox holding empty stubs of both tiers:

```plaintext
.release/
├── program/   # Stub of the immutable program tier.
└── state/     # Stub of the mutable state tier (example office: workspaces/default).
```

These stubs exist to discuss and visualize the layout; they are not a build artifact and do not ship.
