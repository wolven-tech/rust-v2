use dioxus::prelude::*;
use rv2_kit::motion::blur_swap_container::{
    BlurSwapContainer, FOCUSABLE_SELECTOR, SWAP_DURATION_MS, motion_allowed, swap_needed,
};
use rv2_kit::motion::flowing_gradient_text::{
    FlowingGradientText, GradientPhase, gradient_fill, split_emphasis, token_name,
};
use rv2_kit::motion::glow_floor_backdrop::{
    GlowAnchor, GlowFloorBackdrop, GlowIntensity, LAYER_DURATIONS_MS, glow_color, intensity_opacity,
};
use rv2_kit::motion::pointer_tilt_card::{
    PointerTiltCard, TiltStrength, TouchBehaviour, glare_percent, tilt_degrees, unit_pointer,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[test]
fn swap_starts_only_when_the_key_changes() {
    assert!(swap_needed("step-1", "step-2"));
    assert!(!swap_needed("step-1", "step-1"));
}

#[test]
fn reduced_motion_preference_disables_motion_unless_overridden() {
    assert!(motion_allowed(None, false));
    assert!(!motion_allowed(None, true));
    assert!(!motion_allowed(Some(true), false));
    assert!(motion_allowed(Some(false), true));
}

#[test]
fn swap_duration_and_focus_selector_are_fixed_contract_values() {
    assert_eq!(SWAP_DURATION_MS, 480);
    assert!(FOCUSABLE_SELECTOR.contains("button:not([disabled])"));
    assert!(FOCUSABLE_SELECTOR.contains("[tabindex]"));
}

fn blur_swap_app() -> Element {
    rsx! {
        BlurSwapContainer { content_key: "step-1",
            p { "Step one body" }
        }
    }
}

#[test]
fn blur_swap_renders_one_settled_layer_with_no_outgoing_layer() {
    let html = render(blur_swap_app);
    assert!(html.contains("kit-blur-swap__current"));
    assert!(html.contains("data-swap=\"0\""));
    assert!(html.contains("Step one body"));
    assert!(!html.contains("kit-blur-swap__outgoing"));
}

#[test]
fn split_emphasis_defaults_to_the_whole_text() {
    let parts = split_emphasis("Build faster", None);
    assert_eq!(parts.before, "");
    assert_eq!(parts.emphasis, "Build faster");
    assert_eq!(parts.after, "");
}

#[test]
fn split_emphasis_cuts_by_characters_not_bytes() {
    let parts = split_emphasis("café latte", Some(0..4));
    assert_eq!(parts.emphasis, "café");
    assert_eq!(parts.after, " latte");
}

#[test]
fn split_emphasis_clamps_an_out_of_range_end() {
    let parts = split_emphasis("ship it", Some(5..99));
    assert_eq!(parts.before, "ship ");
    assert_eq!(parts.emphasis, "it");
    assert_eq!(parts.after, "");
}

#[test]
fn gradient_token_is_reduced_to_safe_characters() {
    assert_eq!(token_name("accent-2"), "accent-2");
    assert_eq!(token_name("a b;}\"x"), "abx");
    assert_eq!(
        gradient_fill("accent"),
        "var(--kit-gradient-accent, linear-gradient(90deg, var(--kit-ok), var(--kit-warn), var(--kit-ok)))"
    );
}

#[test]
fn empty_gradient_token_falls_back_to_the_ink_gradient() {
    let fill = gradient_fill("!!!");
    assert!(!fill.contains("--kit-gradient-"));
    assert!(fill.starts_with("linear-gradient("));
}

#[test]
fn gradient_phases_have_stable_attribute_values() {
    assert_eq!(GradientPhase::default(), GradientPhase::Waiting);
    assert_eq!(GradientPhase::Waiting.as_str(), "waiting");
    assert_eq!(GradientPhase::Sweeping.as_str(), "sweeping");
    assert_eq!(GradientPhase::Flowing.as_str(), "flowing");
}

fn gradient_app() -> Element {
    rsx! {
        FlowingGradientText {
            text: "Build faster",
            emphasis_range: 6..12,
            replay_enabled: true,
            gradient_token: "accent",
        }
    }
}

fn gradient_no_view_app() -> Element {
    rsx! {
        FlowingGradientText { text: "Ship", play_on_view: false }
    }
}

#[test]
fn gradient_text_waits_for_view_and_names_its_replay_control() {
    let html = render(gradient_app);
    assert!(html.contains("data-phase=\"waiting\""));
    assert!(html.contains("aria-label=\"Replay animation\""));
    assert!(html.contains("kit-gradient-text__replay"));
    assert!(html.contains("--kit-flow-fill:"));
    assert!(html.contains("--kit-gradient-accent"));
}

#[test]
fn gradient_text_keeps_plain_text_outside_the_emphasised_range() {
    let html = render(gradient_app);
    let fill = html.find("kit-gradient-text__fill").expect("fill rendered");
    let plain = html.find("Build ").expect("leading text rendered");
    let emphasis = html.find("faster").expect("emphasis rendered");
    assert!(plain < fill && fill < emphasis);
}

#[test]
fn gradient_text_without_play_on_view_starts_its_sweep_at_mount() {
    let html = render(gradient_no_view_app);
    assert!(html.contains("data-phase=\"sweeping\""));
    assert!(!html.contains("kit-gradient-text__replay"));
}

#[test]
fn glow_intensity_opacity_increases_with_strength() {
    let subtle = intensity_opacity(GlowIntensity::Subtle);
    let standard = intensity_opacity(GlowIntensity::Standard);
    let strong = intensity_opacity(GlowIntensity::Strong);
    assert!(subtle < standard && standard < strong);
    assert!(strong <= 1.0);
}

#[test]
fn glow_layers_drift_on_different_periods_with_no_short_shared_loop() {
    let [first, second] = LAYER_DURATIONS_MS;
    assert_ne!(first, second);
    let (a, b) = (u64::from(first), u64::from(second));
    let lcm = a / gcd(a, b) * b;
    assert!(lcm >= 600_000, "combined loop {lcm}ms repeats too soon");
}

#[test]
fn glow_colour_uses_the_palette_token_or_the_rule_fallback() {
    assert_eq!(glow_color("warm"), "var(--kit-glow-warm, var(--kit-rule))");
    assert_eq!(glow_color(""), "var(--kit-rule)");
    assert!(!glow_color("x}{").contains('{'));
}

fn glow_app() -> Element {
    rsx! {
        GlowFloorBackdrop {
            intensity: GlowIntensity::Strong,
            anchor: GlowAnchor::Top,
            paused: true,
            palette_token: "warm",
            p { "Hero copy" }
        }
    }
}

#[test]
fn glow_backdrop_is_decorative_and_sits_behind_the_content() {
    let html = render(glow_app);
    let backdrop = html.find("kit-glow__backdrop").expect("backdrop rendered");
    let content = html.find("kit-glow__content").expect("content rendered");
    assert!(backdrop < content, "backdrop must precede content");
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(html.contains("Hero copy"));
}

#[test]
fn glow_backdrop_states_reach_the_markup() {
    let html = render(glow_app);
    assert!(html.contains("data-intensity=\"strong\""));
    assert!(html.contains("data-anchor=\"top\""));
    assert!(html.contains("data-paused=\"true\""));
    assert!(html.contains("--kit-glow-opacity: 0.9"));
}

#[test]
fn unit_pointer_maps_corners_and_centre() {
    assert_eq!(unit_pointer(0.0, 0.0, 200.0, 100.0), (-1.0, -1.0));
    assert_eq!(unit_pointer(200.0, 100.0, 200.0, 100.0), (1.0, 1.0));
    assert_eq!(unit_pointer(100.0, 50.0, 200.0, 100.0), (0.0, 0.0));
    assert_eq!(unit_pointer(999.0, -999.0, 200.0, 100.0), (1.0, -1.0));
    assert_eq!(unit_pointer(10.0, 10.0, 0.0, 0.0), (0.0, 0.0));
}

#[test]
fn tilt_leans_the_pointer_side_toward_the_viewer() {
    let (rx, ry) = tilt_degrees(TiltStrength::Standard, (1.0, -1.0));
    assert!(rx < 0.0, "top edge comes forward for a top pointer");
    assert!(ry < 0.0, "right edge comes forward for a right pointer");
}

#[test]
fn tilt_is_flat_at_the_centre_and_scales_with_strength() {
    assert_eq!(tilt_degrees(TiltStrength::Strong, (0.0, 0.0)), (0.0, 0.0));
    assert_eq!(tilt_degrees(TiltStrength::Subtle, (1.0, 0.0)), (0.0, -4.0));
    assert_eq!(tilt_degrees(TiltStrength::Strong, (0.0, 1.0)), (12.0, 0.0));
}

#[test]
fn glare_sits_opposite_the_pointer() {
    assert_eq!(glare_percent((0.0, 0.0)), (50.0, 50.0));
    assert_eq!(glare_percent((1.0, -1.0)), (0.0, 100.0));
}

fn tilt_app() -> Element {
    rsx! {
        PointerTiltCard { touch_behaviour: TouchBehaviour::TapToTilt,
            button { "Choose plan" }
        }
    }
}

fn tilt_no_glare_app() -> Element {
    rsx! {
        PointerTiltCard { glare: false, max_tilt: TiltStrength::Strong,
            p { "Plain card" }
        }
    }
}

#[test]
fn tilt_card_keeps_interactive_children_and_marks_its_glare_decorative() {
    let html = render(tilt_app);
    assert!(html.contains("<button"));
    assert!(html.contains("Choose plan"));
    assert!(html.contains("kit-tilt__glare"));
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(html.contains("data-touch=\"tap_to_tilt\""));
    assert!(html.contains("data-strength=\"standard\""));
}

#[test]
fn tilt_card_without_glare_renders_no_glare_layer() {
    let html = render(tilt_no_glare_app);
    assert!(!html.contains("kit-tilt__glare"));
    assert!(html.contains("data-strength=\"strong\""));
    assert!(html.contains("data-touch=\"none\""));
    assert!(html.contains("Plain card"));
}
