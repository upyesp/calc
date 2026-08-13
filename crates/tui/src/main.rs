//! calc-tui — native full-screen terminal frontend (ADR-0001).
//!
//! Runs natively, so ratatui + crossterm have full terminal access (raw mode,
//! size, colors) with no WASI host required. No GUI/browser is ever initiated.

fn main() {
    // TODO: ratatui event loop wired to calc-core + calc-store + calc-i18n.
    println!("calc-tui: TODO");
}
