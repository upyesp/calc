//! calc-cli — native command-line frontend (ADR-0001).
//!
//! One-shot evaluation when given an expression; interactive REPL otherwise. No
//! TUI/GUI is ever initiated from here.

use std::io::{self, BufRead, Write};

use calc_core::Session;
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
    let result = match cli.expression {
        Some(expr) => one_shot(&expr),
        None => repl(),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

/// Evaluate a single expression and print the result (no UI).
fn one_shot(expr: &str) -> Result<(), calc_core::CalcError> {
    let value = calc_core::evaluate(expr)?;
    println!("{value}");
    Ok(())
}

/// Interactive REPL: scripts run against a persistent environment.
fn repl() -> Result<(), calc_core::CalcError> {
    let mut session = Session::new();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        print!("calc> ");
        io::stdout().flush().map_err(|e| calc_core::CalcError::Io(e.to_string()))?;
        let Some(line) = lines.next() else { break }; // EOF
        let line = line.map_err(|e| calc_core::CalcError::Io(e.to_string()))?.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line == "quit" || line == "exit" {
            break;
        }
        println!("{}", session.submit(&line));
    }
    Ok(())
}
