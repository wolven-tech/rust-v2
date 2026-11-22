use dioxus::prelude::*;
use rv2_kit::illustrations::frosted_folder_mark::{FrostedFolderMark, SizeStep, folder_px};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn folder_sizes_step_up_in_px() {
    assert_eq!(folder_px(SizeStep::Sm), 48);
    assert_eq!(folder_px(SizeStep::Md), 72);
    assert_eq!(folder_px(SizeStep::Lg), 96);
}

fn folder_named_app() -> Element {
    rsx! { FrostedFolderMark { label: "Client documents", size_token: SizeStep::Lg } }
}

#[test]
fn folder_with_label_is_a_named_image() {
    let html = render(folder_named_app);
    assert!(html.contains("role=\"img\""));
    assert!(html.contains("aria-label=\"Client documents\""));
    assert!(!html.contains("aria-hidden"));
    assert!(html.contains("--kit-folder-size: 96px;"));
    assert!(html.contains("data-size=\"lg\""));
}

fn folder_decorative_app() -> Element {
    rsx! { FrostedFolderMark {} }
}

#[test]
fn folder_without_label_is_hidden_from_assistive_tech() {
    let html = render(folder_decorative_app);
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(!html.contains("role=\"img\""));
    assert!(html.contains("kit-folder__pocket"));
    assert!(html.contains("kit-folder__sheet"));
}

use rv2_kit::illustrations::progress_pill_scene::{
    PillState, ProgressPillScene, clamp_progress, percent_text, percent_value,
};

#[test]
fn progress_clamps_and_rounds_for_the_percentage() {
    assert_eq!(clamp_progress(1.7), 1.0);
    assert_eq!(clamp_progress(-0.2), 0.0);
    assert_eq!(clamp_progress(f64::NAN), 0.0);
    assert_eq!(percent_value(0.426), 43);
    assert_eq!(percent_text(0.5, PillState::Working), "50%");
    assert_eq!(percent_text(0.5, PillState::Paused), "50%");
    assert_eq!(percent_text(1.0, PillState::Done), "Done");
}

fn pill_working_app() -> Element {
    rsx! {
        ProgressPillScene {
            title: "Q3 pricing memo",
            agent_label: "Research agent",
            source_line: "12 sources",
            progress: 0.42,
            step_label: "Comparing competitor tiers",
        }
    }
}

#[test]
fn pill_renders_percentage_step_and_bar_value() {
    let html = render(pill_working_app);
    assert!(html.contains("data-state=\"working\""));
    assert!(html.contains("42%"));
    assert!(html.contains("aria-valuenow=\"42\""));
    assert!(html.contains("transform: scaleX(0.42);"));
    assert!(html.contains("role=\"status\""));
    assert!(html.contains("Comparing competitor tiers"));
    assert!(!html.contains("kit-pill-scene__button"));
}

fn pill_done_app() -> Element {
    rsx! {
        ProgressPillScene {
            title: "Launch checklist",
            agent_label: "Ops agent",
            source_line: "3 sources",
            progress: 1.0,
            step_label: "Delivered",
            state: PillState::Done,
            interactive: true,
            on_open: |_| {},
        }
    }
}

use rv2_kit::illustrations::dictation_capture_panel::{
    BAR_COUNT, DictationCapturePanel, DictationState, bar_heights, format_elapsed, loudness,
};

#[test]
fn elapsed_time_reads_as_mm_ss() {
    assert_eq!(format_elapsed(0), "00:00");
    assert_eq!(format_elapsed(125), "02:05");
}

#[test]
fn waveform_is_right_aligned_and_padded_with_zero_bars() {
    assert_eq!(bar_heights(&[0.2, 0.9], 4), vec![0.0, 0.0, 0.2, 0.9]);
    assert_eq!(bar_heights(&[1.5, f64::NAN], 2), vec![1.0, 0.0]);
    assert_eq!(bar_heights(&[], 3), vec![0.0, 0.0, 0.0]);
}

#[test]
fn loudness_averages_the_recent_window() {
    assert_eq!(loudness(&[]), 0.0);
    assert_eq!(loudness(&[1.0; 10]), 1.0);
    assert!((loudness(&[0.0, 0.0, 0.5, 0.5]) - 0.25).abs() < 1e-9);
}

fn dictation_listening_app() -> Element {
    rsx! {
        DictationCapturePanel {
            state: DictationState::Listening,
            level_samples: vec![0.3, 0.7, 0.5],
            elapsed_secs: 65,
            on_finish: |_| {},
            on_cancel: |_| {},
        }
    }
}

#[test]
fn listening_panel_is_a_named_region_with_live_status_and_time() {
    let html = render(dictation_listening_app);
    assert!(html.contains("aria-label=\"Dictation\""));
    assert!(html.contains("data-state=\"listening\""));
    assert!(html.contains("01:05"));
    assert!(html.contains("Listening"));
    assert_eq!(html.matches("kit-dict__bar").count(), BAR_COUNT);
}

fn dictation_idle_app() -> Element {
    rsx! { DictationCapturePanel { elapsed_secs: 90 } }
}

#[test]
fn idle_panel_shows_zero_time_and_the_start_prompt() {
    let html = render(dictation_idle_app);
    assert!(html.contains("00:00"));
    assert!(html.contains("Ready to listen"));
    assert!(html.contains("disabled"));
}

#[test]
fn done_pill_reads_done_and_interactive_mode_is_a_named_button() {
    let html = render(pill_done_app);
    assert!(html.contains("Done"));
    assert!(html.contains("data-state=\"done\""));
    assert!(html.contains("<button"));
    assert!(html.contains("aria-label=\"Launch checklist, Done\""));
}

use rv2_kit::illustrations::voice_assistant_orb::{
    DOT_COUNT, OrbState, VoiceAssistantOrb, clamp_level, dot_opacity, dot_sphere, ring_points,
    state_label,
};

#[test]
fn sphere_has_two_thirds_of_its_dots_on_the_front_hemisphere() {
    let dots = dot_sphere(DOT_COUNT);
    assert_eq!(dots.len(), DOT_COUNT);
    let front = dots.iter().filter(|d| d.z >= 0.0).count();
    assert_eq!(front, DOT_COUNT * 2 / 3);
    for d in &dots {
        let r = (d.x * d.x + d.y * d.y + d.z * d.z).sqrt();
        assert!((r - 1.0).abs() < 1e-9);
    }
}

#[test]
fn ring_points_sit_on_the_unit_circle_and_depth_sets_opacity() {
    let ring = ring_points(8);
    assert_eq!(ring.len(), 8);
    assert!((ring[0].0 - 1.0).abs() < 1e-9 && ring[0].1.abs() < 1e-9);
    assert!((ring[2].0).abs() < 1e-9 && (ring[2].1 - 1.0).abs() < 1e-9);
    assert!((dot_opacity(1.0) - 1.0).abs() < 1e-9);
    assert!((dot_opacity(-1.0) - 0.3).abs() < 1e-9);
    assert_eq!(clamp_level(f64::NAN), 0.0);
    assert_eq!(clamp_level(3.0), 1.0);
    assert_eq!(state_label(OrbState::Thinking), "Thinking");
}

fn orb_speaking_app() -> Element {
    rsx! {
        VoiceAssistantOrb { state: OrbState::Speaking, level: 0.6, label: "Assistant" }
    }
}

#[test]
fn speaking_orb_is_decorative_with_status_text_and_three_ripples() {
    let html = render(orb_speaking_app);
    assert!(html.contains("data-state=\"speaking\""));
    assert_eq!(html.matches("kit-orb__ripple").count(), 3);
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(!html.contains("<button"));
    assert!(html.contains("Speaking"));
    assert_eq!(html.matches("kit-orb__dot\"").count(), DOT_COUNT);
}

fn orb_interactive_app() -> Element {
    rsx! { VoiceAssistantOrb { interactive: true, on_press: |_| {} } }
}

#[test]
fn interactive_orb_is_a_button_named_voice_assistant() {
    let html = render(orb_interactive_app);
    assert!(html.contains("<button"));
    assert!(html.contains("aria-label=\"Voice assistant\""));
    assert!(html.contains("Idle"));
}

use rv2_kit::illustrations::outreach_fanout_lines::{
    Endpoint, OutreachFanoutLines, branch_path, pulse_begin, shown_destinations,
};

fn endpoint(id: &str, label: &str) -> Endpoint {
    Endpoint {
        id: id.to_string(),
        label: label.to_string(),
        icon: None,
    }
}

#[test]
fn fanout_draws_at_most_three_destinations_and_staggers_pulses() {
    let four = [1, 2, 3, 4];
    assert_eq!(shown_destinations(&four), &[1, 2, 3]);
    assert!((pulse_begin(2) - 1.2).abs() < 1e-9);
    assert!(branch_path(0).ends_with("16 72"));
}

fn fanout_app() -> Element {
    rsx! {
        OutreachFanoutLines {
            source: endpoint("src", "Outbox"),
            destinations: vec![
                endpoint("mail", "Mail"),
                endpoint("chat", "Chat"),
                endpoint("crm", "CRM"),
                endpoint("sms", "Text"),
            ],
            active_destination_id: "chat",
        }
    }
}

#[test]
fn fanout_is_decorative_with_three_branches_pulses_and_tiles() {
    let html = render(fanout_app);
    assert!(html.contains("aria-hidden=\"true\""));
    assert_eq!(html.matches("kit-fan__pulse").count(), 3);
    assert_eq!(html.matches("data-index=").count(), 3);
    assert!(!html.contains("Text"));
    assert!(html.contains("kit-fan__branch is-active"));
    assert!(!html.contains("<button"));
}

use rv2_kit::illustrations::task_branch_scene::{
    Assignee, ClientNode, Note, Person, Role, TaskBranchScene, TaskCard, initials,
};

#[test]
fn initials_take_the_first_letter_of_up_to_two_words() {
    assert_eq!(initials("Ada Lovelace"), "AL");
    assert_eq!(initials("grace hopper weaver"), "GH");
    assert_eq!(initials(""), "");
}

fn branch_app() -> Element {
    rsx! {
        TaskBranchScene {
            client: ClientNode {
                label: "Northwind Foods".to_string(),
                connection_status: "Connected".to_string(),
            },
            task: TaskCard {
                title: "Q4 onboarding deck".to_string(),
                assignees: vec![
                    Assignee { id: "ada".to_string(), avatar: "AL".to_string() },
                    Assignee { id: "sam".to_string(), avatar: "SR".to_string() },
                ],
                tool_icons: vec![],
            },
            notes: vec![Note { id: "n1".to_string(), label: "Kickoff call".to_string() }],
            roles: vec![Role {
                id: "design".to_string(),
                title: "Design lead".to_string(),
                person: Person { name: "Ada Lovelace".to_string(), avatar: "AL".to_string() },
                tool_icons: vec![],
            }],
        }
    }
}

#[test]
fn branch_scene_lists_each_assignee_and_is_decorative() {
    let html = render(branch_app);
    assert_eq!(html.matches("kit-branch__avatar\" data-id=").count(), 2);
    assert!(html.contains("data-id=\"ada\""));
    assert!(html.contains("data-id=\"sam\""));
    assert!(html.contains("Connected"));
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(html.contains("kit-branch__node--role"));
    assert!(!html.contains("data-hot"));
}

use rv2_kit::illustrations::model_picker_scene::{
    ModelGroup, ModelOption, ModelPickerScene, ROW_PX, Step, available_ids, row_offset, step_active,
};

fn option(id: &str, name: &str, available: bool) -> ModelOption {
    ModelOption {
        id: id.to_string(),
        name: name.to_string(),
        vendor_mark: None,
        available,
    }
}

fn picker_groups() -> Vec<ModelGroup> {
    vec![
        ModelGroup {
            label: "Recommended".to_string(),
            models: vec![option("fast", "Flash", true), option("deep", "Deep", true)],
        },
        ModelGroup {
            label: "Other".to_string(),
            models: vec![
                option("legacy", "Legacy", false),
                option("large", "Large", true),
            ],
        },
    ]
}

#[test]
fn highlight_offset_counts_headings_and_rows() {
    let groups = picker_groups();
    assert_eq!(row_offset(&groups, "fast"), Some(28));
    assert_eq!(row_offset(&groups, "deep"), Some(28 + ROW_PX));
    assert_eq!(
        row_offset(&groups, "large"),
        Some(28 + 2 * ROW_PX + 28 + ROW_PX)
    );
    assert_eq!(row_offset(&groups, "missing"), None);
}

#[test]
fn arrow_steps_skip_unavailable_models_and_clamp() {
    let ids = available_ids(&picker_groups());
    assert_eq!(ids, vec!["fast", "deep", "large"]);
    assert_eq!(
        step_active(&ids, Some("deep"), Step::Next).as_deref(),
        Some("large")
    );
    assert_eq!(
        step_active(&ids, Some("large"), Step::Next).as_deref(),
        Some("large")
    );
    assert_eq!(
        step_active(&ids, None, Step::Previous).as_deref(),
        Some("large")
    );
    assert_eq!(
        step_active(&ids, Some("fast"), Step::Last).as_deref(),
        Some("large")
    );
    assert_eq!(step_active(&[], None, Step::First), None);
}

fn picker_app() -> Element {
    rsx! { ModelPickerScene { groups: picker_groups(), default_id: "deep" } }
}

#[test]
fn picker_marks_the_selection_and_the_unavailable_row() {
    let html = render(picker_app);
    assert!(html.contains("role=\"listbox\""));
    assert!(html.contains("aria-selected=\"true\""));
    assert!(html.contains("unavailable"));
    assert!(html.contains("aria-disabled=\"true\""));
    assert!(html.contains("transform: translateY(68px);"));
    assert!(!html.contains("aria-expanded"));
}

use rv2_kit::illustrations::continent_highlight_map::{
    BandStyle, ContinentHighlightMap, LandPiece, MapRegion, RegionStep, polygons_path,
    region_for_land, region_land, step_region_index,
};

fn square(x: f64, y: f64) -> Vec<(f64, f64)> {
    vec![(x, y), (x + 10.0, y), (x + 10.0, y + 10.0), (x, y + 10.0)]
}

fn map_regions() -> Vec<MapRegion> {
    vec![
        MapRegion {
            id: "europe".to_string(),
            label: "Europe".to_string(),
            geometry_ref: "europe".to_string(),
        },
        MapRegion {
            id: "americas".to_string(),
            label: "Americas".to_string(),
            geometry_ref: "americas".to_string(),
        },
    ]
}

fn map_land() -> Vec<LandPiece> {
    vec![
        LandPiece {
            geometry_ref: "americas".to_string(),
            polygons: vec![square(100.0, 100.0), square(200.0, 100.0)],
        },
        LandPiece {
            geometry_ref: "europe".to_string(),
            polygons: vec![square(500.0, 80.0)],
        },
    ]
}

#[test]
fn region_names_its_land_piece_and_paths_close() {
    let regions = map_regions();
    let land = map_land();
    assert_eq!(
        region_land(&regions, &land, "americas").map(|p| p.polygons.len()),
        Some(2)
    );
    assert!(region_land(&regions, &land, "missing").is_none());
    assert_eq!(
        region_for_land(&regions, "europe").map(|r| r.id.as_str()),
        Some("europe")
    );
    let d = polygons_path(&[square(0.0, 0.0)]);
    assert_eq!(d, "M0.0 0.0 L10.0 0.0 L10.0 10.0 L0.0 10.0 Z");
}

#[test]
fn region_switch_steps_clamp_at_both_ends() {
    assert_eq!(step_region_index(6, 0, RegionStep::Previous), 0);
    assert_eq!(step_region_index(6, 5, RegionStep::Next), 5);
    assert_eq!(step_region_index(6, 2, RegionStep::Last), 5);
    assert_eq!(step_region_index(6, 4, RegionStep::First), 0);
}

fn map_app() -> Element {
    rsx! {
        ContinentHighlightMap {
            regions: map_regions(),
            land: map_land(),
            default_region_id: "europe",
            band_style: BandStyle::Coastal,
        }
    }
}

#[test]
fn map_is_named_by_its_selected_region_and_marks_one_highlight() {
    let html = render(map_app);
    assert!(html.contains("aria-label=\"World map, Europe highlighted\""));
    assert!(html.contains("aria-haspopup=\"listbox\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert_eq!(html.matches("data-selected=\"true\"").count(), 2);
    assert!(html.contains("Highlighted: Europe"));
    assert!(!html.contains("role=\"listbox\""));
}

use rv2_kit::illustrations::integration_arc_flow::{
    Hub, Integration, IntegrationArcFlow, IntegrationStatus, slot_angle, slot_for, slot_point,
};

#[test]
fn arc_slots_run_from_left_to_right_and_wrap_with_turns() {
    assert_eq!(slot_angle(0, 1), 90.0);
    assert_eq!(slot_angle(0, 3), 165.0);
    assert_eq!(slot_angle(2, 3), 15.0);
    let (x, y) = slot_point(1, 3);
    assert!((x - 50.0).abs() < 1e-9);
    assert!((y - 24.0).abs() < 1e-9);
    assert_eq!(slot_for(0, 3, 0), 0);
    assert_eq!(slot_for(0, 3, 4), 1);
    assert_eq!(slot_for(2, 3, 1), 0);
}

fn integration(id: &str, label: &str, status: IntegrationStatus) -> Integration {
    Integration {
        id: id.to_string(),
        label: label.to_string(),
        sub_label: "Synced 2 min ago".to_string(),
        status,
        logo: None,
    }
}

fn arc_app() -> Element {
    rsx! {
        IntegrationArcFlow {
            hub: Hub {
                title: "Revenue analysis".to_string(),
                subtitle: "One view across tools".to_string(),
            },
            integrations: vec![
                integration("crm", "CRM", IntegrationStatus::Live),
                integration("billing", "Billing", IntegrationStatus::Paused),
                integration("support", "Support", IntegrationStatus::Live),
            ],
        }
    }
}

#[test]
fn arc_is_a_labelled_group_with_one_connector_per_integration() {
    let html = render(arc_app);
    assert!(html.contains("aria-label=\"Integrations feeding one analysis\""));
    assert_eq!(html.matches("class=\"kit-arc__connector\"").count(), 3);
    assert!(html.contains("data-status=\"paused\""));
    assert!(!html.contains("<button"));
    assert!(html.contains("Revenue analysis"));
}

fn arc_interactive_app() -> Element {
    rsx! {
        IntegrationArcFlow {
            hub: Hub {
                title: "Revenue analysis".to_string(),
                subtitle: "One view across tools".to_string(),
            },
            integrations: vec![integration("crm", "CRM", IntegrationStatus::Live)],
            on_toggle: |_| {},
        }
    }
}

#[test]
fn interactive_arc_pills_are_named_buttons() {
    let html = render(arc_interactive_app);
    assert!(html.contains("aria-label=\"CRM integration, live\""));
    assert!(html.contains("<button"));
}

use rv2_kit::illustrations::keyboard_prompt_input::{
    KeyboardPromptInput, TypingMode, char_delay_ms, key_center, next_prompt, should_submit,
};

#[test]
fn keys_have_centres_and_undrawn_characters_have_none() {
    let (x, y) = key_center('q').expect("q is drawn");
    let (ax, _) = key_center('a').expect("a is drawn");
    assert!(x < 20.0 && y < 40.0);
    assert!(ax > x, "the home row is staggered right of the top row");
    assert_eq!(key_center('#'), None);
    assert_eq!(key_center(' ').map(|(x, _)| x), Some(50.0));
    assert_eq!(key_center('Q'), key_center('q'));
}

#[test]
fn typing_pace_and_submit_rules() {
    assert_eq!(char_delay_ms(TypingMode::Steady, 3, 'a'), 70);
    assert_eq!(char_delay_ms(TypingMode::Bursty, 4, 'a'), 190);
    assert_eq!(next_prompt(1, 3), 2);
    assert_eq!(next_prompt(2, 3), 0);
    assert!(should_submit(true, false, "hello"));
    assert!(
        !should_submit(true, true, "hello"),
        "Shift + Enter is a line break"
    );
    assert!(!should_submit(true, false, "   "));
    assert!(!should_submit(false, false, "hello"));
}

fn prompt_decorative_app() -> Element {
    rsx! { KeyboardPromptInput { sample_prompts: vec!["Draft a reply".to_string()] } }
}

#[test]
fn decorative_prompt_is_hidden_from_assistive_tech_and_draws_keys() {
    let html = render(prompt_decorative_app);
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(!html.contains("<textarea"));
    assert_eq!(html.matches("data-key=").count(), 37);
    assert!(html.contains("data-key=\"q\""));
}

use rv2_kit::illustrations::inspected_agent_card_scene::{
    AgentCard, CardFooter, CardRow, HEADER_PX, InspectSpeed, InspectStep, InspectedAgentCardScene,
    ROW_PX as CARD_ROW_PX, is_playing, row_centre, step_by, tick_ms,
};

#[test]
fn guide_centres_sit_under_the_header_at_the_row_pitch() {
    assert_eq!(row_centre(0), HEADER_PX + CARD_ROW_PX / 2);
    assert_eq!(row_centre(2) - row_centre(1), CARD_ROW_PX);
    assert_eq!(tick_ms(InspectSpeed::Slow), 2600);
    assert_eq!(step_by(0, 3, -1), 0);
    assert_eq!(step_by(1, 3, 5), 2);
}

#[test]
fn guide_holds_while_hovered_focused_pinned_hidden_or_reduced() {
    assert!(is_playing(true, false, false, false, true, true));
    assert!(!is_playing(true, true, false, false, true, true));
    assert!(!is_playing(true, false, true, false, true, true));
    assert!(!is_playing(true, false, false, true, true, true));
    assert!(!is_playing(true, false, false, false, false, true));
    assert!(!is_playing(true, false, false, false, true, false));
    assert!(!is_playing(false, false, false, false, true, true));
}

fn agent_card() -> AgentCard {
    AgentCard {
        title: "Invoice triage agent".to_string(),
        model_label: "Model: Sonnet".to_string(),
        status_label: "Running".to_string(),
        task_line: "Sorting 42 invoices".to_string(),
        rows: vec![
            CardRow {
                label: "Tokens".to_string(),
                value: "12.4k".to_string(),
                bar_fraction: Some(0.4),
            },
            CardRow {
                label: "Tools".to_string(),
                value: "3".to_string(),
                bar_fraction: None,
            },
            CardRow {
                label: "Cost".to_string(),
                value: "$0.08".to_string(),
                bar_fraction: None,
            },
        ],
        footer: CardFooter {
            cost_label: "$0.08 spent".to_string(),
            time_label: "1m 12s".to_string(),
        },
    }
}

fn inspect_card_app() -> Element {
    rsx! {
        InspectedAgentCardScene {
            card: agent_card(),
            inspect_steps: vec![
                InspectStep { row_index: 0, note: "Token use".to_string() },
                InspectStep { row_index: 2, note: "Spend".to_string() },
            ],
        }
    }
}

#[test]
fn inspected_card_is_decorative_with_every_row_and_one_marker_each() {
    let html = render(inspect_card_app);
    assert!(html.contains("aria-hidden=\"true\""));
    assert_eq!(html.matches("class=\"kit-inspect__row\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-inspect__marker\"").count(), 3);
    assert!(html.contains("Token use"));
    assert!(html.contains("kit-inspect__bar-fill"));
    assert!(!html.contains("tabindex"));
}

use rv2_kit::illustrations::inspected_composer_scene::{
    COMPOSER_ROW_PX, Composer, InspectedComposerScene, composer_row_centre,
};

#[test]
fn composer_guide_centres_follow_its_own_row_pitch() {
    assert_eq!(composer_row_centre(0), COMPOSER_ROW_PX / 2);
    assert_eq!(
        composer_row_centre(2) - composer_row_centre(1),
        COMPOSER_ROW_PX
    );
}

fn composer_app() -> Element {
    rsx! {
        InspectedComposerScene {
            composer: Composer {
                placeholder_text: "Ask about your pipeline".to_string(),
                mode_label: "Chat".to_string(),
                tools: vec!["Search".to_string(), "Files".to_string()],
                send_label: "Send".to_string(),
            },
            interactive: true,
        }
    }
}

#[test]
fn interactive_composer_is_focusable_and_shows_its_three_rows() {
    let html = render(composer_app);
    assert!(html.contains("tabindex=\"0\""));
    assert_eq!(html.matches("class=\"kit-composer__marker\"").count(), 3);
    assert!(html.contains("Ask about your pipeline"));
    assert!(html.contains("Search"));
    assert!(!html.contains("aria-hidden=\"true\" tabindex"));
}

fn prompt_interactive_app() -> Element {
    rsx! { KeyboardPromptInput { on_submit: |_| {}, value: "Summarise" } }
}

#[test]
fn interactive_prompt_is_a_labelled_textarea_with_its_text() {
    let html = render(prompt_interactive_app);
    assert!(html.contains("<textarea"));
    assert!(html.contains("aria-label=\"Prompt\""));
    assert!(html.contains("Summarise"));
    assert!(html.contains("data-mode=\"interactive\""));
}

fn picker_popover_app() -> Element {
    rsx! { ModelPickerScene { groups: picker_groups(), popover: true } }
}

#[test]
fn popover_picker_starts_closed_behind_its_trigger() {
    let html = render(picker_popover_app);
    assert!(html.contains("aria-haspopup=\"listbox\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(!html.contains("role=\"listbox\""));
}

#[test]
fn keyboard_draws_function_modifier_and_space_rows() {
    let html = render(prompt_decorative_app);
    assert_eq!(html.matches("kit-kbp__key--fn").count(), 13);
    assert_eq!(html.matches("kit-kbp__key--mod").count(), 4);
    assert!(html.contains("kit-kbp__key--space"));
}

#[test]
fn keyboard_stylesheet_sets_no_font_size_below_the_eleven_pixel_floor() {
    let css = include_str!("../assets/keyboard-prompt-input.css");
    for part in css.split("font-size:").skip(1) {
        let value: String = part
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(px) = value.parse::<f64>() {
            assert!(px >= 11.0, "font-size {px}px in keyboard stylesheet");
        }
    }
}

#[test]
fn toggle_orb_names_its_state_and_pressed_flag() {
    let html = render(orb_interactive_app);
    assert!(html.contains("aria-pressed=\"false\""));
    assert!(html.contains("value=\"Idle\""));
}

#[test]
fn sphere_dots_reach_both_poles() {
    let dots = dot_sphere(DOT_COUNT);
    assert!(dots.iter().any(|d| d.y < -0.8));
    assert!(dots.iter().any(|d| d.y > 0.8));
}
