use dioxus::prelude::*;
use rv2_affect::change_summary::{ChangeSummary, MetricChange, MetricTone};
use rv2_affect::code_block::CodeBlock;
use rv2_affect::comment_thread::{CommentThread, ThreadAuthor, ThreadMessage};
use rv2_affect::decision_inbox::{Decision, DecisionAnswer, DecisionInbox};
use rv2_affect::email_draft::EmailDraft;
use rv2_affect::message_actions::{MessageActionKey, MessageActions, MessageText};
use rv2_affect::notification_stack::{Notice, NoticePattern, NotificationStack};
use rv2_affect::progress_tracker::{ProgressStep, ProgressTracker};
use rv2_affect::reaction_bar::{ReactionBar, ReactionKey, ReactionState};
use rv2_affect::smart_recommendation::{Recommendation, SmartRecommendation};
use rv2_affect::thinking_loader::ThinkingLoader;
use rv2_affect::tips_panel::{Tip, TipsPanel};
use rv2_affect::tool_call_card::{GroupCall, ToolCallCard, ToolCallGroup, ToolCallStatus};

use crate::{Item, Section};

const SNIPPET: &str = "const total = items\n  .filter((i) => i.open)\n  .reduce((sum, i) => sum + i.value, 0)\nconsole.log(total)";

fn ada() -> ThreadAuthor {
    ThreadAuthor {
        name: "Ada Lovelace".into(),
        avatar: None,
        assistant: false,
    }
}

fn helper() -> ThreadAuthor {
    ThreadAuthor {
        name: "Helper".into(),
        avatar: None,
        assistant: true,
    }
}

fn decisions() -> Vec<Decision> {
    vec![
        Decision {
            id: "d1".into(),
            title: "Use incremental sync for Northwind?".into(),
            detail: None,
            primary: "Approve".into(),
            secondary: "Decline".into(),
            collapsed: false,
        },
        Decision {
            id: "d2".into(),
            title: "Raise the seat price by 8%?".into(),
            detail: None,
            primary: "Approve".into(),
            secondary: "Hold".into(),
            collapsed: true,
        },
    ]
}

fn notice(id: &str, title: &str) -> Notice {
    Notice {
        id: id.into(),
        badge: Some("Sync".into()),
        title: title.into(),
        text: "Nightly import finished.".into(),
        action: None,
        pattern: NoticePattern::Grid,
    }
}

fn step(id: &str, label: &str, done: bool) -> ProgressStep {
    ProgressStep {
        id: id.into(),
        label: label.into(),
        done,
        icon: None,
    }
}

#[component]
pub fn Content() -> Element {
    let mut thread = use_signal(|| {
        vec![ThreadMessage {
            id: "m1".into(),
            author: ada(),
            created_at: 1_700_000_000_000,
            text: "/edit tighten the intro".into(),
            status: None,
            details: Vec::new(),
        }]
    });
    let mut reactions = use_signal(|| {
        vec![(
            ReactionKey::Like,
            ReactionState {
                count: 1234,
                active: true,
            },
        )]
    });
    let mut stack = use_signal(|| {
        vec![
            notice("n1", "Import finished"),
            notice("n2", "Export ready"),
        ]
    });
    let mut steps = use_signal(|| {
        vec![
            step("connect", "Connect", true),
            step("invite", "Invite", false),
            step("sync", "First sync", false),
        ]
    });
    let mut accepted = use_signal(|| "none".to_string());
    let mut decided = use_signal(|| "none".to_string());
    let mut tips_done = use_signal(|| "none".to_string());
    let mut version = use_signal(|| 0usize);
    let mut feedback = use_signal(|| "none".to_string());
    let mut thinking_done = use_signal(|| false);
    let mut sent = use_signal(|| "not sent".to_string());

    rsx! {
        Section { id: "content", title: "AI and content",
            Item { name: "ChangeSummary",
                ChangeSummary {
                    period: "This week".to_string(),
                    changes: vec![
                        MetricChange { label: "Revenue".into(), change: "Up 6% to 18.2K".into(), tone: MetricTone::Good, direction: None },
                        MetricChange { label: "Churn".into(), change: "Up 1.2 pts".into(), tone: MetricTone::Bad, direction: None },
                        MetricChange { label: "Seats".into(), change: "Flat".into(), tone: MetricTone::Neutral, direction: None },
                    ],
                }
            }
            Item { name: "CodeBlock",
                CodeBlock { code: SNIPPET.to_string(), language: "ts", show_line_numbers: true, highlight_lines: vec![2] }
            }
            Item { name: "CommentThread",
                div { class: "gl-box",
                    CommentThread {
                        messages: thread(),
                        current_user: ada(),
                        assistant: helper(),
                        on_reply: move |text: String| {
                            let n = thread.read().len() + 1;
                            thread.write().push(ThreadMessage {
                                id: format!("m{n}"),
                                author: ada(),
                                created_at: 1_700_000_000_000,
                                text,
                                status: None,
                                details: Vec::new(),
                            });
                        },
                    }
                }
            }
            Item { name: "DecisionInbox",
                div { class: "gl-box",
                    DecisionInbox {
                        decisions: decisions(),
                        on_decide: move |(d, _): (Decision, DecisionAnswer)| decided.set(d.id),
                    }
                }
                p { class: "gl-out", "Last decided: {decided}" }
            }
            Item { name: "EmailDraft",
                div { class: "gl-box",
                    EmailDraft {
                        on_send: move |_| sent.set("sent".to_string()),
                        on_discard: move |_| sent.set("discarded".to_string()),
                    }
                }
                p { class: "gl-out", "Email: {sent}" }
            }
            Item { name: "MessageActions",
                div { class: "gl-stack",
                    MessageActions {
                        text: "Here is a short summary of the renewal terms.".to_string(),
                        versions: 2usize,
                        on_version_change: move |v: usize| version.set(v),
                        on_feedback: move |(kind, reason)| feedback.set(format!("{kind:?} {reason:?}")),
                    }
                    MessageActions { actions: vec![MessageActionKey::Copy, MessageActionKey::Speak] }
                }
                p { class: "gl-out", "Version index: {version}. Feedback: {feedback}" }
                MessageText { version: version().to_string(), p { "Version body number {version}" } }
            }
            Item { name: "NotificationStack",
                div { class: "gl-box",
                    NotificationStack {
                        notices: stack(),
                        on_dismiss: move |n: Notice| stack.write().retain(|x| x.id != n.id),
                    }
                }
                button { r#type: "button", onclick: move |_| stack.set(vec![notice("n3", "Fresh notice")]), "Reset stack" }
            }
            Item { name: "ProgressTracker",
                div { class: "gl-box",
                    ProgressTracker {
                        steps: steps(),
                        on_change: move |next: Vec<ProgressStep>| steps.set(next),
                    }
                }
            }
            Item { name: "ReactionBar",
                ReactionBar {
                    reactions: reactions(),
                    actions: vec![ReactionKey::Like, ReactionKey::Share],
                    on_react: move |(key, state): (ReactionKey, ReactionState)| {
                        let mut all = reactions.write();
                        all.retain(|(k, _)| *k != key);
                        all.push((key, state));
                    },
                }
            }
            Item { name: "SmartRecommendation",
                SmartRecommendation {
                    options: vec![
                        Recommendation { id: "a".into(), label: "Incremental sync".into(), reason: rsx! { "Fewer writes per night" }, signal: 3 },
                        Recommendation { id: "b".into(), label: "Full sync".into(), reason: rsx! { "Simpler to reason about" }, signal: 1 },
                    ],
                    on_accept: move |r: Recommendation| accepted.set(r.label),
                }
                p { class: "gl-out", "Accepted: {accepted}" }
            }
            Item { name: "ThinkingLoader",
                div { class: "gl-row",
                    ThinkingLoader { done: thinking_done() }
                    button { r#type: "button", onclick: move |_| thinking_done.set(!thinking_done()), "Toggle done" }
                }
            }
            Item { name: "TipsPanel",
                div { class: "gl-box",
                    TipsPanel {
                        tips: vec![
                            Tip { id: "shortcut".into(), title: "Shortcut".into(), description: rsx! { "Press Cmd K to search" }, action: "Enable".into(), done_label: None, primary: true, on_action: Some(EventHandler::new(move |_| tips_done.set("shortcut enabled".to_string()))) },
                            Tip { id: "sync".into(), title: "Sync".into(), description: rsx! { "Sync often to avoid conflicts" }, action: "Try".into(), done_label: None, primary: false, on_action: None },
                        ],
                    }
                }
                p { class: "gl-out", "Tip action: {tips_done}" }
            }
            Item { name: "ToolCallCard",
                div { class: "gl-stack",
                    ToolCallCard { name: "lookup_account", status: ToolCallStatus::Success, result: "{{\"ok\": true, \"seats\": 42}}", duration: 400u64 }
                    ToolCallCard { name: "search_web", status: ToolCallStatus::Error, error: "Timed out after 5s", duration: 5012u64 }
                }
            }
            Item { name: "ToolCallGroup",
                ToolCallGroup {
                    calls: vec![
                        GroupCall { name: "search_crm".into(), icon: None, status: ToolCallStatus::Success, duration: Some(180), started_at: None },
                        GroupCall { name: "draft_email".into(), icon: None, status: ToolCallStatus::Success, duration: Some(920), started_at: None },
                        GroupCall { name: "send_email".into(), icon: None, status: ToolCallStatus::Error, duration: Some(50), started_at: None },
                    ],
                    span { "Drafted a reply to Jonas." }
                }
            }
        }
    }
}
