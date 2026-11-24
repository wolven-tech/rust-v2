use dioxus::prelude::*;
use rv2_kit::blocks::chart_answer_chat::{
    AnswerBlock, ChartAnswerChat, ChatMessage, Figure, Prompt, Role, TrendPoint, x_pct, y_pct,
};
use rv2_kit::illustrations::frosted_folder_mark::{FrostedFolderMark, SizeStep, folder_scale};
use rv2_kit::illustrations::integration_arc_flow::{
    Hub, Integration, IntegrationArcFlow, IntegrationStatus, pill_left_pct, spine_opacity,
};
use rv2_kit::illustrations::model_picker_scene::{
    ModelGroup, ModelOption, ModelPickerScene, row_opacity, row_top_pct,
};
use rv2_kit::illustrations::outreach_fanout_lines::{
    BRANCH_POINT, DESTINATION_CENTRE_X, DESTINATION_SIDE, DESTINATION_TOP, Endpoint,
    OutreachFanoutLines, TILE_BOTTOM, branch_path,
};
use rv2_kit::illustrations::voice_assistant_orb::{
    LATITUDE_RINGS, OrbState, VoiceAssistantOrb, lattice_dots,
};
use rv2_kit::motion::glow_floor_backdrop::{GlowFloorBackdrop, TOP_EDGE, field_path};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn css_rule<'a>(css: &'a str, selector: &str) -> &'a str {
    css.split(&format!("{selector} {{"))
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .unwrap_or_else(|| panic!("rule {selector} missing"))
}

const FOLDER_CSS: &str = include_str!("../assets/frosted-folder-mark.css");

#[test]
fn folder_stage_is_five_to_three_and_the_folder_box_is_a_quarter_of_its_width() {
    let stage = css_rule(FOLDER_CSS, ".kit-folder");
    assert!(stage.contains("aspect-ratio: 5 / 3"));
    let folder = css_rule(FOLDER_CSS, ".kit-folder__box");
    assert!(folder.contains("width: 25%"));
    assert!(folder.contains("height: 36%"));
    assert!(folder.contains("top: 51%"));
}

#[test]
fn folder_sheets_top_at_twelve_and_the_pocket_at_twenty_one_percent_of_the_folder() {
    assert!(css_rule(FOLDER_CSS, ".kit-folder__sheet").contains("top: 12%"));
    assert!(FOLDER_CSS.contains(".kit-folder__pocket {\n  left: 0;\n  right: 0;\n  top: 21%;"));
    assert!(css_rule(FOLDER_CSS, ".kit-folder__back").contains("background: var(--kit-accent)"));
    assert!(css_rule(FOLDER_CSS, ".kit-folder__tab").contains("background: var(--kit-accent)"));
}

#[test]
fn folder_size_steps_keep_the_folder_centred_and_scale_it_about_its_centre() {
    assert_eq!(folder_scale(SizeStep::Md), 1.0);
    assert!(folder_scale(SizeStep::Sm) < folder_scale(SizeStep::Md));
    assert!(folder_scale(SizeStep::Lg) > folder_scale(SizeStep::Md));
    assert!(FOLDER_CSS.contains("translate(-50%, -50%) scale(var(--kit-folder-scale, 1))"));
}

fn folder_app() -> Element {
    rsx! { FrostedFolderMark {} }
}

#[test]
fn folder_at_rest_draws_one_folder_with_a_shadow_and_no_neutral_dark_shape() {
    let html = render(folder_app);
    assert_eq!(html.matches("class=\"kit-folder__pocket\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-folder__shadow\"").count(), 1);
    assert!(!html.contains("--kit-folder-size"));
}

const ARC_CSS: &str = include_str!("../assets/integration-arc-flow.css");

#[test]
fn arc_slots_bow_from_the_centre_pill_outward() {
    assert_eq!(pill_left_pct(0), 56.7);
    assert_eq!(pill_left_pct(1), 61.0);
    assert_eq!(pill_left_pct(-1), 61.0);
    assert_eq!(pill_left_pct(2), 71.0);
    assert_eq!(spine_opacity(0), 1.0);
    assert_eq!(spine_opacity(1), 0.5);
    assert!(spine_opacity(2) < spine_opacity(1));
}

#[test]
fn arc_hub_spans_eight_to_forty_three_percent_and_is_centred_on_the_centre_pill() {
    let hub = css_rule(ARC_CSS, ".kit-arc__hub");
    assert!(hub.contains("left: 8%"));
    assert!(hub.contains("width: 35%"));
    assert!(hub.contains("top: 42.5%"));
    assert!(hub.contains("height: 15%"));
    assert!(ARC_CSS.contains("top: calc(42.5% + var(--s) * 26.5%)"));
}

#[test]
fn arc_kind_line_is_larger_than_the_name_line() {
    assert!(css_rule(ARC_CSS, ".kit-arc__kind").contains("font-size: var(--kit-text-sm)"));
    assert!(ARC_CSS.contains(".kit-arc__name {\n  font-size: var(--kit-text-xs);"));
}

#[test]
fn arc_guide_is_dashed_and_the_stage_is_the_card_with_no_inner_border() {
    assert!(css_rule(ARC_CSS, ".kit-arc__guide path").contains("stroke-dasharray"));
    assert!(!css_rule(ARC_CSS, ".kit-arc").contains("border:"));
}

fn arc_app() -> Element {
    let integration = |id: &str, label: &str| Integration {
        id: id.to_string(),
        label: label.to_string(),
        status: IntegrationStatus::Live,
        logo: None,
    };
    rsx! {
        IntegrationArcFlow {
            hub: Hub { title: "Analysis hub".to_string(), subtitle: "Every source in one view".to_string() },
            integrations: vec![
                integration("helpdesk", "Helpdesk"),
                integration("warehouse", "Warehouse"),
                integration("chat", "Chat"),
                integration("tracker", "Tracker"),
                integration("payments", "Payments"),
            ],
        }
    }
}

#[test]
fn arc_at_rest_has_five_slots_and_the_centre_pill_is_the_only_full_strength_one() {
    let html = render(arc_app);
    assert_eq!(html.matches("class=\"kit-arc__pill\"").count(), 5);
    assert_eq!(html.matches("data-centre=\"true\"").count(), 1);
    assert!(html.contains("left: 56.7%;"));
    assert!(html.contains("--o: 0.50;"));
    assert!(html.contains("--o: 0.33;"));
}

const PICKER_CSS: &str = include_str!("../assets/model-picker-scene.css");

fn seven_models() -> Vec<ModelGroup> {
    let names = [
        "Flash 2",
        "Deep reason",
        "Large context",
        "Compact 3",
        "Vision pro",
        "Code mini",
        "Legacy v1",
    ];
    vec![ModelGroup {
        label: "Fast models".to_string(),
        models: names
            .iter()
            .enumerate()
            .map(|(i, name)| ModelOption {
                id: format!("m{i}"),
                name: name.to_string(),
                vendor_mark: None,
                available: true,
            })
            .collect(),
    }]
}

#[test]
fn picker_rows_fade_in_the_last_two_and_stay_on_one_grid() {
    assert_eq!(row_opacity(4), 1.0);
    assert_eq!(row_opacity(5), 0.6);
    assert_eq!(row_opacity(6), 0.35);
    let groups = seven_models();
    let first = row_top_pct(&groups, "m0").expect("row 1");
    let last = row_top_pct(&groups, "m6").expect("row 7");
    assert!(last > first);
    assert!((last - first - 6.0 * (50.0 / 6.0) / 72.0 * 100.0).abs() < 1e-9);
}

fn picker_app() -> Element {
    rsx! { ModelPickerScene { groups: seven_models(), default_id: "m1" } }
}

#[test]
fn picker_at_rest_shows_one_heading_seven_rows_and_one_selected_row() {
    let html = render(picker_app);
    assert_eq!(html.matches("class=\"kit-model__heading\"").count(), 1);
    assert_eq!(html.matches("role=\"option\"").count(), 7);
    assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
    assert!(!html.contains("unavailable"));
    assert!(html.contains("opacity: 0.60;"));
    assert!(html.contains("opacity: 0.35;"));
}

#[test]
fn picker_stage_is_a_card_at_one_point_eight_five_to_one_with_the_list_at_thirty_five_percent() {
    assert!(PICKER_CSS.contains("aspect-ratio: 1.85 / 1;"));
    let panel = css_rule(
        PICKER_CSS,
        ".kit-model:not([data-popover]) .kit-model__panel",
    );
    assert!(panel.contains("left: 35%"));
    assert!(panel.contains("width: 42%"));
    assert!(panel.contains("top: 20%"));
    assert!(panel.contains("height: 72%"));
    assert!(PICKER_CSS.contains("top: 83.33%;"));
}

const FANOUT_CSS: &str = include_str!("../assets/outreach-fanout-lines.css");

#[test]
fn fanout_junction_sits_at_the_centre_and_twenty_nine_percent_of_the_height() {
    assert_eq!(BRANCH_POINT, (50.0, 29.0));
}

#[test]
fn fanout_tiles_share_a_bottom_edge_and_the_centre_tile_is_larger() {
    assert_eq!(TILE_BOTTOM, 68.7);
    assert_eq!(DESTINATION_SIDE, [9.6, 11.2, 9.6]);
    assert_eq!(DESTINATION_TOP[0], 52.9);
    assert_eq!(DESTINATION_TOP[1], 49.8);
    assert_eq!(DESTINATION_CENTRE_X, [34.7, 50.0, 65.3]);
}

#[test]
fn fanout_centre_curve_is_straight_and_outer_curves_arrive_vertically() {
    assert!(branch_path(1).contains(" L 50 "));
    let outer = branch_path(0);
    assert!(outer.contains(" C "));
    assert!(outer.contains(" 34.7 "));
}

#[test]
fn fanout_stage_is_the_card_at_thirty_three_to_twenty() {
    assert!(FANOUT_CSS.contains("aspect-ratio: 33 / 20;"));
    assert!(css_rule(FANOUT_CSS, ".kit-fan__tile").contains("translate: -50% -100%"));
}

fn fanout_app() -> Element {
    let endpoint = |id: &str, label: &str| Endpoint {
        id: id.to_string(),
        label: label.to_string(),
        icon: None,
    };
    rsx! {
        OutreachFanoutLines {
            destinations: vec![endpoint("mail", "Email"), endpoint("chat", "Chat"), endpoint("docs", "Documents")],
        }
    }
}

#[test]
fn fanout_at_rest_draws_one_junction_three_curves_and_three_icon_only_tiles() {
    let html = render(fanout_app);
    assert_eq!(html.matches("class=\"kit-fan__junction\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-fan__branch").count(), 3);
    assert_eq!(html.matches("class=\"kit-fan__tile\"").count(), 3);
    assert_eq!(html.matches("data-index=").count(), 3);
    assert_eq!(html.matches("kit-fan__pulse").count(), 3);
}

const ORB_CSS: &str = include_str!("../assets/voice-assistant-orb.css");

#[test]
fn orb_lattice_has_fifteen_rings_and_between_one_fifty_and_two_hundred_front_dots() {
    let dots = lattice_dots();
    assert_eq!(LATITUDE_RINGS, 15);
    assert!((150..=200).contains(&dots.len()), "{} dots", dots.len());
    let mut heights: Vec<i64> = dots.iter().map(|d| (d.y * 1000.0).round() as i64).collect();
    heights.sort_unstable();
    heights.dedup();
    assert_eq!(heights.len(), LATITUDE_RINGS);
    assert!(dots.iter().all(|d| d.z >= 0.0));
}

#[test]
fn orb_sphere_is_seventeen_percent_of_the_stage_and_centred_at_forty_one() {
    let sphere = css_rule(ORB_CSS, ".kit-orb__button,\n.kit-orb__hit");
    assert!(sphere.contains("width: 17%"));
    assert!(sphere.contains("top: 41%"));
    assert!(ORB_CSS.contains("aspect-ratio: 1.85 / 1;"));
}

#[test]
fn orb_status_word_and_switch_sit_at_sixty_seven_and_seventy_seven_and_a_half_percent() {
    assert!(
        css_rule(ORB_CSS, ".kit-orb__status,\n.kit-orb__label").contains("translate: -50% -50%")
    );
    assert!(css_rule(ORB_CSS, ".kit-orb__status").contains("top: 67%"));
    let switch = css_rule(ORB_CSS, ".kit-orb__switch");
    assert!(switch.contains("top: 77.5%"));
    assert!(switch.contains("width: 36%"));
}

fn orb_app() -> Element {
    rsx! { VoiceAssistantOrb { state: OrbState::Listening, level: 0.5, on_demo_state: |_| {} } }
}

#[test]
fn orb_at_rest_shows_listening_with_the_four_segment_switch_and_no_ripples() {
    let html = render(orb_app);
    assert!(html.contains("Listening…"));
    assert_eq!(html.matches("class=\"kit-orb__segment\"").count(), 4);
    assert_eq!(html.matches("kit-orb__ripple").count(), 0);
}

const GLOW_CSS: &str = include_str!("../assets/glow-floor-backdrop.css");

#[test]
fn glow_top_edge_peaks_in_the_left_fifth_at_fifty_five_to_sixty_percent() {
    let (x, height) = TOP_EDGE
        .iter()
        .copied()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("top edge");
    assert!((0.0..=20.0).contains(&x), "highest point at x {x}");
    assert!((55.0..=60.0).contains(&height), "highest point at {height}");
}

#[test]
fn glow_top_edge_stays_at_or_below_forty_five_outside_the_left_fifth() {
    assert!(
        TOP_EDGE
            .iter()
            .filter(|(x, _)| *x > 20.0)
            .all(|(_, height)| *height <= 45.0)
    );
    assert!(TOP_EDGE.iter().all(|(_, height)| *height <= 66.7));
}

#[test]
fn glow_top_edge_varies_by_at_least_fifteen_points_across_the_stage() {
    let high = TOP_EDGE.iter().map(|p| p.1).fold(f64::MIN, f64::max);
    let low = TOP_EDGE
        .iter()
        .filter(|(x, _)| (0.0..=100.0).contains(x))
        .map(|p| p.1)
        .fold(f64::MAX, f64::min);
    assert!(high - low >= 15.0, "varies by {}", high - low);
}

#[test]
fn glow_top_edge_has_no_flat_run_of_three_points_across_the_stage() {
    let heights: Vec<f64> = TOP_EDGE
        .iter()
        .filter(|(x, _)| (0.0..=100.0).contains(x))
        .map(|p| p.1)
        .collect();
    assert!(
        heights
            .windows(3)
            .all(|w| !((w[0] - w[1]).abs() < 0.01 && (w[1] - w[2]).abs() < 0.01)),
        "flat run in {heights:?}"
    );
}

#[test]
fn glow_top_edge_is_near_forty_percent_at_the_centre_and_forty_to_forty_five_at_the_right_edge() {
    let height_at = |x: f64| {
        TOP_EDGE
            .iter()
            .find(|(px, _)| (px - x).abs() < 0.01)
            .map(|p| p.1)
            .expect("top edge point")
    };
    assert!((38.0..=42.0).contains(&height_at(50.0)));
    assert!((40.0..=45.0).contains(&height_at(100.0)));
}

#[test]
fn glow_field_path_closes_below_the_stage_so_no_column_is_gapped() {
    let path = field_path();
    assert!(path.starts_with("M -10 50 "));
    assert!(path.ends_with("L -10 100 Z"));
}

#[test]
fn glow_stage_is_ninety_percent_of_the_card_width_and_seventy_percent_of_its_height() {
    let stage = css_rule(GLOW_CSS, ".kit-glow__stage");
    assert!(stage.contains("width: 90%"));
    assert!(stage.contains("height: 70%"));
    assert!(stage.contains("left: 5%"));
    assert!(stage.contains("top: 15%"));
}

fn glow_app() -> Element {
    rsx! { GlowFloorBackdrop { children: rsx! {} } }
}

#[test]
fn glow_at_rest_draws_one_field_in_two_layers_and_no_content() {
    let html = render(glow_app);
    assert_eq!(html.matches("class=\"kit-glow__layer\"").count(), 2);
    assert_eq!(html.matches("class=\"kit-glow__wash").count(), 2);
    assert_eq!(html.matches("class=\"kit-glow__shape\"").count(), 2);
    assert!(!html.contains("kit-glow__blob"));
    assert!(!html.contains("<button"));
}

const CHAT_CSS: &str = include_str!("../assets/chart-answer-chat.css");

#[test]
fn chart_axes_map_the_first_point_to_eight_percent_and_the_last_to_ninety_six() {
    assert_eq!(x_pct(0, 12), 8.0);
    assert!((x_pct(12, 12) - 96.0).abs() < 1e-9);
    assert_eq!(y_pct(10.0, 0.0, 10.0), 14.0);
    assert_eq!(y_pct(0.0, 0.0, 10.0), 78.0);
}

#[test]
fn chart_stage_is_one_point_one_five_to_one_and_the_column_spans_nine_to_ninety_one_percent() {
    assert!(CHAT_CSS.contains("aspect-ratio: 1.15 / 1;"));
    let log = css_rule(CHAT_CSS, ".kit-chat__log");
    assert!(log.contains("left: 9%"));
    assert!(log.contains("width: 82%"));
}

#[test]
fn chart_trend_card_is_36_point_2_cqh_tall_and_has_its_legend_at_the_top_right() {
    assert!(CHAT_CSS.contains("height: 36.2cqh;"));
    assert!(css_rule(CHAT_CSS, ".kit-chat__legend").contains("top: 3%"));
}

fn chat_app() -> Element {
    let points: Vec<TrendPoint> = (27..=39)
        .map(|w| TrendPoint {
            x_label: format!("W{w}"),
            value: if w == 36 { 1640.0 } else { 1200.0 },
            comparison_value: Some(1000.0),
        })
        .collect();
    let prompts = [
        "Break it down by region",
        "Forecast next quarter",
        "Why did week 36 jump?",
    ]
    .iter()
    .enumerate()
    .map(|(i, l)| Prompt {
        id: format!("p{i}"),
        label: l.to_string(),
    })
    .collect();
    rsx! {
        ChartAnswerChat {
            messages: vec![ChatMessage {
                id: "a1".into(),
                role: Role::Assistant,
                text: Some("Signups closed at 18.4k, 11% ahead of last year.".into()),
                blocks: vec![
                    AnswerBlock::Figures(vec![Figure {
                        label: "Signups".into(),
                        value: "18.4k".into(),
                        delta_percent: 11.0,
                    }]),
                    AnswerBlock::Trend {
                        block_id: "t1".into(),
                        series_label: "This period".into(),
                        comparison_label: "Last period".into(),
                        points,
                        hint: Some("Drag across the chart to ask about a span of weeks.".into()),
                    },
                ],
                failed: false,
            }],
            suggested_prompts: prompts,
            sources: vec!["CRM".to_string(), "Billing".to_string()],
        }
    }
}

#[test]
fn chart_at_rest_has_three_suggestions_two_sources_and_four_icon_buttons() {
    let html = render(chat_app);
    assert_eq!(html.matches("class=\"kit-chat__action\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-chat__source-chip\"").count(), 2);
    assert_eq!(html.matches("class=\"kit-chat__icon-btn\"").count(), 4);
    assert!(html.contains("kit-chat__source-chip\">CRM<"));
}

#[test]
fn chart_peak_tooltip_shows_week_and_both_series_at_rest() {
    let html = render(chat_app);
    assert!(html.contains("kit-chat__tip"));
    assert!(html.contains("W36"));
    assert!(html.contains("This period 1,640"));
    assert!(html.contains("Last period 1,000"));
}

#[test]
fn chart_has_three_value_labels_three_week_labels_and_a_legend() {
    let html = render(chat_app);
    assert_eq!(html.matches("class=\"kit-chat__axis ").count(), 3);
    assert_eq!(html.matches("class=\"kit-chat__week ").count(), 3);
    assert!(html.contains("kit-chat__legend"));
}

#[test]
fn chart_composer_puts_the_model_picker_right_of_the_text_area_and_the_send_control_last() {
    let html = render(chat_app);
    let plus = html.find("kit-chat__plus").expect("plus");
    let approval = html.find("kit-chat__approval").expect("approval");
    let model = html.find("kit-chat__model\"").expect("model");
    let send = html.find("kit-chat__send").expect("send");
    let input = html.find("kit-chat__input").expect("input");
    assert!(plus < approval && approval < model && model < send);
    assert!(
        input < model,
        "the model picker sits right of the text area"
    );
}

#[test]
fn chart_suggestion_row_sits_before_the_composer_box() {
    let html = render(chat_app);
    let prompts = html.find("kit-chat__prompts").expect("prompts");
    let box_at = html.find("kit-chat__box").expect("box");
    assert!(prompts < box_at);
}

const TIMELINE_CSS: &str = include_str!("../assets/agent-timeline-overview.css");
const GRADIENT_CSS: &str = include_str!("../assets/flowing-gradient-text.css");

#[test]
fn timeline_bar_labels_wrap_onto_a_second_line_before_they_truncate() {
    let span = css_rule(TIMELINE_CSS, ".kit-tl__span");
    assert!(span.contains("flex-wrap: wrap"));
    assert!(span.contains("white-space: normal"));
    let label = css_rule(TIMELINE_CSS, ".kit-tl__span-label");
    assert!(!label.contains("ellipsis"));
    assert!(css_rule(TIMELINE_CSS, ".kit-tl__track").contains("min-height: 48px"));
}

#[test]
fn replay_control_sits_a_gap_below_the_headline_like_the_other_demos() {
    let replay = css_rule(GRADIENT_CSS, ".kit-gradient-text__replay");
    assert!(replay.contains("margin: var(--kit-space-3) auto 0"));
}
