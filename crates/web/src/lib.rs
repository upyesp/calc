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

use crate::graph::{Graph, Graph3D};
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

/// A running parameter animation (ADR-0015): the constant `name` steps by
/// `step` between `lo` and `hi`, wrapping around; `value` is the next value
/// to apply on the coming tick.
#[derive(Debug, Clone, PartialEq)]
struct PlaySpec {
    name: String,
    lo: f64,
    hi: f64,
    step: f64,
    value: f64,
    /// The 3D viewBox frozen at play start: while playing, the plot keeps
    /// this box so the layout (and the pause button) stay put.
    freeze: Option<String>,
}

impl PlaySpec {
    fn ticked(&self) -> PlaySpec {
        let mut next = self.value + self.step;
        if next > self.hi {
            next = self.lo;
        }
        PlaySpec {
            value: next,
            ..self.clone()
        }
    }
}

/// The names of session constants any plotted expression references — each
/// becomes a live slider (ADR-0014). Surfaces count too (ADR-0015): their
/// constants animate the mesh the same way.
fn slider_names(
    curves: &[SampledCurve],
    surfaces: &[epher_core::graph::Surface],
    session: &Session,
) -> Vec<String> {
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
    for surface in surfaces {
        if let Ok((expr, _)) = epher_core::graph::parse_surface_source(&surface.source) {
            let mut found = std::collections::BTreeSet::new();
            free_names(&expr, &mut found);
            for n in found {
                if session.const_sources().contains_key(&n) {
                    names.insert(n);
                }
            }
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

/// Re-sample every surface against the current environment (a moved
/// constant changes the mesh).
fn resample_surfaces(surfaces: &mut [epher_core::graph::Surface], session: &Session) {
    for surface in surfaces.iter_mut() {
        if let Ok(fresh) = epher_core::graph::sample_surface(&surface.source, 30, session.env()) {
            *surface = fresh;
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
    let surface = use_state(Vec::<epher_core::graph::Surface>::new);
    let view = use_state(epher_core::graph::View3D::default);
    let play = use_state(|| Option::<PlaySpec>::None);
    // The live cell behind `play`: the animation loop reads and advances
    // it across ticks; Yew handles captured at spawn read stale snapshots.
    let play_cell = use_state(|| Rc::new(RefCell::new(Option::<PlaySpec>::None)));
    // The 3D viewBox from the latest render; play start freezes it.
    let rendered_box = use_state(|| Rc::new(RefCell::new(None::<String>)));
    let show_install_cli = use_state(|| false);
    let bridge = Bridge::detect();

    // Clear history (the button next to the list): empty the session's
    // history, persist the cleared state in the desktop shell, and leave
    // definitions, constants, and plotted curves untouched.
    let on_clear_history = {
        let session = session.clone();
        let result = result.clone();
        Callback::from(move |_| {
            let mut s = (*session).clone();
            s.clear_history();
            session.set(s);
            bridge.save_history(&[]);
            result.set(String::new());
        })
    };

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
        let live = live.clone();
        let surface = surface.clone();
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
            let mut surfaces = (*surface).clone();
            // Statements join with newlines or `;` — the same separator
            // (ADR-0001). Each piece dispatches in order, exactly as if
            // typed one by one.
            for raw_line in (*input).split('\n') {
                for piece in raw_line.split(';') {
                let line = piece.trim().to_string();
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

                // 3D surfaces (ADR-0015): z = f(x, y) over a square
                // domain, overlaid like curves. History is untouched.
                if let Some(source) = line.strip_prefix("graph3d ") {
                    let source = source.trim();
                    if source == "clear" {
                        surfaces.clear();
                        continue;
                    }
                    match epher_core::graph::sample_surface(source, 30, s.env()) {
                        Ok(fresh) => {
                            surfaces.push(fresh);
                            result.set(format!("graph3d: {source}"));
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
            }
            // Publish the loop's outcomes once: points of interest and the
            // slider set follow from the final curves and session.
            let found = analyze(&curves, s.env());
            let labels = poi_labels(&found, &localizer);
            {
                let mut l = (*live).borrow_mut();
                l.curves = curves.clone();
                l.trace = None;
            }
            graph.set(curves);
            surface.set(surfaces);
            pois.set(labels);
            trace.set(None);
            session.set(s.clone());
            input.set(String::new());
            // Desktop apps are killed, not exited: persist per line (ADR-0010).
            if bridge == Bridge::Tauri {
                bridge.save_history(s.history());
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
        let surface = surface.clone();
        Callback::from(move |(name, value): (String, f64)| {
            let mut s = (*session).clone();
            s.set_constant(
                name.clone(),
                Value::float(value),
                format!("const {name} = {value}"),
            );
            let mut curves = (*graph).clone();
            resample(&mut curves, &s);
            let mut surfaces = (*surface).clone();
            resample_surfaces(&mut surfaces, &s);
            let found = analyze(&curves, s.env());
            session.set(s);
            graph.set(curves);
            surface.set(surfaces);
            pois.set(poi_labels(&found, &localizer));
        })
    };

    // The same resample logic, shared with the animation loop through a
    // live cell (Yew handles captured by the loop would go stale). The
    // cell is refreshed after every render.
    let live_apply =
        use_state(|| Rc::new(RefCell::new(None::<Rc<dyn Fn(String, f64)>>)));
    {
        let live_apply = live_apply.clone();
        let on_slider = on_slider.clone();
        use_effect(move || {
            let apply: Rc<dyn Fn(String, f64)> = Rc::new(move |name: String, value: f64| {
                on_slider.emit((name, value));
            });
            *live_apply.borrow_mut() = Some(apply);
            || {}
        });
    }

    // 3D orbit: drag or arrow keys rotate the view (ADR-0015).
    let on_orbit = {
        let view = view.clone();
        Callback::from(move |(dyaw, dpitch): (f64, f64)| {
            let v = *view;
            view.set(epher_core::graph::View3D {
                yaw: v.yaw + dyaw,
                pitch: (v.pitch + dpitch).clamp(-1.4, 1.4),
                camera: v.camera,
            });
        })
    };

    // Parameter animation (ADR-0015): the play button on a slider starts a
    // loop that steps the constant within the slider's bounds and re-runs
    // the same resample path as a drag. The loop talks to the live cell so
    // it never reads stale state; `play` mirrors it for rendering.
    {
        let play_cell = play_cell.clone();
        let live_apply = live_apply.clone();
        // The loop must be spawned once, not per render: use_effect (no
        // deps) re-runs after every render, so a bare use_effect here
        // would add a new loop on every tick, each tick re-rendering and
        // spawning another — playback would accelerate to a crash.
        use_effect_with((), move |_| {
            spawn_local(async move {
                loop {
                    if (*play_cell).borrow().is_none() {
                        gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                        continue;
                    }
                    // One step per 120 ms: a fresh constant's slider spans
                    // ±10 (200 steps), so one full cycle takes 24 s — the
                    // vendor norm for playback speed.
                    gloo_timers::future::sleep(std::time::Duration::from_millis(120)).await;
                    let Some(spec) = (*play_cell).borrow().clone() else {
                        continue;
                    };
                    let next = spec.ticked();
                    *play_cell.borrow_mut() = Some(next.clone());
                    if let Some(apply) = (*live_apply).borrow().as_ref() {
                        apply(next.name.clone(), next.value);
                    }
                }
            });
            || {}
        });
    }
    let start_play = {
        let play = play.clone();
        let play_cell = play_cell.clone();
        let rendered_box = rendered_box.clone();
        let live_apply = live_apply.clone();
        Callback::from(move |(name, value): (String, f64)| {
            let reduce = web_sys::window()
                .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok())
                .flatten()
                .map(|m| m.matches())
                .unwrap_or(false);
            if reduce {
                // No looping playback under reduced motion: each press
                // steps the parameter once (WCAG 2.3.3).
                let lo = f64::min(-10.0, value - 2.0);
                let hi = f64::max(10.0, value + 2.0);
                let mut next = value + 0.1;
                if next > hi {
                    next = lo;
                }
                if let Some(apply) = (*live_apply).borrow().as_ref() {
                    apply(name.clone(), next);
                }
                return;
            }
            let lo = f64::min(-10.0, value - 2.0);
            let hi = f64::max(10.0, value + 2.0);
            let spec = PlaySpec {
                name,
                lo,
                hi,
                step: 0.1,
                value,
                freeze: (*rendered_box).borrow().clone(),
            };
            play.set(Some(spec.clone()));
            *play_cell.borrow_mut() = Some(spec);
        })
    };
    let stop_play = {
        let play = play.clone();
        let play_cell = play_cell.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            play.set(None);
            *play_cell.borrow_mut() = None;
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

    // Slider rows for a list of constant names — the 2D plot gets the
    // constants its curves reference, the 3D plot the constants its
    // surfaces reference (ADR-0014/0015). Dragging the animated slider
    // stops playback; the play button (re)starts it.
    let build_rows = |names: &[String]| -> Vec<Html> {
        names
            .iter()
            .filter_map(|name| {
                let v = const_value(&session, name)?;
                let lo = f64::min(-10.0, v - 2.0);
                let hi = f64::max(10.0, v + 2.0);
                let on_slider = on_slider.clone();
                let playing_this = (*play).as_ref().is_some_and(|p| p.name == *name);
                let stop_on_drag = {
                    let play = play.clone();
                    let play_cell = play_cell.clone();
                    let name = name.clone();
                    let on_slider = on_slider.clone();
                    Callback::from(move |e: InputEvent| {
                        let target = e.target_unchecked_into::<HtmlInputElement>();
                        let Ok(value) = target.value().parse::<f64>() else {
                            return;
                        };
                        if play.as_ref().is_some_and(|p| p.name == name) {
                            play.set(None);
                            *play_cell.borrow_mut() = None;
                        }
                        on_slider.emit((name.clone(), value));
                    })
                };
                let name_for_play = name.clone();
                let start_play = start_play.clone();
                let stop_play = stop_play.clone();
                let animate_label = if playing_this {
                    localizer.lookup("animate-stop")
                } else {
                    localizer.lookup("animate")
                };
                Some(html! {
                    <div class="slider">
                        <span class="slider-name">{ name.clone() }</span>
                        <input
                            type="range"
                            min={lo.to_string()}
                            max={hi.to_string()}
                            step="0.1"
                            value={v.to_string()}
                            oninput={stop_on_drag}
                        />
                        <span class="slider-value">{ format!("{v:.3}") }</span>
                        <button
                            type="button"
                            class="play-btn"
                            aria-pressed={playing_this.to_string()}
                            aria-label={format!("{animate_label} {name}")}
                            onclick={if playing_this {
                                stop_play
                            } else {
                                Callback::from(move |_: web_sys::MouseEvent| {
                                    start_play.emit((name_for_play.clone(), v))
                                })
                            }}
                        >
                            <span aria-hidden="true">{ if playing_this { "⏸" } else { "▶" } }</span>
                        </button>
                    </div>
                })
            })
            .collect()
    };
    let curve_sliders = slider_names(&graph, &[], &session);
    let surface_sliders = slider_names(&[], &surface, &session);
    let curve_rows = build_rows(&curve_sliders);
    let surface_rows = build_rows(&surface_sliders);

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
                                { for curve_rows }
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
            {
                if !(*surface).is_empty() {
                    let rendered = graph::surface_svg(&surface, &view);
                    let aria = format!(
                        "{}: {}",
                        "3D",
                        (*surface)
                            .iter()
                            .map(|s| format!("z = {}", s.source.trim()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    if let Some((view_box, content)) = rendered {
                        // Record for play-freeze; while playing, keep the
                        // frozen box so the layout stays put.
                        *rendered_box.borrow_mut() = Some(view_box.clone());
                        let shown_box = (*play)
                            .as_ref()
                            .and_then(|p| p.freeze.clone())
                            .unwrap_or(view_box);
                        html! {
                            <section class="graph graph3d">
                                <h2 class="graph3d-title">{ "3D" }</h2>
                                <Graph3D
                                    view_box={shown_box}
                                    content={content}
                                    aria_label={aria}
                                    on_orbit={on_orbit}
                                />
                                <p class="graph3d-hint">{ localizer.lookup("graph3d-hint") }</p>
                                <div class="sliders">
                                    { for surface_rows }
                                </div>
                            </section>
                        }
                    } else {
                        html! {}
                    }
                } else {
                    html! {}
                }
            }
            <div class="history-head">
                <h2>{ localizer.lookup("history") }</h2>
                <button type="button" class="clear-history" onclick={on_clear_history}>
                    { localizer.lookup("clear-history") }
                </button>
            </div>
            <ul class="history">
                { for session.history().iter().rev().map(|h| html! { <li>{ h.clone() } </li> }) }
            </ul>
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<EpherApp>::new().render();
}
