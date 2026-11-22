use dioxus::prelude::*;

use super::Item;

#[component]
pub fn DeliberationDigestDemo() -> Element {
    use rv2_kit::components::deliberation_digest::{
        AgentIdentity, DeliberationDigest, DigestModel, DigestSource, DigestStep, Effort, Emphasis,
        KindSpec, TokenCounts,
    };
    use std::collections::BTreeMap;
    let step = |id: &str, kind: &str, label: &str, elapsed: Option<f64>, model: &str| DigestStep {
        step_id: id.into(),
        kind_key: kind.into(),
        step_label: label.into(),
        elapsed_time: elapsed,
        is_running: elapsed.is_none(),
        model_id: Some(model.into()),
    };
    let model = |id: &str,
                 name: &str,
                 role: &str,
                 effort: Effort,
                 time: f64,
                 tokens: (u64, u64),
                 spend: f64,
                 provider: &str,
                 note: Option<&str>| DigestModel {
        model_id: id.into(),
        display_name: name.into(),
        logo: None,
        role_label: role.into(),
        effort_level: effort,
        time_taken: time,
        provider_name: provider.into(),
        token_counts: Some(TokenCounts {
            input: tokens.0,
            output: tokens.1,
        }),
        spend: Some(spend),
        selection_note: note.map(Into::into),
    };
    let models = vec![
        model(
            "m1",
            "Quill",
            "Retrieval",
            Effort::High,
            4.0,
            (12_400, 1_900),
            0.07,
            "Northwind",
            Some("Chosen for long context"),
        ),
        model(
            "m2",
            "Harbor",
            "Critique",
            Effort::Low,
            1.0,
            (3_100, 420),
            0.01,
            "Kestrel",
            None,
        ),
        model(
            "m3",
            "Lumen",
            "Drafting",
            Effort::Max,
            3.0,
            (30_200, 4_100),
            0.18,
            "Halcyon",
            None,
        ),
    ];
    let mark = |colour: &str| {
        Some(format!(
            "data:image/svg+xml;utf8,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Crect width='24' height='24' rx='6' fill='{colour}'/%3E%3C/svg%3E"
        ))
    };
    let sources = vec![
        DigestSource {
            source_name: "Pricing page".into(),
            source_detail: Some("2024".into()),
            source_href: None,
            logo: mark("%234f46e5"),
        },
        DigestSource {
            source_name: "Changelog".into(),
            source_detail: Some("v3.2".into()),
            source_href: None,
            logo: mark("%230f766e"),
        },
        DigestSource {
            source_name: "Competitor filings".into(),
            source_detail: Some("Q2".into()),
            source_href: None,
            logo: mark("%23b45309"),
        },
    ];
    let kind = |label: &str, mark: &str| KindSpec {
        kind_label: label.into(),
        kind_emphasis: Emphasis::Normal,
        kind_mark: Some(mark.into()),
    };
    let kinds = BTreeMap::from([
        ("lookup".to_string(), kind("Lookup", "⌕")),
        ("compare".to_string(), kind("Compare", "⇄")),
        ("verify".to_string(), kind("Verify", "✓")),
    ]);
    rsx! {
        Item { slug: "deliberation-digest", name: "DeliberationDigest",
            DeliberationDigest {
                step_list: vec![
                    step("s1", "lookup", "Pulled the pricing table", Some(4.0), "m1"),
                    step("s2", "lookup", "Opened the release notes", Some(3.0), "m1"),
                    step("s3", "compare", "Matched tiers to usage", Some(2.0), "m3"),
                    step("s4", "compare", "Ranked the two options", Some(2.0), "m3"),
                    step("s5", "verify", "Checked totals against the invoice", Some(1.0), "m2"),
                    step("s6", "verify", "Checking the final figures", None, "m2"),
                ],
                model_list: models,
                source_list: sources,
                total_source_count: Some(9),
                is_reasoning: true,
                expected_time: Some(14.0),
                kind_table: kinds,
                agent_identity: Some(AgentIdentity { agent_id: "planner".into(), agent_emphasis: Emphasis::Normal }),
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
                subtask_text: "Book the venue".into(),
                is_done: true,
                assignee_name: Some("Ben".into()),
            },
            Subtask {
                subtask_id: "t3".into(),
                subtask_text: "Confirm speakers".into(),
                is_done: false,
                assignee_name: Some("Ada".into()),
            },
            Subtask {
                subtask_id: "t4".into(),
                subtask_text: "Send invites".into(),
                is_done: false,
                assignee_name: None,
            },
            Subtask {
                subtask_id: "t5".into(),
                subtask_text: "Arrange catering".into(),
                is_done: false,
                assignee_name: Some("Chloe".into()),
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
                status_id: "doing",
                status_options: vec![
                    StatusOption { option_id: "todo".into(), option_label: "To do".into(), emphasis: Tone::Neutral, option_mark: None },
                    StatusOption { option_id: "doing".into(), option_label: "In progress".into(), emphasis: Tone::Neutral, option_mark: None },
                    StatusOption { option_id: "done".into(), option_label: "Done".into(), emphasis: Tone::Positive, option_mark: None },
                ],
                on_status_pick: move |id: String| last.set(format!("status {id}")),
                due_text: Some("Fri".to_string()),
                member_list: vec![
                    Member { member_name: "Ada".into(), avatar: None },
                    Member { member_name: "Ben".into(), avatar: None },
                    Member { member_name: "Chloe".into(), avatar: None },
                ],
                primary_action: Some(PrimaryAction { action_label: "Open thread".into(), on_activate: EventHandler::new(move |_| last.set("primary".into())) }),
                footer_hint: Some("Updated 10:42 by Ada".to_string()),
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
    let source = |id: &str, chip: &str, snippet: &str| SourceEntry {
        entry_id: id.into(),
        chip_text: chip.into(),
        link_target: None,
        leading_mark: None,
        snippet: Some(snippet.into()),
    };
    let sources = vec![
        source(
            "s1",
            "Q2 report",
            "Revenue grew 12 percent quarter over quarter.",
        ),
        source(
            "s2",
            "Churn sheet",
            "Gross churn fell to 3.1 percent in June.",
        ),
        source(
            "s3",
            "Pricing review",
            "The Pro tier moved to annual billing in March.",
        ),
        source(
            "s4",
            "Support log",
            "Refund requests dropped after the onboarding change.",
        ),
        source("s5", "Board notes", "Target for Q3: churn under 3 percent."),
    ];
    rsx! {
        Item { slug: "grounded-reply", name: "GroundedReply",
            GroundedReply {
                heading_text: "Why did revenue grow in Q2?",
                source_entries: sources,
                answer_text: "Revenue grew 12 percent in Q2 [1], and churn fell to 3.1 percent in June [2].\n\nThe March move to annual billing drove most of the lift [3].\n\n- Refund requests fell after the onboarding update [4].\n- The Q3 target is churn under 3 percent [5].",
                next_questions: vec!["What drove the growth?".to_string(), "Which cohort churned most?".to_string()],
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
                prompt_chips: vec![
                    "Shorter".to_string(),
                    "More formal".to_string(),
                    "Translate to French".to_string(),
                    "Fix the grammar".to_string(),
                ],
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
                before_text: "The team will ship the update next week, once the pricing notes are final.",
                after_text: "The team ships the update on Tuesday, once the pricing notes and churn figures are final.",
                segments: vec![
                    Segment::Same("The team ".to_string()),
                    Segment::Change { removed: "will ship".to_string(), inserted: "ships".to_string() },
                    Segment::Same(" the update ".to_string()),
                    Segment::Change { removed: "next week".to_string(), inserted: "on Tuesday".to_string() },
                    Segment::Same(", once the ".to_string()),
                    Segment::Change { removed: "pricing notes".to_string(), inserted: "pricing notes and churn figures".to_string() },
                    Segment::Same(" are final.".to_string()),
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
    let at = |secs: i64| Some(1_699_920_000 + secs);
    let entry = |id: &str, from: &str, to: &str, kind: EntryKind, body: &str, sent: i64| Entry {
        entry_id: id.into(),
        sender_id: from.into(),
        recipient_ids: vec![to.into()],
        entry_kind: kind,
        body_text: body.into(),
        sent_at: at(sent),
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
        "Pull three pricing comparables for the Pro tier. Keep figures in USD per seat.",
        33_120,
    );
    task.attachment_list = vec![Attachment {
        label: "pricing-notes.md".into(),
        kind: Some(AttachmentKind::File),
        preview: None,
        meta: Some("3 KB".into()),
    }];
    let started = entry(
        "e2",
        "writer",
        "lead",
        EntryKind::AgentStarted,
        "Writer started on the pricing summary.",
        33_180,
    );
    let mut answer = entry(
        "e3",
        "research",
        "lead",
        EntryKind::Answer,
        "Three found: Acme at $29, Brightline at $24 and Corva at $39 per seat.",
        34_260,
    );
    answer.answers_id = Some("e1".into());
    answer.attachment_list = vec![Attachment {
        label: "comparables.csv".into(),
        kind: None,
        preview: Some("vendor,seat_price\nAcme,29\nBrightline,24\nCorva,39".into()),
        meta: Some("12 rows".into()),
    }];
    answer.detail_card = Some(DetailCard {
        notes: Some("Sources checked".into()),
        rows: vec![
            ("Sources".into(), "4".into()),
            ("Median seat price".into(), "$29".into()),
        ],
    });
    let broadcast = entry(
        "e4",
        "lead",
        "all",
        EntryKind::Broadcast,
        "Keep answers under 200 words and quote figures from the table.",
        34_800,
    );
    let question = entry(
        "e5",
        "writer",
        "you",
        EntryKind::NeedsPerson,
        "Do we publish the Corva figures? Their pricing page is dated 2023.",
        36_120,
    );
    let mut handover = entry(
        "e6",
        "critic",
        "writer",
        EntryKind::Handover,
        "Handing over once the figures are confirmed.",
        36_300,
    );
    handover.is_drafting = true;
    rsx! {
        Item { slug: "inter-agent-message-feed", name: "InterAgentMessageFeed",
            InterAgentMessageFeed {
                participants: vec![
                    p("lead", "Lead"),
                    p("research", "Research"),
                    p("writer", "Writer"),
                    p("critic", "Critic"),
                ],
                entries: vec![task, started, answer, broadcast, question, handover],
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
