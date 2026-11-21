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

const ICON_COPY: &str = "M9 9.5A1.5 1.5 0 0 1 10.5 8h8A1.5 1.5 0 0 1 20 9.5v9a1.5 1.5 0 0 1-1.5 1.5h-8A1.5 1.5 0 0 1 9 18.5zM15 8V5.5A1.5 1.5 0 0 0 13.5 4h-8A1.5 1.5 0 0 0 4 5.5v9A1.5 1.5 0 0 0 5.5 16H9";
const ICON_CHECK: &str = "M5.5 12.5l4.2 4.2L18.5 7.8";
const ICON_REDO: &str = "M19.5 12a7.5 7.5 0 1 1-2.2-5.3L19.5 9M19.5 4.5V9H15";
const ICON_THUMB: &str = "M7.5 10.5v9M7.5 10.5l3.3-6a1.8 1.8 0 0 1 3.3 1.2l-.6 3.8H18a2 2 0 0 1 2 2.3l-1.2 6.4a2 2 0 0 1-2 1.8H5a1 1 0 0 1-1-1v-7.5a1 1 0 0 1 1-1z";
const ICON_SPEAKER: &str =
    "M11 5.5 6.8 9H4v6h2.8l4.2 3.5zM15.2 9.2a4 4 0 0 1 0 5.6M17.9 6.6a7.8 7.8 0 0 1 0 10.8";
const ICON_PREV: &str = "m14.5 7-5 5 5 5";
const ICON_NEXT: &str = "m9.5 7 5 5-5 5";
const ICON_CLOSE: &str = "m7.5 7.5 9 9m0-9-9 9";

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

    let class = class_list("ui-ma__swap", &class);
    rsx! {
        span { class,
            if let Some((out_id, out_node)) = outgoing() {
                span { key: "out-{out_id}", class: "ui-ma__swap-item", "data-out": "", "aria-hidden": "true", {out_node} }
            }
            span { key: "{id}", class: "ui-ma__swap-item", {children} }
        }
    }
}

/// The message itself, crossfading when `version` changes. The block eases to its
/// new height.
#[component]
pub fn MessageText(version: String, children: Element, #[props(default)] class: String) -> Element {
    let id = use_hook(|| format!("ui-ma-text-{}", next_id()));
    let inner_id = format!("{id}-inner");
    let outer_id = id.clone();
    let class = class_list("ui-ma-text", &class);

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
            div { id: "{inner_id}", class: "ui-ma-text__inner", dir: "auto",
                Swap { id: version, class: "ui-ma-text__swap", {children} }
            }
        }
    }
}

#[component]
fn Glyph(d: &'static str) -> Element {
    rsx! {
        svg { class: "ui-ma__glyph", view_box: "0 0 24 24", "aria-hidden": "true",
            path { d }
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
    let btn_class = format!("ui-ma__btn ui-ma__btn--{kind}");
    let pressed = pressed.map(|p| if p { "true" } else { "false" });
    let expanded = expanded.map(|e| if e { "true" } else { "false" });
    rsx! {
        span { class: "ui-ma__item", "data-flash": flash.then_some(""),
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
            span { class: "ui-ma__tip", "aria-hidden": "true",
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
    let uid = use_hook(|| format!("ui-ma-{}", next_id()));
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
    let class = class_list("ui-ma", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-quiet": quiet.then_some(""),
            "data-active": active.then_some(""),
            div { class: "ui-ma__bar", role: "group", aria_label: "{label}",
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
                                        span { class: "ui-ma__icon ui-ma__morph",
                                            span { class: "ui-ma__layer", "data-on": (!copied_now).then_some(""),
                                                Glyph { d: ICON_COPY }
                                            }
                                            span { class: "ui-ma__layer ui-ma__layer--check", "data-on": copied_now.then_some(""),
                                                Glyph { d: ICON_CHECK }
                                            }
                                        }
                                    }
                                }
                            }
                            MessageActionKey::Regenerate => {
                                let spin_now = spin();
                                rsx! {
                                    span { class: "ui-ma__regen",
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
                                            span { class: "ui-ma__icon",
                                                span { key: "{spin_now}", class: "ui-ma__spin", "data-spin": (spin_now > 0).then_some(""),
                                                    Glyph { d: ICON_REDO }
                                                }
                                            }
                                        }
                                        span {
                                            class: "ui-ma__versions",
                                            "data-open": (count > 1).then_some(""),
                                            "inert": (count < 2).then_some("true"),
                                            span { class: "ui-ma__versions-clip", role: "group", aria_label: "Versions",
                                                button {
                                                    r#type: "button",
                                                    class: "ui-ma__step",
                                                    "data-dir": "prev",
                                                    aria_label: "Previous version",
                                                    disabled: index == 0,
                                                    onclick: move |_| go_to(inner, status, version.is_some(), on_version_change, index, count, index.saturating_sub(1)),
                                                    Glyph { d: ICON_PREV }
                                                }
                                                span { class: "ui-ma__count", aria_label: "{versions_label}",
                                                    Swap { id: index.to_string(), class: "ui-ma__count-now", "{index_text}" }
                                                    span { "aria-hidden": "true", "/" }
                                                    Swap { id: count.to_string(), "{count_text}" }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "ui-ma__step",
                                                    "data-dir": "next",
                                                    aria_label: "Next version",
                                                    disabled: index + 1 >= count,
                                                    onclick: move |_| go_to(inner, status, version.is_some(), on_version_change, index, count, index + 1),
                                                    Glyph { d: ICON_NEXT }
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
                                        span { class: "ui-ma__icon ui-ma__thumb", Glyph { d: ICON_THUMB } }
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
                                        span { class: "ui-ma__icon ui-ma__thumb", Glyph { d: ICON_THUMB } }
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
                                        span { class: "ui-ma__icon ui-ma__morph",
                                            span { class: "ui-ma__layer", "data-on": (!playing).then_some(""),
                                                Glyph { d: ICON_SPEAKER }
                                            }
                                            span { class: "ui-ma__layer ui-ma__eq", "data-on": playing.then_some(""),
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
                    class: "ui-ma__fold",
                    "data-open": panel_open.then_some(""),
                    "inert": (!panel_open).then_some("true"),
                    div { class: "ui-ma__clip",
                        div {
                            class: "ui-ma__panel",
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
                            div { class: "ui-ma__panel-head",
                                span { id: "{ask_id}", class: "ui-ma__ask", "What went wrong?" }
                                button {
                                    r#type: "button",
                                    class: "ui-ma__close",
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
                            div { class: "ui-ma__chips",
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
                                                class: "ui-ma__chip",
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
                                    class: "ui-ma__send",
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
                                            Glyph { d: ICON_CHECK }
                                            "Thanks!"
                                        } else {
                                            "Send"
                                        }
                                    }
                                }
                            }
                            if reasons.iter().any(|r| r == "Other") {
                                div {
                                    class: "ui-ma__fold",
                                    "data-open": (current_reason.as_deref() == Some("Other") && !sent_now).then_some(""),
                                    "inert": (current_reason.as_deref() != Some("Other") || sent_now).then_some("true"),
                                    div { class: "ui-ma__clip",
                                        input {
                                            class: "ui-ma__input",
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

            span { class: "ui-ma__sr", aria_live: "polite", "{status}" }
        }
    }
}
