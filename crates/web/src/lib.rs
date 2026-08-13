//! calc-web — the Yew frontend compiled to `wasm32-unknown-unknown`, shared by
//! the PWA and the Tauri desktop shell (ADR-0001).
//!
//! A thin component over the shared [`Session`]: input line, result, history.
//! All logic lives in calc-core; this file is presentation glue.

use calc_core::Session;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::events::{InputEvent, SubmitEvent};
use yew::prelude::*;

#[function_component(CalcApp)]
fn calc_app() -> Html {
    let session = use_state(Session::new);
    let input = use_state(String::new);
    let result = use_state(String::new);

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
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let mut s = (*session).clone();
            let out = s.submit(&input);
            session.set(s);
            result.set(out);
            input.set(String::new());
        })
    };

    let is_error = result.starts_with("error:");

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
                { for session.history().iter().map(|h| html! { <li>{ h.clone() }</li> }) }
            </ul>
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<CalcApp>::new().render();
}
