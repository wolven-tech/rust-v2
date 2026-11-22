//! Open agent questions shown above the chat input, answered by pointer or keyboard without leaving the conversation.
//!
//! Design choice: `preview_node` and `extra_blocks` are not carried in data. A pick choice shows its
//! `preview_text` inside its tile instead.
//! Design choice: `on_answer` receives an `AnswerEvent` carrying a question id and an `Answer`. The host
//! removes the answered question from `pending_questions`; the card then shows the next one.
//! Design choice: `reveal_token` is a counter. Each new value unfolds the card and focuses the selected row.
//! Browser checks: number keys 1 to 5 answer at once; the card collapses after the last answer; pick tiles
//! become rows when the container is narrow; focus never falls to the page body.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/agent-question-card.css");
/// Choices a question may offer; number keys 1 to 5 map to them.
pub const MAX_CHOICES: usize = 5;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QuestionKind {
    #[default]
    Choice,
    Pick,
    Approval,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Choice {
    pub choice_id: String,
    pub choice_title: String,
    pub choice_summary: Option<String>,
    pub is_suggested: bool,
    pub preview_text: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DetailRow {
    pub row_name: String,
    pub row_text: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Question {
    pub question_id: String,
    pub question_text: String,
    pub task_name: Option<String>,
    pub question_kind: QuestionKind,
    pub answer_choices: Vec<Choice>,
    pub detail_rows: Vec<DetailRow>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Answer {
    Chosen(String),
    FreeText(String),
    Approve,
    Decline,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AnswerEvent {
    pub question_id: String,
    pub answer: Answer,
}

/// Labels shown on the card. Every visible string the card renders comes from here.
#[derive(Clone, PartialEq, Debug)]
pub struct QuestionText {
    pub section_title: String,
    pub fold: String,
    pub show: String,
    pub suggested: String,
    pub approve: String,
    pub decline: String,
    pub placeholder: String,
    pub waiting: String,
    pub later: String,
}

impl Default for QuestionText {
    fn default() -> Self {
        Self {
            section_title: "Questions".into(),
            fold: "Fold".into(),
            show: "Show".into(),
            suggested: "Suggested".into(),
            approve: "Approve".into(),
            decline: "Decline".into(),
            placeholder: "Type an answer".into(),
            waiting: "waiting".into(),
            later: "Later".into(),
        }
    }
}

/// Choice index for a number key (1 to 5), when that choice exists.
pub fn number_key_choice(key: &str, choice_count: usize) -> Option<usize> {
    let digit = key.chars().next()?.to_digit(10)? as usize;
    (1..=MAX_CHOICES)
        .contains(&digit)
        .then(|| digit - 1)
        .filter(|index| *index < choice_count)
}

/// "n of N" for the pager; empty when nothing is waiting.
pub fn pager_text(index: usize, total: usize) -> String {
    if total == 0 {
        String::new()
    } else {
        format!("{} of {total}", index + 1)
    }
}

/// Index after a question is removed: the next question takes its place, clamped to the end.
pub fn index_after_removal(index: usize, remaining: usize) -> usize {
    index.min(remaining.saturating_sub(1))
}

/// Next row index with wrap-around, for arrow keys.
pub fn step_row(index: usize, count: usize, step: isize) -> usize {
    if count == 0 {
        return 0;
    }
    (index as isize + step).rem_euclid(count as isize) as usize
}

fn focus_element(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// Question card above the chat input.
#[component]
pub fn AgentQuestionCard(
    pending_questions: Vec<Question>,
    #[props(default)] on_answer: Option<EventHandler<AnswerEvent>>,
    #[props(default)] on_fold: Option<EventHandler<String>>,
    #[props(default)] start_folded: bool,
    #[props(default)] reveal_token: u64,
    #[props(default)] text_map: QuestionText,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-aqc-{}", next_id()));
    let mut folded = use_signal(|| start_folded);
    let mut index = use_signal(|| 0_usize);
    let mut row = use_signal(|| 0_usize);
    let mut free_text = use_signal(String::new);

    let reveal_id = id.clone();
    use_effect(use_reactive((&reveal_token,), move |(token,)| {
        if token > 0 {
            folded.set(false);
            focus_element(&format!("{reveal_id}-{token}-0"));
        }
    }));

    if pending_questions.is_empty() {
        return rsx! {};
    }

    let total = pending_questions.len();
    let current_index = index().min(total - 1);
    let question = pending_questions[current_index].clone();
    let waiting_text = format!("{total} {}", text_map.waiting);
    let pager = pager_text(current_index, total);
    let class = if extra_class.is_empty() {
        "kit-aqc".to_string()
    } else {
        format!("kit-aqc {extra_class}")
    };
    let question_id = question.question_id.clone();
    let choice_count = question.answer_choices.len().min(MAX_CHOICES);
    let selected = row().min(choice_count.saturating_sub(1));
    let is_pick = question.question_kind == QuestionKind::Pick;
    let is_approval = question.question_kind == QuestionKind::Approval;
    let text_labels = text_map.clone();

    let mut answer = {
        let handler = on_answer.clone();
        move |question_id: String, answer: Answer| {
            if let Some(h) = &handler {
                h.call(AnswerEvent {
                    question_id,
                    answer,
                });
            }
            free_text.set(String::new());
        }
    };
    let mut answer_key = answer.clone();
    let mut answer_approve = answer.clone();
    let mut answer_decline = answer.clone();
    let mut answer_text = answer.clone();

    let fold = {
        let handler = on_fold.clone();
        move |fold_id: String| {
            folded.set(true);
            if let Some(h) = &handler {
                h.call(fold_id);
            }
        }
    };
    let mut fold_key = fold.clone();
    let mut fold_later = fold.clone();
    let key_question_id = question_id.clone();
    let button_question_id = question_id.clone();
    let text_question_id = question_id.clone();
    let key_choices = question.answer_choices.clone();

    if folded() {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            div { id: "{id}", class: "{class}", "data-folded": "true",
                span { class: "kit-aqc__count", "{waiting_text}" }
                button {
                    r#type: "button",
                    class: "kit-aqc__show",
                    onclick: move |_| folded.set(false),
                    "{text_labels.show}"
                }
            }
        };
    }

    let choices: Vec<(usize, String, String, Option<String>, bool, bool)> = question
        .answer_choices
        .iter()
        .take(MAX_CHOICES)
        .enumerate()
        .map(|(position, choice)| {
            (
                position,
                choice.choice_id.clone(),
                choice.choice_title.clone(),
                if is_pick {
                    choice.preview_text.clone()
                } else {
                    choice.choice_summary.clone()
                },
                choice.is_suggested,
                position == selected,
            )
        })
        .collect();
    let details: Vec<(String, String)> = question
        .detail_rows
        .iter()
        .take(3)
        .map(|detail| (detail.row_name.clone(), detail.row_text.clone()))
        .collect();
    let task_name = question.task_name.clone().unwrap_or_default();
    let has_task = question.task_name.is_some();
    let question_text = question.question_text.clone();
    let suggested_label = text_map.suggested.clone();
    let approve_label = text_map.approve.clone();
    let decline_label = text_map.decline.clone();
    let placeholder = text_map.placeholder.clone();
    let later_label = text_map.later.clone();
    let section_title = text_map.section_title.clone();
    let fold_label = text_map.fold.clone();
    let row_prefix = format!("{id}-{reveal_token}");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            id: "{id}",
            class: "{class}",
            role: "group",
            aria_label: "{question_text}",
            "data-kind": if is_pick { "pick" } else if is_approval { "approval" } else { "choice" },
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    event.prevent_default();
                    fold_key(key_question_id.clone());
                    return;
                }
                if is_approval {
                    return;
                }
                let key = match event.key() {
                    Key::Character(text) => text,
                    _ => String::new(),
                };
                if let Some(at) = number_key_choice(&key, key_choices.len()) {
                    event.prevent_default();
                    if let Some(choice) = key_choices.get(at) {
                        answer_key(key_question_id.clone(), Answer::Chosen(choice.choice_id.clone()));
                    }
                    return;
                }
                match event.key() {
                    Key::ArrowDown | Key::ArrowRight => {
                        event.prevent_default();
                        row.set(step_row(selected, choice_count, 1));
                    }
                    Key::ArrowUp | Key::ArrowLeft => {
                        event.prevent_default();
                        row.set(step_row(selected, choice_count, -1));
                    }
                    Key::Home => {
                        event.prevent_default();
                        row.set(0);
                    }
                    Key::End => {
                        event.prevent_default();
                        row.set(choice_count.saturating_sub(1));
                    }
                    _ => {}
                }
            },
            header { class: "kit-aqc__head",
                span { class: "kit-aqc__label", "{section_title}" }
                div { class: "kit-aqc__pager",
                    button {
                        r#type: "button",
                        class: "kit-aqc__prev",
                        aria_label: "Previous question",
                        disabled: current_index == 0,
                        onclick: move |_| {
                            index.set(current_index.saturating_sub(1));
                            row.set(0);
                        },
                        "‹"
                    }
                    span { class: "kit-aqc__pager-text", "{pager}" }
                    button {
                        r#type: "button",
                        class: "kit-aqc__next",
                        aria_label: "Next question",
                        disabled: current_index + 1 >= total,
                        onclick: move |_| {
                            index.set((current_index + 1).min(total - 1));
                            row.set(0);
                        },
                        "›"
                    }
                }
                button {
                    r#type: "button",
                    class: "kit-aqc__later",
                    onclick: move |_| fold_later(button_question_id.clone()),
                    "{later_label}"
                }
            }
            if has_task {
                p { class: "kit-aqc__task", "{task_name}" }
            }
            p { class: "kit-aqc__question", "{question_text}" }
            if is_approval {
                dl { class: "kit-aqc__details",
                    for (name, text) in details {
                        div { key: "{name}", class: "kit-aqc__detail",
                            dt { "{name}" }
                            dd { "{text}" }
                        }
                    }
                }
                div { class: "kit-aqc__approval",
                    button {
                        r#type: "button",
                        class: "kit-aqc__decline",
                        onclick: {
                            let target = button_question_id.clone();
                            move |_| answer_decline(target.clone(), Answer::Decline)
                        },
                        "{decline_label}"
                    }
                    button {
                        r#type: "button",
                        class: "kit-aqc__approve",
                        onclick: {
                            let target = button_question_id.clone();
                            move |_| answer_approve(target.clone(), Answer::Approve)
                        },
                        "{approve_label}"
                    }
                }
            } else {
                div { class: if is_pick { "kit-aqc__list kit-aqc__list--pick" } else { "kit-aqc__list" }, role: "listbox", aria_label: "Answers",
                    for (position, choice_id, title, detail, suggested, active) in choices {
                        button {
                            key: "{choice_id}",
                            r#type: "button",
                            id: "{row_prefix}-{position}",
                            class: "kit-aqc__row",
                            role: "option",
                            "aria-selected": if active { "true" } else { "false" },
                            tabindex: if active { "0" } else { "-1" },
                            "data-position": "{position + 1}",
                            onfocus: move |_| row.set(position),
                            onclick: {
                                let target = button_question_id.clone();
                                let picked = choice_id.clone();
                                move |_| {
                                    row.set(position);
                                    answer(target.clone(), Answer::Chosen(picked.clone()))
                                }
                            },
                            span { class: "kit-aqc__num", aria_hidden: "true", "{position + 1}" }
                            span { class: "kit-aqc__text",
                                span { class: "kit-aqc__title", "{title}" }
                                if let Some(text) = detail {
                                    span { class: "kit-aqc__summary", "{text}" }
                                }
                            }
                            if suggested {
                                span { class: "kit-aqc__tag",
                                    svg { view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                                        path { d: "M8 2.2l1.5 3.6 3.8.3-2.9 2.5.9 3.7L8 10.6l-3.3 1.7.9-3.7-2.9-2.5 3.8-.3z" }
                                    }
                                    "{suggested_label}"
                                }
                            }
                        }
                    }
                }
                div { class: "kit-aqc__free",
                    input {
                        r#type: "text",
                        class: "kit-aqc__input",
                        id: "{row_prefix}-free",
                        placeholder: "{placeholder}",
                        aria_label: "Free answer",
                        value: "{free_text}",
                        oninput: move |event: FormEvent| free_text.set(event.value()),
                        onkeydown: move |event: KeyboardEvent| {
                            if event.key() == Key::Enter {
                                event.prevent_default();
                                let text = free_text().trim().to_string();
                                if !text.is_empty() {
                                    answer_text(text_question_id.clone(), Answer::FreeText(text));
                                }
                            }
                        },
                    }
                }
            }
            p { class: "kit-aqc__live", role: "status", "aria-live": "polite", "{waiting_text}" }
            span { class: "kit-aqc__fold-hint", aria_hidden: "true", "{fold_label}" }
        }
    }
}
