//! Outreach fanout lines: one junction point above three destination tiles, with a curve dropping
//! from the point to each tile and a pulse travelling down each curve. It conveys one message going
//! out to several channels. The tiles show icons only; the caller's labels reach assistive tech.
//!
//! Design choice: the drawing space is 100 by 60.6 units, the stage's own 33 to 20 ratio, so circles
//! stay round and the curves keep their shape at every width.
//! Design choice: pulses are SMIL `animateMotion` along each curve, so they follow the curve
//! without a per-frame layout pass; the reduced-motion stylesheet removes them.
//! Design choice: curves fade in on first view rather than drawing a stroke, so the motion stays
//! on opacity and transform.
//! Design choice: a pulse is transparent until its motion starts, because before then it sits at
//! the SVG origin and would show as a stray dot.
//! Design choice: a destination beyond the third is dropped, not drawn.
//! Design choice: the centre tile sits higher than the two outer tiles and carries its own light;
//! the curve to it is straight, because the junction and the tile are on one vertical.
//! Browser checks: pulses run down all three curves; hovering a tile highlights its curve and
//! tile; reduced motion draws no pulses; a fourth destination is not rendered; nothing scrolls
//! sideways at 390 and 320.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/outreach-fanout-lines.css");

/// The layout supports exactly this many destinations.
pub const DESTINATION_COUNT: usize = 3;
/// Seconds between successive pulses leaving the junction.
pub const PULSE_STAGGER_S: f64 = 0.6;
/// Seconds one pulse takes to travel a curve.
pub const PULSE_TRAVEL_S: f64 = 2.4;

/// Horizontal centre of each destination tile, in percent of the stage width.
pub const DESTINATION_CENTRE_X: [f64; DESTINATION_COUNT] = [34.7, 50.0, 65.3];
/// Side of each destination tile, in percent of the stage width. The centre tile is the larger.
pub const DESTINATION_SIDE: [f64; DESTINATION_COUNT] = [9.6, 11.2, 9.6];
/// Top edge of each destination tile, in percent of the stage height. Tiles share one bottom edge.
pub const DESTINATION_TOP: [f64; DESTINATION_COUNT] = [52.9, 49.8, 52.9];
/// Bottom edge shared by the three tiles, in percent of the stage height.
pub const TILE_BOTTOM: f64 = 68.7;
/// The junction: a bare point at 50% of the width and 29% of the height, in percent units.
pub const BRANCH_POINT: (f64, f64) = (50.0, 29.0);

/// Height of the drawing in its own units when the width is 100: the stage is 33 to 20 (1.65 to 1),
/// so its drawing units are 100 wide and 60.6 tall. Percent-of-height values are scaled to these.
pub const VIEW_HEIGHT: f64 = 60.6;

fn unit_y(pct_of_height: f64) -> f64 {
    pct_of_height * VIEW_HEIGHT / 100.0
}

/// Path of the curve to destination `index`, from the junction to the top centre of its tile. The
/// centre curve is straight; the outer curves leave outward and arrive vertically.
pub fn branch_path(index: usize) -> String {
    let x = DESTINATION_CENTRE_X[index];
    let top = unit_y(DESTINATION_TOP[index]);
    let (bx, by) = (BRANCH_POINT.0, unit_y(BRANCH_POINT.1));
    if index == DESTINATION_COUNT / 2 {
        return format!("M {bx} {by} L {x} {top}");
    }
    let outward = (x - bx).signum() * 8.0;
    format!(
        "M {bx} {by} C {} {}, {x} {}, {x} {top}",
        bx + outward,
        by + unit_y(4.0),
        top - unit_y(12.9)
    )
}

/// Start time of pulse `index`, in seconds from first paint.
pub fn pulse_begin(index: usize) -> f64 {
    index as f64 * PULSE_STAGGER_S
}

/// Destinations the layout draws, in order; anything past the third is dropped.
pub fn shown_destinations<T>(destinations: &[T]) -> &[T] {
    &destinations[..destinations.len().min(DESTINATION_COUNT)]
}

/// A destination tile: its icon, and the label assistive tech reads for it.
#[derive(Clone, PartialEq)]
pub struct Endpoint {
    pub id: String,
    pub label: String,
    pub icon: Option<Element>,
}

#[component]
pub fn OutreachFanoutLines(
    destinations: Vec<Endpoint>,
    #[props(default, into)] active_destination_id: Option<String>,
    #[props(default)] on_select: Option<EventHandler<String>>,
    #[props(default)] class: String,
) -> Element {
    let mut hovered = use_signal(|| None::<usize>);
    let shown = shown_destinations(&destinations);
    let interactive = on_select.is_some();
    let class = class_list("kit-fan", &class);
    let (bx, by) = (BRANCH_POINT.0, unit_y(BRANCH_POINT.1));
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class, "aria-hidden": (!interactive).then_some("true"),
            div { class: "kit-fan__stage",
                svg { class: "kit-fan__svg", "viewBox": "0 0 100 {VIEW_HEIGHT}", "preserveAspectRatio": "none",
                    "focusable": "false", "aria-hidden": "true",
                    circle { class: "kit-fan__junction", "cx": "{bx}", "cy": "{by}", "r": "0.5" }
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
                                circle { class: "kit-fan__pulse", "r": "1.4", "opacity": "0",
                                    animateMotion {
                                        "dur": "{dur}",
                                        "begin": "{begin}",
                                        "repeatCount": "indefinite",
                                        "path": "{d}",
                                    }
                                    animate {
                                        "attributeName": "opacity",
                                        "values": "0;0.85;0.85;0",
                                        "keyTimes": "0;0.08;0.9;1",
                                        "dur": "{dur}",
                                        "begin": "{begin}",
                                        "repeatCount": "indefinite",
                                    }
                                }
                            }
                        }
                    }
                }
                for (index, destination) in shown.iter().enumerate() {
                    {
                        let x = DESTINATION_CENTRE_X[index];
                        let side = DESTINATION_SIDE[index];
                        let is_active = active_destination_id.as_deref() == Some(destination.id.as_str());
                        let style = format!(
                            "left: {x}%; top: {TILE_BOTTOM}%; width: {side}%; animation-delay: {}s;",
                            pulse_begin(index) + PULSE_TRAVEL_S
                        );
                        let state = if is_active { "active" } else { "rest" };
                        let icon = destination.icon.clone();
                        let label = destination.label.clone();
                        let id = destination.id.clone();
                        let centre = index == DESTINATION_COUNT / 2;
                        let content = rsx! {
                            if centre {
                                span { class: "kit-fan__light", "aria-hidden": "true" }
                            }
                            span { class: "kit-fan__tile-icon", {icon} }
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
}
