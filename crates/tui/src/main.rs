//! calc-tui — native full-screen terminal frontend (ADR-0001).
//!
//! A thin ratatui shell over the [`App`] seam. Type an expression/script,
//! Enter evaluates against a persistent environment, Esc clears, Ctrl+C or q
//! (with empty input) quits. History and saved functions persist through the
//! shared store (`CALC_STORE_DIR` override, default `~/.calc`).

use std::io;

use calc_core::Session;
use calc_i18n::Localizer;
use calc_store::persist::{default_store_dir, load_language, load_session, save_history};
use calc_store::{DocStore, FsStore};
use calc_tui::{render_ascii, App};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal);
    ratatui::restore();
    result
}

fn run_app(terminal: &mut DefaultTerminal) -> io::Result<()> {
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
    let localizer = Localizer::resolve(preference.as_deref(), &detected);
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
                        if let Some(source) = line.strip_prefix("graph ") {
                            let _ = app.submit_graph(source);
                        } else {
                            app.submit();
                            // best-effort persistence of history
                            let _ = save_history(&store, app.history());
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

fn draw(frame: &mut Frame, app: &App, localizer: &Localizer) {
    let layout = Layout::vertical([
        Constraint::Length(3),  // input
        Constraint::Length(1),  // result
        Constraint::Min(0),     // history
        Constraint::Length(20), // graph
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

    let graph_text = app
        .graph()
        .map(|g| render_ascii(g, 60, 18))
        .unwrap_or_default();
    let graph = Paragraph::new(graph_text)
        .block(Block::default().borders(Borders::ALL).title(localizer.lookup("tui-graph")));
    frame.render_widget(graph, layout[3]);
}
