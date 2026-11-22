//! Outreach fanout lines: one outbound source branching to three destination tools, with pulses
//! travelling down each branch. It conveys one message going out to several channels.
//!
//! Design choice: the drawing uses a square 100-unit space, so its geometry scales with the
//! container and the branch curves keep their shape at every width.
//! Design choice: pulses are SMIL `animateMotion` along each branch path, so they follow the curve
//! without a per-frame layout pass; the reduced-motion stylesheet removes them.
//! Design choice: branches fade in on first view rather than drawing a stroke, so the motion stays
//! on opacity and transform.
//! Design choice: a destination beyond the third is dropped, not drawn.
//! Browser checks: pulses run down all three branches; hovering a tile highlights its branch and
//! tile; reduced motion draws no pulses; a fourth destination is not rendered; nothing scrolls
//! sideways at 390 and 320.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/outreach-fanout-lines.css");

/// The layout supports exactly this many destinations.
pub const DESTINATION_COUNT: usize = 3;
/// Seconds between successive pulses leaving the source.
pub const PULSE_STAGGER_S: f64 = 0.6;
/// Seconds one pulse takes to travel a branch.
pub const PULSE_TRAVEL_S: f64 = 2.4;

/// Centre of each destination tile in the 100-unit drawing space.
pub const DESTINATION_CENTRES: [(f64, f64); DESTINATION_COUNT] =
    [(16.0, 84.0), (50.0, 84.0), (84.0, 84.0)];
/// Where the trunk meets the branches.
pub const BRANCH_POINT: (f64, f64) = (50.0, 40.0);

/// Path of the branch to destination `index`, from the branch point to the top of its tile.
pub fn branch_path(index: usize) -> String {
    let (x, _) = DESTINATION_CENTRES[index];
    let (bx, by) = BRANCH_POINT;
    format!("M {bx} {by} C {bx} 56, {x} 60, {x} 72")
}

/// Path of the trunk, from the bottom of the source tile to the branch point.
pub fn trunk_path() -> String {
    let (bx, by) = BRANCH_POINT;
    format!("M {bx} 22 L {bx} {by}")
}

/// Start time of pulse `index`, in seconds from first paint.
pub fn pulse_begin(index: usize) -> f64 {
    index as f64 * PULSE_STAGGER_S
}

/// Destinations the layout draws, in order; anything past the third is dropped.
pub fn shown_destinations<T>(destinations: &[T]) -> &[T] {
    &destinations[..destinations.len().min(DESTINATION_COUNT)]
}

/// A node of the fanout: the source or one destination.
#[derive(Clone, PartialEq)]
pub struct Endpoint {
    pub id: String,
    pub label: String,
    pub icon: Option<Element>,
}

#[component]
pub fn OutreachFanoutLines(
    source: Endpoint,
    destinations: Vec<Endpoint>,
    #[props(default, into)] active_destination_id: Option<String>,
    #[props(default)] on_select: Option<EventHandler<String>>,
    #[props(default)] class: String,
) -> Element {
    let mut hovered = use_signal(|| None::<usize>);
    let shown = shown_destinations(&destinations);
    let interactive = on_select.is_some();
    let class = class_list("kit-fan", &class);
    let source_icon = source.icon.clone();
    let source_label = source.label.clone();
    let trunk = trunk_path();
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class, "aria-hidden": (!interactive).then_some("true"),
            svg { class: "kit-fan__svg", "viewBox": "0 0 100 100", "preserveAspectRatio": "none",
                "focusable": "false", "aria-hidden": "true",
                path { class: "kit-fan__trunk", d: "{trunk}" }
                for index in 0..shown.len() {
                    {
                        let is_active = active_destination_id.as_deref() == Some(shown[index].id.as_str());
                        let is_hot = hovered() == Some(index);
                        let branch_class = if is_active {
                            "kit-fan__branch is-active"
                        } else if is_hot {
                            "kit-fan__branch is-hot"
                        } else {
                            "kit-fan__branch"
                        };
                        let d = branch_path(index);
                        let begin = format!("{}s", pulse_begin(index));
                        let dur = format!("{PULSE_TRAVEL_S}s");
                        rsx! {
                            path { class: branch_class, d: "{d}" }
                            circle { class: "kit-fan__pulse", "r": "1.4",
                                animateMotion {
                                    "dur": "{dur}",
                                    "begin": "{begin}",
                                    "repeatCount": "indefinite",
                                    "path": "{d}",
                                }
                            }
                        }
                    }
                }
            }
            div { class: "kit-fan__tile kit-fan__tile--source", "data-state": "source",
                span { class: "kit-fan__tile-icon", {source_icon} }
                span { class: "kit-fan__tile-label", "{source_label}" }
            }
            for (index, destination) in shown.iter().enumerate() {
                {
                    let (x, y) = DESTINATION_CENTRES[index];
                    let is_active = active_destination_id.as_deref() == Some(destination.id.as_str());
                    let style = format!("left: {x}%; top: {y}%; animation-delay: {}s;", pulse_begin(index) + PULSE_TRAVEL_S);
                    let state = if is_active { "active" } else { "rest" };
                    let icon = destination.icon.clone();
                    let label = destination.label.clone();
                    let id = destination.id.clone();
                    let content = rsx! {
                        span { class: "kit-fan__tile-icon", {icon} }
                        span { class: "kit-fan__tile-label", "{label}" }
                    };
                    rsx! {
                        div { class: "kit-fan__tile", style: "{style}", "data-index": "{index}", "data-state": state,
                            onmouseenter: move |_| hovered.set(Some(index)),
                            onmouseleave: move |_| hovered.set(None),
                            if interactive {
                                button { r#type: "button", class: "kit-fan__button",
                                    "aria-label": label.clone(),
                                    onclick: move |_| {
                                        if let Some(handler) = on_select {
                                            handler.call(id.clone());
                                        }
                                    },
                                    {content}
                                }
                            } else {
                                {content}
                            }
                        }
                    }
                }
            }
        }
    }
}
