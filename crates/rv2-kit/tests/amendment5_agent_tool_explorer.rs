use dioxus::prelude::*;
use rv2_kit::screens::agent_tool_explorer::{
    Access, AgentInfo, AgentToolExplorer, CallRecord, CallState, Param, ToolInfo,
};

const NOW: u64 = 1_795_219_200_000;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn tool(id: &str, name: &str, access: Access) -> ToolInfo {
    ToolInfo {
        id: id.to_string(),
        entry_name: name.to_string(),
        function_name: format!("{id}_call"),
        source_label: "Built-in".to_string(),
        entry_description: format!("Lets the agent use {name}."),
        access_mode: access,
        parameter_list: vec![Param {
            entry_name: "query".to_string(),
            type_name: "string".to_string(),
            is_required: true,
        }],
    }
}

fn call(
    id: &str,
    tool_id: &str,
    state: CallState,
    ago_ms: u64,
    duration: Option<u64>,
) -> CallRecord {
    CallRecord {
        id: id.to_string(),
        tool_id: tool_id.to_string(),
        started_at: NOW - ago_ms,
        duration_ms: duration,
        call_state: state,
        input_data: format!("{tool_id} request"),
        output_data: Some("ok".to_string()),
        error_text: (state == CallState::Failed).then(|| "timeout".to_string()),
        token_count: Some(420),
        run_id: Some("run-7f2".to_string()),
    }
}

fn fixture_tools() -> Vec<ToolInfo> {
    vec![
        tool("docs", "Docs", Access::Auto),
        tool("mail", "Mail", Access::Ask),
        tool("crm", "CRM", Access::Off),
        tool("calendar", "Calendar", Access::Auto),
        tool("sheets", "Sheets", Access::Auto),
        tool("web", "Web search", Access::Ask),
        tool("files", "Files", Access::Auto),
    ]
}

fn fixture_calls() -> Vec<CallRecord> {
    vec![
        call("c1", "docs", CallState::Running, 4_000, None),
        call("c2", "mail", CallState::Waiting, 20_000, None),
        call("c3", "web", CallState::Failed, 90_000, Some(1_200)),
        call("c4", "docs", CallState::Done, 200_000, Some(480)),
        call("c5", "crm", CallState::Rejected, 300_000, None),
        call("c6", "calendar", CallState::Done, 540_000, Some(350)),
        call("c7", "sheets", CallState::Done, 840_000, Some(910)),
        call("c8", "files", CallState::Done, 1_500_000, Some(260)),
        call("c9", "docs", CallState::Done, 2_400_000, Some(610)),
    ]
}

fn rest_app() -> Element {
    rsx! {
        AgentToolExplorer {
            agent_info: Some(AgentInfo {
                id: "ada".to_string(),
                entry_name: "Ada".to_string(),
                task_name: "Support summary".to_string(),
                model_name: "Claude Sonnet".to_string(),
                is_paused: false,
            }),
            tool_list: fixture_tools(),
            call_log: fixture_calls(),
            now_at: NOW,
            is_live: true,
        }
    }
}

#[test]
fn tool_chart_at_rest_has_one_agent_card_and_seven_tool_cards_each_with_a_connector() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-ate__card\"").count(), 7);
    assert_eq!(
        html.matches("kit-ate__card kit-ate__card--agent").count(),
        1
    );
    assert_eq!(html.matches("kit-ate__link-line").count(), 7);
}

#[test]
fn tool_cards_state_their_access_in_words_four_allowed_two_ask_first_one_paused() {
    let html = render(rest_app);
    assert_eq!(html.matches("kit-ate__access\">Allowed<").count(), 4);
    assert_eq!(html.matches("kit-ate__access\">Ask first<").count(), 2);
    assert_eq!(html.matches("kit-ate__access\">Paused<").count(), 1);
}

#[test]
fn tool_call_table_at_rest_has_nine_rows_with_waiting_approve_and_reject() {
    let html = render(rest_app);
    assert_eq!(html.matches("kit-ate__state-word").count(), 9);
    assert!(html.contains("Approve"));
    assert!(html.contains("Reject"));
}

#[test]
fn only_the_card_of_the_running_call_shows_the_live_indicator() {
    let html = render(rest_app);
    assert_eq!(html.matches("kit-ate__live\"").count(), 1);
}

#[test]
fn header_reads_needs_you_while_a_call_waits_and_the_panel_is_closed() {
    let html = render(rest_app);
    assert!(html.contains("Needs you"));
    assert!(html.contains("Connect tool"));
    assert!(html.contains("Pause agent"));
    assert!(!html.contains("kit-ate__panel"));
}

#[test]
fn summary_tiles_are_five_and_total_calls_equals_the_call_rows() {
    let html = render(rest_app);
    for label in [
        "Failed calls",
        "Total calls",
        "Success share",
        "Average duration",
        "Tokens",
    ] {
        assert!(html.contains(label), "missing tile {label}");
    }
    assert_eq!(fixture_calls().len(), 9);
    assert!(html.contains("kit-ate__value\">9<"));
}
