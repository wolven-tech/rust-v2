//! SSR checks for the phone list of the run process lanes: both layouts render,
//! the list holds every node once in lane order, and one hand-over is lit.

use dioxus::prelude::*;
use rv2_kit::agents::run_process_lanes::{
    Lane, LaneKind, RunProcessLanes, Step, StepShape, StepStatus,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn lane(id: &str, name: &str, kind: LaneKind, role: &str) -> Lane {
    Lane {
        id: id.into(),
        name: name.into(),
        kind,
        role: Some(role.into()),
        mark: None,
        color: None,
        description: None,
    }
}

fn node(id: &str, lane_id: &str, title: &str, status: StepStatus, shape: StepShape) -> Step {
    Step {
        id: id.into(),
        lane_id: lane_id.into(),
        title: title.into(),
        status,
        shape,
        after: None,
        duration_label: None,
        detail: None,
    }
}

/// The gallery demo: five lanes and eight nodes, with one hand-over into a waiting approval.
fn demo_lanes() -> Vec<Lane> {
    vec![
        lane("approver", "Approver", LaneKind::Person, "Signs off"),
        lane("planner", "Planner", LaneKind::Agent, "Plans"),
        lane("coder", "Coder", LaneKind::Agent, "Writes code"),
        lane("tester", "Test runner", LaneKind::Tool, "Runs checks"),
        lane("releaser", "Release bot", LaneKind::Agent, "Ships"),
    ]
}

fn demo_steps() -> Vec<Step> {
    vec![
        node(
            "start",
            "approver",
            "Brief received",
            StepStatus::Done,
            StepShape::Start,
        ),
        node(
            "plan",
            "planner",
            "Plan the change",
            StepStatus::Done,
            StepShape::Step,
        ),
        node(
            "write",
            "coder",
            "Write the helper",
            StepStatus::Done,
            StepShape::Step,
        ),
        node(
            "tests",
            "tester",
            "Run tests",
            StepStatus::Running,
            StepShape::Step,
        ),
        node(
            "approve",
            "approver",
            "Approve deploy",
            StepStatus::Waiting,
            StepShape::Approval,
        ),
        node(
            "ship",
            "releaser",
            "Ship build",
            StepStatus::Queued,
            StepShape::Step,
        ),
        node(
            "legacy",
            "coder",
            "Legacy cleanup",
            StepStatus::Skipped,
            StepShape::Step,
        ),
        node(
            "finish",
            "releaser",
            "Run complete",
            StepStatus::Queued,
            StepShape::Finish,
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, mut step)| {
        step.after = match index {
            0 => None,
            1 => Some(vec!["start".into()]),
            2 => Some(vec!["plan".into()]),
            3 => Some(vec!["write".into()]),
            4 => Some(vec!["tests".into()]),
            5 => Some(vec!["approve".into()]),
            6 => Some(vec!["write".into()]),
            _ => Some(vec!["ship".into()]),
        };
        step
    })
    .collect()
}

fn demo_app() -> Element {
    rsx! { RunProcessLanes { run_title: "Launch", lanes: demo_lanes(), steps: demo_steps() } }
}

/// The phone list's markup, from its opening tag to the end of the document.
fn phone_part(html: &str) -> &str {
    html.split("class=\"kit-rpl__phone\"")
        .nth(1)
        .expect("phone list is rendered")
}

#[test]
fn wide_grid_and_phone_list_both_render_and_a_container_query_switches_them() {
    let html = render(demo_app);
    assert_eq!(html.matches("kit-rpl__wide").count(), 1);
    assert_eq!(html.matches("class=\"kit-rpl__phone\"").count(), 1);

    let css = include_str!("../assets/run-process-lanes.css");
    assert!(css.contains("container: kit-rpl / inline-size;"));
    assert!(css.contains("@container kit-rpl (max-width: 520px)"));
}

#[test]
fn phone_list_holds_every_node_exactly_once() {
    let html = render(demo_app);
    let phone = phone_part(&html);
    assert_eq!(phone.matches("class=\"kit-rpl__pl-step\"").count(), 8);
    for title in [
        "Brief received",
        "Plan the change",
        "Write the helper",
        "Run tests",
        "Approve deploy",
        "Ship build",
        "Legacy cleanup",
        "Run complete",
    ] {
        assert_eq!(
            phone.matches(&format!(">{title}</span>")).count(),
            1,
            "{title} listed once in the phone list"
        );
    }
}

#[test]
fn phone_lanes_stack_in_the_given_order_with_their_role_under_the_name() {
    let html = render(demo_app);
    let phone = phone_part(&html);
    let names = ["Approver", "Planner", "Coder", "Test runner", "Release bot"];
    let mut at = 0;
    for name in names {
        let found = phone[at..]
            .find(&format!(">{name}</h3>"))
            .unwrap_or_else(|| panic!("lane {name} heading follows the one before it"));
        at += found + 1;
    }
    assert!(phone.contains("<span class=\"kit-rpl__pl-role\">Signs off</span>"));
    let approver = phone
        .split("class=\"kit-rpl__pl-lane\"")
        .nth(1)
        .expect("first lane section");
    assert!(approver.contains("data-shape=\"start\""));
    assert!(approver.contains("data-shape=\"approval\""));
}

#[test]
fn phone_steps_read_in_dependency_order_within_the_lane_stack() {
    let html = render(demo_app);
    let phone = phone_part(&html);
    let order: Vec<&str> = phone
        .split("aria-label=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .filter(|label| label.contains(", "))
        .collect();
    let expected = [
        "Brief received, Approver, Done",
        "Approve deploy, Approver, Needs you",
        "Plan the change, Planner, Done",
        "Write the helper, Coder, Done",
        "Legacy cleanup, Coder, Skipped",
        "Run tests, Test runner, Running",
        "Ship build, Release bot, Queued",
        "Run complete, Release bot, Queued",
    ];
    assert_eq!(order, expected);
}

#[test]
fn exactly_one_hand_over_is_lit_and_it_names_its_target_lane() {
    let html = render(demo_app);
    let phone = phone_part(&html);
    assert_eq!(phone.matches("kit-rpl__pl-handover is-live").count(), 1);
    let lit = phone
        .split("kit-rpl__pl-handover is-live")
        .nth(1)
        .expect("lit hand-over");
    assert!(lit.starts_with("\" data-live=\"true\">Hands over to Approver"));
    assert!(lit.contains("Needs you"));
}

#[test]
fn reduced_motion_also_stops_the_phone_shimmer_and_breathing() {
    let css = include_str!("../assets/run-process-lanes.css");
    let block = css
        .split("@media (prefers-reduced-motion: reduce)")
        .nth(1)
        .expect("reduced-motion block");
    assert!(block.contains(".kit-rpl__pl-step[data-status=\"running\"]::after"));
    assert!(block.contains(".kit-rpl__pl-glyph"));
}
