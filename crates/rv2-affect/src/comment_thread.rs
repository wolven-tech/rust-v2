//! Comment thread: one comment with its replies, including an assistant that
//! reports what it did. Assistant replies fold open to show the details.
//!
//! Port losses:
//! - `onReply` returns no promise here. The caller answers by passing the updated
//!   `messages` back, and drives the thinking state with `thinking`.
//! - `messages` follows the prop from then on, so a reply the caller adds is shown,
//!   but a message the viewer just posted is replaced if the caller passes a list
//!   that does not include it yet.
//! - Timestamps come from `Date.now()` through eval, so the first frame reads
//!   "just now" for every message.
//! - The height glide starts from a ResizeObserver set up in eval after mount, so
//!   the first render is not guarded against a transition.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/comment-thread.css");

#[derive(Clone, PartialEq)]
pub enum ThreadAvatar {
    /// Picture URL. A picture that fails to load falls back to the initials.
    Url(String),
    /// Any element, e.g. your own logo.
    Node(Element),
}

#[derive(Clone, PartialEq)]
pub struct ThreadAuthor {
    pub name: String,
    pub avatar: Option<ThreadAvatar>,
    pub assistant: bool,
}

#[derive(Clone, PartialEq)]
pub struct ThreadMessage {
    pub id: String,
    pub author: ThreadAuthor,
    /// Milliseconds since the Unix epoch.
    pub created_at: u64,
    pub text: String,
    /// Assistant replies: a short status line that folds open to show the details.
    pub status: Option<String>,
    pub details: Vec<String>,
}

/// Rounds `a / b` to the nearest whole number, halves up.
fn round_div(a: u64, b: u64) -> u64 {
    (a as f64 / b as f64).round() as u64
}

/// Relative time: `just now`, `5min ago`, `2h ago`, `3d ago`, `2mo ago`, `1y ago`.
pub fn ago(time_ms: u64, now_ms: u64) -> String {
    let s = round_div(now_ms.saturating_sub(time_ms), 1000);
    if s < 45 {
        return "just now".to_string();
    }
    let m = round_div(s, 60);
    if m < 60 {
        return format!("{m}min ago");
    }
    let h = round_div(m, 60);
    if h < 24 {
        return format!("{h}h ago");
    }
    let d = round_div(h, 24);
    if d < 30 {
        format!("{d}d ago")
    } else if d < 365 {
        format!("{}mo ago", round_div(d, 30))
    } else {
        format!("{}y ago", round_div(d, 365))
    }
}

/// Two initials from whole letters. Skips emoji and stray spaces, so `🦊 Fox` gives `F`.
pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().find(|c| c.is_alphanumeric()))
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

/// The text after a leading `/edit`, when the message is an edit command. The
/// command must end at a word boundary, so `/editor` is not one.
pub fn edit_command(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    let head = trimmed.get(..5)?;
    if !head.eq_ignore_ascii_case("/edit") {
        return None;
    }
    let rest = &trimmed[5..];
    if rest
        .chars()
        .next()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
    {
        return None;
    }
    Some(rest.trim_start())
}

async fn now_ms() -> u64 {
    dioxus::document::eval("return Date.now();")
        .await
        .ok()
        .and_then(|value| value.as_f64())
        .map_or(0, |ms| ms as u64)
}

#[component]
pub fn CommentThread(
    #[props(default = "Comment".to_string(), into)] title: String,
    messages: Vec<ThreadMessage>,
    current_user: ThreadAuthor,
    /// The assistant that answers; used for the thinking state and the reply placeholder.
    #[props(default)]
    assistant: Option<ThreadAuthor>,
    /// True while the assistant is working on an answer.
    #[props(default)]
    thinking: bool,
    /// Called with every new reply the viewer posts.
    #[props(default)]
    on_reply: Option<EventHandler<String>>,
    #[props(default)] on_close: Option<EventHandler<()>>,
    #[props(default)] class: String,
) -> Element {
    let mut messages_state = use_signal(|| messages.clone());
    let delayed_ids = use_hook(|| {
        messages
            .iter()
            .filter(|m| m.author.assistant)
            .map(|m| m.id.clone())
            .collect::<Vec<_>>()
    });
    let mut open = use_signal(Vec::<String>::new);
    let mut draft = use_signal(String::new);
    let mut now = use_signal(|| 0u64);
    let id = use_hook(|| format!("ui-ctc-{}", next_id()));
    let list_id = format!("{id}-scroll");
    let morph_id = format!("{id}-morph");

    use_effect(use_reactive((&messages,), move |(next,)| {
        messages_state.set(next);
    }));

    // The clock refreshes every 30 seconds, as the relative times are minute-grained.
    use_effect(move || {
        spawn(async move {
            loop {
                now.set(now_ms().await);
                sleep(30_000).await;
            }
        });
    });

    let list_scroll = list_id.clone();
    use_effect(use_reactive(
        (&messages_state().len(), &thinking),
        move |(_len, _thinking)| {
            let script = format!(
                "const el = document.getElementById({list_scroll:?});
             if (el) el.scrollTo({{ top: el.scrollHeight, behavior: 'smooth' }});"
            );
            document::eval(&script);
        },
    ));

    let morph_for_sync = morph_id.clone();
    let list_for_sync = list_id.clone();
    use_effect(move || {
        let script = format!(
            "const o = document.getElementById({morph_for_sync:?});
             const i = document.getElementById({list_for_sync:?});
             if (!o || !i) return;
             const sync = () => {{ o.style.height = i.offsetHeight + 'px'; }};
             sync();
             const ro = new ResizeObserver(sync);
             ro.observe(i);
             requestAnimationFrame(() => o.setAttribute('data-smooth', ''));
             o.__sync = ro;"
        );
        document::eval(&script);
    });
    let morph_drop = morph_id.clone();
    use_drop(move || {
        document::eval(&format!(
            "document.getElementById({morph_drop:?})?.__sync?.disconnect();"
        ));
    });

    let lock = {
        let list_id = list_id.clone();
        move |on: bool| {
            let script = format!(
                "const l = document.getElementById({list_id:?});
                 if (l) l.style.maxHeight = {on} ? l.offsetHeight + 'px' : '';"
            );
            document::eval(&script);
        }
    };

    let placeholder = match &assistant {
        Some(bot) => format!("Reply, or ask @{}…", bot.name),
        None => "Add comment…".to_string(),
    };
    let send_disabled = draft().trim().is_empty() || thinking;
    let class = class_list("ui-ctc", &class);
    let messages_now = messages_state();
    let count_text = if messages_now.len() == 1 {
        "message"
    } else {
        "messages"
    };
    let message_count = messages_now.len();
    let now_value = now();
    let morph_class = "ui-ctc__morph";

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        section { class, aria_label: "{title}",
            header { class: "ui-ctc__head",
                h3 { class: "ui-ctc__title", title: "{title}", dir: "auto", "{title}" }
                span { class: "ui-ctc__count", "{message_count} {count_text}" }
                if let Some(on_close) = on_close {
                    button {
                        r#type: "button",
                        class: "ui-ctc__icon-btn",
                        aria_label: "Close comment",
                        onclick: move |_| on_close.call(()),
                        Icon { d: "m4.5 4.5 7 7M11.5 4.5l-7 7" }
                    }
                }
            }
            div { id: "{morph_id}", class: morph_class,
                div { id: "{list_id}", class: "ui-ctc__scroll", aria_live: "polite",
                    for m in messages_now {
                        {
                            let is_open = open().contains(&m.id);
                            let command = edit_command(&m.text).map(str::to_string);
                            let delayed = delayed_ids.contains(&m.id);
                            let time_text = ago(m.created_at, now_value);
                            let details_present = !m.details.is_empty();
                            let toggle_id = m.id.clone();
                            let lock_toggle = lock.clone();
                            let key = m.id.clone();
                            let author_name = m.author.name.clone();
                            let body_text = m.text.clone();
                            rsx! {
                                article {
                                    key: "{key}",
                                    class: "ui-ctc__msg",
                                    "data-delayed": delayed.then_some("true"),
                                    div { class: "ui-ctc__meta",
                                        Avatar { author: m.author.clone() }
                                        span { class: "ui-ctc__name", title: "{author_name}", dir: "auto", "{author_name}" }
                                        time { class: "ui-ctc__time", "{time_text}" }
                                    }
                                    if !m.text.is_empty() {
                                        p { class: "ui-ctc__text", dir: "auto",
                                            if let Some(body) = command {
                                                span { class: "ui-ctc__command", "/edit" }
                                                " {body}"
                                            } else {
                                                "{body_text}"
                                            }
                                        }
                                    }
                                    if let Some(status) = m.status.clone() {
                                        div { class: "ui-ctc__status",
                                            button {
                                                r#type: "button",
                                                class: "ui-ctc__status-btn",
                                                aria_expanded: if is_open { "true" } else { "false" },
                                                disabled: !details_present,
                                                onclick: move |_| {
                                                    lock_toggle(false);
                                                    let mut next = open();
                                                    if let Some(pos) = next.iter().position(|o| *o == toggle_id) {
                                                        next.remove(pos);
                                                    } else {
                                                        next.push(toggle_id.clone());
                                                    }
                                                    open.set(next);
                                                },
                                                svg { class: "ui-ctc__check", view_box: "0 0 16 16", "aria-hidden": "true",
                                                    path { d: "M1.5 8a6.5 6.5 0 1 1 13 0 6.5 6.5 0 0 1-13 0Zm8.9-1.2a.5.5 0 0 0-.8-.6L7.44 9.23 6.35 8.15a.5.5 0 1 0-.7.7l1.5 1.5a.5.5 0 0 0 .76-.06l2.5-3.5Z" }
                                                }
                                                span { class: "ui-ctc__status-text", title: "{status}", dir: "auto", "{status}" }
                                                if details_present {
                                                    span { class: "ui-ctc__chevron",
                                                        Icon { d: "m5 6.5 3 3 3-3" }
                                                    }
                                                }
                                            }
                                            if details_present {
                                                div {
                                                    class: "ui-ctc__details",
                                                    "data-open": is_open.then_some("true"),
                                                    "inert": (!is_open).then_some("true"),
                                                    ul {
                                                        for (i, detail) in m.details.iter().enumerate() {
                                                            li {
                                                                key: "{i}",
                                                                dir: "auto",
                                                                style: "--i: {i}",
                                                                "{detail}"
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
                    }
                    if thinking {
                        if let Some(bot) = assistant.clone() {
                            {
                                let bot_name = bot.name.clone();
                                rsx! {
                            article { class: "ui-ctc__msg", aria_label: "{bot_name} is thinking",
                                div { class: "ui-ctc__meta",
                                    Avatar { author: bot.clone() }
                                    span { class: "ui-ctc__name", "{bot_name}" }
                                }
                                p { class: "ui-ctc__text",
                                    span { class: "ui-ctc__thinking", "data-text": "Thinking…", "Thinking…" }
                                }
                            }
                                }
                            }
                        }
                    }
                }
            }
            form {
                class: "ui-ctc__composer",
                onsubmit: move |event| {
                    event.prevent_default();
                    let text = draft().trim().to_string();
                    if text.is_empty() || thinking {
                        return;
                    }
                    draft.set(String::new());
                    lock(true);
                    let author = current_user.clone();
                    spawn(async move {
                        let t = now_ms().await;
                        now.set(t);
                        messages_state.write().push(ThreadMessage {
                            id: format!("m-{t}"),
                            author,
                            created_at: t,
                            text: text.clone(),
                            status: None,
                            details: Vec::new(),
                        });
                        if let Some(handler) = on_reply {
                            handler.call(text);
                        }
                    });
                },
                input {
                    class: "ui-ctc__input",
                    value: "{draft}",
                    oninput: move |event| draft.set(event.value()),
                    placeholder: placeholder,
                    aria_label: "Reply",
                }
                button {
                    r#type: "submit",
                    class: "ui-ctc__send",
                    aria_label: "Send reply",
                    disabled: send_disabled,
                    svg { view_box: "0 0 16 16", "aria-hidden": "true", path { d: "M8 12.5v-9M4.25 7.25 8 3.5l3.75 3.75" } }
                }
            }
        }
    }
}

#[component]
fn Avatar(author: ThreadAuthor) -> Element {
    let mut failed = use_signal(|| false);
    match author.avatar.clone() {
        Some(ThreadAvatar::Node(node)) => rsx! { span { class: "ui-ctc__avatar", {node} } },
        Some(ThreadAvatar::Url(src)) if !failed() => rsx! {
            img {
                class: "ui-ctc__avatar",
                src: "{src}",
                alt: "",
                width: "20",
                height: "20",
                loading: "lazy",
                onerror: move |_| failed.set(true),
            }
        },
        _ => {
            let letters = initials(&author.name);
            rsx! {
                span { class: "ui-ctc__avatar ui-ctc__avatar--initials", "aria-hidden": "true", "{letters}" }
            }
        }
    }
}

#[component]
fn Icon(d: String) -> Element {
    rsx! {
        svg {
            class: "ui-ctc__glyph",
            view_box: "0 0 16 16",
            "aria-hidden": "true",
            path { d }
        }
    }
}
