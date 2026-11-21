//! Tips panel: a list of short tips, each with one action. Taking an action marks
//! the tip done; the list scrolls inside the white card.
//!
//! Port losses: none.

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::internal::class_list;
use crate::reaction_bar::exact_count;

const CSS: Asset = asset!("/assets/tips-panel.css");

#[derive(Clone, PartialEq)]
pub struct Tip {
    pub id: String,
    pub title: String,
    /// Short explanation; may contain a [`Kbd`].
    pub description: Element,
    /// Button text.
    pub action: String,
    /// Text after the action was taken. Defaults to `Done`.
    pub done_label: Option<String>,
    /// Shows the action as the dark button; use once for the most important tip.
    pub primary: bool,
    pub on_action: Option<EventHandler<()>>,
}

/// Keys for a list of tip ids. The first `a` keeps `a`; later ones become `a#1`,
/// `a#2`, so marking one tip done never marks another with the same id.
pub fn tip_keys(ids: &[&str]) -> Vec<String> {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    ids.iter()
        .map(|&id| {
            let n = seen.entry(id).or_insert(0);
            let key = if *n == 0 {
                id.to_string()
            } else {
                format!("{id}#{n}")
            };
            *n += 1;
            key
        })
        .collect()
}

/// How many of `keys` are in `done`. Keys for tips that were removed from the
/// list do not count.
pub fn done_count(keys: &[String], done: &[String]) -> usize {
    keys.iter().filter(|key| done.contains(key)).count()
}

#[component]
pub fn TipsPanel(
    #[props(default = "Tips".to_string(), into)] title: String,
    tips: Vec<Tip>,
    /// Shows the Close button in the footer.
    #[props(default)]
    on_close: Option<EventHandler<()>>,
    #[props(default = "Close".to_string(), into)] close_label: String,
    #[props(default)] class: String,
) -> Element {
    let mut done = use_signal(Vec::<String>::new);

    let ids: Vec<&str> = tips.iter().map(|tip| tip.id.as_str()).collect();
    let keys = tip_keys(&ids);
    let done_now = done_count(&keys, &done());
    let summary = format!(
        "{}/{} done",
        exact_count(done_now as u64),
        exact_count(tips.len() as u64)
    );

    let class = class_list("ui-tp", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        section { class, aria_label: "{title}",
            header { class: "ui-tp__head",
                h3 { class: "ui-tp__title", dir: "auto", "{title}" }
                span { class: "ui-tp__count", aria_live: "polite", "{summary}" }
            }
            div { class: "ui-tp__card",
                ul { class: "ui-tp__grid",
                    for (i, (tip, key)) in tips.into_iter().zip(keys).enumerate() {
                        {
                            let is_done = done().contains(&key);
                            let delay = 80 + i.min(12) * 50;
                            let button_class = if tip.primary { "ui-tp__btn ui-tp__btn--primary" } else { "ui-tp__btn" };
                            let done_label = tip.done_label.clone().unwrap_or_else(|| "Done".to_string());
                            let on_action = tip.on_action;
                            let mark = key.clone();
                            let tip_title = tip.title.clone();
                            let tip_action = tip.action.clone();
                            rsx! {
                                li {
                                    key: "{key}",
                                    class: "ui-tp__tip",
                                    "data-done": is_done.then_some("true"),
                                    style: "animation-delay: {delay}ms",
                                    p { class: "ui-tp__tip-title", dir: "auto", "{tip_title}" }
                                    p { class: "ui-tp__tip-text", dir: "auto", {tip.description} }
                                    if is_done {
                                        span { class: "ui-tp__done", role: "status",
                                            CheckIcon {}
                                            "{done_label}"
                                        }
                                    } else {
                                        button {
                                            r#type: "button",
                                            class: button_class,
                                            onclick: move |_| {
                                                if !done.read().contains(&mark) {
                                                    done.write().push(mark.clone());
                                                }
                                                if let Some(handler) = &on_action {
                                                    handler.call(());
                                                }
                                            },
                                            "{tip_action}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(on_close) = on_close {
                footer { class: "ui-tp__foot",
                    button {
                        r#type: "button",
                        class: "ui-tp__btn",
                        onclick: move |_| on_close.call(()),
                        "{close_label}"
                    }
                }
            }
        }
    }
}

/// Keyboard key for tip descriptions, e.g. `⌘K`.
#[component]
pub fn Kbd(children: Element) -> Element {
    rsx! { kbd { class: "ui-tp__kbd", {children} } }
}

#[component]
fn CheckIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 16 16",
            width: "16",
            height: "16",
            "aria-hidden": "true",
            circle { cx: "8", cy: "8", r: "7", fill: "currentColor" }
            path {
                d: "m5 8.2 2 2 4-4.2",
                fill: "none",
                stroke: "#fff",
                stroke_width: "1.6",
                stroke_linecap: "round",
                stroke_linejoin: "round",
            }
        }
    }
}
