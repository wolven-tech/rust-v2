use dioxus::prelude::*;
use rv2_kit::screens::orchestrator_thread::{
    Delivery, OrchestratorThread, Part, RowState, ThreadMessage, ToolRow,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn tool(id: &str, state: RowState) -> ToolRow {
    ToolRow {
        id: id.to_string(),
        entry_label: format!("Lookup {id}"),
        row_state: state,
        duration_ms: Some(300),
        error_text: None,
    }
}

fn running_thread() -> Vec<ThreadMessage> {
    vec![ThreadMessage::Agent {
        id: "a1".to_string(),
        name: "Orchestrator".to_string(),
        time_label: "09:05".to_string(),
        parts: vec![Part::Tools(vec![
            tool("t1", RowState::Running),
            tool("t2", RowState::Running),
            tool("t3", RowState::Running),
        ])],
        is_streaming: false,
        is_stopped: false,
    }]
}

fn finished_thread() -> Vec<ThreadMessage> {
    vec![
        ThreadMessage::User {
            id: "u1".to_string(),
            text: "Check renewals.".to_string(),
            delivery: Delivery::Sent,
        },
        ThreadMessage::Agent {
            id: "a1".to_string(),
            name: "Orchestrator".to_string(),
            time_label: "09:02".to_string(),
            parts: vec![Part::Tools(vec![
                tool("t1", RowState::Done),
                tool("t2", RowState::Done),
            ])],
            is_streaming: false,
            is_stopped: false,
        },
    ]
}

fn at_rest_app() -> Element {
    rsx! {
        OrchestratorThread {
            message_list: running_thread(),
            running_count: 9,
            needs_count: 5,
            approval_chip: Some("Needs approval".to_string()),
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

fn finished_app() -> Element {
    rsx! {
        OrchestratorThread {
            message_list: finished_thread(),
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

fn open_details_tag(html: &str) -> &str {
    let start = html.find("<details").expect("a tool group");
    let end = start + html[start..].find('>').expect("tag end");
    &html[start..end]
}

#[test]
fn parallel_group_at_rest_shows_three_running_rows_and_is_not_folded() {
    let html = render(at_rest_app);
    assert_eq!(html.matches("data-state=\"running\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-ot__tool-row\"").count(), 3);
    assert!(open_details_tag(&html).contains("open"));
    assert!(html.contains("Running 3 tools in parallel"));
}

#[test]
fn finished_group_folds_to_one_summary_line_with_its_total_time() {
    let html = render(finished_app);
    assert!(!open_details_tag(&html).contains("open"));
    assert!(html.contains("Ran 2 tools"));
    assert!(html.contains("0.6s"));
}

#[test]
fn stop_control_sits_above_the_composer_only_while_a_tool_row_runs() {
    let running = render(at_rest_app);
    let stop = running
        .find("class=\"kit-ot__stop\"")
        .expect("stop while running");
    let composer = running.find("class=\"kit-ot__composer").expect("composer");
    assert!(stop < composer);
    assert!(running.contains(">Stop<"));

    let finished = render(finished_app);
    assert!(!finished.contains("kit-ot__stop\""));
}

#[test]
fn composer_has_plus_approval_chip_mic_and_send_and_no_text_buttons_for_attach_or_voice() {
    let html = render(at_rest_app);
    assert!(html.contains("placeholder=\"Message the orchestrator\""));
    assert!(html.contains("aria-label=\"Attach file\""));
    assert!(html.contains("Needs approval"));
    assert!(html.contains("aria-label=\"Voice note\""));
    assert!(html.contains("aria-label=\"Send\""));
    assert!(!html.contains(">Attach<"));
    assert!(!html.contains(">Voice<"));
}

#[test]
fn top_row_holds_two_dot_counts_and_no_title_dock_or_attachment_row_at_rest() {
    let html = render(at_rest_app);
    assert!(html.contains("9 running"));
    assert!(html.contains("5 waiting on you"));
    assert!(html.contains("kit-ot__dot kit-ot__dot--ok"));
    assert!(html.contains("kit-ot__dot kit-ot__dot--warn"));
    assert!(html.contains("aria-label=\"Toggle panel\""));
    assert!(!html.contains("kit-ot__title"));
    assert!(!html.contains("kit-ot__dock"));
    assert!(!html.contains("kit-ot__attachments"));
    assert!(!html.contains("kit-ot__jump"));
}

#[test]
fn thread_column_is_fifty_eight_percent_wide_and_centred() {
    let css = include_str!("../assets/orchestrator-thread.css");
    assert!(css.contains(".kit-ot__col {\n  width: 58%;\n  margin-inline: auto;\n}"));
    assert!(css.contains("max-width: min(90%, 520px)"));
}
