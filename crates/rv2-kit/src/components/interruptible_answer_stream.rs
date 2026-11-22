//! One assistant answer through its states: writing, stopped, failed and finished.
//! The reader can stop it, continue it from where it stopped, retry after a
//! failure without losing the text so far, and copy or regenerate it once it is
//! finished. Messages sent while it is writing show above it as steers, which
//! bend the answer from that point, or as queued messages for the next answer.
//!
//! Design choice: Stop halts the view at once; the next `status` from the caller
//! takes over, and a change of `status` clears the local halt.
//! Design choice: Escape is a document listener while the answer is writing, so
//! it works with focus anywhere outside a text field.
//! Design choice: while writing, a trailing partial word is held back; a bend
//! divider sits at a character offset and splits the paragraph it falls in.
//! Browser checks: words blur in and the caret pulses; Escape and Stop each stop
//! once; Continue keeps the text; Dismiss fades before `on_dismiss`; copy writes
//! plain text; a code block scrolls inside itself and the page never scrolls
//! sideways at 390px; reduced motion keeps the caret still.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, copy_to_clipboard, next_id, sleep};

const CSS: Asset = asset!("/assets/interruptible-answer-stream.css");
/// Steer chips shown before the "+ n more" control.
pub const VISIBLE_STEERS: usize = 3;
const DISMISS_FADE_MS: u32 = 220;
const COPIED_MS: u32 = 1500;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StreamStatus {
    #[default]
    Done,
    Streaming,
    Stopped,
    Failed,
}

impl StreamStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Streaming => "streaming",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentRef {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

impl Default for AgentRef {
    fn default() -> Self {
        Self {
            id: "assistant".to_string(),
            name: "Assistant".to_string(),
            color: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SteerKind {
    Steer,
    Queued,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Steer {
    pub id: String,
    pub text: String,
    pub kind: SteerKind,
}

/// A steer that took effect at character offset `at` of the answer text.
#[derive(Clone, Debug, PartialEq)]
pub struct Bend {
    pub at: usize,
    pub text: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StreamMeta {
    pub tokens: Option<u64>,
    pub duration_ms: Option<u64>,
}

/// The part of `text` safe to show while writing: everything up to the last
/// whitespace, so a partial word stays hidden until its end is written.
pub fn held_back(text: &str) -> &str {
    match text.char_indices().rev().find(|(_, c)| c.is_whitespace()) {
        Some((index, c)) => &text[..index + c.len_utf8()],
        None => "",
    }
}

/// Visible text for a status: a writing answer holds back its partial word.
pub fn visible_text(text: &str, status: StreamStatus) -> &str {
    if status == StreamStatus::Streaming {
        held_back(text)
    } else {
        text
    }
}

enum Block {
    Para(Vec<(usize, String)>),
    Code(usize, String),
}

fn flush(para: &mut Vec<(usize, String)>, out: &mut Vec<Block>) {
    if !para.is_empty() {
        out.push(Block::Para(std::mem::take(para)));
    }
}

/// Paragraphs and fenced code blocks, with each word's character offset.
fn blocks(text: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut para: Vec<(usize, String)> = Vec::new();
    let mut code: Option<(usize, Vec<&str>)> = None;
    let mut offset = 0usize;
    for line in text.split('\n') {
        let line_chars = line.chars().count();
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            match code.take() {
                Some((start, lines)) => out.push(Block::Code(start, lines.join("\n"))),
                None => {
                    flush(&mut para, &mut out);
                    code = Some((offset, Vec::new()));
                }
            }
        } else if let Some((_, lines)) = code.as_mut() {
            lines.push(line);
        } else if trimmed.is_empty() {
            flush(&mut para, &mut out);
        } else {
            let mut word = String::new();
            let mut word_start = 0usize;
            let mut in_word = false;
            for (index, ch) in line.chars().enumerate() {
                if ch.is_whitespace() {
                    if in_word {
                        para.push((offset + word_start, std::mem::take(&mut word)));
                        in_word = false;
                    }
                } else {
                    if !in_word {
                        word_start = index;
                        in_word = true;
                    }
                    word.push(ch);
                }
            }
            if in_word {
                para.push((offset + word_start, word));
            }
        }
        offset += line_chars + 1;
    }
    if let Some((start, lines)) = code {
        out.push(Block::Code(start, lines.join("\n")));
    }
    flush(&mut para, &mut out);
    out
}

#[derive(Clone, Debug, PartialEq)]
pub enum BodyPart {
    Paragraph(Vec<(usize, String)>),
    Code(String),
    /// A steer divider: its character offset and the steer text it records.
    Divider {
        at: usize,
        text: Option<String>,
    },
}

/// The answer body: paragraphs, code blocks and bend dividers in reading order.
/// A bend past the visible text is held until the text reaches it.
pub fn body_parts(visible: &str, bends: &[Bend]) -> Vec<BodyPart> {
    let total = visible.chars().count();
    let mut pending: Vec<&Bend> = bends.iter().filter(|b| b.at <= total).collect();
    pending.sort_by_key(|b| b.at);
    let mut next = 0usize;
    let mut parts = Vec::new();
    let mut current: Vec<(usize, String)> = Vec::new();

    let emit_before = |limit: usize,
                       next: &mut usize,
                       current: &mut Vec<(usize, String)>,
                       parts: &mut Vec<BodyPart>| {
        while *next < pending.len() && pending[*next].at <= limit {
            if !current.is_empty() {
                parts.push(BodyPart::Paragraph(std::mem::take(current)));
            }
            parts.push(BodyPart::Divider {
                at: pending[*next].at,
                text: pending[*next].text.clone(),
            });
            *next += 1;
        }
    };

    for block in blocks(visible) {
        match block {
            Block::Para(words) => {
                for (offset, word) in words {
                    emit_before(offset, &mut next, &mut current, &mut parts);
                    current.push((offset, word));
                }
                if !current.is_empty() {
                    parts.push(BodyPart::Paragraph(std::mem::take(&mut current)));
                }
            }
            Block::Code(start, code) => {
                emit_before(start, &mut next, &mut current, &mut parts);
                parts.push(BodyPart::Code(code));
            }
        }
    }
    emit_before(usize::MAX, &mut next, &mut current, &mut parts);
    parts
}

/// The steers to show, and how many fold behind "+ n more".
pub fn fold_steers(steers: &[Steer], expanded: bool) -> (usize, usize) {
    if expanded || steers.len() <= VISIBLE_STEERS {
        (steers.len(), 0)
    } else {
        (VISIBLE_STEERS, steers.len() - VISIBLE_STEERS)
    }
}

/// The chip's tag: "Steer", "Next" for the first queued message, else "Queued".
pub fn steer_tag(steers: &[Steer], id: &str) -> &'static str {
    let first_queued = steers.iter().find(|s| s.kind == SteerKind::Queued);
    match steers.iter().find(|s| s.id == id).map(|s| s.kind) {
        Some(SteerKind::Steer) => "Steer",
        Some(SteerKind::Queued) if first_queued.is_some_and(|f| f.id == id) => "Next",
        _ => "Queued",
    }
}

/// The small label naming the state.
pub fn status_label(status: StreamStatus) -> &'static str {
    match status {
        StreamStatus::Streaming => "Writing an answer",
        StreamStatus::Stopped => "Stopped",
        StreamStatus::Failed => "Failed",
        StreamStatus::Done => "Finished",
    }
}

/// The polite line for a status change. Writing announces nothing.
pub fn announcement(status: StreamStatus, error: Option<&str>) -> String {
    match status {
        StreamStatus::Streaming => String::new(),
        StreamStatus::Stopped => "Stopped.".to_string(),
        StreamStatus::Failed => match error.filter(|e| !e.is_empty()) {
            Some(reason) => format!("Failed: {reason}"),
            None => "Failed.".to_string(),
        },
        StreamStatus::Done => "Finished.".to_string(),
    }
}

/// Sentences completed after character offset `from`, and the new offset. A
/// sentence completes at `.`, `!` or `?` followed by whitespace, or at the end
/// of a finished answer.
pub fn completed_sentences(visible: &str, from: usize, finished: bool) -> Option<(String, usize)> {
    let chars: Vec<char> = visible.chars().collect();
    let from = if from > chars.len() { 0 } else { from };
    let mut end = None;
    for (index, ch) in chars.iter().enumerate().skip(from) {
        let terminator = matches!(ch, '.' | '!' | '?');
        let closed = index + 1 == chars.len() && finished
            || chars.get(index + 1).is_some_and(|n| n.is_whitespace());
        if terminator && closed {
            end = Some(index + 1);
        }
    }
    let end = end?;
    let sentence: String = chars[from..end].iter().collect();
    let sentence = sentence.trim().to_string();
    (!sentence.is_empty()).then_some((sentence, end))
}

/// The answer as plain text: fences, emphasis markers, backticks and list
/// markers removed.
pub fn plain_text(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .map(|line| {
            let line = line.trim_start().trim_start_matches('#').trim_start();
            let line = line.strip_prefix("- ").unwrap_or(line);
            line.replace("**", "").replace('`', "")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// `1,240 tokens · 4.2 s`, from whichever figures are present.
pub fn meta_line(meta: &StreamMeta) -> Option<String> {
    let tokens = meta
        .tokens
        .map(|t| format!("{} tokens", group_thousands(t)));
    let seconds = meta
        .duration_ms
        .map(|ms| format!("{:.1} s", ms as f64 / 1000.0));
    match (tokens, seconds) {
        (Some(t), Some(s)) => Some(format!("{t} · {s}")),
        (Some(t), None) => Some(t),
        (None, Some(s)) => Some(s),
        (None, None) => None,
    }
}

fn group_thousands(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn render_part(index: usize, part: BodyPart, caret: bool) -> Element {
    match part {
        BodyPart::Paragraph(words) => rsx! {
            p { key: "p{index}", class: "kit-isa__para",
                for (offset, word) in words {
                    span { key: "{offset}", class: "kit-isa__word", "{word} " }
                }
                if caret {
                    span { class: "kit-isa__caret", "aria-hidden": "true" }
                }
            }
        },
        BodyPart::Code(code) => rsx! {
            pre { key: "c{index}", class: "kit-isa__code",
                code { "{code}" }
            }
        },
        BodyPart::Divider { at: _, text } => {
            let shown = text
                .clone()
                .map_or_else(|| "Steer applied".to_string(), |t| format!("Steered: {t}"));
            let named = text.unwrap_or_else(|| "Steer applied".to_string());
            rsx! {
                div { key: "d{index}", class: "kit-isa__bend", role: "separator", "aria-label": "{named}",
                    span { "{shown}" }
                }
            }
        }
    }
}

fn stop_script(root: &str) -> String {
    format!(
        r#"const key = '__kitIsa{root}';
window[key]?.();
const onKey = (event) => {{
    if (event.key !== 'Escape') return;
    const active = document.activeElement;
    if (active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA' || active.isContentEditable)) return;
    dioxus.send(true);
}};
const cleanup = () => {{ document.removeEventListener('keydown', onKey, true); delete window[key]; dioxus.send(false); }};
window[key] = cleanup;
document.addEventListener('keydown', onKey, true);
await new Promise(() => {{}});"#
    )
}

fn stop_cleanup(root: &str) -> String {
    format!("window['__kitIsa{root}']?.();")
}

/// An assistant answer with steers above it and state-specific controls below.
#[component]
pub fn InterruptibleAnswerStream(
    #[props(default, into)] text: String,
    /// The question this answer responds to, shown as a bubble above the answer.
    #[props(default)]
    question: Option<String>,
    #[props(default)] status: StreamStatus,
    #[props(default)] error_message: Option<String>,
    #[props(default)] agent: AgentRef,
    #[props(default)] steers: Vec<Steer>,
    #[props(default)] bends: Vec<Bend>,
    #[props(default)] meta: Option<StreamMeta>,
    #[props(default)] on_stop: Option<EventHandler<()>>,
    #[props(default)] on_continue: Option<EventHandler<()>>,
    #[props(default)] on_retry: Option<EventHandler<()>>,
    #[props(default)] on_dismiss: Option<EventHandler<()>>,
    #[props(default)] on_remove_steer: Option<EventHandler<String>>,
    #[props(default)] on_copy: Option<EventHandler<String>>,
    #[props(default)] on_regenerate: Option<EventHandler<()>>,
    #[props(default = true)] escape_stops: bool,
    #[props(default)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-isa-{}", next_id()));
    let halted = use_signal(|| false);
    let dismissing = use_signal(|| false);
    let steers_open = use_signal(|| false);
    let copied = use_signal(|| false);
    let spoken = use_signal(String::new);
    let announced = use_signal(|| 0usize);

    {
        let mut halted = halted;
        use_effect(use_reactive((&status,), move |_| halted.set(false)));
    }

    let effective = if halted() && status == StreamStatus::Streaming {
        StreamStatus::Stopped
    } else {
        status
    };
    let streaming = effective == StreamStatus::Streaming;
    let finished = effective == StreamStatus::Done;

    let visible = visible_text(&text, effective).to_string();
    {
        let mut spoken = spoken;
        let mut announced = announced;
        use_effect(use_reactive(
            (&visible, &finished),
            move |(visible, finished)| {
                let from = *announced.peek();
                if let Some((sentence, mark)) = completed_sentences(&visible, from, finished) {
                    spoken.set(sentence);
                    announced.set(mark);
                }
            },
        ));
    }

    {
        let on_stop = on_stop.clone();
        let root = id.clone();
        let mut halted = halted;
        use_effect(use_reactive(
            (&streaming, &escape_stops),
            move |(live, enabled)| {
                if live && enabled {
                    let script = stop_script(&root);
                    let on_stop = on_stop.clone();
                    spawn(async move {
                        let mut eval = document::eval(&script);
                        while let Ok(true) = eval.recv::<bool>().await {
                            if !*halted.peek() {
                                halted.set(true);
                                if let Some(h) = &on_stop {
                                    h.call(());
                                }
                            }
                        }
                    });
                } else {
                    document::eval(&stop_cleanup(&root));
                }
            },
        ));
    }

    let state_word = status_label(effective);
    let announce = announcement(effective, error_message.as_deref());
    let meta_text = meta.as_ref().and_then(meta_line);
    let parts = body_parts(&visible, &bends);
    let last_part = parts.len().saturating_sub(1);
    let caret_in_text = matches!(parts.last(), Some(BodyPart::Paragraph(_)));
    let (shown_steers, hidden) = fold_steers(&steers, steers_open());
    let more_label = format!("+ {hidden} more");
    let plain = plain_text(&text);
    let failed = effective == StreamStatus::Failed;
    let error_line = error_message
        .clone()
        .unwrap_or_else(|| "The answer stopped early.".to_string());
    let mark = agent
        .name
        .chars()
        .find(|c| c.is_alphabetic())
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().to_string());
    let tone = agent.id.bytes().fold(0u32, |hash, b| {
        hash.wrapping_mul(31).wrapping_add(u32::from(b))
    }) % 6
        + 1;
    let mark_style = match &agent.color {
        Some(colour) => format!("--isa-colour: {colour};"),
        None => format!("--isa-colour: var(--kit-id-{tone});"),
    };
    let root_class = class_list("kit-isa", &extra_class);
    let dismiss_state = if dismissing() { "dismissing" } else { "idle" };
    let show_copy_done = copied();
    let has_regenerate = on_regenerate.is_some();

    let stop_handler = on_stop.clone();
    let continue_handler = on_continue.clone();
    let retry_handler = on_retry.clone();
    let dismiss_handler = on_dismiss.clone();
    let copy_handler = on_copy.clone();
    let regenerate_handler = on_regenerate.clone();
    let remove_handler = on_remove_steer.clone();
    let agent_name = agent.name.clone();
    let body_label = format!("{} answer", agent_name);

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        article {
            id: "{id}",
            class: root_class,
            "data-state": effective.as_str(),
            "data-dismissing": dismiss_state,
            if let Some(question) = question {
                div { class: "kit-isa__question", "{question}" }
            }
            if !steers.is_empty() {
                div { class: "kit-isa__steers",
                    ul { class: "kit-isa__chips", "aria-label": "Steers",
                        for steer in steers.iter().take(shown_steers) {
                            li { key: "{steer.id}", class: "kit-isa__chip", "data-kind": if steer.kind == SteerKind::Steer { "steer" } else { "queued" },
                                span { class: "kit-isa__tag", {steer_tag(&steers, &steer.id)} }
                                span { class: "kit-isa__chip-text", "{steer.text}" }
                                button {
                                    r#type: "button",
                                    class: "kit-isa__remove",
                                    "aria-label": "Remove steer: {steer.text}",
                                    onclick: {
                                        let handler = remove_handler.clone();
                                        let steer_id = steer.id.clone();
                                        move |_| {
                                            if let Some(h) = &handler {
                                                h.call(steer_id.clone());
                                            }
                                        }
                                    },
                                    "Remove"
                                }
                            }
                        }
                    }
                    if hidden > 0 {
                        button {
                            r#type: "button",
                            class: "kit-isa__more",
                            "aria-expanded": if steers_open() { "true" } else { "false" },
                            onclick: {
                                let mut steers_open = steers_open;
                                move |_| steers_open.set(!steers_open())
                            },
                            "{more_label}"
                        }
                    }
                }
            }
            section { class: "kit-isa__answer",
                header { class: "kit-isa__who",
                    span {
                        class: if streaming { "kit-isa__mark is-wave" } else { "kit-isa__mark" },
                        "aria-hidden": "true",
                        style: "{mark_style}",
                        "{mark}"
                    }
                    span { class: "kit-isa__name", "{agent_name}" }
                }
                div {
                    class: "kit-isa__body",
                    role: "region",
                    "aria-label": "{body_label}",
                    "aria-busy": if streaming { "true" } else { "false" },
                    for (index, part) in parts.into_iter().enumerate() {
                        {render_part(index, part, streaming && index == last_part)}
                    }
                    if streaming && !caret_in_text {
                        span { class: "kit-isa__caret", "aria-hidden": "true" }
                    }
                }
                div { class: "kit-isa__status",
                    span { class: "kit-isa__state", "data-state": effective.as_str(), "{state_word}" }
                }
                if failed {
                    p { class: "kit-isa__error", "{error_line}" }
                }
            }
            footer { class: "kit-isa__actions",
                if streaming {
                    button {
                        r#type: "button",
                        class: "kit-isa__stop",
                        "aria-label": "Stop answer",
                        onclick: {
                            let mut halted = halted;
                            move |_| {
                                if !*halted.peek() {
                                    halted.set(true);
                                    if let Some(h) = &stop_handler {
                                        h.call(());
                                    }
                                }
                            }
                        },
                        "Stop"
                    }
                    span { class: "kit-isa__hint", "Esc also stops" }
                }
                if effective == StreamStatus::Stopped {
                    button {
                        r#type: "button",
                        class: "kit-isa__continue",
                        "aria-label": "Continue answer",
                        onclick: move |_| {
                            if let Some(h) = &continue_handler {
                                h.call(());
                            }
                        },
                        "Continue"
                    }
                    button {
                        r#type: "button",
                        class: "kit-isa__dismiss",
                        "aria-label": "Dismiss answer",
                        onclick: {
                            let mut dismissing = dismissing;
                            move |_| {
                                if *dismissing.peek() {
                                    return;
                                }
                                dismissing.set(true);
                                let handler = dismiss_handler.clone();
                                spawn(async move {
                                    sleep(DISMISS_FADE_MS).await;
                                    if let Some(h) = &handler {
                                        h.call(());
                                    }
                                });
                            }
                        },
                        "Dismiss"
                    }
                }
                if failed {
                    button {
                        r#type: "button",
                        class: "kit-isa__retry",
                        "aria-label": "Retry answer",
                        onclick: move |_| {
                            if let Some(h) = &retry_handler {
                                h.call(());
                            }
                        },
                        "Retry"
                    }
                }
                if finished {
                    if let Some(line) = meta_text {
                        span { class: "kit-isa__meta", "{line}" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-isa__copy",
                        "aria-label": "Copy answer",
                        "data-copied": if show_copy_done { "true" } else { "false" },
                        onclick: {
                            let mut copied = copied;
                            move |_| {
                                copy_to_clipboard(&plain);
                                if let Some(h) = &copy_handler {
                                    h.call(plain.clone());
                                }
                                copied.set(true);
                                spawn(async move {
                                    sleep(COPIED_MS).await;
                                    copied.set(false);
                                });
                            }
                        },
                        if show_copy_done { "Copied" } else { "Copy" }
                    }
                    if has_regenerate {
                        button {
                            r#type: "button",
                            class: "kit-isa__regenerate",
                            "aria-label": "Regenerate answer",
                            onclick: move |_| {
                                if let Some(h) = &regenerate_handler {
                                    h.call(());
                                }
                            },
                            "Regenerate"
                        }
                    }
                }
            }
            div { class: "kit-isa__sr", role: "status", "aria-live": "polite", "{announce}" }
            div { class: "kit-isa__sr", role: "status", "aria-live": "polite", "{spoken}" }
        }
    }
}
