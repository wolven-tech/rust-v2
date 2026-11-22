use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::request_routing_map::{
    Channel, HandlerNode, LayoutMode, NavKey, NavTarget, RequestRoutingMap, RequestRow,
    RequestState, RequestTag, RouterNode, accent_class, announcement, changed_requests, flow_for,
    header_counts, header_line, is_dimmed, load_percent, nav_order, nav_step, percent_sure,
    queue_for, score_rows, sentence, shows_certainty, top_requests, visible_split,
    waiting_on_person,
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
    assert!(html.contains("1 routed · 1 waiting on you"));
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

/// The gallery's resting data. The sorting instance turns the incoming request into
/// the one being read; nothing else changes.
fn amendment_rows(sorting: bool) -> Vec<RequestRow> {
    let mut incoming = request("r1", "Login failing", RequestState::Incoming, None);
    incoming.certainty_fraction = 0.0;
    if sorting {
        incoming.request_state = RequestState::Sorting;
        incoming.certainty_fraction = 0.56;
    }
    let mut waiting = request(
        "r3",
        "Partnership inquiry",
        RequestState::NeedsPerson,
        Some("sam"),
    );
    waiting.certainty_fraction = 0.71;
    let mut billing = request(
        "r4",
        "Duplicate charge",
        RequestState::Assigned,
        Some("billing"),
    );
    billing.certainty_fraction = 0.94;
    let mut ops = request("r5", "Late delivery", RequestState::Assigned, Some("ops"));
    ops.certainty_fraction = 0.88;
    vec![incoming, waiting, billing, ops]
}

fn amendment_handlers() -> Vec<HandlerNode> {
    vec![
        handler("billing", "Billing", false),
        handler("sam", "Sam", true),
        handler("ops", "Ops", false),
        handler("support", "Support", false),
    ]
}

fn amendment_map() -> Element {
    rsx! {
        RequestRoutingMap {
            router_node: router(),
            handler_nodes: amendment_handlers(),
            request_rows: amendment_rows(false),
        }
    }
}

fn sorting_map() -> Element {
    rsx! {
        RequestRoutingMap {
            router_node: router(),
            handler_nodes: amendment_handlers(),
            request_rows: amendment_rows(true),
        }
    }
}

/// The markup of the first request card in `state`, up to its closing tag.
fn card_html<'a>(html: &'a str, state: &str) -> &'a str {
    let start = html
        .find(&format!("data-state=\"{state}\""))
        .expect("card rendered");
    let end = html[start..]
        .find("</button>")
        .map_or(html.len(), |n| start + n);
    &html[start..end]
}

fn top_row_html(html: &str) -> &str {
    let start = html.find("kit-rrm__requests").expect("top row rendered");
    let end = html.find("kit-rrm__col--router").expect("router rendered");
    &html[start..end]
}

#[test]
fn the_top_row_holds_exactly_the_incoming_card_and_the_one_waiting_on_the_person() {
    let html = render(amendment_map);
    let top = top_row_html(&html);
    assert_eq!(top.matches("class=\"kit-rrm__request\"").count(), 2);
    assert!(top.contains("data-state=\"incoming\""));
    assert!(top.contains("data-state=\"needs-person\""));
    assert_eq!(html.matches("data-state=\"assigned\"").count(), 2);
}

#[test]
fn the_incoming_card_has_no_certainty_bar_and_the_waiting_card_keeps_its_bar() {
    let html = render(amendment_map);
    assert!(!card_html(&html, "incoming").contains("kit-rrm__certainty"));
    assert!(card_html(&html, "needs-person").contains("kit-rrm__certainty"));
}

#[test]
fn the_router_sits_below_the_top_row_and_above_the_handler_lanes() {
    let html = render(amendment_map);
    let top = html.find("kit-rrm__requests").expect("top row");
    let router = html.find("kit-rrm__col--router").expect("router column");
    let lanes = html.find("kit-rrm__col--handlers").expect("handler row");
    assert!(top < router && router < lanes);
}

#[test]
fn billing_and_ops_each_list_one_request_and_support_says_its_queue_is_empty() {
    let rows = amendment_rows(false);
    let billing: Vec<&str> = queue_for("billing", &rows)
        .iter()
        .map(|r| r.request_key.as_str())
        .collect();
    let ops: Vec<&str> = queue_for("ops", &rows)
        .iter()
        .map(|r| r.request_key.as_str())
        .collect();
    assert_eq!(billing, vec!["r4"]);
    assert_eq!(ops, vec!["r5"]);
    assert!(queue_for("support", &rows).is_empty());
    let html = render(amendment_map);
    assert_eq!(html.matches("kit-rrm__queue-count\">1<").count(), 2);
    assert_eq!(html.matches("No requests queued").count(), 1);
    assert!(html.contains("Last routed: Duplicate charge"));
    assert!(html.contains("Last routed: Late delivery"));
    assert!(html.contains("No requests routed yet"));
}

#[test]
fn the_person_entry_is_a_line_and_a_tag_with_no_card() {
    let html = render(amendment_map);
    assert_eq!(html.matches("class=\"kit-rrm__handler\"").count(), 3);
    assert!(!html.contains("aria-label=\"Sam\""));
    assert_eq!(waiting_on_person(&amendment_rows(false)), 1);
    assert!(html.contains("class=\"kit-rrm__person-name\">Sam<"));
    assert!(html.contains("class=\"kit-rrm__person-tag\">1<"));
}

#[test]
fn the_sorting_card_bar_is_partly_filled_and_the_router_shows_its_reading() {
    let html = render(sorting_map);
    assert!(card_html(&html, "sorting").contains("width: 56%"));
    assert!(html.contains("Reading Login failing"));
    assert!(html.contains(">Reading<"));
    for percent in ["71%", "94%", "88%"] {
        assert!(html.contains(&format!("width: {percent}")), "{percent}");
    }
}

#[test]
fn the_header_counts_match_the_cards_and_name_sorting_only_while_it_happens() {
    let rest = render(amendment_map);
    assert!(rest.contains("2 routed · 1 waiting on you"));
    assert!(!rest.contains("sorting"));
    let sorting = render(sorting_map);
    assert!(sorting.contains("1 sorting · 2 routed · 1 waiting on you"));
}

#[test]
fn the_dialog_scores_list_every_handler_in_order() {
    let rows = amendment_rows(false);
    let labels: Vec<String> = score_rows(&rows[0], &amendment_handlers())
        .into_iter()
        .map(|(label, _)| label)
        .collect();
    assert_eq!(labels, vec!["Billing", "Sam", "Ops", "Support"]);
    assert_eq!(header_counts(&rows), (0, 1, 2));
}

#[test]
fn header_line_drops_the_sorting_count_when_nothing_is_sorting() {
    assert_eq!(header_line(0, 2, 1), "2 routed · 1 waiting on you");
    assert_eq!(
        header_line(1, 2, 1),
        "1 sorting · 2 routed · 1 waiting on you"
    );
}

#[test]
fn a_handler_load_bar_fills_one_fifth_per_request_in_progress() {
    assert_eq!(load_percent(None), 0);
    assert_eq!(load_percent(Some(1)), 20);
    assert_eq!(load_percent(Some(9)), 100);
}

#[test]
fn only_a_request_already_being_read_shows_a_certainty_bar() {
    assert!(!shows_certainty(RequestState::Incoming));
    assert!(shows_certainty(RequestState::Sorting));
}
