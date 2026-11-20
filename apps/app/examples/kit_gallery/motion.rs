use dioxus::prelude::*;

use super::{Item, Section};

#[component]
pub fn Motion() -> Element {
    rsx! {
        Section { id: "motion", title: "Motion",
            PointerTiltCardDemo {}
            FlowingGradientTextDemo {}
            GlowFloorBackdropDemo {}
            BlurSwapContainerDemo {}
        }
    }
}

#[component]
fn PointerTiltCardDemo() -> Element {
    use rv2_kit::motion::pointer_tilt_card::{PointerTiltCard, TiltStrength};
    rsx! {
        Item { slug: "pointer-tilt-card", name: "PointerTiltCard",
            PointerTiltCard { max_tilt: TiltStrength::Strong,
                div { class: "gl-canvas", style: "padding: 24px; background: var(--kit-surface, #fff);",
                    "Hover me"
                }
            }
        }
    }
}

#[component]
fn FlowingGradientTextDemo() -> Element {
    use rv2_kit::motion::flowing_gradient_text::FlowingGradientText;
    rsx! {
        Item { slug: "flowing-gradient-text", name: "FlowingGradientText",
            FlowingGradientText { text: "Streaming answer", emphasis_range: Some(0..9), replay_enabled: true }
        }
    }
}

#[component]
fn GlowFloorBackdropDemo() -> Element {
    use rv2_kit::motion::glow_floor_backdrop::{GlowAnchor, GlowFloorBackdrop, GlowIntensity};
    let mut intensity = use_signal(|| GlowIntensity::Standard);
    let mut paused = use_signal(|| false);
    rsx! {
        Item { slug: "glow-floor-backdrop", name: "GlowFloorBackdrop",
            div { class: "gl-row",
                button { r#type: "button", onclick: move |_| paused.toggle(), if paused() { "Resume" } else { "Pause" } }
                button { r#type: "button", onclick: move |_| intensity.set(GlowIntensity::Subtle), "Subtle" }
                button { r#type: "button", onclick: move |_| intensity.set(GlowIntensity::Standard), "Standard" }
                button { r#type: "button", onclick: move |_| intensity.set(GlowIntensity::Strong), "Strong" }
            }
            GlowFloorBackdrop { intensity: intensity(), anchor: GlowAnchor::Bottom, paused: paused(),
                div { class: "gl-canvas", style: "min-height: 120px; padding: 24px;",
                    "Content above the glow"
                }
            }
        }
    }
}

#[component]
fn BlurSwapContainerDemo() -> Element {
    use rv2_kit::motion::blur_swap_container::BlurSwapContainer;
    let mut key = use_signal(|| "one".to_string());
    let mut last = use_signal(String::new);
    let label = if key() == "one" {
        "Panel one"
    } else {
        "Panel two"
    };
    rsx! {
        Item { slug: "blur-swap-container", name: "BlurSwapContainer",
            div { class: "gl-row",
                button { r#type: "button", "data-action": "swap", onclick: move |_| {
                    let next = if key() == "one" { "two" } else { "one" };
                    key.set(next.to_string());
                    last.set(format!("swap {next}"));
                }, "Swap panel" }
            }
            BlurSwapContainer { content_key: key(), on_swap_complete: move |_| last.set("swap complete".into()),
                div { class: "gl-canvas", style: "min-height: 80px; padding: 16px; border-color: var(--kit-rule);", "{label}" }
            }
            div { class: "gl-out", "data-out": "blur-swap-container", "{last()}" }
        }
    }
}
