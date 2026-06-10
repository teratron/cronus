# Contributing

## Repository Layout

```plaintext
release/
├── corporation/       # Portable execution distribution; can live in any user-selected location.
│   ├── app/           # Runtime application code and core services.
│   ├── bin/           # Launchers, CLI entry points, and service wrappers.
│   ├── languages/     # Language packs, prompt locales, and UI localization assets.
│   └── templates/     # Blueprints used to create mutable user state.
│       ├── employee/  # Initial structure for a hired office employee.
│       └── workspace/ # Initial structure for a corporation or project workspace.
└── .corporation/      # Mutable user repository produced and maintained by the runtime.
```

## User Repository Layout

```plaintext
/home/<user>/.corporation/       # User-owned corporation state and configuration.
├── .env                         # Local environment variables and machine-specific secrets.
├── AGENTS.md                    # Global agent rules inherited by all employees and workspaces.
├── app.json                     # User-facing application preferences, including locale.
├── auth.json                    # Authentication provider registry and credential references.
├── channels.json                # Communication channels connected to the corporation.
├── config.json                  # Corporation-level settings and defaults.
├── gateway.json                 # Gateway process state and external platform health.
├── models.json                  # Available AI models and provider bindings.
├── employees/                   # Global pool of hireable office roles.
│   ├── architect/               # Employee profile for architecture decisions.
│   │   ├── memories/            # Long-term employee memory and reusable context.
│   │   ├── skills/              # Employee-specific capabilities and procedures.
│   │   ├── skins/               # Presentation, persona, and interface variants.
│   │   ├── config.json          # Employee runtime configuration.
│   │   └── RULES.md             # Employee-specific operating rules.
│   └── code-reviewer/           # Employee profile for review and quality control.
└── workspaces/                  # Offices where corporation-level and project work happens.
    ├── default/                 # Shared corporation office with the orchestrator or general AI chat.
    │   ├── cache/               # Generated media, graph artifacts, and reusable computed state.
    │   ├── schedules/           # Scheduled routines and recurring office processes.
    │   ├── dashboard/           # Workspace dashboard state and visual summaries.
    │   ├── graph/               # Knowledge graph for people, tasks, decisions, and artifacts.
    │   ├── hooks/               # Workspace event hooks and automation entry points.
    │   ├── kanban/              # Board state for office and project execution.
    │   ├── logs/                # Runtime logs, audit traces, and conversation diagnostics.
    │   ├── memories/            # Workspace-level memory shared by the office.
    │   ├── office/              # Visual office model, rooms, seats, and employee placement.
    │   ├── sandboxes/           # Isolated execution areas for experiments and generated work.
    │   ├── sessions/            # Chat sessions, decision threads, and interaction history.
    │   ├── skills/              # Workspace-specific capabilities and reusable workflows.
    │   ├── snapshots/           # Restorable workspace states and milestone captures.
    │   ├── config.json          # Workspace configuration, team composition, and policies.
    │   └── RULES.md             # Workspace-specific instructions and completion rules.
    ├── project-1/               # Project office created from the workspace template.
    └── project-2/               # Additional project office with independent lifecycle state.
```

### User Repository Block Architecture

```plaintext
.corporation/
├── Runtime configuration block       # Mutable settings used by the execution distribution.
│   ├── .env                          # Local process environment.
│   ├── app.json                      # Application preferences.
│   ├── auth.json                     # Authentication provider state.
│   ├── channels.json                 # Connected communication surfaces.
│   ├── config.json                   # Global corporation defaults.
│   ├── gateway.json                  # Gateway lifecycle and integration health.
│   └── models.json                   # AI model catalog.
├── Policy block                      # Global behavior constraints.
│   └── AGENTS.md                     # Rules inherited across the corporation.
├── Employee block                    # Hireable roles with personal memory and capabilities.
│   └── employees/<employee>/
│       ├── config.json               # Role configuration, model preferences, and limits.
│       ├── RULES.md                  # Role-specific operating policy.
│       ├── memories/                 # Employee-owned memory.
│       ├── skills/                   # Role-specific tools and workflows.
│       └── skins/                    # Persona and presentation variants.
└── Workspace block                   # Corporation and project offices.
    └── workspaces/<workspace>/
        ├── config.json               # Workspace identity, manager, team, and policies.
        ├── RULES.md                  # Workspace operating rules.
        ├── cache/                    # Derived artifacts.
        ├── schedules/                # Scheduled work.
        ├── dashboard/                # Management views.
        ├── graph/                    # Structured project and office knowledge.
        ├── hooks/                    # Event automation.
        ├── kanban/                   # Work planning and execution board.
        ├── logs/                     # Operational trace.
        ├── memories/                 # Shared workspace memory.
        ├── office/                   # Spatial office representation.
        ├── sandboxes/                # Isolated execution environments.
        ├── sessions/                 # Conversations and decision history.
        ├── skills/                   # Workspace-owned capabilities.
        └── snapshots/                # Versioned workspace captures.
```
