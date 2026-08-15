//! app_lib — the Tauri desktop shell (ADR-0001, ADR-0010).
//!
//! The native process owns the Native Store: a `DocStore<FsStore>` rooted
//! at `default_store_dir()` (`EPHER_STORE_DIR` override, `~/.epher` default) —
//! the same files the CLI and TUI use. The webview bridges to it through
//! five IPC commands, all thin wrappers over epher-store's persist helpers;
//! evaluation itself stays in the webview on the wasm core.

use std::path::PathBuf;

use clap::Parser;
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

pub mod cli_install;
pub mod dispatch;

#[tauri::command]
fn save_language(state: State<DesktopStore>, code: String) -> Result<(), String> {
    state.save_language(&code).map_err(|e| e.to_string())
}

/// Can this shell install the `epher` terminal command? (macOS app bundle
/// only — see cli_install.) The webview asks at startup to decide whether
/// to show the button.
#[tauri::command]
fn cli_install_supported() -> bool {
    cfg!(target_os = "macos")
}

/// Install the `epher` command (macOS): symlink `/usr/local/bin/epher` to
/// the app bundle's executable, with an osascript administrator-privilege
/// fallback. Ok carries a Fluent key; Err carries readable instructions.
/// Async + spawn_blocking: the password prompt can be open a long while,
/// and the UI must stay responsive.
#[tauri::command]
async fn install_cli() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(cli_install::install)
        .await
        .map_err(|e| format!("join error: {e}"))?
}

/// Run the desktop GUI (the Tauri event loop). On Windows this is called
/// via [`launch_gui`] after the detach dance; on macOS/Linux it runs
/// in-process in the foreground, like any GUI binary launched from a
/// terminal.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DesktopStore::with_dir(persist::default_store_dir()))
        .invoke_handler(tauri::generate_handler![
            init,
            save_function,
            save_script,
            save_history,
            save_language,
            cli_install_supported,
            install_cli
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

/// The unified-binary entry point (ADR-0011): parse arguments with
/// [`dispatch`], then run the chosen frontend — every mode is a thin call
/// into the frontend's own library entry point, so behavior is defined
/// once (CLI/REPL/stdin: epher-cli; TUI: epher-tui; GUI: this crate).
/// Errors print as `error: …` and exit 1, exactly like the CLI.
pub fn run_with_args<I>(args: I)
where
    I: IntoIterator,
    I::Item: Into<std::ffi::OsString> + Clone,
{
    let parsed = dispatch::Args::try_parse_from(args).unwrap_or_else(|e| e.exit());
    let result = match dispatch::action_from(&parsed) {
        dispatch::Action::OneShot(expr) => epher_cli::run_one_shot(&expr),
        dispatch::Action::Stdin => epher_cli::run_stdin(),
        dispatch::Action::Repl => epher_cli::run_repl(),
        dispatch::Action::Tui => {
            epher_tui::run().map_err(|e| epher_core::EpherError::Io(e.to_string()))
        }
        dispatch::Action::Gui => {
            launch_gui();
            return;
        }
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

/// Launch the desktop GUI.
///
/// The unified binary is a *console* application (so `epher "2 + 2"` can
/// print and pipe from CMD/PowerShell). The cost on Windows: launching the
/// GUI from a double-click would leave a console window open for the app's
/// whole lifetime. The standard cure is a detach dance: the process
/// re-spawns itself with `EPHER_GUI_CHILD` set as a detached process (no
/// console at all) and exits immediately — the double-click console
/// flashes for a few dozen milliseconds, and a terminal prompt returns
/// right away while the GUI window appears. On macOS/Linux the GUI runs
/// in-process in the foreground, like any GUI binary run from a terminal.
fn launch_gui() {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        if std::env::var_os("EPHER_GUI_CHILD").is_none() {
            if let Ok(exe) = std::env::current_exe() {
                let spawned = std::process::Command::new(exe)
                    .env("EPHER_GUI_CHILD", "1")
                    .creation_flags(DETACHED_PROCESS)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
                if spawned.is_ok() {
                    std::process::exit(0);
                }
                // Spawn failed (rare): fall through and run in-process.
            }
        }
    }
    run();
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
