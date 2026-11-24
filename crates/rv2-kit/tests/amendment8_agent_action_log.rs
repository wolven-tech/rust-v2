//! Amendment 8 of the agent action log: day-heading counts, relative times and the panel's sections.

use dioxus::prelude::*;
use rv2_kit::screens::agent_action_log::{Actor, AgentActionLog, AppRecord, StepRecord, ago_text};

const MINUTE: u64 = 60_000;
const NOW: u64 = 1_795_219_200_000 + (16 * 60 + 42) * MINUTE;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn step(id: &str, occurred_at: u64, sentence: &str) -> StepRecord {
    StepRecord {
        id: id.into(),
        occurred_at,
        app_id: "mail".into(),
        sentence: sentence.into(),
        task_name: Some("Weekly update".into()),
        actor: Actor::Automatic,
        rule_label: None,
        approved_at: None,
        detail_rows: vec![],
        preview_lines: vec![],
        undo_window: None,
        final_reason: None,
        is_undone: false,
        undone_at: None,
        cost_label: None,
    }
}

fn apps() -> Vec<AppRecord> {
    vec![AppRecord {
        id: "mail".into(),
        entry_name: "Mail".into(),
        account: Some("dana@acme.example".into()),
        mark: None,
    }]
}

fn two_day_app() -> Element {
    let steps = vec![
        step("a", NOW - 12 * MINUTE, "Sent one"),
        step("b", NOW - 40 * MINUTE, "Sent two"),
        step("c", NOW - 24 * 60 * MINUTE - 5 * MINUTE, "Sent three"),
    ];
    rsx! {
        AgentActionLog { app_list: apps(), step_list: steps, selected_step_id: Some("a".to_string()), clock_ms: Some(NOW) }
    }
}

#[test]
fn relative_times_read_in_minutes_then_hours() {
    assert_eq!(ago_text(NOW, NOW - 12 * MINUTE), "12 minutes ago");
    assert_eq!(ago_text(NOW, NOW - 14 * MINUTE), "14 minutes ago");
    assert_eq!(ago_text(NOW, NOW - 90 * MINUTE), "1 hour ago");
    assert_eq!(ago_text(NOW, NOW - 200 * MINUTE), "3 hours ago");
    assert_eq!(ago_text(NOW, NOW), "just now");
}

#[test]
fn each_day_heading_carries_the_count_of_rows_under_it() {
    let html = render(two_day_app);
    assert_eq!(html.matches("class=\"kit-al__day\"").count(), 2);
    assert!(html.contains("<span class=\"kit-al__daycount\">2</span>"));
    assert!(html.contains("<span class=\"kit-al__daycount\">1</span>"));
}

#[test]
fn the_open_step_panel_lists_its_sections_in_order() {
    let html = render(two_day_app);
    let order = ["Content", "Authority", "Reversal"];
    let positions: Vec<usize> = order
        .iter()
        .map(|title| {
            html.find(&format!("kit-al__section-title\">{title}<"))
                .unwrap_or(usize::MAX)
        })
        .collect();
    assert!(positions.iter().all(|p| *p != usize::MAX), "{positions:?}");
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(html.contains("kit-al__tab-close"));
}

#[test]
fn every_row_puts_its_actor_and_task_on_line_two() {
    let html = render(two_day_app);
    assert_eq!(html.matches("kit-al__meta2").count(), 3);
}
