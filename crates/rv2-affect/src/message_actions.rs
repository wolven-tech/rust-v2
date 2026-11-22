//! Message actions: the small bar under an AI chat message. Copy, regenerate with a
//! version switcher, thumbs up and down with a reasons row, and read aloud. The bar
//! stays faded until the message is hovered or something inside it has focus.
//!
//! Port losses:
//! - `versions` takes the count. The React original also accepts a list of labels;
//!   only its length was ever read, so the list form is dropped.
//! - The version switcher, the copy check and the feedback panel update one render
//!   after the event that changes them, because their state is set in an effect
//!   rather than during render.
//! - The Swap crossfade shows the outgoing line one frame after the change.

use dioxus::prelude::*;

use crate::internal::{class_list, copy_to_clipboard, next_id, prefers_motion_script, sleep};
use crate::reaction_bar::exact_count;

const CSS: Asset = asset!("/assets/message-actions.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageActionKey {
    Copy,
    Regenerate,
    Feedback,
    Speak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackKind {
    Up,
    Down,
}

const DEFAULT_ACTIONS: [MessageActionKey; 4] = [
    MessageActionKey::Copy,
    MessageActionKey::Regenerate,
    MessageActionKey::Feedback,
    MessageActionKey::Speak,
];

const DEFAULT_REASONS: [&str; 4] = [
    "Not accurate",
    "Too long",
    "Didn't follow instructions",
    "Other",
];

const ICON_COPY: &str = "M5.5 2A1.5 1.5 0 0 0 4 3.5V4h5A3 3 0 0 1 12 7v5h.5a1.5 1.5 0 0 0 1.5-1.5v-7A1.5 1.5 0 0 0 12.5 2h-7ZM3.5 5A1.5 1.5 0 0 0 2 6.5v6A1.5 1.5 0 0 0 3.5 14H9a1.5 1.5 0 0 0 1.5-1.5v-6A1.5 1.5 0 0 0 9 5H3.5Z";
const ICON_CHECK: &str = "M1.5 8a6.5 6.5 0 1 1 13 0 6.5 6.5 0 0 1-13 0Zm8.9-1.2a.5.5 0 0 0-.8-.6L7.44 9.23 6.35 8.15a.5.5 0 1 0-.7.7l1.5 1.5a.5.5 0 0 0 .76-.06l2.5-3.5Z";
const ICON_TICK: &str = "M3.5 8.5 6.5 11.5 12.5 4.5";
const ICON_REDO: &str = "M13 8a5 5 0 1 1-1.46-3.54M13 2.5v2.5h-2.5";
const ICON_THUMB: &str = "M4.5 7v6.5H3a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1h1.5Zm1 6.5h5.95a1.5 1.5 0 0 0 1.47-1.2l.86-4.3A1.5 1.5 0 0 0 12.31 6.2H9.5l.42-2.08A1.4 1.4 0 0 0 7.3 3.1L5.5 6.8v6.7Z";
const ICON_SPEAKER_BODY: &str = "M7.4 3.2a.6.6 0 0 1 1 .46v8.68a.6.6 0 0 1-1 .46L4.6 10.5H3A1.5 1.5 0 0 1 1.5 9V7A1.5 1.5 0 0 1 3 5.5h1.6l2.8-2.3Z";
const ICON_SPEAKER_WAVES: &str = "M10.6 6.2a2.6 2.6 0 0 1 0 3.6M12.4 4.4a5.2 5.2 0 0 1 0 7.2";
const ICON_NEXT: &str = "M10.85 7.65a.5.5 0 0 1 0 .7L7.2 12a.5.5 0 0 1-.7-.7L9.79 8 6.5 4.7a.5.5 0 1 1 .7-.7l3.65 3.65Z";
const ICON_CLOSE: &str = "m4.5 4.5 7 7m0-7-7 7";

/// Each action once, in the order given.
pub fn unique_actions(list: &[MessageActionKey]) -> Vec<MessageActionKey> {
    let mut out: Vec<MessageActionKey> = Vec::with_capacity(list.len());
    for &key in list {
        if !out.contains(&key) {
            out.push(key);
        }
    }
    out
}

/// At least one version.
pub fn version_count(versions: usize) -> usize {
    versions.max(1)
}

/// `index` kept inside `0..count`.
pub fn clamp_index(index: usize, count: usize) -> usize {
    index.min(count.max(1) - 1)
}

/// How long read aloud plays when uncontrolled: the override if positive, else an
/// estimate of 380 ms per word, at least 2.6 seconds.
pub fn speak_duration_ms(text: Option<&str>, override_ms: Option<u32>) -> u32 {
    match override_ms {
        Some(ms) if ms > 0 => ms,
        _ => {
            let words = text.map_or(0, |t| t.split_whitespace().count()) as u32;
            2600u32.max(words * 380)
        }
    }
}

/// The reason sent with a thumbs down. `Other` sends the typed text, or `Other`
/// when nothing was typed.
pub fn feedback_reason(reason: &str, other: &str) -> String {
    if reason == "Other" {
        let typed = other.trim();
        if typed.is_empty() {
            "Other".to_string()
        } else {
            typed.to_string()
        }
    } else {
        reason.to_string()
    }
}

/// Screen-reader announcement. A repeat of the last one gets a trailing space, so
/// the live region still changes and speaks again.
pub fn announcement(previous: &str, next: &str) -> String {
    if previous == next {
        format!("{next} ")
    } else {
        next.to_string()
    }
}

fn announce(mut status: Signal<String>, text: &str) {
    let next = announcement(&status.peek(), text);
    status.set(next);
}

/// Sets the shown version and reports it. Controlled versions only report.
fn go_to(
    mut inner: Signal<usize>,
    status: Signal<String>,
    controlled: bool,
    on_change: Option<EventHandler<usize>>,
    index: usize,
    count: usize,
    target: usize,
) {
    let next = clamp_index(target, count);
    if next == index {
        return;
    }
    if !controlled {
        inner.set(next);
    }
    if let Some(handler) = on_change {
        handler.call(next);
    }
    announce(
        status,
        &format!(
            "Version {} of {}",
            exact_count(next as u64 + 1),
            exact_count(count as u64)
        ),
    );
}

async fn motion_allowed() -> bool {
    document::eval(&format!("return {};", prefers_motion_script()))
        .await
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}

/// Text that changes without a jump: the old line fades out underneath while the
/// new one fades in.
#[component]
fn Swap(#[props(into)] id: String, children: Element, #[props(default)] class: String) -> Element {
    let mut current = use_signal(|| id.clone());
    let mut outgoing = use_signal(|| None::<(String, Element)>);
    let mut last = use_signal(|| None::<Element>);

    use_effect(use_reactive((&id, &children), move |(new_id, node)| {
        let previous = current.peek().clone();
        if previous != new_id {
            if let Some(old) = last.peek().clone() {
                outgoing.set(Some((previous, old)));
                spawn(async move {
                    sleep(300).await;
                    outgoing.set(None);
                });
            }
            current.set(new_id);
        }
        last.set(Some(node));
    }));

    let class = class_list("ui-mac__swap", &class);
    rsx! {
        span { class,
            if let Some((out_id, out_node)) = outgoing() {
                span { key: "out-{out_id}", class: "ui-mac__swap-item", "data-out": "", "aria-hidden": "true", {out_node} }
            }
            span { key: "{id}", class: "ui-mac__swap-item", {children} }
        }
    }
}

/// The message itself, crossfading when `version` changes. The block eases to its
/// new height.
#[component]
pub fn MessageText(version: String, children: Element, #[props(default)] class: String) -> Element {
    let id = use_hook(|| format!("ui-mac-text-{}", next_id()));
    let inner_id = format!("{id}-inner");
    let outer_id = id.clone();
    let class = class_list("ui-mac-text", &class);

    let (outer_sync, inner_sync) = (outer_id.clone(), inner_id.clone());
    use_effect(move || {
        document::eval(&format!(
            "const o = document.getElementById({outer_sync:?});
             const i = document.getElementById({inner_sync:?});
             if (!o || !i || typeof ResizeObserver === 'undefined') return;
             const ro = new ResizeObserver(() => {{ o.style.height = i.offsetHeight + 'px'; }});
             ro.observe(i);
             o.__sync = ro;"
        ));
    });
    let outer_drop = outer_id.clone();
    use_drop(move || {
        document::eval(&format!(
            "document.getElementById({outer_drop:?})?.__sync?.disconnect();"
        ));
    });

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { id: "{outer_id}", class,
            div { id: "{inner_id}", class: "ui-mac-text__inner", dir: "auto",
                Swap { id: version, class: "ui-mac-text__swap", {children} }
            }
        }
    }
}

#[component]
fn Glyph(d: &'static str, #[props(default)] solid: bool) -> Element {
    rsx! {
        svg { class: "ui-mac__glyph", view_box: "0 0 16 16", "data-solid": solid.then_some(""), "aria-hidden": "true",
            path { d }
        }
    }
}

/// The speaker: a solid body with two thin sound waves.
#[component]
fn Speaker() -> Element {
    rsx! {
        svg { class: "ui-mac__glyph", view_box: "0 0 16 16", "aria-hidden": "true",
            path { class: "ui-mac__fill", d: ICON_SPEAKER_BODY }
            path { d: ICON_SPEAKER_WAVES }
        }
    }
}

/// One button with its tooltip.
#[component]
fn ActionButton(
    kind: &'static str,
    #[props(into)] tip: String,
    #[props(into)] label: String,
    onclick: EventHandler<()>,
    #[props(default, into)] pressed: Option<bool>,
    #[props(default, into)] expanded: Option<bool>,
    #[props(default, into)] controls: Option<String>,
    #[props(default)] flash: bool,
    #[props(default)] done: bool,
    #[props(default)] busy: bool,
    children: Element,
) -> Element {
    let btn_class = format!("ui-mac__btn ui-mac__btn--{kind}");
    let pressed = pressed.map(|p| if p { "true" } else { "false" });
    let expanded = expanded.map(|e| if e { "true" } else { "false" });
    rsx! {
        span { class: "ui-mac__item", "data-flash": flash.then_some(""),
            button {
                r#type: "button",
                class: btn_class,
                aria_label: "{label}",
                aria_pressed: pressed,
                aria_expanded: expanded,
                aria_controls: controls,
                "data-done": done.then_some(""),
                "data-busy": busy.then_some(""),
                onclick: move |_| onclick.call(()),
                {children}
            }
            span { class: "ui-mac__tip", "aria-hidden": "true",
                Swap { id: tip.clone(), "{tip}" }
            }
        }
    }
}

#[component]
pub fn MessageActions(
    /// The message text: copied by Copy and used to time Read aloud.
    #[props(default)]
    text: Option<String>,
    /// Which actions to show, in this order.
    #[props(default = DEFAULT_ACTIONS.to_vec())]
    actions: Vec<MessageActionKey>,
    /// Number of versions. From 2 on the version switcher shows.
    #[props(default = 1)]
    versions: usize,
    /// Controlled version, 0-based.
    #[props(default)]
    version: Option<usize>,
    /// Starting version when uncontrolled. Defaults to the newest.
    #[props(default)]
    default_version: Option<usize>,
    #[props(default)] on_version_change: Option<EventHandler<usize>>,
    /// Called after Copy, with the text.
    #[props(default)]
    on_copy: Option<EventHandler<Option<String>>>,
    /// Called when Regenerate is pressed. Add a version and the switcher jumps to it.
    #[props(default)]
    on_regenerate: Option<EventHandler<()>>,
    /// Keeps the regenerate arrow spinning while a new version is written.
    #[props(default)]
    regenerating: bool,
    /// Called with the kind when a thumb is pressed, `None` when it is taken back.
    /// A thumbs down that is sent carries its reason.
    #[props(default)]
    on_feedback: Option<EventHandler<(Option<FeedbackKind>, Option<String>)>>,
    /// Quick reasons in the feedback row. `Other` opens a small text field.
    #[props(default = DEFAULT_REASONS.iter().map(|r| r.to_string()).collect())]
    reasons: Vec<String>,
    /// Called with true when Read aloud starts and false when it stops.
    #[props(default)]
    on_speak: Option<EventHandler<bool>>,
    /// Controlled playing state, e.g. tied to a speech player.
    #[props(default)]
    speaking: Option<bool>,
    /// How long an uncontrolled read aloud plays, in ms. Defaults to an estimate from `text`.
    #[props(default)]
    speak_duration: Option<u32>,
    /// Faded until hovered. Put `data-ui-message` on the message wrapper.
    #[props(default = true)]
    quiet: bool,
    #[props(default = "Message actions".to_string(), into)] label: String,
    #[props(default)] class: String,
) -> Element {
    let uid = use_hook(|| format!("ui-mac-{}", next_id()));
    let feedback_id = format!("{uid}-feedback");
    let ask_id = format!("{uid}-ask");
    let down_id = format!("{uid}-down");

    let actions = unique_actions(&actions);
    let count = version_count(versions);
    let mut inner = use_signal(|| clamp_index(default_version.unwrap_or(count - 1), count));
    let mut seen_count = use_signal(move || count);
    let mut vote = use_signal(|| None::<FeedbackKind>);
    let mut panel = use_signal(|| false);
    let mut reason = use_signal(|| None::<String>);
    let mut other = use_signal(String::new);
    let mut sent = use_signal(|| false);
    let status = use_signal(String::new);
    let mut copied = use_signal(|| false);
    let mut copy_ticket = use_signal(|| 0u32);
    let mut spin = use_signal(|| 0u32);
    let mut inner_speaking = use_signal(|| false);
    let mut speak_ticket = use_signal(|| 0u32);

    use_effect(use_reactive((&count,), move |(next,)| {
        let previously = *seen_count.peek();
        if next > previously {
            inner.set(next - 1);
            vote.set(None);
            panel.set(false);
        }
        seen_count.set(next);
    }));

    let index = clamp_index(version.unwrap_or(inner()), count);

    use_effect(use_reactive((&sent(),), move |(is_sent,)| {
        if !is_sent {
            return;
        }
        spawn(async move {
            let delay = if motion_allowed().await { 1500 } else { 1200 };
            sleep(delay).await;
            panel.set(false);
        });
    }));

    let speak_ms = speak_duration_ms(text.as_deref(), speak_duration);
    use_effect(use_reactive(
        (&inner_speaking(), &speaking, &speak_ms),
        move |(on, controlled, ms)| {
            let mine = *speak_ticket.peek() + 1;
            speak_ticket.set(mine);
            if !on || controlled.is_some() {
                return;
            }
            spawn(async move {
                sleep(ms).await;
                if *speak_ticket.peek() != mine {
                    return;
                }
                inner_speaking.set(false);
                if let Some(handler) = on_speak {
                    handler.call(false);
                }
                announce(status, "Finished reading");
            });
        },
    ));

    let playing = speaking.unwrap_or(inner_speaking());
    let copied_now = copied();
    let panel_open = panel();
    let regen_busy = regenerating;
    let active = copied_now || panel_open || playing || regen_busy;
    let current_vote = vote();
    let current_reason = reason();
    let sent_now = sent();
    let other_text = other();
    let index_text = exact_count(index as u64 + 1);
    let count_text = exact_count(count as u64);
    let versions_label = format!("Version {index_text} of {count_text}");
    let has_feedback = actions.contains(&MessageActionKey::Feedback);
    let class = class_list("ui-mac", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-quiet": quiet.then_some(""),
            "data-active": active.then_some(""),
            div { class: "ui-mac__bar", role: "group", aria_label: "{label}",
                for key in actions.iter().copied() {
                    {
                        match key {
                            MessageActionKey::Copy => {
                                let copy_text = text.clone();
                                let copy_label = if copied_now { "Copied" } else { "Copy message" };
                                let copy_tip = if copied_now { "Copied" } else { "Copy" };
                                rsx! {
                                    ActionButton {
                                        kind: "copy",
                                        tip: copy_tip,
                                        label: copy_label,
                                        flash: copied_now,
                                        done: copied_now,
                                        onclick: move |_| {
                                            if let Some(value) = copy_text.clone() {
                                                copy_to_clipboard(&value);
                                            }
                                            if let Some(handler) = on_copy {
                                                handler.call(copy_text.clone());
                                            }
                                            copied.set(true);
                                            let mine = *copy_ticket.peek() + 1;
                                            copy_ticket.set(mine);
                                            spawn(async move {
                                                sleep(1600).await;
                                                if *copy_ticket.peek() == mine {
                                                    copied.set(false);
                                                }
                                            });
                                            announce(status, "Copied");
                                        },
                                        span { class: "ui-mac__icon ui-mac__morph",
                                            span { class: "ui-mac__layer", "data-on": (!copied_now).then_some(""),
                                                Glyph { d: ICON_COPY, solid: true }
                                            }
                                            span { class: "ui-mac__layer ui-mac__layer--check", "data-on": copied_now.then_some(""),
                                                Glyph { d: ICON_CHECK, solid: true }
                                            }
                                        }
                                    }
                                }
                            }
                            MessageActionKey::Regenerate => {
                                let spin_now = spin();
                                rsx! {
                                    span { class: "ui-mac__regen",
                                        ActionButton {
                                            kind: "regenerate",
                                            tip: if regen_busy { "Writing…" } else { "Regenerate" },
                                            label: "Regenerate response",
                                            busy: regen_busy,
                                            onclick: move |_| {
                                                spin.set(spin() + 1);
                                                if let Some(handler) = on_regenerate {
                                                    handler.call(());
                                                }
                                                announce(status, "Regenerating");
                                            },
                                            span { class: "ui-mac__icon",
                                                span { key: "{spin_now}", class: "ui-mac__spin", "data-spin": (spin_now > 0).then_some(""),
                                                    Glyph { d: ICON_REDO }
                                                }
                                            }
                                        }
                                        span {
                                            class: "ui-mac__versions",
                                            "data-open": (count > 1).then_some(""),
                                            "inert": (count < 2).then_some("true"),
                                            span { class: "ui-mac__versions-clip", role: "group", aria_label: "Versions",
                                                button {
                                                    r#type: "button",
                                                    class: "ui-mac__step",
                                                    "data-dir": "prev",
                                                    aria_label: "Previous version",
                                                    disabled: index == 0,
                                                    onclick: move |_| go_to(inner, status, version.is_some(), on_version_change, index, count, index.saturating_sub(1)),
                                                    Glyph { d: ICON_NEXT, solid: true }
                                                }
                                                span { class: "ui-mac__count", aria_label: "{versions_label}",
                                                    Swap { id: index.to_string(), class: "ui-mac__count-now", "{index_text}" }
                                                    span { "aria-hidden": "true", "/" }
                                                    Swap { id: count.to_string(), "{count_text}" }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "ui-mac__step",
                                                    "data-dir": "next",
                                                    aria_label: "Next version",
                                                    disabled: index + 1 >= count,
                                                    onclick: move |_| go_to(inner, status, version.is_some(), on_version_change, index, count, index + 1),
                                                    Glyph { d: ICON_NEXT, solid: true }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            MessageActionKey::Feedback => {
                                rsx! {
                                    ActionButton {
                                        kind: "up",
                                        tip: if current_vote == Some(FeedbackKind::Up) { "Thanks!" } else { "Good response" },
                                        label: "Good response",
                                        pressed: current_vote == Some(FeedbackKind::Up),
                                        onclick: move |_| {
                                            if vote() == Some(FeedbackKind::Up) {
                                                vote.set(None);
                                                if let Some(handler) = on_feedback {
                                                    handler.call((None, None));
                                                }
                                                return;
                                            }
                                            vote.set(Some(FeedbackKind::Up));
                                            panel.set(false);
                                            if let Some(handler) = on_feedback {
                                                handler.call((Some(FeedbackKind::Up), None));
                                            }
                                            announce(status, "Marked as good response");
                                        },
                                        span { class: "ui-mac__icon ui-mac__thumb", Glyph { d: ICON_THUMB, solid: true } }
                                    }
                                    ActionButton {
                                        kind: "down",
                                        tip: "Bad response",
                                        label: "Bad response",
                                        pressed: current_vote == Some(FeedbackKind::Down),
                                        expanded: panel_open,
                                        controls: feedback_id.clone(),
                                        onclick: move |_| {
                                            if vote() == Some(FeedbackKind::Down) {
                                                vote.set(None);
                                                panel.set(false);
                                                if let Some(handler) = on_feedback {
                                                    handler.call((None, None));
                                                }
                                                return;
                                            }
                                            vote.set(Some(FeedbackKind::Down));
                                            panel.set(true);
                                            sent.set(false);
                                            reason.set(None);
                                            other.set(String::new());
                                            if let Some(handler) = on_feedback {
                                                handler.call((Some(FeedbackKind::Down), None));
                                            }
                                            announce(status, "Marked as bad response. Tell us what went wrong below");
                                        },
                                        span { class: "ui-mac__icon ui-mac__thumb", Glyph { d: ICON_THUMB, solid: true } }
                                    }
                                }
                            }
                            MessageActionKey::Speak => {
                                rsx! {
                                    ActionButton {
                                        kind: "speak",
                                        tip: if playing { "Stop" } else { "Read aloud" },
                                        label: if playing { "Stop reading" } else { "Read aloud" },
                                        pressed: playing,
                                        onclick: move |_| {
                                            let next = !playing;
                                            if speaking.is_none() {
                                                inner_speaking.set(next);
                                            }
                                            if let Some(handler) = on_speak {
                                                handler.call(next);
                                            }
                                            announce(status, if next { "Reading aloud" } else { "Stopped reading" });
                                        },
                                        span { class: "ui-mac__icon ui-mac__morph",
                                            span { class: "ui-mac__layer", "data-on": (!playing).then_some(""),
                                                Speaker {}
                                            }
                                            span { class: "ui-mac__layer ui-mac__eq", "data-on": playing.then_some(""),
                                                i {}
                                                i {}
                                                i {}
                                                i {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if has_feedback {
                div {
                    id: "{feedback_id}",
                    class: "ui-mac__fold",
                    "data-open": panel_open.then_some(""),
                    "inert": (!panel_open).then_some("true"),
                    div { class: "ui-mac__clip",
                        div {
                            class: "ui-mac__panel",
                            role: "group",
                            aria_labelledby: "{ask_id}",
                            onkeydown: {
                                let down_for_key = down_id.clone();
                                move |event| {
                                    if event.key() == Key::Escape {
                                        event.stop_propagation();
                                        panel.set(false);
                                        document::eval(&format!("document.getElementById({down_for_key:?})?.focus();"));
                                    }
                                }
                            },
                            div { class: "ui-mac__panel-head",
                                span { id: "{ask_id}", class: "ui-mac__ask", "What went wrong?" }
                                button {
                                    r#type: "button",
                                    class: "ui-mac__close",
                                    aria_label: "Close feedback",
                                    onclick: {
                                        let down_for_close = down_id.clone();
                                        move |_| {
                                            panel.set(false);
                                            document::eval(&format!("document.getElementById({down_for_close:?})?.focus();"));
                                        }
                                    },
                                    Glyph { d: ICON_CLOSE }
                                }
                            }
                            div { class: "ui-mac__chips",
                                for (i, option) in reasons.iter().enumerate() {
                                    {
                                        let option_key = format!("{option}-{i}");
                                        let option_value = option.clone();
                                        let picked = option.clone();
                                        let pressed = current_reason.as_deref() == Some(option.as_str());
                                        rsx! {
                                            button {
                                                key: "{option_key}",
                                                r#type: "button",
                                                class: "ui-mac__chip",
                                                aria_pressed: if pressed { "true" } else { "false" },
                                                disabled: sent_now,
                                                title: "{option_value}",
                                                dir: "auto",
                                                onclick: move |_| {
                                                    let next = if reason().as_deref() == Some(picked.as_str()) { None } else { Some(picked.clone()) };
                                                    reason.set(next);
                                                },
                                                "{option_value}"
                                            }
                                        }
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "ui-mac__send",
                                    "data-sent": sent_now.then_some(""),
                                    disabled: current_reason.is_none(),
                                    aria_disabled: sent_now.then_some("true"),
                                    onclick: move |_| {
                                        let Some(chosen) = reason() else { return };
                                        if sent() {
                                            return;
                                        }
                                        let typed = other();
                                        if let Some(handler) = on_feedback {
                                            handler.call((Some(FeedbackKind::Down), Some(feedback_reason(&chosen, &typed))));
                                        }
                                        sent.set(true);
                                        announce(status, "Thanks for the feedback");
                                    },
                                    Swap { id: if sent_now { "sent" } else { "send" },
                                        if sent_now {
                                            Glyph { d: ICON_TICK }
                                            "Thanks!"
                                        } else {
                                            "Send"
                                        }
                                    }
                                }
                            }
                            if reasons.iter().any(|r| r == "Other") {
                                div {
                                    class: "ui-mac__fold",
                                    "data-open": (current_reason.as_deref() == Some("Other") && !sent_now).then_some(""),
                                    "inert": (current_reason.as_deref() != Some("Other") || sent_now).then_some("true"),
                                    div { class: "ui-mac__clip",
                                        input {
                                            class: "ui-mac__input",
                                            value: "{other_text}",
                                            placeholder: "Tell us more (optional)",
                                            dir: "auto",
                                            aria_label: "What went wrong, in your words",
                                            oninput: move |event| other.set(event.value()),
                                            onkeydown: move |event| {
                                                if event.key() == Key::Enter {
                                                    let Some(chosen) = reason() else { return };
                                                    if sent() {
                                                        return;
                                                    }
                                                    let typed = other();
                                                    if let Some(handler) = on_feedback {
                                                        handler.call((Some(FeedbackKind::Down), Some(feedback_reason(&chosen, &typed))));
                                                    }
                                                    sent.set(true);
                                                    announce(status, "Thanks for the feedback");
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            span { class: "ui-mac__sr", aria_live: "polite", "{status}" }
        }
    }
}
