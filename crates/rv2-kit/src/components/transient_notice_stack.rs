//! Transient notice stack: brief messages raised from anywhere through `use_notifier()`, shown in a
//! stack at one screen edge, each with a timer that freezes while the pointer or focus is inside.
//!
//! Design choice: `NoticeHost` owns the store and provides a `Notifier` through context. Errors go
//! to an assertive region, everything else to a polite one. The visible limit shows the oldest
//! messages first; the rest wait with their timers frozen. Timers tick every 100 ms through
//! `internal::sleep`.
//!
//! Browser checks: swipe to dismiss is not implemented; the focus key is not bound; the stack
//! fans out on hover and focus; a loading message turns into success or error in place.

use std::{future::Future, pin::Pin};

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonSize, ButtonVariant};

use crate::internal::{KIT_CSS, class_list, sleep};

const CSS: Asset = asset!("/assets/transient-notice-stack.css");
const TICK_MS: u32 = 100;

/// A message whose text comes from a background task: success text or error text.
pub type PromiseTask = Pin<Box<dyn Future<Output = Result<String, String>>>>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackEdge {
    TopLeft,
    TopCenter,
    TopRight,
    #[default]
    BottomRight,
    BottomCenter,
    BottomLeft,
}

impl StackEdge {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopCenter => "top-center",
            Self::TopRight => "top-right",
            Self::BottomRight => "bottom-right",
            Self::BottomCenter => "bottom-center",
            Self::BottomLeft => "bottom-left",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoticeVariant {
    #[default]
    Standard,
    Info,
    Success,
    Error,
    Loading,
}

impl NoticeVariant {
    /// Errors are announced assertively; everything else politely.
    pub fn is_assertive(self) -> bool {
        self == Self::Error
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Self::Standard => "•",
            Self::Info => "i",
            Self::Success => "✓",
            Self::Error => "×",
            Self::Loading => "",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldTime {
    Millis(u32),
    Infinite,
}

#[derive(Clone, PartialEq)]
pub struct NoticeAction {
    pub action_label: String,
    /// Returns `true` to close the message after the action; `false` keeps it open.
    pub on_activate: Callback<(), bool>,
}

#[derive(Clone, PartialEq)]
pub struct Notice {
    pub notice_id: usize,
    pub title: String,
    pub detail_text: Option<String>,
    pub variant: NoticeVariant,
    pub action: Option<NoticeAction>,
    pub hold: HoldTime,
    pub stack_key: Option<String>,
    pub remaining_ms: u32,
}

#[derive(Clone, PartialEq, Default)]
pub struct NoticeOptions {
    pub message_id: Option<usize>,
    pub detail_text: Option<String>,
    pub variant: NoticeVariant,
    pub action: Option<NoticeAction>,
    pub hold_time: Option<HoldTime>,
    pub stack_key: Option<String>,
}

/// Indices of the notices that show now (the first `limit` for `key`), and those that wait.
pub fn visible_split(
    notices: &[Notice],
    key: Option<&str>,
    limit: usize,
) -> (Vec<usize>, Vec<usize>) {
    let matching: Vec<usize> = notices
        .iter()
        .enumerate()
        .filter(|(_, n)| n.stack_key.as_deref() == key)
        .map(|(i, _)| i)
        .collect();
    let split = limit.min(matching.len());
    (matching[..split].to_vec(), matching[split..].to_vec())
}

/// Removes `dt` from a finite timer. `true` when it has run out.
pub fn tick(remaining_ms: &mut u32, hold: HoldTime, dt: u32) -> bool {
    match hold {
        HoldTime::Infinite => false,
        HoldTime::Millis(_) => {
            *remaining_ms = remaining_ms.saturating_sub(dt);
            *remaining_ms == 0
        }
    }
}

/// A sideways swipe of more than 80 px, or a downward one of more than 40 px, dismisses.
pub fn swipe_dismisses(dx: f64, dy: f64) -> bool {
    dx.abs() > 80.0 || dy > 40.0
}

fn hold_millis(hold: HoldTime) -> u32 {
    match hold {
        HoldTime::Millis(ms) => ms,
        HoldTime::Infinite => 0,
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Notifier {
    notices: Signal<Vec<Notice>>,
    next_id: Signal<usize>,
    frozen: Signal<bool>,
    ticking: Signal<bool>,
    default_hold: HoldTime,
}

impl Notifier {
    /// Shows a message and returns its id. Passing `message_id` updates that message in place.
    pub fn notify(&self, title: &str) -> usize {
        self.notify_with(title, NoticeOptions::default())
    }

    pub fn notify_with(&self, title: &str, options: NoticeOptions) -> usize {
        let hold = options.hold_time.unwrap_or(self.default_hold);
        let remaining = hold_millis(hold);
        let mut next_id = self.next_id;
        let mut notices = self.notices;
        let id = match options.message_id {
            Some(id) => id,
            None => {
                let id = next_id();
                next_id.set(id + 1);
                id
            }
        };
        let notice = Notice {
            notice_id: id,
            title: title.to_string(),
            detail_text: options.detail_text,
            variant: options.variant,
            action: options.action,
            hold,
            stack_key: options.stack_key,
            remaining_ms: remaining,
        };
        notices.with_mut(|list| match list.iter_mut().find(|n| n.notice_id == id) {
            Some(existing) => *existing = notice,
            None => list.push(notice),
        });
        self.start_ticker();
        id
    }

    /// Shows a loading message that becomes success or error text when `task` settles.
    pub fn notify_promise(&self, loading: &str, task: PromiseTask) -> usize {
        let id = self.notify_with(
            loading,
            NoticeOptions {
                variant: NoticeVariant::Loading,
                hold_time: Some(HoldTime::Infinite),
                ..NoticeOptions::default()
            },
        );
        let this = *self;
        spawn(async move {
            let (title, variant) = match task.await {
                Ok(text) => (text, NoticeVariant::Success),
                Err(text) => (text, NoticeVariant::Error),
            };
            this.notify_with(
                &title,
                NoticeOptions {
                    message_id: Some(id),
                    variant,
                    ..NoticeOptions::default()
                },
            );
        });
        id
    }

    pub fn dismiss(&self, id: Option<usize>) {
        let mut notices = self.notices;
        notices.with_mut(|list| match id {
            Some(id) => list.retain(|n| n.notice_id != id),
            None => list.clear(),
        });
    }

    fn start_ticker(&self) {
        if (self.ticking)() {
            return;
        }
        let mut ticking = self.ticking;
        ticking.set(true);
        let mut notices = self.notices;
        let frozen = self.frozen;
        spawn(async move {
            while !notices().is_empty() {
                sleep(TICK_MS).await;
                if frozen() {
                    continue;
                }
                notices.with_mut(|list| {
                    list.retain_mut(|n| !tick(&mut n.remaining_ms, n.hold, TICK_MS));
                });
            }
            ticking.set(false);
        });
    }
}

/// The notifier of the nearest `NoticeHost`.
pub fn use_notifier() -> Notifier {
    use_context::<Notifier>()
}

#[component]
fn NoticeCard(
    notice: Notice,
    show_timer_line: bool,
    show_close_control: bool,
    newest: bool,
    on_dismiss: EventHandler<usize>,
) -> Element {
    let id = notice.notice_id;
    let total = hold_millis(notice.hold).max(1);
    let fraction = notice.remaining_ms as f64 / total as f64;
    let timer_style = format!("transform: scaleX({fraction:.3})");
    let glyph = notice.variant.glyph();
    let assertive = notice.variant.is_assertive();
    let variant_name = format!("{:?}", notice.variant).to_lowercase();
    let action = notice.action.clone();
    let close_label = format!("Dismiss {}", notice.title);
    let mut drag_start = use_signal(|| None::<(f64, f64)>);
    let mut offset = use_signal(|| (0.0_f64, 0.0_f64));
    let (dx, dy) = offset();
    let slide = format!("translate: {dx}px {dy}px");
    rsx! {
        div {
            class: "kit-tns__card",
            "data-variant": "{variant_name}",
            "data-assertive": assertive.then_some("true"),
            "data-newest": newest.then_some("true"),
            tabindex: "0",
            style: "{slide}",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    on_dismiss.call(id);
                }
            },
            onpointerdown: move |event: PointerEvent| {
                let point = event.client_coordinates();
                drag_start.set(Some((point.x, point.y)));
            },
            onpointermove: move |event: PointerEvent| {
                if let Some((sx, sy)) = drag_start() {
                    let point = event.client_coordinates();
                    offset.set((point.x - sx, point.y - sy));
                }
            },
            onpointerup: move |_| {
                let (dx, dy) = offset();
                drag_start.set(None);
                if swipe_dismisses(dx, dy) {
                    on_dismiss.call(id);
                } else {
                    offset.set((0.0, 0.0));
                }
            },
            onpointercancel: move |_| {
                drag_start.set(None);
                offset.set((0.0, 0.0));
            },
            span { class: "kit-tns__icon", "aria-hidden": "true",
                if notice.variant == NoticeVariant::Loading {
                    span { class: "kit-tns__spinner" }
                } else {
                    "{glyph}"
                }
            }
            div { class: "kit-tns__text",
                span { class: "kit-tns__title", "{notice.title}" }
                if let Some(detail) = notice.detail_text.clone() {
                    span { class: "kit-tns__detail", "{detail}" }
                }
            }
            if let Some(action) = action {
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    label: action.action_label.clone(),
                    onclick: move |_| {
                        if action.on_activate.call(()) {
                            on_dismiss.call(id);
                        }
                    },
                }
            }
            if show_close_control {
                button {
                    r#type: "button",
                    class: "kit-tns__close",
                    "aria-label": "{close_label}",
                    onclick: move |_| on_dismiss.call(id),
                    "×"
                }
            }
            if show_timer_line && notice.hold != HoldTime::Infinite {
                span { class: "kit-tns__timer", style: "{timer_style}" }
            }
        }
    }
}

#[component]
pub fn NoticeHost(
    #[props(default)] stack_edge: StackEdge,
    #[props(default)] inside_parent: bool,
    #[props(default)] stack_key: Option<String>,
    #[props(default = HoldTime::Millis(4000))] default_hold_time: HoldTime,
    #[props(default = 3)] visible_limit: usize,
    #[props(default)] always_expanded: bool,
    #[props(default = 16)] edge_inset: u32,
    #[props(default = 8)] fan_gap: u32,
    #[props(default = true)] show_timer_line: bool,
    #[props(default = true)] show_close_control: bool,
    #[props(default)] focus_key: Option<String>,
    #[props(default = "Notifications".to_string(), into)] region_label: String,
    #[props(default, into)] extra_class: String,
    children: Element,
) -> Element {
    let notices = use_signal(Vec::<Notice>::new);
    let next_id = use_signal(|| 1usize);
    let mut frozen = use_signal(|| false);
    let ticking = use_signal(|| false);
    let notifier = use_hook(|| Notifier {
        notices,
        next_id,
        frozen,
        ticking,
        default_hold: default_hold_time,
    });
    use_context_provider(|| notifier);
    let region_id = use_hook(|| format!("kit-tns-{}", crate::internal::next_id()));
    let focus_region = region_id.clone();
    use_effect(move || {
        if let Some(key) = focus_key.clone() {
            let js = format!(
                "window.addEventListener('keydown', (e) => {{ if (e.key === {key:?} && !e.ctrlKey && !e.metaKey && !e.altKey) {{ const el = document.getElementById({focus_region:?}); const newest = el && el.querySelector('[data-newest]'); if (newest) newest.focus(); }} }});"
            );
            document::eval(&js);
        }
    });

    let list = notices();
    let (shown, _waiting) = visible_split(&list, stack_key.as_deref(), visible_limit);
    let visible: Vec<Notice> = shown.iter().map(|i| list[*i].clone()).collect();
    let polite: Vec<Notice> = visible
        .iter()
        .filter(|n| !n.variant.is_assertive())
        .cloned()
        .collect();
    let assertive: Vec<Notice> = visible
        .iter()
        .filter(|n| n.variant.is_assertive())
        .cloned()
        .collect();
    let newest_id = visible.last().map(|n| n.notice_id);
    let spread = always_expanded || frozen();
    let edge = stack_edge.as_str();
    let position = if inside_parent { "absolute" } else { "fixed" };
    let inset = format!("{edge_inset}px");
    let gap = format!("{fan_gap}px");
    let dismiss = notifier;

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: class_list("kit-tns", &extra_class),
            {children}
            section {
                id: "{region_id}",
                class: "kit-tns__stack",
                role: "region",
                "aria-label": "{region_label}",
                "data-edge": "{edge}",
                "data-spread": spread.then_some("true"),
                "data-position": "{position}",
                style: "--kit-tns-inset: {inset}; --kit-tns-gap: {gap}",
                onmouseenter: move |_| frozen.set(true),
                onmouseleave: move |_| frozen.set(false),
                onfocus: move |_| frozen.set(true),
                onblur: move |_| frozen.set(false),
                div { class: "kit-tns__polite", role: "status", "aria-live": "polite",
                    for notice in polite.iter() {
                        NoticeCard {
                            key: "{notice.notice_id}",
                            notice: notice.clone(),
                            show_timer_line,
                            show_close_control,
                            newest: Some(notice.notice_id) == newest_id,
                            on_dismiss: move |id: usize| dismiss.dismiss(Some(id)),
                        }
                    }
                }
                div { class: "kit-tns__assertive", role: "alert",
                    for notice in assertive.iter() {
                        NoticeCard {
                            key: "{notice.notice_id}",
                            notice: notice.clone(),
                            show_timer_line,
                            show_close_control,
                            newest: Some(notice.notice_id) == newest_id,
                            on_dismiss: move |id: usize| dismiss.dismiss(Some(id)),
                        }
                    }
                }
            }
        }
    }
}
