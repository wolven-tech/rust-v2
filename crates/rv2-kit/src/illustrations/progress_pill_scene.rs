//! Progress pill scene: one compact status pill for a working agent, centred in its stage over a soft
//! two-tinted glow band. It shows the job's title, how far along it is, the agent and its source
//! count, the current step once one is set, and a thin progress bar with a shimmer on its fill.
//!
//! Design choice: the glow is a blurred band drawn under the pill by this scene, not a
//! `GlowFloorBackdrop`; a floor stage would wrap the pill in a second surface. `paused` and done hold
//! the band steady and dimmer, as the working state's band breathes.
//! Design choice: the fill moves by `transform: scaleX`, so a progress change eases without a
//! layout pass.
//! Design choice: only the step line is a live region, so the percentage ticks are silent and
//! the step change is announced.
//! Design choice: the title truncates with an ellipsis; a focused interactive pill shows the full
//! title by wrapping.
//! Browser checks: the fill eases to a new value; the percentage matches the fill; paused and done
//! read in text; reduced motion holds the mark and shimmer still; the title truncates at 390 and 320.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/progress-pill-scene.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PillState {
    #[default]
    Working,
    Paused,
    Done,
}

impl PillState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Paused => "paused",
            Self::Done => "done",
        }
    }
}

/// Progress clamped to 0..=1; NaN reads as 0.
pub fn clamp_progress(progress: f64) -> f64 {
    if progress.is_nan() {
        0.0
    } else {
        progress.clamp(0.0, 1.0)
    }
}

/// The whole-number percentage shown on the pill.
pub fn percent_value(progress: f64) -> u32 {
    (clamp_progress(progress) * 100.0).round() as u32
}

/// The text beside the title: the percentage, or "Done" once the job is done.
pub fn percent_text(progress: f64, state: PillState) -> String {
    match state {
        PillState::Done => "Done".to_string(),
        _ => format!("{}%", percent_value(progress)),
    }
}

#[component]
pub fn ProgressPillScene(
    #[props(into)] title: String,
    #[props(into)] agent_label: String,
    #[props(into)] source_line: String,
    #[props(default)] progress: f64,
    #[props(into, default)] step_label: String,
    #[props(default)] state: PillState,
    #[props(default)] mark: Option<Element>,
    #[props(default)] interactive: bool,
    #[props(default)] on_open: Option<EventHandler<()>>,
    #[props(default)] class: String,
) -> Element {
    let value = clamp_progress(progress);
    let percent = percent_text(value, state);
    let fill = format!("transform: scaleX({value});");
    let aria_name = format!("{title}, {percent}");
    let class = class_list("kit-pill-scene", &class);
    let inner = rsx! {
        div { class: "kit-pill", "data-state": state.as_str(),
            div { class: "kit-pill__mark", "aria-hidden": "true",
                if let Some(mark) = mark {
                    {mark}
                } else {
                    div { class: "kit-pill__dots",
                        span { class: "kit-pill__cell" }
                        span { class: "kit-pill__cell" }
                        span { class: "kit-pill__cell" }
                        span { class: "kit-pill__cell" }
                        span { class: "kit-pill__cell" }
                        span { class: "kit-pill__cell" }
                        span { class: "kit-pill__cell" }
                        span { class: "kit-pill__cell" }
                        span { class: "kit-pill__cell" }
                    }
                }
            }
            div { class: "kit-pill__body",
                div { class: "kit-pill__head",
                    span { class: "kit-pill__title", title: "{title}", "{title}" }
                    span { class: "kit-pill__percent", "{percent}" }
                }
                div { class: "kit-pill__agent",
                    span { class: "kit-pill__agent-name", "{agent_label}" }
                    span { class: "kit-pill__dot", "aria-hidden": "true", "·" }
                    span { class: "kit-pill__source", "{source_line}" }
                }
                if !step_label.is_empty() {
                    div { class: "kit-pill__step", role: "status", "aria-live": "polite",
                        "{step_label}"
                    }
                }
                div { class: "kit-pill__track", role: "progressbar",
                    "aria-label": "Progress",
                    "aria-valuemin": "0",
                    "aria-valuemax": "100",
                    "aria-valuenow": percent_value(value).to_string(),
                    span { class: "kit-pill__fill", style: "{fill}" }
                }
            }
        }
    };
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class, "data-state": state.as_str(),
            span { class: "kit-pill-scene__band", "aria-hidden": "true" }
            if interactive {
                button { r#type: "button", class: "kit-pill-scene__button",
                    "aria-label": "{aria_name}",
                    onclick: move |_| {
                        if let Some(handler) = on_open {
                            handler.call(());
                        }
                    },
                    {inner}
                }
            } else {
                {inner}
            }
        }
    }
}
