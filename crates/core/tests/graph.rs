//! Tests for the graph command grammar, analysis, tables, and tick steps
//! (ADR-0014) — pure math, no rendering.

use epher_core::graph::{
    analyze, free_names, nice_step, parse_graph_source, sample_spec, table_rows, CurveKind, Fill,
    InterestKind, SampledCurve,
};
use epher_core::{parse, Env, Session, Value};

fn env() -> Env {
    Env::default()
}

fn float(v: &Value) -> f64 {
    match v {
        Value::Float(f) => *f,
        other => panic!("expected float, got {other:?}"),
    }
}

fn sampled(source: &str) -> SampledCurve {
    let spec = parse_graph_source(source).unwrap();
    let samples = sample_spec(&spec, 120, &env()).unwrap();
    SampledCurve {
        source: source.to_string(),
        kind: spec.kind,
        domain: spec.domain,
        samples,
        fill: spec.fill,
    }
}

#[test]
fn parses_plain_cartesian_with_default_domain() {
    let spec = parse_graph_source("x ^ 2").unwrap();
    assert!(matches!(spec.kind, CurveKind::Cartesian(_)));
    assert_eq!(spec.domain, (-10.0, 10.0));
    assert_eq!(spec.fill, None);
}

#[test]
fn parses_fill_prefixes() {
    assert_eq!(
        parse_graph_source("y < x ^ 2").unwrap().fill,
        Some(Fill::Below)
    );
    assert_eq!(
        parse_graph_source("y <= x ^ 2").unwrap().fill,
        Some(Fill::Below)
    );
    assert_eq!(
        parse_graph_source("y > sin(x)").unwrap().fill,
        Some(Fill::Above)
    );
    assert_eq!(
        parse_graph_source("y >= sin(x)").unwrap().fill,
        Some(Fill::Above)
    );
    assert_eq!(parse_graph_source("x + 1").unwrap().fill, None);
}

#[test]
fn parses_domain_bounds_including_expressions() {
    let spec = parse_graph_source("x ^ 2 from -5 to 5").unwrap();
    assert_eq!(spec.domain, (-5.0, 5.0));
    let spec = parse_graph_source("sin(x) from 0 to 2*pi").unwrap();
    assert!((spec.domain.0 - 0.0).abs() < 1e-12);
    assert!((spec.domain.1 - std::f64::consts::TAU).abs() < 1e-12);
}

#[test]
fn rejects_backwards_domain() {
    assert!(parse_graph_source("x from 5 to -5").is_err());
    assert!(parse_graph_source("x from 2 to 2").is_err());
}

#[test]
fn parses_parametric_with_commas_in_function_args() {
    let spec = parse_graph_source("param max(0, t), min(t, 1)").unwrap();
    assert!(matches!(spec.kind, CurveKind::Parametric { .. }));
    assert_eq!(spec.domain, (0.0, std::f64::consts::TAU));
    let spec = parse_graph_source("param t, t ^ 2 from 0 to 3").unwrap();
    assert_eq!(spec.domain, (0.0, 3.0));
}

#[test]
fn rejects_parametric_without_two_expressions() {
    assert!(parse_graph_source("param t").is_err());
    assert!(parse_graph_source("param t, t, t").is_err());
}

#[test]
fn parses_polar() {
    let spec = parse_graph_source("polar 2").unwrap();
    assert!(matches!(spec.kind, CurveKind::Polar(_)));
    assert_eq!(spec.domain, (0.0, std::f64::consts::TAU));
}

#[test]
fn samples_each_kind() {
    let spec = parse_graph_source("x ^ 2").unwrap();
    let s = sample_spec(&spec, 51, &env()).unwrap();
    assert_eq!(s.len(), 51);
    assert_eq!(s[0].x, -10.0);
    assert!((s[25].y - 0.0).abs() < 1e-9, "x=0 maps to y=0");

    let spec = parse_graph_source("param t, t").unwrap();
    let s = sample_spec(&spec, 50, &env()).unwrap();
    assert!((s[0].x - 0.0).abs() < 1e-9);
    assert!((s[0].y - 0.0).abs() < 1e-9);

    // A circle of radius 2: every sampled point lies on r = 2.
    let spec = parse_graph_source("polar 2").unwrap();
    let s = sample_spec(&spec, 64, &env()).unwrap();
    assert!(s.iter().all(|p| (p.x * p.x + p.y * p.y - 4.0).abs() < 1e-9));
}

#[test]
fn nice_steps_follow_one_two_five() {
    assert!((nice_step(10.0, 5) - 2.0).abs() < 1e-12);
    assert!((nice_step(100.0, 5) - 20.0).abs() < 1e-12);
    assert!((nice_step(1.0, 10) - 0.1).abs() < 1e-12);
    assert!((nice_step(7.0, 6) - 2.0).abs() < 1e-12);
    assert!((nice_step(0.03, 4) - 0.01).abs() < 1e-12);
}

#[test]
fn tables_keep_x_and_blank_undefined_rows() {
    let expr = parse("x ^ 2").unwrap();
    let rows = table_rows(&expr, -2.0, 2.0, 5, &env());
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0], (-2.0, Some(4.0)));
    assert_eq!(rows[2], (0.0, Some(0.0)));

    let expr = parse("1 / x").unwrap();
    let rows = table_rows(&expr, -1.0, 1.0, 5, &env());
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[2].1, None, "1/x is undefined at x=0");
    assert_eq!(rows[1].1, Some(-2.0));
    assert_eq!(rows[3].1, Some(2.0));
}

fn kinds(curves: &[SampledCurve]) -> Vec<(InterestKind, f64, f64)> {
    let pts = analyze(curves, &env());
    pts.iter().map(|p| (p.kind, p.x, p.y)).collect()
}

fn assert_point(pts: &[(InterestKind, f64, f64)], kind: InterestKind, x: f64, y: f64) {
    assert!(
        pts.iter()
            .any(|(k, px, py)| *k == kind && (px - x).abs() < 1e-4 && (py - y).abs() < 1e-4),
        "expected {kind:?} at ({x}, {y}) in {pts:?}"
    );
}

#[test]
fn finds_roots_by_sign_change() {
    let curves = [sampled("x ^ 2 - 1")];
    let pts = kinds(&curves);
    assert_point(&pts, InterestKind::Root, -1.0, 0.0);
    assert_point(&pts, InterestKind::Root, 1.0, 0.0);
}

#[test]
fn finds_extrema() {
    let curves = [sampled("-(x ^ 2) + 4")];
    let pts = kinds(&curves);
    assert_point(&pts, InterestKind::Maximum, 0.0, 4.0);

    let curves = [sampled("x ^ 2")];
    let pts = kinds(&curves);
    assert_point(&pts, InterestKind::Minimum, 0.0, 0.0);
}

#[test]
fn finds_intersections_between_curves() {
    let curves = [sampled("x ^ 2"), sampled("2 - x")];
    let pts = kinds(&curves);
    assert_point(&pts, InterestKind::Intersection, -2.0, 4.0);
    assert_point(&pts, InterestKind::Intersection, 1.0, 1.0);
}

#[test]
fn intersections_respect_domain_overlap() {
    // Second curve lives entirely left of the first: no overlap, so no
    // *intersections* (roots of the first curve may still appear).
    let curves = [
        sampled("x ^ 2 from 0 to 10"),
        sampled("x ^ 2 - 1 from -10 to -1"),
    ];
    assert!(analyze(&curves, &env())
        .iter()
        .all(|p| p.kind != InterestKind::Intersection));
}

#[test]
fn free_names_collects_variables_deeply() {
    let expr = parse("a * x ^ 2 + sin(b)").unwrap();
    let mut names = std::collections::BTreeSet::new();
    free_names(&expr, &mut names);
    assert_eq!(
        names,
        ["a", "b", "x"].iter().map(|s| s.to_string()).collect()
    );
}

#[test]
fn session_set_constant_updates_value_and_source() {
    let mut s = Session::new();
    s.submit("const a = 2");
    assert!((float(s.env().constant("a").unwrap()) - 2.0).abs() < 1e-12);
    s.set_constant("a", Value::float(3.5), "const a = 3.5".to_string());
    assert!((float(s.env().constant("a").unwrap()) - 3.5).abs() < 1e-12);
    assert_eq!(s.const_sources().get("a").unwrap(), "const a = 3.5");
}
