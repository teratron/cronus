//! Latency benchmark for the model router's provider-selection hot path.
//!
//! Every routed request scores the full provider pool; this measures that
//! decision under a realistic pool size. Std-only harness (`harness = false`),
//! stable toolchain. Invoke with
//! `cargo bench -p cronus --bench model_router`.

use std::hint::black_box;
use std::time::Instant;

use cronus::router::provider::{
    ModelProvider, ProviderHealth, ProviderTier, RoutingRequest, TaskType,
};
use cronus::router::{RouterMode, RouterPool};

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

/// Minimal healthy provider with a fixed task-fit — enough to exercise scoring.
struct BenchProvider {
    id: &'static str,
    fit: f64,
    latency_ms: u64,
    cost: f64,
}

impl ModelProvider for BenchProvider {
    fn id(&self) -> &str {
        self.id
    }
    fn health(&self) -> ProviderHealth {
        ProviderHealth::Healthy
    }
    fn context_window(&self) -> u32 {
        128_000
    }
    fn cost_per_1k_tokens(&self) -> f64 {
        self.cost
    }
    fn latency_p50_ms(&self) -> u64 {
        self.latency_ms
    }
    fn tier(&self) -> ProviderTier {
        ProviderTier::Standard
    }
    fn task_fit(&self, _task: TaskType) -> f64 {
        self.fit
    }
}

fn main() {
    println!("== model_router ==");

    const IDS: [&str; 8] = ["p0", "p1", "p2", "p3", "p4", "p5", "p6", "p7"];

    let pool = RouterPool::new(RouterMode::Quality);
    for (i, id) in IDS.iter().enumerate() {
        pool.register(Box::new(BenchProvider {
            id,
            fit: 0.5 + (i as f64) * 0.05,
            latency_ms: 200 + (i as u64) * 25,
            cost: 0.005 + (i as f64) * 0.002,
        }));
    }

    let req = RoutingRequest {
        prompt_hash: 42,
        required_context: 1_000,
        task_type: TaskType::Chat,
    };

    bench("model_router/route(8 providers)", 50_000, || {
        let decision = pool.route(black_box(&req)).expect("route request");
        black_box(decision);
    });
}
