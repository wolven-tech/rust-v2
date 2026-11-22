//! Smart recommendation: a suggestion card with a confidence signal, alternatives
//! and one-click accept. Fold it away, pick an alternative instead, accept and undo.
//!
//! Port losses: the menu's flip check runs after paint, where the React original
//! measures in a layout effect, so a menu near the card's left edge can show once
//! in the default direction. The menu closes on an outside press or Escape through
//! a one-off listener per opening; a menu closed by picking an option keeps that
//! listener until the next outside press.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/smart-recommendation.css");

pub const SIGNAL_TEXT: [&str; 4] = ["No signal", "Weak signal", "Good signal", "Strong signal"];

#[derive(Clone, PartialEq)]
pub struct Recommendation {
    pub id: String,
    /// Short name, used in the accept button, e.g. "Incremental sync".
    pub label: String,
    /// One sentence why; shown in the card.
    pub reason: Element,
    /// 0 = no signal, 1 = weak, 2 = good, 3 = strong. Other values are clamped.
    pub signal: i32,
}

/// Clamps a signal to 0 through 3.
pub fn signal_level(signal: i32) -> usize {
    signal.clamp(0, 3) as usize
}

/// `"Incremental sync"` → `"incremental sync"`, but `"AWS backup"` and `"S3 sync"`
/// keep their capitals.
pub fn lower_first(text: &str) -> String {
    let mut chars = text.chars();
    let first = chars.next();
    let second = chars.next();
    match (first, second) {
        (Some(first), Some(second)) if first.is_uppercase() && second.is_lowercase() => {
            let rest = &text[first.len_utf8()..];
            format!("{}{rest}", first.to_lowercase())
        }
        _ => text.to_string(),
    }
}

/// The accept button text: `Accept incremental sync`.
pub fn accept_text(label: &str) -> String {
    format!("Accept {}", lower_first(label))
}

fn outside_press_script(alt_id: &str) -> String {
    format!(
        "await new Promise((resolve) => {{
           const done = () => {{
             window.removeEventListener('pointerdown', away);
             window.removeEventListener('keydown', esc);
             resolve(true);
           }};
           const away = (e) => {{
             const alt = document.getElementById({alt_id:?});
             if (!alt || !alt.contains(e.target)) done();
           }};
           const esc = (e) => {{ if (e.key === 'Escape') done(); }};
           window.addEventListener('pointerdown', away);
           window.addEventListener('keydown', esc);
         }});
         return true;"
    )
}

fn flip_script(alt_id: &str) -> String {
    format!(
        "const alt = document.getElementById({alt_id:?});
         const pop = alt?.querySelector('.ui-src__menu');
         const card = alt?.closest('.ui-src');
         if (!alt || !pop || !card || getComputedStyle(alt).position === 'static') return false;
         return alt.getBoundingClientRect().right - pop.offsetWidth < card.getBoundingClientRect().left - 24;"
    )
}

#[component]
pub fn SmartRecommendation(
    #[props(default = "Smart recommendation".to_string(), into)] title: String,
    /// First entry is the recommendation; the rest are offered as alternatives.
    options: Vec<Recommendation>,
    #[props(default)] on_accept: Option<EventHandler<Recommendation>>,
    #[props(default)] on_undo: Option<EventHandler<Recommendation>>,
    #[props(default = true)] default_open: bool,
    #[props(default)] class: String,
) -> Element {
    let mut chosen = use_signal(|| 0usize);
    let mut accepted = use_signal(|| None::<usize>);
    let mut open = use_signal(|| default_open);
    let mut menu = use_signal(|| false);
    let mut flip = use_signal(|| false);
    let alt_id = use_hook(|| format!("ui-sr-alt-{}", next_id()));

    let alt_for_flip = alt_id.clone();
    use_effect(use_reactive((&menu(),), move |(is_open,)| {
        if !is_open {
            flip.set(false);
            return;
        }
        let script = flip_script(&alt_for_flip);
        spawn(async move {
            let flipped = document::eval(&script)
                .await
                .ok()
                .and_then(|value| value.as_bool());
            flip.set(flipped.unwrap_or(false));
        });
    }));

    let alt_for_close = alt_id.clone();
    use_effect(use_reactive((&menu(),), move |(is_open,)| {
        if !is_open {
            return;
        }
        let script = outside_press_script(&alt_for_close);
        spawn(async move {
            let _ = document::eval(&script).await;
            menu.set(false);
        });
    }));

    let at = if chosen() < options.len() {
        chosen()
    } else {
        0
    };
    let Some(current) = options.get(at).cloned() else {
        return rsx! {};
    };
    let is_recommended = at == 0;
    let done = accepted() == Some(at);
    let signal = signal_level(current.signal);
    let signal_text = SIGNAL_TEXT[signal];
    let current_label = current.label.clone();
    let accept = accept_text(&current.label);
    let class = class_list("ui-src", &class);
    let (is_open, is_menu, is_flip) = (open(), menu(), flip());

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class,
            aria_label: "{title}",
            "data-accepted": done.then_some("true"),
            button {
                r#type: "button",
                class: "ui-src__head",
                aria_expanded: if is_open { "true" } else { "false" },
                onclick: move |_| open.set(!is_open),
                SparkIcon {}
                span { class: "ui-src__title", dir: "auto", title: "{title}", "{title}" }
                if !is_recommended {
                    span { class: "ui-src__pill", "Alternative" }
                }
                span { class: "ui-src__chevron",
                    Icon { d: "m4.5 6.25 3.5 3.5 3.5-3.5" }
                }
            }
            div {
                class: "ui-src__body",
                "data-open": is_open.then_some("true"),
                "inert": (!is_open).then_some("true"),
                p { key: "{at}", class: "ui-src__reason", dir: "auto", {current.reason.clone()} }
            }
            footer { class: "ui-src__foot",
                span {
                    class: "ui-src__signal",
                    "data-level": "{signal}",
                    title: "{signal_text}",
                    span { class: "ui-src__bars", "aria-hidden": "true",
                        for n in 1..=3usize {
                            span { key: "{n}", "data-on": (signal >= n).then_some("true") }
                        }
                    }
                    "{signal_text}"
                }
                div { class: "ui-src__actions",
                    if options.len() > 1 && !done {
                        div { id: "{alt_id}", class: "ui-src__alt",
                            button {
                                r#type: "button",
                                class: "ui-src__btn ui-src__btn--icon",
                                aria_haspopup: "menu",
                                aria_expanded: if is_menu { "true" } else { "false" },
                                onclick: move |_| menu.set(!is_menu),
                                Icon { d: "M5 12.5v-9M2.75 5.75 5 3.5l2.25 2.25M11 3.5v9M8.75 10.25 11 12.5l2.25-2.25" }
                                "Alternatives"
                            }
                            if is_menu {
                                div {
                                    class: "ui-src__menu",
                                    role: "menu",
                                    "data-flip": is_flip.then_some("true"),
                                    for (i, option) in options.iter().enumerate() {
                                        {
                                            let option_level = signal_level(option.signal);
                                            let option_text = SIGNAL_TEXT[option_level];
                                            let option_name = option.label.clone();
                                            let option_key = format!("{}-{i}", option.id);
                                            rsx! {
                                                button {
                                                    key: "{option_key}",
                                                    r#type: "button",
                                                    role: "menuitemradio",
                                                    aria_checked: if i == at { "true" } else { "false" },
                                                    class: "ui-src__option",
                                                    onclick: move |_| {
                                                        chosen.set(i);
                                                        menu.set(false);
                                                        open.set(true);
                                                    },
                                                    span { class: "ui-src__option-label",
                                                        span { class: "ui-src__option-name", dir: "auto", title: "{option_name}", "{option_name}" }
                                                        if i == 0 {
                                                            span { class: "ui-src__tag", "Recommended" }
                                                        }
                                                    }
                                                    span {
                                                        class: "ui-src__mini",
                                                        "data-level": "{option_level}",
                                                        aria_label: "{option_text}",
                                                        for n in 1..=3usize {
                                                            span { key: "{n}", "data-on": (option_level >= n).then_some("true") }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if done {
                        span { class: "ui-src__done", role: "status",
                            Icon { d: "m3.5 8.5 3 3 6-7" }
                            span { class: "ui-src__done-text", title: "{current_label} accepted",
                                bdi { "{current_label}" }
                                " accepted"
                            }
                        }
                        button {
                            r#type: "button",
                            class: "ui-src__btn ui-src__btn--icon",
                            onclick: {
                                let undone = current.clone();
                                move |_| {
                                    accepted.set(None);
                                    if let Some(handler) = on_undo {
                                        handler.call(undone.clone());
                                    }
                                }
                            },
                            Icon { d: "M3.5 6.5h6.25a2.75 2.75 0 0 1 0 5.5H8M5.75 4.25 3.5 6.5l2.25 2.25" }
                            "Undo"
                        }
                    } else {
                        button {
                            r#type: "button",
                            class: "ui-src__btn ui-src__btn--primary",
                            title: "{accept}",
                            onclick: {
                                let accepting = current.clone();
                                move |_| {
                                    accepted.set(Some(at));
                                    menu.set(false);
                                    if let Some(handler) = on_accept {
                                        handler.call(accepting.clone());
                                    }
                                }
                            },
                            span { class: "ui-src__btn-text", "{accept}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SparkIcon() -> Element {
    rsx! {
        svg {
            class: "ui-src__mark",
            view_box: "0 0 16 16",
            "aria-hidden": "true",
            path { d: "M6.5 2.2c.2 0 .36.14.4.33.53 2.6 1.4 3.48 4 4.02a.4.4 0 0 1 0 .79c-2.6.53-3.47 1.4-4 4a.4.4 0 0 1-.8 0c-.53-2.6-1.4-3.47-4-4a.4.4 0 0 1 0-.79c2.6-.54 3.47-1.42 4-4.02a.4.4 0 0 1 .4-.33Z" }
            path { d: "M12 9.6c.12 0 .22.08.25.2.28 1.3.72 1.75 2.03 2.03a.25.25 0 0 1 0 .49c-1.31.28-1.75.72-2.03 2.03a.25.25 0 0 1-.5 0c-.28-1.31-.72-1.75-2.03-2.03a.25.25 0 0 1 0-.49c1.31-.28 1.75-.73 2.03-2.03a.25.25 0 0 1 .25-.2Z" }
        }
    }
}

#[component]
fn Icon(d: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 16 16",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.1",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d }
        }
    }
}
