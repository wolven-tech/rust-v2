use dioxus::prelude::*;
use rv2_kit::motion::blur_swap_container::{
    BlurSwapContainer, BlurSwapStepCard, FOCUSABLE_SELECTOR, SWAP_DURATION_MS, StepBody,
    motion_allowed, next_step, swap_needed,
};
use rv2_kit::motion::flowing_gradient_text::{
    FlowingGradientText, GradientPhase, gradient_fill, last_words_range, split_emphasis, token_name,
};
use rv2_kit::motion::glow_floor_backdrop::{
    GlowAnchor, GlowFloorBackdrop, GlowIntensity, LAYER_DURATIONS_MS, glow_color, intensity_opacity,
};
use rv2_kit::motion::pointer_tilt_card::{
    MembershipCard, PointerTiltCard, TiltStrength, TouchBehaviour, glare_percent, masked_number,
    tilt_degrees, unit_pointer,
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
        "var(--kit-gradient-accent, linear-gradient(90deg, var(--kit-accent), var(--kit-id-1)))"
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
fn glow_colour_uses_the_palette_token_or_the_first_identity_colour() {
    assert_eq!(glow_color("warm"), "var(--kit-glow-warm, var(--kit-rule))");
    assert_eq!(glow_color(""), "var(--kit-id-1)");
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
    assert!(html.contains("--kit-glow-opacity: 1;"));
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

fn step(tag: &str, sentence: &str) -> StepBody {
    StepBody {
        tag: tag.to_string(),
        sentence: sentence.to_string(),
    }
}

fn step_card_app() -> Element {
    let steps = vec![
        step("Live", "Offline mode now works on phones."),
        step("Up next", "Bulk export is ready for review."),
        step("Done", "Every change in this version has a named owner."),
    ];
    rsx! {
        BlurSwapStepCard {
            title: "Version 3.1",
            status: "2 changes · draft",
            primary_label: "Publish",
            steps,
            current: 0usize,
            on_next: move |_| {},
        }
    }
}

#[test]
fn step_card_shows_header_body_and_footer_with_step_one_current() {
    let html = render(step_card_app);
    assert!(html.contains("kit-blur-step__header"));
    assert!(html.contains("Version 3.1"));
    assert!(html.contains("Offline mode now works on phones."));
    assert!(html.contains("kit-blur-step__footer"));
    assert_eq!(html.matches("class=\"kit-blur-step__dot\"").count(), 3);
    assert_eq!(html.matches("data-current=\"true\"").count(), 1);
    assert!(html.contains(">Next<"));
    assert!(!html.contains("kit-blur-swap__outgoing"));
}

#[test]
fn next_step_advances_and_returns_to_the_first_step_after_the_last() {
    assert_eq!(next_step(0, 3), 1);
    assert_eq!(next_step(2, 3), 0);
    assert_eq!(next_step(0, 0), 0);
}

#[test]
fn emphasis_covers_the_last_two_words_of_the_headline() {
    let text = "Every figure traces back to its source.";
    let range = last_words_range(text, 2);
    assert_eq!(split_emphasis(text, Some(range)).emphasis, "its source.");
}

fn headline_app() -> Element {
    rsx! {
        FlowingGradientText {
            text: "Every figure traces back to its source.",
            emphasis_range: Some(last_words_range("Every figure traces back to its source.", 2)),
            replay_enabled: true,
            class: "kit-gradient-text--display",
        }
    }
}

#[test]
fn headline_places_its_replay_control_after_the_text_as_a_block() {
    let html = render(headline_app);
    assert!(html.contains("kit-gradient-text--display"));
    assert!(html.contains("aria-label=\"Replay animation\""));
    let emphasis = html.find("its source.").expect("emphasis rendered");
    let replay = html
        .find("kit-gradient-text__replay")
        .expect("replay rendered");
    assert!(emphasis < replay);
}

#[test]
fn masked_number_shows_bullets_and_the_last_four_digits() {
    assert_eq!(masked_number("4111 1111 1111 2750"), "•••• •••• •••• 2750");
    assert_eq!(masked_number("12"), "•••• •••• •••• 12");
}

fn membership_app() -> Element {
    rsx! {
        MembershipCard {
            name: "Ledger Plus",
            holder: "Jamie Rivera",
            since: "03/24",
            last_four: "2750",
            caption: "Move the pointer over the card",
        }
    }
}

#[test]
fn membership_card_shows_its_fields_and_caption_flat_at_rest() {
    let html = render(membership_app);
    for expected in [
        "Ledger Plus",
        "kit-tilt-card__chip",
        "kit-tilt-card__contactless",
        "•••• •••• •••• 2750",
        "Member",
        "Jamie Rivera",
        "Since",
        "03/24",
        "Move the pointer over the card",
    ] {
        assert!(html.contains(expected), "missing {expected}");
    }
    assert!(!html.contains("data-active"));
}
