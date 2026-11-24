//! Glow floor backdrop: one soft wash of colour rises from the bottom edge of a stage and drifts
//! sideways behind the content of a section. The wash is a single field in two layers; the second
//! layer is a lighter overlay that drifts less.
//!
//! Design choice: the stage is 90% of the card's width and 70% of its height. The field is drawn
//! in stage percent, so the wash keeps its shape at every width.
//! Design choice: the top edge is one path, blurred so it has no edge line; the height of the
//! colour falls off through an alpha mask, so the tokens need no luminance tuning per theme.
//! Design choice: each layer is its own SVG, so the two drift at different speeds with no script.
//! Design choice: `palette_token` maps to `--kit-glow-<token>`; an undefined or empty token falls
//! back to `--kit-id-1`, so the light reads as colour rather than a grey wash.
//! Design choice: intensity is the opacity of both layers; a change eases to the new strength.
//!
//! Browser checks: clicks and focus reach content above the glow; the two layers drift at
//! different speeds; pausing holds both; reduced motion holds both still; an intensity change eases;
//! the glow stays inside the stage at phone width.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};
use crate::motion::flowing_gradient_text::token_name;

const CSS: Asset = asset!("/assets/glow-floor-backdrop.css");

/// Layer drift periods, in milliseconds, for the first and second layer.
pub const LAYER_DURATIONS_MS: [u32; 2] = [23_000, 31_000];

/// Height of the wash's top edge above the stage bottom, as (x, height), both in percent of the
/// stage. The points run past 0 and 100 so a drift never uncovers an edge. The highest point lies
/// in the left fifth; no point beyond x 20 is above 45.
pub const TOP_EDGE: [(f64, f64); 18] = [
    (-10.0, 50.0),
    (0.0, 52.0),
    (6.0, 56.0),
    (12.0, 58.5),
    (17.0, 56.0),
    (20.0, 50.0),
    (23.0, 44.5),
    (28.0, 43.0),
    (34.0, 42.0),
    (42.0, 41.0),
    (50.0, 40.0),
    (58.0, 40.5),
    (66.0, 39.0),
    (74.0, 40.5),
    (82.0, 42.0),
    (90.0, 41.5),
    (100.0, 43.0),
    (110.0, 43.5),
];

/// Closed path of the wash: the top edge across the stage, then down past the bottom.
pub fn field_path() -> String {
    let mut d = String::new();
    for (index, (x, height)) in TOP_EDGE.iter().enumerate() {
        let command = if index == 0 { "M" } else { "L" };
        d.push_str(&format!("{command} {x} {} ", 100.0 - height));
    }
    d.push_str("L 110 100 L -10 100 Z");
    d
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlowIntensity {
    Subtle,
    #[default]
    Standard,
    Strong,
}

impl GlowIntensity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Subtle => "subtle",
            Self::Standard => "standard",
            Self::Strong => "strong",
        }
    }
}

/// Layer opacity for an intensity.
pub fn intensity_opacity(intensity: GlowIntensity) -> f64 {
    match intensity {
        GlowIntensity::Subtle => 0.45,
        GlowIntensity::Standard => 0.85,
        GlowIntensity::Strong => 1.0,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlowAnchor {
    #[default]
    Bottom,
    Top,
}

impl GlowAnchor {
    fn as_str(self) -> &'static str {
        match self {
            Self::Bottom => "bottom",
            Self::Top => "top",
        }
    }
}

/// CSS colour for the glow: the palette token's variable, or the first identity colour.
pub fn glow_color(token: &str) -> String {
    let name = token_name(token);
    if name.is_empty() {
        "var(--kit-id-1)".to_string()
    } else {
        format!("var(--kit-glow-{name}, var(--kit-rule))")
    }
}

#[component]
pub fn GlowFloorBackdrop(
    children: Element,
    #[props(default)] intensity: GlowIntensity,
    #[props(default)] anchor: GlowAnchor,
    #[props(default, into)] palette_token: String,
    #[props(default)] paused: bool,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("kit-glow-{}", next_id()));
    let style = format!(
        "--kit-glow-opacity: {}; --kit-glow-color: {};",
        intensity_opacity(intensity),
        glow_color(&palette_token),
    );
    let [first, second] = LAYER_DURATIONS_MS;
    let class = class_list("kit-glow", &class);
    let field = field_path();
    let a = format!("{id}-a");
    let b = format!("{id}-b");
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { id: "{id}", class, style: "{style}",
            "data-intensity": intensity.as_str(),
            "data-anchor": anchor.as_str(),
            "data-paused": paused.then_some("true"),
            div { class: "kit-glow__stage",
                div { class: "kit-glow__backdrop", "aria-hidden": "true",
                    svg { class: "kit-glow__layer", "data-layer": "a", style: "animation-duration: {first}ms;",
                        "viewBox": "0 0 100 100", "preserveAspectRatio": "none", "focusable": "false",
                        defs {
                            linearGradient { id: "{a}-hue", "gradientUnits": "userSpaceOnUse",
                                "x1": "-10", "y1": "0", "x2": "110", "y2": "0",
                                stop { class: "kit-glow__hue-0", "offset": "0" }
                                stop { class: "kit-glow__hue-1", "offset": "0.083" }
                                stop { class: "kit-glow__hue-2", "offset": "0.25" }
                                stop { class: "kit-glow__hue-3", "offset": "0.5" }
                                stop { class: "kit-glow__hue-4", "offset": "0.75" }
                                stop { class: "kit-glow__hue-5", "offset": "1" }
                            }
                            linearGradient { id: "{a}-fade", "gradientUnits": "userSpaceOnUse",
                                "x1": "0", "y1": "100", "x2": "0", "y2": "34",
                                stop { class: "kit-glow__fade-0", "offset": "0" }
                                stop { class: "kit-glow__fade-1", "offset": "0.455" }
                                stop { class: "kit-glow__fade-2", "offset": "0.682" }
                                stop { class: "kit-glow__fade-3", "offset": "0.833" }
                                stop { class: "kit-glow__fade-4", "offset": "0.939" }
                                stop { class: "kit-glow__fade-5", "offset": "1" }
                            }
                            mask { id: "{a}-mask", "maskUnits": "userSpaceOnUse", "mask-type": "alpha",
                                "x": "-10", "y": "-10", "width": "120", "height": "120",
                                path { class: "kit-glow__shape", "d": "{field}", "fill": "url(#{a}-fade)" }
                            }
                        }
                        rect { class: "kit-glow__wash", "x": "-10", "y": "-10", "width": "120", "height": "120",
                            "fill": "url(#{a}-hue)", "mask": "url(#{a}-mask)" }
                    }
                    svg { class: "kit-glow__layer", "data-layer": "b", style: "animation-duration: {second}ms;",
                        "viewBox": "0 0 100 100", "preserveAspectRatio": "none", "focusable": "false",
                        defs {
                            radialGradient { id: "{b}-lift", "gradientUnits": "userSpaceOnUse",
                                "cx": "50", "cy": "100", "r": "62",
                                stop { class: "kit-glow__lift-0", "offset": "0" }
                                stop { class: "kit-glow__lift-1", "offset": "0.5" }
                                stop { class: "kit-glow__lift-2", "offset": "1" }
                            }
                            linearGradient { id: "{b}-fade", "gradientUnits": "userSpaceOnUse",
                                "x1": "0", "y1": "100", "x2": "0", "y2": "34",
                                stop { class: "kit-glow__fade-0", "offset": "0" }
                                stop { class: "kit-glow__fade-1", "offset": "0.455" }
                                stop { class: "kit-glow__fade-2", "offset": "0.682" }
                                stop { class: "kit-glow__fade-3", "offset": "0.833" }
                                stop { class: "kit-glow__fade-4", "offset": "0.939" }
                                stop { class: "kit-glow__fade-5", "offset": "1" }
                            }
                            mask { id: "{b}-mask", "maskUnits": "userSpaceOnUse", "mask-type": "alpha",
                                "x": "-10", "y": "-10", "width": "120", "height": "120",
                                path { class: "kit-glow__shape", "d": "{field}", "fill": "url(#{b}-fade)" }
                            }
                        }
                        rect { class: "kit-glow__wash kit-glow__wash--lift", "x": "-10", "y": "-10", "width": "120", "height": "120",
                            "fill": "url(#{b}-lift)", "mask": "url(#{b}-mask)" }
                    }
                }
                div { class: "kit-glow__content", {children} }
            }
        }
    }
}
