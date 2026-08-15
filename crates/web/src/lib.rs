//! epher-web — the Yew frontend compiled to `wasm32-unknown-unknown`, shared by
//! the PWA and the Tauri desktop shell (ADR-0001).
//!
//! A thin component over the shared [`Session`]: input line, result, history.
//! All logic lives in epher-core and epher-shell; this file is presentation
//! glue. Inside the desktop shell, persistence goes through the native store
//! via the Tauri IPC bridge (ADR-0010); in the browser, the session is the
//! whole state.

mod bridge;
pub mod graph;

use bridge::{Bridge, InitState};
use epher_core::{parse, sample, Sample, Session};
use epher_i18n::Localizer;
use epher_shell::{classify, message, prepare};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::events::{InputEvent, SubmitEvent};
use yew::prelude::*;

#[function_component(EpherApp)]
fn epher_app() -> Html {
    let session = use_state(Session::new);
    let input = use_state(String::new);
    let result = use_state(String::new);
    let localizer = use_state(|| Localizer::resolve(None, &[]));
    let graph = use_state(|| Option::<(Vec<Sample>, String)>::None);
    let bridge = Bridge::detect();

    // Inside the desktop shell: rebuild the session from the native store —
    // history plus saved functions and scripts replayed quietly, the exact
    // load_session recipe — and honor the stored language preference.
    {
        let session = session.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        use_effect_with(
            (),
            move |_| {
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
            },
        );
    }

    let on_input = {
        let input = input.clone();
        Callback::from(move |e: InputEvent| {
            let target = e.target_unchecked_into::<HtmlInputElement>();
            input.set(target.value());
        })
    };

    let on_submit = {
        let session = session.clone();
        let input = input.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let graph = graph.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let line = (*input).trim().to_string();

            // Graphing (ADR-0006: the core samples, the frontend renders).
            // Same command and domain as the TUI; history is untouched.
            if let Some(source) = line.strip_prefix("graph ") {
                let source = source.trim();
                match (|| {
                    let expr = parse(source)?;
                    sample(&expr, -10.0, 10.0, 120, (*session).env())
                })() {
                    Ok(samples) => {
                        graph.set(Some((samples, source.to_string())));
                        result.set(format!("graph: {source}"));
                    }
                    Err(e) => result.set(format!("error: {e}")),
                }
                input.set(String::new());
                return;
            }

            // Shell commands (epher-shell policy): persist through the
            // bridge in the desktop shell; explain the web app's limits
            // otherwise.
            if let Some(cmd) = classify(&line) {
                match bridge {
                    Bridge::Tauri => match prepare(&cmd, &session, &localizer) {
                        Ok(prepared) => {
                            match &prepared {
                                epher_shell::Prepared::SaveFunction { name, source } => {
                                    bridge.save_function(name, source);
                                }
                                epher_shell::Prepared::SaveScript { name, source } => {
                                    bridge.save_script(name, source);
                                }
                                epher_shell::Prepared::Language { code } => {
                                    bridge.save_language(code);
                                    localizer.set(Localizer::resolve(Some(code), &[]));
                                }
                            }
                            result.set(message(&prepared, &localizer));
                        }
                        Err(msg) => result.set(msg),
                    },
                    Bridge::None => result.set(localizer.lookup("web-session-only")),
                }
                input.set(String::new());
                return;
            }

            let mut s = (*session).clone();
            let out = s.submit(&line);
            let history = s.history().to_vec();
            session.set(s);
            result.set(out);
            input.set(String::new());
            // Desktop apps are killed, not exited: persist per line (ADR-0010).
            if bridge == Bridge::Tauri {
                bridge.save_history(&history);
            }
        })
    };

    let is_error = result.starts_with("error:") || result.starts_with("warning:");

    html! {
        <main class="epher">
            <h1>{ "epher" }</h1>
            <form onsubmit={on_submit}>
                <input
                    type="text"
                    placeholder={"expression or script"}
                    value={(*input).clone()}
                    oninput={on_input}
                    autofocus={true}
                    aria-label="expression"
                    aria-invalid={if is_error { "true" } else { "false" }}
                    aria-describedby={if is_error { "epher-result" } else { "" }}
                />
                <button type="submit" aria-label="Evaluate">{ "=" }</button>
            </form>
            <div id="epher-result" class="result" role="status" aria-live="polite">{ (*result).clone() }</div>
            {
                match (*graph).clone() {
                    Some((samples, source)) => {
                        let caption = format!("y = {source}");
                        // The visible text alternative: what is plotted.
                        html! {
                            <section class="graph">
                                <p class="graph-caption">{ caption }</p>
                                { graph::graph_html(&samples, &source) }
                            </section>
                        }
                    }
                    None => html! {},
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
