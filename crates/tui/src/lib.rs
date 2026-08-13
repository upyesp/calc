//! calc-tui — native full-screen terminal frontend (ADR-0001).
//!
//! The testable seam is [`App`] (input/result + the shared [`Session`]) plus
//! the pure [`render_ascii`] plot renderer (ADR-0006: the TUI renders ASCII).
//! The ratatui event loop in `main.rs` is a thin shell over both.

use calc_core::{parse, sample, Sample, Session};

/// The TUI's application state — the testable seam. Rendering is thin.
#[derive(Default)]
pub struct App {
    input: String,
    result: String,
    session: Session,
    graph: Option<Vec<Sample>>,
    graph_source: Option<String>,
}

impl App {
    pub fn with_session(session: Session) -> Self {
        Self {
            input: String::new(),
            result: String::new(),
            session,
            graph: None,
            graph_source: None,
        }
    }

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
        self.session.history()
    }

    /// The sampled points of the current graph, if any.
    pub fn graph(&self) -> Option<&[Sample]> {
        self.graph.as_deref()
    }

    /// The source the current graph was sampled from — rendered as an
    /// accessible caption above the plot (screen readers in terminals read
    /// it instead of raw ASCII art).
    pub fn graph_source(&self) -> Option<&str> {
        self.graph_source.as_deref()
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

    /// Evaluate the current input via the shared [`Session`].
    pub fn submit(&mut self) {
        self.result = self.session.submit(&self.input);
        self.input.clear();
    }

    /// Parse `source` as `y = f(x)` and sample it over [-10, 10] against the
    /// session's environment (so user functions are usable). Returns an error
    /// string on failure; the samples are stored for [`render_ascii`].
    pub fn submit_graph(&mut self, source: &str) -> Result<(), String> {
        let expr = parse(source).map_err(|e| e.to_string())?;
        let samples = sample(&expr, -10.0, 10.0, 120, self.session.env())
            .map_err(|e| e.to_string())?;
        self.graph = Some(samples);
        self.graph_source = Some(source.to_string());
        self.result = format!("graph: {source}");
        Ok(())
    }
}

/// Render samples as an ASCII plot — the TUI's renderer (ADR-0006). The x and
/// y ranges are scaled to the grid; points plot as `o`; non-finite points are
/// skipped.
pub fn render_ascii(samples: &[Sample], width: usize, height: usize) -> String {
    if samples.is_empty() || width == 0 || height == 0 {
        return String::new();
    }
    let x_min = samples
        .iter()
        .map(|s| s.x)
        .fold(f64::INFINITY, f64::min);
    let x_max = samples
        .iter()
        .map(|s| s.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let y_min = samples
        .iter()
        .map(|s| s.y)
        .fold(f64::INFINITY, f64::min);
    let y_max = samples
        .iter()
        .map(|s| s.y)
        .fold(f64::NEG_INFINITY, f64::max);
    let x_span = (x_max - x_min).max(1e-12);
    let y_span = (y_max - y_min).max(1e-12);

    let mut grid = vec![vec!['·'; width]; height];
    for s in samples {
        if !s.x.is_finite() || !s.y.is_finite() {
            continue;
        }
        let col = (((s.x - x_min) / x_span) * (width - 1) as f64).round() as usize;
        let row = height - 1 - (((s.y - y_min) / y_span) * (height - 1) as f64).round() as usize;
        grid[row][col] = 'o';
    }
    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}
