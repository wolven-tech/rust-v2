use dioxus::prelude::*;
use rv2_kit::components::task_assignment_board::{Proposal, TaskAssignmentBoard, WorkItem, Worker};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn worker(id: &str, name: &str, model: Option<&str>, busy: f64, rate: f64) -> Worker {
    Worker {
        worker_id: id.into(),
        worker_name: name.into(),
        worker_model_name: model.map(str::to_string),
        worker_avatar: None,
        busy_minutes: busy,
        cost_per_minute: rate,
    }
}

fn item(id: &str, title: &str, minutes: f64, tools: &[&str], worker: Option<&str>) -> WorkItem {
    WorkItem {
        work_id: id.into(),
        work_title: title.into(),
        estimate_minutes: minutes,
        required_tools: tools.iter().map(|t| t.to_string()).collect(),
        assigned_worker_id: worker.map(str::to_string),
    }
}

fn demo_workers() -> Vec<Worker> {
    vec![
        worker("w1", "Model A", Some("model A"), 25.0, 0.02),
        worker("w2", "Model B", None, 5.0, 0.05),
        worker("w3", "Model C", Some("model C"), 0.0, 0.03),
        worker("w4", "Model D", None, 0.0, 0.04),
    ]
}

/// Five in the pool, queues of two, one, two and none; one item carries tools.
fn demo_items() -> Vec<WorkItem> {
    vec![
        item("k1", "Summarise calls", 10.0, &[], None),
        item("k2", "Draft reply", 20.0, &[], Some("w1")),
        item("k3", "Tag new leads", 15.0, &[], None),
        item("k4", "Check invoices", 8.0, &[], Some("w2")),
        item("k5", "Send weekly recap", 12.0, &[], None),
        item(
            "k6",
            "Reconcile refunds",
            18.0,
            &["CRM", "Sheets"],
            Some("w3"),
        ),
        item("k7", "Triage support inbox", 9.0, &[], None),
        item("k8", "Update pricing page", 14.0, &[], Some("w1")),
        item("k9", "Export ledger", 6.0, &[], Some("w3")),
        item("k10", "Draft follow-up", 7.0, &[], None),
    ]
}

fn demo_board() -> Element {
    rsx! {
        TaskAssignmentBoard {
            worker_list: demo_workers(),
            work_items: demo_items(),
            proposed_assignment: Some(Proposal { work_id: "k1".into(), worker_id: "w2".into() }),
            clock_minutes: 30.0,
        }
    }
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

/// The HTML from `start` up to (not including) `end`.
fn between<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
    let from = html.find(start).expect("start marker rendered");
    let to = html.find(end).expect("end marker rendered");
    &html[from..to]
}

#[test]
fn board_header_holds_heading_and_automatic_switch_before_the_pool() {
    let html = render(demo_board);
    let head = html.find("kit-board__head").expect("header rendered");
    let title = html.find("Delegate").expect("heading rendered");
    let switch = html.find("role=\"switch\"").expect("switch rendered");
    let pool = html.find("kit-board__unassigned").expect("pool rendered");
    assert!(head < title && title < switch && switch < pool);
}

#[test]
fn demo_renders_four_worker_rows_with_exactly_one_free() {
    let html = render(demo_board);
    assert_eq!(count(&html, "aria-label=\"Queue for"), 4);
    assert_eq!(count(&html, "kit-board__status is-free\""), 1);
    assert_eq!(count(&html, ">Free<"), 1);
    assert_eq!(count(&html, "· $0.00"), 1);
}

#[test]
fn unassigned_pool_lists_five_cards_in_one_list_with_title_and_estimate() {
    let html = render(demo_board);
    let pool = between(&html, "kit-board__unassigned", "kit-board__workers");
    assert_eq!(count(pool, "kit-board__item\""), 4);
    assert_eq!(count(pool, "kit-board__item is-proposed\""), 1);
    assert_eq!(count(pool, "kit-board__est"), 5);
    assert_eq!(count(pool, "kit-board__tools"), 0);
    assert_eq!(count(pool, "<ul"), 1);
}

#[test]
fn the_pool_and_the_worker_rows_sit_side_by_side_in_document_order() {
    let html = render(demo_board);
    let pool = html.find("kit-board__unassigned").expect("pool rendered");
    let rows = html.find("kit-board__rows").expect("worker rows rendered");
    assert!(pool < rows);
    assert!(html.contains("kit-board__body"));
}

#[test]
fn proposed_hand_off_shows_as_text_with_a_preview_badge_and_a_marker() {
    let html = render(demo_board);
    assert_eq!(count(&html, "kit-board__worker is-target"), 1);
    assert_eq!(count(&html, "kit-board__mark"), 1);
    assert_eq!(count(&html, ">proposed<"), 1);
    assert_eq!(count(&html, "kit-board__status is-preview"), 1);
    assert!(html.contains("Busy until 00:53"));
    assert!(html.contains("+$0.50 with Summarise calls"));
}

#[test]
fn worker_rows_number_one_to_four_in_list_order() {
    let html = render(demo_board);
    let mut cursor = 0;
    for n in 1..=4 {
        let needle = format!("kit-board__digit\" aria-hidden=\"true\">{n}<");
        let at = html[cursor..]
            .find(&needle)
            .unwrap_or_else(|| panic!("digit {n} missing after position {cursor}"));
        cursor += at + needle.len();
    }
}

#[test]
fn the_queue_item_with_tools_shows_its_list_and_no_other_item_does() {
    let html = render(demo_board);
    assert_eq!(count(&html, "kit-board__tools"), 1);
    assert!(html.contains("CRM, Sheets"));
}

#[test]
fn the_worker_list_total_sums_every_row_and_the_usage_hint_sits_under_both_columns() {
    let html = render(demo_board);
    assert!(html.contains("Total $2.55"));
    assert!(html.contains("$0.02 per min · $1.18"));
    let rows = html.find("kit-board__rows").expect("rows rendered");
    let usage = html.find("kit-board__usage").expect("usage hint rendered");
    assert!(rows < usage);
    assert!(html[usage..].contains("Drag to a worker"));
    assert!(html[usage..].contains("number"));
}

#[test]
fn a_money_formatter_changes_every_displayed_cost() {
    fn app() -> Element {
        rsx! {
            TaskAssignmentBoard {
                worker_list: demo_workers(),
                work_items: demo_items(),
                format_money: Some(Callback::new(|v: f64| format!("€{v:.2}"))),
            }
        }
    }
    let html = render(app);
    assert!(html.contains("Total €2.55"));
    assert!(html.contains("€0.02 per min · €1.18"));
    assert!(!html.contains('$'));
}

#[test]
fn overflow_chip_follows_third_card_of_a_long_queue() {
    fn app() -> Element {
        let workers = vec![worker("a", "Only", None, 0.0, 1.0)];
        let items = (0..5)
            .map(|i| item(&format!("i{i}"), &format!("Job {i}"), 1.0, &[], Some("a")))
            .collect();
        rsx! { TaskAssignmentBoard { worker_list: workers, work_items: items } }
    }
    let html = render(app);
    let third = html.find("Job 2").expect("third card rendered");
    let chip = html
        .find("kit-board__overflow")
        .expect("overflow chip rendered");
    assert!(third < chip);
    assert!(!html.contains("Job 3</span>"));
    assert!(html.contains("+2"));
    assert!(html.contains("Job 3, Job 4"));
}
