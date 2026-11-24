//! Integration arc flow: connected tools sit on one vertical spine beside a hub card. The centre pill
//! is the active one. After each turn a beam sweeps from it to the hub and the hub brightens. At rest
//! no beam is drawn. It tells a visitor that their tools' data gathers in one place.
//!
//! Design choice: every position is a percent of the stage (the card), so the hub, pills and dashed
//! guide keep their proportions at any width. The spine is one column of five slots, two above and
//! two below the centre, and the outer slots run past the stage edges.
//! Design choice: the spine turns one slot every `ROTATE_MS` by a timer, and pills move by `top` on
//! their own transition. The pill that arrives at the centre slot becomes the active one and drives the
//! beam. A paused pill keeps its slot in the turn and fades; it does not stop turning.
//! Design choice: the timer is not started when the viewer prefers reduced motion; the spine stays in
//! its first arrangement and no pulses are drawn.
//! Design choice: pills are buttons only when `on_toggle` is given; otherwise they are not focusable.
//! Browser checks: the centre pill is full strength and the others fade away from it; the beam joins
//! the centre pill to the hub with no gap; reduced motion draws no pulses; at 390 and 320 nothing scrolls
//! sideways.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, prefers_motion_script, sleep};

const CSS: Asset = asset!("/assets/integration-arc-flow.css");

/// Milliseconds between one turn of the spine by a slot.
pub const ROTATE_MS: u32 = 3200;
/// Milliseconds the beam stays drawn after a turn, while it sweeps to the hub.
pub const SWEEP_MS: u32 = 900;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IntegrationStatus {
    #[default]
    Live,
    Paused,
}

impl IntegrationStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Paused => "paused",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Paused => "paused",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Hub {
    pub title: String,
    pub subtitle: String,
}

#[derive(Clone, PartialEq)]
pub struct Integration {
    pub id: String,
    pub label: String,
    pub status: IntegrationStatus,
    pub logo: Option<Element>,
}

/// The slot an integration occupies after `turn` steps.
pub fn slot_for(index: usize, count: usize, turn: usize) -> usize {
    if count == 0 {
        0
    } else {
        (index + turn) % count
    }
}

/// Offset of a slot from the centre of the spine: negative above, positive below, 0 at the centre.
pub fn slot_offset(slot: usize, count: usize) -> i64 {
    slot as i64 - (count / 2) as i64
}

/// Opacity of a pill `offset` slots from the centre: full at the centre, about half one slot away,
/// about a third two slots away.
pub fn spine_opacity(offset: i64) -> f64 {
    match offset.unsigned_abs() {
        0 => 1.0,
        1 => 0.5,
        _ => 0.33,
    }
}

/// Left edge of a pill, in percent of the stage width. The column bows: the centre pill sits
/// furthest left and the outer slots step right.
pub fn pill_left_pct(offset: i64) -> f64 {
    match offset.unsigned_abs() {
        0 => 56.7,
        1 => 61.0,
        _ => 71.0,
    }
}

#[component]
pub fn IntegrationArcFlow(
    hub: Hub,
    integrations: Vec<Integration>,
    #[props(default, into)] active_id: Option<String>,
    #[props(default)] on_toggle: Option<EventHandler<String>>,
    #[props(default, into)] label: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let mut turn = use_signal(|| 0usize);
    let mut sweeping = use_signal(|| false);
    let mut hovered = use_signal(|| None::<String>);
    let mut picked = use_signal(|| None::<String>);
    use_hook(move || {
        spawn(async move {
            let allowed = document::eval(&format!("return {};", prefers_motion_script()))
                .await
                .ok()
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if !allowed {
                return;
            }
            loop {
                sleep(ROTATE_MS - SWEEP_MS).await;
                turn += 1;
                sweeping.set(true);
                sleep(SWEEP_MS).await;
                sweeping.set(false);
            }
        });
    });
    let count = integrations.len();
    let centre_slot = count / 2;
    let centre_status = integrations
        .iter()
        .enumerate()
        .find(|(index, _)| slot_for(*index, count, turn()) == centre_slot)
        .map(|(_, item)| item.status);
    let interactive = on_toggle.is_some();
    let lit = hovered().or_else(|| picked()).or_else(|| active_id.clone());
    let group_label = label.unwrap_or_else(|| "Integrations feeding one analysis".to_string());
    let class = class_list("kit-arc", &class);
    let hub_title = hub.title.clone();
    let hub_subtitle = hub.subtitle.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class, role: "group", "aria-label": "{group_label}",
            span { class: "kit-arc__light", "aria-hidden": "true" }
            div { class: "kit-arc__hub", "data-lit": sweeping().then_some("true"),
                span { class: "kit-arc__hub-text",
                    span { class: "kit-arc__hub-title", "{hub_title}" }
                    span { class: "kit-arc__hub-sub", "{hub_subtitle}" }
                }
                span { class: "kit-arc__hub-mark", "aria-hidden": "true" }
            }
            svg { class: "kit-arc__guide", "aria-hidden": "true", "viewBox": "0 0 100 100", "preserveAspectRatio": "none",
                path { d: "M 72 -3 Q 50 50 72 103", fill: "none", "vector-effect": "non-scaling-stroke" }
            }
            if sweeping() {
                if let Some(status) = centre_status {
                    span { class: "kit-arc__beam", "aria-hidden": "true", "data-status": status.as_str(),
                        span { class: "kit-arc__pulse" }
                    }
                }
            }
            div { class: "kit-arc__pills",
                for (index, item) in integrations.iter().enumerate() {
                    {
                        let slot = slot_for(index, count, turn());
                        let offset = slot_offset(slot, count);
                        let style = format!(
                            "--s: {offset}; --o: {:.2}; left: {:.1}%;",
                            spine_opacity(offset),
                            pill_left_pct(offset)
                        );
                        let centred = offset == 0;
                        let highlighted = lit.as_deref() == Some(item.id.as_str());
                        let is_active = active_id.as_deref() == Some(item.id.as_str());
                        let aria = format!("{} integration, {}", item.label, item.status.word());
                        let id = item.id.clone();
                        let content = rsx! {
                            span { class: "kit-arc__logo", {item.logo.clone()} }
                            span { class: "kit-arc__text",
                                span { class: "kit-arc__kind", "Connector" }
                                span { class: "kit-arc__name", {item.label.clone()} }
                            }
                        };
                        rsx! {
                            div { class: "kit-arc__pill", key: "{id}", style: "{style}",
                                "data-status": item.status.as_str(),
                                "data-centre": centred.then_some("true"),
                                "data-lit": highlighted.then_some("true"),
                                "data-active": is_active.then_some("true"),
                                onmouseenter: {
                                    let id = id.clone();
                                    move |_| hovered.set(Some(id.clone()))
                                },
                                onmouseleave: move |_| hovered.set(None),
                                if interactive {
                                    button { r#type: "button", class: "kit-arc__button",
                                        "aria-label": "{aria}",
                                        onfocus: {
                                            let id = id.clone();
                                            move |_| hovered.set(Some(id.clone()))
                                        },
                                        onblur: move |_| hovered.set(None),
                                        onclick: {
                                            let id = id.clone();
                                            move |_| {
                                                if let Some(handler) = on_toggle {
                                                    handler.call(id.clone());
                                                }
                                            }
                                        },
                                        {content}
                                    }
                                } else {
                                    div { class: "kit-arc__static",
                                        onclick: {
                                            let id = id.clone();
                                            move |_| picked.set(Some(id.clone()))
                                        },
                                        {content}
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
