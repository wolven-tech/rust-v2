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
    use rv2_kit::motion::pointer_tilt_card::{MembershipCard, TiltStrength};
    rsx! {
        Item { slug: "pointer-tilt-card", name: "PointerTiltCard",
            MembershipCard {
                name: "Ledger Plus",
                holder: "Jamie Rivera",
                since: "03/24",
                last_four: "2750",
                caption: "Move the pointer over the card",
                max_tilt: TiltStrength::Strong,
            }
        }
    }
}

#[component]
fn FlowingGradientTextDemo() -> Element {
    use rv2_kit::motion::flowing_gradient_text::{FlowingGradientText, last_words_range};
    let text = "Every figure traces back to its source.";
    rsx! {
        Item { slug: "flowing-gradient-text", name: "FlowingGradientText",
            FlowingGradientText {
                text,
                emphasis_range: Some(last_words_range(text, 2)),
                replay_enabled: true,
                gradient_token: "accent",
                class: "kit-gradient-text--display",
            }
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
            div { style: "width: min(100%, 560px); margin: 0 auto;",
                GlowFloorBackdrop { intensity: intensity(), anchor: GlowAnchor::Bottom, paused: paused(),
                    {rsx! {}}
                }
            }
        }
    }
}

#[component]
fn BlurSwapContainerDemo() -> Element {
    use rv2_kit::motion::blur_swap_container::{BlurSwapStepCard, StepBody, next_step};
    let mut step = use_signal(|| 0usize);
    let steps = vec![
        StepBody {
            tag: "Live".into(),
            sentence: "Offline mode now works on phones.".into(),
        },
        StepBody {
            tag: "Up next".into(),
            sentence: "Bulk export is ready for review. Check the file format before you publish."
                .into(),
        },
        StepBody {
            tag: "Done".into(),
            sentence: "Every change in this version has a named owner.".into(),
        },
    ];
    rsx! {
        Item { slug: "blur-swap-container", name: "BlurSwapContainer",
            BlurSwapStepCard {
                title: "Version 3.1",
                status: "2 changes · draft",
                primary_label: "Publish",
                steps,
                current: step(),
                on_next: move |_| step.set(next_step(step(), 3)),
            }
        }
    }
}
