//! Notification stack: notices stacked behind each other. Only the front card can
//! be read; the others peek out below it. Dismissing the front card slides it away
//! and the next one moves up.
//!
//! Port losses: card heights are measured after each render that changes the
//! notice list, by a one-off `document::eval`. The React original keeps a
//! ResizeObserver on every card, so a card whose height changes without a list
//! change (a window resize that rewraps its text) keeps its old height until the
//! next list change. A changed `notices` list is re-applied in the same render
//! here, where React re-applies it one render later.

use std::collections::HashMap;
use std::f64::consts::PI;

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/notification-stack.css");

/// How far each card behind peeks out below the one in front, in px.
pub const PEEK: f64 = 12.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoticePattern {
    #[default]
    Orbit,
    Grid,
    Rays,
}

#[derive(Clone, PartialEq)]
pub struct NoticeAction {
    pub label: String,
    /// With an href the action is a link; without one it is a button.
    pub href: Option<String>,
    pub on_click: Option<EventHandler<()>>,
    /// Opens in a new tab with an arrow.
    pub external: bool,
}

#[derive(Clone, PartialEq)]
pub struct Notice {
    pub id: String,
    /// Small label in the pill, e.g. "Notice".
    pub badge: Option<String>,
    pub title: String,
    pub text: String,
    pub action: Option<NoticeAction>,
    /// Line art in the top-right corner.
    pub pattern: NoticePattern,
}

/// Index of `id` among the visible notices, or `None` when it is leaving.
pub fn card_depth(visible_ids: &[&str], id: &str) -> Option<usize> {
    visible_ids.iter().position(|visible| *visible == id)
}

/// Height of the stack: the front card plus the peeking cards behind it (at most two).
pub fn stack_height(visible: usize, front: f64) -> f64 {
    if visible == 0 {
        0.0
    } else {
        front + visible.saturating_sub(1).min(2) as f64 * PEEK
    }
}

/// Vertical offset of a card behind the front one. It lines up on the front card's
/// bottom edge and is cut to the front height when it is taller.
pub fn card_y(depth: usize, front: f64, height: f64) -> f64 {
    if depth == 0 {
        0.0
    } else {
        front - height.min(front) + depth.min(2) as f64 * PEEK
    }
}

/// Each card behind shrinks by 5%, down to two steps.
pub fn card_scale(depth: usize) -> f64 {
    1.0 - depth.min(2) as f64 * 0.05
}

/// End point of ray `i` of the rays pattern, on a 220 by 160 viewBox.
pub fn ray_end(i: u32) -> (f64, f64) {
    let angle = i as f64 * PI / 16.0;
    (210.0 - angle.cos() * 230.0, 6.0 + angle.sin() * 230.0)
}

fn dismiss(
    mut leaving: Signal<Vec<String>>,
    mut removed: Signal<Vec<String>>,
    on_dismiss: Option<EventHandler<Notice>>,
    notice: Notice,
) {
    let id = notice.id.clone();
    leaving.write().push(id.clone());
    if let Some(handler) = on_dismiss {
        handler.call(notice);
    }
    spawn(async move {
        sleep(320).await;
        removed.write().push(id.clone());
        leaving.write().retain(|pending| *pending != id);
    });
}

// Measures the body, not the card: the corner art overflows the card and scrollHeight counts it.
fn measure_script(stack_id: &str) -> String {
    format!(
        "const root = document.getElementById({stack_id:?});
         const out = {{}};
         if (root) root.querySelectorAll(':scope > [data-notice]').forEach((el) => {{
           out[el.dataset.notice] = el.querySelector(':scope > .ui-nsc__body')?.offsetHeight ?? 0;
         }});
         return out;"
    )
}

#[component]
pub fn NotificationStack(
    notices: Vec<Notice>,
    /// Called after a notice was dismissed.
    #[props(default)]
    on_dismiss: Option<EventHandler<Notice>>,
    #[props(default)] class: String,
) -> Element {
    let removed = use_signal(Vec::<String>::new);
    let leaving = use_signal(Vec::<String>::new);
    let mut heights = use_signal(HashMap::<String, f64>::new);
    let stack_id = use_hook(|| format!("ui-nsc-{}", next_id()));

    let measure_id = stack_id.clone();
    use_effect(use_reactive(
        (&removed(), &notices),
        move |(_removed, _notices)| {
            let script = measure_script(&measure_id);
            spawn(async move {
                let measured = document::eval(&script).await.ok();
                let next: HashMap<String, f64> = measured
                    .as_ref()
                    .and_then(|value| value.as_object())
                    .map(|map| {
                        map.iter()
                            .filter_map(|(key, value)| value.as_f64().map(|h| (key.clone(), h)))
                            .collect()
                    })
                    .unwrap_or_default();
                heights.set(next);
            });
        },
    ));

    let removed_now = removed();
    let leaving_now = leaving();
    let heights_now = heights();
    let items: Vec<Notice> = notices
        .iter()
        .filter(|notice| !removed_now.contains(&notice.id))
        .cloned()
        .collect();
    let visible_ids: Vec<&str> = items
        .iter()
        .filter(|notice| !leaving_now.contains(&notice.id))
        .map(|notice| notice.id.as_str())
        .collect();
    let front = visible_ids
        .first()
        .and_then(|id| heights_now.get(*id))
        .copied()
        .unwrap_or(0.0);
    let stack_height = stack_height(visible_ids.len(), front);
    let cards: Vec<(Notice, Option<usize>)> = items
        .iter()
        .map(|notice| (notice.clone(), card_depth(&visible_ids, &notice.id)))
        .collect();
    let class = class_list("ui-nsc", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        section {
            id: "{stack_id}",
            class,
            aria_label: "Notifications",
            style: "height: {stack_height}px",
            for (notice, depth) in cards.into_iter().filter(|(_, depth)| depth.is_none_or(|d| d <= 3)) {
                {
                    let gone = depth.is_none();
                    let depth = depth.unwrap_or(0);
                    let is_behind = !gone && depth > 0;
                    let height = heights_now.get(&notice.id).copied().unwrap_or(front);
                    let y = if gone { 0.0 } else { card_y(depth, front, height) };
                    let mut style = format!("z-index: {}", if gone { 10 } else { 10 - depth as i32 });
                    if !gone {
                        style.push_str(&format!("; transform: translateY({y}px) scale({})", card_scale(depth)));
                    }
                    if !gone && depth > 2 {
                        style.push_str("; opacity: 0");
                    }
                    if is_behind && front != 0.0 {
                        style.push_str(&format!("; max-height: {front}px"));
                    }
                    let title_id = format!("{}-title", notice.id);
                    let notice_id = notice.id.clone();
                    let notice_title = notice.title.clone();
                    let notice_text = notice.text.clone();
                    let dismissed = notice.clone();
                    let action = notice.action.clone();
                    rsx! {
                        article {
                            key: "{notice_id}",
                            class: "ui-nsc__card",
                            "data-notice": "{notice_id}",
                            "data-depth": (!gone).then(|| depth.to_string()),
                            "data-leaving": gone.then_some("true"),
                            aria_labelledby: "{title_id}",
                            aria_hidden: is_behind.then_some("true"),
                            "inert": (is_behind || gone).then_some("true"),
                            style,
                            Pattern { kind: notice.pattern }
                            div { class: "ui-nsc__body",
                                div { class: "ui-nsc__top",
                                    if let Some(badge) = notice.badge.clone() {
                                        span { class: "ui-nsc__badge", dir: "auto", title: "{badge}", "{badge}" }
                                    } else {
                                        span {}
                                    }
                                    button {
                                        r#type: "button",
                                        class: "ui-nsc__close",
                                        aria_label: "Dismiss {notice_title}",
                                        onclick: move |event| {
                                            event.stop_propagation();
                                            dismiss(leaving, removed, on_dismiss, dismissed.clone());
                                        },
                                        CloseIcon {}
                                    }
                                }
                                h3 { id: "{title_id}", class: "ui-nsc__title", dir: "auto", "{notice_title}" }
                                p { class: "ui-nsc__text", dir: "auto", "{notice_text}" }
                                if let Some(action) = action {
                                    {
                                        let label = action.label.clone();
                                        let external = action.external;
                                        let on_click = action.on_click;
                                        match action.href.clone() {
                                            Some(href) => rsx! {
                                                a {
                                                    class: "ui-nsc__btn",
                                                    dir: "auto",
                                                    href: "{href}",
                                                    target: external.then_some("_blank"),
                                                    rel: external.then_some("noreferrer"),
                                                    onclick: move |_| {
                                                        if let Some(handler) = on_click {
                                                            handler.call(());
                                                        }
                                                    },
                                                    "{label}"
                                                    if external {
                                                        ArrowIcon {}
                                                    }
                                                }
                                            },
                                            None => rsx! {
                                                button {
                                                    r#type: "button",
                                                    class: "ui-nsc__btn",
                                                    dir: "auto",
                                                    onclick: move |_| {
                                                        if let Some(handler) = on_click {
                                                            handler.call(());
                                                        }
                                                    },
                                                    "{label}"
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
        }
    }
}

/// Quiet line art for the top-right corner, fading out towards the text.
#[component]
fn Pattern(kind: NoticePattern) -> Element {
    rsx! {
        svg {
            class: "ui-nsc__pattern",
            view_box: "0 0 220 160",
            fill: "none",
            "aria-hidden": "true",
            {match kind {
                NoticePattern::Orbit => rsx! {
                    circle { cx: "190", cy: "10", r: "46" }
                    circle { cx: "190", cy: "10", r: "86" }
                    circle { cx: "190", cy: "10", r: "128" }
                    path { d: "M60 160 220 0M0 58h220M150 0v160" }
                    circle { cx: "150", cy: "58", r: "3.5", class: "ui-nsc__dot" }
                },
                NoticePattern::Grid => rsx! {
                    for x in [30, 60, 90, 120, 150, 180, 210] {
                        path { key: "x{x}", d: "M{x} 0v160" }
                    }
                    for y in [20, 50, 80, 110, 140] {
                        path { key: "y{y}", d: "M0 {y}h220" }
                    }
                    rect { x: "150", y: "50", width: "30", height: "30", class: "ui-nsc__fill" }
                    circle { cx: "180", cy: "50", r: "3.5", class: "ui-nsc__dot" }
                },
                NoticePattern::Rays => rsx! {
                    for i in 0..8u32 {
                        {
                            let (x, y) = ray_end(i);
                            rsx! { path { key: "{i}", d: "M210 6 L{x} {y}" } }
                        }
                    }
                    path { d: "M120 6a90 90 0 0 0 90 90" }
                    circle { cx: "146", cy: "70", r: "3.5", class: "ui-nsc__dot" }
                },
            }}
        }
    }
}

#[component]
fn CloseIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 16 16",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.1",
            stroke_linecap: "round",
            "aria-hidden": "true",
            path { d: "m4 4 8 8M12 4l-8 8" }
        }
    }
}

#[component]
fn ArrowIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 16 16",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.1",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M5 11 11 5M6 5h5v5" }
        }
    }
}
