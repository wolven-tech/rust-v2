//! Dictation capture panel: shown while a person dictates. It shows that the microphone is
//! listening, how loud the speech is and how long they have been speaking, and offers finish and
//! cancel.
//!
//! Design choice: the glow is the Phase 1 `GlowFloorBackdrop`; its intensity steps with loudness
//! and it holds still unless listening.
//! Design choice: the actions are native buttons, so a Space press on a focused button fires its
//! click and not the panel's Space handler (which would finish twice).
//! Design choice: `elapsed_secs` comes from the host clock; the panel never counts itself.
//! Design choice: the waveform is a fixed row of bars, right-aligned to the newest sample; a new
//! sample re-keys the track, which slides the bars one step left.
//! Design choice: under reduced motion the waveform holds the samples from the last state change, so
//! new samples do not move the bars; time and status still update.
//! Browser checks: bars change with samples and the time advances while listening; pausing stops
//! both; Space finishes and Escape cancels from the panel; reduced motion keeps the waveform still;
//! hints hide and nothing scrolls sideways at 390 and 320.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, prefers_motion_script};
use crate::motion::glow_floor_backdrop::{GlowAnchor, GlowFloorBackdrop, GlowIntensity};

const CSS: Asset = asset!("/assets/dictation-capture-panel.css");

/// Bars in the waveform well.
pub const BAR_COUNT: usize = 40;
/// Samples averaged for the glow.
pub const GLOW_WINDOW: usize = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DictationState {
    Listening,
    Paused,
    #[default]
    Idle,
}

impl DictationState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Listening => "listening",
            Self::Paused => "paused",
            Self::Idle => "idle",
        }
    }
}

/// Status text for the state; the live region announces it on change.
pub fn status_text(state: DictationState) -> &'static str {
    match state {
        DictationState::Listening => "Listening",
        DictationState::Paused => "Paused",
        DictationState::Idle => "Ready to listen",
    }
}

/// Elapsed seconds as mm:ss.
pub fn format_elapsed(secs: u64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

/// The newest `bars` samples, clamped to 0..=1, right-aligned; missing older bars are 0.
pub fn bar_heights(samples: &[f64], bars: usize) -> Vec<f64> {
    let take = samples.len().min(bars);
    let mut heights = vec![0.0; bars - take];
    heights.extend(
        samples[samples.len() - take..]
            .iter()
            .map(|s| if s.is_nan() { 0.0 } else { s.clamp(0.0, 1.0) }),
    );
    heights
}

/// Mean loudness of the last `GLOW_WINDOW` samples, 0..=1.
pub fn loudness(samples: &[f64]) -> f64 {
    let recent = &samples[samples.len().saturating_sub(GLOW_WINDOW)..];
    if recent.is_empty() {
        0.0
    } else {
        recent.iter().map(|s| s.clamp(0.0, 1.0)).sum::<f64>() / recent.len() as f64
    }
}

fn bar_style(height: f64) -> String {
    format!("transform: scaleY({height:.3});")
}

fn glow_for(loudness: f64) -> GlowIntensity {
    if loudness < 0.2 {
        GlowIntensity::Subtle
    } else if loudness < 0.6 {
        GlowIntensity::Standard
    } else {
        GlowIntensity::Strong
    }
}

#[component]
pub fn DictationCapturePanel(
    #[props(default)] state: DictationState,
    #[props(default)] level_samples: Vec<f64>,
    #[props(default)] elapsed_secs: u64,
    #[props(default)] on_finish: Option<EventHandler<()>>,
    #[props(default)] on_cancel: Option<EventHandler<()>>,
    #[props(default)] finishing: bool,
    #[props(default = "Finish".to_string(), into)] finish_label: String,
    #[props(default = "Cancel".to_string(), into)] cancel_label: String,
    #[props(default)] finish_hint: Option<Element>,
    #[props(default)] cancel_hint: Option<Element>,
    #[props(default = true)] show_shortcuts: bool,
    #[props(default)] class: String,
) -> Element {
    let idle = state == DictationState::Idle;
    let mut motion = use_signal(|| true);
    use_hook(move || {
        spawn(async move {
            let allowed = document::eval(&format!("return {};", prefers_motion_script()))
                .await
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            motion.set(allowed);
        });
    });
    let reduced = !motion();
    let mut held = use_signal(|| None::<(DictationState, Vec<f64>)>);
    let live: Vec<f64> = if idle {
        Vec::new()
    } else {
        level_samples.clone()
    };
    use_effect(use_reactive(
        (&reduced, &state, &live),
        move |(reduced, state, live)| {
            let current = held.peek().clone();
            if !reduced {
                if current.is_some() {
                    held.set(None);
                }
            } else if current.as_ref().is_none_or(|(at, _)| *at != state) {
                held.set(Some((state, live)));
            }
        },
    ));
    let samples: Vec<f64> = match (reduced, held()) {
        (true, Some((_, frozen))) => frozen,
        _ => live.clone(),
    };
    let heights = bar_heights(&samples, BAR_COUNT);
    let newest = samples.last().copied().unwrap_or(0.0);
    let track_key = if state == DictationState::Listening {
        format!("{}-{}", samples.len(), newest.to_bits())
    } else {
        "still".to_string()
    };
    let uid = use_hook(|| format!("kit-dict-{}", next_id()));
    let finish_hint_id = format!("{uid}-finish");
    let cancel_hint_id = format!("{uid}-cancel");
    let time = format_elapsed(if idle { 0 } else { elapsed_secs });
    let class = class_list("kit-dict-scene", &class);
    let finish_disabled = idle || finishing;
    let finish_hint = finish_hint.unwrap_or_else(|| rsx! { kbd { "Space" } });
    let cancel_hint = cancel_hint.unwrap_or_else(|| rsx! { kbd { "Esc" } });
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        GlowFloorBackdrop {
            intensity: glow_for(loudness(&samples)),
            anchor: GlowAnchor::Bottom,
            paused: state != DictationState::Listening,
            class,
            section {
                class: "kit-dict",
                role: "region",
                "aria-label": "Dictation",
                tabindex: "-1",
                "data-state": state.as_str(),
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Escape {
                        event.prevent_default();
                        if let Some(handler) = on_cancel {
                            handler.call(());
                        }
                    } else if event.key() == Key::Character(" ".into()) && !finish_disabled {
                        event.prevent_default();
                        if let Some(handler) = on_finish {
                            handler.call(());
                        }
                    }
                },
                div { class: "kit-dict__well", "aria-hidden": "true",
                    div { class: "kit-dict__track", key: "{track_key}",
                        for height in heights {
                            span { class: "kit-dict__bar", style: bar_style(height) }
                        }
                    }
                }
                div { class: "kit-dict__row",
                    div { class: "kit-dict__status",
                        span { class: "kit-dict__dot", "aria-hidden": "true" }
                        span { class: "kit-dict__state", role: "status", "aria-live": "polite",
                            {status_text(state)}
                        }
                        span { class: "kit-dict__time", "{time}" }
                    }
                    span { class: "kit-dict__divider", "aria-hidden": "true" }
                    div { class: "kit-dict__actions",
                        button {
                            r#type: "button",
                            class: "kit-dict__btn kit-dict__btn--primary",
                            disabled: finish_disabled,
                            "aria-busy": finishing.then_some("true"),
                            "aria-describedby": "{finish_hint_id}",
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() != Key::Escape {
                                    event.stop_propagation();
                                }
                            },
                            onclick: move |_| {
                                if let Some(handler) = on_finish {
                                    handler.call(());
                                }
                            },
                            span { "{finish_label}" }
                            span { class: "kit-dict__hint", id: "{finish_hint_id}",
                                hidden: !show_shortcuts,
                                {finish_hint}
                            }
                        }
                        button {
                            r#type: "button",
                            class: "kit-dict__btn",
                            "aria-describedby": "{cancel_hint_id}",
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() != Key::Escape {
                                    event.stop_propagation();
                                }
                            },
                            onclick: move |_| {
                                if let Some(handler) = on_cancel {
                                    handler.call(());
                                }
                            },
                            span { "{cancel_label}" }
                            span { class: "kit-dict__hint", id: "{cancel_hint_id}",
                                hidden: !show_shortcuts,
                                {cancel_hint}
                            }
                        }
                    }
                }
            }
        }
    }
}
