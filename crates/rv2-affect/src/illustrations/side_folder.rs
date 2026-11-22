//! A glassy folder turned on its side. Three documents peek out from behind a frosted front
//! pocket with a tab on its edge. Hovering the folder swings the pocket forward and slides
//! each document out on its own. Keeps a 3 : 2 frame.
//!
//! Port losses: none. The pocket's shape is a CSS mask (a data URI in the stylesheet), which
//! also clips the frosted blur to the pocket outline.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/side-folder.css");

/// Front pocket on a 380 × 520 grid: a narrow top part and a wider tab towards the bottom.
const FRONT: &str = "M40 0H290Q330 0 330 40V318Q330 338 344 348L364 362Q380 372 380 396V480Q380 520 340 520H40Q0 520 0 480V40Q0 0 40 0Z";

/// Widths of the seven text lines on each document.
const LINES: [f64; 7] = [78.0, 180.0, 140.0, 175.0, 126.0, 168.0, 110.0];

#[component]
pub fn SideFolder(
    #[props(default, into)] class: String,
    #[props(default = "Blue folder on its side, with three documents peeking out behind a frosted front pocket".to_string(), into)]
    label: String,
) -> Element {
    let id = use_hook(|| format!("ui-sf{}", next_id()));
    let class = class_list("ui-sf", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class, role: "img", "aria-label": "{label}",
            div { class: "ui-sf-icon", "aria-hidden": "true",
                svg { class: "ui-sf-layer ui-sf-back", "viewBox": "0 0 440 520",
                    defs {
                        filter { id: "{id}-hl", x: "-10%", y: "-10%", width: "120%", height: "120%",
                            feGaussianBlur { "in": "SourceAlpha", "stdDeviation": "4", result: "blur" }
                            feOffset { "in": "blur", dy: "3", result: "offset" }
                            feComposite { "in": "blur", "in2": "offset", operator: "arithmetic", k2: "1", k3: "-1", result: "diff" }
                            feFlood { "floodColor": "#fff", "floodOpacity": "0.65" }
                            feComposite { "in2": "diff", operator: "in", result: "light" }
                            feComposite { "in": "light", "in2": "SourceAlpha", operator: "in" }
                        }
                    }
                    rect { x: "0", y: "18", width: "440", height: "486", rx: "36", fill: "#5bb7ea" }
                    rect { x: "0", y: "18", width: "440", height: "486", rx: "36", fill: "#000", filter: "url(#{id}-hl)" }
                }

                svg { class: "ui-sf-layer", "viewBox": "0 0 440 520",
                    defs {
                        linearGradient { id: "{id}-paper", x1: "0", y1: "0", x2: "0", y2: "1",
                            stop { offset: "0%", "stopColor": "#fff" }
                            stop { offset: "100%", "stopColor": "#eaf2f8" }
                        }
                        symbol { id: "{id}-doc", overflow: "visible",
                            rect { x: "0.6", y: "0.6", width: "278.8", height: "348.8", rx: "19.4", fill: "url(#{id}-paper)", stroke: "rgba(0,0,0,0.12)", "strokeWidth": "1.2" }
                            for (i, width, top) in LINES.iter().copied().enumerate().map(|(i, width)| (i, width, 87.0 + i as f64 * 28.0)) {
                                rect { key: "{i}", x: "30", y: "{top}", width: "{width}", height: "9", rx: "4.5", fill: "rgba(0,0,0,0.12)" }
                            }
                        }
                    }
                    g { transform: "scale(1.15)",
                        r#use { class: "ui-sf-doc ui-sf-doc--top", href: "#{id}-doc" }
                        r#use { class: "ui-sf-doc ui-sf-doc--bottom", href: "#{id}-doc" }
                        r#use { class: "ui-sf-doc ui-sf-doc--middle", href: "#{id}-doc" }
                    }
                }

                div { class: "ui-sf-front",
                    svg { "viewBox": "0 0 380 520",
                        path { d: FRONT, fill: "none", stroke: "rgba(255,255,255,0.3)", "strokeWidth": "4" }
                    }
                }
            }
        }
    }
}
