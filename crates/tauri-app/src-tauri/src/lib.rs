//! app_lib — the Tauri desktop shell (ADR-0001, ADR-0010).
//!
//! The native process owns the Native Store: a `DocStore<FsStore>` rooted
//! at `default_store_dir()` (`EPHER_STORE_DIR` override, `~/.epher` default) —
//! the same files the CLI and TUI use. The webview bridges to it through
//! five IPC commands, all thin wrappers over epher-store's persist helpers;
//! evaluation itself stays in the webview on the wasm core.

use std::path::PathBuf;

use epher_store::persist;
use epher_store::{DocStore, FsStore};
use serde::Serialize;
use tauri::State;

/// The desktop's native store: one instance, managed by Tauri and shared by
/// every command.
pub struct DesktopStore {
    store: DocStore<FsStore>,
}

impl DesktopStore {
    pub fn with_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            store: DocStore::new(FsStore::new(dir)),
        }
    }

    /// Everything the webview needs at startup: history, the replay lines
    /// (functions, then scripts), and the language preference.
    pub fn init(&self) -> epher_store::StoreResult<InitState> {
        Ok(InitState {
            history: persist::history(&self.store)?,
            replay: persist::replay_lines(&self.store)?,
            language: persist::load_language(&self.store)?,
        })
    }

    pub fn save_function(&self, name: &str, source: &str) -> epher_store::StoreResult<()> {
        persist::save_function(&self.store, name, source)
    }

    pub fn save_script(&self, name: &str, source: &str) -> epher_store::StoreResult<()> {
        persist::save_script(&self.store, name, source)
    }

    pub fn save_history(&self, history: &[String]) -> epher_store::StoreResult<()> {
        persist::save_history(&self.store, history)
    }

    pub fn save_language(&self, language: &str) -> epher_store::StoreResult<()> {
        persist::save_language(&self.store, language)
    }
}

/// The answer to `init`: the store's contents as plain data, so the webview
/// can rebuild its Session exactly like `load_session` does natively.
#[derive(Debug, Serialize)]
pub struct InitState {
    pub history: Vec<String>,
    pub replay: Vec<String>,
    pub language: Option<String>,
}

#[tauri::command]
fn init(state: State<DesktopStore>) -> Result<InitState, String> {
    state.init().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_function(state: State<DesktopStore>, name: String, source: String) -> Result<(), String> {
    state.save_function(&name, &source).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_script(state: State<DesktopStore>, name: String, source: String) -> Result<(), String> {
    state.save_script(&name, &source).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_history(state: State<DesktopStore>, history: Vec<String>) -> Result<(), String> {
    state.save_history(&history).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_language(state: State<DesktopStore>, code: String) -> Result<(), String> {
    state.save_language(&code).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DesktopStore::with_dir(persist::default_store_dir()))
        .invoke_handler(tauri::generate_handler![
            init,
            save_function,
            save_script,
            save_history,
            save_language
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use epher_store::persist::load_session;

    #[test]
    fn init_reports_what_the_cli_would_load() {
        let dir = tempfile::tempdir().unwrap();
        let desktop = DesktopStore::with_dir(dir.path());
        desktop
            .save_function("fib", "def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)")
            .unwrap();
        desktop
            .save_script("count", "x = 0; while x < 5 do x = x + 1; x")
            .unwrap();
        desktop.save_history(&["2 + 3  = 5".to_string()]).unwrap();
        desktop.save_language("fr").unwrap();

        let state = desktop.init().unwrap();
        assert_eq!(state.history, vec!["2 + 3  = 5".to_string()]);
        assert_eq!(state.language, Some("fr".to_string()));
        assert_eq!(state.replay.len(), 2);
        assert!(state.replay[0].starts_with("def fib"));
        assert!(state.replay[1].starts_with("x = 0"));
    }

    #[test]
    fn the_cli_loads_what_the_desktop_saved() {
        // The whole point (ADR-0010): the same files. The CLI's own startup
        // path must see the desktop's writes — function *and* variables set
        // by a saved script.
        let dir = tempfile::tempdir().unwrap();
        let desktop = DesktopStore::with_dir(dir.path());
        desktop.save_function("f", "def f(x) = x ^ 2").unwrap();
        desktop.save_script("vars", "y = 7").unwrap();

        let mut session = load_session(&DocStore::new(FsStore::new(dir.path()))).unwrap();
        assert!(session.def_sources().contains_key("f"));
        assert_eq!(session.submit("f(3) + y"), "= 16");
    }

    #[test]
    fn init_on_an_empty_store_is_empty_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let state = DesktopStore::with_dir(dir.path()).init().unwrap();
        assert!(state.history.is_empty());
        assert!(state.replay.is_empty());
        assert_eq!(state.language, None);
    }
}
