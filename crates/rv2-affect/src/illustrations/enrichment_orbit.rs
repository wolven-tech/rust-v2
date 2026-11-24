//! A Grok tile inside two rings. Coloured comets run around the rings in opposite directions,
//! and three data nodes orbit the outer ring, staying upright as they travel. Drawn on a
//! 320 × 240 viewBox inside a 3 : 2 frame.
//!
//! Port losses: none. The motion is CSS keyframes.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/enrichment-orbit.css");

/// Start offsets of the three orbiting nodes (120° apart on a 22 s orbit).
const NODE_DELAYS: [&str; 3] = ["0s", "-7.33s", "-14.66s"];

#[component]
pub fn EnrichmentOrbit(
    #[props(default, into)] class: String,
    #[props(default = "A Grok tile with data nodes orbiting around it".to_string(), into)]
    label: String,
) -> Element {
    let id = use_hook(|| format!("ui-eo{}", next_id()));
    let class = class_list("ui-eo", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class, role: "img", "aria-label": "{label}",
            svg { class: "ui-eo-art", "viewBox": "0 0 320 240", "aria-hidden": "true",
                defs {
                    filter { id: "{id}-tile", x: "-30%", y: "-30%", width: "160%", height: "160%",
                        feGaussianBlur { "in": "SourceAlpha", "stdDeviation": "4", result: "dropBlur" }
                        feOffset { "in": "dropBlur", dx: "0", dy: "3", result: "dropOffset" }
                        feFlood { class: "ui-eo-drop", "flood-color": "#000000", "flood-opacity": "0.06", result: "dropColor" }
                        feComposite { "in": "dropColor", "in2": "dropOffset", operator: "in", result: "dropShadow" }
                        feOffset { "in": "SourceAlpha", dx: "0", dy: "-4", result: "alphaUp" }
                        feComposite { "in": "SourceAlpha", "in2": "alphaUp", operator: "out", result: "sliver" }
                        feFlood { class: "ui-eo-sliver", "flood-color": "#EAECEF", "flood-opacity": "1", result: "innerColor" }
                        feComposite { "in": "innerColor", "in2": "sliver", operator: "in", result: "innerShadow" }
                        feMerge {
                            feMergeNode { "in": "dropShadow" }
                            feMergeNode { "in": "SourceGraphic" }
                            feMergeNode { "in": "innerShadow" }
                        }
                    }
                    filter { id: "{id}-node", x: "-60%", y: "-60%", width: "220%", height: "220%",
                        feGaussianBlur { "in": "SourceAlpha", "stdDeviation": "2.5", result: "dropBlur" }
                        feOffset { "in": "dropBlur", dx: "0", dy: "2", result: "dropOffset" }
                        feFlood { class: "ui-eo-drop", "flood-color": "#000000", "flood-opacity": "0.07", result: "dropColor" }
                        feComposite { "in": "dropColor", "in2": "dropOffset", operator: "in", result: "dropShadow" }
                        feOffset { "in": "SourceAlpha", dx: "0", dy: "-3", result: "alphaUp" }
                        feComposite { "in": "SourceAlpha", "in2": "alphaUp", operator: "out", result: "sliver" }
                        feFlood { class: "ui-eo-sliver", "flood-color": "#EAECEF", "flood-opacity": "1", result: "innerColor" }
                        feComposite { "in": "innerColor", "in2": "sliver", operator: "in", result: "innerShadow" }
                        feMerge {
                            feMergeNode { "in": "dropShadow" }
                            feMergeNode { "in": "SourceGraphic" }
                            feMergeNode { "in": "innerShadow" }
                        }
                    }
                    linearGradient { id: "{id}-comet",
                        stop { offset: "0%", "stop-color": "#FF6B5C", "stop-opacity": "0" }
                        stop { offset: "22%", "stop-color": "#FFB89E", "stop-opacity": "0.9" }
                        stop { offset: "50%", "stop-color": "#C8A8E5", "stop-opacity": "0.9" }
                        stop { offset: "78%", "stop-color": "#A8B5FF", "stop-opacity": "0.9" }
                        stop { offset: "100%", "stop-color": "#A8B5FF", "stop-opacity": "0" }
                    }
                }

                g { class: "ui-eo-rings", fill: "none", stroke: "#E4E7EC", "stroke-width": "1.2",
                    circle { cx: "160", cy: "120", r: "56", stroke_opacity: "0.45" }
                    circle { cx: "160", cy: "120", r: "80" }
                }

                g { class: "ui-eo-spin",
                    path { fill: "none", stroke: "url(#{id}-comet)", "stroke-width": "1.6", "stroke-linecap": "round", d: "M114.1 54.5 A80 80 0 0 1 205.9 54.5" }
                }
                g { class: "ui-eo-spin ui-eo-reverse",
                    path { fill: "none", stroke: "url(#{id}-comet)", "stroke-width": "1.6", "stroke-linecap": "round", d: "M120.4 80.4 A56 56 0 0 1 199.6 80.4" }
                }

                rect { class: "ui-eo-tile", x: "134", y: "94", width: "52", height: "52", rx: "12", fill: "#FFFFFF", stroke: "#E4E7EC", "stroke-width": "0.5", filter: "url(#{id}-tile)" }
                svg { x: "147", y: "107", width: "26", height: "26", "viewBox": "0 0 33.4 32", overflow: "visible",
                    path { class: "ui-eo-mark", fill: "#171717", d: GROK_TOP }
                    path { class: "ui-eo-mark", fill: "#171717", d: GROK_BOTTOM }
                }

                for delay in NODE_DELAYS {
                    g { class: "ui-eo-orbit", style: "animation-delay: {delay}",
                        g { class: "ui-eo-upright", style: "animation-delay: {delay}",
                            circle { class: "ui-eo-tile", cx: "160", cy: "40", r: "17", fill: "#FFFFFF", stroke: "#E4E7EC", "stroke-width": "0.5", filter: "url(#{id}-node)" }
                            g { transform: "translate(154 34) scale(0.0288)", class: "ui-eo-icon", fill: "#6B6F76", fill_opacity: "0.55",
                                path { d: ORBIT_ICON }
                            }
                        }
                    }
                }
            }
        }
    }
}

const ORBIT_ICON: &str = "M278 88C278 115.614 300.386 138 328 138H366C393.614 138 416 160.386 416 188V228C416 255.614 393.614 278 366 278H328C300.386 278 278 300.386 278 328V366C278 393.614 255.614 416 228 416H188C160.386 416 138 393.614 138 366V328C138 300.386 115.614 278 88 278H50C22.3858 278 0 255.614 0 228V188C0 160.386 22.3858 138 50 138H88C115.614 138 138 115.614 138 88V50C138 22.3858 160.386 0 188 0H228C255.614 0 278 22.3858 278 50V88ZM140 226C140 253.614 162.386 276 190 276H226C253.614 276 276 253.614 276 226V190C276 162.386 253.614 140 226 140H190C162.386 140 140 162.386 140 190V226Z";
const GROK_TOP: &str = "M12.8734 20.5407L23.9549 12.3506C24.4982 11.9491 25.2747 12.1057 25.5336 12.7294C26.896 16.0185 26.2873 19.9712 23.5766 22.6851C20.866 25.3989 17.0944 25.9941 13.6471 24.6386L9.88123 26.3843C15.2826 30.0806 21.8416 29.1665 25.9403 25.0601C29.1914 21.8051 30.1983 17.3683 29.2568 13.3673L29.2653 13.3758C27.9 7.49809 29.601 5.14871 33.0853 0.344576C33.1677 0.230667 33.2502 0.116757 33.3327 0L28.7476 4.59055V4.57631L12.8706 20.5436";
const GROK_BOTTOM: &str = "M10.5867 22.5312C6.70979 18.8234 7.37821 13.0852 10.6862 9.77618C13.1323 7.3271 17.14 6.32755 20.6385 7.79698L24.3959 6.05986C23.719 5.57005 22.8514 5.04322 21.8559 4.67301C17.3562 2.81914 11.969 3.7418 8.31115 7.40114C4.79271 10.9238 3.68626 16.3402 5.58628 20.9621C7.0056 24.4164 4.67893 26.8597 2.3352 29.3259C1.50465 30.2001 0.67126 31.0744 0 31.9999L10.5838 22.534";
