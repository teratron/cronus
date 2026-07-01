//! Benchmarks for the codegraph indexing hot paths: symbol extraction over a
//! source file, and reciprocal-rank fusion over ranked candidate lists.
//!
//! Both are pure/CPU-bound (no database), so they measure the parts of
//! indexing and hybrid search that dominate wall-clock at scale. Std-only
//! harness (`harness = false`), stable toolchain. Invoke with
//! `cargo bench -p codegraph --bench extraction`.

use std::hint::black_box;
use std::time::Instant;

use codegraph::extractor::{Extractor, RegexExtractor};
use codegraph::search::rrf_merge;

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

/// Build a synthetic Rust source with ~1200 extractable symbols.
fn synthetic_source() -> String {
    let mut src = String::new();
    for i in 0..1_000 {
        src.push_str(&format!(
            "pub fn function_{i}(x: i32) -> i32 {{ x + {i} }}\n"
        ));
        if i % 5 == 0 {
            src.push_str(&format!("pub struct Struct{i} {{ field: u32 }}\n"));
        }
        if i % 7 == 0 {
            src.push_str(&format!("pub enum Enum{i} {{ A, B }}\n"));
        }
    }
    src
}

fn main() {
    println!("== codegraph ==");

    let source = synthetic_source();
    let extractor = RegexExtractor;
    bench("codegraph/extract(~1200 symbols)", 2_000, || {
        let symbols = extractor.extract(black_box(&source));
        black_box(symbols);
    });

    // Two overlapping ranked lists, as hybrid keyword + vector recall produces.
    let keyword: Vec<i64> = (0..500).collect();
    let vector: Vec<i64> = (250..750).rev().collect();
    let lists = vec![keyword, vector];
    bench("codegraph/rrf_merge(2x500)", 20_000, || {
        let fused = rrf_merge(black_box(&lists));
        black_box(fused);
    });
}
