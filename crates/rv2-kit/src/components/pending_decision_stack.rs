//! Lists each point where an agent run paused for a person; the open request on top is decided in place.
//!
//! Design choice: decided items are never removed from `decision_items`. The component keeps status
//! overrides so a decision moves the item into history at once; the host can reconcile by passing the
//! updated status.
//! Design choice: only the top open item is interactive. Items behind it are visual peeks; arrow keys
//! and a click on a peek rotate which open item is on top.
//! Design choice: "Revise first" edits the request payload only. Change lines stay read-only.
//! Design choice: the stack does not reserve the height of its tallest card, and the motion is CSS only.
//! Browser checks: the decided card drops toward history; focus lands on the next top card's Approve
//! control; peek cards scale behind the top card; a rejected card shakes once.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/pending-decision-stack.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RequestKind {
    #[default]
    Tool,
    File,
    Agent,
    Message,
}

impl RequestKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tool => "tool",
            Self::File => "file",
            Self::Agent => "agent",
            Self::Message => "message",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RequestStatus {
    #[default]
    Open,
    Approved,
    Edited,
    Rejected,
}

impl RequestStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Blocking",
            Self::Approved => "Approved",
            Self::Edited => "Edited",
            Self::Rejected => "Rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionOutcome {
    Approved,
    Edited,
    Rejected,
}

impl DecisionOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Edited => "edited",
            Self::Rejected => "rejected",
        }
    }

    fn status(self) -> RequestStatus {
        match self {
            Self::Approved => RequestStatus::Approved,
            Self::Edited => RequestStatus::Edited,
            Self::Rejected => RequestStatus::Rejected,
        }
    }
}

/// Lines added and removed in one file.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct FileChange {
    pub path: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DecisionItem {
    pub id: String,
    pub requester: String,
    pub requester_mark: Option<String>,
    pub request_kind: RequestKind,
    pub request_summary: String,
    pub request_reason: String,
    pub request_status: RequestStatus,
    /// Epoch seconds.
    pub requested_at: Option<i64>,
    pub request_payload: Option<String>,
    pub change_lines: Vec<FileChange>,
    pub touched_paths: Vec<String>,
    pub prior_steps: Vec<String>,
    pub waiting_steps: Vec<String>,
    pub person_note: Option<String>,
}

/// Payload of `on_resolve`. `revised_value` is set only for `Edited`.
#[derive(Clone, PartialEq, Debug)]
pub struct Resolution {
    pub id: String,
    pub outcome: DecisionOutcome,
    pub revised_value: Option<String>,
}

pub type StatusOverrides = Vec<(String, RequestStatus)>;

pub fn effective_status(item: &DecisionItem, overrides: &StatusOverrides) -> RequestStatus {
    overrides
        .iter()
        .find(|(id, _)| *id == item.id)
        .map(|(_, status)| *status)
        .unwrap_or(item.request_status)
}

/// Ids of open items, in arrival order.
pub fn open_ids(items: &[DecisionItem], overrides: &StatusOverrides) -> Vec<String> {
    items
        .iter()
        .filter(|item| effective_status(item, overrides) == RequestStatus::Open)
        .map(|item| item.id.clone())
        .collect()
}

/// Steps held by open items, and the number of open items holding them.
pub fn blocking_counts(items: &[DecisionItem], overrides: &StatusOverrides) -> (usize, usize) {
    items
        .iter()
        .filter(|item| effective_status(item, overrides) == RequestStatus::Open)
        .filter(|item| !item.waiting_steps.is_empty())
        .fold((0, 0), |(steps, held), item| {
            (steps + item.waiting_steps.len(), held + 1)
        })
}

/// Heading state word: "Paused" while open items remain, "Stopped" after a rejection, then "Running" or "All decided".
pub fn run_state_word(open: usize, running: bool, rejected: usize) -> &'static str {
    if open > 0 {
        "Paused"
    } else if rejected > 0 {
        "Stopped"
    } else if running {
        "Running"
    } else {
        "All decided"
    }
}

/// Index of the open item on top, given how far the stack has been rotated.
pub fn top_position(open_count: usize, rotation: usize) -> Option<usize> {
    (open_count > 0).then(|| rotation % open_count)
}

pub fn announce_text(outcome: DecisionOutcome, summary: &str, remaining: usize) -> String {
    let verb = match outcome {
        DecisionOutcome::Approved => "Approved",
        DecisionOutcome::Edited => "Approved with edits",
        DecisionOutcome::Rejected => "Rejected",
    };
    format!("{verb}: {summary}. {remaining} remaining.")
}

/// Rotates the stack by `step` (positive forward) over `count` open items.
pub fn rotate(rotation: usize, step: isize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (rotation as isize + step).rem_euclid(count as isize) as usize
}

fn format_clock(epoch_secs: i64) -> String {
    let day_secs = epoch_secs.rem_euclid(86_400);
    format!("{:02}:{:02}", day_secs / 3600, (day_secs % 3600) / 60)
}

fn focus_element(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// Stack of open decisions with a history list of decided ones.
#[component]
pub fn PendingDecisionStack(
    decision_items: Vec<DecisionItem>,
    #[props(default, into)] run_subject: String,
    #[props(default)] is_running: bool,
    #[props(default)] on_resolve: Option<EventHandler<Resolution>>,
    #[props(default = "Pending decisions".to_string(), into)] section_title: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-pds-{}", next_id()));
    let mut overrides = use_signal(StatusOverrides::new);
    let mut rotation = use_signal(|| 0_usize);
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(String::new);
    let mut expanded = use_signal(Vec::<String>::new);
    let mut announce = use_signal(String::new);
    let mut focus_next = use_signal(|| false);

    let current = overrides();
    let open = open_ids(&decision_items, &current);
    let open_total = open.len();
    let rejected_total = decision_items
        .iter()
        .filter(|item| effective_status(item, &current) == RequestStatus::Rejected)
        .count();
    let decided_total = decision_items.len() - open_total;
    let top_index = top_position(open_total, rotation());
    let top_id = top_index.map(|index| open[index].clone());
    let word = run_state_word(open_total, is_running, rejected_total);
    let (blocked_steps, blocked_items) = blocking_counts(&decision_items, &current);

    {
        let top_key = top_id.clone();
        use_effect(use_reactive((&top_key,), move |(top,)| {
            if focus_next() {
                if let Some(next) = top {
                    focus_element(&format!("{next}-approve"));
                }
                focus_next.set(false);
            }
        }));
    }

    let decide = {
        let items = decision_items.clone();
        let handler = on_resolve.clone();
        move |item_id: String, outcome: DecisionOutcome, revised: Option<String>| {
            let Some(item) = items.iter().find(|item| item.id == item_id) else {
                return;
            };
            let remaining = open_total.saturating_sub(1);
            let summary = item.request_summary.clone();
            overrides.with_mut(|list| {
                list.retain(|(known, _)| *known != item_id);
                list.push((item_id.clone(), outcome.status()));
            });
            editing.set(false);
            rotation.set(0);
            announce.set(announce_text(outcome, &summary, remaining));
            focus_next.set(true);
            if let Some(h) = &handler {
                h.call(Resolution {
                    id: item_id,
                    outcome,
                    revised_value: revised,
                });
            }
        }
    };

    let top_card = top_id.as_ref().and_then(|top| {
        decision_items.iter().find(|item| item.id == *top).cloned()
    }).map(|item| {
        let DecisionItem {
            id: item_id,
            requester,
            requester_mark,
            request_kind,
            request_summary,
            request_reason,
            requested_at,
            request_payload,
            change_lines,
            prior_steps,
            waiting_steps,
            person_note,
            ..
        } = item;
        let kind_text = request_kind.as_str();
        let blocking = !waiting_steps.is_empty();
        let time_text = requested_at.map(format_clock).unwrap_or_default();
        let card_id = format!("{id}-card");
        let summary_id = format!("{id}-summary");
        let prior_count = prior_steps.len();
        let prior_label = format!("Prior steps ({prior_count})");
        let waiting_text = waiting_steps.join(", ");
        let note_text = person_note.map(|note| format!("Your note: {note}"));
        let payload_text = request_payload.clone().unwrap_or_default();
        let has_payload = request_payload.is_some();
        let files: Vec<(String, Vec<String>, Vec<String>)> = change_lines
            .into_iter()
            .map(|file| (file.path, file.removed, file.added))
            .collect();
        let approve_id = item_id.clone();
        let reject_id = item_id.clone();
        let save_id = item_id.clone();
        let key_id = item_id.clone();
        let mut key_decide = decide.clone();
        let mut approve_btn = decide.clone();
        let mut reject_btn = decide.clone();
        let mut save_btn = decide.clone();
        let keys_draft = payload_text.clone();
        let back_draft = payload_text.clone();
        let revise_seed = payload_text.clone();
        let revise_focus = format!("{id}-revise");
        let key_focus = revise_focus.clone();
        let back_focus = revise_focus.clone();
        let revise_click = revise_focus.clone();
        rsx! {
            article {
                id: "{card_id}",
                class: "kit-pds__card",
                "data-kind": kind_text,
                "data-status": "open",
                aria_labelledby: "{summary_id}",
                onkeydown: move |event: KeyboardEvent| {
                    let mods = event.modifiers();
                    if event.key() == Key::Escape && editing() {
                        event.prevent_default();
                        editing.set(false);
                        draft.set(keys_draft.clone());
                        focus_element(&key_focus);
                    } else if event.key() == Key::Enter && (mods.meta() || mods.ctrl()) && editing() {
                        event.prevent_default();
                        key_decide(key_id.clone(), DecisionOutcome::Edited, Some(draft()));
                    }
                },
                div { class: "kit-pds__meta",
                    if let Some(mark) = requester_mark {
                        span { class: "kit-pds__mark", aria_hidden: "true", "{mark}" }
                    }
                    span { class: "kit-pds__requester", "{requester}" }
                    if !time_text.is_empty() {
                        span { class: "kit-pds__time", "{time_text}" }
                    }
                    span { class: "kit-pds__kind", "{kind_text}" }
                    if blocking {
                        span { class: "kit-pds__badge", "data-tone": "blocking", "Blocking" }
                    }
                }
                h3 { id: "{summary_id}", class: "kit-pds__summary", "{request_summary}" }
                p { class: "kit-pds__reason", "{request_reason}" }
                if editing() {
                    textarea {
                        class: "kit-pds__editor",
                        aria_label: "Revised payload",
                        value: "{draft}",
                        oninput: move |event: FormEvent| draft.set(event.value()),
                    }
                } else if has_payload {
                    pre { class: "kit-pds__payload", "{payload_text}" }
                }
                for (path, removed, added) in files {
                    div { key: "{path}", class: "kit-pds__file",
                        code { class: "kit-pds__path", "{path}" }
                        ul { class: "kit-pds__lines",
                            for line in removed.iter() {
                                li { class: "kit-pds__line", "data-change": "removed",
                                    span { class: "kit-pds__sign", "Removed" }
                                    code { "{line}" }
                                }
                            }
                            for line in added.iter() {
                                li { class: "kit-pds__line", "data-change": "added",
                                    span { class: "kit-pds__sign", "Added" }
                                    code { "{line}" }
                                }
                            }
                        }
                    }
                }
                if prior_count > 0 {
                    details { class: "kit-pds__prior",
                        summary { "{prior_label}" }
                        ol {
                            for step in prior_steps.iter() {
                                li { "{step}" }
                            }
                        }
                    }
                }
                if blocking {
                    p { class: "kit-pds__waiting", "Waits: {waiting_text}" }
                }
                if let Some(note) = note_text {
                    p { class: "kit-pds__note", "{note}" }
                }
                div { class: "kit-pds__actions",
                    if editing() {
                        button {
                            r#type: "button",
                            class: "kit-pds__save",
                            onclick: move |_| save_btn(save_id.clone(), DecisionOutcome::Edited, Some(draft())),
                            "Save and approve"
                        }
                        button {
                            r#type: "button",
                            class: "kit-pds__back",
                            onclick: move |_| {
                                editing.set(false);
                                draft.set(back_draft.clone());
                                focus_element(&back_focus);
                            },
                            "Back"
                        }
                    } else {
                        button {
                            r#type: "button",
                            id: "{id}-approve",
                            class: "kit-pds__approve",
                            onclick: move |_| approve_btn(approve_id.clone(), DecisionOutcome::Approved, None),
                            "Approve"
                        }
                        button {
                            r#type: "button",
                            id: "{id}-revise",
                            class: "kit-pds__revise",
                            onclick: move |_| {
                                draft.set(revise_seed.clone());
                                editing.set(true);
                                focus_element(&revise_click);
                            },
                            "Revise first"
                        }
                        button {
                            r#type: "button",
                            class: "kit-pds__reject",
                            onclick: move |_| reject_btn(reject_id.clone(), DecisionOutcome::Rejected, None),
                            "Reject"
                        }
                    }
                }
            }
        }
    });

    let peek_list: Vec<(String, usize, usize)> = open
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != top_index)
        .take(2)
        .enumerate()
        .map(|(depth, (index, peek_id))| (format!("{id}-peek-{peek_id}"), depth + 1, index))
        .collect();
    let history_rows: Vec<(String, String, &'static str, bool)> = decision_items
        .iter()
        .filter(|item| effective_status(item, &current) != RequestStatus::Open)
        .map(|item| {
            let status = effective_status(item, &current);
            (
                item.id.clone(),
                item.request_summary.clone(),
                status.label(),
                expanded().contains(&item.id),
            )
        })
        .collect();
    let subject = run_subject.clone();
    let blocking_text = format!("Paused: {blocked_steps} steps wait on {blocked_items} items");
    let open_text = format!("{open_total} waiting on you");
    let decided_text = format!("{decided_total} decided");
    let root_class = if root_class.is_empty() {
        "kit-pds".to_string()
    } else {
        format!("kit-pds {root_class}")
    };
    let stack_label = format!("{open_total} open decisions");
    let placeholder = if is_running {
        "Running"
    } else {
        "Nothing waiting"
    };
    let has_history = !history_rows.is_empty();
    let announcement = announce();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { id: "{id}", class: "{root_class}", aria_label: "{section_title}",
            header { class: "kit-pds__head",
                h2 { class: "kit-pds__title", "{section_title}" }
                if !subject.is_empty() {
                    span { class: "kit-pds__subject", "{subject}" }
                }
                div { class: "kit-pds__counters",
                    span { "{open_text}" }
                    span { "{decided_text}" }
                    span { class: "kit-pds__state", "data-state": word, "{word}" }
                }
            }
            p { class: "kit-pds__banner", role: "status", "aria-live": "polite",
                if blocked_items > 0 { "{blocking_text}" }
            }
            if let Some(card) = top_card {
                div { class: "kit-pds__stack", role: "group", aria_label: "{stack_label}",
                    for (peek_id, depth, target) in peek_list {
                        button {
                            key: "{peek_id}",
                            r#type: "button",
                            class: "kit-pds__peek",
                            tabindex: "-1",
                            aria_hidden: "true",
                            "data-depth": "{depth}",
                            onclick: move |_| rotation.set(target),
                        }
                    }
                    {card}
                }
            } else {
                p { class: "kit-pds__placeholder", "{placeholder}" }
            }
            if has_history {
                ol { class: "kit-pds__history", aria_label: "Decided",
                    for (row_id, row_summary, row_status, row_open) in history_rows {
                        li { key: "{row_id}", class: "kit-pds__row",
                            button {
                                r#type: "button",
                                class: "kit-pds__row-toggle",
                                "aria-expanded": if row_open { "true" } else { "false" },
                                onclick: {
                                    let toggle_id = row_id.clone();
                                    move |_| {
                                        expanded.with_mut(|list| {
                                            if let Some(at) = list.iter().position(|known| *known == toggle_id) {
                                                list.remove(at);
                                            } else {
                                                list.push(toggle_id.clone());
                                            }
                                        });
                                    }
                                },
                                span { class: "kit-pds__row-label", "{row_summary}" }
                                span { class: "kit-pds__row-status", "{row_status}" }
                            }
                        }
                    }
                }
            }
            p { class: "kit-pds__live", role: "status", "aria-live": "polite", "{announcement}" }
        }
    }
}
