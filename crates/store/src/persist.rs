//! Persistence helpers shared by the native frontends (CLI/TUI/desktop):
//! loading a [`Session`] from the store and saving history/functions. Logic
//! once — the web frontend uses the same [`DocStore`] seam with its own
//! backend.

use crate::{DocStore, FunctionDoc, ScriptDoc, Storage, StoreError, StoreResult};
use epher_core::Session;

pub const HISTORY_SETTING: &str = "history";
/// The user's language override (ADR-0008): detection is the default, this
/// setting wins when set.
pub const LANGUAGE_SETTING: &str = "language";

/// The store directory for native frontends: `EPHER_STORE_DIR` override, else
/// `~/.epher` (falls back to `.epher`).
pub fn default_store_dir() -> std::path::PathBuf {
    std::env::var_os("EPHER_STORE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(|h| std::path::PathBuf::from(h).join(".epher"))
                .unwrap_or_else(|| std::path::PathBuf::from(".epher"))
        })
}

/// The saved lines to replay at startup, in load order: functions first,
/// then scripts (the recipe [`load_session`] applies natively; the desktop
/// webview replays them into its own Session — ADR-0010).
pub fn replay_lines<S: Storage>(store: &DocStore<S>) -> StoreResult<Vec<String>> {
    let mut lines = Vec::new();
    for doc in store.list_functions()? {
        lines.push(doc.source);
    }
    for doc in store.list_scripts()? {
        lines.push(doc.source);
    }
    Ok(lines)
}

/// Rebuild a session from the store: history plus saved functions and scripts
/// (re-run as definitions).
pub fn load_session<S: Storage>(store: &DocStore<S>) -> StoreResult<Session> {
    let history = history(store)?;
    let mut session = Session::with_history(history);
    for line in replay_lines(store)? {
        session.submit_quiet(&line);
    }
    Ok(session)
}

pub fn history<S: Storage>(store: &DocStore<S>) -> StoreResult<Vec<String>> {
    match store.get_setting(HISTORY_SETTING)? {
        Some(value) => serde_json::from_value(value)
            .map_err(|e| StoreError::Serialize(e.to_string())),
        None => Ok(Vec::new()),
    }
}

pub fn save_history<S: Storage>(store: &DocStore<S>, history: &[String]) -> StoreResult<()> {
    let value =
        serde_json::to_value(history).map_err(|e| StoreError::Serialize(e.to_string()))?;
    store.set_setting(HISTORY_SETTING, value)
}

pub fn save_function<S: Storage>(
    store: &DocStore<S>,
    name: &str,
    source: &str,
) -> StoreResult<()> {
    store.put_function(&FunctionDoc {
        name: name.to_string(),
        source: source.to_string(),
    })
}

pub fn save_script<S: Storage>(store: &DocStore<S>, name: &str, source: &str) -> StoreResult<()> {
    store.put_script(&ScriptDoc {
        name: name.to_string(),
        source: source.to_string(),
    })
}

pub fn load_language<S: Storage>(store: &DocStore<S>) -> StoreResult<Option<String>> {
    match store.get_setting(LANGUAGE_SETTING)? {
        Some(value) => Ok(value.as_str().map(String::from)),
        None => Ok(None),
    }
}

pub fn save_language<S: Storage>(store: &DocStore<S>, language: &str) -> StoreResult<()> {
    store.set_setting(LANGUAGE_SETTING, serde_json::json!(language))
}
