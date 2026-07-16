//! SQLite-backed project-wiki store — a derived projection CACHE
//! (l2-project-wiki §4.1). Rows are written only by the office regeneration
//! pipeline (later phase tasks); this module owns the schema and the
//! persistence round-trip. Per PW-3 the store is a rebuildable cache, never a
//! source of truth — dropping `wiki.db` loses nothing durable.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use cronus_contract::{WikiCache, WikiChangelogEntry, WikiCitation, WikiPage, WikiPageKind};

#[derive(Debug)]
pub enum WikiError {
    Database(rusqlite::Error),
    /// A stored row held data the type system rejects (an unknown page kind or
    /// malformed citations JSON) — a corrupt/incompatible cache row. Because
    /// the wiki is a rebuildable projection (PW-3), the recovery for this is a
    /// rebuild, not a restore.
    Corrupt(String),
}

impl std::fmt::Display for WikiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WikiError::Database(e) => write!(f, "wiki database error: {e}"),
            WikiError::Corrupt(m) => write!(f, "corrupt wiki row: {m}"),
        }
    }
}

impl std::error::Error for WikiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WikiError::Database(e) => Some(e),
            WikiError::Corrupt(_) => None,
        }
    }
}

impl From<rusqlite::Error> for WikiError {
    fn from(e: rusqlite::Error) -> Self {
        WikiError::Database(e)
    }
}

pub type Result<T> = std::result::Result<T, WikiError>;

/// The per-office wiki cache (`<state>/workspaces/<ws>/wiki/wiki.db`).
pub struct WikiStore {
    conn: Connection,
}

impl WikiStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        setup(&conn)?;
        Ok(WikiStore { conn })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        setup(&conn)?;
        Ok(WikiStore { conn })
    }

    /// Write a page (insert or replace by id) and keep the FTS index in sync.
    /// The client never calls this — only the office regeneration pipeline
    /// (PW-2); it lives here as the store's write half for that pipeline.
    pub fn upsert_page(&self, page: &WikiPage) -> Result<()> {
        upsert_page_on(&self.conn, page)
    }

    /// Read a page by id, or `None` if absent.
    pub fn get_page(&self, id: &str) -> Result<Option<WikiPage>> {
        self.conn
            .query_row(
                "SELECT id, office_id, parent_id, ord, kind, title, body,
                        citations, source_fingerprint, generated_at, stale
                 FROM wiki_page WHERE id = ?1",
                params![id],
                row_to_page,
            )
            .optional()?
            .transpose()
    }

    /// Append a change-history entry (PW-5, newest-first by `at`).
    pub fn append_changelog(&self, entry: &WikiChangelogEntry) -> Result<()> {
        append_changelog_on(&self.conn, entry)
    }

    /// Apply a regeneration transactionally (PW-3): upsert every page and
    /// append every changelog entry inside one SQLite transaction, so a
    /// failure part-way rolls the whole batch back and the prior rows stay
    /// intact. `unchecked_transaction` is used because the store holds the
    /// connection behind `&self` (no `&mut` handle) — the batch is the only
    /// writer, so there is no reentrancy to guard against.
    pub fn apply_regeneration(
        &self,
        pages: &[WikiPage],
        changelog: &[WikiChangelogEntry],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for page in pages {
            upsert_page_on(&tx, page)?;
        }
        for entry in changelog {
            append_changelog_on(&tx, entry)?;
        }
        tx.commit()?;
        Ok(())
    }
}

impl WikiCache for WikiStore {
    fn get_page(&self, id: &str) -> std::result::Result<Option<WikiPage>, String> {
        WikiStore::get_page(self, id).map_err(|e| e.to_string())
    }

    fn apply_regeneration(
        &self,
        pages: &[WikiPage],
        changelog: &[WikiChangelogEntry],
    ) -> std::result::Result<(), String> {
        WikiStore::apply_regeneration(self, pages, changelog).map_err(|e| e.to_string())
    }
}

fn upsert_page_on(conn: &Connection, page: &WikiPage) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO wiki_page
         (id, office_id, parent_id, ord, kind, title, body,
          citations, source_fingerprint, generated_at, stale)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            page.id,
            page.office_id,
            page.parent_id,
            page.ord,
            page.kind.as_str(),
            page.title,
            page.body,
            citations_to_json(&page.citations),
            page.source_fingerprint,
            page.generated_at as i64,
            page.stale as i64,
        ],
    )?;
    // Standalone FTS index (like the memory store): re-sync this page's row.
    conn.execute(
        "DELETE FROM wiki_page_fts WHERE page_id = ?1",
        params![page.id],
    )?;
    conn.execute(
        "INSERT INTO wiki_page_fts (page_id, title, body) VALUES (?1, ?2, ?3)",
        params![page.id, page.title, page.body],
    )?;
    Ok(())
}

fn append_changelog_on(conn: &Connection, entry: &WikiChangelogEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO wiki_changelog (id, office_id, page_id, change, at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            entry.id,
            entry.office_id,
            entry.page_id,
            entry.change,
            entry.at as i64,
        ],
    )?;
    Ok(())
}

/// Map a `wiki_page` row to a `WikiPage`. Returns `Result` inside the row
/// closure's `Ok` so a corrupt row (unknown kind / bad JSON) surfaces as
/// `WikiError::Corrupt` rather than a panic.
fn row_to_page(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<WikiPage>> {
    let kind_str: String = row.get(4)?;
    let citations_json: String = row.get(7)?;
    let stale: i64 = row.get(10)?;
    let generated_at: i64 = row.get(9)?;

    Ok((|| {
        let kind = WikiPageKind::from_db_str(&kind_str)
            .ok_or_else(|| WikiError::Corrupt(format!("unknown page kind: {kind_str}")))?;
        let citations = citations_from_json(&citations_json)?;
        Ok(WikiPage {
            id: row.get(0)?,
            office_id: row.get(1)?,
            parent_id: row.get(2)?,
            ord: row.get(3)?,
            kind,
            title: row.get(5)?,
            body: row.get(6)?,
            citations,
            source_fingerprint: row.get(8)?,
            generated_at: generated_at as u64,
            stale: stale != 0,
        })
    })())
}

fn citations_to_json(citations: &[WikiCitation]) -> String {
    let arr: Vec<serde_json::Value> = citations
        .iter()
        .map(|c| serde_json::json!({ "source_kind": c.source_kind, "source_id": c.source_id }))
        .collect();
    serde_json::Value::Array(arr).to_string()
}

fn citations_from_json(s: &str) -> Result<Vec<WikiCitation>> {
    let value: serde_json::Value =
        serde_json::from_str(s).map_err(|e| WikiError::Corrupt(e.to_string()))?;
    let array = value
        .as_array()
        .ok_or_else(|| WikiError::Corrupt("citations is not a JSON array".to_string()))?;
    array
        .iter()
        .map(|c| {
            let source_kind = c
                .get("source_kind")
                .and_then(|v| v.as_str())
                .ok_or_else(|| WikiError::Corrupt("citation missing source_kind".to_string()))?;
            let source_id = c
                .get("source_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| WikiError::Corrupt("citation missing source_id".to_string()))?;
            Ok(WikiCitation::new(source_kind, source_id))
        })
        .collect()
}

pub(crate) fn setup(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS wiki_page (
            id                 TEXT PRIMARY KEY NOT NULL,
            office_id          TEXT NOT NULL,
            parent_id          TEXT,
            ord                INTEGER NOT NULL DEFAULT 0,
            kind               TEXT NOT NULL,
            title              TEXT NOT NULL,
            body               TEXT NOT NULL,
            citations          TEXT NOT NULL,
            source_fingerprint TEXT NOT NULL,
            generated_at       INTEGER NOT NULL,
            stale              INTEGER NOT NULL DEFAULT 0
        )",
    )?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS wiki_changelog (
            id        TEXT PRIMARY KEY NOT NULL,
            office_id TEXT NOT NULL,
            page_id   TEXT,
            change    TEXT NOT NULL,
            at        INTEGER NOT NULL
        )",
    )?;
    // Standalone FTS5 (synced manually in upsert_page), matching the memory
    // store's proven pattern rather than an external-content table — the spec
    // DDL's `content=wiki_page` is marked illustrative and would couple the
    // FTS rowid to wiki_page's implicit rowid despite its TEXT primary key.
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS wiki_page_fts USING fts5(
            page_id UNINDEXED,
            title,
            body
        )",
    )?;
    conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_wiki_parent ON wiki_page(parent_id, ord)")?;
    conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_wiki_stale ON wiki_page(office_id, stale)")?;
    Ok(())
}

#[cfg(test)]
mod schema {
    use super::*;

    fn names(conn: &Connection, kind: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = ?1 ORDER BY name")
            .unwrap();
        stmt.query_map(params![kind], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    #[test]
    fn tables_and_indices_created() {
        let store = WikiStore::open_in_memory().expect("open");
        let tables = names(&store.conn, "table");
        assert!(tables.contains(&"wiki_page".to_string()));
        assert!(tables.contains(&"wiki_changelog".to_string()));
        assert!(tables.contains(&"wiki_page_fts".to_string()));

        let indices = names(&store.conn, "index");
        assert!(indices.contains(&"idx_wiki_parent".to_string()));
        assert!(indices.contains(&"idx_wiki_stale".to_string()));
    }

    #[test]
    fn a_page_round_trips_with_its_citations_and_fingerprint() {
        let store = WikiStore::open_in_memory().expect("open");

        let mut page = WikiPage::new(
            "overview",
            "office-1",
            WikiPageKind::Overview,
            "Project Overview",
            "This project builds a local-first AI office.",
        );
        page.citations = vec![
            WikiCitation::new("decision", "dec-42"),
            WikiCitation::new("board_item", "card-7"),
        ];
        page.source_fingerprint = "fp-abc123".to_string();
        page.generated_at = 1_700_000_000;

        store.upsert_page(&page).expect("upsert");
        let got = store.get_page("overview").expect("get").expect("present");

        assert_eq!(got, page, "the page round-trips unchanged");
        assert_eq!(got.citations.len(), 2);
        assert_eq!(got.source_fingerprint, "fp-abc123");
        assert!(!got.stale);

        // A missing id is None, not an error.
        assert!(store.get_page("nope").expect("get missing").is_none());
    }

    #[test]
    fn upsert_replaces_by_id_and_resyncs_fts() {
        let store = WikiStore::open_in_memory().expect("open");
        let mut page = WikiPage::new("p", "office-1", WikiPageKind::Area, "T", "old body");
        page.citations = vec![WikiCitation::new("ledger_fact", "f1")];
        store.upsert_page(&page).expect("first upsert");

        page.body = "new body".to_string();
        store.upsert_page(&page).expect("replace");

        let got = store.get_page("p").expect("get").expect("present");
        assert_eq!(got.body, "new body");

        // FTS holds exactly one row for this page after the replace.
        let fts_count: i64 = store
            .conn
            .query_row(
                "SELECT count(*) FROM wiki_page_fts WHERE page_id = ?1",
                params!["p"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fts_count, 1);
    }

    fn changelog(id: &str) -> WikiChangelogEntry {
        WikiChangelogEntry {
            id: id.to_string(),
            office_id: "office-1".to_string(),
            page_id: Some("p".to_string()),
            change: "regenerated".to_string(),
            at: 1_700_000_000,
        }
    }

    #[test]
    fn apply_regeneration_is_atomic_and_rolls_back_on_failure() {
        let store = WikiStore::open_in_memory().expect("open");

        // Seed a committed page + changelog.
        let mut original = WikiPage::new("p", "office-1", WikiPageKind::Overview, "T", "original");
        original.citations = vec![WikiCitation::new("decision", "d1")];
        store
            .apply_regeneration(&[original.clone()], &[changelog("cl-1")])
            .expect("seed regeneration commits");

        // A second batch updates the page but its changelog has a DUPLICATE id
        // (`cl-1` already exists) — the second changelog INSERT violates the
        // PRIMARY KEY, failing the transaction part-way.
        let mut updated = original.clone();
        updated.body = "SHOULD NOT PERSIST".to_string();
        let result = store.apply_regeneration(&[updated], &[changelog("cl-1")]);
        assert!(
            result.is_err(),
            "a duplicate changelog id must fail the batch"
        );

        // The whole batch rolled back: the prior page row is intact...
        let got = store.get_page("p").expect("get").expect("present");
        assert_eq!(
            got.body, "original",
            "the failed regeneration must not have written"
        );
        // ...and no duplicate/partial changelog row was left behind.
        let cl_count: i64 = store
            .conn
            .query_row("SELECT count(*) FROM wiki_changelog", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cl_count, 1, "only the seed changelog row exists");
    }
}
