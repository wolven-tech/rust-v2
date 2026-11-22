//! Working trail panel: shows what an assistant is doing while it works. A live
//! elapsed timer, the sources it searches, and its thinking steps appear one
//! after another, ending with a completion line and the time taken.
//!
//! Design choice: `hidden_source_count` hides the last N sources of a search
//! step behind a "more sources" control.
//! Design choice: steps are revealed on a timer (`step_pause` milliseconds);
//! the elapsed timer advances in 100 ms ticks while steps remain.
//! Design choice: `on_finished` fires once per pass with the elapsed seconds.
//! Design choice: `repeat_sequence` restarts the trail one `step_pause` after
//! completion, by bumping a cycle counter that the reveal effect depends on.
//! Browser checks: the fade of steps, the draw of the completion check, the
//! source tooltip on focus, and the sequence restart under `repeat_sequence`.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/working-trail-panel.css");

#[derive(Clone, PartialEq, Debug)]
pub struct TrailSource {
    pub source_label: String,
    pub source_href: Option<String>,
    pub source_mark: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TrailStep {
    Search {
        query_label: String,
        sources: Vec<TrailSource>,
        hidden_source_count: usize,
    },
    Thinking {
        text: String,
    },
}

/// `0:07` for a duration in milliseconds.
pub fn elapsed_label(ms: u64) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Sources visible for a search step: all of them, less the hidden tail until revealed.
pub fn visible_source_count(total: usize, hidden: usize, revealed: bool) -> usize {
    if revealed {
        total
    } else {
        total.saturating_sub(hidden)
    }
}

/// Steps kept on the rail before the newest one: the active step and three before it.
const RAIL_STEPS_BEFORE_ACTIVE: usize = 3;

/// First step index on the rail, given how many steps are shown and whether earlier
/// steps are unfolded. Steps before the returned index are folded away.
pub fn rail_start(shown: usize, show_all: bool) -> usize {
    if show_all {
        0
    } else {
        shown.saturating_sub(RAIL_STEPS_BEFORE_ACTIVE + 1)
    }
}

/// A live trail of an assistant's searches and thinking, ending in a completion line.
#[component]
pub fn WorkingTrailPanel(
    #[props(default)] trail_steps: Vec<TrailStep>,
    #[props(default = "Finished thinking".to_string(), into)] done_label: String,
    #[props(default = 600)] step_pause: u32,
    #[props(default)] repeat_sequence: bool,
    #[props(default)] fold_when_done: bool,
    /// When false, the trail stays live after its last step until `finish_requested` is set.
    #[props(default = true)]
    auto_finish: bool,
    #[props(default)] finish_requested: bool,
    #[props(default)] on_finished: Option<EventHandler<f64>>,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-trail-{}", next_id()));
    // A zero pause has no reveal sequence, so every step is present on the first render.
    let mut revealed = use_signal(|| {
        if step_pause == 0 {
            trail_steps.len()
        } else {
            0
        }
    });
    let mut elapsed_ms = use_signal(|| 0_u64);
    let mut open = use_signal(|| true);
    let mut show_all = use_signal(|| false);
    let mut more = use_signal(Vec::<(usize, bool)>::new);
    let mut finished = use_signal(|| false);
    let mut announce = use_signal(String::new);
    let mut run_id = use_signal(|| 0_u64);
    let mut cycle = use_signal(|| 0_u64);

    let total = trail_steps.len();
    let shown = revealed().min(total);
    let rail_from = rail_start(shown, show_all());
    let folded = rail_start(shown, false);
    let fold_noun = if folded == 1 { "step" } else { "steps" };
    let fold_label = if show_all() {
        "Hide earlier steps".to_string()
    } else {
        format!("Show {folded} earlier {fold_noun}")
    };
    let fold_expanded = if show_all() { "true" } else { "false" };
    let done = finished();
    let timeline_open = open();
    let root_cls = class_list("kit-trail", &root_class);
    let elapsed = elapsed_label(elapsed_ms());
    let status_word = if done {
        done_label.clone()
    } else {
        "Working".to_string()
    };
    let expanded = if timeline_open { "true" } else { "false" };
    let head_id = format!("{base}-head");
    let body_id = format!("{base}-body");
    let finished_summary = format!("{done_label} · {elapsed}");
    let cycle_now = cycle();

    let steps_len = trail_steps.len();
    use_effect(use_reactive(
        (&steps_len, &step_pause, &cycle_now),
        move |(count, pause, _)| {
            // Peek: a tracked read of a signal this effect writes re-runs it forever.
            let current_run = *run_id.peek() + 1;
            run_id.set(current_run);
            revealed.set(0);
            elapsed_ms.set(0);
            finished.set(false);
            more.set(Vec::new());
            spawn(async move {
                loop {
                    if run_id() != current_run || revealed() >= count {
                        break;
                    }
                    sleep(pause).await;
                    if run_id() != current_run {
                        break;
                    }
                    revealed.set(revealed() + 1);
                }
            });
            spawn(async move {
                while run_id() == current_run && !finished() {
                    sleep(100).await;
                    if run_id() != current_run || finished() || count == 0 {
                        break;
                    }
                    elapsed_ms.set(elapsed_ms() + 100);
                }
            });
        },
    ));

    use_effect(use_reactive(
        (&revealed(), &steps_len, &(auto_finish || finish_requested)),
        move |(shown, count, may_finish)| {
            if shown >= count && count > 0 && may_finish && !*finished.peek() {
                finished.set(true);
                let ms = *elapsed_ms.peek();
                announce.set(format!("{done_label} after {}", elapsed_label(ms)));
                if let Some(handler) = on_finished {
                    handler.call(ms as f64 / 1000.0);
                }
                if fold_when_done {
                    // Folding now would unmount the last step before anyone sees it.
                    spawn(async move {
                        sleep(step_pause).await;
                        if *finished.peek() {
                            open.set(false);
                        }
                    });
                }
                if repeat_sequence {
                    spawn(async move {
                        sleep(step_pause).await;
                        let next = *cycle.peek() + 1;
                        cycle.set(next);
                    });
                }
            }
        },
    ));

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: root_cls, "data-state": if done { "done" } else { "live" },
            button {
                id: "{head_id}",
                r#type: "button",
                class: "kit-trail__head",
                "aria-expanded": "{expanded}",
                "aria-controls": "{body_id}",
                onclick: move |_| open.set(!open()),
                span { class: "kit-trail__timer", "{elapsed}" }
                span { class: "kit-trail__status", "{status_word}" }
                span { class: "kit-trail__mark", "aria-hidden": "true" }
            }
            if timeline_open {
                if folded > 0 {
                    button {
                        r#type: "button",
                        class: "kit-trail__fold",
                        "aria-expanded": "{fold_expanded}",
                        "aria-controls": "{body_id}",
                        onclick: move |_| show_all.set(!show_all()),
                        "{fold_label}"
                    }
                }
                ol { id: "{body_id}", class: "kit-trail__steps", role: "list",
                    for (index, step) in trail_steps.iter().cloned().enumerate().skip(rail_from).take(shown - rail_from) {
                        {
                            let key = format!("{base}-step-{index}");
                            let active = index + 1 == total && !done;
                            let step_class = if active { "kit-trail__step is-active" } else { "kit-trail__step" };
                            match step {
                                TrailStep::Thinking { text } => rsx! {
                                    li { key: "{key}", class: step_class,
                                        span { class: "kit-trail__node", "aria-hidden": "true" }
                                        p { class: "kit-trail__text", "{text}" }
                                    }
                                },
                                TrailStep::Search { query_label, sources, hidden_source_count } => {
                                    let revealed_more = more().iter().any(|(i, v)| *i == index && *v);
                                    let visible = visible_source_count(sources.len(), hidden_source_count, revealed_more);
                                    let remaining = sources.len() - visible;
                                    let shown_sources: Vec<TrailSource> = sources.iter().take(visible).cloned().collect();
                                    let noun = if remaining == 1 { "source" } else { "sources" };
                                    let more_label = format!("{remaining} more {noun}");
                                    let more_id = format!("{key}-more");
                                    rsx! {
                                        li { key: "{key}", class: step_class,
                                            span { class: "kit-trail__node", "aria-hidden": "true" }
                                            p { class: "kit-trail__query", "{query_label}" }
                                            ul { class: "kit-trail__chips",
                                                for (chip_index, source) in shown_sources.into_iter().enumerate() {
                                                    {
                                                        let chip_key = format!("{key}-src-{chip_index}");
                                                        let label = source.source_label.clone();
                                                        let mark = source.source_mark.clone().unwrap_or_else(|| label.chars().take(1).collect());
                                                        let href = source.source_href.clone();
                                                        rsx! {
                                                            li { key: "{chip_key}",
                                                                if let Some(href) = href {
                                                                    a { class: "kit-trail__chip", href: "{href}", target: "_blank",
                                                                        span { "aria-hidden": "true", "{mark}" }
                                                                        span { "{label}" }
                                                                        span { class: "kit-trail__tip", role: "tooltip", "{href}" }
                                                                    }
                                                                } else {
                                                                    span { class: "kit-trail__chip", tabindex: "0",
                                                                        span { "aria-hidden": "true", "{mark}" }
                                                                        span { "{label}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if remaining > 0 {
                                                button {
                                                    id: "{more_id}",
                                                    r#type: "button",
                                                    class: "kit-trail__more",
                                                    onclick: move |_| {
                                                        let mut list = more();
                                                        list.push((index, true));
                                                        more.set(list);
                                                    },
                                                    "{more_label}"
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
                div { class: "kit-trail__done",
                    span { class: "kit-trail__check", "aria-hidden": "true" }
                    span { "{finished_summary}" }
                }
            }
            p { class: "kit-trail__sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
