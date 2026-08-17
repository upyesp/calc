use epher_core::Sample;
use epher_core::Session;
use epher_tui::{render_ascii, App};

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
    assert_eq!(app.graph().len(), 1);
    assert_eq!(app.graph()[0].samples.len(), 120);
    app.submit_graph("1 / x").expect("graph should sample");
    assert_eq!(app.graph().len(), 2, "curves overlay");
}

#[test]
fn graph_command_uses_session_functions() {
    let mut app = App::default();
    app.set_input("def f(x) = x ^ 3");
    app.submit();
    app.submit_graph("f(x)").expect("graph should sample");
    assert_eq!(app.graph().len(), 1);
    assert_eq!(app.graph()[0].samples.len(), 120);
}

#[test]
fn graph_command_records_source_for_caption() {
    let mut app = App::default();
    assert_eq!(app.graph().len(), 0);
    app.submit_graph("x ^ 2").expect("graph should sample");
    assert_eq!(app.graph()[0].source, "x ^ 2");
}

#[test]
fn graph_clear_empties_the_plot() {
    let mut app = App::default();
    app.submit_graph("x ^ 2").expect("graph should sample");
    app.submit_graph("clear").expect("clear should work");
    assert!(app.graph().is_empty());
    assert!(app.pois().is_empty());
}

#[test]
fn graph_reports_points_of_interest() {
    let mut app = App::default();
    app.submit_graph("x ^ 2 - 1").expect("graph should sample");
    let pois = app.pois();
    assert!(
        pois.iter()
            .any(|p| p.kind == epher_core::graph::InterestKind::Root && (p.x - 1.0).abs() < 1e-3),
        "root near x=1 in {pois:?}"
    );
    app.submit_graph("2 - x").expect("graph should sample");
    assert!(app
        .pois()
        .iter()
        .any(|p| p.kind == epher_core::graph::InterestKind::Intersection));
}

#[test]
fn graph_parses_parametric_polar_and_domains() {
    let mut app = App::default();
    app.submit_graph("param t, t ^ 2 from 0 to 3")
        .expect("parametric should sample");
    app.submit_graph("polar 2").expect("polar should sample");
    assert_eq!(app.graph().len(), 2);
    assert!(app.submit_graph("x from 5 to -5").is_err());
}

fn curve_of(ys: &[f64]) -> epher_core::graph::SampledCurve {
    let samples = ys
        .iter()
        .enumerate()
        .map(|(i, y)| Sample {
            x: i as f64,
            y: *y,
        })
        .collect::<Vec<_>>();
    let expr = epher_core::parse("0").unwrap();
    epher_core::graph::SampledCurve {
        source: "test".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr),
        domain: (0.0, (ys.len() - 1) as f64),
        samples,
        fill: None,
    }
}

#[test]
fn render_ascii_plots_a_diagonal() {
    let samples = vec![
        Sample { x: 0.0, y: 0.0 },
        Sample { x: 1.0, y: 1.0 },
        Sample { x: 2.0, y: 2.0 },
    ];
    let curves = [curve_of(&[0.0, 1.0, 2.0])];
    assert_eq!(render_ascii(&curves, 3, 3), "··o\n·o·\no··");
    let _ = samples;
}

#[test]
fn render_ascii_handles_empty_and_non_finite() {
    assert_eq!(render_ascii(&[], 3, 3), "");
    let expr = epher_core::parse("0").unwrap();
    let c = epher_core::graph::SampledCurve {
        source: "test".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr),
        domain: (0.0, 1.0),
        samples: vec![
            Sample { x: f64::NAN, y: 0.0 },
            Sample {
                x: 0.0,
                y: f64::INFINITY,
            },
            Sample { x: 1.0, y: 1.0 },
        ],
        fill: None,
    };
    let out = render_ascii(&[c], 3, 3);
    assert!(out.contains('o'));
    assert!(!out.contains("NaN"));
}

#[test]
fn render_ascii_marks_axes_when_zero_is_inside() {
    // y = x on [-2, 2]: zero is strictly inside both ranges, so a vertical
    // and a horizontal axis must appear (the curve glyph wins on overlap).
    let expr = epher_core::parse("x").unwrap();
    let c = epher_core::graph::SampledCurve {
        source: "x".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr),
        domain: (-2.0, 2.0),
        samples: vec![
            Sample { x: -2.0, y: -2.0 },
            Sample { x: -1.0, y: -1.0 },
            Sample { x: 0.0, y: 0.0 },
            Sample { x: 1.0, y: 1.0 },
            Sample { x: 2.0, y: 2.0 },
        ],
        fill: None,
    };
    let out = render_ascii(&[c], 5, 5);
    assert!(out.contains('|'), "vertical axis: {out}");
    assert!(out.contains('-'), "horizontal axis: {out}");
}

#[test]
fn render_ascii_uses_distinct_glyphs_and_fills() {
    let expr = epher_core::parse("0").unwrap();
    let a = epher_core::graph::SampledCurve {
        source: "a".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr.clone()),
        domain: (0.0, 1.0),
        samples: vec![Sample { x: 0.0, y: 0.0 }, Sample { x: 1.0, y: 1.0 }],
        fill: Some(epher_core::graph::Fill::Below),
    };
    let b = epher_core::graph::SampledCurve {
        source: "b".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr),
        domain: (0.0, 1.0),
        samples: vec![Sample { x: 0.0, y: 1.0 }, Sample { x: 1.0, y: 0.0 }],
        fill: None,
    };
    let out = render_ascii(&[a, b], 4, 4);
    assert!(out.contains('o'), "first curve glyph: {out}");
    assert!(out.contains('x'), "second curve glyph: {out}");
    assert!(out.contains('.'), "fill shading: {out}");
}

// --- shell commands through the App seam (ADR-0010) ---

use epher_i18n::Localizer;
use epher_shell::plain;
use epher_store::persist::{history as load_history, load_language};
use epher_store::{DocStore, FsStore};

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
    assert_eq!(app.graph().len(), 1);
    assert!(app.history().is_empty());
}
