//! calc-web — the Yew frontend compiled to `wasm32-unknown-unknown`, shared by
//! the PWA and the Tauri desktop shell (ADR-0001).
//!
//! A thin component over the shared [`Session`]: input line, result, history.
//! All logic lives in calc-core and calc-shell; this file is presentation
//! glue. Inside the desktop shell, persistence goes through the native store
//! via the Tauri IPC bridge (ADR-0010); in the browser, the session is the
//! whole state.

mod bridge;

use bridge::{Bridge, InitState};
use calc_core::Session;
use calc_i18n::Localizer;
use calc_shell::{classify, message, prepare};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::events::{InputEvent, SubmitEvent};
use yew::prelude::*;

#[function_component(CalcApp)]
fn calc_app() -> Html {
    let session = use_state(Session::new);
    let input = use_state(String::new);
    let result = use_state(String::new);
    let localizer = use_state(|| Localizer::resolve(None, &[]));
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
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let line = (*input).trim().to_string();

            // Shell commands (calc-shell policy): persist through the
            // bridge in the desktop shell; explain the web app's limits
            // otherwise.
            if let Some(cmd) = classify(&line) {
                match bridge {
                    Bridge::Tauri => match prepare(&cmd, &session, &localizer) {
                        Ok(prepared) => {
                            match &prepared {
                                calc_shell::Prepared::SaveFunction { name, source } => {
                                    bridge.save_function(name, source);
                                }
                                calc_shell::Prepared::SaveScript { name, source } => {
                                    bridge.save_script(name, source);
                                }
                                calc_shell::Prepared::Language { code } => {
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
        <main class="calc">
            <h1>{ "Calc" }</h1>
            <form onsubmit={on_submit}>
                <input
                    type="text"
                    placeholder={"expression or script"}
                    value={(*input).clone()}
                    oninput={on_input}
                    autofocus={true}
                    aria-label="expression"
                    aria-invalid={if is_error { "true" } else { "false" }}
                    aria-describedby={if is_error { "calc-result" } else { "" }}
                />
                <button type="submit" aria-label="Evaluate">{ "=" }</button>
            </form>
            <div id="calc-result" class="result" role="status" aria-live="polite">{ (*result).clone() }</div>
            <ul class="history">
                { for session.history().iter().map(|h| html! { <li>{ h.clone() } </li> }) }
            </ul>
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<CalcApp>::new().render();
}
