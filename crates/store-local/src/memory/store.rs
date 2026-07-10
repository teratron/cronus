//! SQLite-backed memory store with FTS5 and session chaining.

use rusqlite::{Connection, params};

use super::{
    CodeChangeType, MemoryEntry, MemoryError, MemoryId, MemoryKind, MemorySource, Result,
    SuggestedAction, TrustUpdate, VerificationState,
    chain::{BELLMAN_MAX_DEPTH, ChainKind, SESSION_CHAIN_WINDOW_SECS, propagated_delta},
    now_secs,
    trust::{TRUST_MIN_SEARCH, apply_delta},
};

pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        setup(&conn)?;
        Ok(MemoryStore { conn })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        setup(&conn)?;
        Ok(MemoryStore { conn })
    }

    // ── write ─────────────────────────────────────────────────────────────────

    /// Insert a new memory entry. Auto-chains to the most recent memory in
    /// the same workspace if one exists within the session window.
    pub fn add(&self, entry: MemoryEntry) -> Result<MemoryId> {
        let id = entry.id.clone();
        self.conn.execute(
            "INSERT INTO memories
             (id, kind, source, title, body, confidence, trust_score,
              valid_at, created_at, superseded_at, workspace_id, verification_state)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                id.as_str(),
                entry.kind.as_str(),
                entry.source.as_str(),
                &entry.title,
                &entry.body,
                entry.confidence,
                entry.trust_score,
                entry.valid_at as i64,
                entry.created_at as i64,
                entry.superseded_at.map(|v| v as i64),
                entry.workspace_id.as_deref(),
                entry.verification_state.as_str(),
            ],
        )?;

        // Sync standalone FTS index (standalone table — no triggers needed)
        self.conn.execute(
            "INSERT INTO memories_fts(memory_id, title, body) VALUES (?1, ?2, ?3)",
            params![id.as_str(), &entry.title, &entry.body],
        )?;

        // Auto-chain within session window
        if let Some(ws) = &entry.workspace_id
            && let Some(prev) = self.latest_in_workspace(ws, entry.created_at)?
            && prev.as_str() != id.as_str()
        {
            self.chain_internal(&prev, &id, ChainKind::Continuation)?;
        }

        Ok(id)
    }

    /// Update the trust score for a memory. Returns the new trust score.
    pub fn update_trust(&self, id: &MemoryId, update: TrustUpdate) -> Result<f64> {
        let current: f64 = self.conn.query_row(
            "SELECT trust_score FROM memories WHERE id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )?;

        let new_score = apply_delta(current, update.positive);

        if let Some(vs) = &update.new_verification_state {
            self.conn.execute(
                "UPDATE memories SET trust_score = ?1, verification_state = ?2 WHERE id = ?3",
                params![new_score, vs.as_str(), id.as_str()],
            )?;
        } else {
            self.conn.execute(
                "UPDATE memories SET trust_score = ?1 WHERE id = ?2",
                params![new_score, id.as_str()],
            )?;
        }

        Ok(new_score)
    }

    /// Create an explicit chain link between two memories.
    pub fn chain(&self, source: &MemoryId, target: &MemoryId, kind: ChainKind) -> Result<()> {
        self.chain_internal(source, target, kind)
    }

    /// Propagate a trust delta through the chain graph (Bellman, depth ≤ 2).
    pub fn propagate_trust(&self, from: &MemoryId, base_delta: f64) -> Result<()> {
        self.propagate_recursive(from, base_delta, 0)
    }

    /// Lower trust of memories in a workspace based on a code change event.
    ///
    /// Returns the IDs of memories whose trust was adjusted.
    pub fn apply_code_change(
        &self,
        workspace_id: &str,
        change: CodeChangeType,
    ) -> Result<Vec<MemoryId>> {
        let action = change.suggested_action();
        let weight = match &action {
            SuggestedAction::Invalidate(w) => *w,
            SuggestedAction::Review(w) => *w,
            SuggestedAction::Update(w) => *w,
            SuggestedAction::None => return Ok(vec![]),
        };

        let ids = self.ids_in_workspace(workspace_id)?;
        for id in &ids {
            let current: f64 = self.conn.query_row(
                "SELECT trust_score FROM memories WHERE id = ?1",
                params![id.as_str()],
                |row| row.get(0),
            )?;
            let new_score = (current - weight * 0.1).clamp(0.0, 1.0);
            self.conn.execute(
                "UPDATE memories SET trust_score = ?1 WHERE id = ?2",
                params![new_score, id.as_str()],
            )?;
        }
        Ok(ids)
    }

    // ── read ──────────────────────────────────────────────────────────────────

    pub fn get(&self, id: &MemoryId) -> Result<Option<MemoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, source, title, body, confidence, trust_score,
                    valid_at, created_at, superseded_at, workspace_id, verification_state
             FROM memories WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id.as_str()], map_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// Full-text search over title+body. Filters entries below TRUST_MIN_SEARCH.
    pub fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        // Step 1: collect matching memory IDs from the FTS index.
        let mut fts_stmt = self
            .conn
            .prepare("SELECT memory_id FROM memories_fts WHERE memories_fts MATCH ?1 LIMIT ?2")?;
        let ids: Vec<String> = fts_stmt
            .query_map(params![query, limit as i64], |row| row.get(0))?
            .collect::<std::result::Result<_, _>>()?;

        if ids.is_empty() {
            return Ok(vec![]);
        }

        // Step 2: fetch full entries for those IDs.
        // The SQL trust_score filter is the gate — effective_trust is not applied
        // here so that newly added entries (Untested verification state) still
        // surface in search results at their raw trust_score.
        let mut out = Vec::with_capacity(ids.len());
        let mut mem_stmt = self.conn.prepare(
            "SELECT id, kind, source, title, body, confidence, trust_score,
                    valid_at, created_at, superseded_at, workspace_id, verification_state
             FROM memories
             WHERE id = ?1 AND trust_score >= ?2 AND superseded_at IS NULL",
        )?;
        for id in &ids {
            let mut rows = mem_stmt.query_map(params![id, TRUST_MIN_SEARCH], map_row)?;
            if let Some(entry) = rows.next() {
                out.push(entry?);
            }
        }
        Ok(out)
    }

    /// Hard-delete a memory entry by raw ID string.
    ///
    /// Returns `true` when a row was removed, `false` when not found.
    pub fn delete(&self, id: &str) -> Result<bool> {
        let affected = self
            .conn
            .execute("DELETE FROM memories WHERE id = ?1", params![id])?;
        if affected > 0 {
            let _ = self
                .conn
                .execute("DELETE FROM memories_fts WHERE memory_id = ?1", params![id]);
        }
        Ok(affected > 0)
    }

    /// HRR encoding seam — returns a zeroed vector (stub).
    /// Real implementation ships with sqlite-vec integration.
    pub fn encode_hrr(&self, _entry: &MemoryEntry) -> Vec<f32> {
        vec![0.0f32; 256]
    }

    /// Every stored entry, unfiltered. Backs `UserDataStore::export` (DN-7:
    /// always able to come home) — unlike `search_fts`, this applies no
    /// trust-score gate, since export is an operator/backup action, not a
    /// ranked retrieval.
    pub fn export_all(&self) -> Result<Vec<MemoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, source, title, body, confidence, trust_score,
                    valid_at, created_at, superseded_at, workspace_id, verification_state
             FROM memories",
        )?;
        let entries = stmt
            .query_map([], map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn latest_in_workspace(
        &self,
        workspace_id: &str,
        before_secs: u64,
    ) -> Result<Option<MemoryId>> {
        let cutoff = before_secs.saturating_sub(SESSION_CHAIN_WINDOW_SECS) as i64;
        let result = self.conn.query_row(
            "SELECT id FROM memories
             WHERE workspace_id = ?1 AND created_at >= ?2
             ORDER BY created_at DESC LIMIT 1",
            params![workspace_id, cutoff],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(id) => Ok(Some(MemoryId::from(id))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(MemoryError::Database(e)),
        }
    }

    fn chain_internal(&self, source: &MemoryId, target: &MemoryId, kind: ChainKind) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO memory_chains (source_id, target_id, kind, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                source.as_str(),
                target.as_str(),
                kind.as_str(),
                now_secs() as i64
            ],
        )?;
        Ok(())
    }

    fn propagate_recursive(&self, from: &MemoryId, base_delta: f64, depth: usize) -> Result<()> {
        if depth >= BELLMAN_MAX_DEPTH {
            return Ok(());
        }
        let delta = propagated_delta(base_delta, depth);

        let mut stmt = self
            .conn
            .prepare("SELECT target_id FROM memory_chains WHERE source_id = ?1")?;
        let targets: Vec<String> = stmt
            .query_map(params![from.as_str()], |row| row.get(0))?
            .collect::<std::result::Result<_, _>>()?;

        for target_str in targets {
            let target = MemoryId::from(target_str);
            let current: f64 = match self.conn.query_row(
                "SELECT trust_score FROM memories WHERE id = ?1",
                params![target.as_str()],
                |row| row.get(0),
            ) {
                Ok(v) => v,
                Err(rusqlite::Error::QueryReturnedNoRows) => continue,
                Err(e) => return Err(MemoryError::Database(e)),
            };
            let new_score = (current + delta).clamp(0.0, 1.0);
            self.conn.execute(
                "UPDATE memories SET trust_score = ?1 WHERE id = ?2",
                params![new_score, target.as_str()],
            )?;
            self.propagate_recursive(&target, base_delta, depth + 1)?;
        }
        Ok(())
    }

    fn ids_in_workspace(&self, workspace_id: &str) -> Result<Vec<MemoryId>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM memories WHERE workspace_id = ?1")?;
        let ids = stmt
            .query_map(params![workspace_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .map(MemoryId::from)
            .collect();
        Ok(ids)
    }
}

// ── MemorySearch (ports tier) ────────────────────────────────────────────────
//
// The pivot (§4.6): `MemoryStore` is the concrete, SQLite-backed
// implementation of the `MemorySearch` seam declared in `cronus-contract`.
// `ContextRouter` depends on the trait, never on this type — this `impl`
// block is the only place that still names both sides.

impl cronus_contract::MemorySearch for MemoryStore {
    fn search_fts(
        &self,
        query: &str,
        limit: usize,
    ) -> std::result::Result<Vec<MemoryEntry>, String> {
        MemoryStore::search_fts(self, query, limit).map_err(|e| e.to_string())
    }
}

impl cronus_contract::UserDataStore for MemoryStore {
    fn put(&self, entry: &MemoryEntry) -> std::result::Result<(), String> {
        MemoryStore::add(self, entry.clone())
            .map(|_id| ())
            .map_err(|e| e.to_string())
    }

    fn export(&self) -> std::result::Result<Vec<MemoryEntry>, String> {
        MemoryStore::export_all(self).map_err(|e| e.to_string())
    }
}

// ── schema ────────────────────────────────────────────────────────────────────

fn setup(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL")?;
    conn.execute_batch("PRAGMA foreign_keys = ON")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memories (
            id                 TEXT PRIMARY KEY NOT NULL,
            kind               TEXT NOT NULL,
            source             TEXT NOT NULL,
            title              TEXT NOT NULL,
            body               TEXT NOT NULL,
            confidence         REAL NOT NULL DEFAULT 1.0,
            trust_score        REAL NOT NULL DEFAULT 0.5,
            valid_at           INTEGER NOT NULL,
            created_at         INTEGER NOT NULL,
            superseded_at      INTEGER,
            workspace_id       TEXT,
            verification_state TEXT NOT NULL DEFAULT 'Untested'
        )",
    )?;
    // Standalone FTS5 table — synced manually in add().
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
            memory_id UNINDEXED,
            title,
            body
        )",
    )?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memory_chains (
            source_id  TEXT NOT NULL,
            target_id  TEXT NOT NULL,
            kind       TEXT NOT NULL DEFAULT 'Continuation',
            created_at INTEGER NOT NULL,
            PRIMARY KEY (source_id, target_id)
        )",
    )?;
    Ok(())
}

// ── row mapper ────────────────────────────────────────────────────────────────

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryEntry> {
    let id: String = row.get(0)?;
    let kind_str: String = row.get(1)?;
    let src_str: String = row.get(2)?;
    let valid_at: i64 = row.get(7)?;
    let created_at: i64 = row.get(8)?;
    let superseded_at: Option<i64> = row.get(9)?;
    let vs_str: String = row.get(11)?;

    Ok(MemoryEntry {
        id: MemoryId::from(id),
        kind: MemoryKind::from_db_str(&kind_str).unwrap_or(MemoryKind::ProjectContext),
        source: MemorySource::from_db_str(&src_str).unwrap_or(MemorySource::System),
        title: row.get(3)?,
        body: row.get(4)?,
        confidence: row.get(5)?,
        trust_score: row.get(6)?,
        valid_at: valid_at as u64,
        created_at: created_at as u64,
        superseded_at: superseded_at.map(|v| v as u64),
        workspace_id: row.get(10)?,
        verification_state: VerificationState::from_db_str(&vs_str)
            .unwrap_or(VerificationState::Untested),
    })
}
