//! Design review fixes that are structural: every categorical encoding carries a keyed legend.

use dioxus::prelude::*;
use rv2_kit::components::change_replay::{ChangeReplay, ReplayAuthor};
use rv2_kit::components::context_treemap::{Block, BlockKind, ContextTreemap};
use rv2_kit::components::daily_usage_bars::{DailyUsageBars, DayValue, Figure};
use rv2_kit::components::parallel_work_grid::{Item, ItemState, ParallelWorkGrid};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn replay_app() -> Element {
    rsx! {
        ChangeReplay {
            author: ReplayAuthor {
                author_id: "agent-7".into(),
                author_name: "Refactor agent".into(),
            },
        }
    }
}

#[test]
fn change_replay_legend_names_added_removed_and_baseline_swatches() {
    let html = render(replay_app);
    assert!(html.contains("kit-replay__legend"), "{html}");
    assert_eq!(count(&html, "kit-replay__key"), 3, "{html}");
    assert!(html.contains("Lines added") && html.contains("Lines removed"));
}

fn block(key: &str, kind: BlockKind, tokens: f64) -> Block {
    Block {
        block_key: key.into(),
        block_label: key.into(),
        block_kind: kind,
        token_count: tokens,
        block_note: None,
        sub_items: Vec::new(),
    }
}

fn treemap_app() -> Element {
    rsx! {
        ContextTreemap {
            block_list: vec![
                block("system", BlockKind::Instructions, 4000.0),
                block("chat", BlockKind::Conversation, 9000.0),
                block("shell", BlockKind::ToolOutput, 2000.0),
                block("chat-2", BlockKind::Conversation, 1000.0),
            ],
            window_size: 64000.0,
        }
    }
}

#[test]
fn context_treemap_legend_lists_each_kind_once_with_a_swatch() {
    let html = render(treemap_app);
    assert_eq!(count(&html, "kit-tmap__legend-row"), 3, "{html}");
    assert_eq!(count(&html, "kit-tmap__legend-swatch"), 3, "{html}");
}

fn grid_app() -> Element {
    let item = |id: &str, state: ItemState| Item {
        item_id: id.into(),
        item_label: id.into(),
        item_state: state,
        worker_name: None,
        start_offset: None,
        end_offset: None,
        item_cost: None,
        attempt_count: 1,
        output_value: None,
        error_text: None,
    };
    rsx! {
        ParallelWorkGrid {
            job_items: vec![
                item("a", ItemState::Done),
                item("b", ItemState::InProgress),
                item("c", ItemState::Retrying),
                item("d", ItemState::Failed),
            ],
        }
    }
}

#[test]
fn parallel_work_grid_filter_chips_carry_a_swatch_for_each_state() {
    let html = render(grid_app);
    assert_eq!(count(&html, "kit-pwg__swatch"), 4, "{html}");
    assert!(html.contains("data-state=\"InProgress\""));
    assert!(html.contains(">In progress<"));
}

fn today_app() -> Element {
    rsx! {
        DailyUsageBars {
            chart_heading: "Usage",
            day_values: vec![
                DayValue { day_label: "Mon".into(), day_value: Some(3.0) },
                DayValue { day_label: "Tue".into(), day_value: Some(5.0) },
                DayValue { day_label: "Wed".into(), day_value: Some(4.0) },
            ],
            total_figures: vec![Figure { figure_label: "Total".into(), figure_value: 12.0 }],
            is_today_live: true,
        }
    }
}

#[test]
fn daily_usage_bars_label_the_live_bar_as_today_on_the_axis() {
    let html = render(today_app);
    assert!(html.contains(">Today<"), "{html}");
    assert!(!html.contains(">Wed<"), "{html}");
}
