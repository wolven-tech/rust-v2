use dioxus::prelude::*;
use rv2_kit::illustrations::frosted_folder_mark::{FrostedFolderMark, SizeStep, folder_scale};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn folder_size_steps_scale_about_the_stage_proportion() {
    assert_eq!(folder_scale(SizeStep::Sm), 0.75);
    assert_eq!(folder_scale(SizeStep::Md), 1.0);
    assert_eq!(folder_scale(SizeStep::Lg), 1.25);
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
    assert!(html.contains("--kit-folder-scale: 1.25;"));
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

#[test]
fn folder_at_rest_has_one_back_one_tab_three_sheets_and_one_pocket() {
    let html = render(folder_decorative_app);
    assert_eq!(html.matches("class=\"kit-folder__back\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-folder__tab\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-folder__sheet\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-folder__pocket\"").count(), 1);
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

fn pill_at_rest_app() -> Element {
    rsx! {
        ProgressPillScene {
            title: "Q3 revenue report",
            agent_label: "Analyst",
            source_line: "24 sources read",
            progress: 0.33,
        }
    }
}

#[test]
fn one_pill_at_rest_has_a_nine_dot_mark_and_no_step_line() {
    let html = render(pill_at_rest_app);
    assert_eq!(html.matches("class=\"kit-pill\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-pill__cell\"").count(), 9);
    assert!(!html.contains("role=\"status\""));
    assert!(html.contains("33%"));
    assert!(html.contains("Analyst"));
    assert!(html.contains("24 sources read"));
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
    BAR_COUNT, DictationCapturePanel, DictationState, bar_heights, format_elapsed, loudness, taper,
};

#[test]
fn elapsed_time_reads_as_m_ss() {
    assert_eq!(format_elapsed(0), "0:00");
    assert_eq!(format_elapsed(42), "0:42");
    assert_eq!(format_elapsed(125), "2:05");
}

#[test]
fn waveform_bars_shrink_toward_the_left_edge() {
    assert!((taper(0, BAR_COUNT) - 0.35).abs() < 1e-9);
    assert!((taper(BAR_COUNT - 1, BAR_COUNT) - 1.0).abs() < 1e-9);
    assert!((0..BAR_COUNT - 1).all(|i| taper(i, BAR_COUNT) < taper(i + 1, BAR_COUNT)));
}

fn dictation_settled_app() -> Element {
    rsx! {
        DictationCapturePanel {
            state: DictationState::Listening,
            level_samples: vec![0.3; 40],
            elapsed_secs: 42,
            finish_label: "Done",
            on_finish: |_| {},
            on_cancel: |_| {},
        }
    }
}

#[test]
fn status_row_holds_time_and_both_actions_with_no_visible_state_label() {
    let html = render(dictation_settled_app);
    assert!(html.contains("0:42"));
    assert!(html.contains(">Done<"));
    assert!(html.contains(">Cancel<"));
    assert!(html.contains("Escape"));
    assert!(html.contains("kit-dict__sr"));
    assert!(!html.contains("kit-dict__state"));
    assert!(html.contains("kit-dict__light"));
    assert_eq!(html.matches("kit-dict__light\"").count(), 2);
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
    assert!(html.contains("1:05"));
    assert!(html.contains("Listening"));
    assert_eq!(html.matches("kit-dict__bar").count(), BAR_COUNT);
}

fn dictation_idle_app() -> Element {
    rsx! { DictationCapturePanel { elapsed_secs: 90 } }
}

#[test]
fn idle_panel_shows_zero_time_and_the_start_prompt() {
    let html = render(dictation_idle_app);
    assert!(html.contains("0:00"));
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
    OrbState, VoiceAssistantOrb, clamp_level, dot_opacity, lattice_dots, ring_points, state_label,
};

#[test]
fn sphere_is_drawn_on_the_front_hemisphere_only_and_every_dot_lies_on_it() {
    let dots = lattice_dots();
    assert!(dots.iter().all(|d| d.z >= 0.0));
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
    assert_eq!(html.matches("kit-orb__dot\"").count(), lattice_dots().len());
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
    BRANCH_POINT, DESTINATION_CENTRE_X, DESTINATION_SIDE, Endpoint, OutreachFanoutLines,
    branch_path, pulse_begin, shown_destinations,
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
    assert!(branch_path(1).starts_with("M 50 "));
    assert!(
        branch_path(1).contains(" L 50 "),
        "the centre curve is straight down to its tile"
    );
    assert!(branch_path(0).contains(" C "));
}

#[test]
fn fanout_junction_sits_in_the_upper_third_and_the_tile_row_spans_two_fifths() {
    let (_, junction_y) = BRANCH_POINT;
    assert!(
        junction_y <= 33.4,
        "junction at {junction_y} is below the upper third"
    );
    let left = DESTINATION_CENTRE_X[0] - DESTINATION_SIDE[0] / 2.0;
    let right = DESTINATION_CENTRE_X[2] + DESTINATION_SIDE[2] / 2.0;
    let span = right - left;
    assert!(
        (span - 40.0).abs() <= 3.0,
        "tile row spans {span} of the stage"
    );
}

fn fanout_app() -> Element {
    rsx! {
        OutreachFanoutLines {
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

#[test]
fn fanout_has_a_bare_junction_no_trunk_and_one_centre_light() {
    let html = render(fanout_app);
    assert_eq!(html.matches("class=\"kit-fan__junction\"").count(), 1);
    assert!(!html.contains("kit-fan__trunk"));
    assert!(!html.contains("kit-fan__tile--source"));
    assert_eq!(html.matches("class=\"kit-fan__light\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-fan__tile-label\"").count(), 0);
}

use rv2_kit::illustrations::task_branch_scene::{
    Assignee, ClientNode, NestedCard, Person, Role, SideNode, SubtaskCard, TaskBranchScene,
    TaskCard, initials,
};

#[test]
fn initials_take_the_first_letter_of_up_to_two_words() {
    assert_eq!(initials("Ada Lovelace"), "AL");
    assert_eq!(initials("grace hopper weaver"), "GH");
    assert_eq!(initials(""), "");
}

fn owner(id: &str, avatar: &str) -> Assignee {
    Assignee {
        id: id.to_string(),
        avatar: avatar.to_string(),
    }
}

fn branch_app() -> Element {
    rsx! {
        TaskBranchScene {
            client: ClientNode {
                label: "Client: Northwind Foods".to_string(),
                connection_status: "Connected".to_string(),
            },
            task: TaskCard {
                title: "Task: Q4 onboarding deck".to_string(),
                assignees: vec![owner("ada", "AL"), owner("sam", "SR")],
                tool_icons: vec![],
            },
            side: SideNode {
                label: "Client notes".to_string(),
                meta: "3 notes".to_string(),
            },
            subtasks: SubtaskCard {
                title: "Subtasks: draft the outline".to_string(),
                assignees: vec![owner("a", "A1"), owner("b", "B2"), owner("c", "C3")],
                nested: Some(NestedCard {
                    title: "Subtask 1: draft the outline".to_string(),
                    assignees: vec![owner("d", "D4")],
                    tool_icons: vec![],
                }),
            },
            split_label: "Role split".to_string(),
            role: Role {
                id: "pm".to_string(),
                title: "Project manager".to_string(),
                subline: "Owns the timeline".to_string(),
                person: Person { name: "Dana Miller".to_string(), avatar: "DM".to_string() },
                tool_icons: vec![],
            },
            ghost_role: Some(Role {
                id: "design".to_string(),
                title: "Design lead".to_string(),
                subline: "Owns the look".to_string(),
                person: Person { name: "Mia Lee".to_string(), avatar: "ML".to_string() },
                tool_icons: vec![],
            }),
        }
    }
}

#[test]
fn branch_scene_lists_each_owner_and_is_decorative() {
    let html = render(branch_app);
    assert_eq!(html.matches("kit-branch__avatar\" data-id=").count(), 6);
    assert!(html.contains("data-id=\"ada\""));
    assert!(html.contains("data-id=\"sam\""));
    assert!(html.contains("Connected"));
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(html.contains("kit-branch__node--role"));
    assert!(!html.contains("data-hot"));
}

#[test]
fn branch_scene_at_rest_has_seven_cards_one_chip_and_six_connectors() {
    let html = render(branch_app);
    for node in [
        "client", "task", "side", "subtasks", "nested", "role", "ghost",
    ] {
        assert_eq!(
            html.matches(&format!("data-node=\"{node}\"")).count(),
            1,
            "one {node} card"
        );
    }
    assert_eq!(html.matches("kit-branch__chip").count(), 1);
    let connectors: std::collections::BTreeSet<&str> = html
        .split("data-connector=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .collect();
    assert_eq!(connectors.len(), 6, "six connectors: {connectors:?}");
    assert!(html.contains("Role split"));
}

#[test]
fn branch_scene_side_connector_carries_no_text() {
    let html = render(branch_app);
    assert!(html.contains(
        "<span class=\"kit-branch__link kit-branch__link--task-side\" data-connector=\"task-side\" data-touch=\"task side\"></span>"
    ));
}

#[test]
fn branch_scene_only_the_role_split_chip_sits_on_a_connector() {
    let html = render(branch_app);
    assert_eq!(html.matches("kit-branch__chip").count(), 1);
    assert_eq!(html.matches("kit-branch__link--roles").count(), 1);
    assert!(!html.contains("kit-branch__link--split-"));
}

#[test]
fn branch_scene_has_three_background_rings_and_a_side_node_icon() {
    let html = render(branch_app);
    assert_eq!(
        html.matches("kit-branch__ring kit-branch__ring--").count(),
        3
    );
    assert!(html.contains("kit-branch__side-icon"));
}

use rv2_kit::illustrations::model_picker_scene::{
    ModelGroup, ModelOption, ModelPickerScene, Step, available_ids, row_top_pct, step_active,
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
fn highlight_follows_the_row_grid_across_groups() {
    let groups = picker_groups();
    let first = row_top_pct(&groups, "fast").expect("first row");
    let second = row_top_pct(&groups, "deep").expect("second row");
    let fourth = row_top_pct(&groups, "large").expect("fourth row");
    assert!(second > first && fourth > second);
    assert!((second - first - (50.0 / 6.0) / 72.0 * 100.0).abs() < 1e-9);
    assert_eq!(row_top_pct(&groups, "missing"), None);
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
    assert!(html.contains("top: "));
    assert!(html.contains("kit-model__band"));
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
    Hub, Integration, IntegrationArcFlow, IntegrationStatus, slot_for, slot_offset, spine_opacity,
};

#[test]
fn spine_turns_one_slot_at_a_time_and_wraps() {
    assert_eq!(slot_for(0, 5, 0), 0);
    assert_eq!(slot_for(0, 5, 4), 4);
    assert_eq!(slot_for(4, 5, 1), 0);
}

#[test]
fn centre_pill_is_full_strength_and_the_rest_fade_with_distance() {
    assert_eq!(slot_offset(2, 5), 0);
    assert_eq!(slot_offset(0, 5), -2);
    assert_eq!(slot_offset(4, 5), 2);
    assert_eq!(spine_opacity(0), 1.0);
    assert!(spine_opacity(1) > spine_opacity(2));
    assert_eq!(spine_opacity(-1), spine_opacity(1));
}

fn integration(id: &str, label: &str, status: IntegrationStatus) -> Integration {
    Integration {
        id: id.to_string(),
        label: label.to_string(),
        status,
        logo: None,
    }
}

fn spine_app() -> Element {
    rsx! {
        IntegrationArcFlow {
            hub: Hub {
                title: "Analysis hub".to_string(),
                subtitle: "Every source in one view".to_string(),
            },
            integrations: vec![
                integration("helpdesk", "Helpdesk", IntegrationStatus::Live),
                integration("warehouse", "Warehouse", IntegrationStatus::Live),
                integration("chat", "Chat", IntegrationStatus::Live),
                integration("tracker", "Tracker", IntegrationStatus::Live),
                integration("payments", "Payments", IntegrationStatus::Live),
            ],
        }
    }
}

#[test]
fn spine_has_five_pills_one_centre_one_hub_and_one_dashed_guide_and_no_beam_at_rest() {
    let html = render(spine_app);
    assert_eq!(html.matches("class=\"kit-arc__pill\"").count(), 5);
    assert_eq!(html.matches("data-centre=\"true\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-arc__beam\"").count(), 0);
    assert_eq!(html.matches("class=\"kit-arc__hub\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-arc__guide\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-arc__kind\"").count(), 5);
    assert_eq!(html.matches("Connector").count(), 5);
    assert!(html.contains("--s: 0;"));
    assert!(html.contains("--s: -2;"));
    assert!(html.contains("--s: 2;"));
    assert!(html.contains("Analysis hub"));
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
fn arc_is_a_labelled_group_with_one_pill_per_integration() {
    let html = render(arc_app);
    assert!(html.contains("aria-label=\"Integrations feeding one analysis\""));
    assert_eq!(html.matches("class=\"kit-arc__pill\"").count(), 3);
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
    KEY_ROWS, KeyboardPromptInput, ROW_UNITS, TypingMode, char_delay_ms, key_center, next_prompt,
    should_submit, spot_x,
};

#[test]
fn keys_have_centres_and_undrawn_characters_have_none() {
    let (x, y) = key_center('q').expect("q is drawn");
    let (ax, _) = key_center('a').expect("a is drawn");
    assert!(x < 20.0 && y > 35.0 && y < 50.0, "q sits in the tab row");
    assert!(ax > x, "the caps row starts right of the tab row");
    assert_eq!(key_center('#'), None);
    let (space_x, _) = key_center(' ').expect("space is drawn");
    assert!(
        (space_x - 46.7).abs() < 0.1,
        "space is the long key after the bottom modifiers"
    );
    assert_eq!(key_center('Q'), key_center('q'));
}

#[test]
fn every_drawn_row_is_the_same_width_in_key_units() {
    for (row, keys) in KEY_ROWS.iter().enumerate() {
        let units: f64 = keys.iter().map(|key| key.units).sum();
        assert!(
            (units - ROW_UNITS).abs() < 1e-9,
            "row {row} is {units} units wide"
        );
    }
}

#[test]
fn spot_sits_under_the_most_recent_key_after_three_presses() {
    for c in ['a', 's', 'd'] {
        assert_eq!(spot_x(Some(c)), key_center(c).expect("drawn").0);
    }
    assert_eq!(spot_x(None), 50.0);
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
    assert_eq!(html.matches("data-key=").count(), 48);
    assert!(html.contains("data-key=\"q\""));
}

fn prompt_rest_app() -> Element {
    rsx! {
        KeyboardPromptInput {
            sample_prompts: vec!["Summarize last week's support tickets".to_string()],
            paused: true,
        }
    }
}

#[test]
fn paused_prompt_rests_on_half_its_text_with_the_last_key_lit() {
    let html = render(prompt_rest_app);
    assert!(html.contains(">Summarize last wee<"));
    assert_eq!(html.matches("data-lit=\"true\"").count(), 1);
    assert!(
        html.contains("data-key=\"e\" data-lit=\"true\"")
            || html.contains("data-lit=\"true\" data-key=\"e\"")
    );
}

fn prompt_one_pill_app() -> Element {
    rsx! { KeyboardPromptInput { sample_prompts: vec!["Draft a reply".to_string()] } }
}

#[test]
fn scene_has_one_pill_one_keyboard_and_one_floor() {
    let html = render(prompt_one_pill_app);
    assert_eq!(html.matches("class=\"kit-kbp__field\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-kbp__keyboard\"").count(), 1);
    assert_eq!(html.matches("kit-glow__backdrop").count(), 1);
    assert_eq!(html.matches("class=\"kit-kbp__mark\"").count(), 1);
}

#[test]
fn keyboard_draws_six_rows_from_the_function_row_to_the_bottom_row() {
    let html = render(prompt_one_pill_app);
    assert_eq!(html.matches("data-row=").count(), 6);
    assert!(html.contains("data-row=\"0\""));
    assert!(html.contains("data-row=\"5\""));
}

use rv2_kit::illustrations::inspected_agent_card_scene::{
    AgentCard, CardFooter, GhostCard, InspectSpeed, InspectStep, InspectedAgentCardScene,
    ROW_HEIGHTS_PX, avatar_letters, is_playing, row_centre, row_top, step_by, tick_ms,
};

#[test]
fn guide_centres_sit_on_each_row_in_order() {
    assert_eq!(row_top(0), 12);
    assert_eq!(row_top(1), 12 + 84 + 16);
    assert_eq!(row_centre(0), 12 + 84 / 2);
    assert!(row_centre(0) < row_centre(1) && row_centre(1) < row_centre(2));
    assert_eq!(ROW_HEIGHTS_PX.len(), 3);
    assert_eq!(avatar_letters("ledger reviewer"), "LR");
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
        title: "Patch writer".to_string(),
        model_label: "Large model".to_string(),
        status_label: "Running".to_string(),
        task_line: "Fix the broken link in the setup guide".to_string(),
        footer: CardFooter {
            cost_label: "$0.42".to_string(),
            time_label: "9 min".to_string(),
        },
    }
}

fn inspect_card_app() -> Element {
    rsx! {
        InspectedAgentCardScene {
            card: agent_card(),
            ghost: Some(GhostCard {
                title: "Export checker".to_string(),
                model_label: "Compact model".to_string(),
                status_label: "Done".to_string(),
                task_line: "Check the totals in the export file".to_string(),
                footer: CardFooter {
                    cost_label: "$0.18".to_string(),
                    time_label: "3 min".to_string(),
                },
            }),
            inspect_steps: vec![
                InspectStep { row_index: 0, note: "Header".to_string() },
                InspectStep { row_index: 2, note: "Footer".to_string() },
            ],
        }
    }
}

#[test]
fn inspected_card_has_three_rows_one_marker_and_one_tag_each_and_a_faded_card_above() {
    let html = render(inspect_card_app);
    assert!(html.contains("aria-hidden=\"true\""));
    assert_eq!(html.matches("class=\"kit-inspect__row").count(), 3);
    assert_eq!(html.matches("class=\"kit-inspect__marker\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-inspect__tag\"").count(), 3);
    assert!(html.contains("Header") && html.contains("Task") && html.contains("Footer"));
    assert!(html.contains("Export checker"));
    assert!(html.contains("Patch writer"));
    assert!(html.contains("kit-inspect__ghost"));
    assert!(!html.contains("tabindex"));
}

#[test]
fn inspected_card_draws_one_focused_and_one_faded_agent_with_different_statuses() {
    let html = render(inspect_card_app);
    assert_eq!(html.matches("class=\"kit-inspect__ghost\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-inspect__card\"").count(), 1);
    assert!(html.contains("Running"));
    assert!(html.contains("Done"));
    assert!(html.contains("Patch writer") && html.contains("Export checker"));
}

#[test]
fn focused_header_row_carries_frame_tag_width_inset_gap_and_pointer() {
    let html = render(inspect_card_app);
    assert_eq!(html.matches("class=\"kit-inspect__frame").count(), 1);
    assert_eq!(html.matches("class=\"kit-inspect__width\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-inspect__inset\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-inspect__gap\"").count(), 2);
    assert_eq!(html.matches("class=\"kit-inspect__pointer\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-inspect__spin\"").count(), 1);
}

use rv2_kit::illustrations::inspected_composer_scene::{
    COMPOSER_TAGS, Composer, InspectedComposerScene, composer_row_centre, composer_row_top,
};

#[test]
fn composer_guide_centres_follow_its_own_row_heights() {
    assert!(composer_row_centre(0) < composer_row_centre(1));
    assert!(composer_row_centre(1) < composer_row_centre(2));
    assert_eq!(composer_row_top(0), 12);
    assert_eq!(COMPOSER_TAGS, ["Toolbar", "Attach", "Message"]);
}

fn composer_app() -> Element {
    rsx! {
        InspectedComposerScene {
            composer: Composer {
                message: "Draft the release notes for version 3".to_string(),
                mode_label: "Auto".to_string(),
            },
            connected_apps: vec![rsx! { span { "A" } }, rsx! { span { "B" } }, rsx! { span { "C" } }, rsx! { span { "D" } }],
            interactive: true,
        }
    }
}

#[test]
fn interactive_composer_is_focusable_and_shows_its_three_rows_with_three_app_marks() {
    let html = render(composer_app);
    assert!(html.contains("tabindex=\"0\""));
    assert_eq!(html.matches("class=\"kit-composer__marker\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-composer__app\"").count(), 3);
    assert!(html.contains("Draft the release notes for version 3"));
    assert!(html.contains("Work in a folder"));
    assert!(html.contains("Connect apps"));
    assert!(html.contains(">Auto<"));
    assert!(!html.contains("aria-hidden=\"true\" tabindex"));
}

#[test]
fn composer_draws_one_column_with_the_message_row_framed_and_no_second_composer() {
    let html = render(composer_app);
    assert_eq!(html.matches("class=\"kit-composer__column\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-composer__surface\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-composer__frame").count(), 1);
    assert_eq!(html.matches("class=\"kit-composer__width\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-composer__inset\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-composer__gap\"").count(), 3);
    assert!(html.contains("Message"));
}

#[test]
fn composer_app_marks_sit_on_the_attach_row_only() {
    let html = render(composer_app);
    assert_eq!(
        html.matches("class=\"kit-composer__chip kit-composer__chip--apps\"")
            .count(),
        1
    );
    assert_eq!(
        html.matches("class=\"kit-composer__row kit-composer__row--attach")
            .count(),
        2
    );
    assert_eq!(
        html.matches("class=\"kit-composer__row kit-composer__row--faded\"")
            .count(),
        2
    );
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
    assert_eq!(html.matches("kit-kbp__key--mod").count(), 10);
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
fn keyboard_pill_is_narrower_than_the_keyboard_and_the_floor_is_wider_than_it() {
    let css = include_str!("../assets/keyboard-prompt-input.css");
    assert!(css.contains("max-width: 720px;"));
    assert!(css.contains("width: min(100%, 340px);"));
    assert!(css.contains("padding: 0 3% 64px;"));
}

#[test]
fn keyboard_pill_halo_is_an_identity_colour_spread_on_all_four_sides() {
    let css = include_str!("../assets/keyboard-prompt-input.css");
    let field = css
        .split(".kit-kbp__field {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("field rule");
    assert!(field.contains("0 0 0 3px color-mix(in srgb, var(--kit-id-2)"));
    assert!(field.contains("0 0 22px 6px color-mix(in srgb, var(--kit-id-2)"));
}

#[test]
fn toggle_orb_names_its_state_and_pressed_flag() {
    let html = render(orb_interactive_app);
    assert!(html.contains("aria-pressed=\"false\""));
    assert!(html.contains("value=\"Idle\""));
}

#[test]
fn sphere_dots_reach_both_poles() {
    let dots = lattice_dots();
    assert!(dots.iter().any(|d| d.y < -0.8));
    assert!(dots.iter().any(|d| d.y > 0.8));
}
