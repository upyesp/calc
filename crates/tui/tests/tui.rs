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

// --- shell commands through the App seam (ADR-0010) ---

use calc_i18n::Localizer;
use calc_shell::plain;
use calc_store::persist::{history as load_history, load_language};
use calc_store::{DocStore, FsStore};

fn scratch_store() -> (DocStore<FsStore>, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let store = DocStore::new(FsStore::new(dir.path()));
    (store, dir)
}

#[test]
fn submit_line_dispatches_save_and_persists() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.set_input("def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)");
    app.submit();
    app.set_input("save fib");
    app.submit_line(&app.input().to_string(), &store, &Localizer::resolve(Some("en"), &[]));
    assert_eq!(app.result(), "saved fib");
    assert_eq!(store.list_functions().unwrap().len(), 1);
    // commands must not enter history
    assert_eq!(app.history().len(), 1);
    assert!(app.input().is_empty());
}

#[test]
fn submit_line_evaluates_and_persists_history() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.set_input("2 + 3");
    app.submit_line("2 + 3", &store, &Localizer::resolve(Some("en"), &[]));
    assert_eq!(app.result(), "= 5");
    assert_eq!(load_history(&store).unwrap(), vec!["2 + 3  = 5".to_string()]);
}

#[test]
fn submit_line_reports_the_new_language() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    let new_lang = app.submit_line("language fr", &store, &Localizer::resolve(Some("en"), &[]));
    assert_eq!(new_lang, Some("fr".to_string()));
    assert_eq!(plain(app.result().to_string()), "language set to fr");
    assert_eq!(load_language(&store).unwrap(), Some("fr".to_string()));
}

#[test]
fn submit_line_keeps_graph_special_case() {
    let mut app = App::default();
    let (store, _keep) = scratch_store();
    app.submit_line("graph x ^ 2", &store, &Localizer::resolve(Some("en"), &[]));
    assert_eq!(app.result(), "graph: x ^ 2");
    assert!(app.graph().is_some());
    assert!(app.history().is_empty());
}
