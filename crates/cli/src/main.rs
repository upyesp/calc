//! epher-cli — native command-line frontend (ADR-0001).
//!
//! One-shot evaluation when given an expression; interactive REPL otherwise. No
//! TUI/GUI is ever initiated from here. The REPL persists history,
//! user-defined functions, and the language preference through the shared
//! store (ADR-0002, ADR-0008), honoring the `EPHER_STORE_DIR` override (default
//! `~/.epher`).

use std::io::{self, BufRead, Write};

use epher_core::Session;
use epher_i18n::Localizer;
use epher_shell::{plain, run_command};
use epher_store::persist::{default_store_dir, load_language, load_session, save_history};
use epher_store::{DocStore, FsStore};
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
        Some(expr) => one_shot(&expr),
        None => repl(),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

/// Evaluate a single expression and print the result (no UI).
fn one_shot(expr: &str) -> Result<(), epher_core::EpherError> {
    let value = epher_core::evaluate(expr)?;
    println!("{value}");
    Ok(())
}

/// Interactive REPL: scripts run against a persistent environment; history,
/// saved functions, and the language preference survive restarts via the
/// shared store. The UI language is the store preference if set, else the
/// detected device locales (ADR-0008).
fn repl() -> Result<(), epher_core::EpherError> {
    let store = DocStore::new(FsStore::new(default_store_dir()));
    let mut session = match load_session(&store) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warning: could not load saved data ({e}); starting fresh");
            Session::new()
        }
    };
    let preference = load_language(&store).unwrap_or(None);
    let detected: Vec<String> = sys_locale::get_locales().collect();
    let mut localizer = Localizer::resolve(preference.as_deref(), &detected);

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        print!("{} ", localizer.lookup("prompt"));
        io::stdout().flush().map_err(|e| epher_core::EpherError::Io(e.to_string()))?;
        let Some(line) = lines.next() else { break }; // EOF
        let line = line.map_err(|e| epher_core::EpherError::Io(e.to_string()))?.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line == "quit" || line == "exit" {
            break;
        }
        if let Some(cmd) = epher_shell::classify(&line) {
            let handled = run_command(&cmd, &mut session, &store, &localizer);
            println!("{}", plain(handled.message));
            if let Some(code) = handled.language {
                localizer = Localizer::resolve(Some(&code), &[]);
            }
            continue;
        }
        let out = session.submit(&line);
        if !out.is_empty() {
            println!("{out}");
        }
        // best-effort persistence of history (atomic, last-write-wins)
        let _ = save_history(&store, session.history());
    }
    Ok(())
}
