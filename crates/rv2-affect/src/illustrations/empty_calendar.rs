//! Three stacked calendar cards marked "Empty", with a disabled "Join meeting" button.
//! Hover the front card to lift it, a back card to slide it further out. Drawn on a
//! 320 × 240 viewBox inside a 3 : 2 frame.
//!
//! Port losses: none.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/empty-calendar.css");

/// One calendar card: title, "Empty" badge, placeholder lines and a disabled button.
fn calendar_card(shadow: String) -> Element {
    rsx! {
        rect { class: "ui-ec-card", x: "40", y: "20", width: "240", height: "140", rx: "10", fill: "#FFFFFF", stroke: "#E4E7EC", "strokeWidth": "0.5", filter: "{shadow}" }
        text { class: "ui-ec-title", x: "56", y: "48", "fontSize": "13", "fontWeight": "500", fill: "#8A8A8A", "Calendar" }
        g { transform: "translate(218 34)",
            rect { class: "ui-ec-badge", width: "46", height: "18", rx: "5", fill: "#FEF8EF" }
            text { x: "23", y: "12", "textAnchor": "middle", "fontSize": "9", "fontWeight": "600", "letterSpacing": "0.06em", fill: "#FAB082", "EMPTY" }
        }
        rect { class: "ui-ec-bar", x: "56", y: "64", width: "208", height: "7", rx: "2", fill: "#EFF1F4" }
        rect { class: "ui-ec-bar", x: "56", y: "74", width: "208", height: "7", rx: "2", fill: "#EFF1F4" }
        rect { class: "ui-ec-bar", x: "56", y: "84", width: "168", height: "7", rx: "2", fill: "#EFF1F4" }
        g { transform: "translate(56 104)",
            rect { class: "ui-ec-btn", width: "208", height: "40", rx: "8", fill: "#F4F4F6", stroke: "#E4E7EC", "strokeWidth": "0.5" }
            g { class: "ui-ec-btn-ink", transform: "translate(64 14)", fill: "none", stroke: "#9A9AA3", "strokeWidth": "1.3", "strokeLinecap": "round", "strokeLinejoin": "round",
                circle { cx: "6", cy: "6", r: "5" }
                line { x1: "2.5", y1: "9.5", x2: "9.5", y2: "2.5" }
            }
            text { class: "ui-ec-btn-label", x: "84", y: "24", "fontSize": "10", "fontWeight": "500", fill: "#9A9AA3", "Join meeting" }
        }
    }
}

#[component]
pub fn EmptyCalendar(
    #[props(default, into)] class: String,
    #[props(default = "Stacked calendar cards marked empty, with a disabled join meeting button".to_string(), into)]
    label: String,
) -> Element {
    let id = use_hook(|| format!("ui-ec{}", next_id()));
    let class = class_list("ui-ec", &class);
    let shadow = format!("url(#{id}-shadow)");
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class, role: "img", "aria-label": "{label}",
            svg { class: "ui-ec-art", "viewBox": "0 0 320 240", "aria-hidden": "true",
                defs {
                    filter { id: "{id}-blur", x: "-30%", y: "-100%", width: "160%", height: "300%",
                        feGaussianBlur { "stdDeviation": "14" }
                    }
                    filter { id: "{id}-shadow", x: "-20%", y: "-20%", width: "140%", height: "140%",
                        feDropShadow { class: "ui-ec-drop", dx: "0", dy: "3", "stdDeviation": "4", "floodColor": "#000000", "floodOpacity": "0.06" }
                    }
                    linearGradient { id: "{id}-blob", x1: "0%", y1: "0%", x2: "100%", y2: "0%",
                        stop { offset: "0%", "stopColor": "#FFA0AB", "stopOpacity": "0.25" }
                        stop { offset: "33%", "stopColor": "#FFB89E", "stopOpacity": "0.25" }
                        stop { offset: "66%", "stopColor": "#C8A8E5", "stopOpacity": "0.25" }
                        stop { offset: "100%", "stopColor": "#A8B5FF", "stopOpacity": "0.25" }
                    }
                    linearGradient { id: "{id}-stroke", x1: "0%", y1: "0%", x2: "100%", y2: "0%",
                        stop { class: "ui-ec-edge", offset: "0%", "stopColor": "#E4E7EC" }
                        stop { offset: "18%", "stopColor": "#FFB5BC" }
                        stop { offset: "40%", "stopColor": "#FFC5AB" }
                        stop { offset: "60%", "stopColor": "#D5BBEF" }
                        stop { offset: "82%", "stopColor": "#B5C0FF" }
                        stop { class: "ui-ec-edge", offset: "100%", "stopColor": "#E4E7EC" }
                    }
                }

                g { transform: "translate(30 47) scale(0.814)",
                    g { class: "ui-ec-back ui-ec-back-left", {calendar_card(shadow.clone())} }
                    g { class: "ui-ec-back ui-ec-back-right", {calendar_card(shadow.clone())} }
                    g { class: "ui-ec-front",
                        ellipse { class: "ui-ec-blob", cx: "160", cy: "152", rx: "125", ry: "35", fill: "url(#{id}-blob)", filter: "url(#{id}-blur)" }
                        {calendar_card(shadow.clone())}
                        path { d: "M40 150 A 10 10 0 0 0 50 160 L270 160 A 10 10 0 0 0 280 150", fill: "none", stroke: "url(#{id}-stroke)", "strokeWidth": "0.5" }
                    }
                }
            }
        }
    }
}
