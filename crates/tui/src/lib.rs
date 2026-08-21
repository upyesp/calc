//! epher-tui — native full-screen terminal frontend (ADR-0001).
//!
//! The testable seam is [`App`] (input/result + the shared [`Session`]) plus
//! the pure [`render_ascii`] plot renderer (ADR-0006: the TUI renders ASCII).
//! [`run`] is the ratatui event loop — a thin shell over both — exposed as a
//! library function so the unified `epher` binary can host it (`epher tui`).

use epher_core::graph::{
    analyze, free_names, parse_graph_source, project_surface, sample_spec, sample_surface,
    surface_frame, InterestPoint, InterestKind, SampledCurve, Surface, View3D,
};
use epher_core::Session;
use epher_i18n::Localizer;
use epher_shell::{classify, plain, run_command};
use epher_store::persist::{default_store_dir, load_language, load_session, save_history};
use epher_store::{DocStore, FsStore};
use unicode_width::UnicodeWidthStr;

/// An active parameter animation: `name` steps by `step` within `lo..=hi`,
/// wrapping around (Desmos-style loop).
#[derive(Debug, Clone, PartialEq)]
pub struct Play {
    pub name: String,
    pub lo: f64,
    pub hi: f64,
    pub step: f64,
}

/// The TUI's application state — the testable seam. Rendering is thin.
#[derive(Default)]
pub struct App {
    input: String,
    result: String,
    session: Session,
    graph: Vec<SampledCurve>,
    pois: Vec<InterestPoint>,
    surface: Vec<Surface>,
    view: View3D,
    play: Option<Play>,
    /// Keypad focus mode (ADR-0016): Tab opens the button grid, arrows
    /// move the highlight, Enter appends the token, Esc/Tab closes.
    keypad: bool,
    kp_row: usize,
    kp_col: usize,
}

/// The TUI keypad (ADR-0016): a condensed 4×5 grid of the most-used
/// tokens — the full set lives on the web keypad; the terminal stays
/// compact. (display, insert-at-end).
const KEYPAD: &[&[(&str, &str)]] = &[
    &[
        ("sin", "sin("),
        ("cos", "cos("),
        ("tan", "tan("),
        ("ln", "ln("),
        ("log", "log("),
    ],
    &[
        ("sqrt", "sqrt("),
        ("abs", "abs("),
        ("floor", "floor("),
        ("ceil", "ceil("),
        ("round", "round("),
    ],
    &[
        ("pi", "pi"),
        ("e", "e"),
        ("tau", "tau"),
        ("frac", "frac("),
        ("dec", "dec("),
    ],
    &[
        ("big", "big("),
        ("graph", "graph "),
        ("graph3d", "graph3d "),
        ("table", "table "),
        ("clear", "clear "),
    ],
];

impl App {
    pub fn with_session(session: Session) -> Self {
        Self {
            input: String::new(),
            result: String::new(),
            session,
            graph: Vec::new(),
            pois: Vec::new(),
            surface: Vec::new(),
            view: View3D::default(),
            play: None,
            keypad: false,
            kp_row: 0,
            kp_col: 0,
        }
    }

    // --- keypad mode (ADR-0016) ---

    pub fn keypad_focused(&self) -> bool {
        self.keypad
    }

    pub fn keypad_row(&self) -> usize {
        self.kp_row
    }

    pub fn keypad_col(&self) -> usize {
        self.kp_col
    }

    pub fn keypad_open(&mut self) {
        self.keypad = true;
    }

    pub fn keypad_close(&mut self) {
        self.keypad = false;
    }

    /// Move the highlight, wrapping around the grid edges.
    pub fn keypad_move(&mut self, dr: isize, dc: isize) {
        let rows = KEYPAD.len() as isize;
        let cols = KEYPAD.first().map(|r| r.len() as isize).unwrap_or(1);
        self.kp_row = (self.kp_row as isize + dr).rem_euclid(rows) as usize;
        self.kp_col = (self.kp_col as isize + dc).rem_euclid(cols) as usize;
    }

    /// Append the highlighted token to the end of the input (the terminal
    /// cursor already lives there).
    pub fn keypad_insert(&mut self) {
        let token = KEYPAD[self.kp_row][self.kp_col].1;
        self.input.push_str(token);
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

    /// The shared session (constants, history) — public so tests can read
    /// animation state.
    pub fn session(&self) -> &Session {
        &self.session
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

    /// Empty the history list (Ctrl+L); definitions and constants stay.
    pub fn clear_history(&mut self) {
        self.session.clear_history();
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
    /// `graph3d ` samples a surface, anything else evaluates — and history
    /// persists. A line may join several statements with `;` (the same
    /// separator as a newline, ADR-0001): each statement dispatches in
    /// order, exactly as if typed one by one. Returns the new language
    /// preference when a `language` command changed it, so the caller can
    /// re-resolve its Localizer.
    pub fn submit_line(
        &mut self,
        line: &str,
        store: &DocStore<FsStore>,
        localizer: &Localizer,
    ) -> Option<String> {
        let mut language = None;
        for piece in line.split(';') {
            let piece = piece.trim();
            if piece.is_empty() {
                continue;
            }
            let changed = self.submit_statement(piece, store, localizer);
            if language.is_none() {
                language = changed;
            }
        }
        language
    }

    /// Dispatch one statement (no `;`, no newline) the way submit_line used
    /// to handle a whole line.
    fn submit_statement(
        &mut self,
        piece: &str,
        store: &DocStore<FsStore>,
        localizer: &Localizer,
    ) -> Option<String> {
        if let Some(source) = piece.strip_prefix("graph ") {
            let _ = self.submit_graph(source);
            return None;
        }
        if let Some(source) = piece.strip_prefix("graph3d ") {
            let _ = self.submit_surface(source);
            return None;
        }
        if let Some(cmd) = classify(piece) {
            let handled = run_command(&cmd, &mut self.session, store, localizer);
            self.result = plain(handled.message);
            self.input.clear();
            return handled.language;
        }
        self.input = piece.to_string();
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
        let spec = match parse_graph_source(source).map_err(|e| e.to_string()) {
            Ok(s) => s,
            Err(e) => {
                self.result = format!("error: {e}");
                return Err(e);
            }
        };
        let samples = match sample_spec(&spec, 120, self.session.env()).map_err(|e| e.to_string()) {
            Ok(samples) => samples,
            Err(e) => {
                self.result = format!("error: {e}");
                return Err(e);
            }
        };
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

    /// Parse `source` as a `graph3d` command (ADR-0015 grammar:
    /// `z = f(x, y)` over an optional square domain) and overlay it on the
    /// current surface set; `graph3d clear` empties it.
    pub fn submit_surface(&mut self, source: &str) -> Result<(), String> {
        if source.trim() == "clear" {
            self.surface.clear();
            self.result.clear();
            return Ok(());
        }
        let surface = match sample_surface(source, 40, self.session.env()).map_err(|e| e.to_string()) {
            Ok(s) => s,
            Err(e) => {
                self.result = format!("error: {e}");
                return Err(e);
            }
        };
        self.result = format!("graph3d: {}", surface.source);
        self.surface.push(surface);
        Ok(())
    }

    /// The plotted surfaces, if any.
    pub fn surfaces(&self) -> &[Surface] {
        &self.surface
    }

    /// The 3D camera pose.
    pub fn view(&self) -> &View3D {
        &self.view
    }

    /// Orbit the 3D view by the given yaw/pitch deltas (radians).
    pub fn rotate_view(&mut self, dyaw: f64, dpitch: f64) {
        self.view = self.view.with_pitch(self.view.pitch + dpitch).with_yaw(self.view.yaw + dyaw);
    }

    /// The active animation, if any.
    pub fn play(&self) -> Option<&Play> {
        self.play.as_ref()
    }

    /// Start or stop the parameter animation. Playing animates the first
    /// constant referenced by any plotted surface (or curve) within its
    /// current value ±2, stepping 0.1 per tick and wrapping around — the
    /// TUI's counterpart of the web sliders' play button (ADR-0015).
    pub fn toggle_play(&mut self) -> bool {
        if self.play.is_some() {
            self.play = None;
            return false;
        }
        let name = self.animated_constant();
        let Some(v) = name.as_ref().and_then(|n| self.session.env().constant(n)) else {
            return false;
        };
        let v = match v {
            epher_core::Value::Float(f) => *f,
            _ => return false,
        };
        self.play = Some(Play {
            name: name.unwrap(),
            lo: v - 2.0,
            hi: v + 2.0,
            step: 0.1,
        });
        true
    }

    /// The first constant referenced by a plotted surface, else a plotted
    /// curve — the parameter animation steps it.
    fn animated_constant(&self) -> Option<String> {
        let mut names = std::collections::BTreeSet::new();
        for s in &self.surface {
            if let Ok((expr, _)) = epher_core::graph::parse_surface_source(&s.source) {
                free_names(&expr, &mut names);
            }
        }
        for c in &self.graph {
            if let Ok(spec) = parse_graph_source(&c.source) {
                match &spec.kind {
                    epher_core::graph::CurveKind::Cartesian(e) => free_names(e, &mut names),
                    epher_core::graph::CurveKind::Parametric { x, y } => {
                        free_names(x, &mut names);
                        free_names(y, &mut names);
                    }
                    epher_core::graph::CurveKind::Polar(e) => free_names(e, &mut names),
                }
            }
        }
        names
            .into_iter()
            .find(|n| self.session.env().constant(n.as_str()).is_some())
    }

    /// Advance the animation by one tick: step the constant, wrapping at the
    /// bounds, and re-sample everything that references it.
    pub fn tick(&mut self) {
        let Some(play) = self.play.clone() else {
            return;
        };
        let Some(v) = self.session.env().constant(&play.name) else {
            self.play = None;
            return;
        };
        let v = match v {
            epher_core::Value::Float(f) => *f,
            _ => {
                self.play = None;
                return;
            }
        };
        let mut next = v + play.step;
        if next > play.hi {
            next = play.lo;
        }
        self.session
            .set_constant(play.name.clone(), epher_core::Value::float(next), String::new());
        self.resample_all();
    }

    /// Re-sample every plot against the current environment (after an
    /// animation tick moved a constant).
    fn resample_all(&mut self) {
        let env = self.session.env().clone();
        for c in &mut self.graph {
            if let Ok(spec) = parse_graph_source(&c.source) {
                if let Ok(samples) = sample_spec(&spec, 120, &env) {
                    c.samples = samples;
                }
            }
        }
        self.pois = analyze(&self.graph, &env);
        for s in &mut self.surface {
            if let Ok(fresh) = sample_surface(&s.source, 40, &env) {
                *s = fresh;
            }
        }
    }
}

/// Render the projected 3D mesh as an ASCII wireframe (ADR-0015): depth-
/// shaded Bresenham lines on a uniform grid — near segments `*`, middle
/// `+`, far `.` — with the ground square and axes (`o`) drawn on top. The
/// painter-sorted segments overpaint in draw order, so nearer mesh lines
/// stay visible over farther ones.
pub fn render_ascii3d(surfaces: &[Surface], view: &View3D, width: usize, height: usize) -> String {
    if surfaces.is_empty() || width == 0 || height == 0 {
        return String::new();
    }
    let mut all = Vec::new();
    for (i, s) in surfaces.iter().enumerate() {
        all.extend(project_surface(s, view));
        if i == 0 {
            all.extend(surface_frame(s, view));
        }
    }
    if all.is_empty() {
        return String::new();
    }
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for seg in &all {
        x_min = x_min.min(seg.x1).min(seg.x2);
        x_max = x_max.max(seg.x1).max(seg.x2);
        y_min = y_min.min(seg.y1).min(seg.y2);
        y_max = y_max.max(seg.y1).max(seg.y2);
    }
    let (x_min, x_max, y_min, y_max) = (x_min, x_max, y_min, y_max);
    if !x_min.is_finite() || x_max - x_min < 1e-9 || y_max - y_min < 1e-9 {
        return String::new();
    }
    let depth_min = all.iter().map(|s| s.depth).fold(f64::INFINITY, f64::min);
    let depth_max = all.iter().map(|s| s.depth).fold(f64::NEG_INFINITY, f64::max);
    let span = depth_max - depth_min;
    let gw = width - 2;
    let gh = height;
    let scale = ((gw as f64) / (x_max - x_min)).min((gh as f64) / (y_max - y_min));
    let ox = (gw as f64 - (x_max - x_min) * scale) / 2.0;
    let oy = (gh as f64 - (y_max - y_min) * scale) / 2.0;
    let to_grid = |x: f64, y: f64| {
        let c = (x - x_min) * scale + ox;
        let r = (y_max - y) * scale + oy;
        (r as isize, c as isize)
    };
    let mut grid = vec![vec![' '; width]; height];
    let mut stamp = |x1: f64, y1: f64, x2: f64, y2: f64, depth: f64, frame: bool| {
        let (r1, c1) = to_grid(x1, y1);
        let (r2, c2) = to_grid(x2, y2);
        let glyph = if frame {
            'o'
        } else if span < 1e-9 {
            '*'
        } else {
            let t = ((depth - depth_min) / span * 2.0).clamp(0.0, 2.0);
            ['*', '+', '.'][t.floor() as usize]
        };
        // Bresenham
        let (dr, dc) = (r2 - r1, c2 - c1);
        let steps = dr.abs().max(dc.abs());
        if steps == 0 {
            if r1 >= 0 && r1 < height as isize && c1 >= 0 && c1 < width as isize {
                grid[r1 as usize][c1 as usize] = glyph;
            }
            return;
        }
        for k in 0..=steps {
            let r = r1 + (dr * k) / steps;
            let c = c1 + (dc * k) / steps;
            if r >= 0 && r < height as isize && c >= 0 && c < width as isize {
                let cell = &mut grid[r as usize][c as usize];
                // Nearer mesh overpaints farther mesh; the frame (drawn
                // last) overpaints everything.
                if frame || *cell != 'o' {
                    *cell = glyph;
                }
            }
        }
    };
    // Far to near: nearer (drawn later) overpaints.
    let mut order = all.iter().collect::<Vec<_>>();
    order.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    for seg in order {
        stamp(seg.x1, seg.y1, seg.x2, seg.y2, seg.depth, false);
    }
    // Frame last, on top.
    for s in surfaces.iter().take(1) {
        for seg in surface_frame(s, view) {
            stamp(seg.x1, seg.y1, seg.x2, seg.y2, seg.depth, true);
        }
    }
    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
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
    // One step per 120 ms while playing — the same rate as the web
    // sliders' play button (ADR-0015). The poll below wakes at 50 ms so
    // key presses stay responsive; the step itself is paced here.
    let mut last_tick = std::time::Instant::now();
    loop {
        terminal.draw(|frame| draw(frame, &app, &localizer))?;
        // While an animation plays, wait at most one tick for input so the
        // plot advances on its own; otherwise block on the next event.
        let event = if app.play().is_some() {
            match event::poll(std::time::Duration::from_millis(50)) {
                Ok(true) => Some(event::read()?),
                Ok(false) => None,
                Err(e) => return Err(e),
            }
        } else {
            Some(event::read()?)
        };
        if app.play().is_some() {
            if last_tick.elapsed() >= std::time::Duration::from_millis(120) {
                app.tick();
                last_tick = std::time::Instant::now();
            }
        }
        if let Some(Event::Key(key)) = event {
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
                    KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.clear_history();
                        let _ = save_history(&store, app.history());
                    }
                    KeyCode::Char('q') if app.input().is_empty() && !app.keypad_focused() => {
                        return Ok(());
                    }
                    // Keypad mode (ADR-0016): Tab opens/closes the button
                    // grid; inside it, arrows move and Enter inserts.
                    KeyCode::Tab => {
                        if app.keypad_focused() {
                            app.keypad_close();
                        } else {
                            app.keypad_open();
                        }
                    }
                    KeyCode::Left if app.keypad_focused() => app.keypad_move(0, -1),
                    KeyCode::Right if app.keypad_focused() => app.keypad_move(0, 1),
                    KeyCode::Up if app.keypad_focused() => app.keypad_move(-1, 0),
                    KeyCode::Down if app.keypad_focused() => app.keypad_move(1, 0),
                    KeyCode::Esc if app.keypad_focused() => app.keypad_close(),
                    // 3D orbit (ADR-0015): arrows rotate when the input line
                    // is empty, so typing never loses an arrow key.
                    KeyCode::Left if app.input().is_empty() => app.rotate_view(-0.15, 0.0),
                    KeyCode::Right if app.input().is_empty() => app.rotate_view(0.15, 0.0),
                    KeyCode::Up if app.input().is_empty() => app.rotate_view(0.0, 0.15),
                    KeyCode::Down if app.input().is_empty() => app.rotate_view(0.0, -0.15),
                    // Space starts/stops the parameter animation (ADR-0015).
                    KeyCode::Char(' ') if app.input().is_empty() => {
                        app.toggle_play();
                    }
                    // Any typed character leaves keypad mode first — typing
                    // is the other spelling of the same input.
                    KeyCode::Char(c) if !is_enter => {
                        app.keypad_close();
                        app.push_char(c);
                    }
                    KeyCode::Backspace => app.pop_char(),
                    KeyCode::Esc => app.clear_input(),
                    _ => {}
                }
                if is_enter && !app.keypad_focused() {
                    let line = app.input().trim().to_string();
                    if let Some(code) = app.submit_line(&line, &store, &localizer) {
                        localizer = Localizer::resolve(Some(&code), &[]);
                    }
                    // Every submit empties the line — including graph
                    // commands, whose path doesn't clear it itself — so a
                    // multi-line paste leaves a clean slate for the next
                    // line instead of appending to the leftover.
                    app.clear_input();
                } else if is_enter {
                    app.keypad_insert();
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

    // Keypad focus (ADR-0016) borrows six rows from the graph panel so
    // the button grid fits without shrinking history.
    let layout = if app.keypad_focused() {
        Layout::vertical([
            Constraint::Length(3),  // input
            Constraint::Length(1),  // result
            Constraint::Min(0),     // history
            Constraint::Length(14), // graph (shrunk while keypad is open)
            Constraint::Length(6),  // keypad
            Constraint::Length(1),  // hints
        ])
    } else {
        Layout::vertical([
            Constraint::Length(3),  // input
            Constraint::Length(1),  // result
            Constraint::Min(0),     // history
            Constraint::Length(20), // graph
            Constraint::Length(1),  // hints
        ])
    }
    .split(frame.area());
    let (graph_area, keypad_area, hints_area) = if app.keypad_focused() {
        (layout[3], Some(layout[4]), layout[5])
    } else {
        (layout[3], None, layout[4])
    };

    let input = Paragraph::new(app.input())
        .block(Block::default().borders(Borders::ALL).title(localizer.lookup("tui-expression")));
    frame.render_widget(input, layout[0]);

    let result = Paragraph::new(app.result())
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    frame.render_widget(result, layout[1]);

    let history_lines: Vec<Line> = app
        .history()
        .iter()
        .rev()
        .map(|h| Line::from(h.as_str()))
        .collect();
    let history = Paragraph::new(history_lines)
        .block(Block::default().borders(Borders::ALL).title(localizer.lookup("tui-history")));
    frame.render_widget(history, layout[2]);

    // Legend + plot + points of interest, capped to the panel height.
    let mut graph_text = String::new();
    let curves = app.graph();
    if !app.surfaces().is_empty() {
        // 3D: the text alternative names each surface, then the wireframe.
        let legend: Vec<String> = app
            .surfaces()
            .iter()
            .map(|s| format!("z = {}", s.source.trim()))
            .collect();
        graph_text.push_str(&legend.join("   "));
        graph_text.push('\n');
        graph_text.push_str(&render_ascii3d(app.surfaces(), app.view(), 60, 15));
    } else if !curves.is_empty() {
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
    frame.render_widget(graph, graph_area);

    // The keypad grid (ADR-0016): the highlighted cell inserts its token.
    if let Some(kp_area) = keypad_area {
        use ratatui::text::Span;
        let rows: Vec<Line> = KEYPAD
            .iter()
            .enumerate()
            .map(|(r, row)| {
                let cells: Vec<Span> = row
                    .iter()
                    .enumerate()
                    .map(|(c, (disp, _))| {
                        let selected = r == app.keypad_row() && c == app.keypad_col();
                        let style = if selected {
                            Style::default()
                                .bg(Color::Cyan)
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        Span::styled(format!(" {:<7}", disp), style)
                    })
                    .collect();
                Line::from(cells)
            })
            .collect();
        let keypad = Paragraph::new(rows)
            .block(Block::default().borders(Borders::ALL).title(localizer.lookup("tui-keypad")));
        frame.render_widget(keypad, kp_area);
    }

    let hints = Paragraph::new(localizer.lookup("tui-hints"))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hints, hints_area);

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
