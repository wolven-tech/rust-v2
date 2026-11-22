use dioxus::prelude::*;
use rv2_kit::components::background_jobs_popover::{BackgroundJobsPopover, Edge, JobState};
use rv2_kit::components::transient_notice_stack::{
    NoticeHost, NoticeOptions, NoticeVariant, StackEdge, use_notifier,
};

use super::Scene;
use super::data::running_jobs;

#[component]
fn NoticeButtons() -> Element {
    let notifier = use_notifier();
    rsx! {
        div { class: "lh-row",
            button { class: "lh-btn", r#type: "button",
                onclick: move |_| {
                    notifier.notify_with("Job ready for review", NoticeOptions {
                        detail_text: Some("Weekly customer newsletter, section 4 is done".into()),
                        variant: NoticeVariant::Info,
                        ..Default::default()
                    });
                },
                "Job ready for review"
            }
            button { class: "lh-btn", r#type: "button",
                onclick: move |_| {
                    notifier.notify_with("Saved to Stored outputs", NoticeOptions {
                        detail_text: Some("Monday pipeline summary, run 12".into()),
                        variant: NoticeVariant::Success,
                        ..Default::default()
                    });
                },
                "Saved to outputs"
            }
            button { class: "lh-btn", r#type: "button",
                onclick: move |_| {
                    notifier.notify_with("Provider timed out", NoticeOptions {
                        detail_text: Some("Refund reply moved to the offline model".into()),
                        variant: NoticeVariant::Error,
                        ..Default::default()
                    });
                },
                "Provider failed"
            }
        }
    }
}

#[component]
pub fn TrayAndNotices() -> Element {
    let mut jobs = use_signal(running_jobs);
    let mut last = use_signal(|| "no job action yet".to_string());
    let job_now = jobs();
    let waiting = job_now
        .iter()
        .filter(|j| j.job_state == JobState::NeedsYou)
        .count();

    rsx! {
        Scene {
            id: "tray",
            title: "Tray and notices",
            story: "Sam keeps Longhand in the background. The tray icon shows what is running and what needs him, and short notices say when a job is ready without taking over the screen.",
            div { class: "lh-box", style: "height: 440px; position: relative;",
                div { class: "lh-between", style: "padding: 16px;",
                    div { class: "lh-stack", style: "gap: 4px;",
                        span { class: "lh-title", "Bramble Studio" }
                        span { class: "lh-muted", "{waiting} job waiting for you, {job_now.len()} in the background" }
                    }
                    BackgroundJobsPopover {
                        job_list: job_now.clone(),
                        default_open: true,
                        align_edge: Edge::Start,
                        on_pause: move |id: String| {
                            jobs.with_mut(|list| list.iter_mut().filter(|j| j.job_id == id).for_each(|j| j.job_state = JobState::Paused));
                            last.set(format!("paused {id}"));
                        },
                        on_resume: move |id: String| {
                            jobs.with_mut(|list| list.iter_mut().filter(|j| j.job_id == id).for_each(|j| j.job_state = JobState::Running));
                            last.set(format!("resumed {id}"));
                        },
                        on_open_job: move |id: String| last.set(format!("opened {id}")),
                        on_open_all: move |_| last.set("opened all jobs".to_string()),
                    }
                }
                NoticeHost { stack_edge: StackEdge::BottomRight, inside_parent: true, region_label: "Notices",
                    div { style: "padding: 16px; display: grid; gap: 12px; justify-items: start;",
                        p { class: "lh-muted", "Fire a notice to see how it lands in the corner." }
                        NoticeButtons {}
                    }
                }
            }
            p { class: "gl-out", "data-out": "tray", "{last()}" }
        }
    }
}
