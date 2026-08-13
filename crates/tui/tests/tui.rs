use calc_core::Sample;
use calc_core::Session;
use calc_tui::{render_ascii, App};

#[test]
fn submit_evaluates_against_persistent_env() {
    let mut app = App::default();
    app.set_input("x = 5; x + 1");
    app.submit();
    assert_eq!(app.result(), "= 6");
    app.set_input("x * 2");
    app.submit();
    assert_eq!(app.result(), "= 10");
    assert_eq!(app.history().len(), 2);
}

#[test]
fn app_with_session_starts_from_seeded_history() {
    let mut app = App::with_session(Session::with_history(vec!["old  = 1".to_string()]));
    assert_eq!(app.history().len(), 1);
    app.set_input("1 + 1");
    app.submit();
    assert_eq!(app.result(), "= 2");
    assert_eq!(app.history().len(), 2);
}

#[test]
fn errors_are_shown_not_crashing() {
    let mut app = App::default();
    app.set_input("1/0");
    app.submit();
    assert_eq!(app.result(), "error: division by zero");
}

#[test]
fn empty_input_does_nothing() {
    let mut app = App::default();
    app.submit();
    assert_eq!(app.result(), "");
    assert_eq!(app.history().len(), 0);
}

#[test]
fn graph_command_samples_expression() {
    let mut app = App::default();
    app.submit_graph("x ^ 2").expect("graph should sample");
    assert_eq!(app.graph().map(|g| g.len()), Some(120));
    app.submit_graph("1 / x").expect("graph should sample");
    assert_eq!(app.graph().map(|g| g.len()), Some(120));
}

#[test]
fn graph_command_uses_session_functions() {
    let mut app = App::default();
    app.set_input("def f(x) = x ^ 3");
    app.submit();
    app.submit_graph("f(x)").expect("graph should sample");
    assert_eq!(app.graph().map(|g| g.len()), Some(120));
}

#[test]
fn graph_command_records_source_for_caption() {
    let mut app = App::default();
    assert_eq!(app.graph_source(), None);
    app.submit_graph("x ^ 2").expect("graph should sample");
    assert_eq!(app.graph_source(), Some("x ^ 2"));
}

#[test]
fn render_ascii_plots_a_diagonal() {
    let samples = vec![
        Sample { x: 0.0, y: 0.0 },
        Sample { x: 1.0, y: 1.0 },
        Sample { x: 2.0, y: 2.0 },
    ];
    assert_eq!(render_ascii(&samples, 3, 3), "··o\n·o·\no··");
}

#[test]
fn render_ascii_handles_empty_and_non_finite() {
    assert_eq!(render_ascii(&[], 3, 3), "");
    let samples = vec![
        Sample { x: f64::NAN, y: 0.0 },
        Sample { x: 0.0, y: f64::INFINITY },
        Sample { x: 1.0, y: 1.0 },
    ];
    let out = render_ascii(&samples, 3, 3);
    assert!(out.contains('o'));
    assert!(!out.contains("NaN"));
}
