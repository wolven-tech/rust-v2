//! Editable step plan: review an agent's proposed steps before anything runs.
//! Rename, remove, reorder or add steps, then start. While executing, each
//! step shows its state, and a failed step can be retried from that point.
//!
//! Design choice: with `plan_steps` supplied the host owns the list; without
//! it the component keeps the list it was given as `initial_steps`.
//! Design choice: a running step's clock counts from when this component
//! first saw it running, because the plan carries no wall-clock start time.
//! `SystemTime` is not used because it panics on `wasm32-unknown-unknown`.
//! Design choice: the drag grip is HTML drag and drop. Alt with the arrow
//! keys is the keyboard path to the same move.
//! Browser checks: drag reorder; the focus moves that follow Up and Down and
//! Alt+Arrow; the rename field; the retry and completed footers, which the
//! demo did not reach.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, sleep};

const CSS: Asset = asset!("/assets/editable-step-plan.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlanPhase {
    #[default]
    Drafting,
    Executing,
    Completed,
}

impl PlanPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Drafting => "drafting",
            Self::Executing => "executing",
            Self::Completed => "completed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepState {
    #[default]
    Pending,
    InProgress,
    Succeeded,
    Errored,
}

impl StepState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in-progress",
            Self::Succeeded => "succeeded",
            Self::Errored => "errored",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanStep {
    pub step_key: String,
    pub step_title: String,
    pub step_detail: Option<String>,
    pub tool_label: Option<String>,
    pub step_state: StepState,
    pub measured_ms: Option<u64>,
    pub error_text: Option<String>,
}

impl PlanStep {
    pub fn new(step_key: &str, step_title: &str) -> Self {
        Self {
            step_key: step_key.to_string(),
            step_title: step_title.to_string(),
            step_detail: None,
            tool_label: None,
            step_state: StepState::Pending,
            measured_ms: None,
            error_text: None,
        }
    }
}

fn position(steps: &[PlanStep], key: &str) -> Option<usize> {
    steps.iter().position(|step| step.step_key == key)
}

pub fn rename_step(steps: &[PlanStep], key: &str, title: &str) -> Vec<PlanStep> {
    let mut next = steps.to_vec();
    if let Some(step) = next.iter_mut().find(|step| step.step_key == key) {
        step.step_title = title.to_string();
    }
    next
}

pub fn remove_step(steps: &[PlanStep], key: &str) -> Vec<PlanStep> {
    steps
        .iter()
        .filter(|step| step.step_key != key)
        .cloned()
        .collect()
}

/// Moves the step at `from` to `to`; out-of-range indices return the list unchanged.
pub fn move_step(steps: &[PlanStep], from: usize, to: usize) -> Vec<PlanStep> {
    let mut next = steps.to_vec();
    if from < next.len() && to < next.len() && from != to {
        let step = next.remove(from);
        next.insert(to, step);
    }
    next
}

/// Appends a pending step with a key no other step uses.
pub fn append_step(steps: &[PlanStep], title: &str) -> Vec<PlanStep> {
    let mut number = steps.len() + 1;
    while position(steps, &format!("step-{number}")).is_some() {
        number += 1;
    }
    let mut next = steps.to_vec();
    next.push(PlanStep::new(&format!("step-{number}"), title));
    next
}

/// Resets the step at `key` and every later step to pending.
pub fn reset_from(steps: &[PlanStep], key: &str) -> Vec<PlanStep> {
    let start = position(steps, key).unwrap_or(steps.len());
    steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let mut step = step.clone();
            if index >= start {
                step.step_state = StepState::Pending;
                step.measured_ms = None;
                step.error_text = None;
            }
            step
        })
        .collect()
}

/// Steps finished (succeeded) out of all steps.
pub fn progress(steps: &[PlanStep]) -> (usize, usize) {
    let done = steps
        .iter()
        .filter(|step| step.step_state == StepState::Succeeded)
        .count();
    (done, steps.len())
}

/// `m:ss`, with the hours in front past one hour.
pub fn clock_text(ms: u64) -> String {
    let total = ms / 1000;
    let (h, m, s) = (total / 3600, total / 60 % 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// A review list of steps with rename, remove, reorder and add, and an
/// executing view that tracks each step.
#[component]
pub fn EditableStepPlan(
    #[props(into)] plan_heading: String,
    #[props(default)] plan_steps: Option<Vec<PlanStep>>,
    #[props(default)] initial_steps: Vec<PlanStep>,
    #[props(default)] plan_phase: PlanPhase,
    #[props(default)] estimate_text: Option<String>,
    #[props(default)] on_edit: Option<EventHandler<Vec<PlanStep>>>,
    on_start: EventHandler<Vec<PlanStep>>,
    #[props(default)] on_cancel: Option<EventHandler<()>>,
    #[props(default)] on_retry_from: Option<EventHandler<String>>,
    #[props(default)] on_repeat: Option<EventHandler<()>>,
    #[props(default)] class_name: String,
) -> Element {
    let controlled = plan_steps.is_some();
    let mut local = use_signal(|| initial_steps);
    let mut renaming = use_signal(|| None::<String>);
    let mut rename_text = use_signal(String::new);
    let mut draft_text = use_signal(String::new);
    let mut announce = use_signal(String::new);
    let mut dragging = use_signal(|| None::<usize>);
    let mut ms = use_signal(|| 0u64);
    let mut active_since = use_signal(|| (None::<String>, 0u64));
    use_future(move || async move {
        loop {
            sleep(1000).await;
            ms += 1000;
        }
    });

    let steps: Vec<PlanStep> = match &plan_steps {
        Some(steps) => steps.clone(),
        None => local(),
    };
    let drafting = plan_phase == PlanPhase::Drafting;
    let active_key = steps
        .iter()
        .find(|step| step.step_state == StepState::InProgress)
        .map(|step| step.step_key.clone());
    use_effect(use_reactive((&active_key,), move |(key,)| {
        if active_since.peek().0 != key {
            active_since.set((key, ms.peek().to_owned()));
        }
    }));
    let (done, total) = progress(&steps);
    let elapsed_active = active_since().1;
    let active_ms = ms().saturating_sub(elapsed_active);
    let elapsed_total: u64 = steps.iter().filter_map(|step| step.measured_ms).sum();
    let executing_clock = clock_text(elapsed_total + active_ms);
    let total_clock = clock_text(elapsed_total);
    let class = class_list("kit-plan", &class_name);
    let list_id = "kit-plan-list";
    let empty = steps.is_empty();
    let draft_value = draft_text();
    let announce_text = announce();

    let mut commit = move |next: Vec<PlanStep>| {
        if !controlled {
            local.set(next.clone());
        }
        if let Some(handler) = on_edit {
            handler.call(next);
        }
    };

    let add_base = steps.clone();
    let start_steps = steps.clone();
    let add_step = move |event: FormEvent| {
        event.prevent_default();
        let title = draft_text().trim().to_string();
        if title.is_empty() {
            return;
        }
        draft_text.set(String::new());
        announce.set(format!("Added step {title}."));
        commit(append_step(&add_base, &title));
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class, "data-phase": plan_phase.as_str(),
            header { class: "kit-plan__head",
                h2 { id: list_id, class: "kit-plan__heading", "{plan_heading}" }
                if plan_phase == PlanPhase::Executing {
                    span { class: "kit-plan__count", "{done} of {total}" }
                }
            }
            if empty {
                p { class: "kit-plan__empty", "There are no steps yet." }
            }
            ol { class: "kit-plan__list", "aria-labelledby": list_id,
                for (index, step) in steps.iter().cloned().enumerate() {
                    PlanRow {
                        step: step.clone(),
                        index,
                        total,
                        phase: plan_phase,
                        renaming: renaming() == Some(step.step_key.clone()),
                        rename_text: rename_text(),
                        on_begin_rename: {
                            let key = step.step_key.clone();
                            let title = step.step_title.clone();
                            move |_| {
                                rename_text.set(title.clone());
                                renaming.set(Some(key.clone()));
                            }
                        },
                        on_rename_input: move |text: String| rename_text.set(text),
                        on_rename_save: {
                            let key = step.step_key.clone();
                            let steps = steps.clone();
                            move |_| {
                                let title = rename_text().trim().to_string();
                                renaming.set(None);
                                if title.is_empty() {
                                    return;
                                }
                                announce.set(format!("Renamed step {}.", index + 1));
                                commit(rename_step(&steps, &key, &title));
                            }
                        },
                        on_rename_cancel: move |_| renaming.set(None),
                        on_remove: {
                            let key = step.step_key.clone();
                            let steps = steps.clone();
                            move |_| {
                                announce.set(format!("Removed step {}.", index + 1));
                                commit(remove_step(&steps, &key));
                            }
                        },
                        on_drag_start: move |_| dragging.set(Some(index)),
                        on_drop_here: {
                            let steps = steps.clone();
                            move |_| {
                                let Some(from) = dragging() else {
                                    return;
                                };
                                dragging.set(None);
                                if from != index {
                                    announce.set(format!("Moved to step {} of {}.", index + 1, steps.len()));
                                    commit(move_step(&steps, from, index));
                                }
                            }
                        },
                        on_move: {
                            let steps = steps.clone();
                            move |to: usize| {
                                if to < steps.len() && to != index {
                                    announce.set(format!("Moved to step {} of {}.", to + 1, steps.len()));
                                    commit(move_step(&steps, index, to));
                                    document::eval(&format!("document.getElementById(\"kit-plan-row-{to}\")?.focus();"));
                                }
                            }
                        },
                        on_retry: {
                            let key = step.step_key.clone();
                            let steps = steps.clone();
                            move |_| {
                                if !controlled {
                                    local.set(reset_from(&steps, &key));
                                }
                                if let Some(handler) = on_retry_from {
                                    handler.call(key.clone());
                                }
                            }
                        },
                    }
                }
            }
            if drafting {
                form { class: "kit-plan__add", onsubmit: add_step,
                    label { class: "kit-plan__sr", "for": "kit-plan-add", "Add a step" }
                    input {
                        id: "kit-plan-add",
                        r#type: "text",
                        class: "kit-plan__input",
                        placeholder: "Add a step",
                        value: "{draft_value}",
                        oninput: move |event: FormEvent| draft_text.set(event.value()),
                    }
                    button { r#type: "submit", class: "kit-plan__btn", "Add" }
                }
            }
            footer { class: "kit-plan__foot",
                if drafting {
                    if let Some(text) = estimate_text.clone() {
                        span { class: "kit-plan__estimate", "{text}" }
                    }
                    if let Some(cancel) = on_cancel {
                        button { r#type: "button", class: "kit-plan__btn", onclick: move |_| cancel.call(()), "Cancel" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-plan__btn kit-plan__btn--primary",
                        disabled: empty,
                        onclick: move |_| on_start.call(start_steps.clone()),
                        "Start"
                    }
                } else if plan_phase == PlanPhase::Executing {
                    span { class: "kit-plan__status", "{done} of {total} · {executing_clock}" }
                } else {
                    span { class: "kit-plan__status", "Total {total_clock}" }
                    if let Some(repeat) = on_repeat {
                        button { r#type: "button", class: "kit-plan__btn", onclick: move |_| repeat.call(()), "Repeat" }
                    }
                }
            }
            div { class: "kit-plan__sr", role: "status", "aria-live": "polite", "{announce_text}" }
        }
    }
}

#[component]
fn PlanRow(
    step: PlanStep,
    index: usize,
    total: usize,
    phase: PlanPhase,
    renaming: bool,
    rename_text: String,
    on_begin_rename: EventHandler<()>,
    on_rename_input: EventHandler<String>,
    on_rename_save: EventHandler<()>,
    on_rename_cancel: EventHandler<()>,
    on_remove: EventHandler<()>,
    on_drag_start: EventHandler<()>,
    on_drop_here: EventHandler<()>,
    on_move: EventHandler<usize>,
    on_retry: EventHandler<()>,
) -> Element {
    let drafting = phase == PlanPhase::Drafting;
    let number = index + 1;
    let row_id = format!("kit-plan-row-{index}");
    let state = step.step_state;
    let marker = match state {
        StepState::Pending => number.to_string(),
        StepState::InProgress => "…".to_string(),
        StepState::Succeeded => "✓".to_string(),
        StepState::Errored => "!".to_string(),
    };
    let state_word = match state {
        StepState::Pending => "Pending",
        StepState::InProgress => "Running",
        StepState::Succeeded => "Done",
        StepState::Errored => "Failed",
    };
    let title = step.step_title.clone();
    let detail = step.step_detail.clone();
    let tool = step.tool_label.clone();
    let error = step.error_text.clone();
    let aria_current = (state == StepState::InProgress).then_some("step");
    let up_id = format!("kit-plan-row-{}", index.saturating_sub(1));
    let down_id = format!("kit-plan-row-{}", index + 1);
    rsx! {
        li {
            id: "{row_id}",
            class: "kit-plan__row",
            "data-state": state.as_str(),
            "aria-current": aria_current,
            tabindex: if drafting { "0" } else { "-1" },
            draggable: if drafting { "true" } else { "false" },
            ondragstart: move |_| on_drag_start.call(()),
            ondragover: move |event: DragEvent| {
                if drafting {
                    event.prevent_default();
                }
            },
            ondrop: move |event: DragEvent| {
                event.prevent_default();
                on_drop_here.call(());
            },
            onkeydown: move |event: KeyboardEvent| {
                if !drafting || renaming {
                    return;
                }
                let alt = event.modifiers().alt();
                match event.key() {
                    Key::ArrowDown if alt && index + 1 < total => {
                        event.prevent_default();
                        on_move.call(index + 1);
                    }
                    Key::ArrowUp if alt && index > 0 => {
                        event.prevent_default();
                        on_move.call(index - 1);
                    }
                    Key::ArrowDown => {
                        event.prevent_default();
                        document::eval(&format!("document.getElementById({down_id:?})?.focus();"));
                    }
                    Key::ArrowUp => {
                        event.prevent_default();
                        document::eval(&format!("document.getElementById({up_id:?})?.focus();"));
                    }
                    Key::Enter => {
                        event.prevent_default();
                        on_begin_rename.call(());
                    }
                    Key::Delete => on_remove.call(()),
                    _ => {}
                }
            },
            span { class: "kit-plan__marker", "data-state": state.as_str(), "aria-hidden": "true", "{marker}" }
            if renaming {
                form {
                    class: "kit-plan__rename",
                    onsubmit: {
                        let submit_row_id = row_id.clone();
                        move |event: FormEvent| {
                            event.prevent_default();
                            on_rename_save.call(());
                            focus_after_frame(&submit_row_id);
                        }
                    },
                    input {
                        r#type: "text",
                        class: "kit-plan__input",
                        "aria-label": "Step name",
                        value: "{rename_text}",
                        onmounted: move |event: MountedEvent| {
                            let node = event.data();
                            spawn(async move {
                                let _ = node.set_focus(true).await;
                            });
                        },
                        oninput: move |event: FormEvent| on_rename_input.call(event.value()),
                        onkeydown: {
                            let cancel_row_id = row_id.clone();
                            move |event: KeyboardEvent| {
                                if event.key() == Key::Escape {
                                    event.prevent_default();
                                    on_rename_cancel.call(());
                                    focus_after_frame(&cancel_row_id);
                                }
                            }
                        },
                    }
                }
            } else {
                div { class: "kit-plan__body",
                    span { class: "kit-plan__title", "{title}" }
                    if let Some(text) = detail {
                        span { class: "kit-plan__detail", "{text}" }
                    }
                    if let Some(label) = tool {
                        span { class: "kit-plan__chip", "{label}" }
                    }
                    span { class: "kit-plan__sr", "{state_word}" }
                    if let Some(text) = error {
                        span { class: "kit-plan__error", "{text}" }
                    }
                }
            }
            if drafting && !renaming {
                span { class: "kit-plan__grip", "aria-hidden": "true", "⋮⋮" }
                button {
                    r#type: "button",
                    class: "kit-plan__icon",
                    "aria-label": "Remove step",
                    onclick: move |_| on_remove.call(()),
                    "×"
                }
            }
            if state == StepState::Errored && !drafting {
                button {
                    r#type: "button",
                    class: "kit-plan__btn",
                    onclick: move |_| on_retry.call(()),
                    "Retry from here"
                }
            }
        }
    }
}
