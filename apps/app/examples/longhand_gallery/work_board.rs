use dioxus::prelude::*;
use rv2_kit::components::background_jobs_popover::{BackgroundJobsPopover, Edge, JobState};
use rv2_kit::components::decision_review_queue::{DecideEvent, DecisionReviewQueue};
use rv2_kit::components::metric_tile::{MetricTile, MetricValue};
use rv2_kit::screens::workspace_frame_inset::WorkspaceFrame;

use super::Scene;
use super::data::{
    account, chats, menu_groups, menu_items, now_secs, percent, review_items, running_jobs,
};

const REJECT_REASONS: [&str; 3] = ["Wrong facts", "Too risky", "Not needed"];

#[component]
pub fn WorkBoard() -> Element {
    let mut jobs = use_signal(running_jobs);
    let mut activity = use_signal(|| {
        vec![
            "Monday pipeline review passed the CRM step".to_string(),
            "Weekly customer newsletter reached section 2".to_string(),
            "Budget crossed 80% of this month's plan".to_string(),
        ]
    });
    let mut nav = use_signal(|| "none yet".to_string());
    let job_now = jobs();
    let running = job_now
        .iter()
        .filter(|j| j.job_state == JobState::Running)
        .count();
    let lines = activity();
    let reasons: Vec<String> = REJECT_REASONS.iter().map(|r| r.to_string()).collect();

    rsx! {
        Scene {
            id: "work_board",
            title: "My Work Board",
            story: "Sam opens Longhand on Monday: three drafts wait for a look, workflows run in the background, and this month's budget has reached 84%.",
            div { class: "gl-canvas lh-frame",
                WorkspaceFrame {
                    current_item: "My Work Board".to_string(),
                    workspace_name: "Bramble Studio",
                    account: account(),
                    menu_items: menu_items(),
                    menu_groups: menu_groups(),
                    chat_list: chats(),
                    project_list: vec!["Clients".to_string()],
                    top_bar_title: rsx! { span { class: "lh-title", "My Work Board" } },
                    top_bar_actions: rsx! {
                        BackgroundJobsPopover {
                            job_list: job_now.clone(),
                            align_edge: Edge::Start,
                            on_pause: move |id: String| {
                                jobs.with_mut(|list| list.iter_mut().filter(|j| j.job_id == id).for_each(|j| j.job_state = JobState::Paused));
                                activity.with_mut(|a| a.insert(0, "A workflow was paused".to_string()));
                            },
                            on_resume: move |id: String| {
                                jobs.with_mut(|list| list.iter_mut().filter(|j| j.job_id == id).for_each(|j| j.job_state = JobState::Running));
                                activity.with_mut(|a| a.insert(0, "A workflow was resumed".to_string()));
                            },
                            on_open_job: move |id: String| nav.set(format!("open {id}")),
                        }
                    },
                    on_navigate: move |label: String| nav.set(label),
                    content: rsx! {
                        div { class: "lh-stack", style: "padding: 16px;",
                            div { class: "lh-tiles",
                                MetricTile {
                                    metric_label: "Waiting for you",
                                    metric_value: MetricValue::Number(3.0),
                                    change_note: Some("1 due today".to_string()),
                                }
                                MetricTile {
                                    metric_label: "Running now",
                                    metric_value: MetricValue::Number(running as f64),
                                    change_note: Some("workflows in the background".to_string()),
                                }
                                MetricTile {
                                    metric_label: "Budget used",
                                    metric_value: MetricValue::Text("84%".to_string()),
                                    change_note: Some("of 500 credits this month".to_string()),
                                    lower_is_better: true,
                                }
                            }
                            div { class: "lh-grid-main",
                                DecisionReviewQueue {
                                    review_items: review_items(),
                                    board_title: "Waiting for you",
                                    current_time: now_secs(),
                                    reject_reasons: reasons,
                                    on_decide: move |e: DecideEvent| {
                                        activity.with_mut(|a| a.insert(0, format!("Decided on {}", e.item.item_title)));
                                    },
                                    on_skip: move |_| nav.set("skipped a review".to_string()),
                                }
                                div { class: "lh-stack",
                                    section { class: "lh-card", "aria-label": "Running now",
                                        h3 { class: "lh-h", "Running now" }
                                        for job in job_now.iter() {
                                            div { class: "lh-job", key: "{job.job_id}",
                                                div { class: "lh-between",
                                                    span { class: "lh-title", "{job.job_title}" }
                                                    span { class: "lh-chip", "data-tone": if job.job_state == JobState::NeedsYou { "warn" } else { "" },
                                                        {job_state_label(job.job_state)}
                                                    }
                                                }
                                                p { class: "lh-muted", "{job.current_step}" }
                                                div { class: "lh-bar",
                                                    div { class: "lh-bar-fill", style: "width: {percent(job.steps_done, job.steps_total)}%;" }
                                                }
                                            }
                                        }
                                    }
                                    section { class: "lh-card", "aria-label": "Recent activity",
                                        h3 { class: "lh-h", "Recent activity" }
                                        ul { class: "lh-list",
                                            for line in lines.iter() {
                                                li { key: "{line}", "{line}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }
            p { class: "gl-out", "data-out": "work-board", "Last navigation: {nav()}" }
        }
    }
}

fn job_state_label(state: JobState) -> &'static str {
    match state {
        JobState::Running => "running",
        JobState::Paused => "paused",
        JobState::NeedsYou => "needs you",
        JobState::Done => "done",
    }
}
