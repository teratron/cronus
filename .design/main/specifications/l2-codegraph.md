# Code Graph

**Version:** 1.0.0
**Status:** Stable
**Layer:** implementation
**Implements:** l1-storage-model.md

## Overview

An in-workspace code intelligence index: language-aware entity extraction builds a graph of symbols and their relationships, stored in SQLite with FTS5 full-text search and dense vector embeddings. Queries fuse keyword and semantic signals via Reciprocal Rank Fusion. The index builds automatically on first use and updates incrementally on file change.

## Related Specifications

- [l1-storage-model.md](l1-storage-model.md) - STO-8 (SQLite as the durable local store); `codegraph.db` follows the same placement rules.
- [l2-filesystem-layout.md](l2-filesystem-layout.md) - `codegraph.db` lives in the workspace state tier.
- [l2-memory-store.md](l2-memory-store.md) - Memory store uses the same `sqlite-vec` extension for dense embeddings; shared pattern.
- [l2-execution-workspace.md](l2-execution-workspace.md) - Codegraph indexes an execution workspace's source tree during active tasks.
- [l2-quality-pipeline.md](l2-quality-pipeline.md) - Quality pipeline can seed codegraph queries to scope check commands.

## 1. Motivation

Agents working on code tasks need to find where a function is defined, which modules depend on it, and what semantically related symbols exist. Grepping is brittle on large codebases; full-file context floods the context window. A pre-built graph with FTS5 + embedding search lets agents ask precise questions with sub-second answers, without consuming token budget on whole-file reads.

## 2. Constraints & Assumptions

- The index lives entirely on-device in `<ws>/codegraph.db`; no remote calls for indexing or search.
- Embeddings are computed lazily on first search (not at index time) to keep initial indexing fast.
- The index is best-effort: it may be stale by up to one file-save cycle. Agents treat it as a fast cache, not an authoritative source, and must validate symbol locations before acting on them.
- Only files under the workspace's source root are indexed; generated files and vendored dependencies are excluded by default via `.codegraphignore` (same syntax as `.gitignore`).

## 3. Invariant Compliance (Layer 2 only)

| L1 Invariant | Implementation |
| --- | --- |
| STO-2 Immutable program / mutable state | `codegraph.db` is state-tier; the source code being indexed is program-tier (read-only from the index's perspective). |
| STO-8 SQLite as durable store | `codegraph.db` uses WAL mode; all writes are transactional. |

## 4. Detailed Design

### 4.1 Extraction

Language-aware extraction parses each source file and emits a list of symbols and edges.

**Supported languages** (via grammar packages): Rust, TypeScript/JavaScript, Python, Go, C/C++. Additional grammars can be loaded as extension plugins (interface: `CodeGraphGrammar`).

**Extracted entity kinds**:

| Kind | Examples |
| --- | --- |
| `function` | Free functions, methods, closures assigned to `const` |
| `class` | Classes, structs, records |
| `trait` | Traits, interfaces, protocols, abstract classes |
| `enum` | Enums, union types |
| `constant` | `const`, `static`, top-level `let`/`var` with literal initializers |
| `macro` | Rust macros, TypeScript decorators |
| `module` | Files, `mod` declarations, `namespace`s |
| `import` | Import/use statements (edge source only; not emitted as symbol rows) |

**Extracted edge types**:

| Edge | Meaning |
| --- | --- |
| `calls` | Function A calls function B |
| `imports` | Module A imports symbol B |
| `implements` | Type A implements trait/interface B |
| `extends` | Class A extends class B |
| `references` | Symbol A references type/const B (non-call, non-import) |

### 4.2 Storage schema

```text
[REFERENCE]
-- codegraph.db (WAL mode)

CREATE TABLE symbols (
  id          TEXT PRIMARY KEY,   -- SHA-256("{file}:{kind}:{qualified}")[0..16]
  kind        TEXT NOT NULL,      -- entity kind (§4.1)
  name        TEXT NOT NULL,
  qualified   TEXT NOT NULL,      -- fully qualified name ("crate::mod::fn_name")
  file        TEXT NOT NULL,
  start_line  INTEGER NOT NULL,
  end_line    INTEGER NOT NULL,
  signature   TEXT,               -- abbreviated signature / type annotation
  docstring   TEXT,               -- first doc comment block
  lang        TEXT NOT NULL,
  indexed_at  INTEGER NOT NULL    -- Unix ms
);

CREATE VIRTUAL TABLE symbols_fts USING fts5(
  name, qualified, signature, docstring,
  content='symbols', content_rowid='rowid'
);

CREATE TABLE edges (
  from_id   TEXT NOT NULL REFERENCES symbols(id),
  to_id     TEXT NOT NULL REFERENCES symbols(id),
  edge_type TEXT NOT NULL,
  file      TEXT,
  line      INTEGER
);

CREATE TABLE codegraph_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
-- Keys used: last_full_index_at (ISO-8601), file_mtimes (JSON map path→mtime)

-- Dense embeddings via the sqlite-vec extension (same DB):
CREATE VIRTUAL TABLE symbol_vecs USING vec0(
  id        TEXT PRIMARY KEY,
  embedding FLOAT[768]    -- dimension matches the configured embedding model
);
```

### 4.3 Query interface

```text
[REFERENCE]
CodeGraphQuery {
  text:        Option<String>,       // keyword or natural-language query
  kinds:       Option<Vec<String>>,  // filter to specific entity kinds
  lang:        Option<String>,       // filter to a language
  file_prefix: Option<String>,       // filter to a file subtree
  limit:       u32,                  // default 20
  include_edges: bool,               // include outbound edges in results
}

CodeSymbol {
  id:         String,
  kind:       String,
  name:       String,
  qualified:  String,
  file:       String,
  start_line: u32,
  end_line:   u32,
  signature:  Option<String>,
  docstring:  Option<String>,
  lang:       String,
  score:      f32,                   // RRF-fused relevance score
  edges:      Option<Vec<CodeEdge>>,
}

CodeEdge {
  to_id:     String,
  to_name:   String,
  edge_type: String,
  file:      Option<String>,
  line:      Option<u32>,
}
```

### 4.4 RRF fusion

When `text` is provided, the query runs two parallel retrieval passes and fuses their ranked lists:

1. **FTS5 pass**: `SELECT … FROM symbols_fts WHERE symbols_fts MATCH ?` — BM25-ranked.
2. **Embedding pass**: compute query embedding (same model as symbol embeddings), then `SELECT … FROM symbol_vecs ORDER BY vec_distance_cosine(embedding, ?) LIMIT K`.

Scores are fused via Reciprocal Rank Fusion:

```text
[REFERENCE]
rrf_score(doc) = Σ  1 / (k + rank_i(doc))
  where k = 60 (constant), rank_i is the document's 1-based rank in list i, summed over lists
```

Results are sorted descending by `rrf_score`, then trimmed to `limit`. Filters (kind, lang, file_prefix) are applied before ranking.

### 4.5 Auto-index and incremental updates

**First-use trigger**: on the first call to `codegraph.search()` or `codegraph.index()`, the index is built if `codegraph.db` does not exist or its `symbols` table is empty.

**Incremental update**: after each successful build, file mtimes are stored in `codegraph_meta["file_mtimes"]`. On subsequent searches, files with changed mtimes are re-extracted and their symbols upserted; deleted files' symbols are removed. Incremental updates run synchronously but are bounded to ≤ 50 changed files per search call; larger batches are deferred to a background rebuild.

**Background full rebuild**: triggered by the scheduler (configurable cadence; default: daily at idle) or manually via `cronus codegraph index --full`.

### 4.6 Embedding computation

Embeddings are computed on first search, not at index time:

1. `codegraph.search()` detects that `symbol_vecs` is empty or stale (fewer rows than `symbols`).
2. A background task computes embeddings for all symbols in batches (default: 64 symbols per batch).
3. The current query proceeds with FTS5-only ranking until the embedding batch for the query's top results completes; subsequent calls use RRF fusion.

The embedding model is configurable and can be swapped via the extension registry (interface: `EmbeddingModel`). The default is a quantized sentence-embedding model bundled with the application. Embedding dimension must match the `symbol_vecs` schema; changing the model requires a `codegraph purge` and full rebuild.

### 4.7 Command surface

| Action | CLI | TUI | Library (no code) |
| --- | --- | --- | --- |
| build / rebuild index | `cronus codegraph index [--path <root>] [--full]` | `/codegraph index` | `codegraph.index(path, full) -> IndexStats` |
| search symbols | `cronus codegraph search <query> [--kind fn] [--lang rust] [--limit N]` | `/codegraph search …` | `codegraph.search(query) -> CodeSymbol[]` |
| show symbol + edges | `cronus codegraph show <id>` | `/codegraph show <id>` | `codegraph.get(id) -> CodeSymbol` |
| purge index | `cronus codegraph purge` | `/codegraph purge` | `codegraph.purge() -> void` |
| index stats | `cronus codegraph status` | `/codegraph status` | `codegraph.stats() -> IndexStats` |

## 5. Drawbacks & Alternatives

- **Lazy embedding computation**: the first search after a cold index returns FTS5-only results. A warm-up step (`codegraph index --embed`) pre-computes all embeddings if full-quality results from the first query are required.
- **Grammar coverage**: less-common languages fall back to regex-based extraction (function/class name patterns only). Grammar plugins extend coverage without modifying core.
- **Stale entries for deleted files**: if incremental update does not detect a deletion (e.g. file renamed outside tracked paths), stale symbols remain until full rebuild. Agents must validate that referenced files still exist before acting on symbol locations.
- **Alternative — external language servers (LSP)**: richer analysis (type inference, cross-file resolution) but requires per-language server processes, complex lifecycle management, and IPC. The self-contained SQLite approach is more portable and embeddable in the Tauri context.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[STORAGE]` | `.design/main/specifications/l1-storage-model.md` | STO-8 SQLite invariant |
| `[LAYOUT]` | `.design/main/specifications/l2-filesystem-layout.md` | `codegraph.db` placement |
| `[MEM]` | `.design/main/specifications/l2-memory-store.md` | sqlite-vec pattern (shared) |
