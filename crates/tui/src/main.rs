//! calc-tui — native full-screen terminal frontend (ADR-0001).
//!
//! A thin ratatui shell over the [`App`] seam. Type an expression/script,
//! Enter evaluates against a persistent environment, Esc clears, Ctrl+C or q
//! (with empty input) quits.

use std::io;

use calc_tui::App;
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
    let mut app = App::default();
    loop {
        terminal.draw(|frame| draw(frame, &app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(c) => app.push_char(c),
                    KeyCode::Backspace => app.pop_char(),
                    KeyCode::Enter => app.submit(),
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

fn draw(frame: &mut Frame, app: &App) {
    let layout = Layout::vertical([
        Constraint::Length(3), // input
        Constraint::Length(1), // result
        Constraint::Min(0),    // history
    ])
    .split(frame.area());

    let input = Paragraph::new(app.input())
        .block(Block::default().borders(Borders::ALL).title("Expression"));
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
        .block(Block::default().borders(Borders::ALL).title("History"));
    frame.render_widget(history, layout[2]);
}
