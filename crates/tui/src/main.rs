//! Cronus TUI binary — thin entry point. All behavior lives in the library and,
//! ultimately, the core; this binary only launches the render loop and reports a
//! fatal error if the terminal could not be driven.

fn main() {
    if let Err(error) = cronus_tui::run() {
        eprintln!("[cronus-tui] fatal: {error}");
        std::process::exit(1);
    }
}
