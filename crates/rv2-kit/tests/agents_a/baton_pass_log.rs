use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::baton_pass_log::{
    AgentState, Assignment, BatonPassLog, BatonStage, PayloadItem, RelayAgent, Transfer,
    TransferStatus, agent_state, can_hand_over, column_of, held_by, preview_texts, transfer_status,
    transfers_between, up_next,
};

fn agent(id: &str, name: &str) -> RelayAgent {
    RelayAgent {
        id: id.to_string(),
        name: name.to_string(),
        model_label: Some("model".to_string()),
        role_text: None,
    }
}

fn relay() -> Vec<RelayAgent> {
    vec![
        agent("plan", "Planner"),
        agent("code", "Coder"),
        agent("review", "Reviewer"),
    ]
}

fn card(id: &str, stage: BatonStage, holder: Option<&str>) -> Assignment {
    Assignment {
        id: id.to_string(),
        caption: format!("Task {id}"),
        stage,
        holder_id: holder.map(str::to_string),
        memo: None,
    }
}

fn item(id: &str, dropped: bool) -> PayloadItem {
    PayloadItem {
        id: id.to_string(),
        text: format!("Item {id}"),
        entry_type: None,
        dropped,
        drop_reason: dropped.then(|| "outdated".to_string()),
    }
}

fn transfer(id: &str, from: &str, to: &str, task: &str) -> Transfer {
    Transfer {
        id: id.to_string(),
        from_agent_id: from.to_string(),
        to_agent_id: to.to_string(),
        task_ref: task.to_string(),
        payload: vec![
            item("1", false),
            item("2", true),
            item("3", false),
            item("4", false),
        ],
    }
}

#[test]
fn hand_over_only_moves_forward_and_pending_cards_go_anywhere() {
    assert!(can_hand_over(None, 0));
    assert!(can_hand_over(Some(0), 1));
    assert!(!can_hand_over(Some(1), 1));
    assert!(!can_hand_over(Some(2), 0));
}

#[test]
fn up_next_is_the_first_pending_todo_card() {
    let cards = vec![
        card("a", BatonStage::Working, Some("plan")),
        card("b", BatonStage::Todo, None),
        card("c", BatonStage::Todo, None),
    ];
    assert_eq!(up_next(&cards).map(|c| c.id.as_str()), Some("b"));
}

#[test]
fn held_card_prefers_unfinished_work() {
    let cards = vec![
        card("done", BatonStage::Done, Some("code")),
        card("live", BatonStage::Working, Some("code")),
    ];
    assert_eq!(held_by(&cards, "code").map(|c| c.id.as_str()), Some("live"));
}

#[test]
fn agent_is_working_when_it_holds_a_working_card_and_ready_when_todo() {
    let cards = vec![
        card("a", BatonStage::Working, Some("code")),
        card("b", BatonStage::Todo, Some("review")),
    ];
    assert_eq!(agent_state("code", &cards), AgentState::Working);
    assert_eq!(agent_state("review", &cards), AgentState::Ready);
    assert_eq!(agent_state("plan", &cards), AgentState::Idle);
}

#[test]
fn column_of_a_held_card_is_its_relay_position() {
    let cards = [card("a", BatonStage::Working, Some("code"))];
    assert_eq!(column_of(&relay(), &cards[0]), Some(1));
    assert_eq!(
        column_of(&relay(), &card("p", BatonStage::Todo, None)),
        None
    );
}

#[test]
fn transfers_between_neighbours_are_newest_first() {
    let transfers = vec![
        transfer("t1", "plan", "code", "Task a"),
        transfer("t2", "plan", "code", "Task b"),
        transfer("t3", "code", "review", "Task c"),
    ];
    let ids: Vec<&str> = transfers_between(&transfers, &relay(), 0)
        .into_iter()
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(ids, vec!["t2", "t1"]);
    assert!(transfers_between(&transfers, &relay(), 2).is_empty());
}

#[test]
fn transfer_status_reads_with_receiver_then_looks_good_when_done() {
    let t = transfer("t", "plan", "code", "Task a");
    let working = vec![card("a", BatonStage::Working, Some("code"))];
    assert_eq!(
        transfer_status(&t, &working, &relay()),
        TransferStatus::With("Coder".to_string())
    );
    let done = vec![card("a", BatonStage::Done, Some("code"))];
    assert_eq!(
        transfer_status(&t, &done, &relay()),
        TransferStatus::LooksGood
    );
}

#[test]
fn preview_shows_first_three_payload_texts() {
    let t = transfer("t", "plan", "code", "Task a");
    assert_eq!(
        preview_texts(&t.payload),
        vec!["Item 1", "Item 2", "Item 3"]
    );
}

fn relay_app() -> Element {
    rsx! {
        BatonPassLog {
            relay_agents: relay(),
            assignments: vec![
                card("p", BatonStage::Todo, None),
                card("c", BatonStage::Working, Some("code")),
            ],
            transfers: vec![transfer("t1", "plan", "code", "Task c")],
        }
    }
}

#[test]
fn relay_renders_agents_in_order_with_state_and_up_next() {
    let html = render(relay_app);
    let planner = html.find("Planner").expect("planner rendered");
    let coder = html.find("Coder").expect("coder rendered");
    let reviewer = html.find("Reviewer").expect("reviewer rendered");
    assert!(planner < coder && coder < reviewer);
    assert!(html.contains("Up next: Task p"));
    assert!(html.contains("Holding: Task c"));
    assert!(html.contains("data-state=\"working\""));
}

#[test]
fn dropped_payload_item_is_labelled_dropped_and_has_no_add_back_without_handler() {
    let html = render(relay_app);
    assert!(html.contains("data-dropped=\"true\""));
    assert!(html.contains("kit-baton__dropped"));
    assert!(html.contains("dropped</span>"));
    assert!(!html.contains("Add back"));
}

#[test]
fn transfer_card_present_at_mount_is_collapsed_and_not_in_transit() {
    let html = render(relay_app);
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(!html.contains("data-transit"));
    assert!(html.contains("aria-label=\"Payload\""));
}

#[test]
fn pending_card_is_a_draggable_button_with_pressed_state() {
    let html = render(relay_app);
    assert!(html.contains("draggable=\"true\""));
    assert!(html.contains("aria-pressed=\"false\""));
}
