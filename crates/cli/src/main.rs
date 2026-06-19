//! Cronus CLI — a thin frontend over the core. No domain logic lives here;
//! it maps commands to core capabilities and renders the result.

use cronus_core::{Capabilities, Engine};

fn main() {
    let engine = Engine::new();
    // Placeholder command surface until the command framework lands (Phase 3).
    println!("{}", engine.status());
}
