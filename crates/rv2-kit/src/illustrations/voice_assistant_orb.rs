//! Voice assistant orb: a dot sphere that shows the assistant's mode. It breathes at rest, pushes
//! outward with input loudness while listening, collapses into a spinning ring while thinking, and
//! emits ripples while speaking.
//!
//! Design choice: every dot is positioned by two custom properties (sphere and ring) and moved by
//! `transform` only, so collapsing into the ring is one continuous transition.
//! Design choice: the dots form a lattice of latitude rings, each holding dots in proportion to its
//! radius, so spacing stays even. Only the front hemisphere is drawn, and depth sets opacity.
//! Design choice: the caller owns `state` and `level`. The four-state switch is demonstration only and
//! is drawn only when `on_demo_state` is given.
//! Design choice: the interactive orb is a toggle button named "Voice assistant"; `aria-pressed`
//! follows listening and `value` carries the state word. The status region announces changes only.
//! Browser checks: thinking collapses the dots into a spinning ring; raising `level` in listening
//! pushes dots outward; speaking ripples fade with distance; idle keeps breathing and reduced motion
//! stops it; the interactive orb calls `on_press` once per press; the status region announces once.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/voice-assistant-orb.css");

/// Latitude rings from pole to pole.
pub const LATITUDE_RINGS: usize = 15;
/// Dots on a ring equal this times the ring's sine, so the spacing is the same on every ring.
pub const DOTS_PER_RING_SCALE: f64 = 36.0;
/// Sphere radius and ring radius, in percent of the orb's width (the radius is half the diameter).
pub const SPHERE_RADIUS_PCT: f64 = 50.0;
pub const RING_RADIUS_PCT: f64 = 36.9;

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
        OrbState::Listening => "Listening…",
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

/// The front hemisphere (z >= 0) of the lattice: `LATITUDE_RINGS` rings, pole to pole, ring k at
/// polar angle (k + 1/2) pi / rings. The axis is vertical, so the outline reads as a sphere.
pub fn lattice_dots() -> Vec<Dot> {
    let mut dots = Vec::new();
    for k in 0..LATITUDE_RINGS {
        let theta = (k as f64 + 0.5) * std::f64::consts::PI / LATITUDE_RINGS as f64;
        let (ring_r, y) = (theta.sin(), theta.cos());
        let n = (DOTS_PER_RING_SCALE * ring_r).round().max(1.0) as usize;
        for j in 0..n {
            let phi = std::f64::consts::TAU * j as f64 / n as f64;
            let dot = Dot {
                x: ring_r * phi.cos(),
                y,
                z: ring_r * phi.sin(),
            };
            if dot.z >= 0.0 {
                dots.push(dot);
            }
        }
    }
    dots
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
    /// Shows the four-state demonstration switch under the status word; without it the orb renders alone.
    #[props(default)]
    on_demo_state: Option<EventHandler<OrbState>>,
    #[props(default)] class: String,
) -> Element {
    let status_id = use_hook(|| format!("kit-orb-status-{}", next_id()));
    let level = clamp_level(level);
    let root_style = format!("--level: {level}; --ripple-max: {};", 1.4 + 0.8 * level);
    let class = class_list("kit-orb", &class);
    let dots = lattice_dots();
    let rings = ring_points(dots.len());
    let status = state_label(state);
    let listening = state == OrbState::Listening;
    let orb = rsx! {
        span { class: "kit-orb__field",
            span { class: "kit-orb__dots",
                for (i, dot) in dots.iter().enumerate() {
                    {
                        let (rx, ry) = rings[i];
                        let style = format!(
                            "--sx: {:.3}cqw; --sy: {:.3}cqw; --rx: {:.3}cqw; --ry: {:.3}cqw; --o: {:.2};",
                            dot.x * SPHERE_RADIUS_PCT,
                            dot.y * SPHERE_RADIUS_PCT,
                            rx * RING_RADIUS_PCT,
                            ry * RING_RADIUS_PCT,
                            dot_opacity(dot.z),
                        );
                        rsx! { span { class: "kit-orb__dot", style: "{style}" } }
                    }
                }
            }
            if state == OrbState::Speaking {
                span { class: "kit-orb__ripple", "aria-hidden": "true" }
                span { class: "kit-orb__ripple", "aria-hidden": "true" }
                span { class: "kit-orb__ripple", "aria-hidden": "true" }
            }
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
            if let Some(on_demo_state) = on_demo_state {
                div { class: "kit-orb__switch", role: "group", "aria-label": "Demonstration state",
                    for (name, value) in [
                        ("Idle", OrbState::Idle),
                        ("Listening", OrbState::Listening),
                        ("Thinking", OrbState::Thinking),
                        ("Speaking", OrbState::Speaking),
                    ] {
                        button { r#type: "button", class: "kit-orb__segment",
                            "aria-pressed": "{state == value}",
                            "data-selected": (state == value).then_some("true"),
                            onclick: move |_| on_demo_state.call(value),
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}
