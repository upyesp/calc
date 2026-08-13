//! calc-tui — native full-screen terminal frontend (ADR-0001).
//!
//! The testable seam is [`App`] (input/result/history + a persistent [`Env`]);
//! the ratatui event loop in `main.rs` is a thin shell over it.

use calc_core::{parse_script, run, Env};

/// The TUI's application state — the testable seam. Rendering is thin.
#[derive(Default)]
pub struct App {
    input: String,
    result: String,
    history: Vec<String>,
    env: Env,
}

impl App {
    pub fn set_input(&mut self, input: &str) {
        self.input = input.to_string();
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn result(&self) -> &str {
        &self.result
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn pop_char(&mut self) {
        self.input.pop();
    }

    /// Evaluate the current input as a script against the persistent
    /// environment, record it in history, and show the result.
    pub fn submit(&mut self) {
        let line = self.input.trim().to_string();
        if line.is_empty() {
            return;
        }
        let output = match parse_script(&line) {
            Ok(script) => match run(&script, &mut self.env) {
                Ok(value) => format!("= {value}"),
                Err(e) => format!("error: {e}"),
            },
            Err(e) => format!("error: {e}"),
        };
        self.history.push(format!("{line}  {output}"));
        self.result = output;
        self.input.clear();
    }
}
