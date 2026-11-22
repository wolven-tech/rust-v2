use dioxus::prelude::*;

use super::{Item, NOW_MS};

#[component]
pub fn DecisionReviewQueueDemo() -> Element {
    use rv2_kit::components::decision_review_queue::{
        BodyFormat, DecideEvent, DecisionReviewQueue, FileChange, FlagNote, PlanStep, ReviewItem,
        ReviewRisk,
    };
    let mut last = use_signal(String::new);
    let item = |id: &str,
                title: &str,
                agent: &str,
                kind: &str,
                risk: ReviewRisk,
                conf: f64,
                waited_secs: i64|
     -> ReviewItem {
        ReviewItem {
            item_id: id.into(),
            item_title: title.into(),
            agent_name: agent.into(),
            model_name: Some("model A".into()),
            item_kind: kind.into(),
            risk_level: risk,
            confidence_score: conf,
            created_at: NOW_MS as i64 / 1000 - waited_secs,
            agent_reasoning: Some(format!(
                "{title}: the proposed change matches the source record."
            )),
            body_heading: None,
            body_text: None,
            body_format: BodyFormat::Text,
            flags: vec![],
            file_changes: vec![],
            plan_steps: vec![],
            cited_sources: vec!["PR #412".into()],
        }
    };
    let mut changelog = item(
        "i1",
        "Update release notes",
        "Release agent",
        "Code change",
        ReviewRisk::High,
        0.92,
        3_000,
    );
    changelog.body_heading = Some("Release notes".into());
    changelog.body_text = Some("Adds the tactile kit to the 1.4.0 notes.".into());
    changelog.body_format = BodyFormat::Text;
    changelog.flags = vec![FlagNote {
        phrase: "tactile".into(),
        note: "Not in the release notes".into(),
    }];
    changelog.file_changes = vec![
        FileChange {
            path: "CHANGELOG.md".into(),
            lines_added: Some(3),
            lines_removed: Some(1),
            diff_text: Some(
                " ## 1.4.0\n-- Adds kit\n+- Adds the tactile kit\n+- Adds the readout kit\n+- Adds the overlay kit\n Fixes build"
                    .into(),
            ),
        },
        FileChange {
            path: "src/kit.rs".into(),
            lines_added: Some(5),
            lines_removed: Some(2),
            diff_text: Some(
                " pub mod tactile;\n-pub mod old_tactile;\n-pub mod legacy;\n+pub mod tactile_v2;\n+pub mod readout;\n+pub mod overlay;\n+pub mod catalogue;\n+pub mod tokens;\n pub mod grid;"
                    .into(),
            ),
        },
    ];
    let mut stale = item(
        "i2",
        "Clean up stale branches",
        "Release agent",
        "Plan",
        ReviewRisk::High,
        0.55,
        300,
    );
    stale.plan_steps = vec![
        PlanStep {
            step_title: "List branches merged before 1 Oct".into(),
            tool_name: Some("Git".into()),
            step_detail: None,
            flagged: false,
        },
        PlanStep {
            step_title: "Delete the 14 listed remote branches".into(),
            tool_name: Some("Git".into()),
            step_detail: None,
            flagged: true,
        },
    ];
    let mut clause = item(
        "i3",
        "Update renewal clause",
        "Legal agent",
        "Contract edit",
        ReviewRisk::Medium,
        0.72,
        540,
    );
    clause.body_text = Some("Renewal notice moves from 30 to 60 days.".into());
    let mut refund = item(
        "i4",
        "Refund duplicate charge",
        "Billing agent",
        "Refund",
        ReviewRisk::Medium,
        0.81,
        900,
    );
    refund.body_text = Some("Refund 49.00 on order 1182.".into());
    let mut message = item(
        "i5",
        "Reply to partner message",
        "Support agent",
        "Message",
        ReviewRisk::Low,
        0.64,
        1_500,
    );
    message.body_text = Some("Thanks, the June date works for us.".into());
    let mut access = item(
        "i6",
        "Grant calendar access",
        "Scheduling agent",
        "Access",
        ReviewRisk::Medium,
        0.48,
        2_400,
    );
    access.body_text = Some("Read and write access to the Team calendar.".into());
    let drafts = item(
        "i7",
        "Archive old drafts",
        "Docs agent",
        "Edit",
        ReviewRisk::Low,
        0.6,
        1_800,
    );
    let tag = item(
        "i8",
        "Rename release tag",
        "Release agent",
        "Edit",
        ReviewRisk::Low,
        0.9,
        60,
    );
    let items = vec![
        changelog, stale, clause, refund, message, access, drafts, tag,
    ];
    rsx! {
        Item { slug: "decision-review-queue", name: "DecisionReviewQueue",
            DecisionReviewQueue {
                review_items: items,
                board_title: "Review queue",
                reviewed_before: 24usize,
                reject_reasons: vec!["Off-policy".to_string(), "Wrong facts".to_string()],
                current_time: NOW_MS as i64 / 1000,
                on_decide: move |e: DecideEvent| last.set(format!("{:?} {}", e.result.outcome, e.item.item_id)),
                on_undo: move |e: DecideEvent| last.set(format!("undo {}", e.item.item_id)),
                on_skip: move |i: ReviewItem| last.set(format!("skip {}", i.item_id)),
            }
            div { class: "gl-out", "data-out": "decision-review-queue", "{last()}" }
        }
    }
}

#[component]
pub fn PendingDecisionStackDemo() -> Element {
    use rv2_kit::components::pending_decision_stack::{
        DecisionItem, PendingDecisionStack, RequestKind, RequestStatus, Resolution,
    };
    let mut last = use_signal(String::new);
    let item = |id: &str,
                requester: &str,
                mark: &str,
                summary: &str,
                kind: RequestKind,
                status: RequestStatus,
                waited_secs: i64| DecisionItem {
        id: id.into(),
        requester: requester.into(),
        requester_mark: Some(mark.into()),
        request_kind: kind,
        request_summary: summary.into(),
        request_reason: format!("{summary} is the next step in the migration."),
        request_status: status,
        requested_at: Some(NOW_MS as i64 / 1000 - waited_secs),
        request_payload: Some("rm -rf build/".into()),
        change_lines: vec![],
        touched_paths: vec!["build/".into()],
        prior_steps: vec!["Compiled".into()],
        waiting_steps: vec!["Deploy".into()],
        person_note: None,
    };
    let items = vec![
        item(
            "d1",
            "Ops agent",
            "OA",
            "Delete build output",
            RequestKind::Tool,
            RequestStatus::Open,
            120,
        ),
        item(
            "d2",
            "Sales agent",
            "SA",
            "Message the vendor",
            RequestKind::Message,
            RequestStatus::Open,
            480,
        ),
        item(
            "d3",
            "Finance agent",
            "FA",
            "Edit pricing sheet",
            RequestKind::File,
            RequestStatus::Open,
            900,
        ),
        item(
            "d4",
            "Release agent",
            "RA",
            "Run cleanup job",
            RequestKind::Agent,
            RequestStatus::Approved,
            1_500,
        ),
        item(
            "d5",
            "Docs agent",
            "DA",
            "Publish changelog",
            RequestKind::File,
            RequestStatus::Edited,
            2_100,
        ),
        item(
            "d6",
            "Mail agent",
            "MA",
            "Send test email",
            RequestKind::Message,
            RequestStatus::Rejected,
            2_700,
        ),
    ];
    rsx! {
        Item { slug: "pending-decision-stack", name: "PendingDecisionStack",
            PendingDecisionStack {
                decision_items: items,
                run_subject: "Migration run",
                is_running: true,
                on_resolve: move |r: Resolution| last.set(format!("{} {:?}", r.id, r.outcome)),
            }
            div { class: "gl-out", "data-out": "pending-decision-stack", "{last()}" }
        }
    }
}

#[component]
pub fn SourceLinkedAnswerDemo() -> Element {
    use rv2_kit::components::source_linked_answer::{Reference, Segment, SourceLinkedAnswer};
    let mut last = use_signal(String::new);
    let segs = vec![
        Segment {
            text: "The launch moved to June after the partner review.".into(),
            cited_ids: vec!["r1".into()],
            starts_paragraph: true,
        },
        Segment {
            text: "Beta invitations go out the week of June 9.".into(),
            cited_ids: vec!["r3".into()],
            starts_paragraph: false,
        },
        Segment {
            text: "Engineering has signed off on the June build.".into(),
            cited_ids: vec![],
            starts_paragraph: false,
        },
        Segment {
            text: "Pricing stays the same for the Team plan.".into(),
            cited_ids: vec!["r2".into(), "r1".into()],
            starts_paragraph: true,
        },
        Segment {
            text: "The Starter tier gained two seats at no extra cost.".into(),
            cited_ids: vec!["r4".into()],
            starts_paragraph: false,
        },
        Segment {
            text: "Support hours are unchanged for every plan.".into(),
            cited_ids: vec![],
            starts_paragraph: false,
        },
    ];
    let refs = vec![
        Reference {
            ref_id: "r1".into(),
            headline: "Launch schedule update".into(),
            origin_site: "example.com".into(),
            link_target: None,
            published_on: Some("2 Oct 2026".into()),
            passage: Some("We now target June, pending the partner review.".into()),
            exact_phrase: None,
            leading_mark: Some("L".into()),
        },
        Reference {
            ref_id: "r2".into(),
            headline: "Pricing FAQ".into(),
            origin_site: "example.org".into(),
            link_target: None,
            published_on: None,
            passage: None,
            exact_phrase: None,
            leading_mark: Some("P".into()),
        },
        Reference {
            ref_id: "r3".into(),
            headline: "Beta program notes".into(),
            origin_site: "docs.example.net".into(),
            link_target: None,
            published_on: Some("20 Oct 2026".into()),
            passage: Some("Invitations go out in the week of June 9.".into()),
            exact_phrase: Some("week of June 9".into()),
            leading_mark: Some("B".into()),
        },
        Reference {
            ref_id: "r4".into(),
            headline: "Plan changes 2026".into(),
            origin_site: "example.org".into(),
            link_target: None,
            published_on: Some("28 Sep 2026".into()),
            passage: Some("The Starter plan gained two seats at no extra cost.".into()),
            exact_phrase: None,
            leading_mark: Some("C".into()),
        },
    ];
    rsx! {
        Item { slug: "source-linked-answer", name: "SourceLinkedAnswer",
            SourceLinkedAnswer {
                answer_segments: segs,
                reference_list: refs,
                pages_consulted: 6,
                start_open_ref: Some("r1".to_string()),
                trailing_action_text: Some("Open all sources".to_string()),
                on_trailing_action: move |_| last.set("open all".into()),
                on_open_request: move |r: Reference| last.set(format!("open {}", r.ref_id)),
                on_preview_change: move |id: Option<String>| last.set(format!("preview {id:?}")),
            }
            div { class: "gl-out", "data-out": "source-linked-answer", "{last()}" }
        }
    }
}

#[component]
fn NoticeTrigger(
    label: &'static str,
    title: &'static str,
    detail: &'static str,
    variant: rv2_kit::components::transient_notice_stack::NoticeVariant,
    undo: bool,
) -> Element {
    use rv2_kit::components::transient_notice_stack::{
        NoticeAction, NoticeOptions, NoticeVariant, use_notifier,
    };
    let notifier = use_notifier();
    rsx! {
        button {
            r#type: "button",
            onclick: move |_| {
                let action = if undo {
                    Some(NoticeAction { action_label: "Undo".into(), on_activate: Callback::new(|_| true) })
                } else if variant == NoticeVariant::Error {
                    Some(NoticeAction { action_label: "Retry upload".into(), on_activate: Callback::new(|_| false) })
                } else {
                    None
                };
                notifier.notify_with(title, NoticeOptions { detail_text: Some(detail.into()), variant, action, ..Default::default() });
            },
            "{label}"
        }
    }
}

#[component]
fn NoticeSeed() -> Element {
    use rv2_kit::components::transient_notice_stack::HoldTime;
    use rv2_kit::components::transient_notice_stack::{
        NoticeAction, NoticeOptions, NoticeVariant, use_notifier,
    };
    let notifier = use_notifier();
    let mut seeded = use_signal(|| false);
    let mut loading_id = use_signal(|| Option::<usize>::None);
    use_effect(move || {
        // Peek: the notifier reads signals while it works; a tracked read would seed again on each change.
        if *seeded.peek() {
            return;
        }
        seeded.set(true);
        // Oldest first: the loading message is the front card.
        notifier.notify_with(
            "Settings saved",
            NoticeOptions {
                detail_text: Some("Notifications are on for this workspace".into()),
                variant: NoticeVariant::Success,
                ..Default::default()
            },
        );
        notifier.notify_with(
            "Draft archived",
            NoticeOptions {
                detail_text: Some("Q3 pricing memo".into()),
                variant: NoticeVariant::Success,
                action: Some(NoticeAction {
                    action_label: "Undo".into(),
                    on_activate: Callback::new(|_| true),
                }),
                ..Default::default()
            },
        );
        let id = notifier.notify_with(
            "Exporting report",
            NoticeOptions {
                detail_text: Some("report-q3.xlsx".into()),
                variant: NoticeVariant::Loading,
                hold_time: Some(HoldTime::Infinite),
                ..Default::default()
            },
        );
        loading_id.set(Some(id));
    });
    rsx! {
        button {
            r#type: "button",
            onclick: move |_| {
                if let Some(id) = loading_id() {
                    notifier.notify_with(
                        "Export ready",
                        NoticeOptions {
                            message_id: Some(id),
                            detail_text: Some("report-q3.xlsx".into()),
                            variant: NoticeVariant::Success,
                            ..Default::default()
                        },
                    );
                }
            },
            "Resolve export"
        }
    }
}

#[component]
pub fn TransientNoticeStackDemo() -> Element {
    use rv2_kit::components::transient_notice_stack::{
        HoldTime, NoticeHost, NoticeVariant, StackEdge,
    };
    rsx! {
        Item { slug: "transient-notice-stack", name: "TransientNoticeStack",
            div { class: "gl-canvas", style: "min-height: 440px;",
                NoticeHost { stack_edge: StackEdge::BottomCenter, inside_parent: true, region_label: "Notices, press N to focus",
                    focus_key: Some("n".to_string()),
                    default_hold_time: HoldTime::Millis(60_000),
                    div {
                        style: "display: grid; gap: var(--kit-space-2); max-width: 360px; padding: var(--kit-space-3); border: 1px solid var(--kit-rule-soft); border-radius: var(--kit-radius-card); background: var(--kit-surface); color: var(--kit-ink); font-family: var(--kit-font);",
                        div { style: "font-weight: 600;", "report-q3.xlsx" }
                        div { style: "font-size: var(--kit-text-md); color: var(--kit-ink-muted);", "Q3 report · edited today" }
                        div { class: "gl-row",
                            NoticeTrigger { label: "Archive draft", title: "Draft archived", detail: "Q3 pricing memo", variant: NoticeVariant::Success, undo: true }
                            NoticeTrigger { label: "Save settings", title: "Settings saved", detail: "Notifications are on", variant: NoticeVariant::Success, undo: false }
                            NoticeTrigger { label: "Raise upload error", title: "Upload failed", detail: "report-q3.xlsx was not sent", variant: NoticeVariant::Error, undo: false }
                            NoticeSeed {}
                        }
                    }
                    p { style: "margin: 0; font-size: var(--kit-text-md); color: var(--kit-ink-muted);", "Press N to focus the newest notice." }
                }
            }
        }
    }
}

#[component]
pub fn WorkingTrailPanelDemo() -> Element {
    use rv2_kit::components::working_trail_panel::{TrailSource, TrailStep, WorkingTrailPanel};
    let mut last = use_signal(String::new);
    let mut run = use_signal(|| 0_u32);
    let mut finish = use_signal(|| false);
    let source = |label: &str, mark: &str| TrailSource {
        source_label: label.into(),
        source_href: None,
        source_mark: Some(mark.into()),
    };
    let steps = vec![
        TrailStep::Thinking {
            text: "The question is about pricing history across the last two plan changes.".into(),
        },
        TrailStep::Search {
            query_label: "pricing page changes 2025".into(),
            sources: vec![
                source("Pricing page", "P"),
                source("Changelog", "C"),
                source("Release notes", "R"),
            ],
            hidden_source_count: 1,
        },
        TrailStep::Thinking {
            text: "Comparing the Starter and Team tiers before and after March.".into(),
        },
        TrailStep::Thinking {
            text: "Summarising both changes with dates and sources.".into(),
        },
    ];
    rsx! {
        Item { slug: "working-trail-panel", name: "WorkingTrailPanel",
            WorkingTrailPanel {
                key: "{run()}",
                trail_steps: steps.clone(),
                step_pause: 600,
                auto_finish: false,
                finish_requested: finish(),
                on_finished: move |secs: f64| last.set(format!("finished {secs:.1}s")),
            }
            div { class: "gl-row",
                button {
                    r#type: "button",
                    class: "gl-btn",
                    onclick: move |_| {
                        finish.set(true);
                    },
                    "Finish"
                }
                button {
                    r#type: "button",
                    class: "gl-btn",
                    onclick: move |_| {
                        run.set(run() + 1);
                        finish.set(false);
                        last.set("restarted".into());
                    },
                    "Restart"
                }
            }
            div { class: "gl-out", "data-out": "working-trail-panel", "{last()}" }
        }
    }
}

#[component]
pub fn QuickActionPaletteDemo() -> Element {
    use rv2_kit::components::quick_action_palette::{Command, CommandGroup, QuickActionPalette};
    let mut last = use_signal(String::new);
    let cmd = |id: &str, name: &str, mark: Option<&str>, note: &str, combo: Option<&str>| Command {
        command_id: id.into(),
        command_name: name.into(),
        command_note: Some(note.into()),
        command_mark: mark.map(|m| rsx! { "{m}" }),
        key_combo: combo.map(|c| c.to_string()),
        match_terms: vec![name.to_lowercase()],
    };
    let groups = vec![
        CommandGroup {
            group_id: "suggested".into(),
            group_name: "Suggested".into(),
            commands: vec![
                cmd("sync", "Sync sources", Some("S"), "2 new", Some("Mod 1")),
                cmd(
                    "digest",
                    "Morning digest",
                    Some("D"),
                    "Today",
                    Some("Mod 2"),
                ),
                cmd(
                    "ledger",
                    "Ledger export",
                    Some("L"),
                    "Monthly",
                    Some("Mod 3"),
                ),
                cmd("plan", "Plan the week", Some("C"), "Mon", Some("Mod 4")),
                cmd("reply", "Draft reply", Some("M"), "Inbox", Some("Mod 5")),
            ],
        },
        CommandGroup {
            group_id: "nav".into(),
            group_name: "Go to".into(),
            commands: vec![
                cmd("inbox", "Inbox", None, "6 waiting on you", Some("g i")),
                cmd("runs", "Runs", None, "3 running", Some("g r")),
                cmd("billing", "Billing", None, "Renews 1 Dec", Some("g b")),
            ],
        },
        CommandGroup {
            group_id: "act".into(),
            group_name: "Actions".into(),
            commands: vec![
                cmd("new", "New request", None, "Start a run", Some("n")),
                cmd("export", "Export log", None, "Last 30 days", None),
            ],
        },
    ];
    rsx! {
        Item { slug: "quick-action-palette", name: "QuickActionPalette",
            QuickActionPalette {
                command_groups: groups,
                focus_on_mount: false,
                on_command: move |(id, arg): (String, Option<String>)| last.set(format!("command {id} {arg:?}")),
                on_dismiss: move |_| last.set("dismiss".into()),
            }
            div { class: "gl-out", "data-out": "quick-action-palette", "{last()}" }
        }
    }
}

#[component]
pub fn PromptInputBarDemo() -> Element {
    use rv2_kit::components::prompt_input_bar::{
        AttachAction, Model, PromptInputBar, Service, Submission,
    };
    let mut last = use_signal(String::new);
    let mut links =
        use_signal(|| vec!["drive".to_string(), "notion".to_string(), "cal".to_string()]);
    rsx! {
        Item { slug: "prompt-input-bar", name: "PromptInputBar",
            PromptInputBar {
                on_model_pick: move |m: Model| last.set(format!("model {}", m.model_id)),
                on_links_change: move |ids: Vec<String>| {
                    last.set(format!("links {}", ids.len()));
                    links.set(ids);
                },
                linked_service_ids: links(),
                model_options: vec![
                    Model { model_id: "a".into(), model_name: "Model A".into(), model_mark: Some("A".into()), model_note: Some("Fast".into()) },
                    Model { model_id: "b".into(), model_name: "Model B".into(), model_mark: Some("B".into()), model_note: Some("Deep reasoning".into()) },
                ],
                folder_options: vec![
                    "~/Projects/notes".to_string(),
                    "~/Projects/invoices".to_string(),
                    "~/Downloads".to_string(),
                ],
                selected_folder: Some("~/Projects/notes".to_string()),
                service_options: vec![
                    Service { service_id: "drive".into(), service_name: "Drive".into(), service_mark: Some("D".into()) },
                    Service { service_id: "notion".into(), service_name: "Notion".into(), service_mark: Some("N".into()) },
                    Service { service_id: "cal".into(), service_name: "Calendar".into(), service_mark: Some("C".into()) },
                    Service { service_id: "mail".into(), service_name: "Mail".into(), service_mark: Some("M".into()) },
                ],
                voice_sample_text: "Summarise the notes folder",
                show_context_row: true,
                on_submit: move |s: Submission| last.set(format!("submit {}", s.text)),
                on_attach_pick: move |a: AttachAction| last.set(format!("attach {a:?}")),
            }
            div { class: "gl-out", "data-out": "prompt-input-bar", "{last()}" }
        }
    }
}
