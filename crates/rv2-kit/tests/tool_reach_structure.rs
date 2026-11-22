use dioxus::prelude::*;
use rv2_kit::components::agent_tool_reach_map::{
    AccessTool, AgentState, AgentToolReachMap, CallState, Grant, MapAgent, ToolCall,
    call_count_text,
};

#[test]
fn one_call_reads_singular_and_any_other_count_plural() {
    assert_eq!(call_count_text(1), "1 call");
    assert_eq!(call_count_text(0), "0 calls");
    assert_eq!(call_count_text(4), "4 calls");
}

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn call(id: &str, state: CallState) -> ToolCall {
    ToolCall {
        call_id: id.into(),
        call_state: state,
        call_input: "{\"q\":\"pricing\"}".into(),
        call_output: Some("3 results".into()),
        call_error: None,
        called_at: 1.0,
        duration: Some(0.4),
        is_replay: false,
    }
}

fn tool(id: &str, name: &str) -> AccessTool {
    AccessTool {
        tool_id: id.into(),
        tool_name: name.into(),
        tool_mark: None,
        tool_summary: None,
    }
}

fn catalog() -> Vec<AccessTool> {
    vec![
        tool("search", "Web search"),
        tool("mail", "Mail"),
        tool("docs", "Docs"),
        tool("crm", "CRM"),
        tool("calendar", "Calendar"),
        tool("sheets", "Sheets"),
        tool("drive", "Drive"),
        tool("tickets", "Tickets"),
        tool("billing", "Billing"),
    ]
}

fn reviewer(docs_state: CallState) -> MapAgent {
    MapAgent {
        agent_id: "a3".into(),
        agent_name: "Reviewer".into(),
        model_name: Some("model B".into()),
        agent_mark: None,
        agent_state: AgentState::Waiting,
        agent_task: Some("Check citations".into()),
        granted_tools: vec![
            Grant {
                tool_id: "search".into(),
                call_list: vec![call("c3", CallState::Done), call("c4", CallState::Failed)],
            },
            Grant {
                tool_id: "mail".into(),
                call_list: vec![call("c6", CallState::Done)],
            },
            Grant {
                tool_id: "docs".into(),
                call_list: vec![call("c5", docs_state)],
            },
            Grant {
                tool_id: "crm".into(),
                call_list: vec![call("c7", CallState::Done), call("c8", CallState::Done)],
            },
            Grant {
                tool_id: "calendar".into(),
                call_list: vec![call("c9", CallState::Done)],
            },
            Grant {
                tool_id: "sheets".into(),
                call_list: vec![],
            },
        ],
    }
}

fn other_agent() -> MapAgent {
    MapAgent {
        agent_id: "a1".into(),
        agent_name: "Research".into(),
        model_name: None,
        agent_mark: None,
        agent_state: AgentState::Running,
        agent_task: None,
        granted_tools: vec![],
    }
}

fn board(docs_state: CallState) -> Element {
    rsx! {
        AgentToolReachMap {
            agent_list: vec![other_agent(), reviewer(docs_state)],
            tool_catalog: catalog(),
            selected_agent_id: Some("a3".to_string()),
            board_title: "Tool access",
        }
    }
}

fn demo() -> Element {
    board(CallState::Running)
}

fn demo_call_done() -> Element {
    board(CallState::Done)
}

/// Slice of the rendered map from one marker to the next.
fn between<'a>(html: &'a str, from: &str, to: &str) -> &'a str {
    let start = html.find(from).unwrap_or_else(|| panic!("no {from}"));
    let end = html[start..]
        .find(to)
        .map(|n| start + n)
        .unwrap_or(html.len());
    &html[start..end]
}

/// Tool names in the order they appear inside one column.
fn names_in(column: &str) -> Vec<String> {
    column
        .split("kit-atr__name\">")
        .skip(1)
        .map(|s| s.split('<').next().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn heading_row_puts_title_before_the_counts_line() {
    let html = render(demo);
    let title = html.find("kit-atr__title").expect("title rendered");
    let counts = html.find("kit-atr__counts").expect("counts rendered");
    assert!(title < counts);
}

#[test]
fn agent_tabs_carry_name_and_state_word_with_the_selected_one_marked() {
    let html = render(demo);
    assert_eq!(count(&html, "role=\"tab\""), 2);
    assert!(html.contains("Research · Running"));
    assert!(html.contains("Reviewer · Needs you"));
    assert_eq!(count(&html, "aria-selected=\"true\""), 1);
}

#[test]
fn leading_column_then_agent_then_trailing_column_in_that_order() {
    let html = render(demo);
    let lead = html.find("data-column=\"lead\"").expect("lead column");
    let agent = html.find("kit-atr__agent").expect("agent card");
    let trail = html.find("data-column=\"trail\"").expect("trailing column");
    assert!(lead < agent && agent < trail);
}

#[test]
fn leading_column_holds_granted_tools_with_calls_in_catalogue_order() {
    let html = render(demo);
    let lead = between(&html, "data-column=\"lead\"", "data-side=\"lead\"");
    assert_eq!(count(lead, "class=\"kit-atr__slot\""), 5);
    assert_eq!(
        names_in(lead),
        ["Web search", "Mail", "Docs", "CRM", "Calendar"]
    );
}

#[test]
fn trailing_column_holds_unused_granted_tool_then_no_access_tools() {
    let html = render(demo);
    let trail = between(&html, "data-column=\"trail\"", "</ul>");
    assert_eq!(count(trail, "class=\"kit-atr__slot\""), 4);
    assert_eq!(names_in(trail), ["Sheets", "Drive", "Tickets", "Billing"]);
    assert_eq!(count(trail, "kit-atr__state\">Not used yet<"), 1);
    assert_eq!(count(trail, "kit-atr__state\">No access<"), 3);
}

#[test]
fn map_shows_nine_tool_cards_six_solid_connectors_and_three_faded() {
    let html = render(demo);
    assert_eq!(count(&html, "class=\"kit-atr__card\""), 9);
    assert_eq!(count(&html, "kit-atr__card\" data-granted=\"true\""), 6);
    assert_eq!(count(&html, "kit-atr__card\" data-granted=\"false\""), 3);
    assert_eq!(
        count(&html, "class=\"kit-atr__wire\" data-granted=\"true\""),
        6
    );
    assert_eq!(
        count(&html, "class=\"kit-atr__wire\" data-granted=\"false\""),
        3
    );
}

#[test]
fn at_rest_one_card_reads_running_one_reads_not_used_yet_and_no_calls_panel_is_open() {
    let html = render(demo);
    assert_eq!(count(&html, "kit-atr__state\">Running<"), 1);
    assert_eq!(count(&html, "kit-atr__state\">Not used yet<"), 1);
    assert_eq!(count(&html, "class=\"kit-atr__calls\""), 0);
}

#[test]
fn running_card_is_in_the_leading_column_and_its_connector_carries_the_highlight() {
    let html = render(demo);
    let lead = between(&html, "data-column=\"lead\"", "data-side=\"lead\"");
    assert_eq!(count(lead, "kit-atr__state\">Running<"), 1);
    assert_eq!(count(&html, "data-running=\"true\""), 1);
    let gutter = between(&html, "data-side=\"lead\"", "data-side=\"trail\"");
    assert_eq!(count(gutter, "data-running=\"true\""), 1);
}

#[test]
fn counts_line_reads_running_used_of_catalogue_and_calls() {
    let html = render(demo);
    assert!(html.contains("1 running · 5 of 9 tools used · 7 calls"));
}

#[test]
fn changing_a_call_state_leaves_every_card_in_its_column() {
    let running = render(demo);
    let done = render(demo_call_done);
    for column in [
        ("data-column=\"lead\"", "data-side=\"lead\""),
        ("data-column=\"trail\"", "</ul>"),
    ] {
        assert_eq!(
            names_in(between(&running, column.0, column.1)),
            names_in(between(&done, column.0, column.1)),
        );
    }
}
