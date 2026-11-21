//! Tool call card: one tool call of an AI agent. A header with the tool's icon, its
//! name in mono, a summary and the status: a spinner with a live timer, a check with
//! the duration, or "Failed". Click it open to see the arguments and the result, each
//! with a copy button; long blocks are capped with a fade and "Show all". Errors show
//! the message and an optional Retry button. `ToolCallGroup` folds several calls into
//! one line: "Used 3 tools · 2.4s".
//!
//! Port losses:
//! - Arguments and results are text. The React original pretty-prints objects and
//!   parses JSON strings; this crate has no JSON parser, so text is shown as given
//!   and only colored when it starts with `{` or `[`.
//! - `ToolCallGroup` takes its calls as a `calls` list. The React original reads the
//!   props of its child cards.
//! - The auto-open on error reacts one render late, because it is set in an effect.
//! - The size of a result block is measured when its text changes. Resizing the
//!   window does not re-measure it.
//! - Copy shows "Copied" whether or not the clipboard accepted the text.

use dioxus::prelude::*;

use crate::internal::{class_list, copy_to_clipboard, next_id, sleep};
use crate::reaction_bar::exact_count;

const CSS: Asset = asset!("/assets/tool-call-card.css");

const OPEN_MAX: f64 = 640.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolCallStatus {
    Running,
    Success,
    Error,
}

impl ToolCallStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Error => "error",
        }
    }
}

/// `0.4s`, `12.3s`, `2m 05s`, `3h 12m`, `2d 4h`. Live values always show tenths
/// below a minute; a finished value under a second shows whole milliseconds.
pub fn format_ms(ms: u64, live: bool) -> String {
    if !live && ms < 1000 {
        return format!("{}ms", ms.max(1));
    }
    if ms < 60_000 {
        return format!("{:.1}s", (ms / 100) as f64 / 10.0);
    }
    let s = ms / 1000;
    if s < 3600 {
        return format!("{}m {:02}s", s / 60, s % 60);
    }
    let h = s / 3600;
    if h < 24 {
        format!("{h}h {:02}m", (s / 60) % 60)
    } else {
        format!("{}d {}h", h / 24, h % 24)
    }
}

/// Header title of a group: the running label while a call runs, else the label
/// or `Used 3 tools`.
pub fn group_title(
    count: usize,
    running: bool,
    label: Option<&str>,
    running_label: &str,
) -> String {
    if running {
        return running_label.to_string();
    }
    match label {
        Some(text) => text.to_string(),
        None => format!(
            "Used {} {}",
            exact_count(count as u64),
            if count == 1 { "tool" } else { "tools" }
        ),
    }
}

/// Segments of JSON-like text: each with its class (`key`, `str`, `lit`, `num`)
/// or `None` for punctuation and whitespace. Keys are strings followed by a colon.
pub fn highlight_json(text: &str) -> Vec<(Option<&'static str>, String)> {
    let bytes = text.as_bytes();
    let mut out: Vec<(Option<&'static str>, String)> = Vec::new();
    let mut plain = String::new();
    let mut i = 0;
    let push = |out: &mut Vec<(Option<&'static str>, String)>,
                plain: &mut String,
                class: Option<&'static str>,
                s: &str| {
        if class.is_none() {
            plain.push_str(s);
        } else {
            if !plain.is_empty() {
                out.push((None, std::mem::take(plain)));
            }
            out.push((class, s.to_string()));
        }
    };
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'"'
            && let Some(close) =
                (i + 1..bytes.len()).find(|&j| bytes[j] == b'"' && !is_escaped(bytes, j))
        {
            let after = close + 1;
            let mut colon = after;
            while colon < bytes.len() && bytes[colon].is_ascii_whitespace() {
                colon += 1;
            }
            if bytes.get(colon) == Some(&b':') {
                push(&mut out, &mut plain, Some("key"), &text[i..after]);
                push(&mut out, &mut plain, None, &text[after..colon + 1]);
                i = colon + 1;
            } else {
                push(&mut out, &mut plain, Some("str"), &text[i..after]);
                i = after;
            }
            continue;
        }
        if let Some(end) = json_number_end(bytes, i) {
            push(&mut out, &mut plain, Some("num"), &text[i..end]);
            i = end;
            continue;
        }
        if let Some(word) = ["true", "false", "null"]
            .into_iter()
            .find(|w| text[i..].starts_with(w))
        {
            let end = i + word.len();
            let boundary = !bytes
                .get(end)
                .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_');
            let before_ok =
                i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            if boundary && before_ok {
                push(&mut out, &mut plain, Some("lit"), word);
                i = end;
                continue;
            }
        }
        let ch = text[i..].chars().next().unwrap_or_default();
        push(&mut out, &mut plain, None, &ch.to_string());
        i += ch.len_utf8();
    }
    if !plain.is_empty() {
        out.push((None, plain));
    }
    out
}

fn is_escaped(bytes: &[u8], at: usize) -> bool {
    let backslashes = bytes[..at]
        .iter()
        .rev()
        .take_while(|b| **b == b'\\')
        .count();
    backslashes % 2 == 1
}

/// End of `-?\d+(\.\d+)?([eE][+-]?\d+)?` starting at `i`, when a digit follows the optional minus.
fn json_number_end(bytes: &[u8], i: usize) -> Option<usize> {
    let mut j = i;
    if bytes.get(j) == Some(&b'-') {
        j += 1;
    }
    if !bytes.get(j).is_some_and(|b| b.is_ascii_digit()) {
        return None;
    }
    while bytes.get(j).is_some_and(|b| b.is_ascii_digit()) {
        j += 1;
    }
    if bytes.get(j) == Some(&b'.') && bytes.get(j + 1).is_some_and(|b| b.is_ascii_digit()) {
        j += 1;
        while bytes.get(j).is_some_and(|b| b.is_ascii_digit()) {
            j += 1;
        }
    }
    if matches!(bytes.get(j), Some(b'e' | b'E')) {
        let mut k = j + 1;
        if matches!(bytes.get(k), Some(b'+' | b'-')) {
            k += 1;
        }
        if bytes.get(k).is_some_and(|b| b.is_ascii_digit()) {
            while bytes.get(k).is_some_and(|b| b.is_ascii_digit()) {
                k += 1;
            }
            j = k;
        }
    }
    Some(j)
}

/// Runs a clock while `running`, from `started_at` when given, else from now.
/// `accumulate` adds up several running stretches.
#[derive(Clone, Copy)]
struct Clock {
    start: Signal<Option<u64>>,
    base: Signal<u64>,
    elapsed: Signal<Option<u64>>,
    ticket: Signal<u32>,
}

impl Clock {
    fn new() -> Self {
        Self {
            start: Signal::new(None),
            base: Signal::new(0),
            elapsed: Signal::new(None),
            ticket: Signal::new(0),
        }
    }

    fn follow(self, running: bool, started_at: Option<u64>, accumulate: bool) {
        let mut start = self.start;
        let mut base = self.base;
        let mut elapsed = self.elapsed;
        let mut ticket = self.ticket;
        let mine = *ticket.peek() + 1;
        ticket.set(mine);
        spawn(async move {
            let now = now_ms().await;
            if !running {
                let started = *start.peek();
                if let Some(from) = started {
                    let total = *base.peek() + now.saturating_sub(from);
                    base.set(total);
                    start.set(None);
                    elapsed.set(Some(total));
                }
                return;
            }
            if !accumulate {
                base.set(0);
            }
            let from = started_at.unwrap_or(now);
            start.set(Some(from));
            loop {
                let now = now_ms().await;
                if *ticket.peek() != mine {
                    break;
                }
                let total = *base.peek() + now.saturating_sub(from);
                elapsed.set(Some(total));
                sleep(100).await;
            }
        });
    }
}

async fn now_ms() -> u64 {
    dioxus::document::eval("return Date.now();")
        .await
        .ok()
        .and_then(|value| value.as_f64())
        .map_or(0, |ms| ms as u64)
}

#[component]
pub fn ToolCallCard(
    /// Tool name as the agent calls it, shown in mono, e.g. `search_web`.
    #[props(into)]
    name: String,
    /// Short human summary, e.g. "Searched the web for 'CRM pricing 2026'".
    #[props(default, into)]
    summary: Option<String>,
    /// Icon in the tile on the left. Defaults to a wrench.
    #[props(default)]
    icon: Option<Element>,
    status: ToolCallStatus,
    /// Arguments as text. Shown as given.
    #[props(default, into)]
    args: Option<String>,
    /// Result as text. Shown as given.
    #[props(default, into)]
    result: Option<String>,
    /// Error message, shown when the status is `Error`.
    #[props(default, into)]
    error: Option<String>,
    /// When the call started, in ms since the epoch. The live timer counts from here.
    #[props(default)]
    started_at: Option<u64>,
    /// Final duration in ms. Otherwise measured while the card shows running.
    #[props(default)]
    duration: Option<u64>,
    #[props(default)] default_open: bool,
    /// Expanded state, controlled.
    #[props(default)]
    open: Option<bool>,
    #[props(default)] on_open_change: Option<EventHandler<bool>>,
    /// Opens the card when the call fails and closes it on the next run, unless someone toggled it.
    #[props(default = true)]
    open_on_error: bool,
    /// Shows a Retry button next to the error.
    #[props(default)]
    on_retry: Option<EventHandler<()>>,
    /// Height in px after which arguments and result are capped with a fade and "Show all".
    #[props(default = 220)]
    max_code_height: u32,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("ui-tcl-{}", next_id()));
    let body_id = format!("{id}-body");
    let mut open_state = use_signal(|| default_open);
    let mut auto_opened = use_signal(|| false);
    let mut prev_status = use_signal(|| status);
    let is_open = open.unwrap_or(open_state());
    let controlled = open.is_some();

    use_effect(use_reactive((&status,), move |(next,)| {
        let previous = *prev_status.peek();
        prev_status.set(next);
        if previous == next || controlled || !open_on_error {
            return;
        }
        if next == ToolCallStatus::Error && !*open_state.peek() {
            open_state.set(true);
            auto_opened.set(true);
        } else if previous == ToolCallStatus::Error && *auto_opened.peek() {
            open_state.set(false);
            auto_opened.set(false);
        }
    }));

    let clock = use_hook(Clock::new);
    let running = status == ToolCallStatus::Running;
    use_effect(use_reactive(
        (&running, &started_at),
        move |(is_running, from)| {
            clock.follow(is_running, from, false);
        },
    ));
    let measured = *clock.elapsed.read();
    let ms = if running {
        measured.unwrap_or(0)
    } else {
        duration.or(measured).unwrap_or(0)
    };
    let show_time = running || duration.is_some() || measured.is_some();

    let announce = match status {
        ToolCallStatus::Running => format!("{name} running"),
        ToolCallStatus::Success => format!("{name} finished"),
        ToolCallStatus::Error => match &error {
            Some(message) => format!("{name} failed: {message}"),
            None => format!("{name} failed"),
        },
    };
    let time_text = if status == ToolCallStatus::Error {
        "Failed".to_string()
    } else if show_time {
        format_ms(ms, running)
    } else {
        String::new()
    };
    let error_text = error
        .clone()
        .unwrap_or_else(|| "The tool call failed.".to_string());
    let class = class_list("ui-tcl", &class);
    let icon_el = icon.unwrap_or_else(wrench_icon);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-status": status.as_str(),
            "data-open": is_open.then_some("true"),
            span { class: "ui-tcl-sr", role: "status", aria_live: "polite", "{announce}" }
            button {
                r#type: "button",
                class: "ui-tcl__head",
                aria_expanded: if is_open { "true" } else { "false" },
                aria_controls: "{body_id}",
                onclick: move |_| {
                    auto_opened.set(false);
                    let next = !is_open;
                    if !controlled {
                        open_state.set(next);
                    }
                    if let Some(handler) = on_open_change {
                        handler.call(next);
                    }
                },
                span { class: "ui-tcl__icon", "aria-hidden": "true", {icon_el} }
                span { class: "ui-tcl__main",
                    code { class: "ui-tcl__name", title: "{name}", "{name}" }
                    if let Some(text) = summary.clone() {
                        span { class: "ui-tcl__summary", dir: "auto", title: "{text}", "{text}" }
                    }
                }
                span { class: "ui-tcl__status", key: "{status.as_str()}",
                    if status == ToolCallStatus::Running {
                        Spinner {}
                    }
                    if status == ToolCallStatus::Success {
                        svg { class: "ui-tcl__ok", view_box: "0 0 16 16", "aria-hidden": "true",
                            path { d: "M3.5 8.4 6.6 11.3 12.5 4.9", path_length: "1" }
                        }
                    }
                    if status == ToolCallStatus::Error {
                        ErrorIcon {}
                    }
                    span { class: "ui-tcl__time", "{time_text}" }
                }
                Chevron {}
            }
            div { id: "{body_id}", class: "ui-tcl__body", "inert": (!is_open).then_some("true"),
                div { class: "ui-tcl__clip",
                    div { class: "ui-tcl__sections",
                        if status == ToolCallStatus::Error {
                            div { class: "ui-tcl__error",
                                ErrorIcon {}
                                p { class: "ui-tcl__error-text", dir: "auto", "{error_text}" }
                                if let Some(retry) = on_retry {
                                    button {
                                        r#type: "button",
                                        class: "ui-tcl__retry",
                                        onclick: move |_| retry.call(()),
                                        svg { view_box: "0 0 16 16", "aria-hidden": "true",
                                            path { d: "M2.75 8a5.25 5.25 0 1 0 1.6-3.77" }
                                            path { d: "M2.5 2.5v2.75h2.75" }
                                        }
                                        "Retry"
                                    }
                                }
                            }
                        }
                        if let Some(value) = args.clone() {
                            CodeSection { label: "Arguments", value, cap: max_code_height }
                        }
                        if let Some(value) = result.clone() {
                            CodeSection { label: "Result", value, cap: max_code_height }
                        } else if status == ToolCallStatus::Running {
                            div { class: "ui-tcl__section",
                                div { class: "ui-tcl__section-head",
                                    span { class: "ui-tcl__label", "Result" }
                                }
                                p { class: "ui-tcl__pending", "Waiting for result…" }
                            }
                        }
                        if args.is_none() && result.is_none() && status == ToolCallStatus::Success {
                            p { class: "ui-tcl__pending", "No arguments or result." }
                        }
                    }
                }
            }
        }
    }
}

/// One call listed in a group header.
#[derive(Clone, PartialEq)]
pub struct GroupCall {
    pub name: String,
    pub icon: Option<Element>,
    pub status: ToolCallStatus,
    pub duration: Option<u64>,
    pub started_at: Option<u64>,
}

#[component]
pub fn ToolCallGroup(
    /// The calls, summarised in the header.
    calls: Vec<GroupCall>,
    children: Element,
    /// Label once all calls are done. Defaults to `Used 3 tools`.
    #[props(default, into)]
    label: Option<String>,
    /// Label while any call is running.
    #[props(default = "Using tools…".to_string(), into)]
    running_label: String,
    /// Total duration in ms. Otherwise the sum of the calls' durations, or the time measured while running.
    #[props(default)]
    duration: Option<u64>,
    #[props(default)] default_open: bool,
    #[props(default)] open: Option<bool>,
    #[props(default)] on_open_change: Option<EventHandler<bool>>,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("ui-tcl-group-{}", next_id()));
    let list_id = format!("{id}-list");
    let mut open_state = use_signal(|| default_open);
    let is_open = open.unwrap_or(open_state());
    let controlled = open.is_some();

    let running = calls.iter().any(|c| c.status == ToolCallStatus::Running);
    let failed = calls
        .iter()
        .filter(|c| c.status == ToolCallStatus::Error)
        .count();
    let since = calls
        .iter()
        .filter(|c| c.status == ToolCallStatus::Running)
        .filter_map(|c| c.started_at)
        .min();
    let clock = use_hook(Clock::new);
    use_effect(use_reactive(
        (&running, &since),
        move |(is_running, from)| {
            clock.follow(is_running, from, true);
        },
    ));
    let measured = *clock.elapsed.read();
    let sum = if !calls.is_empty() && calls.iter().all(|c| c.duration.is_some()) {
        Some(calls.iter().filter_map(|c| c.duration).sum::<u64>())
    } else {
        None
    };
    let ms = if running {
        measured
    } else {
        duration.or(sum).or(measured)
    };

    let count = calls.len();
    let title = group_title(count, running, label.as_deref(), &running_label);
    let summary_announce = if running {
        running_label.clone()
    } else if failed > 0 {
        format!(
            "{}, {failed} failed",
            group_title(count, false, label.as_deref(), &running_label)
        )
    } else {
        group_title(count, false, label.as_deref(), &running_label)
    };
    let icon_tail: Vec<(usize, Element)> = calls
        .iter()
        .enumerate()
        .skip(count.saturating_sub(3))
        .map(|(i, c)| (i, c.icon.clone().unwrap_or_else(wrench_icon)))
        .collect();
    let time_text = ms.map(|value| format_ms(value, running));
    let class = class_list("ui-tcl-group", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-running": running.then_some("true"),
            "data-open": is_open.then_some("true"),
            span { class: "ui-tcl-sr", role: "status", aria_live: "polite", "{summary_announce}" }
            button {
                r#type: "button",
                class: "ui-tcl-group__head",
                aria_expanded: if is_open { "true" } else { "false" },
                aria_controls: "{list_id}",
                onclick: move |_| {
                    if !controlled {
                        open_state.set(!is_open);
                    }
                    if let Some(handler) = on_open_change {
                        handler.call(!is_open);
                    }
                },
                span { class: "ui-tcl-group__stack", "aria-hidden": "true",
                    for (i, icon_el) in icon_tail {
                        span { key: "{i}", class: "ui-tcl__icon", {icon_el} }
                    }
                }
                span { class: "ui-tcl-group__title", key: "{running}",
                    span {
                        class: if running { "ui-tcl-group__label ui-tcl__shine" } else { "ui-tcl-group__label" },
                        "data-text": "{title}",
                        "{title}"
                    }
                    if !running && failed > 0 {
                        span { class: "ui-tcl-group__failed", "· {failed} failed" }
                    }
                    if let Some(text) = time_text {
                        span { class: "ui-tcl-group__time", "· {text}" }
                    }
                }
                Chevron {}
            }
            div { id: "{list_id}", class: "ui-tcl__body", "inert": (!is_open).then_some("true"),
                div { class: "ui-tcl__clip",
                    div { class: "ui-tcl-group__list", {children} }
                }
            }
        }
    }
}

#[component]
fn CodeSection(#[props(into)] label: String, value: String, cap: u32) -> Element {
    let mut full = use_signal(|| false);
    let mut height = use_signal(|| 0.0f64);
    let id = use_hook(|| format!("ui-tcl-code-{}", next_id()));
    let pre_id = id.clone();

    let json = value.trim_start().starts_with(['{', '[']);
    let segments = if json {
        highlight_json(&value)
    } else {
        vec![(None, value.clone())]
    };

    use_effect(use_reactive((&value,), move |(_text,)| {
        let script = format!(
            "const el = document.getElementById({pre_id:?});
             return el && el.firstElementChild ? el.scrollHeight : 0;"
        );
        spawn(async move {
            let measured = document::eval(&script)
                .await
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            height.set(measured);
        });
    }));

    let cap = cap as f64;
    let natural = height();
    let capped = natural > cap + 24.0;
    let expanded = full();
    let max_style = if capped {
        let max = if expanded { natural.min(OPEN_MAX) } else { cap };
        Some(format!("max-height: {max}px"))
    } else {
        None
    };
    let copy_label = label.to_lowercase();
    let copy_text = value.clone();
    let heading = label.clone();

    rsx! {
        div { class: "ui-tcl__section",
            div { class: "ui-tcl__section-head",
                span { class: "ui-tcl__label", "{heading}" }
                CopyButton { text: copy_text, label: copy_label }
            }
            pre {
                id: "{id}",
                class: "ui-tcl__code",
                "data-capped": (capped && !expanded).then_some("true"),
                "data-scroll": (capped && expanded && natural > OPEN_MAX).then_some("true"),
                style: max_style,
                code {
                    for (i, (class, text)) in segments.into_iter().enumerate() {
                        if let Some(class) = class {
                            span { key: "{i}", class: format!("ui-tcl__t-{class}"), "{text}" }
                        } else {
                            "{text}"
                        }
                    }
                }
            }
            if capped {
                button {
                    r#type: "button",
                    class: "ui-tcl__more",
                    aria_expanded: if expanded { "true" } else { "false" },
                    onclick: move |_| full.set(!expanded),
                    if expanded { "Show less" } else { "Show all" }
                }
            }
        }
    }
}

#[component]
fn CopyButton(text: String, label: String) -> Element {
    let mut copied = use_signal(|| false);
    let mut ticket = use_signal(|| 0u32);
    let copy_name = if copied() {
        "Copied".to_string()
    } else {
        format!("Copy {label}")
    };

    rsx! {
        button {
            r#type: "button",
            class: "ui-tcl__copy",
            "data-copied": copied().then_some("true"),
            aria_label: "{copy_name}",
            title: "{copy_name}",
            onclick: move |_| {
                copy_to_clipboard(&text);
                copied.set(true);
                let mine = *ticket.peek() + 1;
                ticket.set(mine);
                spawn(async move {
                    sleep(1400).await;
                    if *ticket.peek() == mine {
                        copied.set(false);
                    }
                });
            },
            svg { class: "ui-tcl__copy-icon", view_box: "0 0 16 16", "aria-hidden": "true",
                rect { x: "5.25", y: "5.25", width: "8", height: "8", rx: "2" }
                path { d: "M10.75 5.25v-1a1.5 1.5 0 0 0-1.5-1.5h-4.5a2 2 0 0 0-2 2v4.5a1.5 1.5 0 0 0 1.5 1.5h1" }
            }
            svg { class: "ui-tcl__copy-done", view_box: "0 0 16 16", "aria-hidden": "true",
                path { d: "M3.5 8.4 6.6 11.3 12.5 4.9", path_length: "1" }
            }
        }
    }
}

#[component]
fn Spinner() -> Element {
    rsx! {
        svg { class: "ui-tcl__spinner", view_box: "0 0 16 16", "aria-hidden": "true",
            circle { cx: "8", cy: "8", r: "5.75" }
            path { d: "M8 2.25a5.75 5.75 0 0 1 5.75 5.75" }
        }
    }
}

#[component]
fn ErrorIcon() -> Element {
    rsx! {
        svg { class: "ui-tcl__err", view_box: "0 0 16 16", "aria-hidden": "true",
            circle { cx: "8", cy: "8", r: "6" }
            path { d: "M8 5v3.4M8 10.9v.1" }
        }
    }
}

#[component]
fn Chevron() -> Element {
    rsx! {
        svg { class: "ui-tcl__chevron", view_box: "0 0 16 16", "aria-hidden": "true",
            path { d: "M4.5 6.25 8 9.75l3.5-3.5" }
        }
    }
}

fn wrench_icon() -> Element {
    rsx! {
        svg { view_box: "0 0 16 16",
            path { d: "M10.2 2.4a3.5 3.5 0 0 0-4.3 4.6L2.6 10.3a1.4 1.4 0 0 0 2 2L7.9 9a3.5 3.5 0 0 0 4.6-4.3l-2 2-1.6-.4-.4-1.6 1.7-2.3Z" }
        }
    }
}
