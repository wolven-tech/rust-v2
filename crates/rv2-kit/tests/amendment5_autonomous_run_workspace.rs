use dioxus::prelude::*;
use rv2_kit::screens::autonomous_run_workspace::{
    ActivityEntry, Approval, AutonomousRunWorkspace, CallState, DraftRecord, FactRow, FileEntry,
    FileKind, Risk, RunRecord, RunState, RunStep, StepPhase, ToolCall, activity_newest_first,
};

const NOW: u64 = 1_795_219_200_000;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn run(activity: Vec<ActivityEntry>) -> RunRecord {
    RunRecord {
        id: "run-7".to_string(),
        entry_title: "Renewal reminders".to_string(),
        agent_name: "Ada".to_string(),
        started_at_ms: NOW - 192_000,
        run_state: RunState::Running,
        elapsed_seconds: 192,
        use_count: 3,
        cost_label: "$0.18".to_string(),
        activity_list: activity,
    }
}

fn step(id: &str, title: &str, phase: StepPhase, approval: bool) -> RunStep {
    RunStep {
        id: id.to_string(),
        entry_title: title.to_string(),
        detail_text: None,
        tool_name: "Docs".to_string(),
        step_phase: phase,
        tool_call_list: vec![],
        approval_request: approval.then(|| Approval {
            entry_title: "Send the renewal email".to_string(),
            entry_description: "Sends one email to a customer.".to_string(),
            risk_level: Risk::High,
            fact_rows: vec![FactRow {
                entry_label: "To".to_string(),
                entry_value: "ops@harbor.example".to_string(),
            }],
        }),
    }
}

/// Step 1 is running with its first tool call done and its second running; steps 2 to 5 are pending.
fn rest_steps() -> Vec<RunStep> {
    let mut collect = step("p1", "Collect renewals", StepPhase::Running, false);
    collect.tool_call_list = vec![
        ToolCall {
            entry_name: "crm.list".to_string(),
            summary: "status: due".to_string(),
            args_data: "{}".to_string(),
            result_data: Some("14 accounts".to_string()),
            duration_ms: 910,
            call_state: CallState::Done,
        },
        ToolCall {
            entry_name: "docs.search".to_string(),
            summary: "query: renewals".to_string(),
            args_data: "{}".to_string(),
            result_data: None,
            duration_ms: 0,
            call_state: CallState::Running,
        },
    ];
    vec![
        collect,
        step("p2", "Draft the reminder", StepPhase::Pending, false),
        step("p3", "Check the terms", StepPhase::Pending, false),
        step("p4", "Send the email", StepPhase::Pending, false),
        step("p5", "Log the send", StepPhase::Pending, false),
    ]
}

fn steps(waiting: bool) -> Vec<RunStep> {
    vec![
        step("p1", "Collect renewals", StepPhase::Done, false),
        step("p2", "Draft the reminder", StepPhase::Running, false),
        step("p3", "Check the terms", StepPhase::Pending, false),
        step(
            "p4",
            "Send the email",
            if waiting {
                StepPhase::Waiting
            } else {
                StepPhase::Pending
            },
            waiting,
        ),
        step("p5", "Log the send", StepPhase::Pending, false),
    ]
}

fn activity() -> Vec<ActivityEntry> {
    vec![
        ActivityEntry {
            at_ms: NOW - 190_000,
            entry_text: "Run started".to_string(),
        },
        ActivityEntry {
            at_ms: NOW - 150_000,
            entry_text: "Read renewals.csv".to_string(),
        },
        ActivityEntry {
            at_ms: NOW - 120_000,
            entry_text: "Started step 1".to_string(),
        },
        ActivityEntry {
            at_ms: NOW - 75_000,
            entry_text: "docs.search returned 3 results".to_string(),
        },
        ActivityEntry {
            at_ms: NOW - 60_000,
            entry_text: "Collected 14 renewals".to_string(),
        },
        ActivityEntry {
            at_ms: NOW - 15_000,
            entry_text: "Drafting the reminder".to_string(),
        },
    ]
}

fn files() -> Vec<FileEntry> {
    ["renewals.csv", "reminder.md", "terms.md"]
        .iter()
        .map(|name| FileEntry {
            name: name.to_string(),
            kind: FileKind::Document,
            body: format!("{name} body"),
        })
        .collect()
}

fn rest_app() -> Element {
    rsx! {
        AutonomousRunWorkspace {
            run_record: Some(run(vec![ActivityEntry {
                at_ms: NOW - 15_000,
                entry_text: "Step 1, collect renewals: 2 tool calls, 0.9 s".to_string(),
            }])),
            step_list: rest_steps(),
            draft_record: Some(DraftRecord {
                original_text: "Your renewal is due soon.".to_string(),
                suggested_text: "Your renewal is due on 30 March.".to_string(),
                entry_label: Some("Reminder email · 9 changes".to_string()),
            }),
            file_map: files(),
            app_mark_list: vec!["M".to_string(), "D".to_string(), "C".to_string()],
            clock_ms: Some(NOW),
        }
    }
}

fn waiting_app() -> Element {
    rsx! {
        AutonomousRunWorkspace {
            run_record: Some(run(activity())),
            step_list: steps(true),
            file_map: files(),
            clock_ms: Some(NOW),
        }
    }
}

#[test]
fn needs_your_input_is_the_only_fixed_tab_and_reads_zero_at_rest() {
    let html = render(rest_app);
    assert!(html.contains("Needs your input"), "{html}");
    assert_eq!(html.matches("role=\"tab\"").count(), 1, "{html}");
    assert!(html.contains("kit-arw__count\">0<"), "{html}");
    assert!(html.contains("Nothing needs you yet."), "{html}");
}

#[test]
fn a_pending_approval_reads_waiting_under_needs_your_input_with_count_one() {
    let html = render(waiting_app);
    assert!(html.contains("kit-arw__count\">1<"));
    let needs = html.find("Needs your input").expect("needs tab");
    let approval = html.find("kit-arw__approval").expect("approval card");
    assert!(needs < approval);
}

#[test]
fn activity_lists_newest_first_with_six_entries() {
    let sorted = activity_newest_first(&activity());
    assert_eq!(sorted.len(), 6);
    assert_eq!(sorted[0].entry_text, "Drafting the reminder");
    assert_eq!(sorted[5].entry_text, "Run started");
    assert!(sorted.windows(2).all(|pair| pair[0].at_ms >= pair[1].at_ms));
}

#[test]
fn activity_sits_in_the_main_column_with_one_entry_at_rest() {
    let html = render(rest_app);
    let plan = html.find("kit-arw__plan").expect("plan");
    let activity = html
        .find("aria-label=\"Activity\"")
        .expect("activity section");
    let pane = html.find("kit-arw__pane").expect("right pane");
    assert!(plan < activity && activity < pane, "{html}");
    assert_eq!(html.matches("kit-arw__activity-list").count(), 1, "{html}");
    assert!(
        html.contains("Step 1, collect renewals: 2 tool calls, 0.9 s"),
        "{html}"
    );
}

#[test]
fn files_list_at_rest_has_three_rows_in_the_pane_under_the_draft() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-arw__file\"").count(), 3);
    let draft = html.find("aria-label=\"Draft\"").expect("draft section");
    let files = html.find("aria-label=\"Files\"").expect("files section");
    assert!(draft < files, "{html}");
}

#[test]
fn draft_actions_sit_above_the_diff_at_rest() {
    let html = render(rest_app);
    let draft = html.find("aria-label=\"Draft\"").expect("draft section");
    let accept = html.find("Accept all").expect("accept");
    let reject = html.find("Reject all").expect("reject");
    assert!(draft < accept && accept < reject);
}

#[test]
fn draft_caption_states_nine_changes_at_rest() {
    let html = render(rest_app);
    assert!(html.contains("Reminder email · 9 changes"), "{html}");
}

#[test]
fn run_figures_show_steps_done_tool_calls_elapsed_and_cost_with_app_marks() {
    let html = render(rest_app);
    assert!(html.contains("0 of 5"), "{html}");
    assert!(html.contains("kit-arw__apps"), "{html}");
    assert!(!html.contains("Uses"), "{html}");
}

#[test]
fn step_one_is_running_with_its_tool_calls_beneath_it() {
    let html = render(rest_app);
    assert!(html.contains("crm.list done 910 ms"), "{html}");
    assert!(html.contains("docs.search running 0 ms"), "{html}");
}

#[test]
fn header_names_the_agent_and_when_the_run_started() {
    let html = render(rest_app);
    assert!(html.contains("Ada · started 3 min ago"));
}

#[test]
fn each_step_shows_its_tool_calls_inside_the_plan() {
    fn app() -> Element {
        let mut collect = step("p1", "Collect renewals", StepPhase::Done, false);
        collect.tool_call_list = vec![ToolCall {
            entry_name: "docs.search".to_string(),
            summary: "query: renewals".to_string(),
            args_data: "{}".to_string(),
            result_data: Some("3 results".to_string()),
            duration_ms: 480,
            call_state: CallState::Done,
        }];
        rsx! {
            AutonomousRunWorkspace {
                run_record: Some(run(activity())),
                step_list: vec![collect],
                clock_ms: Some(NOW),
            }
        }
    }
    let html = render(app);
    assert!(html.contains("docs.search done 480 ms"));
    assert_eq!(html.matches("kit-arw__plan").count(), 1);
}
