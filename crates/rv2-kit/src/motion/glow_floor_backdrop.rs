//! Glow floor backdrop: two blurred light bands drift and reshape behind the
//! content of a section, anchored to its bottom (or top) edge.
//!
//! Design choice: drift and reshape are CSS keyframes; `paused` sets
//! `animation-play-state`, so no script runs.
//! Design choice: the two layers run on different durations, so their combined
//! pattern does not repeat in any visible span.
//! Design choice: `palette_token` maps to `--kit-glow-<token>`; an undefined token
//! falls back to `--kit-rule`.
//! Design choice: intensity is an opacity on the layers; the layers transition
//! opacity, so a change eases to the new strength.
//!
//! Browser checks: clicks and focus reach content above the glow; the two layers drift
//! at different speeds; pausing holds both; reduced motion holds both still; an
//! intensity change eases; the glow stays inside the section at phone width.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};
use crate::motion::flowing_gradient_text::token_name;

const CSS: Asset = asset!("/assets/glow-floor-backdrop.css");

/// Layer drift periods, in milliseconds, for the first and second layer.
pub const LAYER_DURATIONS_MS: [u32; 2] = [23_000, 31_000];

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
        GlowIntensity::Subtle => 0.35,
        GlowIntensity::Standard => 0.6,
        GlowIntensity::Strong => 0.9,
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

/// CSS colour for the glow: the palette token's variable, or the rule colour.
pub fn glow_color(token: &str) -> String {
    let name = token_name(token);
    if name.is_empty() {
        "var(--kit-rule)".to_string()
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
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { id: "{id}", class, style: "{style}",
            "data-intensity": intensity.as_str(),
            "data-anchor": anchor.as_str(),
            "data-paused": paused.then_some("true"),
            div { class: "kit-glow__backdrop", "aria-hidden": "true",
                div { class: "kit-glow__layer", "data-layer": "a", style: "animation-duration: {first}ms;" }
                div { class: "kit-glow__layer", "data-layer": "b", style: "animation-duration: {second}ms;" }
            }
            div { class: "kit-glow__content", {children} }
        }
    }
}
