//! Decision inbox: open decisions with two quick answers each. Answering a decision
//! confirms it briefly, then folds the row away and opens the next folded one.
//!
//! Port losses: none.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::internal::{class_list, sleep};
use crate::reaction_bar::exact_count;

const CSS: Asset = asset!("/assets/decision-inbox.css");

#[derive(Clone, PartialEq)]
pub struct Decision {
    pub id: String,
    pub title: String,
    /// Short context under the title.
    pub detail: Option<String>,
    /// Main answer, shown as the dark button.
    pub primary: String,
    /// Other answer, shown as the light button.
    pub secondary: String,
    /// Starts folded to one muted line; click to open.
    pub collapsed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionAnswer {
    Primary,
    Secondary,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DecisionState {
    pub open: bool,
    pub answered: Option<String>,
    pub gone: bool,
}

pub fn initial_state(decisions: &[Decision]) -> HashMap<String, DecisionState> {
    decisions
        .iter()
        .map(|d| {
            (
                d.id.clone(),
                DecisionState {
                    open: !d.collapsed,
                    ..Default::default()
                },
            )
        })
        .collect()
}

/// Decisions not answered yet.
pub fn pending_count(decisions: &[Decision], states: &HashMap<String, DecisionState>) -> usize {
    decisions
        .iter()
        .filter(|d| {
            states
                .get(&d.id)
                .and_then(|s| s.answered.as_ref())
                .is_none()
        })
        .count()
}

/// The first decision other than `except` that is folded and unanswered. A
/// decision with no state counts as folded.
pub fn next_folded<'a>(
    decisions: &'a [Decision],
    states: &HashMap<String, DecisionState>,
    except: &str,
) -> Option<&'a str> {
    decisions
        .iter()
        .find(|d| {
            d.id != except
                && !states.get(&d.id).is_some_and(|s| s.open)
                && states
                    .get(&d.id)
                    .and_then(|s| s.answered.as_ref())
                    .is_none()
        })
        .map(|d| d.id.as_str())
}

fn decide(
    mut state: Signal<HashMap<String, DecisionState>>,
    all: Rc<Vec<Decision>>,
    on_decide: Option<EventHandler<(Decision, DecisionAnswer)>>,
    decision: Decision,
    answer: DecisionAnswer,
) {
    let label = match answer {
        DecisionAnswer::Primary => decision.primary.clone(),
        DecisionAnswer::Secondary => decision.secondary.clone(),
    };
    state
        .write()
        .entry(decision.id.clone())
        .or_default()
        .answered = Some(label);
    if let Some(handler) = on_decide {
        handler.call((decision.clone(), answer));
    }
    spawn(async move {
        sleep(1100).await;
        let mut next = state.peek().clone();
        next.entry(decision.id.clone()).or_default().gone = true;
        if let Some(folded) = next_folded(&all, &next, &decision.id).map(str::to_string) {
            next.entry(folded).or_default().open = true;
        }
        state.set(next);
    });
}

#[component]
pub fn DecisionInbox(
    decisions: Vec<Decision>,
    /// Called with the decision and the chosen answer.
    #[props(default)]
    on_decide: Option<EventHandler<(Decision, DecisionAnswer)>>,
    #[props(default)] class: String,
) -> Element {
    let mut state = use_signal(|| initial_state(&decisions));
    let states = state();
    let open = pending_count(&decisions, &states);
    let count = exact_count(open as u64);
    let noun = if open == 1 { "needs" } else { "need" };
    let headline = format!("{count} {noun} your input");
    let class = class_list("ui-di", &class);
    let all = Rc::new(decisions.clone());

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class,
            "data-clear": (open == 0).then_some("true"),
            header { class: "ui-di__head", aria_live: "polite",
                if open > 0 {
                    AlertIcon {}
                    span { "{headline}" }
                } else {
                    DoneIcon {}
                    span { "All caught up" }
                }
            }
            div { class: "ui-di__card",
                for d in decisions.iter().cloned() {
                    {
                        let s = states.get(&d.id).cloned().unwrap_or(DecisionState { open: true, ..Default::default() });
                        let row_key = d.id.clone();
                        let title = d.title.clone();
                        let detail = d.detail.clone();
                        let primary_label = d.primary.clone();
                        let secondary_label = d.secondary.clone();
                        let fold_id = d.id.clone();
                        let primary_decision = d.clone();
                        let secondary_decision = d.clone();
                        let all_primary = Rc::clone(&all);
                        let all_secondary = Rc::clone(&all);
                        rsx! {
                            div {
                                key: "{row_key}",
                                class: "ui-di__row",
                                "data-gone": s.gone.then_some("true"),
                                "inert": s.gone.then_some("true"),
                                div { class: "ui-di__row-inner",
                                    div {
                                        class: "ui-di__item",
                                        "data-open": s.open.then_some("true"),
                                        "data-answered": s.answered.is_some().then_some("true"),
                                        if s.open {
                                            p { class: "ui-di__title", dir: "auto", "{title}" }
                                        } else {
                                            button {
                                                r#type: "button",
                                                class: "ui-di__title ui-di__title--folded",
                                                dir: "auto",
                                                title: "{title}",
                                                onclick: move |_| {
                                                    state.write().entry(fold_id.clone()).or_default().open = true;
                                                },
                                                "{title}"
                                            }
                                        }
                                        div { class: "ui-di__body", "data-open": s.open.then_some("true"),
                                            div { class: "ui-di__body-inner",
                                                if let Some(detail) = detail {
                                                    p { class: "ui-di__detail", dir: "auto", "{detail}" }
                                                }
                                                div { class: "ui-di__actions",
                                                    if let Some(answer) = s.answered.clone() {
                                                        span { class: "ui-di__done", role: "status",
                                                            DoneIcon {}
                                                            span { dir: "auto", "{answer}" }
                                                        }
                                                    } else {
                                                        button {
                                                            r#type: "button",
                                                            class: "ui-di__btn ui-di__btn--primary",
                                                            dir: "auto",
                                                            onclick: move |_| decide(state, Rc::clone(&all_primary), on_decide, primary_decision.clone(), DecisionAnswer::Primary),
                                                            "{primary_label}"
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            class: "ui-di__btn",
                                                            dir: "auto",
                                                            onclick: move |_| decide(state, Rc::clone(&all_secondary), on_decide, secondary_decision.clone(), DecisionAnswer::Secondary),
                                                            "{secondary_label}"
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
                }
                p { class: "ui-di__empty", "Nothing waiting on you – new decisions show up here." }
            }
        }
    }
}

#[component]
fn AlertIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 16 16", width: "16", height: "16", "aria-hidden": "true",
            path { d: "M6.7 2.3a1.5 1.5 0 0 1 2.6 0l5.4 9.4A1.5 1.5 0 0 1 13.4 14H2.6a1.5 1.5 0 0 1-1.3-2.3Z", fill: "currentColor" }
            path { d: "M8 5.6v3.6", stroke: "#fff", stroke_width: "1.5", stroke_linecap: "round" }
            circle { cx: "8", cy: "11.4", r: "0.85", fill: "#fff" }
        }
    }
}

#[component]
fn DoneIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 16 16", width: "16", height: "16", "aria-hidden": "true",
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
