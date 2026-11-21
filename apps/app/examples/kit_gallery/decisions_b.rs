use dioxus::prelude::*;

use super::Item;

#[component]
pub fn DeliberationDigestDemo() -> Element {
    use rv2_kit::components::deliberation_digest::{
        AgentIdentity, DeliberationDigest, DigestModel, DigestSource, DigestStep, Effort, Emphasis,
        KindSpec, TokenCounts,
    };
    use std::collections::BTreeMap;
    let step = |id: &str, kind: &str, label: &str, running: bool| DigestStep {
        step_id: id.into(),
        kind_key: kind.into(),
        step_label: label.into(),
        elapsed_time: (!running).then_some(1.8),
        is_running: running,
        model_id: Some("m1".into()),
    };
    let model = DigestModel {
        model_id: "m1".into(),
        display_name: "Model A".into(),
        logo: None,
        role_label: "Planner".into(),
        effort_level: Effort::High,
        time_taken: 4.2,
        provider_name: "Provider".into(),
        token_counts: Some(TokenCounts {
            input: 12_400,
            output: 1_900,
        }),
        spend: Some(0.07),
        selection_note: Some("Chosen for long context".into()),
    };
    let sources = vec![
        DigestSource {
            source_name: "Pricing page".into(),
            source_detail: Some("2024".into()),
            source_href: None,
            logo: None,
        },
        DigestSource {
            source_name: "Changelog".into(),
            source_detail: None,
            source_href: None,
            logo: None,
        },
    ];
    let kinds = BTreeMap::from([
        (
            "search".to_string(),
            KindSpec {
                kind_label: "Search".into(),
                kind_emphasis: Emphasis::Normal,
                kind_mark: None,
            },
        ),
        (
            "reason".to_string(),
            KindSpec {
                kind_label: "Reasoning".into(),
                kind_emphasis: Emphasis::Strong,
                kind_mark: None,
            },
        ),
    ]);
    rsx! {
        Item { slug: "deliberation-digest", name: "DeliberationDigest",
            DeliberationDigest {
                step_list: vec![
                    step("s1", "search", "Searched the pricing page", false),
                    DigestStep {
                        step_id: "s2".into(),
                        kind_key: "search".into(),
                        step_label: "Read the changelog".into(),
                        elapsed_time: Some(3.4),
                        is_running: false,
                        model_id: Some("m1".into()),
                    },
                    step("s3", "reason", "Comparing tiers", true),
                ],
                running_started_ms: Some(1_700_000_000_000),
                clock_ms: Some(1_700_000_004_200),
                model_list: vec![model],
                source_list: sources,
                total_source_count: Some(7),
                is_reasoning: true,
                expected_time: Some(12.0),
                kind_table: kinds,
                agent_identity: Some(AgentIdentity { agent_id: "planner".into(), agent_emphasis: Emphasis::Normal }),
                start_open: true,
            }
        }
    }
}

#[component]
pub fn ExpandableWorkItemCardDemo() -> Element {
    use rv2_kit::components::expandable_work_item_card::{
        ExpandableWorkItemCard, Member, OpenOn, PrimaryAction, StatusOption, Subtask, Tone,
    };
    let mut last = use_signal(String::new);
    let mut subtasks = use_signal(|| {
        vec![
            Subtask {
                subtask_id: "t1".into(),
                subtask_text: "Draft the agenda".into(),
                is_done: true,
                assignee_name: Some("Ada".into()),
            },
            Subtask {
                subtask_id: "t2".into(),
                subtask_text: "Book the room".into(),
                is_done: false,
                assignee_name: None,
            },
        ]
    });
    rsx! {
        Item { slug: "expandable-work-item-card", name: "ExpandableWorkItemCard",
            ExpandableWorkItemCard {
                item_title: "Plan the offsite",
                subtask_list: subtasks(),
                on_subtasks_update: move |list: Vec<Subtask>| {
                    last.set(format!("subtasks {}", list.iter().filter(|t| t.is_done).count()));
                    subtasks.set(list);
                },
                status_id: "todo",
                status_options: vec![
                    StatusOption { option_id: "todo".into(), option_label: "To do".into(), emphasis: Tone::Neutral, option_mark: None },
                    StatusOption { option_id: "doing".into(), option_label: "In progress".into(), emphasis: Tone::Caution, option_mark: None },
                    StatusOption { option_id: "done".into(), option_label: "Done".into(), emphasis: Tone::Positive, option_mark: None },
                ],
                on_status_pick: move |id: String| last.set(format!("status {id}")),
                due_text: Some("Fri".to_string()),
                member_list: vec![Member { member_name: "Ada".into(), avatar: None }, Member { member_name: "Ben".into(), avatar: None }],
                primary_action: Some(PrimaryAction { action_label: "Open thread".into(), on_activate: EventHandler::new(move |_| last.set("primary".into())) }),
                footer_hint: Some("Press Enter to toggle".to_string()),
                open_on: OpenOn::Activate,
                default_open: true,
                on_open_change: move |o: bool| last.set(format!("open {o}")),
            }
            div { class: "gl-out", "data-out": "expandable-work-item-card", "{last()}" }
        }
    }
}

#[component]
pub fn GroundedReplyDemo() -> Element {
    use rv2_kit::components::grounded_reply::{GroundedReply, SourceEntry, VoteEvent};
    let mut last = use_signal(String::new);
    let sources = vec![
        SourceEntry {
            entry_id: "s1".into(),
            chip_text: "Q2 report".into(),
            link_target: None,
            leading_mark: None,
            snippet: Some("Revenue grew 12 percent.".into()),
        },
        SourceEntry {
            entry_id: "s2".into(),
            chip_text: "Churn sheet".into(),
            link_target: None,
            leading_mark: None,
            snippet: Some("Churn fell to 3.1 percent.".into()),
        },
    ];
    rsx! {
        Item { slug: "grounded-reply", name: "GroundedReply",
            GroundedReply {
                heading_text: "Answer",
                source_entries: sources,
                answer_text: "Revenue grew 12 percent [1] while churn fell to 3.1 percent [2].",
                next_questions: vec!["What drove the growth?".to_string()],
                vote_down_reasons: vec!["Wrong facts".to_string(), "Too vague".to_string()],
                on_vote: move |v: VoteEvent| last.set(format!("vote {:?} {:?}", v.vote, v.reason)),
                on_next_question_chosen: move |q: String| last.set(format!("next {q}")),
                on_copy_requested: move |t: String| last.set(format!("copy {} chars", t.len())),
                on_retry_requested: move |_| last.set("retry".into()),
            }
            div { class: "gl-out", "data-out": "grounded-reply", "{last()}" }
        }
    }
}

#[component]
pub fn InlineDraftingAssistantDemo() -> Element {
    use rv2_kit::components::inline_drafting_assistant::{
        InlineDraftingAssistant, ReplyResult, SpeechMode,
    };
    let mut reply = use_signal(|| None::<ReplyResult>);
    let mut last = use_signal(String::new);
    rsx! {
        Item { slug: "inline-drafting-assistant", name: "InlineDraftingAssistant",
            InlineDraftingAssistant {
                assistant_name: "Quill",
                assistant_tagline: "Drafts and edits in place",
                greeting_text: "What should we write?",
                prompt_chips: vec!["Shorter".to_string(), "More formal".to_string()],
                input_hint: "Describe the change",
                reply: reply(),
                speech_mode: SpeechMode::Simulate,
                sample_dictation: "Make the intro shorter",
                on_message: move |m: String| {
                    last.set(format!("sent {m}"));
                    reply.set(Some(ReplyResult::Ok(format!("Draft for: {m}"))));
                },
            }
            div { class: "gl-out", "data-out": "inline-drafting-assistant", "{last()}" }
        }
    }
}

#[component]
pub fn InlineEditReviewDemo() -> Element {
    use rv2_kit::components::inline_edit_review::{InlineEditReview, Segment};
    let mut last = use_signal(String::new);
    rsx! {
        Item { slug: "inline-edit-review", name: "InlineEditReview",
            InlineEditReview {
                before_text: "The team will ship the update next week.",
                after_text: "The team ships the update on Tuesday.",
                segments: vec![
                    Segment::Same("The team ".to_string()),
                    Segment::Change { removed: "will ship the update next week".to_string(), inserted: "ships the update on Tuesday".to_string() },
                    Segment::Same(".".to_string()),
                ],
                caption_text: "Suggested rewrite",
                on_complete: move |t: String| last.set(format!("accepted {t}")),
                on_text_change: move |t: String| last.set(format!("edited {} chars", t.len())),
            }
            div { class: "gl-out", "data-out": "inline-edit-review", "{last()}" }
        }
    }
}

#[component]
pub fn InterAgentMessageFeedDemo() -> Element {
    use rv2_kit::components::inter_agent_message_feed::{
        Attachment, AttachmentKind, ComposeMessage, DetailCard, Entry, EntryKind,
        InterAgentMessageFeed, Participant,
    };
    let mut last = use_signal(String::new);
    let mut picked = use_signal(|| None::<String>);
    let p = |id: &str, name: &str| Participant {
        participant_id: id.into(),
        display_name: name.into(),
        model_name: Some("model A".into()),
        badge_mark: None,
        logo: None,
    };
    let entry = |id: &str, from: &str, to: &str, kind: EntryKind, body: &str| Entry {
        entry_id: id.into(),
        sender_id: from.into(),
        recipient_ids: vec![to.into()],
        entry_kind: kind,
        body_text: body.into(),
        sent_at: Some(match id {
            "e1" => 12,
            "e2" => 40,
            _ => 75,
        }),
        answers_id: None,
        is_drafting: false,
        attachment_list: vec![],
        detail_card: None,
    };
    let mut task = entry(
        "e1",
        "lead",
        "research",
        EntryKind::TaskRequest,
        "Find three pricing comparables.",
    );
    task.attachment_list = vec![Attachment {
        label: "pricing-notes.md".into(),
        kind: Some(AttachmentKind::File),
        preview: None,
        meta: Some("3 KB".into()),
    }];
    let mut answer = entry(
        "e2",
        "research",
        "lead",
        EntryKind::Answer,
        "Found three: A, B, C.",
    );
    answer.answers_id = Some("e1".into());
    answer.detail_card = Some(DetailCard {
        notes: Some("Sources checked".into()),
        rows: vec![("Sources".into(), "4".into())],
    });
    rsx! {
        Item { slug: "inter-agent-message-feed", name: "InterAgentMessageFeed",
            InterAgentMessageFeed {
                participants: vec![p("lead", "Lead"), p("research", "Research")],
                entries: vec![task, answer, entry("e3", "lead", "research", EntryKind::Broadcast, "Keep answers under 200 words.")],
                run_subject: Some("Pricing review".to_string()),
                is_running: true,
                picked_entry_id: picked(),
                on_pick_change: move |id: Option<String>| {
                    last.set(format!("pick {id:?}"));
                    picked.set(id);
                },
                on_compose: move |m: ComposeMessage| last.set(format!("compose {}", m.text)),
                compose_hint: "Message the agents",
            }
            div { class: "gl-out", "data-out": "inter-agent-message-feed", "{last()}" }
        }
    }
}
