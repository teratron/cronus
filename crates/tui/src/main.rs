//! Cronus TUI — a thin terminal frontend over the core. No domain logic here;
//! the interactive terminal surface is built in Phase 7 (Leaf).

use cronus_core::{Capabilities, Engine};

fn main() {
    let engine = Engine::new();
    println!("[cronus-tui] {}", engine.status());
}
