// The unified `epher` binary (ADR-0011): one executable hosts every
// frontend — one-shot evaluation, REPL, piped scripts, the TUI, and the
// desktop GUI. Deliberately a *console* application on Windows (no
// `windows_subsystem = "windows"` attribute): console mode is what lets
// `epher "2 + 2"` print into CMD/PowerShell and pipe cleanly. The GUI path
// handles its own detach dance (see app_lib::launch_gui), so double-click
// launches only flash a console for milliseconds.

fn main() {
    app_lib::run_with_args(std::env::args_os());
}
