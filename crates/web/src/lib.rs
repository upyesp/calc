//! epher-web — the Yew frontend compiled to `wasm32-unknown-unknown`, shared by
//! the PWA and the Tauri desktop shell (ADR-0001).
//!
//! A thin component over the shared [`Session`]: input line, result, history,
//! and the graph panel (ADR-0006/0014 — the core samples and analyzes, this
//! file is presentation glue: curves, trace, points of interest, sliders).
//! Inside the desktop shell, persistence goes through the native store via
//! the Tauri IPC bridge (ADR-0010); in the browser, the session is the whole
//! state.

mod bridge;
pub mod graph;

use crate::graph::Graph;
use bridge::{Bridge, InitState};
use epher_core::graph::{
    analyze, free_names, parse_graph_source, sample_spec, CurveKind, CurveSpec, InterestPoint,
    SampledCurve,
};
use epher_core::{Session, Value};
use epher_i18n::Localizer;
use epher_shell::{classify, message, prepare};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::events::{InputEvent, SubmitEvent};
use yew::prelude::*;

/// Live graph-interaction state. The SVG's event listeners are attached
/// once, at mount, so the callbacks they hold must read current values —
/// but a cloned `UseStateHandle` reads the snapshot it was created with
/// (Yew replaces the handle's inner `Rc` on every `set`). This cell is the
/// live copy; the Yew states mirror it for rendering.
#[derive(Default)]
struct GraphLive {
    curves: Vec<SampledCurve>,
    trace: Option<graph::TracePoint>,
}

/// The names of session constants any plotted expression references — each
/// becomes a live slider (ADR-0014).
fn slider_names(curves: &[SampledCurve], session: &Session) -> Vec<String> {
    let mut names = std::collections::BTreeSet::new();
    for c in curves {
        let mut visit = |expr: &epher_core::Expression| {
            let mut found = std::collections::BTreeSet::new();
            free_names(expr, &mut found);
            for n in found {
                if session.const_sources().contains_key(&n) {
                    names.insert(n);
                }
            }
        };
        match &c.kind {
            CurveKind::Cartesian(e) => visit(e),
            CurveKind::Parametric { x, y } => {
                visit(x);
                visit(y);
            }
            CurveKind::Polar(e) => visit(e),
        }
    }
    names.into_iter().collect()
}

/// The spec that would reproduce a sampled curve (slider re-sampling).
fn curve_spec(c: &SampledCurve) -> CurveSpec {
    CurveSpec {
        kind: c.kind.clone(),
        domain: c.domain,
        fill: c.fill,
    }
}

/// Re-sample every curve against the (possibly changed) session environment.
fn resample(curves: &mut [SampledCurve], session: &Session) {
    for c in curves.iter_mut() {
        if let Ok(samples) = sample_spec(&curve_spec(c), 120, session.env()) {
            c.samples = samples;
        }
    }
}

/// Localized renderer labels for the analyzed points of interest.
fn poi_labels(points: &[InterestPoint], localizer: &Localizer) -> Vec<graph::Poi> {
    points
        .iter()
        .map(|p| {
            let key = match p.kind {
                epher_core::graph::InterestKind::Root => "poi-root",
                epher_core::graph::InterestKind::Intersection => "poi-intersection",
                epher_core::graph::InterestKind::Maximum => "poi-maximum",
                epher_core::graph::InterestKind::Minimum => "poi-minimum",
            };
            graph::Poi {
                kind: p.kind,
                label: localizer.lookup(key),
                x: p.x,
                y: p.y,
            }
        })
        .collect()
}

/// The value of a session constant as an f64, when it is one.
fn const_value(session: &Session, name: &str) -> Option<f64> {
    match session.env().constant(name)? {
        Value::Float(v) => Some(*v),
        _ => None,
    }
}

/// The curve whose sampled points are the closest to a trace position.
fn curve_at(curves: &[SampledCurve], index: usize) -> Option<&SampledCurve> {
    curves.get(index)
}

#[function_component(EpherApp)]
fn epher_app() -> Html {
    let session = use_state(Session::new);
    let input = use_state(String::new);
    let form_ref = use_node_ref();
    let result = use_state(String::new);
    let localizer = use_state(|| Localizer::resolve(None, &[]));
    let graph = use_state(Vec::<SampledCurve>::new);
    let pois = use_state(Vec::<graph::Poi>::new);
    let trace = use_state(|| Option::<graph::TracePoint>::None);
    let live = use_state(|| Rc::new(RefCell::new(GraphLive::default())));
    let sliders = use_state(Vec::<String>::new);
    let show_install_cli = use_state(|| false);
    let bridge = Bridge::detect();

    // Inside the desktop shell: rebuild the session from the native store —
    // history plus saved functions and scripts replayed quietly, the exact
    // load_session recipe — and honor the stored language preference.
    {
        let session = session.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        use_effect_with((), move |_| {
            if bridge == Bridge::Tauri {
                spawn_local(async move {
                    match bridge.init().await {
                        Ok(InitState {
                            history,
                            replay,
                            language,
                        }) => {
                            let mut s = Session::with_history(history);
                            for line in &replay {
                                s.submit_quiet(line);
                            }
                            session.set(s);
                            if let Some(code) = language {
                                localizer.set(Localizer::resolve(Some(&code), &[]));
                            }
                        }
                        Err(e) => {
                            result.set(format!(
                                "warning: could not load saved data ({e}); starting fresh"
                            ));
                        }
                    }
                });
            }
            || {}
        });
    }

    // macOS desktop builds offer to install the `epher` terminal command
    // (ADR-0011): a one-click symlink into /usr/local/bin.
    {
        let show_install_cli = show_install_cli.clone();
        use_effect_with((), move |_| {
            if bridge == Bridge::Tauri {
                spawn_local(async move {
                    if let Ok(true) = bridge.cli_install_supported().await {
                        show_install_cli.set(true);
                    }
                });
            }
            || {}
        });
    }

    let on_install_cli = {
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |_| {
            let result = result.clone();
            let localizer = localizer.clone();
            spawn_local(async move {
                let outcome = bridge.install_cli().await;
                let message = match outcome {
                    Ok(key) => localizer.lookup(&key),
                    Err(detail) => {
                        format!("{} {detail}", localizer.lookup("install-cli-failed"))
                    }
                };
                result.set(message);
            });
        })
    };

    let on_input = {
        let input = input.clone();
        Callback::from(move |e: InputEvent| {
            let target = e.target_unchecked_into::<HtmlTextAreaElement>();
            input.set(target.value());
        })
    };

    // Enter submits (the textarea's own Enter would insert a newline);
    // Shift+Enter inserts a newline so multi-line scripts can be composed
    // by hand. Submitting goes through the form so the `=` button and the
    // keyboard share one path.
    let on_keydown = {
        let form_ref = form_ref.clone();
        Callback::from(move |e: web_sys::KeyboardEvent| {
            if e.key() == "Enter" && !e.shift_key() && !e.is_composing() {
                e.prevent_default();
                if let Some(form) = form_ref.cast::<web_sys::HtmlFormElement>() {
                    let _ = form.request_submit();
                }
            }
        })
    };

    let on_submit = {
        let session = session.clone();
        let input = input.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let graph = graph.clone();
        let pois = pois.clone();
        let trace = trace.clone();
        let sliders = sliders.clone();
        let live = live.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            // A submitted entry may be several lines (pasted from the
            // guide, or composed with Shift+Enter). Each line runs in
            // order against one session snapshot — script semantics, like
            // the REPL and piped mode. Yew state handles do not expose
            // writes made earlier in the same callback, so the loop works
            // on locals and the states are published once, after the loop.
            let mut s = (*session).clone();
            let mut curves = (*graph).clone();
            for line in (*input).split('\n') {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                // Graphing (ADR-0006/0014: the core samples, the frontend renders).
                // Each `graph` line overlays one more curve; history is untouched.
                if let Some(source) = line.strip_prefix("graph ") {
                    let source = source.trim();
                    if source == "clear" {
                        curves.clear();
                        continue;
                    }
                    match parse_graph_source(source)
                        .and_then(|spec| sample_spec(&spec, 120, s.env()).map(|samples| (spec, samples)))
                    {
                        Ok((spec, samples)) => {
                            curves.push(SampledCurve {
                                source: source.to_string(),
                                kind: spec.kind,
                                domain: spec.domain,
                                samples,
                                fill: spec.fill,
                            });
                            result.set(format!("graph: {source}"));
                        }
                        Err(e) => result.set(format!("error: {e}")),
                    }
                    continue;
                }

                // Shell commands (epher-shell policy): persist through the
                // bridge in the desktop shell; explain the web app's limits
                // otherwise.
                if let Some(cmd) = classify(&line) {
                    match bridge {
                        Bridge::Tauri => match prepare(&cmd, &s, &localizer) {
                            Ok(prepared) => {
                                match &prepared {
                                    epher_shell::Prepared::SaveFunction { name, source } => {
                                        bridge.save_function(name, source);
                                    }
                                    epher_shell::Prepared::SaveConstant { name, source } => {
                                        bridge.save_constant(name, source);
                                    }
                                    epher_shell::Prepared::SaveScript { name, source } => {
                                        bridge.save_script(name, source);
                                    }
                                    epher_shell::Prepared::Language { code } => {
                                        bridge.save_language(code);
                                        localizer.set(Localizer::resolve(Some(code), &[]));
                                    }
                                    epher_shell::Prepared::Table { .. } => {}
                                }
                                result.set(message(&prepared, &localizer));
                            }
                            Err(msg) => result.set(msg),
                        },
                        Bridge::None => {
                            // Tables are pure computation — they work in the
                            // browser session just like an evaluation.
                            match &cmd {
                                epher_shell::Command::Table { .. } => {
                                    match prepare(&cmd, &s, &localizer) {
                                        Ok(prepared) => result.set(message(&prepared, &localizer)),
                                        Err(msg) => result.set(msg),
                                    }
                                }
                                _ => result.set(localizer.lookup("web-session-only")),
                            }
                        }
                    }
                    continue;
                }

                let out = s.submit(&line);
                result.set(out);
            }
            // Publish the loop's outcomes once: points of interest and the
            // slider set follow from the final curves and session.
            let found = analyze(&curves, s.env());
            let labels = poi_labels(&found, &localizer);
            sliders.set(slider_names(&curves, &s));
            {
                let mut l = (*live).borrow_mut();
                l.curves = curves.clone();
                l.trace = None;
            }
            graph.set(curves);
            pois.set(labels);
            trace.set(None);
            session.set(s.clone());
            input.set(String::new());
            // Desktop apps are killed, not exited: persist per line (ADR-0010).
            if bridge == Bridge::Tauri {
                bridge.save_history(&s.history().to_vec());
            }
        })
    };

    // Sliders: adjusting a constant re-samples every curve against the new
    // environment and re-runs the analysis (ADR-0014).
    let on_slider = {
        let session = session.clone();
        let graph = graph.clone();
        let pois = pois.clone();
        let localizer = localizer.clone();
        Callback::from(move |(name, value): (String, f64)| {
            let mut s = (*session).clone();
            s.set_constant(
                name.clone(),
                Value::float(value),
                format!("const {name} = {value}"),
            );
            let mut curves = (*graph).clone();
            resample(&mut curves, &s);
            let found = analyze(&curves, s.env());
            session.set(s);
            graph.set(curves);
            pois.set(poi_labels(&found, &localizer));
        })
    };

    // Trace: pointer moves/taps find the nearest sampled point; arrow keys
    // step along the traced curve (up/down switch curves). These callbacks
    // are bound to the SVG once at mount, so they read the live cell.
    let on_trace = {
        let live = live.clone();
        let trace = trace.clone();
        Callback::from(move |(px, py): (f64, f64)| {
            let found = {
                let l = (*live).borrow();
                graph::geometry(&l.curves)
                    .and_then(|geom| graph::trace_nearest(&l.curves, &geom, px, py))
            };
            (*live).borrow_mut().trace = found;
            trace.set(found);
        })
    };
    let on_trace_leave = {
        let live = live.clone();
        let trace = trace.clone();
        Callback::from(move |()| {
            (*live).borrow_mut().trace = None;
            trace.set(None);
        })
    };
    let on_trace_key = {
        let live = live.clone();
        let trace = trace.clone();
        Callback::from(move |e: web_sys::KeyboardEvent| {
            let Some(current) = (*live).borrow().trace else {
                return;
            };
            let data = (*live).borrow().curves.clone();
            let Some(curve) = curve_at(&data, current.curve) else {
                return;
            };
            let last = curve.samples.len().saturating_sub(1);
            match e.key().as_str() {
                "ArrowRight" => {
                    if current.index < last {
                        let s = &curve.samples[current.index + 1];
                        let next = Some(graph::TracePoint {
                            index: current.index + 1,
                            x: s.x,
                            y: s.y,
                            ..current
                        });
                        (*live).borrow_mut().trace = next;
                        trace.set(next);
                    }
                    e.prevent_default();
                }
                "ArrowLeft" => {
                    if current.index > 0 {
                        let s = &curve.samples[current.index - 1];
                        let next = Some(graph::TracePoint {
                            index: current.index - 1,
                            x: s.x,
                            y: s.y,
                            ..current
                        });
                        (*live).borrow_mut().trace = next;
                        trace.set(next);
                    }
                    e.prevent_default();
                }
                "ArrowDown" if !data.is_empty() => {
                    let ci = (current.curve + 1) % data.len();
                    let c = &data[ci];
                    if let Some(s) = c.samples.get(current.index.min(last)) {
                        let next = Some(graph::TracePoint {
                            curve: ci,
                            index: current.index.min(last),
                            x: s.x,
                            y: s.y,
                        });
                        (*live).borrow_mut().trace = next;
                        trace.set(next);
                    }
                    e.prevent_default();
                }
                "ArrowUp" if !data.is_empty() => {
                    let ci = (current.curve + data.len() - 1) % data.len();
                    let c = &data[ci];
                    if let Some(s) = c.samples.get(current.index.min(last)) {
                        let next = Some(graph::TracePoint {
                            curve: ci,
                            index: current.index.min(last),
                            x: s.x,
                            y: s.y,
                        });
                        (*live).borrow_mut().trace = next;
                        trace.set(next);
                    }
                    e.prevent_default();
                }
                _ => {}
            }
        })
    };

    // Copy the plot as standalone SVG (the same string renderer the tests
    // exercise), with a localized outcome message.
    let on_copy_svg = {
        let curves = graph.clone();
        let pois = pois.clone();
        let trace = trace.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |_| {
            let svg = graph::graph_svg(&curves, &pois, *trace);
            if svg.is_empty() {
                return;
            }
            let result = result.clone();
            let localizer = localizer.clone();
            spawn_local(async move {
                if let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) {
                    match clipboard.write_text(&svg).await {
                        Ok(_) => result.set(localizer.lookup("graph-copied")),
                        Err(_) => result.set(localizer.lookup("graph-copy-failed")),
                    }
                } else {
                    result.set(localizer.lookup("graph-copy-failed"));
                }
            });
        })
    };

    let is_error = result.starts_with("error:") || result.starts_with("warning:");

    // The trace announcement: coordinates in the current UI language-free
    // numeric form, announced politely (the plot itself is an image).
    let trace_text = (*trace).map(|t| format!("x = {:.3}, y = {:.3}", t.x, t.y));

    let slider_rows: Vec<Html> = (*sliders)
        .iter()
        .filter_map(|name| {
            let v = const_value(&session, name)?;
            let lo = f64::min(-10.0, v - 2.0);
            let hi = f64::max(10.0, v + 2.0);
            let on_slider = on_slider.clone();
            let name_for_event = name.clone();
            Some(html! {
                <label class="slider">
                    <span class="slider-name">{ name.clone() }</span>
                    <input
                        type="range"
                        min={lo.to_string()}
                        max={hi.to_string()}
                        step="0.1"
                        value={v.to_string()}
                        oninput={Callback::from(move |e: InputEvent| {
                            let target = e.target_unchecked_into::<HtmlInputElement>();
                            if let Ok(value) = target.value().parse::<f64>() {
                                on_slider.emit((name_for_event.clone(), value));
                            }
                        })}
                    />
                    <span class="slider-value">{ format!("{v:.3}") }</span>
                </label>
            })
        })
        .collect();

    let legend_items: Vec<Html> = (*graph)
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let caption = graph::curve_caption(c);
            html! {
                <li>
                    <span class={format!("swatch curve-{i}")} aria-hidden="true"></span>
                    { caption }
                </li>
            }
        })
        .collect();

    let poi_items: Vec<Html> = (*pois)
        .iter()
        .map(|p| {
            let text = format!("{} ({}, {})", p.label, graph::label(p.x), graph::label(p.y));
            html! { <li>{ text }</li> }
        })
        .collect();

    html! {
        <main class="epher">
            <h1>{ "epher" }</h1>
            <form ref={form_ref.clone()} onsubmit={on_submit}>
                <textarea
                    rows="1"
                    placeholder={"expression or script"}
                    value={(*input).clone()}
                    oninput={on_input}
                    onkeydown={on_keydown}
                    autofocus={true}
                    aria-label="expression"
                    aria-invalid={if is_error { "true" } else { "false" }}
                    aria-describedby={if is_error { "epher-result" } else { "" }}
                />
                <button type="submit" aria-label="Evaluate">{ "=" }</button>
            </form>
            <div id="epher-result" class="result" role="status" aria-live="polite">{ (*result).clone() }</div>
            {
                if *show_install_cli {
                    html! {
                        <button
                            type="button"
                            class="install-cli"
                            onclick={on_install_cli}
                        >
                            { localizer.lookup("install-cli") }
                        </button>
                    }
                } else {
                    html! {}
                }
            }
            {
                if !(*graph).is_empty() {
                    html! {
                        <section class="graph">
                            <ul class="legend">
                                { for legend_items }
                            </ul>
                            <Graph
                                curves={(*graph).clone()}
                                pois={(*pois).clone()}
                                trace={*trace}
                                on_trace={on_trace}
                                on_key={on_trace_key}
                                on_leave={on_trace_leave}
                            />
                            <p class="trace" role="status" aria-live="polite">
                                { trace_text }
                            </p>
                            {
                                if !(*pois).is_empty() {
                                    html! {
                                        <>
                                            <p class="poi-heading">{ localizer.lookup("graph-points") }</p>
                                            <ul class="poi-list">
                                                { for poi_items }
                                            </ul>
                                        </>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            <div class="sliders">
                                { for slider_rows }
                            </div>
                            <button type="button" class="copy-svg" onclick={on_copy_svg}>
                                { localizer.lookup("graph-copy") }
                            </button>
                        </section>
                    }
                } else {
                    html! {}
                }
            }
            <ul class="history">
                { for session.history().iter().map(|h| html! { <li>{ h.clone() } </li> }) }
            </ul>
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<EpherApp>::new().render();
}
