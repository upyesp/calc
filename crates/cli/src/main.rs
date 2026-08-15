//! epher-cli — native command-line frontend (ADR-0001).
//!
//! A thin binary wrapper over the library: one-shot evaluation when given
//! an expression; interactive REPL otherwise. The unified `epher` binary
//! (crates/tauri-app) hosts the same modes — plus `tui`, `gui`, and `epher -`
//! for piped input — by calling the library directly.

use clap::Parser;

/// epher: a programmable, scriptable calculator.
#[derive(Parser, Debug)]
#[command(name = "epher", version, about)]
struct Cli {
    /// An expression to evaluate. If omitted, starts an interactive REPL.
    expression: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.expression {
        Some(expr) => epher_cli::run_one_shot(&expr),
        None => epher_cli::run_repl(),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
