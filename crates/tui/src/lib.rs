//! epher-tui — native full-screen terminal frontend (ADR-0001).
//!
//! The testable seam is [`App`] (input/result + the shared [`Session`]) plus
//! the pure [`render_ascii`] plot renderer (ADR-0006: the TUI renders ASCII).
//! [`run`] is the ratatui event loop — a thin shell over both — exposed as a
//! library function so the unified `epher` binary can host it (`epher tui`).

use epher_core::{parse, sample, Sample, Session};
use epher_i18n::Localizer;
use epher_shell::{classify, plain, run_command};
use epher_store::persist::{default_store_dir, load_language, load_session, save_history};
use epher_store::{DocStore, FsStore};
use unicode_width::UnicodeWidthStr;

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

    /// Handle one submitted line the way the event loop does: shell commands
    /// dispatch through the shared kernel (epher-shell), `graph ` samples,
    /// anything else evaluates — and history persists. Returns the new
    /// language preference when a `language` command changed it, so the
    /// caller can re-resolve its Localizer.
    pub fn submit_line(
        &mut self,
        line: &str,
        store: &DocStore<FsStore>,
        localizer: &Localizer,
    ) -> Option<String> {
        let line = line.trim();
        if let Some(source) = line.strip_prefix("graph ") {
            let _ = self.submit_graph(source);
            return None;
        }
        if let Some(cmd) = classify(line) {
            let handled = run_command(&cmd, &mut self.session, store, localizer);
            self.result = plain(handled.message);
            self.input.clear();
            return handled.language;
        }
        self.input = line.to_string();
        self.submit();
        let _ = save_history(store, self.history());
        None
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

/// Run the interactive terminal UI (ratatui event loop). Blocks until the
/// user quits (Ctrl+C, or `q` with empty input). The loop itself is a thin
/// shell over [`App`]: it loads the shared store (ADR-0002), resolves the
/// UI language (ADR-0008), and forwards keys; all state transitions go
/// through the tested [`App`] seam.
pub fn run() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal);
    ratatui::restore();
    result
}

fn run_loop(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

    let store = DocStore::new(FsStore::new(default_store_dir()));
    let session = match load_session(&store) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warning: could not load saved data ({e}); starting fresh");
            Session::new()
        }
    };
    let preference = load_language(&store).unwrap_or(None);
    let detected: Vec<String> = sys_locale::get_locales().collect();
    let mut localizer = Localizer::resolve(preference.as_deref(), &detected);
    let mut app = App::with_session(session);
    loop {
        terminal.draw(|frame| draw(frame, &app, &localizer))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(c) => app.push_char(c),
                    KeyCode::Backspace => app.pop_char(),
                    KeyCode::Enter => {
                        let line = app.input().trim().to_string();
                        if let Some(code) = app.submit_line(&line, &store, &localizer) {
                            localizer = Localizer::resolve(Some(&code), &[]);
                        }
                    }
                    KeyCode::Esc => app.clear_input(),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('q') if app.input().is_empty() => return Ok(()),
                    _ => {}
                }
            }
        }
    }
}

fn draw(frame: &mut ratatui::Frame, app: &App, localizer: &Localizer) {
    use ratatui::layout::{Constraint, Layout, Position};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::Line;
    use ratatui::widgets::{Block, Borders, Paragraph};

    let layout = Layout::vertical([
        Constraint::Length(3),  // input
        Constraint::Length(1),  // result
        Constraint::Min(0),     // history
        Constraint::Length(20), // graph
        Constraint::Length(1),  // hints
    ])
    .split(frame.area());

    let input = Paragraph::new(app.input())
        .block(Block::default().borders(Borders::ALL).title(localizer.lookup("tui-expression")));
    frame.render_widget(input, layout[0]);

    let result = Paragraph::new(app.result())
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    frame.render_widget(result, layout[1]);

    let history_lines: Vec<Line> = app
        .history()
        .iter()
        .map(|h| Line::from(h.as_str()))
        .collect();
    let history = Paragraph::new(history_lines)
        .block(Block::default().borders(Borders::ALL).title(localizer.lookup("tui-history")));
    frame.render_widget(history, layout[2]);

    let graph_text = match app.graph() {
        Some(g) => {
            // A text caption above the plot: terminal screen readers read it
            // instead of raw ASCII art.
            let caption = app
                .graph_source()
                .map(|s| format!("y = {s}"))
                .unwrap_or_default();
            let plot = render_ascii(g, 60, 18);
            if caption.is_empty() {
                plot
            } else {
                format!("{caption}\n{plot}")
            }
        }
        None => String::new(),
    };
    let graph = Paragraph::new(graph_text)
        .block(Block::default().borders(Borders::ALL).title(localizer.lookup("tui-graph")));
    frame.render_widget(graph, layout[3]);

    let hints = Paragraph::new(localizer.lookup("tui-hints"))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hints, layout[4]);

    // Focus visible: the terminal cursor must sit at the end of the input
    // text, not wherever the shell left it.
    let input_area = layout[0];
    let text_width = UnicodeWidthStr::width(app.input());
    let x = input_area
        .x
        .saturating_add(1)
        .saturating_add(text_width as u16)
        .min(input_area.right().saturating_sub(2));
    frame.set_cursor_position(ratatui::layout::Position::new(x, input_area.y + 1));
}
