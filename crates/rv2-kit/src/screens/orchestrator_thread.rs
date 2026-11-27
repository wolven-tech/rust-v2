//! Orchestrator thread: one conversation with the orchestrating agent. The agent
//! replies in text, with small cards for tasks and decisions. Tool work folds into
//! one line per group. The first waiting decision docks above the composer. A send
//! made while offline waits in the thread and goes out once the connection returns.
//!
//! Design choice: a tool group is a native `details`, so it folds and expands from
//! the keyboard with no extra state. Its summary carries the total time.
//! Design choice: connection state comes from the host (`connection_state`). Without
//! one, the screen follows the browser's `online` and `offline` events.
//! Design choice: the away card shows when `clock_ms` is more than
//! `away_after_minutes` past `last_seen_at`. The host decides when the tab was
//! hidden.
//! Design choice: the Jump pill is driven by `new_below_count`, which the host
//! sets while the thread is scrolled up. It sits inside the scroll area, above the
//! dock, so it never covers the dock.
//! Design choice: digits 1 to 9 answer the docked decision only while focus is on
//! the dock, so typing a number in the composer types it. Arrow keys and Home and
//! End move the highlight between options.
//! Design choice: attachments are the host's list. The file picker reports each
//! chosen file as a `FileData`; the host uploads it and sets its state. Send carries
//! the ids of the attachments that finished uploading.
//! Design choice: voice notes record a duration on the clock. Audio capture and
//! transcription are the host's; `on_voice_clip` reports the length of the clip.
//! Design choice: a streaming answer reveals word by word only where the browser
//! allows motion; otherwise the text shows whole.
//! Not implemented: the reconnect countdown. A reconnecting state shows its text only.

use dioxus::html::FileData;
use dioxus::prelude::*;

use crate::internal::{
    KIT_CSS, focus_after_frame, next_id, prefers_motion_script, use_now_ms, use_pin_to_end,
};

const CSS: Asset = asset!("/assets/orchestrator-thread.css");

/// Time between revealed words while an answer streams, in milliseconds.
const WORD_MS: u32 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    Sent,
    Queued,
    Sending,
    Failed,
}

impl Delivery {
    fn word(self) -> &'static str {
        match self {
            Self::Sent => "",
            Self::Queued => "Waiting to send",
            Self::Sending => "Sending",
            Self::Failed => "Not sent",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowState {
    Running,
    Done,
    Failed,
    Stopped,
}

impl RowState {
    fn word(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolRow {
    pub id: String,
    pub entry_label: String,
    pub row_state: RowState,
    pub duration_ms: Option<u64>,
    pub error_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    Text(String),
    Tools(Vec<ToolRow>),
    Task {
        id: String,
        entry_title: String,
        status: String,
        done: u32,
        total: u32,
    },
    Decision,
    Artifacts(Vec<String>),
    Error {
        text: String,
        retry_allowed: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ThreadMessage {
    User {
        id: String,
        text: String,
        delivery: Delivery,
    },
    Agent {
        id: String,
        name: String,
        time_label: String,
        parts: Vec<Part>,
        is_streaming: bool,
        is_stopped: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionOption {
    pub id: String,
    pub entry_title: String,
    pub summary: Option<String>,
    pub is_recommended: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    pub id: String,
    pub question_text: String,
    pub task_name: String,
    pub option_list: Vec<DecisionOption>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    Online,
    Offline,
    Reconnecting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UploadState {
    /// Percent sent, 0 to 100.
    Uploading(u8),
    Done,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub upload_state: UploadState,
}

/// Whether the tab has been away long enough to show the away card.
pub fn is_away(now_ms: u64, last_seen_at: u64, away_after_minutes: u64) -> bool {
    now_ms.saturating_sub(last_seen_at) > away_after_minutes * 60_000
}

/// Option index for a digit key `1` to `9`, if it is in range.
pub fn option_for_key(key: &str, count: usize) -> Option<usize> {
    let digit = key.parse::<usize>().ok()?;
    (1..=9)
        .contains(&digit)
        .then(|| digit - 1)
        .filter(|i| *i < count)
}

/// Total time of a tool group, in seconds.
pub fn group_seconds(rows: &[ToolRow]) -> f64 {
    rows.iter().filter_map(|r| r.duration_ms).sum::<u64>() as f64 / 1000.0
}

/// Pager text for the decision dock.
pub fn pager_text(index: usize, total: usize) -> String {
    format!("{} of {}", index + 1, total)
}

/// Whole seconds until the host's next reconnect attempt, rounded up; 0 once due.
pub fn retry_seconds(now_ms: u64, retry_at_ms: u64) -> u64 {
    retry_at_ms.saturating_sub(now_ms).div_ceil(1000)
}

/// `m:ss` for a length in whole seconds.
pub fn clock_text(seconds: u64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// Number of words in a streamed answer.
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// The text up to the start of word `words + 1`: the first `words` words, with the
/// spacing after them. Words past the end return the whole text.
pub fn reveal_prefix(text: &str, words: usize) -> &str {
    let mut seen = 0;
    let mut in_word = false;
    for (index, ch) in text.char_indices() {
        if ch.is_whitespace() {
            in_word = false;
        } else if !in_word {
            if seen == words {
                return &text[..index];
            }
            seen += 1;
            in_word = true;
        }
    }
    text
}

/// Hands a send to the host, or holds it while the connection is down.
fn dispatch(
    online: bool,
    text: String,
    ids: Vec<String>,
    mut waiting: Signal<Vec<(String, Vec<String>)>>,
    on_send: Option<EventHandler<(String, Vec<String>)>>,
) {
    if !online {
        waiting.with_mut(|queue| queue.push((text, ids)));
    } else if let Some(handler) = on_send {
        handler.call((text, ids));
    }
}

/// An answer that reveals word by word while it streams.
#[component]
fn StreamedText(text: String, streaming: bool) -> Element {
    let mut shown = use_signal(|| 0usize);
    let mut motion = use_signal(|| false);
    let mut total = use_signal(|| 0usize);
    let words = count_words(&text);
    if *total.peek() != words {
        total.set(words);
    }
    use_hook(move || {
        spawn(async move {
            if let Ok(value) = document::eval(&format!("return {};", prefers_motion_script())).await
            {
                motion.set(value.as_bool().unwrap_or(false));
            }
            loop {
                let tick = format!(
                    "await new Promise((resolve) => setTimeout(resolve, {WORD_MS})); return true;"
                );
                if document::eval(&tick).await.is_err() {
                    break;
                }
                if *shown.peek() < *total.peek() {
                    shown += 1;
                }
            }
        })
    });
    let visible = if streaming && motion() {
        reveal_prefix(&text, shown())
    } else {
        text.as_str()
    };
    rsx! {
        p { class: "kit-ot__text", "{visible}" }
    }
}

/// The thread as messages, with the first waiting decision docked above the composer.
#[component]
pub fn OrchestratorThread(
    #[props(default)] message_list: Vec<ThreadMessage>,
    #[props(default)] decision_queue: Vec<Decision>,
    #[props(default)] attachment_list: Vec<Attachment>,
    #[props(default)] connection_state: Option<Connection>,
    #[props(default)] last_seen_at: Option<u64>,
    #[props(default = 30)] away_after_minutes: u64,
    #[props(default)] running_count: usize,
    #[props(default)] needs_count: usize,
    #[props(default)] new_below_count: usize,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] retry_at_ms: Option<u64>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_send: Option<EventHandler<(String, Vec<String>)>>,
    #[props(default)] on_stop: Option<EventHandler<()>>,
    #[props(default)] on_retry: Option<EventHandler<String>>,
    #[props(default)] on_answer: Option<EventHandler<(String, String)>>,
    #[props(default)] on_later: Option<EventHandler<String>>,
    #[props(default)] on_open_task: Option<EventHandler<String>>,
    #[props(default)] on_open_inbox: Option<EventHandler<()>>,
    #[props(default)] on_mark_read: Option<EventHandler<()>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] on_attach: Option<EventHandler<FileData>>,
    #[props(default)] on_retry_attachment: Option<EventHandler<String>>,
    #[props(default)] on_remove_attachment: Option<EventHandler<String>>,
    #[props(default)] on_voice_clip: Option<EventHandler<u64>>,
    #[props(default)] on_toggle_panel: Option<EventHandler<()>>,
    #[props(default)] approval_chip: Option<String>,
) -> Element {
    let uid = use_hook(next_id);
    let now = use_now_ms(clock_ms, 1000);
    let mut draft = use_signal(String::new);
    let mut dock_index = use_signal(|| 0usize);
    let mut highlight = use_signal(|| 0usize);
    let mut away_dismissed = use_signal(|| false);
    let mut recording = use_signal(|| None::<u64>);
    let mut waiting = use_signal(Vec::<(String, Vec<String>)>::new);
    let mut browser_online = use_signal(|| true);
    let mut online_seen = use_signal(|| true);
    let scroll_id = format!("kit-ot-{uid}-scroll");
    let file_id = format!("kit-ot-{uid}-file");
    let input_id = format!("kit-ot-{uid}-input");
    let root_id = format!("kit-ot-{uid}-root");
    let (hook_root, hook_input, hook_scroll) =
        (root_id.clone(), input_id.clone(), scroll_id.clone());

    // Stays at the bottom while the reader is there; a one-time mount scroll can run before layout.
    use_pin_to_end(scroll_id.clone());

    use_hook(move || {
        document::eval(&format!(
            "const onKey = (e) => {{\
                 const root = document.getElementById({hook_root:?});\
                 if (!root || e.defaultPrevented || e.ctrlKey || e.metaKey || e.altKey) return;\
                 const a = document.activeElement;\
                 if (a && (a.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(a.tagName))) return;\
                 if (a && a.closest && a.closest('.kit-ot__dock')) return;\
                 const focusedHere = a !== null && a !== document.body && root.contains(a);\
                 const firstOnPage = document.querySelector('.kit-ot') === root;\
                 if (!(focusedHere || ((a === null || a === document.body) && firstOnPage))) return;\
                 if (e.key === '/') {{\
                     e.preventDefault();\
                     document.getElementById({hook_input:?})?.focus();\
                 }} else if (e.key === 'End') {{\
                     e.preventDefault();\
                     const s = document.getElementById({hook_scroll:?});\
                     if (s) s.scrollTop = s.scrollHeight;\
                 }}\
             }};\
             document.addEventListener('keydown', onKey);\
             (window.__kitOt ||= new Map()).set({uid}, () => document.removeEventListener('keydown', onKey));"
        ));
    });
    use_drop(move || {
        document::eval(&format!(
            "window.__kitOt?.get({uid})?.(); window.__kitOt?.delete({uid});"
        ));
    });

    use_hook(move || {
        spawn(async move {
            let mut channel = document::eval(&format!(
                "const report = () => dioxus.send(navigator.onLine);\
                 const off = () => {{\
                     window.removeEventListener('online', report);\
                     window.removeEventListener('offline', report);\
                 }};\
                 window.addEventListener('online', report);\
                 window.addEventListener('offline', report);\
                 (window.__kitOnline ||= new Map()).set({uid}, off);\
                 report();\
                 await new Promise(() => {{}});"
            ));
            while let Ok(online) = channel.recv::<bool>().await {
                browser_online.set(online);
            }
        })
    });
    use_drop(move || {
        document::eval(&format!(
            "window.__kitOnline?.get({uid})?.(); window.__kitOnline?.delete({uid});"
        ));
    });

    let connection = connection_state.unwrap_or(if browser_online() {
        Connection::Online
    } else {
        Connection::Offline
    });
    let online = connection == Connection::Online;
    if *online_seen.peek() != online {
        online_seen.set(online);
    }
    use_effect(move || {
        if online_seen() {
            let queued = waiting.with_mut(std::mem::take);
            for (text, ids) in queued {
                if let Some(handler) = on_send {
                    handler.call((text, ids));
                }
            }
        }
    });

    let streaming = message_list.iter().any(|m| {
        matches!(
            m,
            ThreadMessage::Agent {
                is_streaming: true,
                ..
            }
        )
    });
    let tool_running = message_list.iter().any(|m| {
        match m {
        ThreadMessage::Agent { parts, .. } => parts.iter().any(|p| {
            matches!(p, Part::Tools(rows) if rows.iter().any(|r| r.row_state == RowState::Running))
        }),
        ThreadMessage::User { .. } => false,
    }
    });
    let away = last_seen_at.is_some_and(|seen| is_away(now, seen, away_after_minutes))
        && !away_dismissed();
    let dock_count = decision_queue.len();
    let dock_at = dock_index().min(dock_count.saturating_sub(1));
    let docked = decision_queue.get(dock_at).cloned();
    let dock_options: Vec<DecisionOption> = docked
        .as_ref()
        .map_or_else(Vec::new, |d| d.option_list.clone());
    let docked_key_count = dock_options.len();
    let highlight_at = highlight().min(docked_key_count.saturating_sub(1));
    let dock_decision_id = docked.as_ref().map_or_else(String::new, |d| d.id.clone());
    let banner = match connection {
        Connection::Online => None,
        Connection::Offline => Some("Offline. New messages wait until the connection returns."),
        Connection::Reconnecting => Some("Reconnecting…"),
    };
    let reconnect_countdown = match (connection, retry_at_ms) {
        (Connection::Reconnecting, Some(at)) => Some(match retry_seconds(now, at) {
            0 => "retrying now".to_string(),
            secs => format!("retry in {secs} s"),
        }),
        _ => None,
    };
    let running_text = format!("{running_count} running");
    let needs_text = format!("{needs_count} waiting on you");
    let jump_label = format!("Jump to latest, {new_below_count} new lines. Press End to jump.");
    let away_line =
        format!("While you were away: {running_count} running, {needs_count} need you.");
    let streaming_text = streaming.to_string();
    let send_idle = draft().trim().is_empty();
    let ready_ids: Vec<String> = attachment_list
        .iter()
        .filter(|a| a.upload_state == UploadState::Done)
        .map(|a| a.id.clone())
        .collect();
    let recording_secs = recording().map(|start| now.saturating_sub(start) / 1000);
    let queued_sends = waiting()
        .into_iter()
        .map(|(text, _)| text)
        .collect::<Vec<_>>();
    let form_ids = ready_ids.clone();
    let key_ids = ready_ids.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{root_id}",
            class: "kit-ot",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": (is_loading || streaming).then_some("true"),
            header { class: "kit-ot__head",
                div { class: "kit-ot__counts kit-ot__col",
                    button {
                        r#type: "button",
                        class: "kit-ot__count-btn",
                        onclick: move |_| {
                            if let Some(handler) = on_open_inbox {
                                handler.call(());
                            }
                        },
                        span { class: "kit-ot__dot kit-ot__dot--ok", "aria-hidden": "true" }
                        "{running_text}"
                    }
                    button {
                        r#type: "button",
                        class: "kit-ot__count-btn",
                        onclick: move |_| {
                            if let Some(handler) = on_open_inbox {
                                handler.call(());
                            }
                        },
                        span { class: "kit-ot__dot kit-ot__dot--warn", "aria-hidden": "true" }
                        "{needs_text}"
                    }
                }
                button {
                    r#type: "button",
                    class: "kit-ot__panel",
                    "aria-label": "Toggle panel",
                    onclick: move |_| {
                        if let Some(handler) = on_toggle_panel {
                            handler.call(());
                        }
                    },
                    svg { class: "kit-ot__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", "aria-hidden": "true",
                        rect { x: "2.25", y: "3", width: "11.5", height: "10", rx: "1.5" }
                        path { d: "M9.75 3v10" }
                    }
                }
            }

            if let Some(text) = banner {
                div { class: "kit-ot__banner", role: "status", "aria-live": "polite",
                    "{text}"
                    if let Some(countdown) = reconnect_countdown.clone() {
                        span { class: "kit-ot__meta", "aria-hidden": "true", " · {countdown}" }
                    }
                }
            }

            div { class: "kit-ot__scroll kit-ot__col", id: "{scroll_id}",
                if is_loading {
                    div { class: "kit-ot__ph kit-ot__ph--agent", "aria-hidden": "true" }
                    div { class: "kit-ot__ph kit-ot__ph--user", "aria-hidden": "true" }
                    div { class: "kit-ot__ph kit-ot__ph--agent", "aria-hidden": "true" }
                } else if is_empty || message_list.is_empty() {
                    div { class: "kit-ot__empty",
                        p { "Start a chat with your orchestrator. It breaks the work into tasks and asks when it needs you." }
                        button {
                            r#type: "button",
                            class: "kit-ot__primary",
                            onclick: move |_| {
                                if let Some(handler) = on_navigate {
                                    handler.call("New chat".to_string());
                                }
                            },
                            "Start a chat"
                        }
                    }
                } else {
                    ol { class: "kit-ot__log", "aria-label": "Conversation", "aria-busy": "{streaming_text}",
                        for message in message_list.iter().cloned() {
                            {
                                match message {
                                    ThreadMessage::User { id, text, delivery } => {
                                        let note = delivery.word();
                                        let failed = delivery == Delivery::Failed;
                                        let retry_id = id.clone();
                                        rsx! {
                                            li { key: "{id}", class: "kit-ot__msg kit-ot__msg--user",
                                                p { class: "kit-ot__bubble", "{text}" }
                                                if !note.is_empty() {
                                                    span { class: "kit-ot__meta", "{note}" }
                                                }
                                                if failed {
                                                    button {
                                                        r#type: "button",
                                                        class: "kit-ot__link",
                                                        onclick: move |_| {
                                                            if let Some(handler) = on_retry {
                                                                handler.call(retry_id.clone());
                                                            }
                                                        },
                                                        "Retry"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    ThreadMessage::Agent { id, name, time_label, parts, is_streaming, is_stopped } => {
                                        let caret = if is_streaming { "▍" } else { "" };
                                        let stopped_note = if is_stopped { "Stopped." } else { "" };
                                        let message_key = id.clone();
                                        let mark = name.chars().next().map(|c| c.to_string()).unwrap_or_default();
                                        rsx! {
                                            li { key: "{message_key}", class: "kit-ot__msg kit-ot__msg--agent", "data-streaming": "{is_streaming}",
                                                header { class: "kit-ot__agent-head",
                                                    span { class: "kit-ot__mark", "aria-hidden": "true", "{mark}" }
                                                    span { class: "kit-ot__agent-name", "{name}" }
                                                    span { class: "kit-ot__meta", "{time_label}" }
                                                }
                                                div { class: "kit-ot__agent-body",
                                                    for (index, part) in parts.iter().cloned().enumerate() {
                                                        {
                                                            let part_key = format!("{id}-{index}");
                                                            match part {
                                                                Part::Text(text) => {
                                                                    if is_streaming {
                                                                        rsx! { StreamedText { key: "{part_key}", text: text, streaming: true } }
                                                                    } else {
                                                                        rsx! { p { key: "{part_key}", class: "kit-ot__text", "{text}" } }
                                                                    }
                                                                }
                                                                Part::Tools(rows) => {
                                                                    let seconds = group_seconds(&rows);
                                                                    let count = rows.len();
                                                                    let running = rows.iter().any(|r| r.row_state == RowState::Running);
                                                                    let summary = if running {
                                                                        format!("Running {count} tools in parallel")
                                                                    } else {
                                                                        format!("Ran {count} tools")
                                                                    };
                                                                    let total = format!("{seconds:.1}s");
                                                                    rsx! {
                                                                        details { key: "{part_key}", class: "kit-ot__tools", open: running,
                                                                            summary { class: "kit-ot__tools-head",
                                                                                if running {
                                                                                    span { class: "kit-ot__spin", "aria-hidden": "true" }
                                                                                } else {
                                                                                    svg { class: "kit-ot__glyph kit-ot__glyph--ok", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                                                                                        path { d: "M3.25 8.5 6.5 11.75 12.75 4.75" }
                                                                                    }
                                                                                }
                                                                                span { class: "kit-ot__tools-label", "{summary}" }
                                                                                if !running {
                                                                                    span { class: "kit-ot__duration", "{total}" }
                                                                                }
                                                                            }
                                                                            ul { class: "kit-ot__tool-rows",
                                                                                for row in rows.iter().cloned() {
                                                                                    {
                                                                                        let row_key = row.id.clone();
                                                                                        let row_label = row.entry_label.clone();
                                                                                        let row_word = row.row_state.word();
                                                                                        let row_error = row.error_text.clone().unwrap_or_default();
                                                                                        let row_time = row.duration_ms.map(|ms| format!("{:.1}s", ms as f64 / 1000.0)).unwrap_or_default();
                                                                                        let row_state = row.row_state;
                                                                                        rsx! {
                                                                                            li { key: "{row_key}", class: "kit-ot__tool-row", "data-state": "{row_word}",
                                                                                                span { class: "kit-ot__tool-status", "aria-hidden": "true",
                                                                                                    {
                                                                                                        match row_state {
                                                                                                            RowState::Running => rsx! { span { class: "kit-ot__spin" } },
                                                                                                            RowState::Done => rsx! {
                                                                                                                svg { class: "kit-ot__glyph kit-ot__glyph--ok", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round", stroke_linejoin: "round",
                                                                                                                    path { d: "M3.25 8.5 6.5 11.75 12.75 4.75" }
                                                                                                                }
                                                                                                            },
                                                                                                            RowState::Failed => rsx! {
                                                                                                                svg { class: "kit-ot__glyph kit-ot__glyph--fault", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round",
                                                                                                                    path { d: "M4 4l8 8M12 4l-8 8" }
                                                                                                                }
                                                                                                            },
                                                                                                            RowState::Stopped => rsx! {
                                                                                                                svg { class: "kit-ot__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round",
                                                                                                                    path { d: "M4 8h8" }
                                                                                                                }
                                                                                                            },
                                                                                                        }
                                                                                                    }
                                                                                                }
                                                                                                svg { class: "kit-ot__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                                                                                                    path { d: "M10.5 2.75a3 3 0 0 0-3.9 3.9L2.9 10.35a1.4 1.4 0 0 0 2 2l3.7-3.7a3 3 0 0 0 3.9-3.9l-1.6 1.6-1.6-.4-.4-1.6z" }
                                                                                                }
                                                                                                span { class: "kit-ot__tool-label", "{row_label}" }
                                                                                                span { class: "kit-sr", "{row_word}" }
                                                                                                if !row_error.is_empty() {
                                                                                                    span { class: "kit-ot__error", "{row_error}" }
                                                                                                }
                                                                                                span { class: "kit-ot__duration", "{row_time}" }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                Part::Task { id: task_id, entry_title, status, done, total } => {
                                                                    let progress = format!("{done} of {total} steps");
                                                                    let meta = format!("{status} · {progress}");
                                                                    let open_id = task_id.clone();
                                                                    rsx! {
                                                                        div { key: "{part_key}", class: "kit-ot__task",
                                                                            span { class: "kit-ot__task-title", "{entry_title}" }
                                                                            span { class: "kit-ot__meta", "{meta}" }
                                                                            button {
                                                                                r#type: "button",
                                                                                class: "kit-ot__link",
                                                                                onclick: move |_| {
                                                                                    if let Some(handler) = on_open_task {
                                                                                        handler.call(open_id.clone());
                                                                                    }
                                                                                },
                                                                                "Open"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                Part::Decision => rsx! {
                                                                    p { key: "{part_key}", class: "kit-ot__meta", "A decision is waiting above the composer." }
                                                                },
                                                                Part::Artifacts(names) => rsx! {
                                                                    ul { key: "{part_key}", class: "kit-ot__chips",
                                                                        for name in names.iter().cloned() {
                                                                            li { key: "{name}", class: "kit-ot__chip", "{name}" }
                                                                        }
                                                                    }
                                                                },
                                                                Part::Error { text, retry_allowed } => {
                                                                    let retry_key = message_key.clone();
                                                                    rsx! {
                                                                        p { key: "{part_key}", class: "kit-ot__error", role: "alert",
                                                                            "{text} "
                                                                            if retry_allowed {
                                                                                button {
                                                                                    r#type: "button",
                                                                                    class: "kit-ot__link",
                                                                                    onclick: move |_| {
                                                                                        if let Some(handler) = on_retry {
                                                                                            handler.call(retry_key.clone());
                                                                                        }
                                                                                    },
                                                                                    "Retry"
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                },
                                                            }
                                                        }
                                                    }
                                                    if is_streaming {
                                                        span { class: "kit-ot__caret", "aria-hidden": "true", "{caret}" }
                                                    }
                                                    if is_stopped {
                                                        span { class: "kit-ot__meta", "{stopped_note}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        for (index, text) in queued_sends.iter().cloned().enumerate() {
                            li { key: "queued-{index}", class: "kit-ot__msg kit-ot__msg--user",
                                p { class: "kit-ot__bubble", "{text}" }
                                span { class: "kit-ot__meta", "Waiting to send" }
                            }
                        }
                    }
                }

                if away {
                    section { class: "kit-ot__away", "aria-label": "While you were away",
                        p { "{away_line}" }
                        div { class: "kit-ot__away-actions",
                            button {
                                r#type: "button",
                                class: "kit-ot__primary",
                                onclick: move |_| {
                                    if let Some(handler) = on_open_inbox {
                                        handler.call(());
                                    }
                                },
                                "Open Inbox"
                            }
                            button {
                                r#type: "button",
                                class: "kit-ot__ghost",
                                onclick: move |_| {
                                    away_dismissed.set(true);
                                    if let Some(handler) = on_mark_read {
                                        handler.call(());
                                    }
                                },
                                "Mark as read"
                            }
                        }
                    }
                }

                if new_below_count > 0 {
                    button {
                        r#type: "button",
                        class: "kit-ot__jump",
                        "aria-label": "{jump_label}",
                        onclick: {
                            let jump_id = scroll_id.clone();
                            move |_| {
                                document::eval(&format!(
                                    "const el = document.getElementById({jump_id:?}); if (el) el.scrollTop = el.scrollHeight;"
                                ));
                            }
                        },
                        "{new_below_count} new"
                    }
                }
            }

            if let Some(decision) = docked.clone() {
                {
                    let question = decision.question_text.clone();
                    let task_name = decision.task_name.clone();
                    let dock_label = format!("Decision for {task_name}");
                    let pager = pager_text(dock_at, dock_count);
                    let has_pager = dock_count > 1;
                    let next_index = (dock_at + 1) % dock_count.max(1);
                    let later_id = dock_decision_id.clone();
                    let answer_id = dock_decision_id.clone();
                    let option_base = format!("kit-ot-{uid}-opt");
                    let dock_answer = dock_options.clone();
                    rsx! {
                        section {
                            class: "kit-ot__dock kit-ot__col",
                            role: "group",
                            "aria-label": "{dock_label}",
                            onkeydown: move |event: KeyboardEvent| {
                                let count = docked_key_count;
                                if count == 0 {
                                    if event.key() == Key::Escape {
                                        if let Some(handler) = on_later {
                                            handler.call(later_id.clone());
                                        }
                                    }
                                    return;
                                }
                                let current = highlight().min(count - 1);
                                let next = match event.key() {
                                    Key::Escape => {
                                        if let Some(handler) = on_later {
                                            handler.call(later_id.clone());
                                        }
                                        return;
                                    }
                                    Key::ArrowDown | Key::ArrowRight => (current + 1) % count,
                                    Key::ArrowUp | Key::ArrowLeft => (current + count - 1) % count,
                                    Key::Home => 0,
                                    Key::End => count - 1,
                                    Key::Character(digit) => {
                                        if let Some(index) = option_for_key(&digit, count) {
                                            let choice = dock_answer.get(index).map(|o| o.id.clone());
                                            if let (Some(choice), Some(handler)) = (choice, on_answer) {
                                                handler.call((answer_id.clone(), choice));
                                            }
                                        }
                                        return;
                                    }
                                    _ => return,
                                };
                                event.prevent_default();
                                highlight.set(next);
                                focus_after_frame(&format!("{option_base}-{next}"));
                            },
                            p { class: "kit-ot__question", "{question}" }
                            ul { class: "kit-ot__options", role: "listbox", "aria-label": "Options",
                                for (index, option) in decision.option_list.iter().cloned().enumerate() {
                                    {
                                        let option_key = option.id.clone();
                                        let option_title = option.entry_title.clone();
                                        let option_summary = option.summary.clone().unwrap_or_default();
                                        let option_label = format!("{}. {}", index + 1, option_title);
                                        let is_highlight = index == highlight_at;
                                        let selected = is_highlight.to_string();
                                        let tab_stop = if is_highlight { "0" } else { "-1" };
                                        let option_dom = format!("kit-ot-{uid}-opt-{index}");
                                        let choice_decision = dock_decision_id.clone();
                                        let choice_id = option.id.clone();
                                        let recommended = option.is_recommended;
                                        rsx! {
                                            li { key: "{option_key}",
                                                button {
                                                    id: "{option_dom}",
                                                    r#type: "button",
                                                    class: "kit-ot__option",
                                                    role: "option",
                                                    tabindex: "{tab_stop}",
                                                    "aria-selected": "{selected}",
                                                    "aria-label": "{option_label}",
                                                    onclick: move |_| {
                                                        if let Some(handler) = on_answer {
                                                            handler.call((choice_decision.clone(), choice_id.clone()));
                                                        }
                                                    },
                                                    span { class: "kit-ot__option-title", "{option_title}" }
                                                    if !option_summary.is_empty() {
                                                        span { class: "kit-ot__meta", "{option_summary}" }
                                                    }
                                                    if recommended {
                                                        span { class: "kit-ot__meta", "Recommended" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "kit-ot__dock-foot",
                                if has_pager {
                                    span { class: "kit-ot__meta", "{pager}" }
                                    button {
                                        r#type: "button",
                                        class: "kit-ot__link",
                                        onclick: move |_| dock_index.set(next_index),
                                        "Next"
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "kit-ot__link",
                                    onclick: {
                                        let later_click = dock_decision_id.clone();
                                        move |_| {
                                            if let Some(handler) = on_later {
                                                handler.call(later_click.clone());
                                            }
                                        }
                                    },
                                    "Later"
                                }
                            }
                        }
                    }
                }
            }

            if tool_running {
                div { class: "kit-ot__stop-row kit-ot__col",
                    button {
                        r#type: "button",
                        class: "kit-ot__stop",
                        onclick: move |_| {
                            if let Some(handler) = on_stop {
                                handler.call(());
                            }
                        },
                        svg { class: "kit-ot__stop-square", view_box: "0 0 10 10", "aria-hidden": "true",
                            rect { x: "0", y: "0", width: "10", height: "10", rx: "2", fill: "currentColor" }
                        }
                        "Stop"
                    }
                }
            }

            form {
                class: "kit-ot__composer kit-ot__col",
                "aria-label": "Message",
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Escape && recording().is_some() {
                        recording.set(None);
                    }
                },
                onsubmit: move |event: FormEvent| {
                    event.prevent_default();
                    let text = draft().trim().to_string();
                    if text.is_empty() {
                        return;
                    }
                    dispatch(online, text, form_ids.clone(), waiting, on_send);
                    draft.set(String::new());
                },
                if !attachment_list.is_empty() {
                    ul { class: "kit-ot__attachments", "aria-label": "Attachments",
                        for attachment in attachment_list.iter().cloned() {
                            {
                                let att_key = attachment.id.clone();
                                let att_name = attachment.name.clone();
                                let remove_label = format!("Remove {att_name}");
                                let retry_id = attachment.id.clone();
                                let remove_id = attachment.id.clone();
                                let state_key = match attachment.upload_state {
                                    UploadState::Uploading(_) => "uploading",
                                    UploadState::Done => "done",
                                    UploadState::Failed => "failed",
                                };
                                rsx! {
                                    li { key: "{att_key}", class: "kit-ot__attachment", "data-state": "{state_key}",
                                        span { class: "kit-ot__attach-name", "{att_name}" }
                                        {
                                            match attachment.upload_state {
                                                UploadState::Uploading(percent) => {
                                                    let pct = percent.to_string();
                                                    rsx! {
                                                        progress { class: "kit-ot__progress", max: "100", value: "{pct}", "aria-label": "Uploading {att_name}" }
                                                        span { class: "kit-ot__meta", "Uploading {pct}%" }
                                                    }
                                                }
                                                UploadState::Done => rsx! {
                                                    span { class: "kit-ot__meta", "Attached" }
                                                },
                                                UploadState::Failed => rsx! {
                                                    span { class: "kit-ot__error", role: "alert", "Upload failed" }
                                                    button {
                                                        r#type: "button",
                                                        class: "kit-ot__link",
                                                        onclick: move |_| {
                                                            if let Some(handler) = on_retry_attachment {
                                                                handler.call(retry_id.clone());
                                                            }
                                                        },
                                                        "Retry"
                                                    }
                                                },
                                            }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "kit-ot__link",
                                            "aria-label": "{remove_label}",
                                            onclick: move |_| {
                                                if let Some(handler) = on_remove_attachment {
                                                    handler.call(remove_id.clone());
                                                }
                                            },
                                            "Remove"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(secs) = recording_secs {
                    div { class: "kit-ot__recording", role: "group", "aria-label": "Voice note",
                        span { class: "kit-ot__meta", "Recording voice note · {clock_text(secs)}" }
                        button {
                            r#type: "button",
                            class: "kit-ot__primary",
                            "aria-label": "Stop recording",
                            onclick: move |_| {
                                recording.set(None);
                                if let Some(handler) = on_voice_clip {
                                    handler.call(secs);
                                }
                            },
                            "Stop"
                        }
                        button {
                            r#type: "button",
                            class: "kit-ot__ghost",
                            "aria-label": "Discard voice note",
                            onclick: move |_| recording.set(None),
                            "Discard"
                        }
                    }
                }
                textarea {
                    id: "{input_id}",
                    class: "kit-ot__input",
                    rows: "1",
                    placeholder: "Message the orchestrator",
                    "aria-label": "Message",
                    value: "{draft}",
                    oninput: move |event| draft.set(event.value()),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape && streaming {
                            if let Some(handler) = on_stop {
                                handler.call(());
                            }
                        } else if event.key() == Key::Enter && !event.modifiers().shift() {
                            event.prevent_default();
                            let text = draft().trim().to_string();
                            if !text.is_empty() {
                                dispatch(online, text, key_ids.clone(), waiting, on_send);
                                draft.set(String::new());
                            }
                        }
                    },
                }
                div { class: "kit-ot__composer-foot",
                    button {
                        r#type: "button",
                        class: "kit-ot__icon",
                        "aria-label": "Attach file",
                        onclick: {
                            let click_id = file_id.clone();
                            move |_| {
                                document::eval(&format!("document.getElementById({click_id:?})?.click();"));
                            }
                        },
                        svg { class: "kit-ot__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round", "aria-hidden": "true",
                            path { d: "M8 3v10M3 8h10" }
                        }
                    }
                    if let Some(chip) = approval_chip.clone() {
                        span { class: "kit-ot__approval",
                            svg { class: "kit-ot__glyph kit-ot__glyph--sm", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linejoin: "round", "aria-hidden": "true",
                                path { d: "M4 7.5V4a1 1 0 0 1 2 0v3.5M6 6.5V2.75a1 1 0 0 1 2 0V6.5M8 6.5V3a1 1 0 0 1 2 0v3.5M10 6.5V4.25a1 1 0 0 1 2 0V9c0 2.5-1.6 4.25-3.9 4.25H7.5c-1.2 0-2-.5-2.8-1.5L2.6 9.3a1 1 0 0 1 1.5-1.3L6 9.2" }
                            }
                            span { "{chip}" }
                        }
                    }
                    input {
                        id: "{file_id}",
                        r#type: "file",
                        multiple: true,
                        hidden: true,
                        onchange: move |event: FormEvent| {
                            if let Some(handler) = on_attach {
                                for file in event.files() {
                                    handler.call(file);
                                }
                            }
                        },
                    }
                    span { class: "kit-ot__spacer" }
                    button {
                        r#type: "button",
                        class: "kit-ot__icon",
                        "aria-label": "Voice note",
                        onclick: move |_| {
                            if recording().is_none() {
                                recording.set(Some(now));
                            }
                        },
                        svg { class: "kit-ot__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", "aria-hidden": "true",
                            rect { x: "5.5", y: "2", width: "5", height: "8", rx: "2.5" }
                            path { d: "M3.5 7.5a4.5 4.5 0 0 0 9 0M8 12v2" }
                        }
                    }
                    button {
                        r#type: "submit",
                        class: "kit-ot__send",
                        "data-idle": "{send_idle}",
                        "aria-label": "Send",
                        svg { class: "kit-ot__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                            path { d: "M8 13V3.5M3.9 7.6 8 3.5l4.1 4.1" }
                        }
                    }
                }
            }

            p { class: "kit-sr", role: "status", "aria-live": "polite", "{running_text}, {needs_text}" }
        }
    }
}
