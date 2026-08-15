//! The unified-binary command surface (ADR-0011): one `epher` executable
//! hosts every frontend.
//!
//! - `epher "2 + 2"` — one-shot evaluation
//! - `epher -` — piped script mode (each stdin line evaluated in turn)
//! - `epher repl` — interactive REPL in the terminal
//! - `epher tui` — full-screen terminal UI
//! - `epher gui` / bare `epher` — the desktop GUI (also what double-click,
//!   Start Menu, and Finder launches pass: no arguments)
//!
//! This module owns only the *decision* — parsing arguments into an
//! [`Action`]. Side effects (running a mode, the Windows detach dance for
//! the GUI) live in thin wrappers over the frontends' own entry points.

use clap::{Parser, Subcommand};

/// epher: a programmable, scriptable calculator.
#[derive(Parser, Debug)]
#[command(name = "epher", version, about, args_conflicts_with_subcommands = true)]
pub struct Args {
    /// An expression to evaluate one-shot; `-` reads a script from stdin.
    #[arg(allow_hyphen_values = true)]
    pub expression: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Command {
    /// Interactive REPL in the terminal.
    Repl,
    /// Full-screen terminal UI.
    Tui,
    /// Desktop graphical app (the default when no arguments are given).
    Gui,
}

/// What the unified binary should do. Derived purely from [`Args`] so the
/// mapping is testable without launching anything.
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Evaluate one expression and print the result.
    OneShot(String),
    /// Read a script from stdin, line by line.
    Stdin,
    /// Interactive REPL in the terminal.
    Repl,
    /// Full-screen terminal UI.
    Tui,
    /// Desktop GUI.
    Gui,
}

/// Decide the mode. Subcommands win over the expression positional;
/// no arguments at all means GUI (that is what double-click and Start
/// Menu/Finder launches pass, and terminal users get the GUI with a bare
/// `epher` too).
pub fn action_from(args: &Args) -> Action {
    if let Some(command) = &args.command {
        return match command {
            Command::Repl => Action::Repl,
            Command::Tui => Action::Tui,
            Command::Gui => Action::Gui,
        };
    }
    match args.expression.as_deref() {
        Some("-") => Action::Stdin,
        Some(expr) => Action::OneShot(expr.to_string()),
        None => Action::Gui,
    }
}
