use dioxus::prelude::*;
use rv2_kit::components::traffic_split_router::{
    Fallback, Policy, Route, Strategy, TrafficSplitRouter, normalise_shares,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn route(id: &str, name: &str, cost: f64, p95: f64) -> Route {
    Route {
        route_id: id.into(),
        route_name: name.into(),
        route_provider: None,
        route_mark: None,
        route_tone: None,
        cost_per_1k: cost,
        p95_seconds: p95,
    }
}

fn demo_routes() -> Vec<Route> {
    vec![
        route("fast", "Fast model", 0.4, 1.2),
        route("deep", "Deep model", 1.9, 4.8),
        route("batch", "Batch model", 0.2, 9.5),
        route("edge", "Edge model", 0.6, 2.1),
    ]
}

fn demo_strategies() -> Vec<Strategy> {
    vec![
        Strategy {
            strategy_id: "even".into(),
            strategy_label: "Even".into(),
            share_map: vec![
                ("fast".into(), 25.0),
                ("deep".into(), 25.0),
                ("batch".into(), 25.0),
                ("edge".into(), 25.0),
            ],
        },
        Strategy {
            strategy_id: "cost".into(),
            strategy_label: "Lowest cost".into(),
            share_map: vec![
                ("fast".into(), 20.0),
                ("deep".into(), 0.0),
                ("batch".into(), 60.0),
                ("edge".into(), 20.0),
            ],
        },
        Strategy {
            strategy_id: "latency".into(),
            strategy_label: "Lowest latency".into(),
            share_map: vec![
                ("fast".into(), 70.0),
                ("deep".into(), 0.0),
                ("batch".into(), 0.0),
                ("edge".into(), 30.0),
            ],
        },
    ]
}

fn live() -> Policy {
    Policy {
        active_strategy: "even".into(),
        fallback: Fallback {
            enabled: true,
            from_route_id: Some("fast".into()),
            to_route_id: Some("deep".into()),
        },
    }
}

fn demo() -> Element {
    let draft = Policy {
        active_strategy: "cost".into(),
        fallback: live().fallback,
    };
    rsx! {
        TrafficSplitRouter {
            route_list: demo_routes(),
            strategy_list: demo_strategies(),
            live_policy: live(),
            initial_draft: Some(draft),
            policy_revision: 7,
            environment_label: "Production",
        }
    }
}

#[test]
fn header_holds_title_above_environment_label_and_precedes_the_summary_row() {
    let html = render(demo);
    let title = html.find("kit-tsr__title").expect("title rendered");
    let env = html.find("kit-tsr__env").expect("environment rendered");
    let summary = html
        .find("kit-tsr__summary-row")
        .expect("summary row rendered");
    assert!(title < env && env < summary);
}

#[test]
fn summary_row_holds_draft_and_live_costs_leading_and_strategy_control_trailing() {
    let html = render(demo);
    let row = html
        .find("kit-tsr__summary-row")
        .expect("summary row rendered");
    let draft = html
        .find("data-role=\"draft\"")
        .expect("draft cost rendered");
    let live_cost = html.find("data-role=\"live\"").expect("live cost rendered");
    let strategies = html
        .find("kit-tsr__strategies")
        .expect("strategies rendered");
    let flow = html.find("kit-tsr__flow").expect("flow rendered");
    assert!(row < draft && draft < live_cost && live_cost < strategies && strategies < flow);
    assert!(html.contains("$0.3200"));
    assert!(html.contains("$0.7750"));
    assert!(html.contains("Live · Even"));
}

#[test]
fn strategy_control_is_one_row_of_three_segments_with_one_active() {
    let html = render(demo);
    let start = html
        .find("kit-tsr__strategies")
        .expect("strategies rendered");
    let end = html.find("kit-tsr__flow").expect("flow rendered");
    let row = &html[start..end];
    assert_eq!(count(row, "role=\"radio\""), 3);
    assert_eq!(count(row, "aria-checked=\"true\""), 1);
    assert!(row.contains("Lowest cost"));
}

#[test]
fn flow_draws_one_labelled_line_end_per_route_in_order_with_no_line_for_zero_share() {
    let html = render(demo);
    assert_eq!(count(&html, "class=\"kit-tsr__label-row\""), 4);
    assert_eq!(count(&html, "class=\"kit-tsr__line\""), 3);
    assert_eq!(count(&html, "data-share=\"zero\""), 1);
    let fast = html.find("Fast model").expect("fast label");
    let deep = html.find("Deep model").expect("deep label");
    let batch = html.find("Batch model").expect("batch label");
    let edge = html.find("Edge model").expect("edge label");
    assert!(fast < deep && deep < batch && batch < edge);
}

#[test]
fn flow_band_carries_no_input_slider_or_button_element() {
    let html = render(demo);
    let start = html.find("kit-tsr__flow").expect("flow rendered");
    let end = html.find("kit-tsr__table").expect("table rendered");
    let flow = &html[start..end];
    for control in [
        "<button",
        "<input",
        "<select",
        "<textarea",
        "slider",
        "tabindex",
    ] {
        assert!(!flow.contains(control), "flow holds {control}");
    }
}

#[test]
fn route_table_has_four_rows_with_share_bar_p95_and_cost_columns() {
    let html = render(demo);
    assert_eq!(count(&html, "th scope=\"row\""), 4);
    assert_eq!(count(&html, "class=\"kit-tsr__bar\""), 4);
    assert!(html.contains("scope=\"col\">Share"));
    assert!(html.contains("scope=\"col\">p95"));
    assert!(html.contains("scope=\"col\">Cost / 1k"));
}

#[test]
fn demo_shares_add_up_to_one_hundred_percent() {
    let strategy = demo_strategies()
        .into_iter()
        .find(|s| s.strategy_id == "cost")
        .expect("cost strategy");
    let ids: Vec<String> = demo_routes().iter().map(|r| r.route_id.clone()).collect();
    let total: f64 = normalise_shares(&strategy, &ids)
        .iter()
        .map(|(_, s)| s)
        .sum();
    assert!((total - 100.0).abs() < 1e-9);
}

#[test]
fn fallback_row_sits_between_table_and_footer_with_toggle_and_two_pickers() {
    let html = render(demo);
    let table = html.find("kit-tsr__table").expect("table rendered");
    let fallback = html.find("kit-tsr__fallback").expect("fallback rendered");
    let footer = html.find("kit-tsr__foot").expect("footer rendered");
    assert!(table < fallback && fallback < footer);
    assert_eq!(count(&html[fallback..footer], "kit-tsr__select"), 2);
    assert_eq!(count(&html[fallback..footer], "kit-tsr__switch"), 1);
}

#[test]
fn footer_reads_policy_state_revert_then_deploy_from_left_to_right() {
    let html = render(demo);
    let policy = html.find("kit-tsr__policy").expect("policy rendered");
    let state = html.find("kit-tsr__state").expect("state rendered");
    let revert = html.find("kit-tsr__revert").expect("revert rendered");
    let deploy = html.find("kit-tsr__deploy").expect("deploy rendered");
    assert!(policy < state && state < revert && revert < deploy);
    assert!(html.contains("Policy v7 · Production"));
}

#[test]
fn at_rest_the_draft_reads_unsaved_with_revert_and_deploy_enabled() {
    let html = render(demo);
    assert!(html.contains("Unsaved changes"));
    let revert = html.find("kit-tsr__revert").expect("revert rendered");
    let deploy = html.find("kit-tsr__deploy").expect("deploy rendered");
    for at in [revert, deploy] {
        let tag = &html[at..at + html[at..].find('>').expect("tag closes")];
        assert!(!tag.contains(" disabled"), "{tag}");
        assert!(!tag.contains("aria-disabled=\"true\""), "{tag}");
    }
}

#[test]
fn zero_share_route_stays_listed_in_the_table() {
    let html = render(demo);
    let table_start = html.find("kit-tsr__table").expect("table rendered");
    assert!(html[table_start..].contains("Deep model"));
}
