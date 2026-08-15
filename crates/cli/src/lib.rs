//! epher-cli — native command-line frontend (ADR-0001).
//!
//! The library hosts every mode so the unified `epher` binary
//! (crates/tauri-app) can offer the same behavior without duplicating a
//! line:
//!
//! - [`run_one_shot`] — evaluate a single expression, print the result;
//! - [`run_repl`] — interactive REPL (prompt, persistent store);
//! - [`run_stdin`] — piped script mode (`epher -`): evaluate stdin line by
//!   line, no prompts, history untouched.
//!
//! All three share [`step`], the one-line-at-a-time seam that classifies a
//! line (shell command vs. language statement), runs it against the shared
//! session/store, and reports the printed output plus any language switch.

use std::io::{self, BufRead, Write};

use epher_core::{EpherError, Session};
use epher_i18n::Localizer;
use epher_shell::{classify, plain, run_command};
use epher_store::persist::{default_store_dir, load_language, load_session, save_history};
use epher_store::{DocStore, FsStore};

/// The outcome of processing one line: what to print (if anything) and a
/// language switch requested by a `lang` command (if any).
pub struct Step {
    pub output: Option<String>,
    pub language: Option<String>,
}

/// Process one input line against the session and store. Shell commands
/// (`save`, `lang`, …) run through epher-shell; anything else is a language
/// statement evaluated against the session. Errors come back as `error: …`
/// output — the session stays usable, exactly like the REPL.
pub fn step(session: &mut Session, store: &DocStore<FsStore>, localizer: &Localizer, line: &str) -> Step {
    if let Some(cmd) = classify(line) {
        let handled = run_command(&cmd, session, store, localizer);
        return Step {
            output: Some(plain(handled.message)),
            language: handled.language,
        };
    }
    let out = session.submit(line);
    Step {
        output: if out.is_empty() { None } else { Some(out) },
        language: None,
    }
}

/// Open the shared native store (ADR-0002): `EPHER_STORE_DIR` override,
/// else `~/.epher`, and load the saved session (functions, scripts,
/// history) — warning and starting fresh if the store is unreadable.
fn open_store_with_session() -> (DocStore<FsStore>, Session, Localizer) {
    let store = DocStore::new(FsStore::new(default_store_dir()));
    let session = match load_session(&store) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warning: could not load saved data ({e}); starting fresh");
            Session::new()
        }
    };
    let preference = load_language(&store).unwrap_or(None);
    let detected: Vec<String> = sys_locale::get_locales().collect();
    let localizer = Localizer::resolve(preference.as_deref(), &detected);
    (store, session, localizer)
}

/// Evaluate a single expression and print the result (no UI, no store).
pub fn run_one_shot(expr: &str) -> Result<(), EpherError> {
    let value = epher_core::evaluate(expr)?;
    println!("{value}");
    Ok(())
}

/// Interactive REPL: scripts run against a persistent environment; history,
/// saved functions, and the language preference survive restarts via the
/// shared store. The UI language is the store preference if set, else the
/// detected device locales (ADR-0008).
pub fn run_repl() -> Result<(), EpherError> {
    let (store, mut session, mut localizer) = open_store_with_session();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        print!("{} ", localizer.lookup("prompt"));
        io::stdout().flush().map_err(|e| EpherError::Io(e.to_string()))?;
        let Some(line) = lines.next() else { break }; // EOF
        let line = line.map_err(|e| EpherError::Io(e.to_string()))?.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line == "quit" || line == "exit" {
            break;
        }
        let out = step(&mut session, &store, &localizer, &line);
        if let Some(text) = out.output {
            println!("{text}");
        }
        if let Some(code) = out.language {
            localizer = Localizer::resolve(Some(&code), &[]);
        }
        // best-effort persistence of history (atomic, last-write-wins)
        let _ = save_history(&store, session.history());
    }
    Ok(())
}

/// Piped script mode (`epher -`): evaluate stdin line by line, printing each
/// result. Sessions load from (and `save` commands write to) the shared
/// store, but interactive history is not written — scripts are not
/// interactive pasts. Errors print and evaluation continues, like the REPL.
pub fn run_stdin() -> Result<(), EpherError> {
    let stdin = io::stdin();
    run_stdin_from(stdin.lock())
}

/// The testable core of [`run_stdin`]: any line-oriented reader.
pub fn run_stdin_from<R: BufRead>(input: R) -> Result<(), EpherError> {
    let (store, mut session, localizer) = open_store_with_session();
    for line in input.lines() {
        let line = line.map_err(|e| EpherError::Io(e.to_string()))?.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let out = step(&mut session, &store, &localizer, &line);
        if let Some(text) = out.output {
            println!("{text}");
        }
    }
    Ok(())
}
