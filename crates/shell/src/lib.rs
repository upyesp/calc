//! epher-shell — the interactive-shell kernel shared by the CLI, TUI, and web
//! frontends (ADR-0010).
//!
//! One policy for shell commands: [`classify`] recognizes `save`,
//! `save script`, and `language` lines; [`prepare`] resolves them against
//! the session (validation and source lookups); [`run_command`] additionally
//! persists through the store for native shells. The webview reuses
//! classify/prepare and persists through its IPC bridge instead.

use epher_core::Session;
use epher_i18n::Localizer;
use epher_store::persist;
use epher_store::{DocStore, Storage};

/// A shell command recognized in an input line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    SaveFunction { name: String },
    SaveScript { name: String },
    Language { code: String },
}

/// A command resolved against the session, ready to persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prepared {
    SaveFunction { name: String, source: String },
    SaveScript { name: String, source: String },
    Language { code: String },
}

/// Recognize a shell command in an input line. Anything else (including
/// `save` or `language` without an argument) is `None` — the caller
/// evaluates it, exactly as the CLI always has.
pub fn classify(line: &str) -> Option<Command> {
    let line = line.trim();
    // Order matters: "save script " must win over the shorter "save ".
    if let Some(name) = line.strip_prefix("save script ") {
        let name = name.trim();
        if !name.is_empty() {
            return Some(Command::SaveScript { name: name.to_string() });
        }
        return None;
    }
    if let Some(name) = line.strip_prefix("save ") {
        let name = name.trim();
        if !name.is_empty() {
            return Some(Command::SaveFunction { name: name.to_string() });
        }
        return None;
    }
    if let Some(code) = line.strip_prefix("language ") {
        let code = code.trim();
        if !code.is_empty() {
            return Some(Command::Language { code: code.to_string() });
        }
        return None;
    }
    None
}

/// A `last_line` qualifies as a savable script only if it was a real
/// evaluation, not another shell command.
fn savable(source: &str) -> bool {
    !source.starts_with("save") && !source.starts_with("language") && !source.starts_with("quit")
}

/// Resolve a command against the session: validation and source lookups.
/// `Err` carries the localized message to show the user.
pub fn prepare(cmd: &Command, session: &Session, localizer: &Localizer) -> Result<Prepared, String> {
    match cmd {
        Command::SaveFunction { name } => match session.def_sources().get(name) {
            Some(source) => Ok(Prepared::SaveFunction {
                name: name.clone(),
                source: source.clone(),
            }),
            None => Err(localizer.lookup_args("no-definition", &[("name", name)])),
        },
        Command::SaveScript { name } => match session.last_line() {
            Some(source) if savable(source) => Ok(Prepared::SaveScript {
                name: name.clone(),
                source: source.to_string(),
            }),
            _ => Err(localizer.lookup("nothing-to-save")),
        },
        Command::Language { code } => {
            if epher_i18n::SUPPORTED_LOCALES.contains(&code.as_str()) {
                Ok(Prepared::Language { code: code.clone() })
            } else {
                Err(localizer.lookup_args(
                    "unsupported-language",
                    &[
                        ("code", code),
                        ("supported", &epher_i18n::SUPPORTED_LOCALES.join(", ")),
                    ],
                ))
            }
        }
    }
}

/// The localized success message for a prepared command.
pub fn message(prepared: &Prepared, localizer: &Localizer) -> String {
    match prepared {
        Prepared::SaveFunction { name, .. } => localizer.lookup_args("saved", &[("name", name)]),
        Prepared::SaveScript { name, .. } => {
            localizer.lookup_args("saved-script", &[("name", name)])
        }
        Prepared::Language { code, .. } => {
            localizer.lookup_args("language-set", &[("code", code)])
        }
    }
}

/// Strip the bidi isolating characters Fluent wraps around interpolated
/// values (U+2068/U+2069). Browsers want them (they keep RTL fragments
/// readable); terminals render them as invisible-but-annoying gaps, so the
/// CLI and TUI pass every message through here.
pub fn plain(message: String) -> String {
    message
        .chars()
        .filter(|c| *c != '\u{2068}' && *c != '\u{2069}')
        .collect()
}

/// The outcome of handling a command: the message to show, plus the new
/// language preference when it changed (shells re-resolve their Localizer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handled {
    pub message: String,
    pub language: Option<String>,
}

/// Handle a command for a native shell: prepare, persist, and answer with
/// the localized message (or the prepare error, without touching the store).
pub fn run_command<S: Storage>(
    cmd: &Command,
    session: &mut Session,
    store: &DocStore<S>,
    localizer: &Localizer,
) -> Handled {
    let prepared = match prepare(cmd, session, localizer) {
        Ok(p) => p,
        Err(msg) => return Handled { message: msg, language: None },
    };
    let result = match &prepared {
        Prepared::SaveFunction { name, source } => persist::save_function(store, name, source),
        Prepared::SaveScript { name, source } => persist::save_script(store, name, source),
        Prepared::Language { code } => persist::save_language(store, code),
    };
    match result {
        Ok(()) => {
            let language = if let Prepared::Language { code } = &prepared {
                Some(code.clone())
            } else {
                None
            };
            Handled { message: message(&prepared, localizer), language }
        }
        Err(e) => Handled { message: format!("error: {e}"), language: None },
    }
}
