//! Symbol deduplication — entropy gate + Jaro-Winkler similarity.

// ── Entropy gate ──────────────────────────────────────────────────────────────

/// Low-entropy names that bypass deduplication.
const LOW_ENTROPY: &[&str] = &[
    "new", "init", "default", "clone", "drop", "fmt", "eq", "hash",
    "get", "set", "len", "is_empty", "iter", "next", "into", "from",
];

/// Returns true when the name has too low entropy to be deduplicated safely.
pub fn is_low_entropy(name: &str) -> bool {
    LOW_ENTROPY.contains(&name)
}

// ── Jaro-Winkler ─────────────────────────────────────────────────────────────

/// Compute the Jaro similarity between two strings.
fn jaro(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }
    let s1: Vec<char> = s1.chars().collect();
    let s2: Vec<char> = s2.chars().collect();
    let len1 = s1.len();
    let len2 = s2.len();
    if len1 == 0 || len2 == 0 {
        return 0.0;
    }
    let match_dist = (len1.max(len2) / 2).saturating_sub(1);
    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];
    let mut matches = 0usize;
    let mut transpositions = 0usize;

    for i in 0..len1 {
        let start = i.saturating_sub(match_dist);
        let end = (i + match_dist + 1).min(len2);
        for j in start..end {
            if s2_matches[j] || s1[i] != s2[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }
    if matches == 0 {
        return 0.0;
    }

    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1[i] != s2[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let m = matches as f64;
    let t = (transpositions / 2) as f64;
    (m / len1 as f64 + m / len2 as f64 + (m - t) / m) / 3.0
}

/// Compute Jaro-Winkler similarity (prefix weight `p = 0.1`, max prefix `4`).
pub fn jaro_winkler(s1: &str, s2: &str) -> f64 {
    let j = jaro(s1, s2);
    let s1c: Vec<char> = s1.chars().collect();
    let s2c: Vec<char> = s2.chars().collect();
    let prefix = s1c
        .iter()
        .zip(s2c.iter())
        .take(4)
        .take_while(|(a, b)| a == b)
        .count() as f64;
    j + prefix * 0.1 * (1.0 - j)
}

// ── Deduplication gate ────────────────────────────────────────────────────────

/// Default similarity threshold above which two symbols are considered duplicates.
pub const DEFAULT_SIMILARITY_THRESHOLD: f64 = 0.90;

/// Returns true when `a` and `b` should be considered the same symbol.
///
/// Bypasses for low-entropy names and enforces the similarity threshold.
pub fn should_merge(a: &str, b: &str, threshold: f64) -> bool {
    if a == b {
        return true;
    }
    if is_low_entropy(a) || is_low_entropy(b) {
        return false;
    }
    jaro_winkler(a, b) >= threshold
}
