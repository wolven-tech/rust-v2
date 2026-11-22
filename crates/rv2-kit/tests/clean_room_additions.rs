//! SSR and pure-logic checks for the agent clash map, run process lanes and the
//! interruptible answer stream. Interactions (pointer, focus, Escape, Stop,
//! Approve, Dismiss) are checked in the browser through the kit gallery.

use dioxus::prelude::*;
use rv2_kit::agents::agent_clash_map::{
    AgentClashMap, Clash, ClashWeight, Emphasis, Participant, Side, Target, agent_label,
    count_line, curve_path, emphasis, first_name, normalise, slot_centres, stage_height, sync_line,
};
use rv2_kit::agents::run_process_lanes::{
    Lane, LaneKind, RunProcessLanes, RunStatus, Step, StepShape, StepStatus, connector_paths,
    counted_total, done_count, frame_for, normalise as normalise_run, run_status,
};
use rv2_kit::components::interruptible_answer_stream::{
    Bend, BodyPart, InterruptibleAnswerStream, Steer, SteerKind, StreamMeta, StreamStatus,
    announcement, body_parts, completed_sentences, fold_steers, held_back, meta_line, plain_text,
    steer_tag, visible_text,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn person(id: &str, name: &str) -> Participant {
    Participant {
        id: id.to_string(),
        name: name.to_string(),
        note: None,
    }
}

fn side(participant: &str, agent: &str) -> Side {
    Side {
        participant_id: participant.to_string(),
        agent_name: agent.to_string(),
        harness_key: None,
    }
}

fn clash(id: &str, weight: ClashWeight, label: &str, sides: Vec<Side>) -> Clash {
    Clash {
        id: id.to_string(),
        weight,
        label: label.to_string(),
        summary: format!("{label} summary"),
        sides,
    }
}

fn lane(id: &str, name: &str) -> Lane {
    Lane {
        id: id.to_string(),
        name: name.to_string(),
        kind: LaneKind::Agent,
        role: None,
        mark: None,
        color: None,
        description: None,
    }
}

fn step(id: &str, lane_id: &str, title: &str) -> Step {
    Step {
        id: id.to_string(),
        lane_id: lane_id.to_string(),
        title: title.to_string(),
        status: StepStatus::Queued,
        shape: StepShape::Step,
        after: None,
        duration_label: None,
        detail: None,
    }
}

fn edges_of(lanes: &[Lane], steps: &[Step]) -> Vec<(String, String)> {
    let diagram = normalise_run(lanes, steps);
    diagram
        .edges
        .iter()
        .map(|&(from, to)| {
            (
                diagram.steps[from].step.id.clone(),
                diagram.steps[to].step.id.clone(),
            )
        })
        .collect()
}

#[test]
fn one_curve_per_clash_and_agent_pair_and_a_repeated_agent_counts_once() {
    let participants = vec![person("ana", "Ana Lee"), person("ben", "Ben Cole")];
    let clashes = vec![
        clash(
            "c1",
            ClashWeight::Decide,
            "Shared files",
            vec![
                side("ana", "Coder"),
                side("ana", "Coder"),
                side("ben", "Tester"),
            ],
        ),
        clash(
            "c2",
            ClashWeight::Faint,
            "Repeat",
            vec![side("ana", "Coder")],
        ),
    ];
    let map = normalise(&participants, &clashes);
    assert_eq!(map.edges.len(), 3);
    assert_eq!(map.agents.len(), 2);
    let html = render(|| {
        let participants = vec![person("ana", "Ana Lee"), person("ben", "Ben Cole")];
        let clashes = vec![
            clash(
                "c1",
                ClashWeight::Decide,
                "Shared files",
                vec![
                    side("ana", "Coder"),
                    side("ana", "Coder"),
                    side("ben", "Tester"),
                ],
            ),
            clash(
                "c2",
                ClashWeight::Faint,
                "Repeat",
                vec![side("ana", "Coder")],
            ),
        ];
        rsx! { AgentClashMap { participants, clashes } }
    });
    assert_eq!(html.matches("kit-acm__curve\"").count(), 3);
}

#[test]
fn unknown_side_is_dropped_and_a_clash_left_empty_is_not_rendered() {
    let participants = vec![person("ana", "Ana Lee")];
    let clashes = vec![
        clash(
            "ghost",
            ClashWeight::Notice,
            "Ghost",
            vec![side("nobody", "Bot")],
        ),
        clash(
            "real",
            ClashWeight::Notice,
            "Real",
            vec![side("ana", "Coder"), side("nobody", "Bot")],
        ),
    ];
    let map = normalise(&participants, &clashes);
    assert_eq!(map.clashes.len(), 1);
    assert_eq!(map.clashes[0].id, "real");
    assert_eq!(map.clashes[0].sides.len(), 1);
}

#[test]
fn a_participant_in_no_clash_is_listed_as_in_sync() {
    let html = render(|| {
        let participants = vec![person("ana", "Ana Lee"), person("ben", "Ben Cole")];
        let clashes = vec![clash(
            "c1",
            ClashWeight::Notice,
            "Files",
            vec![side("ana", "Coder")],
        )];
        rsx! { AgentClashMap { participants, clashes } }
    });
    assert!(html.contains("In sync: Ben Cole"));
    assert_eq!(sync_line(&[]), None);
}

#[test]
fn every_weight_is_named_by_its_word_and_the_legend_needs_clashes() {
    let html = render(|| {
        let participants = vec![person("ana", "Ana Lee")];
        let clashes = vec![
            clash(
                "c1",
                ClashWeight::Decide,
                "Files",
                vec![side("ana", "Coder")],
            ),
            clash("c2", ClashWeight::Faint, "Tone", vec![side("ana", "Coder")]),
        ];
        rsx! { AgentClashMap { participants, clashes } }
    });
    assert!(html.contains("Decide: needs a person now"));
    assert!(html.contains("Notice: a heads-up"));
    assert!(html.contains("Faint: background"));
    assert!(html.contains(">Decide<"));
    assert!(html.contains(">Faint<"));
}

#[test]
fn with_no_clashes_the_in_sync_message_shows_and_the_legend_does_not() {
    let html = render(|| {
        let participants = vec![person("ana", "Ana Lee")];
        rsx! { AgentClashMap { participants, clashes: Vec::new() } }
    });
    assert!(html.contains("Everyone is in sync."));
    assert!(!html.contains("Decide: needs a person now"));
    assert!(!html.contains("kit-acm__stage"));
}

#[test]
fn pointing_at_a_clash_lights_only_its_curves() {
    let focus = Target::Clash("c1".to_string());
    assert_eq!(
        emphasis(Some(&focus), "c1", &[("ana", "Coder")]),
        Emphasis::Full
    );
    assert_eq!(
        emphasis(Some(&focus), "c2", &[("ana", "Coder")]),
        Emphasis::Faded
    );
    assert_eq!(emphasis(None, "c2", &[("ana", "Coder")]), Emphasis::Normal);
    let agent = Target::Agent("ana".to_string(), "Coder".to_string());
    assert_eq!(
        emphasis(Some(&agent), "c2", &[("ana", "Coder")]),
        Emphasis::Full
    );
    assert_eq!(
        emphasis(Some(&agent), "c2", &[("ben", "Tester")]),
        Emphasis::Faded
    );
}

#[test]
fn first_name_is_the_first_word_with_a_letter() {
    assert_eq!(first_name("🚀 Ana Lee"), "Ana");
    assert_eq!(first_name("→Ana"), "Ana");
    assert_eq!(first_name("  Mo  "), "Mo");
    assert_eq!(first_name("123"), "123");
}

#[test]
fn chips_keep_their_rows_and_the_card_wording_reads_plainly() {
    let height = stage_height(3, 2, 2, 0.0);
    assert_eq!(height, 3.0 * 64.0 + 24.0);
    let centres = slot_centres(3, height);
    assert!(centres.windows(2).all(|pair| pair[1] - pair[0] >= 64.0));
    assert_eq!(stage_height(1, 0, 1, 400.0), 400.0);
    assert_eq!(count_line(1), "1 clash");
    assert_eq!(
        agent_label("Ana Lee", "Coder", 2),
        "Ana Lee, Coder, 2 clashes"
    );
    assert_eq!(
        curve_path(28.0, 10.0, 50.0, 20.0),
        "M 28 10 C 39 10 39 20 50 20"
    );
}

#[test]
fn chip_is_a_labelled_button_and_the_agent_name_is_in_its_card() {
    let html = render(|| {
        let participants = vec![person("ana", "Ana Lee")];
        let clashes = vec![clash(
            "c1",
            ClashWeight::Notice,
            "Files",
            vec![side("ana", "Coder")],
        )];
        rsx! { AgentClashMap { participants, clashes } }
    });
    assert!(html.contains("aria-label=\"Ana Lee, Coder, 1 clash\""));
    assert!(html.contains("aria-expanded=\"false\""));
}

#[test]
fn clash_map_reduced_motion_block_removes_all_motion() {
    let css = include_str!("../assets/agent-clash-map.css");
    let block = css
        .split("@media (prefers-reduced-motion: reduce)")
        .nth(1)
        .expect("reduced-motion block");
    assert!(block.contains(".kit-acm__curve"));
    assert!(block.contains(".kit-acm__badge"));
    assert!(block.contains("animation: none"));
}

#[test]
fn hand_over_connects_from_the_previous_step_unless_it_starts_a_branch() {
    let lanes = vec![lane("l1", "Writer"), lane("l2", "Reviewer")];
    let mut s3 = step("s3", "l1", "Branch");
    s3.after = Some(Vec::new());
    let mut s4 = step("s4", "l2", "Review");
    s4.after = Some(vec!["s1".to_string()]);
    let steps = vec![step("s1", "l1", "Draft"), step("s2", "l2", "Check"), s3, s4];
    let edges = edges_of(&lanes, &steps);
    assert!(edges.contains(&("s1".to_string(), "s2".to_string())));
    assert!(edges.contains(&("s1".to_string(), "s4".to_string())));
    assert!(!edges.iter().any(|(_, to)| to == "s3"));
    assert_eq!(edges.len(), 2);
}

#[test]
fn steps_on_unknown_lanes_drop_with_their_connectors() {
    let lanes = vec![lane("l1", "Writer")];
    let steps = vec![
        step("s1", "l1", "Draft"),
        step("s2", "ghost", "Lost"),
        step("s3", "l1", "Publish"),
    ];
    let diagram = normalise_run(&lanes, &steps);
    assert_eq!(diagram.steps.len(), 2);
    assert!(
        edges_of(&lanes, &steps)
            .iter()
            .all(|(a, b)| a != "s2" && b != "s2")
    );
}

#[test]
fn connector_from_an_unknown_id_is_dropped() {
    let lanes = vec![lane("l1", "Writer")];
    let mut a = step("a", "l1", "A");
    a.after = Some(vec!["nope".to_string()]);
    assert!(edges_of(&lanes, &[a]).is_empty());
}

#[test]
fn a_connector_that_would_close_a_cycle_is_cut() {
    let lanes = vec![lane("l1", "Writer")];
    let mut a = step("a", "l1", "A");
    a.after = Some(vec!["c".to_string()]);
    let mut b = step("b", "l1", "B");
    b.after = Some(vec!["a".to_string()]);
    let mut c = step("c", "l1", "C");
    c.after = Some(vec!["b".to_string()]);
    let edges = edges_of(&lanes, &[a, b, c]);
    assert_eq!(edges.len(), 2);
    assert!(edges.contains(&("c".to_string(), "a".to_string())));
    assert!(edges.contains(&("a".to_string(), "b".to_string())));
    assert!(!edges.contains(&("b".to_string(), "c".to_string())));
}

#[test]
fn repeated_step_ids_keep_the_first_occurrence() {
    let lanes = vec![lane("l1", "Writer")];
    let steps = vec![step("s1", "l1", "First"), step("s1", "l1", "Second")];
    let diagram = normalise_run(&lanes, &steps);
    assert_eq!(diagram.steps.len(), 1);
    assert_eq!(diagram.steps[0].step.title, "First");
}

#[test]
fn connectors_run_left_to_right_across_columns() {
    let lanes = vec![lane("l1", "Writer"), lane("l2", "Reviewer")];
    let steps = vec![
        step("s1", "l1", "Draft"),
        step("s2", "l2", "Check"),
        step("s3", "l1", "Ship"),
    ];
    let diagram = normalise_run(&lanes, &steps);
    for (from, to) in &diagram.edges {
        assert!(diagram.steps[*from].column < diagram.steps[*to].column);
    }
}

#[test]
fn run_status_follows_the_steps() {
    let lanes = vec![lane("l1", "Writer")];
    let mut running = step("s1", "l1", "Draft");
    running.status = StepStatus::Running;
    let mut failed = step("s2", "l1", "Test");
    failed.status = StepStatus::Failed;
    let mut approval = step("s3", "l1", "Approve");
    approval.shape = StepShape::Approval;
    approval.status = StepStatus::Waiting;
    let mut done = step("s4", "l1", "Ship");
    done.status = StepStatus::Done;
    let mut skipped = step("s5", "l1", "Skip");
    skipped.status = StepStatus::Skipped;
    let status_of = |steps: Vec<Step>| run_status(&normalise_run(&lanes, &steps).steps);

    assert_eq!(
        status_of(vec![running.clone(), failed.clone()]),
        RunStatus::Running
    );
    assert_eq!(status_of(vec![failed.clone()]), RunStatus::Failed);
    assert_eq!(status_of(vec![approval.clone()]), RunStatus::Waiting);
    assert_eq!(status_of(vec![done.clone(), skipped]), RunStatus::Done);
    assert_eq!(done_count(&normalise_run(&lanes, &[done]).steps), 1);
}

#[test]
fn columns_share_the_container_width_and_a_narrow_one_scales_the_diagram_as_one_unit() {
    let wide = frame_for(Some(1200.0), 4);
    assert_eq!(wide.label, 136.0);
    assert_eq!(wide.column, 266.0);
    assert_eq!(wide.scale, 1.0);
    assert_eq!(wide.label + wide.stage_width, 1200.0);
    let narrow = frame_for(Some(300.0), 4);
    assert_eq!(narrow.label, 44.0);
    assert_eq!(narrow.column, 96.0);
    assert!(narrow.scale > 0.6 && narrow.scale < 1.0);
    assert!((narrow.scale * (narrow.label + narrow.stage_width) - 300.0).abs() < 1e-9);
}

#[test]
fn header_counts_boxes_and_diamonds_and_leaves_out_the_start_circle() {
    let lanes = vec![lane("l1", "Writer")];
    let mut start = step("a", "l1", "Start");
    start.shape = StepShape::Start;
    start.status = StepStatus::Done;
    let mut draft = step("b", "l1", "Draft");
    draft.status = StepStatus::Done;
    let ship = step("c", "l1", "Ship");
    let mut approve = step("d", "l1", "Approve");
    approve.shape = StepShape::Approval;
    approve.status = StepStatus::Waiting;
    let diagram = normalise_run(&lanes, &[start, draft, ship, approve]);
    assert_eq!(done_count(&diagram.steps), 1);
    assert_eq!(counted_total(&diagram.steps), 3);
}

#[test]
fn only_a_hand_over_into_a_waiting_step_carries_the_light() {
    let html = render(|| {
        let lanes = vec![Lane {
            id: "l1".into(),
            name: "Writer".into(),
            kind: LaneKind::Agent,
            role: None,
            mark: None,
            color: None,
            description: None,
        }];
        let mut draft = Step {
            id: "s1".into(),
            lane_id: "l1".into(),
            title: "Draft".into(),
            status: StepStatus::Running,
            shape: StepShape::Step,
            after: None,
            duration_label: None,
            detail: None,
        };
        draft.status = StepStatus::Running;
        let mut check = Step {
            id: "s2".into(),
            lane_id: "l1".into(),
            title: "Approve".into(),
            status: StepStatus::Waiting,
            shape: StepShape::Approval,
            after: None,
            duration_label: None,
            detail: None,
        };
        check.status = StepStatus::Waiting;
        let mut review = Step {
            id: "s3".into(),
            lane_id: "l1".into(),
            title: "Review".into(),
            status: StepStatus::Running,
            shape: StepShape::Step,
            after: None,
            duration_label: None,
            detail: None,
        };
        review.status = StepStatus::Running;
        rsx! { RunProcessLanes { run_title: "Launch", lanes, steps: vec![draft, review, check] } }
    });
    assert_eq!(
        html.matches("class=\"kit-rpl__light is-waiting\"").count(),
        1
    );
    assert_eq!(html.matches("class=\"kit-rpl__light\"").count(), 0);
}

#[test]
fn skipped_step_is_struck_through_and_a_waiting_run_says_waiting() {
    let html = render(|| {
        let lanes = vec![Lane {
            id: "l1".into(),
            name: "Writer".into(),
            kind: LaneKind::Agent,
            role: None,
            mark: None,
            color: None,
            description: None,
        }];
        let mut legacy = Step {
            id: "s1".into(),
            lane_id: "l1".into(),
            title: "Legacy cleanup".into(),
            status: StepStatus::Skipped,
            shape: StepShape::Step,
            after: None,
            duration_label: None,
            detail: None,
        };
        legacy.status = StepStatus::Skipped;
        let mut approve = Step {
            id: "s2".into(),
            lane_id: "l1".into(),
            title: "Approve".into(),
            status: StepStatus::Waiting,
            shape: StepShape::Approval,
            after: None,
            duration_label: None,
            detail: None,
        };
        approve.status = StepStatus::Waiting;
        rsx! { RunProcessLanes { run_title: "Launch", lanes, steps: vec![legacy, approve] } }
    });
    assert!(html.contains("class=\"kit-rpl__title-text is-skipped\""));
    assert!(html.contains(">Skipped<"));
    assert!(html.contains(">Waiting<"));
}

#[test]
fn connector_meets_the_ports_of_both_cells() {
    let (body, head) = connector_paths(0.0, 24.0, 100.0, 56.0, 50.0);
    assert_eq!(body, "M 0 24 L 50 24 L 50 56 L 100 56");
    assert!(head.contains("L 100 56"));
}

#[test]
fn connectors_turn_in_the_gap_before_their_column_and_never_inside_a_cell() {
    let frame = frame_for(Some(1200.0), 8);
    for column in 1..8 {
        let channel = frame.channel(column);
        let left_cell_right = frame.ports(column - 1, StepShape::Step).1;
        let right_cell_left = frame.ports(column, StepShape::Step).0;
        assert!(
            left_cell_right < channel && channel < right_cell_left,
            "column {column}: vertical run at {channel} between {left_cell_right} and {right_cell_left}"
        );
    }
}

#[test]
fn empty_run_says_nothing_to_hand_over() {
    let html = render(|| {
        rsx! {
            RunProcessLanes { lanes: vec![Lane { id: "l1".into(), name: "Writer".into(), kind: LaneKind::Agent, role: None, mark: None, color: None, description: None }], steps: Vec::new() }
        }
    });
    assert!(html.contains("Nothing to hand over yet"));
}

#[test]
fn box_is_named_by_title_lane_and_status_word() {
    let html = render(|| {
        let lanes = vec![Lane {
            id: "l1".into(),
            name: "Writer".into(),
            kind: LaneKind::Agent,
            role: Some("drafts".into()),
            mark: None,
            color: None,
            description: None,
        }];
        let mut draft = Step {
            id: "s1".into(),
            lane_id: "l1".into(),
            title: "Draft".into(),
            status: StepStatus::Running,
            shape: StepShape::Step,
            after: None,
            duration_label: None,
            detail: None,
        };
        draft.status = StepStatus::Running;
        rsx! { RunProcessLanes { run_title: "Launch", lanes, steps: vec![draft] } }
    });
    assert!(html.contains("aria-label=\"Draft, Writer, Running\""));
    assert!(html.contains("Launch"));
    assert!(html.contains("0 of 1 done"));
}

#[test]
fn run_lanes_reduced_motion_block_removes_connector_and_light_motion() {
    let css = include_str!("../assets/run-process-lanes.css");
    let block = css
        .split("@media (prefers-reduced-motion: reduce)")
        .nth(1)
        .expect("reduced-motion block");
    assert!(block.contains(".kit-rpl__edge"));
    assert!(block.contains(".kit-rpl__light"));
    assert!(block.contains("animation: none"));
}

#[test]
fn held_back_hides_a_partial_word_until_its_end_is_written() {
    assert_eq!(held_back("Hello wor"), "Hello ");
    assert_eq!(held_back("Hello"), "");
    assert_eq!(held_back("Hi there "), "Hi there ");
    assert_eq!(visible_text("Hello wor", StreamStatus::Streaming), "Hello ");
    assert_eq!(
        visible_text("Hello wor", StreamStatus::Stopped),
        "Hello wor"
    );
}

#[test]
fn stopped_offers_continue_and_dismiss_and_streaming_offers_stop_with_the_hint() {
    let streaming = render(|| {
        rsx! { InterruptibleAnswerStream { text: "Hello there", status: StreamStatus::Streaming } }
    });
    assert!(streaming.contains("Stop answer"));
    assert!(streaming.contains("Esc also stops"));
    assert!(!streaming.contains("Copy answer"));
    let stopped = render(|| {
        rsx! { InterruptibleAnswerStream { text: "Hello there", status: StreamStatus::Stopped } }
    });
    assert!(stopped.contains("Continue answer"));
    assert!(stopped.contains("Dismiss answer"));
    assert!(!stopped.contains("Stop answer"));
}

#[test]
fn failed_keeps_the_text_and_shows_the_error_with_retry() {
    let html = render(|| {
        rsx! {
            InterruptibleAnswerStream {
                text: "The first part.",
                status: StreamStatus::Failed,
                error_message: "The connection timed out.".to_string(),
            }
        }
    });
    assert!(html.contains("part. "));
    assert!(html.contains("The connection timed out."));
    assert!(html.contains("Retry answer"));
    assert!(!html.contains("Continue answer"));
}

#[test]
fn finished_shows_meta_copy_and_regenerate_only_when_supplied() {
    let meta = StreamMeta {
        tokens: Some(1240),
        duration_ms: Some(4200),
    };
    let plain = render(|| {
        let meta = StreamMeta {
            tokens: Some(1240),
            duration_ms: Some(4200),
        };
        rsx! { InterruptibleAnswerStream { text: "Done.", status: StreamStatus::Done, meta } }
    });
    assert!(plain.contains("1,240 tokens · 4.2 s"));
    assert!(plain.contains("Copy answer"));
    assert!(!plain.contains("Regenerate answer"));
    assert_eq!(meta_line(&meta).as_deref(), Some("1,240 tokens · 4.2 s"));

    let regen = render(|| {
        rsx! {
            InterruptibleAnswerStream {
                text: "Done.",
                status: StreamStatus::Done,
                on_regenerate: |_: ()| {},
            }
        }
    });
    assert!(regen.contains("Regenerate answer"));
}

#[test]
fn copied_text_carries_no_markup() {
    let text = "**Bold** and `code`\n```\nlet x = 1;\n```\n- item";
    assert_eq!(plain_text(text), "Bold and code\nlet x = 1;\nitem");
}

#[test]
fn steers_fold_past_three_and_the_first_queued_reads_next() {
    let steers: Vec<Steer> = (0..5)
        .map(|n| Steer {
            id: format!("s{n}"),
            text: format!("steer {n}"),
            kind: if n < 2 {
                SteerKind::Steer
            } else {
                SteerKind::Queued
            },
        })
        .collect();
    assert_eq!(fold_steers(&steers, false), (3, 2));
    assert_eq!(fold_steers(&steers, true), (5, 0));
    assert_eq!(steer_tag(&steers, "s2"), "Next");
    assert_eq!(steer_tag(&steers, "s3"), "Queued");
    assert_eq!(steer_tag(&steers, "s0"), "Steer");
}

#[test]
fn bend_splits_the_paragraph_it_falls_in() {
    let bends = vec![Bend {
        at: 8,
        text: Some("use a table".to_string()),
    }];
    let parts = body_parts("one two three four", &bends);
    assert_eq!(parts.len(), 3);
    assert!(matches!(parts[0], BodyPart::Paragraph(_)));
    assert_eq!(
        parts[1],
        BodyPart::Divider {
            at: 8,
            text: Some("use a table".to_string())
        }
    );
    assert!(matches!(parts[2], BodyPart::Paragraph(_)));
}

#[test]
fn bend_past_the_visible_text_waits_for_the_text_to_reach_it() {
    let bends = vec![Bend {
        at: 100,
        text: None,
    }];
    assert!(
        !body_parts("short", &bends)
            .iter()
            .any(|p| matches!(p, BodyPart::Divider { .. }))
    );
    assert!(
        body_parts(&"word ".repeat(30), &bends)
            .iter()
            .any(|p| matches!(p, BodyPart::Divider { .. }))
    );
}

#[test]
fn fenced_block_becomes_a_code_block_between_paragraphs() {
    let parts = body_parts("Intro\n```rust\nlet x = 1;\n```\nafter", &[]);
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[1], BodyPart::Code("let x = 1;".to_string()));
}

#[test]
fn sentences_are_announced_once_each_and_only_when_complete() {
    assert_eq!(
        completed_sentences("Hello there. How", 0, false),
        Some(("Hello there.".to_string(), 12))
    );
    assert_eq!(
        completed_sentences("Hello there. How are you? ", 12, false),
        Some(("How are you?".to_string(), 25))
    );
    assert_eq!(completed_sentences("Hello there", 0, false), None);
    assert_eq!(
        completed_sentences("Done.", 0, true),
        Some(("Done.".to_string(), 5))
    );
}

#[test]
fn status_changes_are_announced_with_their_reason() {
    assert_eq!(announcement(StreamStatus::Stopped, None), "Stopped.");
    assert_eq!(
        announcement(StreamStatus::Failed, Some("Timed out")),
        "Failed: Timed out"
    );
    assert_eq!(announcement(StreamStatus::Done, None), "Finished.");
    assert_eq!(announcement(StreamStatus::Streaming, None), "");
}

#[test]
fn code_block_scrolls_inside_itself_in_the_stream_stylesheet() {
    let css = include_str!("../assets/interruptible-answer-stream.css");
    let code = css
        .split(".kit-isa__code {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("code rule");
    assert!(code.contains("overflow-x: auto"));
    assert!(css.contains("@media (prefers-reduced-motion: reduce)"));
}
