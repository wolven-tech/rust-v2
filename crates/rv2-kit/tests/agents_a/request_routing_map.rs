use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::request_routing_map::{
    Channel, HandlerNode, LayoutMode, NavKey, NavTarget, RequestRoutingMap, RequestRow,
    RequestState, RequestTag, RouterNode, accent_class, announcement, changed_requests, flow_for,
    header_counts, is_dimmed, nav_order, nav_step, percent_sure, queue_for, score_rows, sentence,
    top_requests, visible_split, waiting_on_person,
};
use std::collections::BTreeMap;

fn request(key: &str, title: &str, state: RequestState, assigned: Option<&str>) -> RequestRow {
    RequestRow {
        request_key: key.to_string(),
        request_title: title.to_string(),
        sender_label: Some("Ana".to_string()),
        channel_kind: Channel::Mail,
        preview_text: None,
        request_state: state,
        assigned_to_key: assigned.map(str::to_string),
        certainty_fraction: 0.4,
        score_map: BTreeMap::from([("support".to_string(), 0.8), ("legal".to_string(), 0.1)]),
        reason_text: None,
        tag_list: Vec::new(),
    }
}

fn handler(key: &str, label: &str, is_person: bool) -> HandlerNode {
    HandlerNode {
        handler_key: key.to_string(),
        handler_label: label.to_string(),
        model_label: None,
        handler_state: None,
        activity_text: None,
        load_count: None,
        queued_request_keys: Vec::new(),
        is_person,
        accent_slot: 2,
        handler_mark: None,
        model_mark: None,
    }
}

fn router() -> RouterNode {
    RouterNode {
        node_key: "router".to_string(),
        node_label: "Router".to_string(),
        model_label: Some("Fast model".to_string()),
        reading_text: None,
        model_mark: None,
    }
}

#[test]
fn certainty_reads_as_a_clamped_whole_percent() {
    assert_eq!(percent_sure(0.8), 80);
    assert_eq!(percent_sure(1.4), 100);
    assert_eq!(percent_sure(-0.2), 0);
    assert_eq!(percent_sure(f64::NAN), 0);
}

#[test]
fn accent_slots_clamp_to_one_through_six() {
    assert_eq!(accent_class(0), "slot-1");
    assert_eq!(accent_class(9), "slot-6");
    assert_eq!(accent_class(3), "slot-3");
}

#[test]
fn card_sentence_names_title_sender_state_handler_and_certainty() {
    let mut row = request("r", "Refund", RequestState::Assigned, Some("support"));
    row.certainty_fraction = 0.8;
    assert_eq!(
        sentence(&row, Some("Support")),
        "Refund, from Ana: Assigned, to Support, 80% sure"
    );
    let fresh = request("n", "Refund", RequestState::Incoming, None);
    assert_eq!(
        sentence(&fresh, None),
        "Refund, from Ana: Incoming, 40% sure"
    );
}

#[test]
fn a_route_goes_incoming_then_sorting_then_assigned_or_waiting_on_the_person() {
    assert_eq!(
        flow_for(false),
        vec![
            RequestState::Incoming,
            RequestState::Sorting,
            RequestState::Assigned
        ]
    );
    assert_eq!(flow_for(true).last(), Some(&RequestState::NeedsPerson));
}

#[test]
fn only_the_first_rows_within_the_limit_show_and_the_rest_count_as_hidden() {
    assert_eq!(visible_split(6, 4), (4, 2));
    assert_eq!(visible_split(2, 4), (2, 0));
}

#[test]
fn assigned_requests_leave_the_top_list_and_join_their_handlers_queue() {
    let rows = vec![
        request("a", "Refund", RequestState::Incoming, None),
        request("b", "Login", RequestState::Assigned, Some("support")),
    ];
    let top: Vec<&str> = top_requests(&rows)
        .iter()
        .map(|r| r.request_key.as_str())
        .collect();
    assert_eq!(top, vec!["a"]);
    let queue: Vec<&str> = queue_for("support", &rows)
        .iter()
        .map(|r| r.request_key.as_str())
        .collect();
    assert_eq!(queue, vec!["b"]);
}

#[test]
fn the_keyboard_walk_visits_top_requests_router_handler_then_its_queue() {
    let rows = vec![
        request("a", "Refund", RequestState::Incoming, None),
        request("b", "Login", RequestState::Assigned, Some("support")),
    ];
    let handlers = vec![
        handler("support", "Support", false),
        handler("person", "You", true),
    ];
    let top = top_requests(&rows);
    let order = nav_order(&top, &rows, &handlers);
    assert_eq!(
        order,
        vec![
            NavTarget::Request("a".to_string()),
            NavTarget::Router,
            NavTarget::Handler("support".to_string()),
            NavTarget::Request("b".to_string()),
        ]
    );
}

#[test]
fn arrow_keys_step_and_home_and_end_jump_to_the_ends() {
    assert_eq!(nav_step(None, 3, NavKey::Forward), Some(0));
    assert_eq!(nav_step(None, 3, NavKey::Back), Some(2));
    assert_eq!(nav_step(Some(0), 3, NavKey::Forward), Some(1));
    assert_eq!(nav_step(Some(0), 3, NavKey::Back), Some(0));
    assert_eq!(nav_step(Some(1), 3, NavKey::Home), Some(0));
    assert_eq!(nav_step(Some(1), 3, NavKey::End), Some(2));
    assert_eq!(nav_step(None, 0, NavKey::Forward), None);
}

#[test]
fn a_focused_handler_dims_every_other_element() {
    assert!(!is_dimmed(None, Some("support")));
    assert!(!is_dimmed(Some("support"), Some("support")));
    assert!(is_dimmed(Some("support"), Some("legal")));
    assert!(is_dimmed(Some("support"), None));
}

#[test]
fn every_handler_score_is_listed_in_handler_order() {
    let row = request("a", "Refund", RequestState::Incoming, None);
    let handlers = vec![
        handler("support", "Support", false),
        handler("legal", "Legal", false),
    ];
    assert_eq!(
        score_rows(&row, &handlers),
        vec![("Support".to_string(), 80), ("Legal".to_string(), 10)]
    );
}

#[test]
fn the_live_line_names_where_a_request_went() {
    let legal = request("l", "Contract", RequestState::NeedsPerson, None);
    assert_eq!(announcement(&legal, None), "Contract needs you.");
    let assigned = request("b", "Login", RequestState::Assigned, Some("support"));
    assert_eq!(
        announcement(&assigned, Some("Support")),
        "Login assigned to Support."
    );
}

#[test]
fn only_requests_whose_state_changed_since_mount_are_announced() {
    let known = vec![("a".to_string(), RequestState::Incoming)];
    let rows = vec![
        request("a", "Refund", RequestState::Sorting, None),
        request("b", "Login", RequestState::Incoming, None),
        request("c", "Stable", RequestState::Incoming, None),
    ];
    let known_stable = vec![
        ("a".to_string(), RequestState::Incoming),
        ("c".to_string(), RequestState::Incoming),
    ];
    let changed: Vec<&str> = changed_requests(&known, &rows)
        .iter()
        .map(|r| r.request_key.as_str())
        .collect();
    assert_eq!(changed, vec!["a", "b", "c"]);
    let changed: Vec<&str> = changed_requests(&known_stable, &rows)
        .iter()
        .map(|r| r.request_key.as_str())
        .collect();
    assert_eq!(changed, vec!["a", "b"]);
}

#[test]
fn header_counts_and_person_waiting_tally_the_states() {
    let rows = vec![
        request("a", "A", RequestState::Sorting, None),
        request("b", "B", RequestState::NeedsPerson, None),
        request("c", "C", RequestState::Assigned, Some("support")),
        request("d", "D", RequestState::NeedsPerson, None),
    ];
    assert_eq!(header_counts(&rows), (1, 2, 1));
    assert_eq!(waiting_on_person(&rows), 2);
}

fn map() -> Element {
    let mut r1 = request("r1", "Refund", RequestState::Incoming, None);
    r1.reason_text = Some("Refund policy".to_string());
    let r2 = request("r2", "Login bug", RequestState::Assigned, Some("support"));
    let mut r3 = request("r3", "Contract", RequestState::NeedsPerson, None);
    r3.tag_list = vec![
        RequestTag {
            label: "legal".to_string(),
            accent_slot: Some(3),
        },
        RequestTag {
            label: "vip".to_string(),
            accent_slot: None,
        },
        RequestTag {
            label: "eu".to_string(),
            accent_slot: None,
        },
        RequestTag {
            label: "dropped".to_string(),
            accent_slot: None,
        },
    ];
    rsx! {
        RequestRoutingMap {
            router_node: router(),
            handler_nodes: vec![handler("support", "Support", false), handler("person", "You", true)],
            request_rows: vec![r1, r2, r3],
            on_reassign: move |_: (RequestRow, String)| {},
        }
    }
}

#[test]
fn stacked_map_puts_assigned_work_in_the_queue_and_counts_person_waiting() {
    let html = render(map);
    assert!(html.contains("data-layout=\"stacked\""));
    assert!(html.contains("0 sorting · 1 waiting on you · 1 assigned"));
    assert!(html.contains("kit-rrm__queue"));
    assert!(html.contains("Login bug"));
    assert!(html.contains("data-count=\"1\""));
    assert!(html.contains("1 waiting"));
}

#[test]
fn only_card_handlers_get_a_card() {
    let html = render(map);
    assert_eq!(html.matches("class=\"kit-rrm__handler\"").count(), 1);
    assert!(!html.contains("aria-label=\"You\""));
}

#[test]
fn request_cards_read_as_sentences_and_open_a_dialog() {
    let html = render(map);
    assert!(html.contains("aria-label=\"Refund, from Ana: Incoming, 40% sure\""));
    assert!(html.contains("aria-haspopup=\"dialog\""));
}

#[test]
fn the_map_has_a_single_tab_stop() {
    let html = render(map);
    assert_eq!(html.matches("tabindex=\"0\"").count(), 1);
}

#[test]
fn tags_are_capped_at_three() {
    let html = render(map);
    assert!(html.contains(">legal<"));
    assert!(html.contains(">eu<"));
    assert!(!html.contains("dropped"));
}

#[test]
fn dialog_is_closed_until_a_request_opens() {
    let html = render(map);
    assert!(!html.contains("kit-rrm__dialog"));
    assert!(html.contains("aria-live=\"polite\""));
}

fn crowded() -> Element {
    let rows: Vec<RequestRow> = (0..5)
        .map(|n| {
            request(
                &format!("q{n}"),
                &format!("Item {n}"),
                RequestState::Incoming,
                None,
            )
        })
        .collect();
    rsx! {
        RequestRoutingMap {
            router_node: router(),
            handler_nodes: vec![handler("support", "Support", false)],
            request_rows: rows,
            max_visible_requests: 2usize,
            layout_mode: LayoutMode::SideBySide,
        }
    }
}

#[test]
fn past_the_limit_the_top_list_says_how_many_are_hidden() {
    let html = render(crowded);
    assert!(html.contains("3 more requests"));
    assert!(html.contains("data-layout=\"side-by-side\""));
    assert!(html.contains("Item 1"));
    assert!(!html.contains("Item 4"));
}
