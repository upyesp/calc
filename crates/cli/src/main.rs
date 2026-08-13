//! calc-cli — native command-line frontend (ADR-0001).
//!
//! One-shot evaluation when given an expression; interactive REPL otherwise. No
//! TUI/GUI is ever initiated from here. The REPL persists history,
//! user-defined functions, and the language preference through the shared
//! store (ADR-0002, ADR-0008), honoring the `CALC_STORE_DIR` override (default
//! `~/.calc`).

use std::io::{self, BufRead, Write};

use calc_core::Session;
use calc_i18n::Localizer;
use calc_store::persist::{
    default_store_dir, load_language, load_session, save_function, save_history, save_language,
};
use calc_store::{DocStore, FsStore};
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

/// Interactive REPL: scripts run against a persistent environment; history,
/// saved functions, and the language preference survive restarts via the
/// shared store. The UI language is the store preference if set, else the
/// detected device locales (ADR-0008).
fn repl() -> Result<(), calc_core::CalcError> {
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
        io::stdout().flush().map_err(|e| calc_core::CalcError::Io(e.to_string()))?;
        let Some(line) = lines.next() else { break }; // EOF
        let line = line.map_err(|e| calc_core::CalcError::Io(e.to_string()))?.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line == "quit" || line == "exit" {
            break;
        }
        if let Some(code) = line.strip_prefix("language ") {
            let code = code.trim();
            if calc_i18n::SUPPORTED_LOCALES.contains(&code) {
                match save_language(&store, code) {
                    Ok(()) => {
                        localizer = Localizer::resolve(Some(code), &[]);
                        println!(
                            "{}",
                            strip(localizer.lookup_args("language-set", &[("code", code)]))
                        );
                    }
                    Err(e) => println!("error: {e}"),
                }
            } else {
                println!(
                    "{}",
                    strip(localizer.lookup_args(
                        "unsupported-language",
                        &[("code", code), ("supported", &calc_i18n::SUPPORTED_LOCALES.join(", "))]
                    ))
                );
            }
            continue;
        }
        if let Some(name) = line.strip_prefix("save ") {
            let name = name.trim();
            match session.def_sources().get(name) {
                Some(source) => match save_function(&store, name, source) {
                    Ok(()) => println!("{}", strip(localizer.lookup_args("saved", &[("name", name)]))),
                    Err(e) => println!("error: {e}"),
                },
                None => println!(
                    "{}",
                    strip(localizer.lookup_args("no-definition", &[("name", name)]))
                ),
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

/// Fluent wraps interpolated values in bidi isolating characters for RTL
/// safety; strip them for terminal display (terminals don't need the
/// protection and the isolates render as invisible-but-annoying gaps).
fn strip(s: String) -> String {
    s.chars().filter(|c| *c != '\u{2068}' && *c != '\u{2069}').collect()
}
