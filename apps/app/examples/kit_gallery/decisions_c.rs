use dioxus::prelude::*;

use super::{Item, NOW_MS};

#[component]
pub fn DecisionReviewQueueDemo() -> Element {
    use rv2_kit::components::decision_review_queue::{
        BodyFormat, DecideEvent, DecisionReviewQueue, FileChange, FlagNote, PlanStep, ReviewItem,
        ReviewRisk,
    };
    let mut last = use_signal(String::new);
    let item = |id: &str, title: &str, risk: ReviewRisk, conf: f64| ReviewItem {
        item_id: id.into(),
        item_title: title.into(),
        agent_name: "Release agent".into(),
        model_name: Some("model A".into()),
        item_kind: "Edit".into(),
        risk_level: risk,
        confidence_score: conf,
        created_at: NOW_MS as i64 / 1000 - 300,
        agent_reasoning: Some("The changelog entry matches the merged PRs.".into()),
        body_heading: Some("CHANGELOG.md".into()),
        body_text: Some("## 1.4.0\n- Adds the tactile kit".into()),
        body_format: BodyFormat::Code,
        flags: vec![FlagNote {
            phrase: "tactile".into(),
            note: "Not in the release notes".into(),
        }],
        file_changes: vec![FileChange {
            path: "CHANGELOG.md".into(),
            lines_added: Some(3),
            lines_removed: Some(1),
            diff_text: None,
        }],
        plan_steps: vec![PlanStep {
            step_title: "Edit changelog".into(),
            tool_name: Some("Files".into()),
            step_detail: None,
            flagged: false,
        }],
        cited_sources: vec!["PR #412".into()],
    };
    let items = vec![
        item("i1", "Changelog entry", ReviewRisk::Low, 0.92),
        item("i2", "Delete stale branch", ReviewRisk::High, 0.55),
        item("i3", "Rename release tag", ReviewRisk::Medium, 0.7),
    ];
    rsx! {
        Item { slug: "decision-review-queue", name: "DecisionReviewQueue",
            DecisionReviewQueue {
                review_items: items,
                board_title: "Review queue",
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
    let item = |id: &str, summary: &str, kind: RequestKind| DecisionItem {
        id: id.into(),
        requester: "Ops agent".into(),
        requester_mark: None,
        request_kind: kind,
        request_summary: summary.into(),
        request_reason: "Needed to finish the migration".into(),
        request_status: RequestStatus::Open,
        requested_at: Some(NOW_MS as i64 / 1000 - 120),
        request_payload: Some("rm -rf build/".into()),
        change_lines: vec![],
        touched_paths: vec!["build/".into()],
        prior_steps: vec!["Compiled".into()],
        waiting_steps: vec!["Deploy".into()],
        person_note: None,
    };
    let items = vec![
        item("d1", "Delete build output", RequestKind::Tool),
        item("d2", "Message the vendor", RequestKind::Message),
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
            text: "The launch moved to June.".into(),
            cited_ids: vec!["r1".into()],
            starts_paragraph: true,
        },
        Segment {
            text: "Pricing stays the same.".into(),
            cited_ids: vec!["r2".into(), "r1".into()],
            starts_paragraph: false,
        },
    ];
    let refs = vec![
        Reference {
            ref_id: "r1".into(),
            headline: "Launch schedule update".into(),
            origin_site: "example.com".into(),
            link_target: None,
            published_on: Some("2026-10-02".into()),
            passage: Some("We now target June.".into()),
            exact_phrase: Some("target June".into()),
            leading_mark: None,
        },
        Reference {
            ref_id: "r2".into(),
            headline: "Pricing FAQ".into(),
            origin_site: "example.org".into(),
            link_target: None,
            published_on: None,
            passage: None,
            exact_phrase: None,
            leading_mark: None,
        },
    ];
    rsx! {
        Item { slug: "source-linked-answer", name: "SourceLinkedAnswer",
            SourceLinkedAnswer {
                answer_segments: segs,
                reference_list: refs,
                pages_consulted: 6,
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
fn NoticeTrigger(label: &'static str) -> Element {
    use rv2_kit::components::transient_notice_stack::{NoticeOptions, NoticeVariant, use_notifier};
    let notifier = use_notifier();
    rsx! {
        button {
            r#type: "button",
            onclick: move |_| {
                notifier.notify_with(label, NoticeOptions { detail_text: Some("Saved to drafts".into()), variant: NoticeVariant::Success, ..Default::default() });
            },
            "Fire notice"
        }
    }
}

#[component]
pub fn TransientNoticeStackDemo() -> Element {
    use rv2_kit::components::transient_notice_stack::{NoticeHost, StackEdge};
    rsx! {
        Item { slug: "transient-notice-stack", name: "TransientNoticeStack",
            div { class: "gl-canvas", style: "min-height: 320px;",
                NoticeHost { stack_edge: StackEdge::BottomRight, inside_parent: true, region_label: "Notices",
                    NoticeTrigger { label: "Draft saved" }
                }
            }
        }
    }
}

#[component]
pub fn WorkingTrailPanelDemo() -> Element {
    use rv2_kit::components::working_trail_panel::{TrailSource, TrailStep, WorkingTrailPanel};
    let mut last = use_signal(String::new);
    let steps = vec![
        TrailStep::Thinking {
            text: "The question is about pricing history.".into(),
        },
        TrailStep::Search {
            query_label: "pricing 2025".into(),
            sources: vec![TrailSource {
                source_label: "Pricing page".into(),
                source_href: None,
                source_mark: None,
            }],
            hidden_source_count: 2,
        },
        TrailStep::Thinking {
            text: "Comparing the two versions.".into(),
        },
    ];
    rsx! {
        Item { slug: "working-trail-panel", name: "WorkingTrailPanel",
            WorkingTrailPanel {
                trail_steps: steps,
                step_pause: 400,
                fold_when_done: true,
                on_finished: move |secs: f64| last.set(format!("finished {secs:.1}s")),
            }
            div { class: "gl-out", "data-out": "working-trail-panel", "{last()}" }
        }
    }
}

#[component]
pub fn QuickActionPaletteDemo() -> Element {
    use rv2_kit::components::quick_action_palette::{Command, CommandGroup, QuickActionPalette};
    let mut last = use_signal(String::new);
    let cmd = |id: &str, name: &str, combo: Option<&str>| Command {
        command_id: id.into(),
        command_name: name.into(),
        command_note: None,
        command_mark: None,
        key_combo: combo.map(|c| c.to_string()),
        match_terms: vec![name.to_lowercase()],
    };
    let groups = vec![
        CommandGroup {
            group_id: "nav".into(),
            group_name: "Go to".into(),
            commands: vec![
                cmd("inbox", "Inbox", Some("g i")),
                cmd("runs", "Runs", None),
            ],
        },
        CommandGroup {
            group_id: "act".into(),
            group_name: "Actions".into(),
            commands: vec![
                cmd("new", "New request", Some("n")),
                cmd("export", "Export log", None),
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
    let mut links = use_signal(Vec::<String>::new);
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
                    Model { model_id: "a".into(), model_name: "Model A".into(), model_mark: None, model_note: Some("Fast".into()) },
                    Model { model_id: "b".into(), model_name: "Model B".into(), model_mark: None, model_note: None },
                ],
                folder_options: vec!["notes".to_string(), "invoices".to_string()],
                selected_folder: Some("notes".to_string()),
                service_options: vec![Service { service_id: "mail".into(), service_name: "Mail".into(), service_mark: None }],
                voice_sample_text: "Summarise the notes folder",
                show_context_row: true,
                on_submit: move |s: Submission| last.set(format!("submit {}", s.text)),
                on_attach_pick: move |a: AttachAction| last.set(format!("attach {a:?}")),
            }
            div { class: "gl-out", "data-out": "prompt-input-bar", "{last()}" }
        }
    }
}
