//! Search — RRF fusion of keyword (FTS5) and vector results.

use crate::index::IndexedSymbol;
use std::collections::HashMap;

/// Reciprocal Rank Fusion constant.
const RRF_K: f64 = 60.0;

/// A ranked search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub symbol: IndexedSymbol,
    /// Combined RRF score (higher = more relevant).
    pub score: f64,
}

/// A ranked list of symbol IDs from a single search modality.
pub type RankedList = Vec<i64>;

/// Merge two ranked lists with RRF.
///
/// `lists[i]` is a `RankedList` (ordered by relevance, best first).
/// Returns deduplicated results ordered by descending RRF score.
pub fn rrf_merge(lists: &[RankedList]) -> Vec<(i64, f64)> {
    let mut scores: HashMap<i64, f64> = HashMap::new();
    for list in lists {
        for (rank, &id) in list.iter().enumerate() {
            let contribution = 1.0 / (RRF_K + rank as f64 + 1.0);
            *scores.entry(id).or_default() += contribution;
        }
    }
    let mut result: Vec<(i64, f64)> = scores.into_iter().collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

/// Fuse keyword and vector ranked lists, then look up symbols from `candidates`.
///
/// `keyword_ids`: FTS5 results ordered by relevance (best first).
/// `vector_ids`: vector similarity results ordered by similarity (best first).
/// `candidates`: all symbols that appeared in either list, keyed by id.
pub fn fuse(
    keyword_ids: RankedList,
    vector_ids: RankedList,
    candidates: &HashMap<i64, IndexedSymbol>,
) -> Vec<SearchResult> {
    let merged = rrf_merge(&[keyword_ids, vector_ids]);
    merged
        .into_iter()
        .filter_map(|(id, score)| {
            candidates.get(&id).map(|sym| SearchResult {
                symbol: sym.clone(),
                score,
            })
        })
        .collect()
}
