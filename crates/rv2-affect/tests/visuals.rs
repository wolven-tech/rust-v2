use dioxus::prelude::*;
use rv2_affect::blocks::logo_marquee::{
    LogoMarquee, MarqueeDirection, MarqueeLogo, initial, row_plan,
};
use rv2_affect::blocks::testimonial_wall::{
    Testimonial, TestimonialWall, deal, fit_columns, initials,
};
use rv2_affect::illustrations::agent_team::{
    AgentTeam, AgentTeamMember, AgentTool, Phase, Step, cycle_len, schedule,
};
use rv2_affect::illustrations::cover_flow::{CoverFlow, CoverFlowItem};
use rv2_affect::illustrations::empty_calendar::EmptyCalendar;
use rv2_affect::illustrations::enrichment_orbit::EnrichmentOrbit;
use rv2_affect::illustrations::frost_cloud::{CloudColor, FrostCloud};
use rv2_affect::illustrations::ghosted_chat::GhostedChat;
use rv2_affect::illustrations::glow_cloud::GlowCloud;
use rv2_affect::illustrations::integration_beam::{BeamCenter, BeamTool, IntegrationBeam};
use rv2_affect::illustrations::integration_grid::{IntegrationGrid, fill_for, slots, width_for};
use rv2_affect::illustrations::layer_stack::{LayerStack, layer_y};
use rv2_affect::illustrations::lead_sourcing::LeadSourcing;
use rv2_affect::illustrations::logo_cluster::LogoCluster;
use rv2_affect::illustrations::model_wheel::{ModelWheel, WheelModel, fill_drum};
use rv2_affect::illustrations::region_dach::{Region, RegionDach};
use rv2_affect::illustrations::side_folder::SideFolder;
use rv2_affect::illustrations::tab_overload::TabOverload;
use rv2_affect::illustrations::team_invite::{TeamInvite, TeamInviteAvatar};
use rv2_affect::motion::grid_background::{GridBackground, GridDirection, grid_style};
use rv2_affect::motion::number_ticker::{
    NumberTicker, Part, column_offset_percent, format_value, roll_target, split,
};
use rv2_affect::motion::shimmer_loader::{Length, ShimmerText, Skeleton, skeleton_style};
use rv2_affect::motion::text_reveal::{Chunk, TextBy, TextReveal, chunks, timing_style};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn number_formats_group_thousands_and_keep_decimals() {
    assert_eq!(format_value(1234.5, 1, true), "1,234.5");
    assert_eq!(format_value(1234567.0, 0, true), "1,234,567");
    assert_eq!(format_value(1234.0, 0, false), "1234");
    assert_eq!(format_value(-48240.0, 0, true), "-48,240");
}

#[test]
fn non_finite_number_shows_a_dash_instead_of_the_raw_word() {
    assert_eq!(format_value(f64::NAN, 0, true), "–");
    assert_eq!(format_value(f64::INFINITY, 0, true), "–");
}

#[test]
fn digit_parts_are_keyed_from_the_right_so_the_ones_column_keeps_its_key() {
    let parts = split("$48");
    assert_eq!(
        parts[0],
        Part::Static {
            key: "s0".into(),
            ch: '$'
        }
    );
    assert_eq!(
        parts[1],
        Part::Digit {
            key: "d1".into(),
            digit: 4
        }
    );
    assert_eq!(
        parts[2],
        Part::Digit {
            key: "d0".into(),
            digit: 8
        }
    );
}

#[test]
fn ticker_rolls_forward_and_back_through_the_wrap() {
    assert_eq!(roll_target(9, 0, true), 10);
    assert_eq!(roll_target(0, 9, false), -1);
    assert_eq!(roll_target(3, 5, true), 5);
    assert_eq!(column_offset_percent(0.0), 0.0);
}

#[test]
fn fixed_pixel_skeleton_does_not_shrink_in_a_row() {
    assert_eq!(
        skeleton_style(Some(&Length::Px(120.0)), None, None, false, ""),
        "flex-shrink: 0; width: 120px"
    );
}

#[test]
fn circle_skeleton_takes_its_width_as_height_and_is_round() {
    let style = skeleton_style(Some(&Length::Px(40.0)), None, None, true, "");
    assert!(style.contains("height: 40px"));
    assert!(style.contains("border-radius: 50%"));
}

#[test]
fn word_reveal_numbers_pieces_in_reading_order_and_keeps_spaces() {
    let parts = chunks("Hello big world", TextBy::Word);
    let indices: Vec<usize> = parts
        .iter()
        .filter_map(|c| match c {
            Chunk::Piece { index, .. } => Some(*index),
            _ => None,
        })
        .collect();
    assert_eq!(indices, vec![0, 1, 2]);
    assert!(matches!(parts[1], Chunk::Space(_)));
}

#[test]
fn letter_reveal_groups_each_word_and_continues_the_count_across_words() {
    let parts = chunks("ab cd", TextBy::Char);
    assert_eq!(
        parts[0],
        Chunk::Group(vec![("a".into(), 0), ("b".into(), 1)])
    );
    assert_eq!(
        parts[2],
        Chunk::Group(vec![("c".into(), 2), ("d".into(), 3)])
    );
    assert_eq!(
        timing_style(40, 0, 700),
        "--ui-trv-stagger: 40ms; --ui-trv-delay: 0ms; --ui-trv-duration: 700ms;"
    );
}

#[test]
fn grid_drifts_one_cell_down_left_when_diagonal() {
    assert_eq!(GridDirection::Diagonal.offset(), (-1, 1));
    let style = grid_style(48, GridDirection::Up, 3.0, 70.0, None);
    assert!(style.contains("--ui-gb-dy: -48px;"));
    assert!(style.contains("--ui-gb-dx: 0px;"));
}

#[test]
fn negative_grid_speed_is_clamped_to_still() {
    assert!(grid_style(48, GridDirection::Down, -2.0, 70.0, None).contains("--ui-gb-speed: 0s;"));
}

#[test]
fn two_marquee_rows_split_logos_alternately_and_run_opposite_ways() {
    let plan = row_plan(5, 2, MarqueeDirection::Left);
    assert_eq!(plan[0], (vec![0, 2, 4], 1));
    assert_eq!(plan[1], (vec![1, 3], -1));
}

#[test]
fn logo_letter_tile_uses_the_first_grapheme_uppercased() {
    assert_eq!(initial("ølafur"), "Ø");
    assert_eq!(initial("   "), "?");
}

#[test]
fn testimonial_initials_skip_name_suffixes() {
    assert_eq!(initials("Christopher Montgomery III"), "CM");
    assert_eq!(initials("Ada"), "A");
    assert_eq!(initials("🦊 Fox Ross"), "FR");
}

#[test]
fn wall_fits_columns_to_its_width_and_the_maximum() {
    assert_eq!(fit_columns(800.0, 3), 3);
    assert_eq!(fit_columns(600.0, 3), 2);
    assert_eq!(fit_columns(360.0, 3), 1);
    assert_eq!(fit_columns(800.0, 2), 2);
}

#[test]
fn wall_deals_quotes_round_robin_across_columns() {
    assert_eq!(deal(5, 2), vec![vec![0, 2, 4], vec![1, 3]]);
}

#[test]
fn integration_grid_is_seven_nine_seven_with_one_centre_tile() {
    let all = slots(9);
    assert_eq!(all.len(), 23);
    assert_eq!(all.iter().filter(|s| s.center).count(), 1);
    assert_eq!(width_for(9), 1520.0);
    assert!((fill_for(0.0) - 0.64).abs() < 1e-9);
    assert!((fill_for(100.0) - 0.22).abs() < 1e-9);
}

#[test]
fn layer_stack_opens_a_gap_below_and_above_the_chosen_layer() {
    assert_eq!(layer_y(0, 0, 4), 109.0 - 34.0);
    assert_eq!(layer_y(1, 0, 4), 109.0 + 26.0);
    assert_eq!(layer_y(0, 2, 4), 109.0 - 58.0);
    assert_eq!(layer_y(4, 4, 4), 109.0 + 4.0 * 26.0);
}

#[test]
fn agent_team_timeline_types_then_sends_then_builds_the_team() {
    let steps = schedule(3);
    assert_eq!(steps[0], (522, Step::Typed(1)));
    assert!(steps.iter().any(|(_, s)| *s == Step::Phase(Phase::Sent)));
    let team_at = steps
        .iter()
        .find(|(_, s)| *s == Step::Phase(Phase::Team))
        .map(|(t, _)| *t);
    let reset_at = steps
        .iter()
        .find(|(_, s)| *s == Step::Phase(Phase::Reset))
        .map(|(t, _)| *t);
    assert!(team_at.unwrap() < reset_at.unwrap());
    assert!(cycle_len(3) > reset_at.unwrap());
}

#[test]
fn model_drum_is_filled_to_seven_rows_by_repeating_the_models() {
    let two = vec![
        WheelModel {
            name: "A".into(),
            view_box: "0 0 24 24".into(),
            color: "currentColor".into(),
            d: "M0 0".into(),
        },
        WheelModel {
            name: "B".into(),
            view_box: "0 0 24 24".into(),
            color: "currentColor".into(),
            d: "M0 0".into(),
        },
    ];
    let drum = fill_drum(&two);
    assert!(drum.len() >= 7);
    assert_eq!(drum[2].name, "A");
}

#[test]
fn beam_tools_and_cover_items_keep_their_names() {
    let item = CoverFlowItem::titled("1", "Night Drive");
    assert_eq!(item.title.as_deref(), Some("Night Drive"));
    let tool = BeamTool {
        name: "Slack".into(),
        logo: rsx! { svg {} },
    };
    assert_eq!(tool.name, "Slack");
}

#[test]
fn grid_background_renders_reveal_and_static_state() {
    let html = render(|| rsx! { GridBackground { speed: 0.0, span { "content" } } });
    assert!(html.contains("class=\"ui-gb\""));
    assert!(html.contains("data-reveal"));
    assert!(html.contains("data-static"));
    assert!(html.contains("ui-gb-content"));
}

#[test]
fn number_ticker_renders_a_polite_live_region_with_one_column_per_digit() {
    let html = render(|| rsx! { NumberTicker { value: 4824.0, prefix: "$", grouping: true } });
    assert!(html.contains("aria-live=\"polite\""));
    assert!(html.contains("ui-nt__sr\">$4,824"));
    assert_eq!(html.matches("ui-nt__col").count(), 4);
}

#[test]
fn skeleton_renders_hidden_from_assistive_tech_with_its_size() {
    let html =
        render(|| rsx! { Skeleton { width: Length::Px(120.0), height: Length::from("2em") } });
    assert!(html.contains("class=\"ui-shm\""));
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(html.contains("width: 120px"));
    assert!(html.contains("height: 2em"));
}

#[test]
fn shimmer_text_sets_its_duration() {
    let html = render(|| rsx! { ShimmerText { duration: 3.0, "Thinking" } });
    assert!(html.contains("--ui-shm-text-dur: 3s"));
    assert!(html.contains("Thinking"));
}

#[test]
fn text_reveal_keeps_the_whole_sentence_readable_by_screen_readers() {
    let html = render(
        || rsx! { TextReveal { text: "Ship it", as_tag: rv2_affect::motion::text_reveal::TextTag::H2 } },
    );
    assert!(html.contains("<h2"));
    assert!(html.contains("ui-trv__sr\">Ship it"));
    assert_eq!(html.matches("class=\"ui-trv__p\"").count(), 2);
}

#[test]
fn logo_marquee_renders_a_labelled_region_with_its_logos() {
    let html = render(|| {
        rsx! { LogoMarquee { logos: vec![MarqueeLogo::named("Acme"), MarqueeLogo::named("Globex")], label: "Customers" } }
    });
    assert!(html.contains("role=\"region\""));
    assert!(html.contains("aria-label=\"Customers\""));
    assert!(html.contains("ui-lm__row"));
    assert!(html.contains("ui-lm__sr"));
}

#[test]
fn testimonial_wall_renders_quotes_with_stars_and_verified_mark() {
    let html = render(|| {
        let mut quote = Testimonial::new("Shipped in a week.", "Ada Lovelace");
        quote.rating = Some(5);
        quote.verified = true;
        rsx! { TestimonialWall { items: vec![quote], columns: 1 } }
    });
    assert!(html.contains("ui-tw__quote"));
    assert!(html.contains("aria-label=\"Rated 5 out of 5\""));
    assert!(html.contains("aria-label=\"Verified customer\""));
    assert!(html.contains(">AL<"));
}

#[test]
fn empty_calendar_is_a_labelled_image_with_the_empty_badge() {
    let html = render(|| rsx! { EmptyCalendar {} });
    assert!(html.contains("role=\"img\""));
    assert!(html.contains("ui-ec-back-left"));
    assert!(html.contains("EMPTY"));
    assert!(html.contains("Join meeting"));
}

#[test]
fn enrichment_orbit_draws_three_orbiting_nodes() {
    let html = render(|| rsx! { EnrichmentOrbit {} });
    assert_eq!(html.matches("ui-eo-orbit").count(), 3);
}

#[test]
fn tab_overload_marks_the_front_window_and_its_dots() {
    let html = render(|| rsx! { TabOverload {} });
    assert!(html.contains("ui-to-front-3"));
    assert!(html.contains("ui-to-tab-1"));
}

#[test]
fn side_folder_renders_three_documents_behind_the_pocket() {
    let html = render(|| rsx! { SideFolder {} });
    assert_eq!(html.matches("ui-sf-doc--").count(), 3);
    assert!(html.contains("ui-sf-front"));
}

#[test]
fn team_invite_names_the_teammate_being_invited_in_its_label() {
    let html = render(|| {
        let avatar = |src: &str| TeamInviteAvatar {
            src: src.into(),
            name: None,
        };
        rsx! { TeamInvite { avatars: [avatar("a.png"), avatar("b.png"), avatar("c.png")] } }
    });
    assert!(html.contains("Teammate 3 is dragged into the empty slot"));
    assert!(html.contains("ui-ti-line--joined"));
}

#[test]
fn lead_sourcing_links_gmail_and_grok_around_the_chatgpt_tile() {
    let html = render(|| rsx! { LeadSourcing {} });
    assert!(html.contains("A ChatGPT tile connected to Gmail and Grok"));
    assert!(html.contains("ui-ls-tile"));
}

#[test]
fn frost_and_glow_clouds_carry_their_colour_and_follow_flags() {
    let frost = render(|| rsx! { FrostCloud { color: CloudColor::Orange, follow_pointer: false } });
    assert!(frost.contains("data-color=\"orange\""));
    assert!(!frost.contains("data-follow"));
    let glow = render(|| rsx! { GlowCloud {} });
    assert!(glow.contains("data-color=\"orange\""));
    assert!(glow.contains("data-follow=\"true\""));
    assert!(glow.contains("ui-glw__eyes"));
}

#[test]
fn ghosted_chat_shows_the_three_messages_and_the_read_marker() {
    let html = render(|| rsx! { GhostedChat {} });
    assert!(html.contains("Hello?"));
    assert!(html.contains("ui-gc-read"));
    assert!(html.contains("No new messages"));
}

#[test]
fn layer_stack_marks_the_first_entry_selected_and_its_layer_active() {
    let html = render(|| rsx! { LayerStack {} });
    assert!(html.contains("role=\"tablist\""));
    assert!(html.contains("aria-selected=\"true\""));
    assert!(html.contains("data-active"));
    assert!(html.contains("Triggered by events"));
}

#[test]
fn logo_cluster_places_seven_bubbles_around_the_mark() {
    let html = render(|| rsx! { LogoCluster {} });
    assert_eq!(html.matches("ui-lc-bubble").count(), 7);
    assert!(html.contains("ui-lc-mid"));
}

#[test]
fn agent_team_builds_a_team_of_three_with_tool_logos() {
    let html = render(|| {
        let member = |role: &str| AgentTeamMember {
            role: role.into(),
            avatar: "a.png".into(),
            tools: vec![AgentTool::Slack, AgentTool::Gmail],
        };
        rsx! { AgentTeam { members: [member("SDR"), member("AE"), member("CSM")] } }
    });
    assert_eq!(html.matches("ui-atm-agent").count(), 3);
    assert_eq!(html.matches("ui-atm-tool\"").count(), 6);
    assert!(html.contains("data-phase=\"typing\""));
}

#[test]
fn integration_grid_renders_the_centre_mark_and_outer_logos() {
    let html = render(|| rsx! { IntegrationGrid {} });
    assert!(html.contains("ui-ig-tile--center"));
    assert_eq!(html.matches("class=\"ui-ig-tile\"").count(), 22);
    assert!(html.contains("Integrates with Notion, Stripe"));
}

#[test]
fn model_wheel_renders_seven_rows_and_the_hold_time() {
    let html = render(|| rsx! { ModelWheel {} });
    assert_eq!(html.matches("ui-mw-row").count(), 8);
    assert!(html.contains("data-hold=\"1500\""));
    assert!(html.contains("Claude"));
}

#[test]
fn integration_beam_renders_its_tools_and_default_centre() {
    let html = render(
        || rsx! { IntegrationBeam { center: BeamCenter { name: "Acme".into(), logo: None } } },
    );
    assert_eq!(html.matches("ui-ibm__tool\"").count(), 6);
    assert!(html.contains("Acme connected to Slack"));
    assert!(html.contains("data-left=\"3\""));
}

#[test]
fn cover_flow_renders_a_carousel_with_one_slide_per_item() {
    let html = render(
        || rsx! { CoverFlow { items: vec![CoverFlowItem::titled("a", "One"), CoverFlowItem::titled("b", "Two")] } },
    );
    assert!(html.contains("aria-roledescription=\"carousel\""));
    assert_eq!(html.matches("ui-cf__item\"").count(), 2);
    assert!(html.contains("ui-cf__title\">One"));
}

#[test]
fn region_map_renders_the_dach_outline_and_switcher_by_default() {
    let html = render(|| rsx! { RegionDach {} });
    assert!(html.contains("Map of the DACH region"));
    assert!(html.contains("ui-rd-trigger"));
    assert!(html.contains("ui-rd-band--2"));
}

#[test]
fn region_map_hides_the_switcher_when_asked() {
    let html = render(|| rsx! { RegionDach { region: Region::Jp, show_switcher: false } });
    assert!(html.contains("Map of Japan"));
    assert!(!html.contains("ui-rd-trigger"));
}
