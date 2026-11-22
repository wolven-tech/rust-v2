use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::baton_pass_log::{
    AgentState, Assignment, BatonPassLog, BatonStage, EntryType, PayloadItem, RelayAgent, Transfer,
    TransferStatus, agent_state, can_hand_over, column_of, held_by, lane_links, pending_cards,
    preview_texts, route_text, stage_of, transfer_status, transfers_between, up_next,
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

fn gallery_agent(id: &str, name: &str, role: &str) -> RelayAgent {
    RelayAgent {
        id: id.to_string(),
        name: name.to_string(),
        model_label: None,
        role_text: Some(role.to_string()),
    }
}

fn gallery_card(id: &str, caption: &str, stage: BatonStage, holder: Option<&str>) -> Assignment {
    Assignment {
        id: id.to_string(),
        caption: caption.to_string(),
        stage,
        holder_id: holder.map(str::to_string),
        memo: None,
    }
}

fn gallery_item(id: &str, text: &str, dropped: bool) -> PayloadItem {
    PayloadItem {
        id: id.to_string(),
        text: text.to_string(),
        entry_type: Some(EntryType::Fact),
        dropped,
        drop_reason: dropped.then(|| "Superseded by 2024 data".to_string()),
    }
}

fn gallery_transfer(
    id: &str,
    from: &str,
    to: &str,
    task: &str,
    payload: Vec<PayloadItem>,
) -> Transfer {
    Transfer {
        id: id.to_string(),
        from_agent_id: from.to_string(),
        to_agent_id: to.to_string(),
        task_ref: task.to_string(),
        payload,
    }
}

/// The amendment 2 gallery counts: three agents, two hand-over lanes of two cards each.
fn gallery_app() -> Element {
    rsx! {
        BatonPassLog {
            relay_agents: vec![
                gallery_agent("a1", "Intake", "Logs the brief"),
                gallery_agent("a2", "Drafter", "Writes the first draft"),
                gallery_agent("a3", "Checker", "Verifies claims"),
            ],
            assignments: vec![
                gallery_card("as1", "Interview notes", BatonStage::Done, None),
                gallery_card("as2", "Q3 draft", BatonStage::Working, Some("a2")),
                gallery_card("as3", "Customer quotes", BatonStage::Todo, Some("a3")),
                gallery_card("as4", "Q3 summary", BatonStage::Warning, Some("a3")),
                gallery_card("as5", "Launch notes", BatonStage::Todo, None),
                gallery_card("as6", "Press copy", BatonStage::Todo, None),
            ],
            transfers: vec![
                gallery_transfer("t1", "a1", "a2", "Interview notes", vec![
                    gallery_item("p1", "Interview transcript", false),
                    gallery_item("p2", "Pricing quote", false),
                ]),
                gallery_transfer("t2", "a1", "a2", "Q3 draft", vec![
                    gallery_item("p3", "Opening paragraph", false),
                    gallery_item("p4", "Quote from 2019 report", false),
                    gallery_item("p5", "Pricing table", false),
                ]),
                gallery_transfer("t3", "a2", "a3", "Customer quotes", vec![
                    gallery_item("p6", "Three customer quotes", false),
                    gallery_item("p7", "Churn figure", false),
                    gallery_item("p8", "Stale churn note", true),
                ]),
                gallery_transfer("t4", "a2", "a3", "Q3 summary", vec![
                    gallery_item("p9", "Headline figures", false),
                    gallery_item("p10", "Chart captions", false),
                ]),
            ],
            on_reinstate: move |_: (Transfer, PayloadItem)| {},
        }
    }
}

#[test]
fn route_names_both_agents_and_stage_follows_the_caption() {
    let relay = vec![
        gallery_agent("a1", "Intake", ""),
        gallery_agent("a2", "Drafter", ""),
    ];
    let t = gallery_transfer("t", "a1", "a2", "Q3 draft", Vec::new());
    assert_eq!(route_text(&t, &relay), "Intake → Drafter");
    let cards = [gallery_card(
        "c",
        "Q3 draft",
        BatonStage::Working,
        Some("a2"),
    )];
    assert_eq!(stage_of(&cards, "Q3 draft"), Some(BatonStage::Working));
    assert_eq!(stage_of(&cards, "Unknown"), None);
}

#[test]
fn pending_list_holds_only_unheld_todo_assignments() {
    let cards = vec![
        gallery_card("a", "Held todo", BatonStage::Todo, Some("x")),
        gallery_card("b", "Unheld done", BatonStage::Done, None),
        gallery_card("c", "Unheld warning", BatonStage::Warning, None),
        gallery_card("d", "Unheld todo", BatonStage::Todo, None),
    ];
    let ids: Vec<&str> = pending_cards(&cards)
        .into_iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(ids, vec!["d"]);
}

#[test]
fn gallery_agent_cards_render_in_relay_order_ahead_of_every_hand_over_lane() {
    let html = render(gallery_app);
    let names = ["Intake", "Drafter", "Checker"];
    let positions: Vec<usize> = names
        .iter()
        .map(|name| {
            html.find(&format!("aria-label=\"{name}, "))
                .expect("agent rendered")
        })
        .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    let last_agent = html
        .rfind("class=\"kit-baton__agent\"")
        .expect("agent card");
    let first_transfer = html
        .find("class=\"kit-baton__transfer\"")
        .expect("transfer card");
    assert!(
        last_agent < first_transfer,
        "a transfer card sits among the agent cards"
    );
}

#[test]
fn each_adjacent_pair_has_one_lane_holding_two_transfer_cards() {
    let html = render(gallery_app);
    let lanes: Vec<usize> = html
        .split("class=\"kit-baton__lane\"")
        .skip(1)
        .map(|segment| segment.matches("class=\"kit-baton__transfer\"").count())
        .collect();
    assert_eq!(lanes, vec![2, 2]);
    assert!(html.contains("Intake → Drafter</span>"));
    assert!(html.contains("Drafter → Checker</span>"));
    assert_eq!(html.matches("5 items</span>").count(), 2);
}

#[test]
fn lane_header_sits_on_the_gap_between_its_two_agents() {
    for lane in 0..2 {
        let (sender, receiver, header) = lane_links(3, lane);
        assert!((header - (sender + receiver) / 2.0).abs() < 1e-9);
    }
}

#[test]
fn connectors_drop_into_the_header_and_no_lane_crosses_a_neighbour() {
    let (s0, r0, h0) = lane_links(3, 0);
    let (s1, r1, h1) = lane_links(3, 1);
    assert!(s0 < h0 && h0 < r0);
    assert!(s1 < h1 && h1 < r1);
    assert!(r0 <= s1 + 1e-9, "lane one starts where lane zero stops");
}

#[test]
fn lane_cards_fade_toward_the_lower_edge() {
    let css = include_str!("../../assets/baton-pass-log.css");
    assert!(css.contains("mask-image: linear-gradient(to bottom, currentColor 60%"));
}

#[test]
fn gallery_relay_shows_every_stage_word_and_the_at_rest_agent_states() {
    let html = render(gallery_app);
    for word in ["Waiting", "Working", "Done", "Needs a look"] {
        assert!(html.contains(word), "missing stage word {word}");
    }
    assert!(html.contains("Up next: Launch notes · Waiting"));
    assert!(html.contains("Ready for a task"));
    assert!(html.contains("Looks good"));
    assert!(html.contains("With Checker"));
    assert!(html.contains("Needs a look"));
}

#[test]
fn gallery_transfer_header_names_both_agents_and_its_item_count() {
    let html = render(gallery_app);
    assert!(html.contains("Intake → Drafter · 2 items"));
    assert!(html.contains("Intake → Drafter · 3 items"));
    assert!(html.contains("Drafter → Checker · 3 items"));
    assert!(html.contains("Drafter → Checker · 2 items"));
}

#[test]
fn gallery_dropped_item_is_labelled_and_its_reason_stays_hidden_until_opened() {
    let html = render(gallery_app);
    assert_eq!(html.matches("data-dropped=\"true\"").count(), 1);
    assert!(html.contains("dropped</span>"));
    assert!(!html.contains("Dropped: Superseded"));
    assert!(!html.contains("Add back"));
}

#[test]
fn gallery_pending_list_shows_only_the_two_unheld_todo_cards() {
    let html = render(gallery_app);
    let pending = html
        .split("class=\"kit-baton__pending\"")
        .nth(1)
        .expect("pending section rendered");
    assert!(pending.contains("Launch notes"));
    assert!(pending.contains("Press copy"));
    for other in [
        "Q3 draft",
        "Q3 summary",
        "Interview notes",
        "Customer quotes",
    ] {
        assert!(!pending.contains(other), "pending list shows {other}");
    }
}
