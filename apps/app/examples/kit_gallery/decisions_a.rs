use dioxus::prelude::*;

use super::{Item, NOW_MS, gallery_now};

#[component]
pub fn AccessGrantPromptDemo() -> Element {
    use rv2_kit::components::access_grant_prompt::{
        AccessGrantPrompt, Decision, ImpactItem, Requester, RiskLevel,
    };
    let mut last = use_signal(String::new);
    rsx! {
        Item { slug: "access-grant-prompt", name: "AccessGrantPrompt",
            AccessGrantPrompt {
                requester: Requester { name: "Research agent".into(), model_name: Some("model A".into()), avatar: None },
                requested_action: "Read files in ~/Projects/notes",
                target_label: Some("notes/".to_string()),
                risk_level: RiskLevel::Risky,
                agent_rationale: Some("The summary needs the meeting notes from last week.".to_string()),
                impact_items: vec![
                    ImpactItem { item_id: "1".into(), item_label: "2024-meetings.md".into(), item_meta: Some("12 KB".into()), item_mark: None },
                    ImpactItem { item_id: "2".into(), item_label: "roadmap.md".into(), item_meta: None, item_mark: None },
                    ImpactItem { item_id: "3".into(), item_label: "people.csv".into(), item_meta: None, item_mark: None },
                    ImpactItem { item_id: "4".into(), item_label: "drafts/".into(), item_meta: Some("9 files".into()), item_mark: None },
                ],
                current_time: NOW_MS as i64 / 1000,
                on_decision_change: move |d: Option<Decision>| {
                    last.set(match d {
                        Some(d) => format!("allowed={} window={:?}", d.allowed, d.window),
                        None => "cleared".to_string(),
                    })
                },
            }
            div { class: "gl-out", "data-out": "access-grant-prompt", "{last()}" }
        }
    }
}

#[component]
pub fn AgentQuestionCardDemo() -> Element {
    use rv2_kit::components::agent_question_card::{
        AgentQuestionCard, Answer, AnswerEvent, Choice, DetailRow, Question, QuestionKind,
    };
    let mut last = use_signal(String::new);
    let questions = vec![
        Question {
            question_id: "q1".into(),
            question_text: "Which export format should the report use?".into(),
            task_name: Some("Quarterly report".into()),
            question_kind: QuestionKind::Choice,
            answer_choices: vec![
                Choice {
                    choice_id: "pdf".into(),
                    choice_title: "PDF".into(),
                    choice_summary: Some("Print-ready".into()),
                    is_suggested: true,
                    preview_text: None,
                },
                Choice {
                    choice_id: "md".into(),
                    choice_title: "Markdown".into(),
                    choice_summary: None,
                    is_suggested: false,
                    preview_text: None,
                },
            ],
            detail_rows: vec![DetailRow {
                row_name: "Pages".into(),
                row_text: "14".into(),
            }],
        },
        Question {
            question_id: "q2".into(),
            question_text: "Send the draft to the finance team?".into(),
            task_name: None,
            question_kind: QuestionKind::Approval,
            answer_choices: vec![],
            detail_rows: vec![],
        },
    ];
    let mut pending = use_signal(|| questions);
    rsx! {
        Item { slug: "agent-question-card", name: "AgentQuestionCard",
            AgentQuestionCard {
                pending_questions: pending(),
                on_answer: move |e: AnswerEvent| {
                    last.set(match &e.answer {
                        Answer::Chosen(c) => format!("{} chose {c}", e.question_id),
                        Answer::FreeText(t) => format!("{} text {t}", e.question_id),
                        Answer::Approve => format!("{} approved", e.question_id),
                        Answer::Decline => format!("{} declined", e.question_id),
                    });
                    // A chosen option stays on screen so its selection can be seen; other answers close the question.
                    if !matches!(e.answer, Answer::Chosen(_)) {
                        pending.with_mut(|list| list.retain(|q| q.question_id != e.question_id));
                    }
                },
                on_fold: move |id: String| last.set(format!("fold {id}")),
            }
            div { class: "gl-out", "data-out": "agent-question-card", "{last()}" }
        }
    }
}

#[component]
pub fn AnnotationSidebarDemo() -> Element {
    use rv2_kit::components::annotation_sidebar::PatchFuture;
    use rv2_kit::components::annotation_sidebar::{AnnotationSidebar, Note, NoteState, Writer};
    let mut last = use_signal(String::new);
    let writer = |name: &str| Writer {
        writer_name: name.into(),
        writer_mark: None,
    };
    let mut notes = use_signal(|| {
        vec![
            Note {
                note_id: "n1".into(),
                writer: writer("Ada"),
                posted_at: NOW_MS as i64 / 1000 - 3_600,
                note_text: "Tighten the intro".into(),
                note_state: NoteState::Open,
            },
            Note {
                note_id: "n2".into(),
                writer: writer("Ben"),
                posted_at: NOW_MS as i64 / 1000 - 7_200,
                note_text: "Rewrite this as a patch: shorter sentence.".into(),
                note_state: NoteState::Open,
            },
            Note {
                note_id: "n3".into(),
                writer: writer("Ada"),
                posted_at: NOW_MS as i64 / 1000 - 9_000,
                note_text: "Cite the 2024 figure".into(),
                note_state: NoteState::Resolved,
            },
        ]
    });
    rsx! {
        Item { slug: "annotation-sidebar", name: "AnnotationSidebar",
            AnnotationSidebar {
                notes: notes(),
                acting_writer: writer("You"),
                now_secs: gallery_now() as i64 / 1000,
                on_run_patch: move |n: rv2_kit::components::annotation_sidebar::Note| {
                    last.set(format!("patch {}", n.note_id));
                    Box::pin(async { Ok(()) }) as PatchFuture
                },
                on_note_added: move |n: rv2_kit::components::annotation_sidebar::Note| {
                    last.set(format!("added {}", n.note_text));
                    notes.with_mut(|list| list.push(n));
                },
                on_close_request: move |_| last.set("close".into()),
            }
            div { class: "gl-out", "data-out": "annotation-sidebar", "{last()}" }
        }
    }
}

#[component]
pub fn AssistantMemoryManagerDemo() -> Element {
    use rv2_kit::components::assistant_memory_manager::{
        AssistantMemoryManager, FooterAction, Memory,
    };
    let mut last = use_signal(String::new);
    let mem = |id: &str, text: &str, group: &str| Memory {
        memory_id: id.into(),
        memory_text: text.into(),
        memory_group: group.into(),
        source_label: Some("Chat".into()),
        date_label: Some("Mar 3".into()),
        is_paused: false,
    };
    let items = vec![
        mem("m1", "Prefers metric units", "Preferences"),
        mem("m2", "Works in Berlin time", "Preferences"),
        mem("m3", "Project Atlas launches in June", "Projects"),
    ];
    rsx! {
        Item { slug: "assistant-memory-manager", name: "AssistantMemoryManager",
            AssistantMemoryManager {
                memory_items: items,
                group_order: vec!["Preferences".to_string(), "Projects".to_string()],
                on_forget: move |id: String| last.set(format!("forget {id}")),
                on_restore: move |m: Memory| last.set(format!("restore {}", m.memory_id)),
                on_pause_change: move |(id, p): (String, bool)| last.set(format!("pause {id} {p}")),
                on_text_edit: move |(id, t): (String, String)| last.set(format!("edit {id} {t}")),
                on_add: move |m: Memory| last.set(format!("add {}", m.memory_text)),
                on_use_change: move |v: bool| last.set(format!("use {v}")),
                footer_action: Some(FooterAction { action_label: "Manage all".into(), on_action: EventHandler::new(move |_| last.set("manage all".into())) }),
            }
            div { class: "gl-out", "data-out": "assistant-memory-manager", "{last()}" }
        }
    }
}

#[component]
pub fn BackgroundJobsPopoverDemo() -> Element {
    use rv2_kit::components::background_jobs_popover::{
        BackgroundJob, BackgroundJobsPopover, Edge, JobState,
    };
    let mut last = use_signal(String::new);
    let job = |id: &str, title: &str, state: JobState, done: u32| BackgroundJob {
        job_id: id.into(),
        job_title: title.into(),
        current_step: "Indexing".into(),
        steps_done: done,
        steps_total: 10,
        job_state: state,
    };
    let mut jobs = use_signal(|| {
        vec![
            job("b1", "Index notes", JobState::Running, 4),
            job("b2", "Sync invoices", JobState::NeedsYou, 7),
            job("b3", "Backup", JobState::Paused, 2),
        ]
    });
    rsx! {
        Item { slug: "background-jobs-popover", name: "BackgroundJobsPopover",
            div { class: "gl-canvas", style: "min-height: 340px;",
            BackgroundJobsPopover {
                job_list: jobs(),
                default_open: true,
                align_edge: Edge::Start,
                on_pause: move |id: String| {
                    last.set(format!("pause {id}"));
                    jobs.with_mut(|list| list.iter_mut().filter(|j| j.job_id == id).for_each(|j| j.job_state = JobState::Paused));
                },
                on_resume: move |id: String| {
                    last.set(format!("resume {id}"));
                    jobs.with_mut(|list| list.iter_mut().filter(|j| j.job_id == id).for_each(|j| j.job_state = JobState::Running));
                },
                on_open_job: move |id: String| last.set(format!("open {id}")),
                on_open_all: move |_| last.set("open all".into()),
                on_open_change: move |o: bool| last.set(format!("open {o}")),
            }
            }
            div { class: "gl-out", "data-out": "background-jobs-popover", "{last()}" }
        }
    }
}

#[component]
pub fn ConsentRequestCardDemo() -> Element {
    use rv2_kit::components::consent_request_card::{
        ConsentRequestCard, ConsentRisk, DeclineCause, DetailRow, GrantOptions,
    };
    let mut last = use_signal(String::new);
    rsx! {
        Item { slug: "consent-request-card", name: "ConsentRequestCard",
            ConsentRequestCard {
                request_title: "Send weekly digest to 12 subscribers",
                context_line: "Requested by Digest agent",
                risk_level: Some(ConsentRisk::Medium),
                detail_rows: vec![
                    DetailRow { key_label: "Recipients".into(), value: "12".into() },
                    DetailRow { key_label: "Sends at".into(), value: "08:00".into() },
                ],
                timeout_seconds: Some(45),
                on_grant: move |o: GrantOptions| last.set(format!("grant keep={}", o.keep_for_action)),
                on_decline: move |c: DeclineCause| last.set(format!("decline {c:?}")),
                on_amend: move |_| last.set("amend".into()),
                on_timeout: move |_| last.set("timeout".into()),
                current_time: NOW_MS as i64 / 1000,
            }
            div { class: "gl-out", "data-out": "consent-request-card", "{last()}" }
        }
    }
}
