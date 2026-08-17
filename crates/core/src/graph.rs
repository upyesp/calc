//! Graph command parsing and analysis (ADR-0006, ADR-0014): everything a
//! frontend needs to turn a `graph …` line into plottable data lives here —
//! the command grammar, per-curve sampling, points of interest (roots,
//! intersections, extrema), tables of values, and tick-step selection.
//! Frontends only render.

use std::collections::BTreeSet;

use crate::{eval, evaluate, parse, Env, EpherError, Expression, Sample, Value};

/// The default x (or t/θ) domain for a curve kind, when the command names no
/// bounds.
pub fn default_domain(kind: &CurveKind) -> (f64, f64) {
    match kind {
        CurveKind::Cartesian(_) => (-10.0, 10.0),
        CurveKind::Parametric { .. } | CurveKind::Polar(_) => (0.0, std::f64::consts::TAU),
    }
}

/// How a curve fills toward the plot edge (`y < f(x)` / `y > f(x)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    Below,
    Above,
}

/// The kind of curve requested by a graph command.
#[derive(Debug, Clone, PartialEq)]
pub enum CurveKind {
    Cartesian(Expression),
    Parametric { x: Expression, y: Expression },
    Polar(Expression),
}

/// A parsed graph command: what to plot and over which domain.
#[derive(Debug, Clone, PartialEq)]
pub struct CurveSpec {
    pub kind: CurveKind,
    pub domain: (f64, f64),
    pub fill: Option<Fill>,
}

/// A sampled curve ready to render — the seam payload every frontend holds.
#[derive(Debug, Clone, PartialEq)]
pub struct SampledCurve {
    /// What the user typed after `graph` (the accessible caption/legend text).
    pub source: String,
    pub kind: CurveKind,
    pub domain: (f64, f64),
    pub samples: Vec<Sample>,
    pub fill: Option<Fill>,
}

/// Parse the text after `graph ` into a [`CurveSpec`].
///
/// Grammar (case-sensitive, matching the expression language):
/// - `y = f(x)`-style: `expr`, optionally prefixed `y <`, `y <=`, `y >`,
///   `y >=` for region filling
/// - parametric: `param <x(t)>, <y(t)>`
/// - polar: `polar <r(θ)>`
/// - any form may end with `from a to b` (numeric bounds, expressions with
///   built-in constants allowed — the language has no `from` identifier, so
///   the keyword can never collide with the expression itself)
pub fn parse_graph_source(source: &str) -> Result<CurveSpec, EpherError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(EpherError::Parse("empty graph command".to_string()));
    }

    let (body, domain) = split_domain(source)?;
    let fill: Option<Fill>;
    let kind = if let Some(rest) = body.strip_prefix("param ") {
        fill = None;
        let parts = split_top_level(rest, ',');
        match parts.as_slice() {
            [x, y] => CurveKind::Parametric {
                x: parse(x.trim())?,
                y: parse(y.trim())?,
            },
            _ => {
                return Err(EpherError::Parse(
                    "parametric graphs need two expressions: `param <x(t)>, <y(t)>`"
                        .to_string(),
                ))
            }
        }
    } else if let Some(rest) = body.strip_prefix("polar ") {
        fill = None;
        CurveKind::Polar(parse(rest.trim())?)
    } else {
        let (expr, f) = match body.strip_prefix("y <=") {
            Some(r) => (r, Some(Fill::Below)),
            None => match body.strip_prefix("y >=") {
                Some(r) => (r, Some(Fill::Above)),
                None => match body.strip_prefix("y <") {
                    Some(r) => (r, Some(Fill::Below)),
                    None => match body.strip_prefix("y >") {
                        Some(r) => (r, Some(Fill::Above)),
                        None => (body, None),
                    },
                },
            },
        };
        fill = f;
        CurveKind::Cartesian(parse(expr.trim())?)
    };
    let domain = match domain {
        Some(d) => d,
        None => default_domain(&kind),
    };
    if domain.0 >= domain.1 {
        return Err(EpherError::Parse(format!(
            "graph domain must run low to high, got {:.3} .. {:.3}",
            domain.0, domain.1
        )));
    }
    Ok(CurveSpec {
        kind,
        domain,
        fill,
    })
}

/// A parsed `from a to b` domain suffix (or none), paired with the body
/// text that preceded it.
type DomainSplit<'a> = Result<(&'a str, Option<(f64, f64)>), EpherError>;

/// Split a trailing `from a to b` off the body; the bounds evaluate as
/// expressions (so `2*pi` works) over the built-in constants.
fn split_domain(source: &str) -> DomainSplit<'_> {
    let Some(idx) = source.rfind(" from ") else {
        return Ok((source, None));
    };
    let (body, bounds) = source.split_at(idx);
    let bounds = bounds.trim_start_matches(" from ");
    let Some((a, b)) = bounds.split_once(" to ") else {
        return Err(EpherError::Parse(
            "expected `from a to b` after the expression".to_string(),
        ));
    };
    let fa = evaluate(a.trim())?;
    let fb = evaluate(b.trim())?;
    let (Value::Float(a), Value::Float(b)) = (fa, fb) else {
        return Err(EpherError::Type("graph domain bounds must be numbers".to_string()));
    };
    Ok((body.trim(), Some((a, b))))
}

/// Split on a separator at paren depth zero (parametric commands use commas
/// while function calls may contain their own).
fn split_top_level(s: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            c if c == sep && depth == 0 => {
                parts.push(&s[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

/// Sample a parsed spec (ADR-0006: the core computes plot data).
pub fn sample_spec(spec: &CurveSpec, points: usize, env: &Env) -> Result<Vec<Sample>, EpherError> {
    let (a, b) = spec.domain;
    match &spec.kind {
        CurveKind::Cartesian(expr) => crate::sample(expr, a, b, points, env),
        CurveKind::Parametric { x, y } => crate::sample_parametric(x, y, a, b, points, env),
        CurveKind::Polar(expr) => crate::sample_polar(expr, a, b, points, env),
    }
}

/// The Cartesian expression, for curves that are `y = f(x)` (points-of-
/// interest analysis only applies to them).
pub fn cartesian_expr(kind: &CurveKind) -> Option<&Expression> {
    match kind {
        CurveKind::Cartesian(e) => Some(e),
        _ => None,
    }
}

/// Evaluate `expr` with `x` bound in a child environment (constant tables
/// and function tables stay visible; session bindings do not).
fn eval_at(expr: &Expression, x: f64, env: &Env) -> Option<f64> {
    let mut child = Env::new_child(env);
    child.set("x", Value::float(x));
    match eval(expr, &child) {
        Ok(Value::Float(v)) if v.is_finite() => Some(v),
        _ => None,
    }
}

/// A parsed `table` command: what to evaluate and over which x values.
/// Defaults match TI's table (start −5, end 5, 11 rows); `points` is
/// capped so a bad command can't demand unbounded work.
#[derive(Debug, Clone)]
pub struct TableSpec {
    pub expr: Expression,
    pub x_min: f64,
    pub x_max: f64,
    pub points: usize,
}

/// Parse the text after `table `: `expr [from a to b] [points n]`.
/// The language has no `from`/`to`/`points` identifiers, so the keywords
/// can never collide with the expression.
pub fn parse_table_source(source: &str) -> Result<TableSpec, EpherError> {
    const DEFAULT_POINTS: usize = 11;
    const MAX_POINTS: usize = 1000;

    let source = source.trim();
    if source.is_empty() {
        return Err(EpherError::Parse("empty table command".to_string()));
    }
    // The `points n` suffix sits after the domain, so strip it first.
    let (rest, points) = match source.rfind(" points ") {
        Some(idx) => {
            let (expr, n) = source.split_at(idx);
            let n = n.trim_start_matches(" points ").trim();
            let n: usize = n
                .parse()
                .map_err(|_| EpherError::Parse(format!("`points {n}` needs a whole number")))?;
            if !(1..=MAX_POINTS).contains(&n) {
                return Err(EpherError::Parse(format!(
                    "`points` must be between 1 and {MAX_POINTS}"
                )));
            }
            (expr.trim(), n)
        }
        None => (source, DEFAULT_POINTS),
    };
    let (body, domain) = split_domain(rest)?;
    let (x_min, x_max) = domain.unwrap_or((-5.0, 5.0));
    if x_min >= x_max {
        return Err(EpherError::Parse(format!(
            "table domain must run low to high, got {x_min:.3} .. {x_max:.3}"
        )));
    }
    Ok(TableSpec {
        expr: parse(body)?,
        x_min,
        x_max,
        points,
    })
}

/// A row of a table of values: x always present; y absent where the
/// expression has no value (TI-style blank rows).
pub fn table_rows(
    expr: &Expression,
    x_min: f64,
    x_max: f64,
    points: usize,
    env: &Env,
) -> Vec<(f64, Option<f64>)> {
    let mut out = Vec::new();
    for i in 0..points {
        let t = if points == 1 {
            0.0
        } else {
            i as f64 / (points - 1) as f64
        };
        let x = x_min + t * (x_max - x_min);
        out.push((x, eval_at(expr, x, env)));
    }
    out
}

/// What kind of notable point an [`InterestPoint`] marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterestKind {
    Root,
    Intersection,
    Maximum,
    Minimum,
}

/// A point of interest on a plot: roots and extrema of a curve, or the
/// intersection of two curves (ADR-0014).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterestPoint {
    pub kind: InterestKind,
    pub x: f64,
    pub y: f64,
}

impl InterestPoint {
    pub fn coords(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

/// Find roots, intersections, and extrema across the plotted curves.
/// Cartesian curves only (parametric/polar analysis is a documented
/// deferral); intersections need overlapping x domains.
pub fn analyze(curves: &[SampledCurve], env: &Env) -> Vec<InterestPoint> {
    let mut out = Vec::new();
    for (i, curve) in curves.iter().enumerate() {
        let Some(expr) = cartesian_expr(&curve.kind) else {
            continue;
        };
        roots_and_extrema(expr, &curve.samples, env, &mut out);
        for other in curves.iter().take(i) {
            if let Some(other_expr) = cartesian_expr(&other.kind) {
                intersections(expr, other_expr, curve, other, env, &mut out);
            }
        }
    }
    out.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    out.dedup_by(|a, b| (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6);
    out
}

/// Roots (sign changes over the sampled data, refined by bisection) and
/// local extrema (sampled turning points, refined by golden-section search).
fn roots_and_extrema(
    expr: &Expression,
    samples: &[Sample],
    env: &Env,
    out: &mut Vec<InterestPoint>,
) {
    let finite: Vec<&Sample> = samples.iter().filter(|s| s.y.is_finite()).collect();
    // Roots.
    for w in finite.windows(2) {
        let (a, b) = (w[0], w[1]);
        if a.y == 0.0 {
            out.push(InterestPoint {
                kind: InterestKind::Root,
                x: a.x,
                y: 0.0,
            });
        } else if a.y * b.y < 0.0 {
            if let Some(x) = bisect(expr, a.x, b.x, 0.0, env) {
                out.push(InterestPoint {
                    kind: InterestKind::Root,
                    x,
                    y: 0.0,
                });
            }
        }
    }
    if let Some(last) = finite.last() {
        if last.y == 0.0 {
            out.push(InterestPoint {
                kind: InterestKind::Root,
                x: last.x,
                y: 0.0,
            });
        }
    }
    // Extrema: a sample strictly above (or below) at least one neighbor and
    // no lower (higher) than the other — catches symmetric peaks where the
    // two neighbors tie, but never fires on a flat line (both sides strict).
    for w in finite.windows(3) {
        let (l, m, r) = (w[0], w[1], w[2]);
        if (m.y > l.y && m.y >= r.y) || (m.y >= l.y && m.y > r.y) {
            if let Some(x) = golden_extremum(expr, l.x, r.x, true, env) {
                out.push(InterestPoint {
                    kind: InterestKind::Maximum,
                    x,
                    y: eval_at(expr, x, env).unwrap_or(m.y),
                });
            }
        } else if (m.y < l.y && m.y <= r.y) || (m.y <= l.y && m.y < r.y) {
            if let Some(x) = golden_extremum(expr, l.x, r.x, false, env) {
                out.push(InterestPoint {
                    kind: InterestKind::Minimum,
                    x,
                    y: eval_at(expr, x, env).unwrap_or(m.y),
                });
            }
        }
    }
}

/// Intersections of two Cartesian curves over their overlapping domain:
/// sign changes of `f(x) - g(x)` on a shared grid, refined by bisection.
fn intersections(
    f: &Expression,
    g: &Expression,
    a: &SampledCurve,
    b: &SampledCurve,
    env: &Env,
    out: &mut Vec<InterestPoint>,
) {
    let lo = a.domain.0.max(b.domain.0);
    let hi = a.domain.1.min(b.domain.1);
    if lo >= hi {
        return;
    }
    const POINTS: usize = 160;
    let mut prev: Option<(f64, f64)> = None;
    for i in 0..POINTS {
        let x = lo + (hi - lo) * i as f64 / (POINTS - 1) as f64;
        let (Some(fx), Some(gx)) = (eval_at(f, x, env), eval_at(g, x, env)) else {
            prev = None;
            continue;
        };
        let d = fx - gx;
        if let Some((px, pd)) = prev {
            if d == 0.0 {
                out.push(InterestPoint {
                    kind: InterestKind::Intersection,
                    x,
                    y: fx,
                });
            } else if pd * d < 0.0 {
                if let Some(x) = bisect_diff(f, g, px, x, env) {
                    out.push(InterestPoint {
                        kind: InterestKind::Intersection,
                        x,
                        y: eval_at(f, x, env).unwrap_or(fx),
                    });
                }
            }
        }
        prev = Some((x, d));
    }
}

/// Bisect a sign change of `f(x) - target` on `(a, b)`.
fn bisect(expr: &Expression, mut a: f64, mut b: f64, target: f64, env: &Env) -> Option<f64> {
    let mut fa = eval_at(expr, a, env)? - target;
    for _ in 0..64 {
        let m = 0.5 * (a + b);
        let fm = eval_at(expr, m, env)? - target;
        if fa * fm <= 0.0 {
            b = m;
        } else {
            a = m;
            fa = fm;
        }
    }
    Some(0.5 * (a + b))
}

/// Bisect a sign change of `f(x) - g(x)` on `(a, b)`.
fn bisect_diff(f: &Expression, g: &Expression, mut a: f64, mut b: f64, env: &Env) -> Option<f64> {
    let mut da = eval_at(f, a, env)? - eval_at(g, a, env)?;
    for _ in 0..64 {
        let m = 0.5 * (a + b);
        let dm = eval_at(f, m, env)? - eval_at(g, m, env)?;
        if da * dm <= 0.0 {
            b = m;
        } else {
            a = m;
            da = dm;
        }
    }
    Some(0.5 * (a + b))
}

/// Golden-section search for a local extremum of `f` on `[a, b]`.
fn golden_extremum(
    expr: &Expression,
    a: f64,
    b: f64,
    maximum: bool,
    env: &Env,
) -> Option<f64> {
    let phi = 0.618_033_988_749_894_9;
    let (mut lo, mut hi) = (a, b);
    let mut c = hi - phi * (hi - lo);
    let mut d = lo + phi * (hi - lo);
    let better = |v: f64, w: f64| if maximum { v > w } else { v < w };
    for _ in 0..64 {
        let (fc, fd) = (eval_at(expr, c, env)?, eval_at(expr, d, env)?);
        if better(fc, fd) {
            hi = d;
            d = c;
            c = hi - phi * (hi - lo);
        } else {
            lo = c;
            c = d;
            d = lo + phi * (hi - lo);
        }
    }
    Some(0.5 * (lo + hi))
}

/// A "nice" tick step for a value span: 1, 2, or 5 × 10^k, aiming for at
/// most `target` intervals (both renderers grid to these steps).
pub fn nice_step(span: f64, target: usize) -> f64 {
    if !span.is_finite() || span <= 0.0 || target == 0 {
        return 1.0;
    }
    let raw = span / target as f64;
    let k = raw.log10().floor();
    let base = 10f64.powf(k);
    for m in [1.0, 2.0, 5.0, 10.0] {
        if m * base >= raw {
            return m * base;
        }
    }
    10f64.powf(k + 1.0)
}

/// Every variable name referenced anywhere in an expression (sliders bind
/// the constants among these — ADR-0014).
pub fn free_names(expr: &Expression, out: &mut BTreeSet<String>) {
    match expr {
        Expression::Literal(_) => {}
        Expression::Var(name) => {
            out.insert(name.clone());
        }
        Expression::Call(_, args) => {
            for a in args {
                free_names(a, out);
            }
        }
        Expression::Neg(e)
        | Expression::Factorial(e)
        | Expression::Not(e) => free_names(e, out),
        Expression::Add(a, b)
        | Expression::Sub(a, b)
        | Expression::Mul(a, b)
        | Expression::Div(a, b)
        | Expression::Pow(a, b)
        | Expression::And(a, b)
        | Expression::Or(a, b) => {
            free_names(a, out);
            free_names(b, out);
        }
        Expression::Compare(_, a, b) => {
            free_names(a, out);
            free_names(b, out);
        }
        Expression::If(a, b, c) => {
            free_names(a, out);
            free_names(b, out);
            free_names(c, out);
        }
    }
}
