//! Community detection — Leiden algorithm seam with a union-find stub.
//!
//! The real Leiden implementation wires in Phase 5. At Phase 4, connected
//! symbols are grouped by a simple union-find on explicit edges.

use std::collections::HashMap;

// ── Edge ─────────────────────────────────────────────────────────────────────

/// A directed edge between two symbols.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: i64,
    pub to: i64,
}

// ── Community ─────────────────────────────────────────────────────────────────

/// A community of related symbols.
#[derive(Debug, Clone)]
pub struct Community {
    pub id: u64,
    pub members: Vec<i64>,
}

// ── CommunityDetector seam ────────────────────────────────────────────────────

/// Seam trait for community detection (Leiden in Phase 5).
pub trait CommunityDetector: Send + Sync {
    fn detect(&self, nodes: &[i64], edges: &[Edge]) -> Vec<Community>;
}

// ── UnionFindDetector (Phase 4 stub) ─────────────────────────────────────────

/// Union-find community detection — groups connected components.
pub struct UnionFindDetector;

impl CommunityDetector for UnionFindDetector {
    fn detect(&self, nodes: &[i64], edges: &[Edge]) -> Vec<Community> {
        let mut parent: HashMap<i64, i64> = nodes.iter().map(|&n| (n, n)).collect();

        fn find(parent: &mut HashMap<i64, i64>, x: i64) -> i64 {
            if parent[&x] == x {
                return x;
            }
            let p = find(parent, parent[&x]);
            parent.insert(x, p);
            p
        }

        for edge in edges {
            let ra = find(&mut parent, edge.from);
            let rb = find(&mut parent, edge.to);
            if ra != rb {
                parent.insert(ra, rb);
            }
        }

        // Collect members by root.
        let mut communities: HashMap<i64, Vec<i64>> = HashMap::new();
        for &node in nodes {
            let root = find(&mut parent, node);
            communities.entry(root).or_default().push(node);
        }

        // Stable ordering by smallest member.
        let mut result: Vec<Community> = communities
            .into_values()
            .enumerate()
            .map(|(i, mut members)| {
                members.sort_unstable();
                Community {
                    id: i as u64,
                    members,
                }
            })
            .collect();
        result.sort_by_key(|c| c.members.first().copied().unwrap_or(0));
        // Re-number stably.
        for (i, c) in result.iter_mut().enumerate() {
            c.id = i as u64;
        }
        result
    }
}
