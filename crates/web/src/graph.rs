//! The web graph renderer (ADR-0006, ADR-0014): the core's sampler and
//! analyzer provide the data; this module turns multiple curves, points of
//! interest, and the trace cursor into SVG. Pure math in [`geometry`],
//! [`segments`], [`ticks`], and [`trace_nearest`] (unit-tested natively),
//! then two renderers over it: [`graph_svg`] (string form — tests and the
//! copy-to-clipboard button) and [`graph_html`] (Yew VNodes — the
//! production renderer, so the SVG lands in the proper namespace;
//! innerHTML-parsed `<svg>` is HTML-namespaced and invisible in WebKit).
//!
//! Accessibility (ADR-0009): the SVG is `role="img"` with a `title` and an
//! `aria-label` naming every plotted expression; the visible caption and
//! legend live next to it. Curve colors are CSS classes (`curve-0` …
//! `curve-3`, contrast-verified in `index.html`), and each index also gets
//! a distinct dash pattern so curves stay distinguishable without color
//! (WCAG 1.4.1). Axes/gridlines inherit `currentColor` at recorded
//! opacities (1.4.11).

use epher_core::graph::{InterestKind, SampledCurve, Segment3D, Surface, View3D};
use epher_core::Sample;
use wasm_bindgen::JsCast;
use yew::prelude::*;

pub const WIDTH: f64 = 640.0;
pub const HEIGHT: f64 = 400.0;
const LEFT: f64 = 48.0;
const RIGHT: f64 = 632.0;
const TOP: f64 = 12.0;
const BOTTOM: f64 = 368.0;

/// The plot geometry shared by every rendered layer: value ranges, tick
/// steps, and whether the horizontal zero axis belongs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub step_x: f64,
    pub step_y: f64,
    pub zero_axis: bool,
}

impl Geometry {
    /// Map data x → viewBox x.
    pub fn sx(&self, x: f64) -> f64 {
        LEFT + (x - self.x_min) / (self.x_max - self.x_min) * (RIGHT - LEFT)
    }

    /// Map data y → viewBox y.
    pub fn sy(&self, y: f64) -> f64 {
        TOP + (1.0 - (y - self.y_min) / (self.y_max - self.y_min)) * (BOTTOM - TOP)
    }

    /// Map a viewBox x back to data x.
    pub fn unx(&self, px: f64) -> f64 {
        self.x_min + (px - LEFT) / (RIGHT - LEFT) * (self.x_max - self.x_min)
    }

    /// Map a viewBox y back to data y.
    pub fn uny(&self, py: f64) -> f64 {
        self.y_min + (1.0 - (py - TOP) / (BOTTOM - TOP)) * (self.y_max - self.y_min)
    }
}

/// Compute the shared plot geometry for a set of curves: the union of their
/// domains, the y range padded 6%, and 1/2/5-style tick steps. `None` when
/// nothing can be drawn.
pub fn geometry(curves: &[SampledCurve]) -> Option<Geometry> {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    let mut any = false;
    for c in curves {
        if c.samples.is_empty() {
            continue;
        }
        x_min = x_min.min(c.domain.0);
        x_max = x_max.max(c.domain.1);
        for s in &c.samples {
            if s.y.is_finite() {
                y_min = y_min.min(s.y);
                y_max = y_max.max(s.y);
                any = true;
            }
        }
    }
    if !any || !x_min.is_finite() || x_max <= x_min {
        return None;
    }
    let y_span = (y_max - y_min).max(1e-9);
    let pad = y_span * 0.06;
    let (y_min, y_max) = (y_min - pad, y_max + pad);
    let y_span = y_max - y_min;
    Some(Geometry {
        x_min,
        x_max,
        y_min,
        y_max,
        step_x: epher_core::graph::nice_step(x_max - x_min, 10),
        step_y: epher_core::graph::nice_step(y_span, 8),
        zero_axis: y_min <= 0.0 && y_max >= 0.0,
    })
}

/// Split a curve's samples into polyline segments at non-finite points
/// (gaps, not jumps) *and* at vertical jumps larger than a third of the
/// sampled value range — a false asymptote line must never connect the two
/// branches of `1 / x` or `tan(x)`.
pub fn segments(samples: &[Sample], y_span: f64) -> Vec<Vec<(f64, f64)>> {
    let threshold = 0.35 * y_span;
    let mut out = Vec::new();
    let mut current: Vec<(f64, f64)> = Vec::new();
    for w in samples.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        if current.is_empty() && a.y.is_finite() {
            current.push((a.x, a.y));
        }
        if !b.y.is_finite() || (a.y.is_finite() && (b.y - a.y).abs() > threshold) {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
        } else {
            if current.is_empty() && a.y.is_finite() {
                current.push((a.x, a.y));
            }
            current.push((b.x, b.y));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Tick positions across `[lo, hi]` at the given step, snapping values
/// within a hair of zero (float drift) to exact 0.
pub fn ticks(lo: f64, hi: f64, step: f64) -> Vec<f64> {
    let mut out = Vec::new();
    if !lo.is_finite() || !hi.is_finite() || step <= 0.0 {
        return out;
    }
    let start = (lo / step).ceil() as i64;
    let end = (hi / step).floor() as i64;
    for i in start..=end {
        let v = i as f64 * step;
        out.push(if v.abs() < step * 1e-9 { 0.0 } else { v });
    }
    out
}

/// A readable label for a tick value: up to 3 decimals, trailing zeros
/// trimmed, no exponent surprises for graph-scale numbers.
pub fn label(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    let s = if s == "-0" { "0" } else { s };
    s.to_string()
}

/// XML-escape text that lands in SVG attributes and elements.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// A point of interest as the renderer sees it: a localized kind label and
/// its coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct Poi {
    pub kind: InterestKind,
    pub label: String,
    pub x: f64,
    pub y: f64,
}

/// The trace cursor: which curve and sample, with its data coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TracePoint {
    pub curve: usize,
    pub index: usize,
    pub x: f64,
    pub y: f64,
}

/// The nearest sample point to a viewBox position, within a 100px radius.
pub fn trace_nearest(
    curves: &[SampledCurve],
    geom: &Geometry,
    px: f64,
    py: f64,
) -> Option<TracePoint> {
    const MAX_D2: f64 = 100.0 * 100.0;
    let mut best: Option<(f64, TracePoint)> = None;
    for (ci, c) in curves.iter().enumerate() {
        for (si, s) in c.samples.iter().enumerate() {
            if !s.y.is_finite() {
                continue;
            }
            let d2 = (geom.sx(s.x) - px).powi(2) + (geom.sy(s.y) - py).powi(2);
            if d2 < MAX_D2 && best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
                best = Some((
                    d2,
                    TracePoint {
                        curve: ci,
                        index: si,
                        x: s.x,
                        y: s.y,
                    },
                ));
            }
        }
    }
    best.map(|(_, t)| t)
}

/// A visible legend entry: which curve index, and its display text.
pub fn curve_caption(c: &SampledCurve) -> String {
    match &c.kind {
        epher_core::graph::CurveKind::Cartesian(_) => format!("y = {}", c.source.trim()),
        _ => c.source.trim().to_string(),
    }
}

/// The aria-label listing every plotted expression.
pub fn aria_label(curves: &[SampledCurve]) -> String {
    let names: Vec<String> = curves.iter().map(curve_caption).collect();
    format!("Graph of {}", names.join(", "))
}

fn polyline_points(seg: &[(f64, f64)], geom: &Geometry) -> String {
    seg.iter()
        .map(|(x, y)| format!("{:.1},{:.1}", geom.sx(*x), geom.sy(*y)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Fill polygon points for a curve with a region fill: the curve, then the
/// bottom (or top) edge of the plot, closed back to the start.
fn fill_points(seg: &[(f64, f64)], below: bool, geom: &Geometry) -> String {
    let edge = if below { BOTTOM } else { TOP };
    let mut pts: Vec<String> = seg
        .iter()
        .map(|(x, y)| format!("{:.1},{:.1}", geom.sx(*x), geom.sy(*y)))
        .collect();
    let first = seg.first().map(|(x, _)| *x).unwrap_or(0.0);
    let last = seg.last().map(|(x, _)| *x).unwrap_or(0.0);
    pts.push(format!("{:.1},{edge:.1}", geom.sx(last)));
    pts.push(format!("{:.1},{edge:.1}", geom.sx(first)));
    pts.join(" ")
}

/// The shared SVG layer stack (grid, axes, tick labels) as a string.
fn layers_svg(geom: &Geometry, x_axis: bool) -> String {
    let mut s = String::new();
    let x_min = geom.x_min;
    let x_max = geom.x_max;
    let (x0, x1) = (geom.sx(x_min), geom.sx(x_max));
    let (y0, y1) = (geom.sy(geom.y_min), geom.sy(geom.y_max));

    // Gridlines.
    for v in ticks(x_min, x_max, geom.step_x) {
        if v.abs() > (x_max - x_min) * 1e-9 {
            let x = geom.sx(v);
            s.push_str(&format!(
                "<line class=\"grid\" x1=\"{x:.1}\" y1=\"{TOP:.1}\" x2=\"{x:.1}\" y2=\"{BOTTOM:.1}\" />"
            ));
        }
    }
    for v in ticks(geom.y_min, geom.y_max, geom.step_y) {
        if v.abs() > (geom.y_max - geom.y_min) * 1e-9 {
            let y = geom.sy(v);
            s.push_str(&format!(
                "<line class=\"grid\" x1=\"{LEFT:.1}\" y1=\"{y:.1}\" x2=\"{RIGHT:.1}\" y2=\"{y:.1}\" />"
            ));
        }
    }

    // Axes: x = 0 only when it lies inside the plotted domain; y = 0 only
    // when it lies inside the value range.
    if x_min <= 0.0 && x_max >= 0.0 {
        let x = geom.sx(0.0);
        s.push_str(&format!(
            "<line class=\"axis\" x1=\"{x:.1}\" y1=\"{TOP:.1}\" x2=\"{x:.1}\" y2=\"{BOTTOM:.1}\" />"
        ));
    }
    if x_axis {
        let y = geom.sy(0.0);
        s.push_str(&format!(
            "<line class=\"axis\" x1=\"{LEFT:.1}\" y1=\"{y:.1}\" x2=\"{RIGHT:.1}\" y2=\"{y:.1}\" />"
        ));
    }

    // Tick labels: x along the bottom, y along the left.
    for v in ticks(x_min, x_max, geom.step_x) {
        let x = geom.sx(v);
        s.push_str(&format!(
            "<text class=\"tick\" x=\"{x:.1}\" y=\"{:.1}\" text-anchor=\"middle\">{}</text>",
            HEIGHT - 6.0,
            escape(&label(v))
        ));
    }
    for v in ticks(geom.y_min, geom.y_max, geom.step_y) {
        let y = geom.sy(v);
        s.push_str(&format!(
            "<text class=\"tick\" x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\">{}</text>",
            LEFT - 4.0,
            y + 4.0,
            escape(&label(v))
        ));
    }

    // Frame markers so the plot area reads as bounded without a heavy box.
    let _ = (x0, x1, y0, y1);
    s
}

/// Render curves, points of interest, and the trace cursor as an inline SVG
/// string (tests and the copy button; the app renders [`graph_html`]).
/// Nothing to draw renders the empty string.
pub fn graph_svg(curves: &[SampledCurve], pois: &[Poi], trace: Option<TracePoint>) -> String {
    let Some(geom) = geometry(curves) else {
        return String::new();
    };
    let y_span = geom.y_max - geom.y_min;

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg viewBox=\"0 0 {WIDTH} {HEIGHT}\" role=\"img\" aria-label=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">",
        escape(&aria_label(curves))
    ));
    svg.push_str(&format!("<title>{}</title>", escape(&aria_label(curves))));
    svg.push_str(&layers_svg(&geom, geom.zero_axis));

    for (i, c) in curves.iter().enumerate() {
        let segs = segments(&c.samples, y_span);
        if let Some(fill) = c.fill {
            let below = matches!(fill, epher_core::graph::Fill::Below);
            for seg in &segs {
                svg.push_str(&format!(
                    "<polygon class=\"fill curve-{i}\" points=\"{}\" fill=\"currentColor\" fill-opacity=\"0.18\" />",
                    fill_points(seg, below, &geom)
                ));
            }
        }
        for seg in &segs {
            svg.push_str(&format!(
                "<polyline class=\"curve curve-{i}\" points=\"{}\" fill=\"none\" />",
                polyline_points(seg, &geom)
            ));
        }
    }

    for p in pois {
        let (x, y) = (geom.sx(p.x), geom.sy(p.y));
        svg.push_str(&format!(
            "<circle class=\"poi\" cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"4\" />"
        ));
        svg.push_str(&format!(
            "<text class=\"poi-label\" x=\"{:.1}\" y=\"{:.1}\">{}</text>",
            x + 7.0,
            y - 7.0,
            escape(&format!("{} ({}, {})", p.label, label(p.x), label(p.y)))
        ));
    }

    if let Some(t) = trace {
        let (x, y) = (geom.sx(t.x), geom.sy(t.y));
        svg.push_str(&format!(
            "<circle class=\"trace\" cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"5\" />"
        ));
    }

    svg.push_str("</svg>");
    svg
}

/// Render the same layers as Yew SVG VNodes — the production renderer.
/// Yew creates SVG elements in the SVG namespace, so the plot actually
/// paints in every engine (innerHTML-parsed SVG does not, in WebKit).
/// Pointer/keyboard interaction uses native listeners (gloo-events) bound
/// to the element through a NodeRef — Yew's synthetic event delegation
/// does not reach SVG children.
#[derive(Properties, PartialEq)]
pub struct GraphProps {
    pub curves: Vec<SampledCurve>,
    pub pois: Vec<Poi>,
    pub trace: Option<TracePoint>,
    /// Mouse move/tap over the plot: viewBox coordinates.
    pub on_trace: Callback<(f64, f64)>,
    /// Keyboard input while the plot has focus (arrow-key tracing).
    pub on_key: Callback<web_sys::KeyboardEvent>,
    /// End of pointer interaction: hide the trace cursor.
    pub on_leave: Callback<()>,
}

#[function_component(Graph)]
pub fn graph_html(props: &GraphProps) -> Html {
    let svg_ref = use_node_ref();

    // Attach the interaction listeners once, directly on the SVG element.
    {
        let svg_ref = svg_ref.clone();
        let on_trace = props.on_trace.clone();
        let on_key = props.on_key.clone();
        let on_leave = props.on_leave.clone();
        let listeners = use_state(Vec::<gloo_events::EventListener>::new);
        use_effect_with((), move |_| {
            let Some(el) = svg_ref.cast::<web_sys::Element>() else {
                return;
            };
            let mut bound = Vec::new();
            {
                let el_closure = el.clone();
                let on_trace = on_trace.clone();
                bound.push(gloo_events::EventListener::new(
                    &el,
                    "mousemove",
                    move |e| {
                        let Some(me) = e.dyn_ref::<web_sys::MouseEvent>() else {
                            return;
                        };
                        let w = el_closure.client_width().max(1) as f64;
                        let h = el_closure.client_height().max(1) as f64;
                        let px = me.offset_x() as f64 * WIDTH / w;
                        let py = me.offset_y() as f64 * HEIGHT / h;
                        on_trace.emit((px, py));
                    },
                ));
            }
            {
                let el_closure = el.clone();
                let on_trace = on_trace.clone();
                bound.push(gloo_events::EventListener::new(&el, "click", move |e| {
                    let Some(me) = e.dyn_ref::<web_sys::MouseEvent>() else {
                        return;
                    };
                    let w = el_closure.client_width().max(1) as f64;
                    let h = el_closure.client_height().max(1) as f64;
                    let px = me.offset_x() as f64 * WIDTH / w;
                    let py = me.offset_y() as f64 * HEIGHT / h;
                    on_trace.emit((px, py));
                }));
            }
            {
                let el = el.clone();
                let on_key = on_key.clone();
                bound.push(gloo_events::EventListener::new(&el, "keydown", move |e| {
                    if let Some(ke) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                        on_key.emit(ke.clone());
                    }
                }));
            }
            {
                let el = el.clone();
                let on_leave = on_leave.clone();
                bound.push(gloo_events::EventListener::new(&el, "blur", move |_| {
                    on_leave.emit(());
                }));
            }
            listeners.set(bound);
        });
    }
    let Some(geom) = geometry(&props.curves) else {
        return html! {};
    };
    let y_span = geom.y_max - geom.y_min;

    let mut curve_layers = Vec::new();
    for (i, c) in props.curves.iter().enumerate() {
        let segs = segments(&c.samples, y_span);
        if let Some(fill) = c.fill {
            let below = matches!(fill, epher_core::graph::Fill::Below);
            for seg in &segs {
                curve_layers.push(html! {
                    <polygon class={format!("fill curve-{i}")} points={fill_points(seg, below, &geom)} fill="currentColor" fill-opacity="0.18" />
                });
            }
        }
        for seg in &segs {
            curve_layers.push(html! {
                <polyline class={format!("curve curve-{i}")} points={polyline_points(seg, &geom)} fill="none" />
            });
        }
    }

    let mut grid_lines = Vec::new();
    for v in ticks(geom.x_min, geom.x_max, geom.step_x) {
        if v.abs() > (geom.x_max - geom.x_min) * 1e-9 {
            let x = geom.sx(v);
            grid_lines.push(html! {
                <line class="grid" x1={x.to_string()} y1={TOP.to_string()} x2={x.to_string()} y2={BOTTOM.to_string()} />
            });
        }
    }
    for v in ticks(geom.y_min, geom.y_max, geom.step_y) {
        if v.abs() > (geom.y_max - geom.y_min) * 1e-9 {
            let y = geom.sy(v);
            grid_lines.push(html! {
                <line class="grid" x1={LEFT.to_string()} y1={y.to_string()} x2={RIGHT.to_string()} y2={y.to_string()} />
            });
        }
    }

    let x_axis = (geom.x_min <= 0.0 && geom.x_max >= 0.0).then(|| {
        let x = geom.sx(0.0);
        html! {
            <line class="axis" x1={x.to_string()} y1={TOP.to_string()} x2={x.to_string()} y2={BOTTOM.to_string()} />
        }
    });
    let y_axis = geom.zero_axis.then(|| {
        let y = geom.sy(0.0);
        html! {
            <line class="axis" x1={LEFT.to_string()} y1={y.to_string()} x2={RIGHT.to_string()} y2={y.to_string()} />
        }
    });

    let mut x_labels = Vec::new();
    for v in ticks(geom.x_min, geom.x_max, geom.step_x) {
        let x = geom.sx(v);
        x_labels.push(html! {
            <text class="tick" x={x.to_string()} y={(HEIGHT - 6.0).to_string()} text-anchor="middle">{ label(v) }</text>
        });
    }
    let mut y_labels = Vec::new();
    for v in ticks(geom.y_min, geom.y_max, geom.step_y) {
        let y = geom.sy(v);
        y_labels.push(html! {
            <text class="tick" x={(LEFT - 4.0).to_string()} y={(y + 4.0).to_string()} text-anchor="end">{ label(v) }</text>
        });
    }

    let mut poi_nodes = Vec::new();
    for p in &props.pois {
        let (x, y) = (geom.sx(p.x), geom.sy(p.y));
        let text = format!("{} ({}, {})", p.label, label(p.x), label(p.y));
        poi_nodes.push(html! {
            <circle class="poi" cx={x.to_string()} cy={y.to_string()} r="4" />
        });
        poi_nodes.push(html! {
            <text class="poi-label" x={(x + 7.0).to_string()} y={(y - 7.0).to_string()}>{ text }</text>
        });
    }

    let trace_node = props.trace.map(|t| {
        let x = geom.sx(t.x);
        let y = geom.sy(t.y);
        html! {
            <circle class="trace" cx={x.to_string()} cy={y.to_string()} r="5" />
        }
    });

    html! {
        <svg ref={svg_ref} viewBox={format!("0 0 {WIDTH} {HEIGHT}")} role="img" aria-label={aria_label(&props.curves)} tabindex="0" xmlns="http://www.w3.org/2000/svg">
            <title>{ aria_label(&props.curves) }</title>
            { for grid_lines }
            { x_axis }
            { y_axis }
            { for x_labels }
            { for y_labels }
            { for curve_layers }
            { for poi_nodes }
            { trace_node }
        </svg>
    }
}

// ===== 3D surfaces (ADR-0015) =====

/// Render the plotted surfaces as SVG content: mesh lines per grid row and
/// column with per-line depth shading (nearer lines more opaque), the
/// ground square and axes of the first surface on top, all painter-sorted
/// far to near. Built as a string (not diffed elements) so orbiting a
/// thousand-line mesh stays cheap. Returns (viewBox, inner content) — the
/// component puts both on one <svg> element.
pub fn surface_svg(surfaces: &[Surface], view: &View3D) -> Option<(String, String)> {
    use epher_core::graph::{project_mesh, surface_frame, Polyline3D};
    if surfaces.is_empty() {
        return None;
    }
    let mut mesh: Vec<Polyline3D> = Vec::new();
    for s in surfaces {
        mesh.extend(project_mesh(s, view));
    }
    let frame: Vec<Segment3D> = surface_frame(&surfaces[0], view);
    if mesh.is_empty() && frame.is_empty() {
        return None;
    }
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    let mut z_min = f64::INFINITY;
    let mut z_max = f64::NEG_INFINITY;
    for line in &mesh {
        for &(x, y) in &line.points {
            x_min = x_min.min(x);
            x_max = x_max.max(x);
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        }
        z_min = z_min.min(line.depth);
        z_max = z_max.max(line.depth);
    }
    for seg in &frame {
        x_min = x_min.min(seg.x1).min(seg.x2);
        x_max = x_max.max(seg.x1).max(seg.x2);
        y_min = y_min.min(seg.y1).min(seg.y2);
        y_max = y_max.max(seg.y1).max(seg.y2);
    }
    if !x_min.is_finite() || x_max - x_min < 1e-9 || y_max - y_min < 1e-9 {
        return None;
    }
    let pad = (x_max - x_min).max(y_max - y_min) * 0.06;
    let x_min = x_min - pad;
    let x_max = x_max + pad;
    let y_min = y_min - pad;
    let y_max = y_max + pad;
    let span = z_max - z_min;
    let view_box = format!("{x_min:.3} {y_min:.3} {:.3} {:.3}", x_max - x_min, y_max - y_min);
    let mut parts = String::new();
    // Painter's order: project_mesh already sorts far-to-near, so drawing
    // in order lets nearer lines overpaint farther ones.
    for line in &mesh {
        let t = if span < 1e-9 {
            1.0
        } else {
            ((line.depth - z_min) / span).clamp(0.0, 1.0)
        };
        // Depth cue without color: opacity 0.35 far → 0.95 near.
        let opacity = 0.35 + 0.6 * t;
        let points = line
            .points
            .iter()
            .map(|(x, y)| format!("{x:.3},{y:.3}"))
            .collect::<Vec<_>>()
            .join(" ");
        parts.push_str(&format!(
            "<polyline points=\"{points}\" fill=\"none\" stroke=\"currentColor\" stroke-opacity=\"{opacity:.3}\" stroke-width=\"1.2\"/>"
        ));
    }
    for seg in &frame {
        parts.push_str(&format!(
            "<line x1=\"{:.3}\" y1=\"{:.3}\" x2=\"{:.3}\" y2=\"{:.3}\" stroke=\"currentColor\" stroke-width=\"1.4\" stroke-opacity=\"0.9\"/>",
            seg.x1, seg.y1, seg.x2, seg.y2
        ));
    }
    Some((view_box, parts))
}

/// The orbit + keyboard interaction surface for a 3D plot: drag rotates,
/// arrow keys rotate (ADR-0015, WCAG 2.1.1). The SVG content is raw HTML
/// (innerHTML-style) so thousand-line meshes re-render without diffing.
#[derive(Properties, PartialEq)]
pub struct Graph3DProps {
    pub view_box: String,
    pub content: String,
    pub aria_label: String,
    /// (dyaw, dpitch) from a drag or arrow key.
    pub on_orbit: Callback<(f64, f64)>,
}

#[function_component(Graph3D)]
pub fn graph3d_html(props: &Graph3DProps) -> Html {
    let svg_ref = use_node_ref();
    let g_ref = use_node_ref();
    let drag = use_state(|| std::rc::Rc::new(std::cell::RefCell::new(Option::<(f64, f64)>::None)));

    // The mesh is injected with Element::set_inner_html on an SVG <g>, not
    // via Yew vnodes: Yew's from_html_unchecked parses fragments in an HTML
    // <div>, so the polyline nodes would carry the HTML namespace and the
    // SVG renderer would never paint them (blank plot in every browser).
    {
        let g_ref = g_ref.clone();
        let content = props.content.clone();
        use_effect_with(content, move |content| {
            if let Some(el) = g_ref.cast::<web_sys::Element>() {
                el.set_inner_html(content);
            }
        });
    }

    {
        let svg_ref = svg_ref.clone();
        let on_orbit = props.on_orbit.clone();
        let drag = drag.clone();
        let listeners = use_state(Vec::<gloo_events::EventListener>::new);
        use_effect_with((), move |_| {
            let Some(el) = svg_ref.cast::<web_sys::Element>() else {
                return;
            };
            let mut bound = Vec::new();
            {
                let el_closure = el.clone();
                let drag = drag.clone();
                bound.push(gloo_events::EventListener::new(&el, "pointerdown", move |e| {
                    if let Some(pe) = e.dyn_ref::<web_sys::PointerEvent>() {
                        el_closure.set_pointer_capture(pe.pointer_id()).ok();
                        *drag.borrow_mut() = Some((pe.client_x() as f64, pe.client_y() as f64));
                    }
                }));
            }
            {
                let el = el.clone();
                let drag = drag.clone();
                let on_orbit = on_orbit.clone();
                bound.push(gloo_events::EventListener::new(&el, "pointermove", move |e| {
                    if let Some(pe) = e.dyn_ref::<web_sys::PointerEvent>() {
                        if let Some((lx, ly)) = *drag.borrow() {
                            let dx = pe.client_x() as f64 - lx;
                            let dy = pe.client_y() as f64 - ly;
                            *drag.borrow_mut() = Some((pe.client_x() as f64, pe.client_y() as f64));
                            if dx.abs() > 0.5 || dy.abs() > 0.5 {
                                on_orbit.emit((dx * 0.01, dy * 0.01));
                            }
                        }
                    }
                }));
            }
            {
                let drag = drag.clone();
                bound.push(gloo_events::EventListener::new(&el, "pointerup", move |_| {
                    *drag.borrow_mut() = None;
                }));
            }
            {
                let drag = drag.clone();
                bound.push(gloo_events::EventListener::new(&el, "pointerleave", move |_| {
                    *drag.borrow_mut() = None;
                }));
            }
            {
                let el = el.clone();
                let on_orbit = on_orbit.clone();
                bound.push(gloo_events::EventListener::new(&el, "keydown", move |e| {
                    if let Some(ke) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                        let (dyaw, dpitch) = match ke.key().as_str() {
                            "ArrowLeft" => (-0.15, 0.0),
                            "ArrowRight" => (0.15, 0.0),
                            "ArrowUp" => (0.0, 0.15),
                            "ArrowDown" => (0.0, -0.15),
                            _ => return,
                        };
                        ke.prevent_default();
                        on_orbit.emit((dyaw, dpitch));
                    }
                }));
            }
            listeners.set(bound);
        });
    }

    html! {
        <svg
            ref={svg_ref}
            tabindex="0"
            role="img"
            aria-label={props.aria_label.clone()}
            viewBox={props.view_box.clone()}
            class="graph3d-svg"
        >
            <g ref={g_ref}></g>
        </svg>
    }
}
