//! The web graph renderer (ADR-0006): the core's Sampler provides the
//! points; this module turns them into SVG. Pure math in [`plot_data`]
//! (unit-tested natively), then two renderers over it: [`graph_svg`]
//! (string form, also natively tested) and [`graph_html`] (Yew VNodes —
//! the production renderer, so the SVG lands in the proper namespace;
//! innerHTML-parsed `<svg>` is HTML-namespaced and invisible in WebKit).
//!
//! Accessibility (ADR-0009): the SVG is `role="img"` with a `title` and an
//! `aria-label` naming the plotted expression; the visible caption lives
//! next to it in the component. Colors inherit `currentColor` so the theme
//! controls contrast (the plot area color must stay >= 3:1, WCAG 1.4.11).

use calc_core::Sample;
use yew::html;
use yew::prelude::Html;

const WIDTH: f64 = 640.0;
const HEIGHT: f64 = 400.0;
const LEFT: f64 = 45.0;
const RIGHT: f64 = 625.0;
const TOP: f64 = 15.0;
const BOTTOM: f64 = 365.0;

/// The geometry of a plot: the value range, whether a horizontal zero axis
/// belongs, and the curve as segments split at non-finite points (gaps,
/// not jumps). Pure data — both renderers consume it.
#[derive(Debug, PartialEq)]
pub struct PlotData {
    pub y_min: f64,
    pub y_max: f64,
    pub zero_axis: bool,
    /// Each segment is (x, y) pairs in plot (data) coordinates.
    pub segments: Vec<Vec<(f64, f64)>>,
}

/// Compute the plot geometry; `None` when nothing can be drawn.
pub fn plot_data(samples: &[Sample]) -> Option<PlotData> {
    let finite: Vec<&Sample> = samples.iter().filter(|s| s.y.is_finite()).collect();
    if finite.is_empty() {
        return None;
    }
    let y_min = finite.iter().map(|s| s.y).fold(f64::INFINITY, f64::min);
    let y_max = finite.iter().map(|s| s.y).fold(f64::NEG_INFINITY, f64::max);

    let mut segments = Vec::new();
    let mut current: Vec<(f64, f64)> = Vec::new();
    for s in samples {
        if s.y.is_finite() {
            current.push((s.x, s.y));
        } else if !current.is_empty() {
            segments.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }

    Some(PlotData {
        y_min,
        y_max,
        zero_axis: y_min <= 0.0 && y_max >= 0.0,
        segments,
    })
}

fn sx(x: f64) -> f64 {
    LEFT + (x + 10.0) / 20.0 * (RIGHT - LEFT)
}

fn sy(y: f64, y_min: f64, y_span: f64) -> f64 {
    TOP + (1.0 - (y - y_min) / y_span) * (BOTTOM - TOP)
}

/// A readable label for a tick value: up to 3 decimals, trailing zeros
/// trimmed, no exponent surprises for graph-scale numbers.
fn label(v: f64) -> String {
    let s = format!("{:.3}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
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

/// Render `y = source` samples as an inline SVG string (used by tests and
/// tooling; the app renders [`graph_html`]). Empty input renders nothing.
pub fn graph_svg(samples: &[Sample], source: &str) -> String {
    let Some(data) = plot_data(samples) else {
        return String::new();
    };
    let y_span = (data.y_max - data.y_min).max(1e-12);

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg viewBox=\"0 0 {WIDTH} {HEIGHT}\" role=\"img\" aria-label=\"Graph of y = {}\" xmlns=\"http://www.w3.org/2000/svg\">",
        escape(source)
    ));
    svg.push_str(&format!("<title>y = {}</title>", escape(source)));

    // Axes: the vertical one (x = 0) is always in domain; the horizontal
    // one only when 0 lies within the sampled value range.
    svg.push_str(&format!(
        "<line class=\"axis\" x1=\"{:.1}\" y1=\"{TOP:.1}\" x2=\"{:.1}\" y2=\"{BOTTOM:.1}\" stroke=\"currentColor\" opacity=\"0.4\" />",
        sx(0.0),
        sx(0.0)
    ));
    if data.zero_axis {
        svg.push_str(&format!(
            "<line class=\"axis\" x1=\"{LEFT:.1}\" y1=\"{:.1}\" x2=\"{RIGHT:.1}\" y2=\"{:.1}\" stroke=\"currentColor\" opacity=\"0.4\" />",
            sy(0.0, data.y_min, y_span),
            sy(0.0, data.y_min, y_span)
        ));
    }

    // Tick labels: the x domain ends and the y value extremes.
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"currentColor\" font-size=\"11\">-10</text>",
        LEFT,
        HEIGHT - 8.0
    ));
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"currentColor\" font-size=\"11\" text-anchor=\"end\">10</text>",
        RIGHT,
        HEIGHT - 8.0
    ));
    svg.push_str(&format!(
        "<text x=\"4.0\" y=\"{:.1}\" fill=\"currentColor\" font-size=\"11\">{}</text>",
        TOP + 4.0,
        label(data.y_max)
    ));
    svg.push_str(&format!(
        "<text x=\"4.0\" y=\"{:.1}\" fill=\"currentColor\" font-size=\"11\">{}</text>",
        BOTTOM,
        label(data.y_min)
    ));

    // The curve, split into segments at non-finite points (gaps, not jumps).
    for segment in &data.segments {
        let points: Vec<String> = segment
            .iter()
            .map(|(x, y)| format!("{:.1},{:.1}", sx(*x), sy(*y, data.y_min, y_span)))
            .collect();
        svg.push_str(&format!(
            "<polyline points=\"{}\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linejoin=\"round\" stroke-linecap=\"round\" />",
            points.join(" ")
        ));
    }

    svg.push_str("</svg>");
    svg
}

/// Render `y = source` samples as Yew SVG VNodes — the production renderer.
/// Yew creates SVG elements in the SVG namespace, so the plot actually
/// paints in every engine (innerHTML-parsed SVG does not, in WebKit).
pub fn graph_html(samples: &[Sample], source: &str) -> Html {
    let Some(data) = plot_data(samples) else {
        return html! {};
    };
    let y_span = (data.y_max - data.y_min).max(1e-12);

    let mut segments = Vec::new();
    for segment in &data.segments {
        let points: Vec<String> = segment
            .iter()
            .map(|(x, y)| format!("{:.1},{:.1}", sx(*x), sy(*y, data.y_min, y_span)))
            .collect();
        segments.push(points.join(" "));
    }

    let y_axis = if data.zero_axis {
        let y = sy(0.0, data.y_min, y_span);
        html! {
            <line class="axis" x1={LEFT.to_string()} y1={y.to_string()} x2={RIGHT.to_string()} y2={y.to_string()} stroke="currentColor" opacity="0.4" />
        }
    } else {
        html! {}
    };

    html! {
        <svg viewBox={format!("0 0 {WIDTH} {HEIGHT}")} role="img" aria-label={format!("Graph of y = {source}")} xmlns="http://www.w3.org/2000/svg">
            <title>{ format!("y = {source}") }</title>
            <line class="axis" x1={sx(0.0).to_string()} y1={TOP.to_string()} x2={sx(0.0).to_string()} y2={BOTTOM.to_string()} stroke="currentColor" opacity="0.4" />
            { y_axis }
            <text x={LEFT.to_string()} y={(HEIGHT - 8.0).to_string()} fill="currentColor" font-size="11">{ "-10" }</text>
            <text x={RIGHT.to_string()} y={(HEIGHT - 8.0).to_string()} fill="currentColor" font-size="11" text-anchor="end">{ "10" }</text>
            <text x="4.0" y={(TOP + 4.0).to_string()} fill="currentColor" font-size="11">{ label(data.y_max) }</text>
            <text x="4.0" y={BOTTOM.to_string()} fill="currentColor" font-size="11">{ label(data.y_min) }</text>
            {
                for segments.into_iter().map(|points| {
                    html! {
                        <polyline points={points} fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" stroke-linecap="round" />
                    }
                })
            }
        </svg>
    }
}
