//! Persistence helpers shared by the native frontends (CLI/TUI/desktop):
//! loading a [`Session`] from the store and saving history/functions. Logic
//! once — the web frontend uses the same [`DocStore`] seam with its own
//! backend.

use crate::{DocStore, FunctionDoc, ScriptDoc, Storage, StoreError, StoreResult};
use calc_core::Session;

pub const HISTORY_SETTING: &str = "history";

/// The store directory for native frontends: `CALC_STORE_DIR` override, else
/// `~/.calc` (falls back to `.calc`).
pub fn default_store_dir() -> std::path::PathBuf {
    std::env::var_os("CALC_STORE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(|h| std::path::PathBuf::from(h).join(".calc"))
                .unwrap_or_else(|| std::path::PathBuf::from(".calc"))
        })
}

/// Rebuild a session from the store: history plus saved functions and scripts
/// (re-run as definitions).
pub fn load_session<S: Storage>(store: &DocStore<S>) -> StoreResult<Session> {
    let history = history(store)?;
    let mut session = Session::with_history(history);
    for doc in store.list_functions()? {
        session.submit_quiet(&doc.source);
    }
    for doc in store.list_scripts()? {
        session.submit_quiet(&doc.source);
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
