//! Voice assistant orb: a dot sphere that shows the assistant's mode. It breathes at rest, pushes
//! outward with input loudness while listening, collapses into a spinning ring while thinking, and
//! emits ripples while speaking.
//!
//! Design choice: every dot is positioned by two custom properties (sphere and ring) and moved by
//! `transform` only, so collapsing into the ring is one continuous transition.
//! Design choice: the dots are denser on the front hemisphere (two thirds of the dots on the half
//! facing the viewer), and depth sets their opacity.
//! Design choice: the demonstration state switch is not part of this component; the caller owns
//! `state` and `level`.
//! Design choice: the interactive orb is a toggle button named "Voice assistant"; `aria-pressed`
//! follows listening and `value` carries the state word. The status region announces changes only.
//! Browser checks: thinking collapses the dots into a spinning ring; raising `level` in listening
//! pushes dots outward; speaking ripples fade with distance; idle keeps breathing and reduced motion
//! stops it; the interactive orb calls `on_press` once per press; the status region announces once.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/voice-assistant-orb.css");

/// Dots on the sphere.
pub const DOT_COUNT: usize = 120;
/// Sphere radius and ring radius, in CSS px.
pub const SPHERE_RADIUS_PX: f64 = 84.0;
pub const RING_RADIUS_PX: f64 = 62.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrbState {
    #[default]
    Idle,
    Listening,
    Thinking,
    Speaking,
}

impl OrbState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Listening => "listening",
            Self::Thinking => "thinking",
            Self::Speaking => "speaking",
        }
    }
}

/// The status word shown under the orb.
pub fn state_label(state: OrbState) -> &'static str {
    match state {
        OrbState::Idle => "Idle",
        OrbState::Listening => "Listening",
        OrbState::Thinking => "Thinking",
        OrbState::Speaking => "Speaking",
    }
}

/// Loudness clamped to 0..=1; NaN reads as 0.
pub fn clamp_level(level: f64) -> f64 {
    if level.is_nan() {
        0.0
    } else {
        level.clamp(0.0, 1.0)
    }
}

/// A point on the unit sphere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dot {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// `count` points spread over the sphere by the Fibonacci lattice, with two thirds on the front
/// hemisphere (z >= 0).
pub fn dot_sphere(count: usize) -> Vec<Dot> {
    let golden = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    let raw: Vec<Dot> = (0..count * 2)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f64 + 0.5) / (count * 2) as f64;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let theta = golden * i as f64;
            Dot {
                x: theta.cos() * r,
                y,
                z: theta.sin() * r,
            }
        })
        .collect();
    let front: Vec<Dot> = raw.iter().copied().filter(|d| d.z >= 0.0).collect();
    let back: Vec<Dot> = raw.iter().copied().filter(|d| d.z < 0.0).collect();
    let take_front = (count * 2 / 3).min(front.len());
    let take_back = (count - take_front).min(back.len());
    let mut dots = evenly_picked(&front, take_front);
    dots.extend(evenly_picked(&back, take_back));
    dots
}

/// `n` items from `items`, spaced evenly through the list. The lattice is ordered top to bottom, so
/// taking a prefix would leave the sphere's lower half empty.
fn evenly_picked(items: &[Dot], n: usize) -> Vec<Dot> {
    (0..n).map(|k| items[k * items.len() / n]).collect()
}

/// `count` points evenly spaced on a unit circle, starting at 3 o'clock.
pub fn ring_points(count: usize) -> Vec<(f64, f64)> {
    (0..count)
        .map(|k| {
            let angle = std::f64::consts::TAU * k as f64 / count as f64;
            (angle.cos(), angle.sin())
        })
        .collect()
}

/// Opacity for a dot at depth `z` (-1 back to 1 front).
pub fn dot_opacity(z: f64) -> f64 {
    0.3 + 0.7 * (z.clamp(-1.0, 1.0) + 1.0) / 2.0
}

#[component]
pub fn VoiceAssistantOrb(
    #[props(default)] state: OrbState,
    #[props(default)] level: f64,
    #[props(default, into)] label: Option<String>,
    #[props(default)] interactive: bool,
    #[props(default)] on_press: Option<EventHandler<()>>,
    #[props(default)] class: String,
) -> Element {
    let status_id = use_hook(|| format!("kit-orb-status-{}", next_id()));
    let level = clamp_level(level);
    let root_style = format!("--level: {level}; --ripple-max: {};", 1.4 + 0.8 * level);
    let class = class_list("kit-orb", &class);
    let dots = dot_sphere(DOT_COUNT);
    let rings = ring_points(DOT_COUNT);
    let status = state_label(state);
    let listening = state == OrbState::Listening;
    let orb = rsx! {
        span { class: "kit-orb__field",
            span { class: "kit-orb__dots",
                for (i, dot) in dots.iter().enumerate() {
                    {
                        let (rx, ry) = rings[i];
                        let style = format!(
                            "--sx: {:.2}px; --sy: {:.2}px; --rx: {:.2}px; --ry: {:.2}px; --o: {:.2};",
                            dot.x * SPHERE_RADIUS_PX,
                            dot.y * SPHERE_RADIUS_PX,
                            rx * RING_RADIUS_PX,
                            ry * RING_RADIUS_PX,
                            dot_opacity(dot.z),
                        );
                        rsx! { span { class: "kit-orb__dot", style: "{style}" } }
                    }
                }
            }
            span { class: "kit-orb__ripple", "aria-hidden": "true" }
            span { class: "kit-orb__ripple", "aria-hidden": "true" }
            span { class: "kit-orb__ripple", "aria-hidden": "true" }
        }
    };
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class, style: "{root_style}",
            "data-state": state.as_str(),
            if interactive {
                button { r#type: "button", class: "kit-orb__button",
                    "aria-label": "Voice assistant",
                    "aria-pressed": "{listening}",
                    value: "{status}",
                    "aria-describedby": "{status_id}",
                    onclick: move |_| {
                        if let Some(handler) = on_press {
                            handler.call(());
                        }
                    },
                    {orb}
                }
            } else {
                div { class: "kit-orb__hit", "aria-hidden": "true", {orb} }
            }
            if let Some(label) = label {
                span { class: "kit-orb__label", "{label}" }
            }
            span { class: "kit-orb__status", id: "{status_id}", role: "status", "aria-live": "polite",
                "{status}"
            }
        }
    }
}
