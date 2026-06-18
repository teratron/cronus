# Global Memory (installation level)

Cross-project, long-lived memory about the human client and globally learned facts.

- `global.db` — SQLite + vector index (facts, preferences). Created at runtime.
- `graph.db` — global knowledge graph. Created at runtime.
- `notes/` — human-readable Markdown memory (editable).

Read order at retrieval: employee → workspace → global (most specific wins).
