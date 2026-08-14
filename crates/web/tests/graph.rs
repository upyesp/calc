//! Pure tests for the web graph renderer (ADR-0006: the core samples, each
//! frontend renders). SVG as a string so the tests run natively — no browser
//! needed.

use calc_core::Sample;
use calc_web::graph::{graph_html, graph_svg, plot_data};

fn samples_of(ys: &[f64]) -> Vec<Sample> {
    ys.iter()
        .enumerate()
        .map(|(i, y)| Sample {
            x: -10.0 + i as f64 * (20.0 / (ys.len() as f64 - 1.0)),
            y: *y,
        })
        .collect()
}

#[test]
fn empty_samples_render_nothing() {
    assert_eq!(graph_svg(&[], "x ^ 2"), "");
}

#[test]
fn a_line_maps_the_domain_onto_the_plot_area() {
    // y = x sampled from -10 to 10: the first point sits at the left edge
    // and the bottom (y range is [-10, 10]), the last at the right edge top.
    let svg = graph_svg(&samples_of(&[-10.0, 0.0, 10.0]), "x");
    assert!(svg.contains("viewBox=\"0 0 640 400\""));
    assert!(svg.contains("points=\"45.0,365.0 "), "{svg}");
    assert!(svg.contains(" 625.0,15.0\""), "{svg}");
}

#[test]
fn non_finite_points_split_the_curve_into_segments() {
    let svg = graph_svg(
        &samples_of(&[-1.0, f64::NAN, 1.0]),
        "1 / x",
    );
    assert_eq!(svg.matches("<polyline").count(), 2);
}

#[test]
fn axes_appear_only_when_zero_is_in_range() {
    // y from -10..10: both the vertical axis (x=0) and the horizontal one
    let both = graph_svg(&samples_of(&[-10.0, 0.0, 10.0]), "x");
    assert_eq!(both.matches("class=\"axis\"").count(), 2);

    // y from 5..105: zero is outside — only the vertical axis
    let pos = graph_svg(&samples_of(&[5.0, 105.0]), "x ^ 2 + 5");
    assert_eq!(pos.matches("class=\"axis\"").count(), 1);
}

#[test]
fn the_source_is_escaped_everywhere_it_appears() {
    let svg = graph_svg(&samples_of(&[0.0, 1.0]), "x & <y>");
    assert!(svg.contains("x &amp; &lt;y&gt;"));
    assert!(!svg.contains("x & <y>"));
}

#[test]
fn the_svg_names_itself_for_assistive_tech() {
    let svg = graph_svg(&samples_of(&[0.0, 1.0]), "x ^ 2");
    assert!(svg.contains("role=\"img\""));
    assert!(svg.contains("<title>y = x ^ 2</title>"));
    assert!(svg.contains("aria-label=\"Graph of y = x ^ 2\""));
}

#[test]
fn plot_data_describes_segments_axes_and_range() {
    let data = plot_data(&samples_of(&[-10.0, 0.0, 10.0])).unwrap();
    assert_eq!(data.y_min, -10.0);
    assert_eq!(data.y_max, 10.0);
    assert!(data.zero_axis);
    assert_eq!(data.segments.len(), 1);
    assert_eq!(data.segments[0].len(), 3);

    // non-finite points split segments
    let data = plot_data(&samples_of(&[-1.0, f64::NAN, 1.0])).unwrap();
    assert_eq!(data.segments.len(), 2);

    // zero outside the range: no horizontal axis
    let data = plot_data(&samples_of(&[5.0, 105.0])).unwrap();
    assert!(!data.zero_axis);

    // nothing drawable
    assert!(plot_data(&[]).is_none());
    assert!(plot_data(&samples_of(&[f64::NAN])).is_none());
}

#[test]
fn graph_html_is_the_same_plot_as_the_svg_renderer() {
    let samples = samples_of(&[-10.0, 0.0, 10.0]);
    let html = format!("{:?}", graph_html(&samples, "x"));
    // the same structural elements as graph_svg
    assert!(html.contains("polyline"), "{html}");
    assert!(html.contains("aria-label"), "{html}");
    assert!(html.contains("title"), "{html}");
    assert!(html.contains("line"), "{html}");

    // empty input renders nothing
    let empty = format!("{:?}", graph_html(&[], "x"));
    assert!(!empty.contains("polyline"));
}

#[test]
fn tick_labels_show_the_domain_and_the_value_range() {
    let svg = graph_svg(&samples_of(&[-10.0, 0.0, 10.0]), "x");
    assert!(svg.contains(">-10</text>"));
    assert!(svg.contains(">10</text>"));
    // y range labels at the extremes
    assert!(svg.contains(">-10</text>") && svg.matches("</text>").count() >= 4);
}
