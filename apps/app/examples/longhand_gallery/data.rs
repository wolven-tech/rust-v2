//! Shared Longhand fixtures: the signed-in user, the sidebar, and the review queue.

use rv2_kit::components::background_jobs_popover::{BackgroundJob, JobState};
use rv2_kit::components::decision_review_queue::{
    BodyFormat, FlagNote, PlanStep, ReviewItem, ReviewRisk,
};
use rv2_kit::screens::workspace_frame_inset::{
    ChatRow, FrameAccount, MenuGroup, MenuIcon, MenuItem,
};

use super::NOW_MS;

pub fn now_secs() -> i64 {
    NOW_MS as i64 / 1000
}

pub fn percent(done: u32, total: u32) -> u32 {
    done * 100 / total.max(1)
}

pub fn account() -> FrameAccount {
    FrameAccount {
        name: "Sam Okafor".into(),
        avatar_url: None,
        email: Some("sam@example.com".into()),
    }
}

fn item(label: &str, icon: MenuIcon) -> MenuItem {
    MenuItem {
        item_label: label.into(),
        item_icon: icon,
    }
}

pub fn menu_items() -> Vec<MenuItem> {
    vec![
        item("My Work Board", MenuIcon::Dot),
        item("Mission Control", MenuIcon::Dot),
        item("New goal", MenuIcon::Chat),
    ]
}

pub fn menu_groups() -> Vec<MenuGroup> {
    vec![
        MenuGroup {
            title: "Build".into(),
            items: vec![
                item("Skills", MenuIcon::Folder),
                item("Workflows", MenuIcon::Folder),
                item("Goals", MenuIcon::Folder),
            ],
        },
        MenuGroup {
            title: "Connections".into(),
            items: vec![
                item("Models and tools", MenuIcon::Plug),
                item("Stored outputs", MenuIcon::Folder),
            ],
        },
    ]
}

pub fn chats() -> Vec<ChatRow> {
    vec![
        ChatRow {
            id: "c1".into(),
            title: "Juniper renewal prep".into(),
            pinned: true,
            unread: false,
            project_id: None,
            section: None,
        },
        ChatRow {
            id: "c2".into(),
            title: "Newsletter ideas".into(),
            pinned: false,
            unread: true,
            project_id: Some("Clients".into()),
            section: None,
        },
    ]
}

pub fn running_jobs() -> Vec<BackgroundJob> {
    vec![
        job(
            "job-news",
            "Weekly customer newsletter",
            "Drafting section 2",
            2,
            4,
            JobState::NeedsYou,
        ),
        job(
            "job-pipeline",
            "Monday pipeline review",
            "Pulling deals from the CRM",
            3,
            6,
            JobState::Running,
        ),
        job(
            "job-summary",
            "Summarise sales call, Juniper",
            "Writing the three bullets",
            4,
            5,
            JobState::Running,
        ),
        job(
            "job-backup",
            "Store weekly outputs",
            "Paused by you",
            1,
            3,
            JobState::Paused,
        ),
    ]
}

fn job(id: &str, title: &str, step: &str, done: u32, total: u32, state: JobState) -> BackgroundJob {
    BackgroundJob {
        job_id: id.into(),
        job_title: title.into(),
        current_step: step.into(),
        steps_done: done,
        steps_total: total,
        job_state: state,
    }
}

fn review(
    id: &str,
    title: &str,
    kind: &str,
    risk: ReviewRisk,
    confidence: f64,
    age_secs: i64,
    body: &str,
) -> ReviewItem {
    ReviewItem {
        item_id: id.into(),
        item_title: title.into(),
        agent_name: "Longhand workflow".into(),
        model_name: Some("Balanced tier".into()),
        item_kind: kind.into(),
        risk_level: risk,
        confidence_score: confidence,
        created_at: now_secs() - age_secs,
        agent_reasoning: None,
        body_heading: None,
        body_text: Some(body.into()),
        body_format: BodyFormat::Text,
        flags: vec![],
        file_changes: vec![],
        plan_steps: vec![],
        cited_sources: vec![],
    }
}

pub fn review_items() -> Vec<ReviewItem> {
    vec![
        newsletter_item(),
        review(
            "job-refund",
            "Refund reply to Juniper Labs",
            "Reply",
            ReviewRisk::High,
            0.55,
            300,
            "Hi Priya, the refund for seat 4 went out on Friday and should land within five days.",
        ),
        review(
            "job-invoices",
            "Invoice reminders, 6 clients",
            "Batch",
            ReviewRisk::Low,
            0.93,
            1_500,
            "Friendly reminder that invoice 1042 is due on the 28th. Thanks for your business.",
        ),
    ]
}

pub fn newsletter_item() -> ReviewItem {
    ReviewItem {
        item_id: "job-newsletter".into(),
        item_title: "Weekly customer newsletter".into(),
        agent_name: "Newsletter workflow".into(),
        model_name: Some("Balanced tier".into()),
        item_kind: "Draft".into(),
        risk_level: ReviewRisk::Medium,
        confidence_score: 0.78,
        created_at: now_secs() - 900,
        agent_reasoning: Some(
            "Built from this week's support threads and the release notes.".into(),
        ),
        body_heading: Some("Newsletter, week 47".into()),
        body_text: Some(
            "Hi friends, this week the Juniper team got the new export reports. We will ship the update next week. Thanks for reading."
                .into(),
        ),
        body_format: BodyFormat::Text,
        flags: vec![FlagNote {
            phrase: "next week".into(),
            note: "The release notes say Tuesday".into(),
        }],
        file_changes: vec![],
        plan_steps: vec![
            PlanStep {
                step_title: "Read support threads".into(),
                tool_name: Some("Mail".into()),
                step_detail: None,
                flagged: false,
            },
            PlanStep {
                step_title: "Check release notes".into(),
                tool_name: Some("Docs search".into()),
                step_detail: None,
                flagged: false,
            },
            PlanStep {
                step_title: "Draft the newsletter".into(),
                tool_name: None,
                step_detail: None,
                flagged: true,
            },
        ],
        cited_sources: vec!["Support thread 412".into(), "Release notes 4.2".into()],
    }
}
