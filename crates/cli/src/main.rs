//! calc-cli — native command-line frontend (ADR-0001).
//!
//! One-shot evaluation when given an expression; interactive REPL otherwise. No
//! TUI/GUI is ever initiated from here.

use clap::Parser;

/// Calc: a programmable, scriptable calculator.
#[derive(Parser, Debug)]
#[command(name = "calc", version, about)]
struct Cli {
    /// An expression to evaluate. If omitted, starts an interactive REPL.
    expression: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    // TODO: wire to the calc-core evaluator + calc-i18n; one-shot vs REPL (Q16).
    match cli.expression {
        Some(expr) => println!("calc: evaluate {expr:?}"),
        None => println!("calc: interactive REPL (TODO)"),
    }
}
