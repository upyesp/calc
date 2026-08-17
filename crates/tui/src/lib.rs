//! epher-tui — native full-screen terminal frontend (ADR-0001).
//!
//! The testable seam is [`App`] (input/result + the shared [`Session`]) plus
//! the pure [`render_ascii`] plot renderer (ADR-0006: the TUI renders ASCII).
//! [`run`] is the ratatui event loop — a thin shell over both — exposed as a
//! library function so the unified `epher` binary can host it (`epher tui`).

use epher_core::graph::{
    analyze, parse_graph_source, sample_spec, InterestPoint, InterestKind, SampledCurve,
};
use epher_core::Session;
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
    graph: Vec<SampledCurve>,
    pois: Vec<InterestPoint>,
}

impl App {
    pub fn with_session(session: Session) -> Self {
        Self {
            input: String::new(),
            result: String::new(),
            session,
            graph: Vec::new(),
            pois: Vec::new(),
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

    /// The plotted curves of the current graph, if any (ADR-0014: the TUI
    /// overlays curves the way the web app does).
    pub fn graph(&self) -> &[SampledCurve] {
        &self.graph
    }

    /// The points of interest of the current graph (roots, intersections,
    /// extrema), recomputed after every graph command.
    pub fn pois(&self) -> &[InterestPoint] {
        &self.pois
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

    /// Parse `source` as a graph command (ADR-0014 grammar: cartesian,
    /// `param`, `polar`, domain bounds, `y <`/`y >` fills) and overlay it on
    /// the current plot; `graph clear` empties the plot. Returns an error
    /// string on failure; points of interest are recomputed for the whole
    /// set.
    pub fn submit_graph(&mut self, source: &str) -> Result<(), String> {
        if source.trim() == "clear" {
            self.graph.clear();
            self.pois.clear();
            self.result.clear();
            return Ok(());
        }
        let spec = parse_graph_source(source).map_err(|e| e.to_string())?;
        let samples = sample_spec(&spec, 120, self.session.env())
            .map_err(|e| e.to_string())?;
        self.graph.push(SampledCurve {
            source: source.to_string(),
            kind: spec.kind,
            domain: spec.domain,
            samples,
            fill: spec.fill,
        });
        self.pois = analyze(&self.graph, self.session.env());
        self.result = format!("graph: {source}");
        Ok(())
    }
}

/// Render the plotted curves as an ASCII plot — the TUI's renderer
/// (ADR-0006/0014). The x and y ranges are scaled to the grid; each curve
/// plots with its own glyph (`o`, `x`, `+`, `*`); region fills shade with
/// `.`; axes draw as `|`/`-` when zero lies strictly inside the range
/// (edge-zero plots stay clean); non-finite points are skipped.
pub fn render_ascii(curves: &[SampledCurve], width: usize, height: usize) -> String {
    if curves.is_empty() || curves.iter().all(|c| c.samples.is_empty()) || width == 0 || height == 0
    {
        return String::new();
    }
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for c in curves {
        for s in &c.samples {
            if s.x.is_finite() && s.y.is_finite() {
                x_min = x_min.min(s.x);
                x_max = x_max.max(s.x);
                y_min = y_min.min(s.y);
                y_max = y_max.max(s.y);
            }
        }
    }
    if !x_min.is_finite() {
        return String::new();
    }
    let x_span = (x_max - x_min).max(1e-12);
    let y_span = (y_max - y_min).max(1e-12);

    let mut grid = vec![vec!['·'; width]; height];

    // Region fills under/above each curve.
    for c in curves {
        let Some(fill) = c.fill else { continue };
        let below = matches!(fill, epher_core::graph::Fill::Below);
        for s in &c.samples {
            if !s.x.is_finite() || !s.y.is_finite() {
                continue;
            }
            let col = (((s.x - x_min) / x_span) * (width - 1) as f64).round() as usize;
            let row = height as f64
                - 1.0
                - ((s.y - y_min) / y_span) * (height - 1) as f64;
            let row = row.round() as usize;
            if below {
                for cell_row in grid[(row + 1)..].iter_mut() {
                    if cell_row[col] == '·' {
                        cell_row[col] = '.';
                    }
                }
            } else {
                for cell_row in grid[..row].iter_mut() {
                    if cell_row[col] == '·' {
                        cell_row[col] = '.';
                    }
                }
            }
        }
    }

    // Axes, only when zero is strictly inside the range.
    let eps_x = x_span * 1e-9;
    let eps_y = y_span * 1e-9;
    if x_min + eps_x < 0.0 && 0.0 < x_max - eps_x {
        let col = ((-x_min) / x_span * (width - 1) as f64).round() as usize;
        let col = col.min(width - 1);
        for row in grid.iter_mut() {
            if row[col] == '·' {
                row[col] = '|';
            }
        }
    }
    if y_min + eps_y < 0.0 && 0.0 < y_max - eps_y {
        let row = height as f64 - 1.0 - ((-y_min) / y_span) * (height - 1) as f64;
        let row = (row.round() as usize).min(height - 1);
        for cell in grid[row].iter_mut() {
            if *cell == '·' {
                *cell = '-';
            }
        }
    }

    // Curves, glyph per curve index.
    const GLYPHS: [char; 4] = ['o', 'x', '+', '*'];
    for (i, c) in curves.iter().enumerate() {
        let glyph = GLYPHS[i % GLYPHS.len()];
        for s in &c.samples {
            if !s.x.is_finite() || !s.y.is_finite() {
                continue;
            }
            let col = (((s.x - x_min) / x_span) * (width - 1) as f64).round() as usize;
            let row = height
                - 1
                - (((s.y - y_min) / y_span) * (height - 1) as f64).round() as usize;
            grid[row][col] = glyph;
        }
    }
    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The localized kind label for a point of interest (the same fluent keys
/// the web legend uses).
fn poi_label(kind: InterestKind, localizer: &Localizer) -> String {
    localizer.lookup(match kind {
        InterestKind::Root => "poi-root",
        InterestKind::Intersection => "poi-intersection",
        InterestKind::Maximum => "poi-maximum",
        InterestKind::Minimum => "poi-minimum",
    })
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
                // Pasted newlines arrive as LF, which crossterm parses as
                // Ctrl+J (the terminal convention for line feed). Treat it
                // as Enter so multi-line pastes submit line by line, like
                // the REPL and piped scripts.
                let is_enter = key.code == KeyCode::Enter
                    || (key.code == KeyCode::Char('j')
                        && key.modifiers.contains(KeyModifiers::CONTROL));
                match key.code {
                    // Guarded arms must precede the generic `Char` arm — the
                    // catch-all would swallow Ctrl+C and type a 'c' instead.
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('q') if app.input().is_empty() => return Ok(()),
                    KeyCode::Char(c) if !is_enter => app.push_char(c),
                    KeyCode::Backspace => app.pop_char(),
                    KeyCode::Esc => app.clear_input(),
                    _ => {}
                }
                if is_enter {
                    let line = app.input().trim().to_string();
                    if let Some(code) = app.submit_line(&line, &store, &localizer) {
                        localizer = Localizer::resolve(Some(&code), &[]);
                    }
                    // Every submit empties the line — including graph
                    // commands, whose path doesn't clear it itself — so a
                    // multi-line paste leaves a clean slate for the next
                    // line instead of appending to the leftover.
                    app.clear_input();
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

    // Legend + plot + points of interest, capped to the panel height.
    let mut graph_text = String::new();
    let curves = app.graph();
    if !curves.is_empty() {
        // The visible text alternative: what is plotted (screen readers in
        // terminals read this instead of raw ASCII art).
        let legend: Vec<String> = curves
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let glyph = ['o', 'x', '+', '*'][i % 4];
                let caption = match &c.kind {
                    epher_core::graph::CurveKind::Cartesian(_) => format!("y = {}", c.source.trim()),
                    _ => c.source.trim().to_string(),
                };
                format!("{glyph} {caption}")
            })
            .collect();
        graph_text.push_str(&legend.join("   "));
        graph_text.push('\n');
        let plot = render_ascii(curves, 60, 15);
        graph_text.push_str(&plot);
        let poi_lines: Vec<String> = app
            .pois()
            .iter()
            .take(2)
            .map(|p| {
                format!(
                    "{} ({:.3}, {:.3})",
                    poi_label(p.kind, localizer),
                    p.x,
                    p.y
                )
            })
            .collect();
        if !poi_lines.is_empty() {
            graph_text.push('\n');
            graph_text.push_str(&poi_lines.join("   "));
        }
    }
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
    frame.set_cursor_position(Position::new(x, input_area.y + 1));
}
