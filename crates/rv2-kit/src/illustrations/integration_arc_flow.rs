//! Integration arc flow: several connected tools feed data into one central analysis hub. It tells a
//! visitor that their tools' data gathers in one place.
//!
//! Design choice: the arc is laid out in container-width units (`cqw`), so pills, connectors and the
//! hub scale together with no measurement.
//! Design choice: the arc turns one slot every `ROTATE_MS` by a timer, and pills and connectors move
//! by transform. A paused pill keeps its slot in the turn (so the arc stays full) and fades; it does
//! not stop turning.
//! Design choice: the timer is not started when the viewer prefers reduced motion; the arc stays in
//! its first arrangement and no pulses are drawn.
//! Design choice: pills are buttons only when `on_toggle` is given; otherwise they highlight on hover
//! or click and are not focusable.
//! Browser checks: each live connector shows pulses toward the hub; hovering a pill highlights it and
//! its connector; paused pills show no pulses and are faded; reduced motion draws no pulses; at 390 and
//! 320 the arc becomes a stack with no sideways scroll.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, prefers_motion_script, sleep};

const CSS: Asset = asset!("/assets/integration-arc-flow.css");

/// Milliseconds between one turn of the arc by a slot.
pub const ROTATE_MS: u32 = 3200;

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
    pub sub_label: String,
    pub status: IntegrationStatus,
    pub logo: Option<Element>,
}

/// Angle in degrees of the arc slot `slot` of `count`: the first slot is at the left end (165), the
/// last at the right end (15), and the front (90) sits between them.
pub fn slot_angle(slot: usize, count: usize) -> f64 {
    if count <= 1 {
        90.0
    } else {
        165.0 - slot as f64 * 150.0 / (count - 1) as f64
    }
}

/// Pill centre for a slot, in cqw, with the hub centred at (50, 60).
pub fn slot_point(slot: usize, count: usize) -> (f64, f64) {
    let angle = slot_angle(slot, count).to_radians();
    (50.0 + 36.0 * angle.cos(), 60.0 - 36.0 * angle.sin())
}

/// Connector geometry from a pill centre to the hub's top centre: (x, y, angle in degrees, length in
/// cqw).
pub fn connector(slot: usize, count: usize) -> (f64, f64, f64, f64) {
    let (x, y) = slot_point(slot, count);
    let (tx, ty) = (50.0, 56.0);
    let (dx, dy) = (tx - x, ty - y);
    (x, y, dy.atan2(dx).to_degrees(), (dx * dx + dy * dy).sqrt())
}

/// The slot an integration occupies after `turn` steps.
pub fn slot_for(index: usize, count: usize, turn: usize) -> usize {
    if count == 0 {
        0
    } else {
        (index + turn) % count
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
                sleep(ROTATE_MS).await;
                turn += 1;
            }
        });
    });
    let count = integrations.len();
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
            div { class: "kit-arc__connectors", "aria-hidden": "true",
                for (index, item) in integrations.iter().enumerate() {
                    {
                        let slot = slot_for(index, count, turn());
                        let (x, y, angle, len) = connector(slot, count);
                        let style = format!(
                            "--x: {x:.2}; --y: {y:.2}; --angle: {angle:.2}deg; --len: {:.4};",
                            len / 100.0,
                        );
                        let highlighted = lit.as_deref() == Some(item.id.as_str());
                        rsx! {
                            span { class: "kit-arc__connector", style: "{style}",
                                "data-status": item.status.as_str(),
                                "data-lit": highlighted.then_some("true"),
                                "data-id": item.id.clone(),
                                span { class: "kit-arc__pulse" }
                            }
                        }
                    }
                }
            }
            div { class: "kit-arc__pills",
                for (index, item) in integrations.iter().enumerate() {
                    {
                        let slot = slot_for(index, count, turn());
                        let (x, y) = slot_point(slot, count);
                        let style = format!("--x: {x:.2}; --y: {y:.2};");
                        let highlighted = lit.as_deref() == Some(item.id.as_str());
                        let is_active = active_id.as_deref() == Some(item.id.as_str());
                        let aria = format!("{} integration, {}", item.label, item.status.word());
                        let id = item.id.clone();
                        let content = rsx! {
                            span { class: "kit-arc__logo", {item.logo.clone()} }
                            span { class: "kit-arc__text",
                                span { class: "kit-arc__name", {item.label.clone()} }
                                span { class: "kit-arc__sub", {item.sub_label.clone()} }
                            }
                        };
                        rsx! {
                            div { class: "kit-arc__pill", key: "{id}", style: "{style}",
                                "data-status": item.status.as_str(),
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
            div { class: "kit-arc__hub",
                span { class: "kit-arc__hub-title", "{hub_title}" }
                span { class: "kit-arc__hub-sub", "{hub_subtitle}" }
            }
        }
    }
}
