//! Hybrid semantic + keyword retrieval (l2-knowledge-store §4.3, KB-1/KB-6/KB-7).
//!
//! Composes the already-KB-1-scoped store primitives
//! ([`KnowledgeStore::ann_search`], [`KnowledgeStore::fts_search`],
//! [`KnowledgeStore::hydrate_chunks`] — all built in the schema/store-
//! scaffolding task) with Reciprocal Rank Fusion. This module owns none of
//! the collection-isolation or curation-floor logic itself — it composes
//! store primitives that already enforce them, then ranks the union.

use std::collections::HashMap;

use cronus_contract::{KnowledgeStore, RetrievalRequest, RetrievedChunk};

use crate::knowledge_ingest::EmbeddingBackend;

/// A failure during retrieval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrievalError {
    /// The embedding backend refused or failed on the query.
    Embed(String),
    /// The knowledge store rejected a search or hydration call.
    Store(String),
}

impl std::fmt::Display for RetrievalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetrievalError::Embed(m) => write!(f, "query embedding failed: {m}"),
            RetrievalError::Store(m) => write!(f, "store search failed: {m}"),
        }
    }
}

impl std::error::Error for RetrievalError {}

/// Reciprocal Rank Fusion constant (l2-knowledge-store §4.3: "RRF, k=60").
const RRF_K: f64 = 60.0;

/// Run a hybrid retrieval request: embed the query, search both modalities
/// (each over-fetched at `top_k * 2` beyond the final cut, giving RRF enough
/// candidates per list to rank cross-modal matches well), fuse by
/// Reciprocal Rank Fusion, then hydrate the fused top `top_k` ids into fully
/// attributed [`RetrievedChunk`]s (KB-6) — `hydrate_chunks` also applies the
/// KB-10 `min_curation` floor, so a chunk RRF ranked highly can still be
/// absent from the final set if it falls below the requested curation level.
///
/// `request.min_score` filters on the **fused RRF score** — a relative
/// ranking signal (small positive reciprocals), not a normalized probability;
/// the default `None` applies no floor. KB-1 (never implicit "search
/// everything") is enforced by delegating to `ann_search`/`fts_search`
/// unconditionally on `request.collection_ids` — an empty list short-circuits
/// to an empty result without touching the store at all. KB-7 (non-
/// authoritative recall) holds by construction: [`RetrievedChunk`] carries
/// only `(text, source_ref, score)`, never an assertion of correctness.
pub fn retrieve(
    store: &dyn KnowledgeStore,
    embedder: &dyn EmbeddingBackend,
    request: &RetrievalRequest,
) -> Result<Vec<RetrievedChunk>, RetrievalError> {
    if request.collection_ids.is_empty() || request.top_k == 0 {
        return Ok(Vec::new());
    }

    let over_fetch = request.top_k.saturating_mul(2);
    let query_vector = embedder
        .embed(&request.query)
        .map_err(RetrievalError::Embed)?;
    let ann = store
        .ann_search(&request.collection_ids, &query_vector, over_fetch)
        .map_err(RetrievalError::Store)?;
    let fts = store
        .fts_search(&request.collection_ids, &request.query, over_fetch)
        .map_err(RetrievalError::Store)?;

    let mut fused = rrf_fuse(&ann, &fts);
    fused.truncate(request.top_k);
    let score_by_id: HashMap<String, f64> = fused.iter().cloned().collect();

    let chunk_ids: Vec<String> = fused.into_iter().map(|(id, _)| id).collect();
    let hydrated = store
        .hydrate_chunks(&chunk_ids, request.min_curation)
        .map_err(RetrievalError::Store)?;

    let mut results: Vec<RetrievedChunk> = hydrated
        .into_iter()
        .map(|mut c| {
            c.score = score_by_id.get(&c.chunk_id).copied().unwrap_or(0.0) as f32;
            c
        })
        .filter(|c| request.min_score.is_none_or(|floor| c.score >= floor))
        .collect();
    // `hydrate_chunks` may return rows in an arbitrary (e.g. SQL) order and
    // may drop some ids (the KB-10 curation floor) — re-sort by the fused
    // rank so the caller sees best-match-first regardless.
    results.sort_by(|a, b| b.score.total_cmp(&a.score));
    Ok(results)
}

/// Fuse two ranked (best-first) candidate lists by Reciprocal Rank Fusion:
/// each list contributes `1 / (k + rank)` per appearance (rank 1-indexed), summed
/// per chunk id. Returns pairs sorted by descending fused score.
fn rrf_fuse(ann: &[(String, f32)], fts: &[(String, f32)]) -> Vec<(String, f64)> {
    let mut scores: HashMap<String, f64> = HashMap::new();
    for (rank, (chunk_id, _)) in ann.iter().enumerate() {
        *scores.entry(chunk_id.clone()).or_insert(0.0) += 1.0 / (RRF_K + (rank + 1) as f64);
    }
    for (rank, (chunk_id, _)) in fts.iter().enumerate() {
        *scores.entry(chunk_id.clone()).or_insert(0.0) += 1.0 / (RRF_K + (rank + 1) as f64);
    }
    let mut fused: Vec<(String, f64)> = scores.into_iter().collect();
    fused.sort_by(|a, b| b.1.total_cmp(&a.1));
    fused
}

#[cfg(test)]
mod tests {
    use super::*;
    use cronus_contract::{
        Chunk, Collection, Curation, Directory, Document, SourceRef, WriteOverride,
    };
    use std::cell::RefCell;

    #[derive(Default)]
    struct ScriptedStore {
        ann: Vec<(String, f32)>,
        fts: Vec<(String, f32)>,
        chunks: Vec<RetrievedChunk>,
        last_ann_collections: RefCell<Vec<String>>,
        last_fts_collections: RefCell<Vec<String>>,
    }

    impl KnowledgeStore for ScriptedStore {
        fn create_collection(&self, _c: &Collection) -> Result<(), String> {
            Ok(())
        }
        fn get_collection(&self, _id: &str) -> Result<Option<Collection>, String> {
            Ok(None)
        }
        fn create_directory(&self, _d: &Directory) -> Result<(), String> {
            Ok(())
        }
        fn write_document(&self, _d: &Document, _o: &WriteOverride) -> Result<(), String> {
            Ok(())
        }
        fn get_document(&self, _id: &str) -> Result<Option<Document>, String> {
            Ok(None)
        }
        fn set_curation(
            &self,
            _id: &str,
            _next: Curation,
            _human_auth: Option<&str>,
        ) -> Result<(), String> {
            Ok(())
        }
        fn soft_delete_document(&self, _id: &str) -> Result<(), String> {
            Ok(())
        }
        fn gc(&self, _older_than_secs: u64) -> Result<u64, String> {
            Ok(0)
        }
        fn delete_chunks(&self, _document_id: &str) -> Result<(), String> {
            Ok(())
        }
        fn insert_chunk(&self, _chunk: &Chunk, _embedding: &[f32]) -> Result<(), String> {
            Ok(())
        }
        fn reindex_chunks(
            &self,
            _document_id: &str,
            _chunks: &[(Chunk, Vec<f32>)],
        ) -> Result<(), String> {
            Ok(())
        }
        fn ann_search(
            &self,
            collection_ids: &[String],
            _query_vector: &[f32],
            _top_k: usize,
        ) -> Result<Vec<(String, f32)>, String> {
            *self.last_ann_collections.borrow_mut() = collection_ids.to_vec();
            Ok(self.ann.clone())
        }
        fn fts_search(
            &self,
            collection_ids: &[String],
            _query_text: &str,
            _top_k: usize,
        ) -> Result<Vec<(String, f32)>, String> {
            *self.last_fts_collections.borrow_mut() = collection_ids.to_vec();
            Ok(self.fts.clone())
        }
        fn hydrate_chunks(
            &self,
            chunk_ids: &[String],
            _min_curation: Option<Curation>,
        ) -> Result<Vec<RetrievedChunk>, String> {
            Ok(self
                .chunks
                .iter()
                .filter(|c| chunk_ids.contains(&c.chunk_id))
                .cloned()
                .collect())
        }
    }

    fn chunk(id: &str, collection_id: &str) -> RetrievedChunk {
        RetrievedChunk {
            chunk_id: id.to_string(),
            document_id: format!("doc-{id}"),
            collection_id: collection_id.to_string(),
            text: format!("text for {id}"),
            source_ref: Some(SourceRef {
                page: Some(1),
                ..Default::default()
            }),
            score: 0.0,
        }
    }

    struct FakeEmbedder;
    impl EmbeddingBackend for FakeEmbedder {
        fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
            Ok(vec![0.1, 0.2])
        }
    }

    #[test]
    fn retrieve_fuses_ann_and_fts_by_reciprocal_rank() {
        let store = ScriptedStore {
            ann: vec![("a".into(), 0.1), ("b".into(), 0.2)],
            fts: vec![("b".into(), 1.0), ("c".into(), 2.0)],
            chunks: vec![
                chunk("a", "col-1"),
                chunk("b", "col-1"),
                chunk("c", "col-1"),
            ],
            ..Default::default()
        };
        let request = RetrievalRequest::new("query", vec!["col-1".to_string()]);
        let results = retrieve(&store, &FakeEmbedder, &request).expect("retrieve");

        // "b" appears in BOTH lists (rank 2 in ann, rank 1 in fts) — its
        // summed reciprocal-rank score beats either single-list top hit.
        assert_eq!(results[0].chunk_id, "b");
        let ids: Vec<&str> = results.iter().map(|c| c.chunk_id.as_str()).collect();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"c"));
    }

    #[test]
    fn kb1_retrieve_scopes_both_searches_to_the_requested_collections() {
        let store = ScriptedStore::default();
        let request = RetrievalRequest::new("q", vec!["col-a".to_string(), "col-b".to_string()]);
        retrieve(&store, &FakeEmbedder, &request).expect("retrieve");
        assert_eq!(
            *store.last_ann_collections.borrow(),
            vec!["col-a".to_string(), "col-b".to_string()]
        );
        assert_eq!(
            *store.last_fts_collections.borrow(),
            vec!["col-a".to_string(), "col-b".to_string()]
        );
    }

    #[test]
    fn kb1_no_collection_ids_returns_empty_without_touching_the_store() {
        let store = ScriptedStore {
            ann: vec![("leak".into(), 0.0)],
            chunks: vec![chunk("leak", "col-x")],
            ..Default::default()
        };
        let request = RetrievalRequest::new("q", vec![]);
        let results = retrieve(&store, &FakeEmbedder, &request).expect("retrieve");
        assert!(
            results.is_empty(),
            "KB-1: no implicit search-everything on an empty collection scope"
        );
        assert!(
            store.last_ann_collections.borrow().is_empty(),
            "the store must not even be called"
        );
    }

    #[test]
    fn kb6_every_result_carries_source_ref_attribution() {
        let store = ScriptedStore {
            ann: vec![("a".into(), 0.0)],
            chunks: vec![chunk("a", "col-1")],
            ..Default::default()
        };
        let request = RetrievalRequest::new("q", vec!["col-1".to_string()]);
        let results = retrieve(&store, &FakeEmbedder, &request).expect("retrieve");
        assert!(results[0].source_ref.is_some());
    }

    #[test]
    fn min_score_filters_out_low_ranked_fused_results() {
        let store = ScriptedStore {
            ann: vec![("a".into(), 0.0), ("b".into(), 0.0), ("c".into(), 0.0)],
            chunks: vec![
                chunk("a", "col-1"),
                chunk("b", "col-1"),
                chunk("c", "col-1"),
            ],
            ..Default::default()
        };
        let mut request = RetrievalRequest::new("q", vec!["col-1".to_string()]);
        request.top_k = 3;
        // Fused scores (ann-only, ranks 1/2/3): 1/61≈0.016393, 1/62≈0.016129,
        // 1/63≈0.015873. A floor of 0.0162 admits only rank 1 ("a").
        request.min_score = Some(0.0162);
        let results = retrieve(&store, &FakeEmbedder, &request).expect("retrieve");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk_id, "a");
    }

    #[test]
    fn embed_failure_surfaces_as_a_retrieval_error() {
        struct FailingEmbedder;
        impl EmbeddingBackend for FailingEmbedder {
            fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
                Err("no model".to_string())
            }
        }
        let store = ScriptedStore::default();
        let request = RetrievalRequest::new("q", vec!["col-1".to_string()]);
        let err = retrieve(&store, &FailingEmbedder, &request).expect_err("embed failure");
        assert!(matches!(err, RetrievalError::Embed(_)));
    }

    #[test]
    fn a_chunk_ranked_by_rrf_but_dropped_at_hydration_is_simply_absent() {
        // Simulates hydrate_chunks applying the KB-10 curation floor: "b" is
        // RRF-ranked but never returned by hydrate_chunks (as if filtered).
        let store = ScriptedStore {
            ann: vec![("a".into(), 0.0), ("b".into(), 0.0)],
            chunks: vec![chunk("a", "col-1")], // "b" deliberately absent
            ..Default::default()
        };
        let request = RetrievalRequest::new("q", vec!["col-1".to_string()]);
        let results = retrieve(&store, &FakeEmbedder, &request).expect("retrieve");
        let ids: Vec<&str> = results.iter().map(|c| c.chunk_id.as_str()).collect();
        assert_eq!(ids, vec!["a"]);
    }
}
