//! Three overlapping browser windows; the front one is active, with coloured dots, an outline
//! and a glow. Hovering a window behind it moves the highlight to that one. Drawn on a
//! 360 × 240 viewBox and faded out towards the right and bottom edges.
//!
//! Port losses: none. The highlight follows the pointer through `:has()` in the stylesheet.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/tab-overload.css");

/// Top-left corner of each window: (number, x, y).
const WINDOWS: [(u8, f64, f64); 3] = [(1, 30.0, 68.0), (2, 125.0, 56.0), (3, 220.0, 44.0)];

#[component]
pub fn TabOverload(
    #[props(default, into)] class: String,
    #[props(default = "Three overlapping browser windows full of placeholder content".to_string(), into)]
    label: String,
) -> Element {
    let id = use_hook(|| format!("ui-to{}", next_id()));
    let class = class_list("ui-to", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class, role: "img", "aria-label": "{label}",
            svg { class: "ui-to-art", "viewBox": "0 0 360 240", "aria-hidden": "true",
                defs {
                    linearGradient { id: "{id}-blob-h", x1: "0%", y1: "0%", x2: "100%", y2: "0%",
                        stop { offset: "0%", "stopColor": "#FFB89E", "stopOpacity": "0.3" }
                        stop { offset: "50%", "stopColor": "#C8A8E5", "stopOpacity": "0.3" }
                        stop { offset: "100%", "stopColor": "#A8B5FF", "stopOpacity": "0.3" }
                    }
                    linearGradient { id: "{id}-blob-v", x1: "0%", y1: "100%", x2: "0%", y2: "0%",
                        stop { offset: "0%", "stopColor": "#FF6B5C", "stopOpacity": "0.3" }
                        stop { offset: "100%", "stopColor": "#FFB89E", "stopOpacity": "0.3" }
                    }
                    filter { id: "{id}-blur", x: "-200%", y: "-200%", width: "500%", height: "500%",
                        feGaussianBlur { "stdDeviation": "12" }
                    }
                    linearGradient { id: "{id}-stroke", x1: "100%", y1: "0%", x2: "0%", y2: "100%",
                        stop { offset: "0%", "stopColor": "#B5C0FF" }
                        stop { offset: "40%", "stopColor": "#D5BBEF" }
                        stop { offset: "75%", "stopColor": "#FFC5AB" }
                        stop { offset: "100%", "stopColor": "#FFB5BC" }
                    }
                }

                for (n, x, y, blob_x, blob_y, side_x, side_y) in WINDOWS.map(|(n, x, y)| (n, x, y, x + 70.0, y + 8.0, x + 8.0, y + 96.0)) {
                    g { key: "{n}",
                        g { class: "ui-to-blobs ui-to-blobs-{n}",
                            ellipse { cx: "{blob_x}", cy: "{blob_y}", rx: "90", ry: "14", fill: "url(#{id}-blob-h)", filter: "url(#{id}-blur)" }
                            ellipse { cx: "{side_x}", cy: "{side_y}", rx: "14", ry: "90", fill: "url(#{id}-blob-v)", filter: "url(#{id}-blur)" }
                        }
                        g { class: "ui-to-tab ui-to-tab-{n}", transform: "translate({x} {y})",
                            rect { class: "ui-to-frame", width: "380", height: "300", rx: "6", fill: "#F4F5F7", stroke: "#E4E7EC", "strokeWidth": "0.7" }
                            circle { class: "ui-to-dot", cx: "14", cy: "14", r: "3", fill: "#CCCCCC" }
                            circle { class: "ui-to-dot", cx: "24", cy: "14", r: "3", fill: "#CCCCCC" }
                            circle { class: "ui-to-dot", cx: "34", cy: "14", r: "3", fill: "#CCCCCC" }
                            rect { class: "ui-to-page", x: "4", y: "24", width: "372", height: "272", rx: "5", fill: "#FFFFFF", stroke: "#E4E7EC", "strokeWidth": "0.7" }
                        }
                        g { class: "ui-to-content ui-to-content-{n}", transform: "translate({x} {y})",
                            rect { class: "ui-to-bar", x: "12", y: "36", width: "30", height: "30", rx: "4", fill: "#EFF1F4" }
                            rect { class: "ui-to-bar", x: "52", y: "38", width: "100", height: "6", rx: "2", fill: "#EFF1F4" }
                            rect { class: "ui-to-bar", x: "52", y: "46", width: "100", height: "6", rx: "2", fill: "#EFF1F4" }
                            rect { class: "ui-to-bar", x: "52", y: "54", width: "35", height: "6", rx: "2", fill: "#EFF1F4" }
                            rect { class: "ui-to-bar", x: "12", y: "72", width: "140", height: "6", rx: "2", fill: "#EFF1F4" }
                            rect { class: "ui-to-bar", x: "12", y: "80", width: "140", height: "6", rx: "2", fill: "#EFF1F4" }
                            rect { class: "ui-to-bar", x: "12", y: "88", width: "55", height: "6", rx: "2", fill: "#EFF1F4" }
                        }
                    }
                }

                for (n, x, y, dot_y, dot_1, dot_2, dot_3, curve_y, curve_x, top_end) in WINDOWS.map(|(n, x, y)| {
                    let top_end = if n == 3 { 360.0 } else { x + 95.0 };
                    (n, x, y, y + 14.0, x + 14.0, x + 24.0, x + 34.0, y + 6.0, x + 6.0, top_end)
                }) {
                    g { key: "front-{n}", class: "ui-to-front ui-to-front-{n}",
                        circle { cx: "{dot_1}", cy: "{dot_y}", r: "3", fill: "#FF6B5C" }
                        circle { cx: "{dot_2}", cy: "{dot_y}", r: "3", fill: "#FFB89E" }
                        circle { cx: "{dot_3}", cy: "{dot_y}", r: "3", fill: "#A8B5FF" }
                        path {
                            d: "M {x} 240 L {x} {curve_y} A 6 6 0 0 1 {curve_x} {y} L {top_end} {y}",
                            fill: "none",
                            stroke: "url(#{id}-stroke)",
                            "strokeWidth": "0.8",
                        }
                    }
                }
            }
        }
    }
}
