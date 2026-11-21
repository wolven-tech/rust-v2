//! Progress tracker: a checklist with a progress bar. Auto play ticks the open
//! steps one after another and stops as soon as the viewer presses or clicks.
//!
//! Port losses: none.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::internal::{class_list, sleep};
use crate::reaction_bar::exact_count;

const CSS: Asset = asset!("/assets/progress-tracker.css");

#[derive(Clone, PartialEq)]
pub struct ProgressStep {
    pub id: String,
    pub label: String,
    pub done: bool,
    /// Small icon on the right, e.g. the app the step belongs to.
    pub icon: Option<Element>,
}

/// Ids of the steps marked done in the list.
pub fn initial_done(steps: &[ProgressStep]) -> Vec<String> {
    steps
        .iter()
        .filter(|s| s.done)
        .map(|s| s.id.clone())
        .collect()
}

/// Ids of done steps that were not in `seen`. Steps that arrive later bring
/// their own done state.
pub fn newly_done(seen: &[String], steps: &[ProgressStep]) -> Vec<String> {
    steps
        .iter()
        .filter(|s| s.done && !seen.contains(&s.id))
        .map(|s| s.id.clone())
        .collect()
}

/// Flips membership of `id` in `done`.
pub fn toggle_id(done: &[String], id: &str) -> Vec<String> {
    if done.iter().any(|d| d == id) {
        done.iter().filter(|d| d.as_str() != id).cloned().collect()
    } else {
        let mut next = done.to_vec();
        next.push(id.to_string());
        next
    }
}

/// Share of steps done, from 0 to 1. An empty list is 0.
pub fn progress_fraction(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64
    }
}

fn toggle_step(
    mut done: Signal<Vec<String>>,
    steps: &[ProgressStep],
    on_change: Option<EventHandler<Vec<ProgressStep>>>,
    id: &str,
) {
    let next = toggle_id(&done.peek(), id);
    done.set(next.clone());
    if let Some(handler) = on_change {
        let updated = steps
            .iter()
            .map(|s| ProgressStep {
                done: next.contains(&s.id),
                ..s.clone()
            })
            .collect();
        handler.call(updated);
    }
}

#[component]
pub fn ProgressTracker(
    #[props(default = "Getting started".to_string(), into)] title: String,
    steps: Vec<ProgressStep>,
    /// Ticks the open steps one after another, then starts over. Stops as soon as someone interacts.
    #[props(default)]
    auto_play: bool,
    /// Pause between two automatic steps, in ms.
    #[props(default = 2200)]
    interval: u32,
    #[props(default)] on_change: Option<EventHandler<Vec<ProgressStep>>>,
    #[props(default)] class: String,
) -> Element {
    let mut done = use_signal(|| initial_done(&steps));
    let mut playing = use_signal(|| auto_play);
    let mut seen = use_signal(|| steps.iter().map(|s| s.id.clone()).collect::<Vec<_>>());
    let mut ticket = use_signal(|| 0u32);

    // Steps added later bring their own done state; steps that go away still count as seen.
    use_effect(use_reactive((&steps,), move |(steps,)| {
        let known = seen.peek().clone();
        let arrivals = newly_done(&known, &steps);
        let mut all = known;
        for step in &steps {
            if !all.contains(&step.id) {
                all.push(step.id.clone());
            }
        }
        seen.set(all);
        if !arrivals.is_empty() {
            let mut next = done.peek().clone();
            for arrival in arrivals {
                if !next.contains(&arrival) {
                    next.push(arrival);
                }
            }
            done.set(next);
        }
    }));

    // Auto play: after each pause, tick the next open step; when all are done, start over.
    use_effect(use_reactive(
        (&playing(), &done(), &steps, &interval),
        move |(on, done_ids, steps, interval)| {
            let mine = *ticket.peek() + 1;
            ticket.set(mine);
            if !on {
                return;
            }
            let open = steps
                .iter()
                .find(|s| !done_ids.contains(&s.id))
                .map(|s| s.id.clone());
            let delay = if open.is_some() {
                interval
            } else {
                interval * 2
            };
            spawn(async move {
                sleep(delay).await;
                if *ticket.peek() != mine {
                    return;
                }
                match open {
                    Some(open) => toggle_step(done, &steps, on_change, &open),
                    None => done.set(initial_done(&steps)),
                }
            });
        },
    ));

    let total = steps.len();
    let count = steps.iter().filter(|s| done().contains(&s.id)).count();
    let progress = progress_fraction(count, total);
    let count_text = exact_count(count as u64);
    let total_text = exact_count(total as u64);
    let value_text = format!("{count_text} of {total_text} done");
    let fill_style = format!("transform: scaleX({progress})");
    let class = class_list("ui-progress", &class);
    let shared = Rc::new(steps.clone());

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-complete": (count == total).then_some("true"),
            onpointerdown: move |_| playing.set(false),
            onkeydown: move |_| playing.set(false),
            div { class: "ui-progress__head",
                h3 { class: "ui-progress__title", dir: "auto", title: "{title}", "{title}" }
                span { class: "ui-progress__count", "aria-hidden": "true",
                    span { key: "{count}", class: "ui-progress__digit", "{count_text}" }
                    "/{total_text}"
                }
            }
            div {
                class: "ui-progress__bar",
                role: "progressbar",
                aria_label: "{title}",
                aria_valuemin: "0",
                aria_valuemax: "{total}",
                aria_valuenow: "{count}",
                aria_valuetext: "{value_text}",
                span { class: "ui-progress__fill", style: "{fill_style}",
                    span { key: "{count}", class: "ui-progress__shine" }
                }
            }
            ul { class: "ui-progress__list",
                for (index, step) in steps.iter().enumerate() {
                    {
                        let checked = done().contains(&step.id);
                        let delay = 180 + index.min(12) * 90;
                        let step_id = step.id.clone();
                        let all = Rc::clone(&shared);
                        let label_title = (!step.label.is_empty()).then(|| step.label.clone());
                        let label_text = if step.label.is_empty() { "\u{a0}".to_string() } else { step.label.clone() };
                        rsx! {
                            li { key: "{step_id}", style: "animation-delay: {delay}ms",
                                button {
                                    r#type: "button",
                                    role: "checkbox",
                                    aria_checked: if checked { "true" } else { "false" },
                                    class: "ui-progress__step",
                                    onclick: move |_| toggle_step(done, &all, on_change, &step_id),
                                    span { class: "ui-progress__check", "aria-hidden": "true",
                                        svg { view_box: "0 0 24 24",
                                            path { d: "M6.5 12.5l3.5 3.5 7.5-8", path_length: "1" }
                                        }
                                    }
                                    span { class: "ui-progress__label", title: label_title,
                                        span { dir: "auto", "{label_text}" }
                                    }
                                    if let Some(icon) = step.icon.clone() {
                                        span { class: "ui-progress__icon", "aria-hidden": "true", {icon} }
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
