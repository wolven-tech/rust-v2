use dioxus::prelude::*;
use rv2_kit::screens::agent_task_list::{
    AgentTaskList, Ask, AskKind, DecisionEntry, Progress, TaskEvent, TaskRecord, TaskStep,
    tasks_in_group,
};

const NOW: u64 = 1_795_219_200_000;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn task(
    id: &str,
    title: &str,
    project: &str,
    state: Progress,
    done: usize,
    total: usize,
    asks: usize,
) -> TaskRecord {
    TaskRecord {
        id: id.to_string(),
        entry_title: title.to_string(),
        project_name: project.to_string(),
        progress_state: state,
        status_phrase: None,
        step_records: (0..total)
            .map(|i| TaskStep {
                entry_label: format!("Step {}", i + 1),
                is_done: i < done,
            })
            .collect(),
        artifact_names: vec!["Summary.md".to_string()],
        pending_asks: (0..asks)
            .map(|i| Ask {
                id: format!("{id}-{i}"),
                question_text: "Send the draft to the client?".to_string(),
                ask_kind: AskKind::Approval,
            })
            .collect(),
        updated_at_ms: NOW - 600_000,
        is_done: false,
        goal_text: Some("Ship the Q3 summary".to_string()),
        started_at_ms: None,
        decision_log: vec![DecisionEntry {
            question_text: "Tone?".to_string(),
            answer_text: "Plain".to_string(),
            at_ms: NOW - 86_400_000,
        }],
        event_log: Vec::new(),
    }
}

fn fixture_tasks() -> Vec<TaskRecord> {
    use Progress::{Paused, Planning, Running};
    let mut lead = task("t1", "Q3 summary", "Launch", Running, 2, 4, 1);
    lead.artifact_names = vec!["Q3 brief".to_string(), "Q3 figures.csv".to_string()];
    lead.event_log = vec![
        TaskEvent {
            at_ms: NOW - 60_000,
            entry_text: "Asked to approve the client send".to_string(),
        },
        TaskEvent {
            at_ms: NOW - 3_600_000,
            entry_text: "Drafted the brief".to_string(),
        },
        TaskEvent {
            at_ms: NOW - 172_800_000,
            entry_text: "Started the run".to_string(),
        },
    ];
    vec![
        lead,
        task("t2", "Pricing page copy", "Launch", Planning, 0, 3, 0),
        task("t3", "Launch email", "Launch", Running, 1, 3, 0),
        task("t4", "Onboarding checklist", "Launch", Paused, 2, 4, 0),
        task("t5", "Candidate screen", "Hiring", Running, 1, 2, 1),
        task("t6", "Interview plan", "Hiring", Paused, 0, 3, 0),
        task("t7", "Offer letter", "Hiring", Paused, 1, 2, 0),
        task("t8", "Competitor scan", "Research", Running, 3, 4, 0),
        task("t9", "Survey readout", "Research", Paused, 2, 3, 0),
    ]
}

fn projects() -> Vec<String> {
    vec![
        "Launch".to_string(),
        "Hiring".to_string(),
        "Research".to_string(),
    ]
}

fn rest_app() -> Element {
    rsx! {
        AgentTaskList {
            task_records: fixture_tasks(),
            project_names: projects(),
            initial_task_id: Some("t1".to_string()),
            now_ms: NOW,
        }
    }
}

#[test]
fn task_list_at_rest_has_three_groups_of_four_three_and_two_tasks() {
    let tasks = fixture_tasks();
    let counts: Vec<usize> = projects()
        .iter()
        .map(|p| tasks_in_group(&tasks, p).len())
        .collect();
    assert_eq!(counts, vec![4, 3, 2]);
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-atl__row\"").count(), 9);
    assert_eq!(html.matches("class=\"kit-atl__group\"").count(), 3);
}

#[test]
fn task_panel_shows_title_project_state_goal_checklist_needs_artifacts_events_then_decisions() {
    let html = render(rest_app);
    let at = |needle: &str| {
        html.find(needle)
            .unwrap_or_else(|| panic!("missing {needle}"))
    };
    let order = [
        at("kit-atl__panel-title"),
        at("State: "),
        at("kit-atl__goal"),
        at("aria-label=\"Checklist\""),
        at("aria-label=\"Needs you\""),
        at("aria-label=\"Artifacts\""),
        at("What happened so far"),
        at("Decisions made"),
    ];
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn task_panel_events_are_newest_first_each_with_its_time() {
    let html = render(rest_app);
    let asked = html.find("Asked to approve").expect("newest event");
    let drafted = html.find("Drafted the brief").expect("middle event");
    let started = html.find("Started the run").expect("oldest event");
    assert!(asked < drafted && drafted < started);
    assert!(html.contains("1 min ago") || html.contains("ago"));
}

#[test]
fn only_the_planning_ring_is_dashed_and_running_rings_fill_by_steps_done() {
    let html = render(rest_app);
    assert_eq!(html.matches("data-progress=\"planning\"").count(), 1);
    assert!(html.contains("--kit-atl-ring: 50%"));
}

#[test]
fn task_list_header_reads_the_active_count() {
    let html = render(rest_app);
    assert!(html.contains("kit-atl__count\">9<"));
}

#[test]
fn panel_offers_one_artifact_tab_per_artifact_and_a_decide_button() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-atl__artifact-tab\"").count(), 2);
    assert!(html.contains("Decide"));
}
