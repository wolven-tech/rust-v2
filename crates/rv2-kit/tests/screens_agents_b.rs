use dioxus::prelude::*;
use rv2_kit::screens::autonomous_run_workspace::{
    Approval, AutonomousRunWorkspace, CallState, FactRow, Risk, RunRecord, RunState, RunStep,
    StepPhase, ToolCall, elapsed_text, in_plan_order, simulate, waiting_text,
};

#[test]
fn plan_order_keeps_the_persons_order_and_appends_unnamed_steps() {
    let step = |id: &str| run_step(id, id, StepPhase::Pending, false);
    let steps = vec![step("a"), step("b"), step("c")];
    let keys = vec!["c".to_string(), "a".to_string()];
    let ids: Vec<String> = in_plan_order(&steps, Some(&keys))
        .into_iter()
        .map(|s| s.id)
        .collect();
    assert_eq!(ids, vec!["c", "a", "b"]);
    let host: Vec<String> = in_plan_order(&steps, None)
        .into_iter()
        .map(|s| s.id)
        .collect();
    assert_eq!(host, vec!["a", "b", "c"]);
}
use rv2_kit::screens::multi_agent_run_canvas::{
    AgentNode, CanvasRun, CanvasRunState, MultiAgentRunCanvas, NodeState, fit_zoom,
    nearest_in_direction, zoom_step,
};
use rv2_kit::screens::orchestrator_thread::{
    Attachment, Connection, Decision, DecisionOption, Delivery, OrchestratorThread, Part,
    ThreadMessage, UploadState, clock_text, count_words, retry_seconds, reveal_prefix,
};

fn grid_positions() -> Vec<(String, f64, f64)> {
    [
        ("lead", 50.0, 12.0),
        ("research", 25.0, 50.0),
        ("write", 75.0, 50.0),
        ("review", 50.0, 88.0),
    ]
    .iter()
    .map(|(id, x, y)| (id.to_string(), *x, *y))
    .collect()
}

#[test]
fn arrow_moves_to_the_nearest_node_in_that_direction() {
    let p = grid_positions();
    let at = |from: &str, dx: i8, dy: i8| nearest_in_direction(&p, from, dx, dy);
    assert_eq!(at("lead", 0, 1).as_deref(), Some("research"));
    assert!(at("review", 0, -1).is_some());
    assert_eq!(at("review", -1, 0).as_deref(), Some("research"));
    assert_eq!(at("write", 1, 0), None);
}

#[test]
fn reconnect_countdown_rounds_up_and_stops_at_zero() {
    assert_eq!(retry_seconds(0, 12_000), 12);
    assert_eq!(retry_seconds(11_500, 12_000), 1);
    assert_eq!(retry_seconds(12_000, 12_000), 0);
    assert_eq!(retry_seconds(13_000, 12_000), 0);
}

fn reconnecting_app() -> Element {
    rsx! {
        OrchestratorThread {
            connection_state: Some(Connection::Reconnecting),
            retry_at_ms: Some(1_795_219_200_000 + 12_000),
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

#[test]
fn reconnecting_banner_names_the_countdown() {
    let html = render(reconnecting_app);
    assert!(html.contains("Reconnecting…"));
    assert!(html.contains("retry in 12 s"));
}

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn approval(title: &str) -> Approval {
    Approval {
        entry_title: title.to_string(),
        entry_description: "Sends one email to a customer.".to_string(),
        risk_level: Risk::High,
        fact_rows: vec![FactRow {
            entry_label: "To".to_string(),
            entry_value: "ops@harbor.example".to_string(),
        }],
    }
}

fn run_step(id: &str, title: &str, phase: StepPhase, with_approval: bool) -> RunStep {
    RunStep {
        id: id.to_string(),
        entry_title: title.to_string(),
        detail_text: None,
        tool_name: "Docs".to_string(),
        step_phase: phase,
        tool_call_list: vec![ToolCall {
            entry_name: "docs.search".to_string(),
            summary: "query: renewals".to_string(),
            args_data: "{}".to_string(),
            result_data: None,
            duration_ms: 480,
            call_state: CallState::Done,
        }],
        approval_request: with_approval.then(|| approval("Send the renewal email")),
    }
}

fn live_run(started_at_ms: u64, run_state: RunState, elapsed_seconds: u64) -> RunRecord {
    RunRecord {
        id: "run-7".to_string(),
        entry_title: "Renewal reminders".to_string(),
        agent_name: "Ada".to_string(),
        started_at_ms,
        run_state,
        elapsed_seconds,
        use_count: 3,
        cost_label: "$0.18".to_string(),
        activity_list: Vec::new(),
    }
}

#[test]
fn simulation_finishes_steps_in_order_and_holds_the_step_it_is_on() {
    let steps = vec![
        run_step("a", "Collect", StepPhase::Pending, false),
        run_step("b", "Send", StepPhase::Pending, true),
        run_step("c", "Log", StepPhase::Pending, false),
    ];
    let phases = |out: Vec<RunStep>| out.iter().map(|s| s.step_phase).collect::<Vec<_>>();
    assert_eq!(
        phases(simulate(&steps, 1, StepPhase::Waiting)),
        vec![StepPhase::Done, StepPhase::Waiting, StepPhase::Pending]
    );
    assert_eq!(
        phases(simulate(&steps, 3, StepPhase::Running)),
        vec![StepPhase::Done, StepPhase::Done, StepPhase::Done]
    );
    assert_eq!(
        phases(simulate(&steps, 0, StepPhase::Running)),
        vec![StepPhase::Running, StepPhase::Pending, StepPhase::Pending]
    );
}

#[test]
fn live_text_names_only_a_step_that_is_waiting() {
    let pending = vec![run_step("p", "Send", StepPhase::Pending, true)];
    assert_eq!(waiting_text(&pending), "");
    let waiting = vec![run_step("w", "Send", StepPhase::Waiting, true)];
    assert_eq!(
        waiting_text(&waiting),
        "Waiting for your approval: Send the renewal email"
    );
}

#[test]
fn elapsed_counts_from_the_host_start_and_falls_back_without_one() {
    let start = 1_795_000_000_000u64;
    let live = live_run(start, RunState::Waiting, 0);
    assert_eq!(elapsed_text(&live, start + 192_000), "3:12");
    let unset = live_run(0, RunState::Waiting, 192);
    assert_eq!(elapsed_text(&unset, 1_795_219_200_000), "3:12");
    let finished = live_run(start, RunState::Done, 65);
    assert_eq!(elapsed_text(&finished, start + 900_000), "1:05");
}

fn waiting_app() -> Element {
    rsx! {
        AutonomousRunWorkspace {
            run_record: Some(live_run(0, RunState::Waiting, 0)),
            step_list: vec![
                run_step("p1", "Collect renewals", StepPhase::Done, false),
                run_step("p3", "Send the email", StepPhase::Waiting, true),
            ],
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

#[test]
fn approval_card_shows_for_the_waiting_step_without_a_selection() {
    let html = render(waiting_app);
    assert!(html.contains("Approve: Send the renewal email"));
    assert!(html.contains("Deny: Send the renewal email"));
    assert!(!html.contains("Nothing needs you yet"));
}

fn pending_approval_app() -> Element {
    rsx! {
        AutonomousRunWorkspace {
            run_record: Some(live_run(0, RunState::Running, 0)),
            step_list: vec![run_step("p3", "Send the email", StepPhase::Pending, true)],
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

#[test]
fn approval_on_a_step_that_has_not_started_is_not_asked_for_yet() {
    let html = render(pending_approval_app);
    assert!(html.contains("Nothing needs you yet"));
    assert!(!html.contains("Approve: Send the renewal email"));
}

#[test]
fn zoom_moves_in_tenths_within_bounds() {
    assert!((zoom_step(1.0, 1) - 1.1).abs() < 1e-9);
    assert!((zoom_step(1.0, -1) - 0.9).abs() < 1e-9);
    assert!((zoom_step(1.5, 1) - 1.5).abs() < 1e-9);
    assert!((zoom_step(0.5, -1) - 0.5).abs() < 1e-9);
}

#[test]
fn fit_steps_down_in_tenths_until_the_stage_fits() {
    assert!((fit_zoom(None) - 1.0).abs() < 1e-9);
    assert!((fit_zoom(Some(900.0)) - 1.0).abs() < 1e-9);
    assert!((fit_zoom(Some(504.0)) - 0.7).abs() < 1e-9);
    assert!((fit_zoom(Some(100.0)) - 0.5).abs() < 1e-9);
}

fn canvas_app() -> Element {
    rsx! {
        MultiAgentRunCanvas {
            run_record: Some(CanvasRun {
                id: "run-01".to_string(),
                entry_title: "Market brief".to_string(),
                run_state: CanvasRunState::Running,
                elapsed_seconds: 70.0,
                cost_label: "$1.26".to_string(),
                token_count: 27_000,
                agents_done: 1,
                agents_total: 2,
            }),
            agent_nodes: vec![AgentNode {
                id: "lead".to_string(),
                entry_name: "Orchestrator".to_string(),
                node_role: "orchestrator".to_string(),
                model_name: "Claude Sonnet".to_string(),
                node_state: NodeState::Running,
                parent_id: None,
                cost_label: Some("$0.42".to_string()),
                duration_seconds: None,
                token_count: None,
            }],
        }
    }
}

#[test]
fn canvas_offers_zoom_and_fit_controls_in_a_named_group() {
    let html = render(canvas_app);
    assert!(html.contains("aria-label=\"Canvas zoom\""));
    assert!(html.contains("aria-label=\"Zoom in\""));
    assert!(html.contains("aria-label=\"Zoom out\""));
    assert!(html.contains("aria-label=\"Fit to view\""));
}

#[test]
fn stream_reveal_moves_one_word_at_a_time() {
    let text = "a bc  d";
    assert_eq!(count_words(text), 3);
    assert_eq!(reveal_prefix(text, 0), "");
    assert_eq!(reveal_prefix(text, 1), "a ");
    assert_eq!(reveal_prefix(text, 2), "a bc  ");
    assert_eq!(reveal_prefix(text, 3), text);
    assert_eq!(reveal_prefix(text, 99), text);
}

#[test]
fn clock_text_is_minutes_and_two_digit_seconds() {
    assert_eq!(clock_text(192), "3:12");
    assert_eq!(clock_text(5), "0:05");
}

fn thread_messages() -> Vec<ThreadMessage> {
    vec![
        ThreadMessage::User {
            id: "u1".to_string(),
            text: "Draft the summary.".to_string(),
            delivery: Delivery::Failed,
        },
        ThreadMessage::Agent {
            id: "a1".to_string(),
            name: "Orchestrator".to_string(),
            time_label: "09:41".to_string(),
            parts: vec![Part::Text("one two three".to_string())],
            is_streaming: true,
            is_stopped: false,
        },
    ]
}

fn thread_decisions() -> Vec<Decision> {
    vec![Decision {
        id: "d1".to_string(),
        question_text: "Which tone should the summary use?".to_string(),
        task_name: "Q3 summary".to_string(),
        option_list: vec![
            DecisionOption {
                id: "o1".to_string(),
                entry_title: "Plain".to_string(),
                summary: None,
                is_recommended: true,
            },
            DecisionOption {
                id: "o2".to_string(),
                entry_title: "Formal".to_string(),
                summary: None,
                is_recommended: false,
            },
        ],
    }]
}

fn offline_app() -> Element {
    rsx! {
        OrchestratorThread {
            message_list: thread_messages(),
            connection_state: Some(Connection::Offline),
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

#[test]
fn offline_connection_shows_the_banner() {
    let html = render(offline_app);
    assert!(html.contains("Offline. New messages wait until the connection returns."));
}

fn online_app() -> Element {
    rsx! {
        OrchestratorThread {
            message_list: thread_messages(),
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

#[test]
fn no_banner_while_the_connection_is_up() {
    let html = render(online_app);
    assert!(!html.contains("kit-ot__banner"));
}

fn failed_and_streaming_app() -> Element {
    rsx! {
        OrchestratorThread {
            message_list: thread_messages(),
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

#[test]
fn failed_message_offers_retry_and_a_streaming_answer_is_marked_busy() {
    let html = render(failed_and_streaming_app);
    assert!(html.contains("Not sent"));
    assert!(html.contains(">Retry<"));
    assert!(html.contains("data-streaming=\"true\""));
    assert!(html.contains("one two three"));
}

fn attachments_app() -> Element {
    rsx! {
        OrchestratorThread {
            attachment_list: vec![
                Attachment { id: "f1".to_string(), name: "brief.docx".to_string(), upload_state: UploadState::Done },
                Attachment { id: "f2".to_string(), name: "notes.pdf".to_string(), upload_state: UploadState::Failed },
                Attachment { id: "f3".to_string(), name: "data.csv".to_string(), upload_state: UploadState::Uploading(40) },
            ],
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

#[test]
fn attachments_show_their_upload_state_and_failures_retry() {
    let html = render(attachments_app);
    assert!(html.contains("Attached"));
    assert!(html.contains("Upload failed"));
    assert!(html.contains("Uploading 40%"));
    assert!(html.contains("aria-label=\"Remove notes.pdf\""));
}

fn jump_app() -> Element {
    rsx! {
        OrchestratorThread {
            message_list: thread_messages(),
            new_below_count: 3,
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

#[test]
fn jump_pill_names_the_new_count_and_the_end_key() {
    let html = render(jump_app);
    assert!(html.contains("aria-label=\"Jump to latest, 3 new lines. Press End to jump.\""));
}

fn away_app() -> Element {
    rsx! {
        OrchestratorThread {
            message_list: thread_messages(),
            last_seen_at: Some(0),
            away_after_minutes: 30,
            clock_ms: Some(31 * 60_000),
        }
    }
}

#[test]
fn away_card_shows_after_the_threshold() {
    let html = render(away_app);
    assert!(html.contains("While you were away"));
}

fn decision_app() -> Element {
    rsx! {
        OrchestratorThread {
            message_list: thread_messages(),
            decision_queue: thread_decisions(),
            clock_ms: Some(1_795_219_200_000),
        }
    }
}

#[test]
fn docked_decision_puts_one_option_in_the_tab_order_and_names_the_recommended_one() {
    let html = render(decision_app);
    assert!(html.contains("tabindex=\"0\""));
    assert!(html.contains("tabindex=\"-1\""));
    assert!(html.contains("Recommended"));
    assert!(!html.contains("Recording voice note"));
}
