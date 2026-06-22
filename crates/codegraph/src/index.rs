//! SQLite + FTS5 code symbol index.

use rusqlite::{params, Connection};

use crate::extractor::Symbol;

// ── Schema ────────────────────────────────────────────────────────────────────

/// Create the `symbols` and `symbols_fts` tables.
pub fn migrate(conn: &Connection) -> IndexResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS symbols (
            id       INTEGER PRIMARY KEY AUTOINCREMENT,
            name     TEXT    NOT NULL,
            kind     TEXT    NOT NULL,
            file     TEXT    NOT NULL,
            line     INTEGER NOT NULL,
            doc      TEXT
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
            name,
            doc,
            content='symbols',
            content_rowid='id'
        );
        CREATE TRIGGER IF NOT EXISTS symbols_ai
            AFTER INSERT ON symbols BEGIN
                INSERT INTO symbols_fts(rowid, name, doc)
                VALUES (new.id, new.name, new.doc);
        END;
        CREATE TRIGGER IF NOT EXISTS symbols_ad
            AFTER DELETE ON symbols BEGIN
                INSERT INTO symbols_fts(symbols_fts, rowid, name, doc)
                VALUES ('delete', old.id, old.name, old.doc);
        END;
        ",
    )?;
    Ok(())
}

// ── IndexedSymbol ─────────────────────────────────────────────────────────────

/// A symbol retrieved from the index.
#[derive(Debug, Clone)]
pub struct IndexedSymbol {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: u32,
    pub doc: Option<String>,
}

// ── Store ─────────────────────────────────────────────────────────────────────

/// Insert a batch of symbols from `file` into the index.
pub fn store_symbols(conn: &Connection, file: &str, symbols: &[Symbol]) -> IndexResult<usize> {
    let mut count = 0;
    for sym in symbols {
        conn.execute(
            "INSERT INTO symbols (name, kind, file, line, doc) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                sym.name,
                format!("{:?}", sym.kind),
                file,
                sym.line,
                sym.doc
            ],
        )?;
        count += 1;
    }
    Ok(count)
}

/// Retrieve a symbol by exact name.
pub fn get_by_name(conn: &Connection, name: &str) -> IndexResult<Option<IndexedSymbol>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, kind, file, line, doc FROM symbols WHERE name = ?1 LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![name], map_row)?;
    Ok(rows.next().and_then(|r| r.ok()))
}

/// FTS5 keyword search — returns up to `limit` symbols ranked by relevance.
pub fn fts_search(conn: &Connection, query: &str, limit: usize) -> IndexResult<Vec<IndexedSymbol>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.kind, s.file, s.line, s.doc
         FROM symbols s
         JOIN symbols_fts f ON f.rowid = s.id
         WHERE symbols_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    )?;
    let rows: Vec<IndexedSymbol> = stmt
        .query_map(params![query, limit as i64], map_row)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

fn map_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<IndexedSymbol> {
    Ok(IndexedSymbol {
        id: r.get(0)?,
        name: r.get(1)?,
        kind: r.get(2)?,
        file: r.get(3)?,
        line: r.get::<_, u32>(4)?,
        doc: r.get(5)?,
    })
}

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum IndexError {
    Db(rusqlite::Error),
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::Db(e) => write!(f, "codegraph index error: {e}"),
        }
    }
}

impl std::error::Error for IndexError {}

impl From<rusqlite::Error> for IndexError {
    fn from(e: rusqlite::Error) -> Self {
        IndexError::Db(e)
    }
}

pub type IndexResult<T> = Result<T, IndexError>;
