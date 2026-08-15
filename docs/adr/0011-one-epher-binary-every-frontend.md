# 0011 — One `epher` Binary Hosts Every Frontend

Date: 2025-06-28 · Status: accepted · Supersedes: nothing (extends ADR-0001's
frontend layout)

## Context

Releases shipped three downloads per platform (CLI, TUI, desktop), each with
its own binary. Users had to choose before trying anything, and the desktop
install put an `epher-desktop` on disk that could not do what the terminal
binary could. The ask: **one download, one install, one executable** —
`epher` on every platform — that is simultaneously the one-shot CLI, the
REPL, the TUI, and the desktop GUI. Example flow for a Windows user: run one
installer, then in PowerShell `epher "2 + 2"`, `epher repl`, `epher tui`,
and a bare `epher` for the GUI.

The frontends already share everything through the workspace crates
(ADR-0001); only the entry points were separate binaries.

## Decision

**The Tauri binary is the unified `epher` executable.** It dispatches on
arguments to the frontends' own library entry points — no frontend logic
lives in the dispatcher:

| Invocation | Mode | Implementation |
| --- | --- | --- |
| `epher "2 + 2"` | one-shot evaluation | `epher_cli::run_one_shot` |
| `epher -` | piped script (stdin, line by line) | `epher_cli::run_stdin` |
| `epher repl` | interactive REPL | `epher_cli::run_repl` |
| `epher tui` | full-screen terminal UI | `epher_tui::run` |
| `epher gui`, bare `epher` | desktop GUI | the Tauri loop (`app_lib::run`) |

- **Bare `epher` opens the GUI.** Double-click, Start Menu, and Finder
  launches pass no arguments — the no-args case must be the GUI. Terminal
  users get the same: a bare `epher` is the windowed app, `epher repl`/`tui`
  are the terminal modes.
- **The dispatch decision is pure** (`app_lib::dispatch`): `Args → Action`,
  tested without launching anything. Subcommands conflict with the
  expression positional (`args_conflicts_with_subcommands`) so
  `epher "1+1" repl` errors instead of silently merging meanings;
  `allow_hyphen_values` keeps `-` (stdin convention, like `sh -`) and `-5`
  (negative literals) working while `--help`/`--version` stay flags.
- **The binary is a console application on Windows** (no
  `windows_subsystem = "windows"`): `epher "2 + 2"` must print into
  CMD/PowerShell and pipe. The cost — a console window on a double-click —
  is paid off with the **detach dance**: on GUI launch the process re-spawns
  itself (`EPHER_GUI_CHILD=1`, `DETACHED_PROCESS`, null stdio) and exits
  immediately, so the double-click console flashes for milliseconds and the
  terminal prompt returns instantly. macOS/Linux run the GUI in-process in
  the foreground, like any GUI binary launched from a terminal.
- **The dev binaries stay** for fast iteration (`epher-cli`, `epher-tui`)
  but releases ship only the unified binary inside each platform installer
  (NSIS on Windows, dmg on macOS, deb/rpm/AppImage on Linux). The Windows
  installer adds the install directory to the user `PATH` so `epher` works
  from any terminal; macOS offers an in-GUI "install the `epher` command"
  action (symlink into `/usr/local/bin`, osascript fallback for permission).
- **The PWA is unchanged** — a browser cannot host native frontends.

## Consequences

- One install per platform; every mode shares the Native Store (`~/.epher`)
  by construction, not convention (ADR-0002, ADR-0010).
- The Tauri package (webkit dependencies) becomes a dependency of *every*
  mode for installed users; headless-server users still have the release
  archives of old versions or build from source. Acceptable: epher's
  installed audience is desktop users.
- Frontend crates must keep their entry points callable as libraries
  (`run_*` functions, thin binaries) — the seam that makes this ADR cheap
  is the same one tests use.
- Bare `epher` changing from REPL (v0.2.0 CLI) to GUI is a breaking UX
  change → version 0.3.0.
