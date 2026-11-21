//! Candidate race board: several candidate results for one task, side by side while they run, and a choice of which to use.
//!
//! Design choice: the in-use candidate resolves as `chosen_key`, then the person's own last pick, then the best
//! finished candidate. A `chosen_key` naming a candidate that is not choosable still shows it in use, with no
//! choose control.
//! Design choice: marks are text glyphs (`mark: Option<String>`), not nodes.
//! Design choice: the choose control is a native radio inside one radio group, so arrow keys move the choice.
//! Browser checks: row grow-in, progress sheen, best and in-use marker transitions, phone-width reflow.

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/candidate-race-board.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    Running,
    Finished,
    Halted,
}

impl RunState {
    pub fn word(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Finished => "finished",
            Self::Halted => "halted",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Candidate {
    pub candidate_key: String,
    pub display_name: String,
    pub method_summary: String,
    pub model_id: Option<String>,
    pub mark: Option<String>,
    pub run_state: RunState,
    /// 0..=1. A halted candidate keeps the value where it stopped.
    pub fraction_done: Option<f64>,
    pub current_action: Option<String>,
    /// 0..=100, set once finished.
    pub quality_score: Option<f64>,
    pub outcome_note: Option<String>,
    pub elapsed_secs: Option<f64>,
    pub run_cost: Option<f64>,
}

/// Key of the best finished candidate: the highest score, first on ties.
pub fn best_key(list: &[Candidate]) -> Option<String> {
    list.iter()
        .filter(|c| c.run_state == RunState::Finished)
        .filter_map(|c| c.quality_score.map(|s| (c, s)))
        .fold(None::<(&Candidate, f64)>, |best, (c, s)| match best {
            Some((_, b)) if b >= s => best,
            _ => Some((c, s)),
        })
        .map(|(c, _)| c.candidate_key.clone())
}

/// The candidate in use: the caller's choice, then the person's own pick, then the best finished one.
pub fn in_use_key(
    list: &[Candidate],
    prop_choice: Option<&str>,
    local_choice: Option<&str>,
) -> Option<String> {
    prop_choice
        .or(local_choice)
        .map(str::to_string)
        .or_else(|| best_key(list))
        .filter(|k| list.iter().any(|c| c.candidate_key == *k))
}

/// Only finished candidates can be adopted.
pub fn choosable(candidate: &Candidate) -> bool {
    candidate.run_state == RunState::Finished
}

/// Progress as a whole percentage. Finished is always 100, and an unknown fraction reads 0.
pub fn progress_percent(candidate: &Candidate) -> f64 {
    match candidate.run_state {
        RunState::Finished => 100.0,
        _ => candidate.fraction_done.unwrap_or(0.0).clamp(0.0, 1.0) * 100.0,
    }
}

/// Count line: finished of total.
pub fn finished_line(list: &[Candidate]) -> String {
    let done = list
        .iter()
        .filter(|c| c.run_state == RunState::Finished)
        .count();
    format!("{done} of {} finished", list.len())
}

/// Candidates side by side while they run, with a choice of which result to use.
#[component]
pub fn CandidateRaceBoard(
    candidate_list: Vec<Candidate>,
    #[props(default)] chosen_key: Option<String>,
    #[props(default)] on_choose: Option<EventHandler<Candidate>>,
    #[props(default)] format_spend: Option<Callback<f64, String>>,
    #[props(default)] task_text: String,
    #[props(default)] board_title: Option<String>,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-crb-{}", next_id()));
    let mut local_choice = use_signal(|| None::<String>);

    let class = class_list("kit-crb", &extra_class);
    let title = board_title.unwrap_or_else(|| format!("Best of {}", candidate_list.len()));
    let count_line = finished_line(&candidate_list);
    let best = best_key(&candidate_list);
    let in_use = in_use_key(
        &candidate_list,
        chosen_key.as_deref(),
        local_choice().as_deref(),
    );
    let spend = |amount: f64| match format_spend {
        Some(f) => f.call(amount),
        None => format!("${amount:.2}"),
    };

    if candidate_list.is_empty() {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            section { class: "{class}", "aria-label": "{title}",
                h2 { class: "kit-crb__title", "{title}" }
                if !task_text.is_empty() { p { class: "kit-crb__task", "{task_text}" } }
                p { class: "kit-crb__empty", "Nothing to compare yet." }
            }
        };
    }

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title",
            header { class: "kit-crb__head",
                h2 { id: "{base}-title", class: "kit-crb__title", "{title}" }
                if !task_text.is_empty() { p { class: "kit-crb__task", "{task_text}" } }
                p { class: "kit-crb__count", role: "status", "aria-live": "polite", "{count_line}" }
            }
            div { class: "kit-crb__rows", role: "radiogroup", "aria-label": "Candidates",
                for (i, c) in candidate_list.iter().enumerate() {
                    {
                        let is_best = best.as_deref() == Some(c.candidate_key.as_str());
                        let is_used = in_use.as_deref() == Some(c.candidate_key.as_str());
                        let can_choose = choosable(c);
                        let pct = progress_percent(c);
                        let bar_style = format!("width: {pct:.1}%;");
                        let action = match c.run_state {
                            RunState::Finished => c.outcome_note.clone().unwrap_or_default(),
                            RunState::Halted => c.outcome_note.clone().unwrap_or_else(|| "Halted".to_string()),
                            RunState::Running => c.current_action.clone().unwrap_or_else(|| "Starting".to_string()),
                        };
                        let score = c.quality_score.map(|s| format!("{s:.0}"));
                        let spend_text = c.run_cost.map(|v| spend(v));
                        let elapsed = c.elapsed_secs.map(|s| format!("{s:.0}s"));
                        let mark = c.mark.clone().unwrap_or_else(|| c.display_name.chars().take(1).collect());
                        let state = c.run_state.word();
                        let radio_label = match score.clone() {
                            Some(s) => format!("Use {}, score {s}", c.display_name),
                            None => format!("Use {}", c.display_name),
                        };
                        let dom_id = format!("{base}-c-{i}");
                        let name = c.display_name.clone();
                        let method = c.method_summary.clone();
                        let model = c.model_id.clone();
                        let candidate = c.clone();
                        rsx! {
                            div { key: "{c.candidate_key}", class: "kit-crb__row", "data-state": state,
                                "data-best": is_best.then_some("true"), "data-in-use": is_used.then_some("true"),
                                "aria-label": "{name}",
                                span { class: "kit-crb__mark", "aria-hidden": "true", "{mark}" }
                                div { class: "kit-crb__main",
                                    div { class: "kit-crb__line",
                                        span { class: "kit-crb__name", "{name}" }
                                        span { class: "kit-crb__state", "{state}" }
                                        if is_best { span { class: "kit-crb__tag", "best" } }
                                        if is_used { span { class: "kit-crb__tag", "in use" } }
                                    }
                                    if let Some(m) = model { span { class: "kit-crb__model", "{m}" } }
                                    span { class: "kit-crb__method", "{method}" }
                                    div { class: "kit-crb__bar", "aria-hidden": "true", span { class: "kit-crb__fill", style: "{bar_style}" } }
                                    span { class: "kit-crb__action", "{action}" }
                                }
                                div { class: "kit-crb__figures",
                                    if let Some(s) = score { span { class: "kit-crb__score", "{s}" } }
                                    if let Some(sp) = spend_text { span { class: "kit-crb__spend", "{sp}" } }
                                    if let Some(e) = elapsed { span { class: "kit-crb__time", "{e}" } }
                                }
                                if can_choose {
                                    label { class: "kit-crb__choose", r#for: "{dom_id}",
                                        input {
                                            id: "{dom_id}", r#type: "radio", name: "{base}-choice",
                                            checked: is_used, "aria-label": "{radio_label}",
                                            onchange: move |_| {
                                                local_choice.set(Some(candidate.candidate_key.clone()));
                                                if let Some(h) = on_choose {
                                                    h.call(candidate.clone());
                                                }
                                            },
                                        }
                                        span { "Use" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !task_text.is_empty() {
                p { class: "kit-crb__hint", "Choose the result to keep. Running and halted candidates cannot be chosen." }
            }
        }
    }
}
