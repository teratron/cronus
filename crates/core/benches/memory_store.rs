//! Throughput benchmarks for the memory store — the persistence hot path that
//! every recall and consolidation cycle passes through.
//!
//! Runs on the stable toolchain via a std-only timing harness (`harness =
//! false`); no external benchmark crate is pulled in. Invoke with
//! `cargo bench -p cronus --bench memory_store`.

use std::hint::black_box;
use std::time::Instant;

use cronus::memory::{MemoryEntry, MemoryKind, MemorySource, MemoryStore};

/// Time `iters` calls of `f`, discarding a short warm-up, and report ns/op.
fn bench(label: &str, iters: u32, mut f: impl FnMut()) {
    for _ in 0..(iters / 10).max(1) {
        f();
    }
    let start = Instant::now();
    for _ in 0..iters {
        f();
    }
    let elapsed = start.elapsed();
    let per_op = elapsed.as_nanos() as f64 / f64::from(iters);
    println!("{label:<38} {iters:>7} iters  {per_op:>11.1} ns/op  ({elapsed:?} total)");
}

fn sample_entry(i: u32) -> MemoryEntry {
    MemoryEntry::new(
        MemoryKind::ProjectContext,
        MemorySource::Agent,
        format!("entry {i}"),
        format!("body number {i} about routing, memory search and consolidation"),
    )
}

fn main() {
    println!("== memory_store ==");

    // Insert throughput into a fresh in-memory database.
    let insert_store = MemoryStore::open_in_memory().expect("open in-memory store");
    let mut counter = 0u32;
    bench("memory_store/add", 5_000, || {
        counter += 1;
        let id = insert_store
            .add(black_box(sample_entry(counter)))
            .expect("add entry");
        black_box(id);
    });

    // Full-text search over a pre-populated store.
    let search_store = MemoryStore::open_in_memory().expect("open in-memory store");
    for i in 0..2_000 {
        search_store.add(sample_entry(i)).expect("seed entry");
    }
    bench("memory_store/search_fts(limit=10)", 2_000, || {
        let hits = search_store
            .search_fts(black_box("routing"), 10)
            .expect("fts search");
        black_box(hits);
    });
}
