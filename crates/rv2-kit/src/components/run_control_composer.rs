//! One input for working alongside an agent that is running or about to run: streamed answer, steering, queue and run controls.
//!
//! Design choice: `steer_notes` are placed in the answer at `at_char`, using char indices, so multi-byte text stays intact.
//! Design choice: the access menu is a radio group; the chosen level is the only checked item.
//! Design choice: Alt/Option+Enter queues while running. Enter steers while running and submits when idle.
//! Browser checks: the run area keeps a fixed height and scrolls; the caret pulses while running;
//! Escape halts only when the access menu is closed; the access control shows only its icon below narrow widths.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/run-control-composer.css");

fn focus_query(query: &str) {
    document::eval(&format!("document.querySelector({query:?})?.focus();"));
}
/// Queued messages shown before the tray scrolls.
pub const TRAY_VISIBLE: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RunState {
    #[default]
    Idle,
    Running,
    Stopped,
    Error,
    Done,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkMode {
    Chat,
    #[default]
    Act,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccessLevel {
    #[default]
    Ask,
    Edits,
    Full,
}

impl AccessLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ask => "Ask",
            Self::Edits => "Edits",
            Self::Full => "Full",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub agent_name: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SteerNote {
    pub at_char: usize,
    pub note_text: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct QueuedMessage {
    pub queue_id: String,
    pub message_text: String,
}

/// Splits `answer` at `at_char` (a char index), clamped to the answer length.
pub fn split_at_char(answer: &str, at_char: usize) -> (&str, &str) {
    let byte = answer
        .char_indices()
        .nth(at_char)
        .map(|(index, _)| index)
        .unwrap_or(answer.len());
    answer.split_at(byte)
}

/// Notes in the order they landed, so a later note never jumps before an earlier one at the same character.
pub fn ordered_notes(notes: &[SteerNote]) -> Vec<SteerNote> {
    let mut out = notes.to_vec();
    out.sort_by_key(|note| note.at_char);
    out
}

/// True when the context share is past the warning threshold.
pub fn context_warning(share: Option<f64>, threshold: f64) -> bool {
    share.is_some_and(|value| value > threshold)
}

/// Keys that mean "submit or steer" versus "queue": Alt/Option with Enter queues while running.
pub fn enter_action(running: bool, alt: bool) -> &'static str {
    match (running, alt) {
        (true, true) => "queue",
        (true, false) => "steer",
        (false, _) => "submit",
    }
}

/// The run composer: request, streamed answer, queue, input and controls.
#[component]
pub fn RunControlComposer(
    agent_identity: AgentIdentity,
    #[props(default)] request_text: Option<String>,
    #[props(default)] answer_text: Option<String>,
    #[props(default)] run_state: RunState,
    #[props(default)] steer_notes: Vec<SteerNote>,
    #[props(default)] queued_messages: Vec<QueuedMessage>,
    #[props(default)] work_mode: Option<WorkMode>,
    #[props(default)] initial_work_mode: WorkMode,
    #[props(default)] on_work_mode_change: Option<EventHandler<WorkMode>>,
    #[props(default)] access_level: Option<AccessLevel>,
    #[props(default)] initial_access_level: AccessLevel,
    #[props(default)] on_access_change: Option<EventHandler<AccessLevel>>,
    #[props(default)] context_share: Option<f64>,
    #[props(default = 0.8)] context_warn_at: f64,
    #[props(default)] elapsed_secs: Option<u64>,
    #[props(default)] on_submit: Option<EventHandler<String>>,
    #[props(default)] on_steer: Option<EventHandler<String>>,
    #[props(default)] on_queue: Option<EventHandler<String>>,
    #[props(default)] on_dequeue: Option<EventHandler<String>>,
    #[props(default)] on_stop: Option<EventHandler<()>>,
    #[props(default)] on_resume: Option<EventHandler<()>>,
    #[props(default)] on_retry: Option<EventHandler<()>>,
    #[props(default)] on_dismiss: Option<EventHandler<()>>,
    #[props(default, into)] input_hint: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-rcc-{}", next_id()));
    let input_id = format!("{id}-input");
    let mut draft = use_signal(String::new);
    let mut mode = use_signal(|| initial_work_mode);
    let mut access = use_signal(|| initial_access_level);
    let mut access_open = use_signal(|| false);

    let current_mode = work_mode.unwrap_or_else(|| mode());
    let current_access = access_level.unwrap_or_else(|| access());
    let running = run_state == RunState::Running;
    let chat = current_mode == WorkMode::Chat;
    let warn = context_warning(context_share, context_warn_at);
    let full = current_access == AccessLevel::Full;
    let share_text = context_share
        .map(|share| format!("{}%", (share * 100.0).round() as i64))
        .unwrap_or_default();
    let agent_name = agent_identity.agent_name.clone();
    let agent_key = agent_identity.agent_id.clone();
    let request_present = request_text.is_some();
    let request_body = request_text.clone().unwrap_or_default();
    let answer = answer_text.clone().unwrap_or_default();
    let notes = ordered_notes(&steer_notes);
    let answer_segments = render_answer(&answer, &notes);
    let result_line = match run_state {
        RunState::Done => elapsed_secs.map(|secs| format!("Finished in {secs}s")),
        RunState::Stopped => Some("Stopped".to_string()),
        RunState::Error => Some("Error".to_string()),
        _ => None,
    };
    let queue_count = queued_messages.len();
    let queue_head: Vec<QueuedMessage> =
        queued_messages.iter().take(TRAY_VISIBLE).cloned().collect();
    let input_label = if running {
        "Steer the agent"
    } else {
        "Message"
    };
    let hint = if input_hint.is_empty() {
        "Enter sends · Shift+Enter new line".to_string()
    } else {
        input_hint.clone()
    };
    let root_class = if root_class.is_empty() {
        "kit-rcc".to_string()
    } else {
        format!("kit-rcc {root_class}")
    };
    let state_word = match run_state {
        RunState::Idle => "Idle",
        RunState::Running => "Running",
        RunState::Stopped => "Stopped",
        RunState::Error => "Error",
        RunState::Done => "Done",
    };
    let access_label = format!("Access: {}", current_access.label());
    let access_menu_open = access_open();
    let access_menu_id = format!("{id}-access-menu");
    let access_toggle_id = format!("{id}-access-toggle");
    let stop_now = on_stop.clone();

    let submit = {
        let on_submit = on_submit.clone();
        let on_steer = on_steer.clone();
        let on_queue = on_queue.clone();
        move |queue: bool| {
            let text = draft().trim().to_string();
            if text.is_empty() {
                return;
            }
            draft.set(String::new());
            if running {
                let handler = if queue { &on_queue } else { &on_steer };
                if let Some(h) = handler {
                    h.call(text);
                }
            } else if let Some(h) = &on_submit {
                h.call(text);
            }
        }
    };
    let mut submit_key = submit.clone();
    let mut submit_click = submit.clone();
    let set_mode = {
        let handler = on_work_mode_change.clone();
        move |next: WorkMode| {
            mode.set(next);
            if let Some(h) = &handler {
                h.call(next);
            }
        }
    };
    let mut mode_left = set_mode.clone();
    let mut mode_right = set_mode.clone();
    let mut mode_chat = set_mode.clone();
    let mut mode_act = set_mode.clone();
    let set_access = {
        let handler = on_access_change.clone();
        move |next: AccessLevel| {
            access.set(next);
            access_open.set(false);
            if let Some(h) = &handler {
                h.call(next);
            }
        }
    };
    let mut access_pick_ask = set_access.clone();
    let mut access_pick_edits = set_access.clone();
    let mut access_pick_full = set_access.clone();
    let resume = on_resume.clone();
    let retry = on_retry.clone();
    let dismiss = on_dismiss.clone();
    let dequeue = on_dequeue.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            id: "{id}",
            class: "{root_class}",
            "data-state": state_word,
            "data-mode": if chat { "chat" } else { "act" },
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    if access_menu_open {
                        event.prevent_default();
                        access_open.set(false);
                    } else if running {
                        event.prevent_default();
                        if let Some(h) = &stop_now {
                            h.call(());
                        }
                    }
                }
            },
            header { class: "kit-rcc__head",
                span { class: "kit-rcc__agent", title: "{agent_name}", "data-agent": "{agent_key}", "{agent_name}" }
                span { class: "kit-rcc__state", role: "status", aria_live: "polite", "{state_word}" }
            }
            div { class: "kit-rcc__run", tabindex: "0", aria_label: "Run",
                if request_present {
                    div { class: "kit-rcc__request", "{request_body}" }
                } else {
                    p { class: "kit-rcc__placeholder", "No run yet" }
                }
                if !answer.is_empty() || running {
                    div { class: "kit-rcc__answer",
                        for node in answer_segments {
                            {node}
                        }
                        if running {
                            span { class: "kit-rcc__caret", aria_hidden: "true" }
                        }
                    }
                }
                if let Some(line) = result_line {
                    p { class: "kit-rcc__result", "{line}" }
                }
            }
            if queue_count > 0 {
                div { class: "kit-rcc__tray",
                    span { class: "kit-rcc__tray-count", "Queued {queue_count}" }
                    ul { class: "kit-rcc__queue",
                        for (position, item) in queue_head.into_iter().enumerate() {
                            li { key: "{item.queue_id}", class: "kit-rcc__queued",
                                if position == 0 {
                                    span { class: "kit-rcc__next", "Next" }
                                }
                                span { class: "kit-rcc__queued-text", "{item.message_text}" }
                                button {
                                    r#type: "button",
                                    class: "kit-rcc__remove",
                                    aria_label: "Remove queued message",
                                    onclick: {
                                        let queue_id = item.queue_id.clone();
                                        let handler = dequeue.clone();
                                        move |_| {
                                            if let Some(h) = &handler {
                                                h.call(queue_id.clone());
                                            }
                                        }
                                    },
                                    "Remove"
                                }
                            }
                        }
                    }
                }
            }
            label { class: "kit-rcc__input-label", r#for: "{input_id}", "{input_label}" }
            textarea {
                id: "{input_id}",
                class: "kit-rcc__input",
                rows: "2",
                value: "{draft}",
                placeholder: "{hint}",
                aria_describedby: "{id}-keys",
                oninput: move |event: FormEvent| draft.set(event.value()),
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Enter && !event.modifiers().shift() {
                        event.prevent_default();
                        let queue = event.modifiers().alt();
                        if running && queue {
                            submit_key(true);
                        } else {
                            submit_key(false);
                        }
                    }
                },
            }
            p { id: "{id}-keys", class: "kit-rcc__keys", "Enter steers or sends · Alt+Enter queues · Escape stops" }
            div { class: "kit-rcc__controls",
                span { class: "kit-rcc__mode-label", "Mode" }
                div { class: "kit-rcc__mode", role: "radiogroup", aria_label: "Mode",
                    onkeydown: move |event: KeyboardEvent| {
                        match event.key() {
                            Key::ArrowLeft => mode_left(WorkMode::Chat),
                            Key::ArrowRight => mode_right(WorkMode::Act),
                            _ => {}
                        }
                    },
                    button {
                        r#type: "button",
                        role: "radio",
                        class: "kit-rcc__mode-option",
                        "aria-checked": if chat { "true" } else { "false" },
                        onclick: move |_| mode_chat(WorkMode::Chat),
                        "Chat"
                    }
                    button {
                        r#type: "button",
                        role: "radio",
                        class: "kit-rcc__mode-option",
                        "aria-checked": if chat { "false" } else { "true" },
                        onclick: move |_| mode_act(WorkMode::Act),
                        "Act"
                    }
                }
                div { class: "kit-rcc__access", "data-level": current_access.label(), "data-warn": if full { "true" } else { "false" },
                    button {
                        id: "{access_toggle_id}",
                        r#type: "button",
                        class: "kit-rcc__access-toggle",
                        aria_label: "{access_label}",
                        disabled: chat,
                        "aria-haspopup": "menu",
                        "aria-expanded": if access_menu_open { "true" } else { "false" },
                        onclick: move |_| access_open.set(!access_open()),
                        span { class: "kit-rcc__access-text", "{current_access.label()}" }
                    }
                    if access_menu_open && !chat {
                        div { class: "kit-rcc__access-menu", role: "menu", id: "{access_menu_id}",
                            onmounted: {
                                let menu = access_menu_id.clone();
                                move |_: MountedEvent| focus_query(&format!("#{menu} [aria-checked=\"true\"]"))
                            },
                            onclick: {
                                let toggle = access_toggle_id.clone();
                                move |_| focus_query(&format!("#{toggle}"))
                            },
                            onkeydown: {
                                let menu = access_menu_id.clone();
                                let toggle = access_toggle_id.clone();
                                move |event: KeyboardEvent| match event.key() {
                                    Key::Escape => {
                                        event.prevent_default();
                                        access_open.set(false);
                                        focus_query(&format!("#{toggle}"));
                                    }
                                    Key::ArrowDown | Key::ArrowUp => {
                                        event.prevent_default();
                                        let step = if event.key() == Key::ArrowDown { 1 } else { -1 };
                                        document::eval(&format!(
                                            "const items = [...document.querySelectorAll('#{menu} [role=menuitemradio]')]; if (items.length) {{ const at = items.indexOf(document.activeElement); items[(at + {step} + items.length) % items.length].focus(); }}"
                                        ));
                                    }
                                    _ => {}
                                }
                            },
                            button {
                                r#type: "button",
                                role: "menuitemradio",
                                "aria-checked": if current_access == AccessLevel::Ask { "true" } else { "false" },
                                onclick: move |_| access_pick_ask(AccessLevel::Ask),
                                "Ask"
                            }
                            button {
                                r#type: "button",
                                role: "menuitemradio",
                                "aria-checked": if current_access == AccessLevel::Edits { "true" } else { "false" },
                                onclick: move |_| access_pick_edits(AccessLevel::Edits),
                                "Edits"
                            }
                            button {
                                r#type: "button",
                                role: "menuitemradio",
                                "aria-checked": if current_access == AccessLevel::Full { "true" } else { "false" },
                                onclick: move |_| access_pick_full(AccessLevel::Full),
                                "Full"
                            }
                        }
                    }
                }
                if context_share.is_some() {
                    span { class: "kit-rcc__context", "data-warn": if warn { "true" } else { "false" },
                        "aria-label": "Context used {share_text}",
                        span { class: "kit-rcc__ring", aria_hidden: "true" }
                        span { class: "kit-rcc__context-label", "Context" }
                        span { class: "kit-rcc__share", "{share_text}" }
                    }
                }
                div { class: "kit-rcc__actions",
                    if running {
                        button {
                            r#type: "button",
                            class: "kit-rcc__primary kit-rcc__stop",
                            onclick: {
                                let handler = on_stop.clone();
                                move |_| {
                                    if let Some(h) = &handler {
                                        h.call(());
                                    }
                                }
                            },
                            "Stop"
                        }
                    } else if matches!(run_state, RunState::Stopped | RunState::Error) {
                        button {
                            r#type: "button",
                            class: "kit-rcc__primary",
                            onclick: move |_| {
                                if let Some(h) = &resume {
                                    h.call(());
                                }
                            },
                            "Resume"
                        }
                        button {
                            r#type: "button",
                            class: "kit-rcc__secondary",
                            onclick: move |_| {
                                if let Some(h) = &retry {
                                    h.call(());
                                }
                            },
                            "Retry"
                        }
                        button {
                            r#type: "button",
                            class: "kit-rcc__secondary",
                            onclick: move |_| {
                                if let Some(h) = &dismiss {
                                    h.call(());
                                }
                            },
                            "Dismiss"
                        }
                    } else if run_state == RunState::Done {
                        button {
                            r#type: "button",
                            class: "kit-rcc__secondary",
                            onclick: move |_| {
                                if let Some(h) = &retry {
                                    h.call(());
                                }
                            },
                            "Retry"
                        }
                    } else {
                        button {
                            r#type: "button",
                            class: "kit-rcc__primary",
                            disabled: draft().trim().is_empty(),
                            onclick: move |_| submit_click(false),
                            "Submit"
                        }
                    }
                }
            }
        }
    }
}

/// Answer text with each steer note inserted at its character position.
fn render_answer(answer: &str, notes: &[SteerNote]) -> Vec<Element> {
    let mut out: Vec<Element> = Vec::new();
    let mut rest = answer;
    let mut consumed = 0_usize;
    for (index, note) in notes.iter().enumerate() {
        let target = note.at_char.saturating_sub(consumed);
        let (head, tail) = split_at_char(rest, target);
        let head = head.to_string();
        let key = format!("seg-{index}");
        let note_text = note.note_text.clone();
        let note_key = format!("note-{index}");
        out.push(rsx! { span { key: "{key}", "{head}" } });
        out.push(rsx! { span { key: "{note_key}", class: "kit-rcc__steer", "{note_text}" } });
        consumed += head.chars().count();
        rest = tail;
    }
    let tail = rest.to_string();
    let tail_key = format!("seg-tail-{}", notes.len());
    out.push(rsx! { span { key: "{tail_key}", "{tail}" } });
    out
}
