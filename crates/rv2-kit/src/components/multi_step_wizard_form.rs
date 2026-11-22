//! Multi-step wizard form: stages of one form, checked before each step, a review stage, then a
//! completion that can await the host's save.
//!
//! Design choice: values are a string map (`FormValues`) rather than a generic `V`, so one
//! component type serves every form. Chip entry lives in `WizardChips`. A Continue after a failed
//! check focuses the first invalid field, found by its `name` matching `FieldMessage::field`.
//!
//! Browser checks: the stage glides in from the side of travel; narrow widths collapse the step
//! list to "Step n of N".

use std::{collections::BTreeMap, future::Future, pin::Pin};

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/multi-step-wizard-form.css");

pub type FormValues = BTreeMap<String, String>;
/// A save the wizard awaits; `Err` shows beside the action and keeps the step.
pub type Task = Pin<Box<dyn Future<Output = Result<(), String>>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldMessage {
    pub field: String,
    pub text: String,
}

/// What a step's render function receives.
#[derive(Clone)]
pub struct StepContext {
    pub form_values: FormValues,
    pub set_form_values: Signal<FormValues>,
    pub field_errors: Vec<FieldMessage>,
    pub go_to: Callback<usize, ()>,
    pub go_next: Callback<(), ()>,
}

#[derive(Clone, PartialEq)]
pub struct WizardStep {
    pub step_id: String,
    pub step_title: String,
    pub step_heading: Option<String>,
    pub step_description: Option<String>,
    pub render_step: Callback<StepContext, Element>,
    pub check_step: Option<Callback<FormValues, Vec<FieldMessage>>>,
}

/// `Step 2 of 4`.
pub fn compact_line(index: usize, total: usize) -> String {
    format!("Step {} of {}", index + 1, total)
}

/// `done` before the current step, `current` on it, `todo` after.
pub fn segment_state(index: usize, current: usize) -> &'static str {
    match index.cmp(&current) {
        std::cmp::Ordering::Less => "done",
        std::cmp::Ordering::Equal => "current",
        std::cmp::Ordering::Greater => "todo",
    }
}

/// The next stage index, or `None` on the last stage.
pub fn next_index(index: usize, total: usize) -> Option<usize> {
    (index + 1 < total).then_some(index + 1)
}

/// Comma-separated entries, trimmed, empties dropped.
pub fn split_chips(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// Appends `value` unless it is empty or already present.
pub fn add_chip(list: &[String], value: &str) -> Vec<String> {
    let value = value.trim();
    let mut next = list.to_vec();
    if !value.is_empty() && !next.iter().any(|item| item == value) {
        next.push(value.to_string());
    }
    next
}

/// Splits a draft at its last comma: the entries before it are committed as chips, and the text
/// after it stays typed. `None` when there is no comma yet.
pub fn split_at_last_comma(text: &str) -> Option<(Vec<String>, String)> {
    let (done, rest) = text.rsplit_once(',')?;
    Some((split_chips(done), rest.to_string()))
}

/// A labelled chip input for lists such as emails. Enter or comma adds the typed entry; on an empty
/// field Enter calls `on_empty_enter` (the wizard's Continue). `on_make_primary`, when given, adds a
/// "Make primary" control to every chip after the first.
#[component]
pub fn WizardChips(
    values: Vec<String>,
    on_change: EventHandler<Vec<String>>,
    #[props(default, into)] label: String,
    #[props(default)] on_empty_enter: Option<Callback<(), ()>>,
    #[props(default)] on_make_primary: Option<EventHandler<String>>,
) -> Element {
    let id = use_hook(|| format!("kit-wiz-chips-{}", next_id()));
    let mut draft = use_signal(String::new);
    let shown = values.clone();
    rsx! {
        div { class: "kit-wiz__chips",
            label { class: "kit-wiz__label", r#for: "{id}", "{label}" }
            div { class: "kit-wiz__chip-box",
                for (position, chip) in shown.iter().enumerate() {
                    {
                        let remove_chip = chip.clone();
                        let promote_chip = chip.clone();
                        let remaining = shown.clone();
                        let show_promote = position > 0 && on_make_primary.is_some();
                        rsx! {
                            span { key: "{chip}", class: "kit-wiz__chip", "{chip}"
                                if show_promote {
                                    button {
                                        r#type: "button",
                                        class: "kit-wiz__chip-primary",
                                        "aria-label": "Make {promote_chip} primary",
                                        onclick: move |_| {
                                            if let Some(make) = on_make_primary {
                                                make.call(promote_chip.clone());
                                            }
                                        },
                                        "Make primary"
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "kit-wiz__chip-remove",
                                    "aria-label": "Remove {remove_chip}",
                                    onclick: move |_| {
                                        let next: Vec<String> = remaining.iter().filter(|v| **v != remove_chip).cloned().collect();
                                        on_change.call(next);
                                    },
                                    "×"
                                }
                            }
                        }
                    }
                }
                input {
                    id: "{id}",
                    class: "kit-wiz__input",
                    r#type: "text",
                    value: "{draft}",
                    oninput: {
                        let current = values.clone();
                        move |event: FormEvent| {
                            let value = event.value();
                            match split_at_last_comma(&value) {
                                Some((entries, rest)) => {
                                    let mut next = current.clone();
                                    for part in entries {
                                        next = add_chip(&next, &part);
                                    }
                                    draft.set(rest);
                                    if next != current {
                                        on_change.call(next);
                                    }
                                }
                                None => draft.set(value),
                            }
                        }
                    },
                    onkeydown: {
                        let current = values.clone();
                        move |event: KeyboardEvent| {
                            if event.key() == Key::Enter {
                                event.prevent_default();
                                event.stop_propagation();
                                let typed = draft();
                                if typed.trim().is_empty() {
                                    if let Some(advance) = on_empty_enter {
                                        advance.call(());
                                    }
                                } else {
                                    let mut next = current.clone();
                                    for part in split_chips(&typed) {
                                        next = add_chip(&next, &part);
                                    }
                                    draft.set(String::new());
                                    on_change.call(next);
                                }
                            } else if event.key() == Key::Backspace && draft().is_empty() && !current.is_empty() {
                                let mut next = current.clone();
                                next.pop();
                                on_change.call(next);
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
pub fn MultiStepWizardForm(
    step_list: Vec<WizardStep>,
    #[props(default)] initial_values: FormValues,
    on_complete: Callback<FormValues, Task>,
    #[props(default)] initial_index: usize,
    #[props(default)] current_index: Option<usize>,
    #[props(default)] on_index_change: EventHandler<usize>,
    #[props(default = "Finish".to_string(), into)] final_label: String,
    #[props(default = "Creating…".to_string(), into)] busy_label: String,
    #[props(default)] render_success: Option<Callback<(FormValues, Callback<(), ()>), Element>>,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-wiz-{}", next_id()));
    let mut index_state = use_signal(|| initial_index);
    let mut values = use_signal(|| initial_values.clone());
    let mut errors = use_signal(Vec::<FieldMessage>::new);
    let mut busy = use_signal(|| false);
    let mut submit_error = use_signal(|| None::<String>);
    let mut done = use_signal(|| false);

    let total = step_list.len();
    let index = current_index
        .unwrap_or(index_state())
        .min(total.saturating_sub(1));
    let is_last = total > 0 && index + 1 == total;

    let steps_next = step_list.clone();
    let scope_id = id.clone();
    let go_next = use_callback(move |_: ()| {
        let current = index_state().min(steps_next.len().saturating_sub(1));
        let messages = steps_next
            .get(current)
            .and_then(|step| step.check_step)
            .map(|check| check.call(values()))
            .unwrap_or_default();
        errors.set(messages.clone());
        if let Some(first) = messages.first() {
            focus_field(&scope_id, &first.field);
            return;
        }
        if let Some(next) = next_index(current, steps_next.len()) {
            index_state.set(next);
            on_index_change.call(next);
            return;
        }
        if busy() {
            return;
        }
        busy.set(true);
        submit_error.set(None);
        let task = on_complete.call(values());
        spawn(async move {
            match task.await {
                Ok(()) => done.set(true),
                Err(message) => {
                    submit_error.set(Some(message));
                    busy.set(false);
                }
            }
        });
    });

    let go_to = use_callback(move |target: usize| {
        index_state.set(target);
        on_index_change.call(target);
    });

    let reset = use_callback(move |_: ()| {
        values.set(initial_values.clone());
        errors.set(Vec::new());
        busy.set(false);
        submit_error.set(None);
        done.set(false);
        index_state.set(0);
        on_index_change.call(0);
    });

    if done() {
        let finished = values();
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            div { class: class_list("kit-wiz kit-wiz--done", &extra_class), role: "status",
                {match render_success {
                    Some(render) => render.call((finished.clone(), reset)),
                    None => rsx! {
                        h2 { class: "kit-wiz__heading", "All set" }
                        Button { variant: ButtonVariant::Secondary, label: "Start over", onclick: move |_| reset.call(()) }
                    },
                }}
            }
        };
    }

    let Some(step) = step_list.get(index).cloned() else {
        return rsx! {};
    };
    let body = step.render_step.call(StepContext {
        form_values: values(),
        set_form_values: values,
        field_errors: errors(),
        go_to,
        go_next,
    });
    let compact = compact_line(index, total);
    let heading = step
        .step_heading
        .clone()
        .unwrap_or_else(|| step.step_title.clone());
    let description = step.step_description.clone().unwrap_or_default();
    let action_label = if is_last {
        final_label.clone()
    } else {
        "Continue".to_string()
    };
    let busy_now = busy();
    let shown_label = if busy_now {
        busy_label.clone()
    } else {
        action_label
    };
    let first_error = errors().first().map(|m| m.text.clone());

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { id: "{id}", class: class_list("kit-wiz", &extra_class),
            p { class: "kit-wiz__compact", "{compact}" }
            ol { class: "kit-wiz__steps", role: "list",
                for (position, item) in step_list.iter().enumerate() {
                    li {
                        key: "{item.step_id}",
                        class: "kit-wiz__step",
                        "data-state": segment_state(position, index),
                        "aria-current": (position == index).then_some("step"),
                        if position < index {
                            button {
                                r#type: "button",
                                class: "kit-wiz__step-link",
                                onclick: move |_| go_to.call(position),
                                span { class: "kit-wiz__step-number", "{position + 1}" }
                                span { class: "kit-wiz__step-label", "{item.step_title}" }
                            }
                        } else {
                            span { class: "kit-wiz__step-link",
                                span { class: "kit-wiz__step-number", "{position + 1}" }
                                span { class: "kit-wiz__step-label", "{item.step_title}" }
                            }
                        }
                    }
                }
            }
            section { class: "kit-wiz__stage", "aria-live": "polite",
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Enter && !event.modifiers().shift() {
                        spawn(async move {
                            let typing = document::eval(
                                "const el = document.activeElement; return !!el && el.tagName === 'INPUT' && !['button', 'submit', 'checkbox', 'radio'].includes(el.type);",
                            )
                            .await;
                            if typing.ok().and_then(|value| value.as_bool()) == Some(true) {
                                go_next.call(());
                            }
                        });
                    }
                },
                p { class: "kit-wiz__sr", role: "status", "{compact}: {step.step_title}" }
                h2 { class: "kit-wiz__heading", "{heading}" }
                if !description.is_empty() {
                    p { class: "kit-wiz__description", "{description}" }
                }
                {body}
                if let Some(text) = first_error {
                    p { class: "kit-wiz__error", role: "alert", "{text}" }
                }
                if let Some(message) = submit_error() {
                    p { class: "kit-wiz__error", role: "alert", "{message}" }
                }
            }
            div { class: "kit-wiz__actions",
                if index > 0 {
                    Button { variant: ButtonVariant::Secondary, label: "Back", onclick: move |_| go_to.call(index - 1) }
                }
                Button { label: shown_label, loading: busy_now, onclick: move |_| go_next.call(()) }
            }
        }
    }
}

/// A labelled text input whose fixed `prefix` sits inside the field, before the entry. The hint, when
/// non-empty, is linked to the input for assistive technology.
#[component]
pub fn WizardField(
    #[props(into)] field_id: String,
    #[props(into)] field_name: String,
    #[props(into)] label: String,
    #[props(into)] prefix: String,
    value: String,
    #[props(default, into)] hint: String,
    on_input: EventHandler<String>,
) -> Element {
    let hint_id = format!("{field_id}-hint");
    let described_by = (!hint.is_empty()).then_some(hint_id.clone());
    rsx! {
        div { class: "kit-wiz__field",
            label { class: "kit-wiz__label", r#for: "{field_id}", "{label}" }
            div { class: "kit-wiz__prefixed",
                span { class: "kit-wiz__prefix", "{prefix}" }
                input {
                    id: "{field_id}",
                    name: "{field_name}",
                    class: "kit-wiz__input",
                    r#type: "text",
                    value: "{value}",
                    "aria-describedby": described_by,
                    oninput: move |event: FormEvent| on_input.call(event.value()),
                }
            }
            if !hint.is_empty() {
                p { id: "{hint_id}", class: "kit-wiz__hint", "{hint}" }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewRow {
    pub row_label: String,
    pub row_value: String,
    /// Index of the step that edits this row; `None` renders the row without an edit control.
    pub edit_step: Option<usize>,
}

/// Summary rows for the last step, each with an edit control that jumps back to its step.
#[component]
pub fn WizardReview(rows: Vec<ReviewRow>, go_to: Callback<usize, ()>) -> Element {
    rsx! {
        dl { class: "kit-wiz__review",
            for row in rows.iter().cloned() {
                div { key: "{row.row_label}", class: "kit-wiz__review-row",
                    dt { class: "kit-wiz__review-label", "{row.row_label}" }
                    dd { class: "kit-wiz__review-value", "{row.row_value}" }
                    if let Some(step) = row.edit_step {
                        button {
                            r#type: "button",
                            class: "kit-wiz__review-edit",
                            onclick: move |_| go_to.call(step),
                            "Edit"
                        }
                    }
                }
            }
        }
    }
}

/// Step bodies are consumer markup, so the wizard can only find a field by the `name` it was given.
fn focus_field(scope: &str, field: &str) {
    let selector = format!("#{scope} [name={field:?}]");
    document::eval(&format!("document.querySelector({selector:?})?.focus();"));
}
