use dioxus::prelude::*;
use rv2_affect::change_summary::{ChangeSummary, MetricChange, MetricTone};
use rv2_affect::code_block::CodeBlock;
use rv2_affect::comment_thread::{CommentThread, ThreadAuthor, ThreadMessage};
use rv2_affect::decision_inbox::{Decision, DecisionAnswer, DecisionInbox};
use rv2_affect::email_draft::EmailDraft;
use rv2_affect::message_actions::{MessageActionKey, MessageActions, MessageText};
use rv2_affect::notification_stack::{Notice, NoticeAction, NoticePattern, NotificationStack};
use rv2_affect::progress_tracker::{ProgressStep, ProgressTracker};
use rv2_affect::reaction_bar::{ReactionBar, ReactionKey, ReactionState};
use rv2_affect::smart_recommendation::{Recommendation, SmartRecommendation};
use rv2_affect::step_rail::{RailState, RailStep, StepRail};
use rv2_affect::thinking_loader::{
    DoneLabel, ThinkingLoader, ThinkingLoaderSize, ThinkingLoaderVariant,
};
use rv2_affect::tips_panel::{Tip, TipsPanel};
use rv2_affect::tool_call_card::{GroupCall, ToolCallCard, ToolCallGroup, ToolCallStatus};

use crate::gallery::Ex;
use crate::{Item, Section};

const SNIPPET: &str = "const total = items\n  .filter((i) => i.open)\n  .reduce((sum, i) => sum + i.value, 0)\nconsole.log(total)";
const SHELL: &str = "cargo xtask ci\ncargo xtask live";

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
            detail: Some("Test run cut the nightly import to 6 minutes · 2 weeks of data".into()),
            primary: "Approve".into(),
            secondary: "Decline".into(),
            collapsed: false,
        },
        Decision {
            id: "d2".into(),
            title: "Raise the seat price by 8%?".into(),
            detail: Some("12 seats · $1,800 per year · renews in 5 days".into()),
            primary: "Raise price".into(),
            secondary: "Keep price".into(),
            collapsed: false,
        },
    ]
}

fn notice(id: &str, title: &str, pattern: NoticePattern) -> Notice {
    Notice {
        id: id.into(),
        badge: Some("Sync".into()),
        title: title.into(),
        text: "Nightly import finished.".into(),
        action: None,
        pattern,
    }
}

fn deck() -> Vec<Notice> {
    vec![
        Notice {
            id: "n1".into(),
            badge: Some("New".into()),
            title: "Dark mode is here".into(),
            text: "Every screen now follows your system theme, or pick one yourself in settings."
                .into(),
            action: Some(NoticeAction {
                label: "Try it".into(),
                href: None,
                on_click: None,
                external: false,
            }),
            pattern: NoticePattern::Orbit,
        },
        notice("n2", "Import finished", NoticePattern::Grid),
        notice("n3", "Export ready", NoticePattern::Rays),
    ]
}

fn tip(id: &str, title: &str, text: &str, action: &str, primary: bool) -> Tip {
    Tip {
        id: id.into(),
        title: title.into(),
        description: rsx! { "{text}" },
        action: action.into(),
        done_label: None,
        primary,
        on_action: None,
    }
}

fn tips(mut done: Signal<String>) -> Vec<Tip> {
    let mut search = tip(
        "search",
        "Quick search",
        "Press Cmd K to jump to any page, project or setting.",
        "Try it",
        false,
    );
    search.on_action = Some(EventHandler::new(move |_| {
        done.set("search tried".to_string())
    }));
    vec![
        tip(
            "invite",
            "Invite your team",
            "Work on projects together and keep feedback in one place.",
            "Invite",
            false,
        ),
        search,
        tip(
            "digest",
            "Weekly digest",
            "Get a short summary of what changed every Monday morning.",
            "Turn on",
            false,
        ),
        tip(
            "slack",
            "Connect Slack",
            "Post an update to a channel when something needs attention.",
            "Connect",
            false,
        ),
        tip(
            "mobile",
            "Mobile app",
            "Check activity and approve requests while you're on the go.",
            "Get the app",
            false,
        ),
        tip(
            "upgrade",
            "Ready to upgrade?",
            "More seats, priority support and advanced analytics.",
            "See plans",
            true,
        ),
        tip(
            "keys",
            "Keyboard map",
            "Press ? anywhere to see every shortcut.",
            "Open",
            false,
        ),
        tip(
            "import",
            "Import a CSV",
            "Bring runs and contacts in from a spreadsheet.",
            "Import",
            false,
        ),
        tip(
            "audit",
            "Audit log",
            "See who changed what, and when.",
            "View",
            false,
        ),
    ]
}

fn step(id: &str, label: &str, done: bool) -> ProgressStep {
    ProgressStep {
        id: id.into(),
        label: label.into(),
        done,
        icon: None,
    }
}

const RITUAL: [&str; 8] = [
    "Think", "Spec", "Approve", "Plan", "Build", "Review", "Ship", "Reflect",
];

/// The ritual's steps with the flow standing at `at`, in `live`.
fn ritual(at: usize, live: RailState) -> Vec<RailStep> {
    RITUAL
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let state = if i < at {
                RailState::Finished
            } else if i == at {
                live
            } else {
                RailState::Queued
            };
            RailStep::new(name.to_lowercase(), *name, state)
        })
        .collect()
}

#[component]
pub fn Content() -> Element {
    let mut thread = use_signal(|| {
        vec![
            ThreadMessage {
                id: "m1".into(),
                author: ada(),
                created_at: 1_700_000_000_000,
                text: "/edit tighten the intro".into(),
                status: None,
                details: Vec::new(),
            },
            ThreadMessage {
                id: "m2".into(),
                author: helper(),
                created_at: 1_700_000_060_000,
                text: "Tightened the intro to two sentences.".into(),
                status: Some("Edit completed".into()),
                details: vec!["Replaced three sentences with two".into()],
            },
        ]
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
    let mut stack = use_signal(deck);
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
                Ex { label: "This week",
                    ChangeSummary {
                        period: "This week".to_string(),
                        changes: vec![
                            MetricChange { label: "Revenue".into(), change: "Up 6% to 18.2K".into(), tone: MetricTone::Good, direction: None },
                            MetricChange { label: "Churn".into(), change: "Up 1.2 pts".into(), tone: MetricTone::Bad, direction: None },
                            MetricChange { label: "Seats".into(), change: "Flat".into(), tone: MetricTone::Neutral, direction: None },
                        ],
                    }
                }
                Ex { label: "Single metric, this month",
                    ChangeSummary {
                        period: "This month".to_string(),
                        changes: vec![
                            MetricChange { label: "Trial starts".into(), change: "Up 22 to 140".into(), tone: MetricTone::Good, direction: None },
                        ],
                    }
                }
            }
            Item { name: "CodeBlock",
                Ex { label: "TypeScript, line numbers and highlight",
                    CodeBlock { code: SNIPPET.to_string(), language: "ts", show_line_numbers: true, highlight_lines: vec![2] }
                }
                Ex { label: "Shell, no copy button",
                    CodeBlock { code: SHELL.to_string(), language: "bash", copyable: false }
                }
            }
            Item { name: "CommentThread",
                Ex { label: "With a reply thread",
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
            }
            Item { name: "DecisionInbox",
                Ex { label: "Two open decisions",
                    div { class: "gl-box",
                        DecisionInbox {
                            decisions: decisions(),
                            on_decide: move |(d, _): (Decision, DecisionAnswer)| decided.set(d.id),
                        }
                    }
                }
                Ex { label: "Nothing waiting",
                    div { class: "gl-box",
                        DecisionInbox { decisions: Vec::<Decision>::new(), on_decide: move |_: (Decision, DecisionAnswer)| {} }
                    }
                }
                p { class: "gl-out", "Last decided: {decided}" }
            }
            Item { name: "EmailDraft",
                Ex { label: "Reply to a renewal",
                    div { class: "gl-box",
                        EmailDraft {
                            on_send: move |_| sent.set("sent".to_string()),
                            on_discard: move |_| sent.set("discarded".to_string()),
                        }
                    }
                }
                p { class: "gl-out", "Email: {sent}" }
            }
            Item { name: "MessageActions",
                Ex { label: "In a conversation, rated down",
                    div { class: "gl-box", style: "display: grid; gap: 10px;",
                        div { style: "justify-self: end; max-width: 80%; padding: 8px 12px; border-radius: 12px; background: #0000000d; font-size: 13px;",
                            "Summarize the 2.4 release notes in one sentence."
                        }
                        div { style: "display: grid; gap: 6px; font-size: 13px;",
                            span { style: "font-size: 12px; opacity: 0.6;", "Sonnet 5.5" }
                            p { style: "margin: 0;", "Version 2.4 adds shared folders, halves sync time and fixes the Windows export bug." }
                        }
                        MessageActions {
                            text: "Version 2.4 adds shared folders, halves sync time and fixes the Windows export bug.".to_string(),
                            feedback_open: true,
                            on_feedback: move |(kind, reason)| feedback.set(format!("{kind:?} {reason:?}")),
                        }
                    }
                }
                Ex { label: "Two versions, feedback",
                    MessageActions {
                        text: "Here is a short summary of the renewal terms.".to_string(),
                        versions: 2usize,
                        on_version_change: move |v: usize| version.set(v),
                        on_feedback: move |(kind, reason)| feedback.set(format!("{kind:?} {reason:?}")),
                    }
                }
                Ex { label: "Copy and speak only",
                    MessageActions { actions: vec![MessageActionKey::Copy, MessageActionKey::Speak] }
                }
                Ex { label: "Regenerate and feedback",
                    MessageActions { actions: vec![MessageActionKey::Regenerate, MessageActionKey::Feedback] }
                }
                p { class: "gl-out", "Version index: {version}. Feedback: {feedback}" }
                MessageText { version: version().to_string(), p { "Version body number {version}" } }
            }
            Item { name: "NotificationStack",
                Ex { label: "One deck, front notice with an action",
                    div { class: "gl-box",
                        NotificationStack {
                            notices: stack(),
                            on_dismiss: move |n: Notice| stack.write().retain(|x| x.id != n.id),
                        }
                    }
                }
                button { r#type: "button", onclick: move |_| stack.set(deck()), "Reset stack" }
            }
            Item { name: "ProgressTracker",
                Ex { label: "Onboarding, in progress",
                    div { class: "gl-box",
                        ProgressTracker {
                            steps: steps(),
                            on_change: move |next: Vec<ProgressStep>| steps.set(next),
                        }
                    }
                }
                Ex { label: "All done",
                    div { class: "gl-box",
                        ProgressTracker {
                            steps: vec![step("a", "Connect", true), step("b", "Invite", true), step("c", "First sync", true)],
                            on_change: move |_: Vec<ProgressStep>| {},
                        }
                    }
                }
            }
            Item { name: "ReactionBar",
                Ex { label: "Like, active",
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
                Ex { label: "Comment, repost and share",
                    ReactionBar {
                        reactions: vec![(ReactionKey::Comment, ReactionState { count: 18, active: false })],
                        actions: vec![ReactionKey::Comment, ReactionKey::Repost, ReactionKey::Share],
                        on_react: move |_: (ReactionKey, ReactionState)| {},
                    }
                }
            }
            Item { name: "SmartRecommendation",
                Ex { label: "Two options, signal shown",
                    SmartRecommendation {
                        options: vec![
                            Recommendation { id: "a".into(), label: "Incremental sync".into(), reason: rsx! { "Fewer writes per night" }, signal: 3 },
                            Recommendation { id: "b".into(), label: "Full sync".into(), reason: rsx! { "Simpler to reason about" }, signal: 1 },
                        ],
                        on_accept: move |r: Recommendation| accepted.set(r.label),
                    }
                }
                Ex { label: "Single option",
                    SmartRecommendation {
                        options: vec![
                            Recommendation { id: "c".into(), label: "Sync on save".into(), reason: rsx! { "Matches how Northwind edits" }, signal: 2 },
                        ],
                        on_accept: move |_: Recommendation| {},
                    }
                }
                p { class: "gl-out", "Accepted: {accepted}" }
            }
            Item { name: "StepRail", wide: true,
                Ex { label: "Ritual, at Build",
                    StepRail { steps: ritual(4, RailState::Current), aria_label: "Ritual steps" }
                }
                Ex { label: "Ritual, Spec drifted after approval",
                    StepRail { steps: ritual(1, RailState::Errored), aria_label: "Ritual steps" }
                }
                Ex { label: "Ritual, awaiting approval",
                    StepRail { steps: ritual(2, RailState::AwaitingApproval), aria_label: "Ritual steps" }
                }
            }
            Item { name: "ThinkingLoader",
                Ex { label: "Pill, running",
                    ThinkingLoader { done: thinking_done() }
                    button { r#type: "button", onclick: move |_| thinking_done.set(!thinking_done()), "Toggle done" }
                }
                Ex { label: "Plain, small, in a bubble",
                    ThinkingLoader { variant: ThinkingLoaderVariant::Plain, size: ThinkingLoaderSize::Sm, done: false }
                }
                Ex { label: "Done with a label",
                    ThinkingLoader { done: true, done_label: DoneLabel::Text("Answer ready".to_string()) }
                }
            }
            Item { name: "TipsPanel", wide: true,
                Ex { label: "Nine tips in a grid, with close",
                    div { class: "gl-box", style: "max-width: none;",
                        TipsPanel {
                            tips: tips(tips_done),
                            on_close: Some(EventHandler::new(move |_| tips_done.set("closed".to_string()))),
                        }
                    }
                }
                p { class: "gl-out", "Tip action: {tips_done}" }
            }
            Item { name: "ToolCallCard",
                Ex { label: "Success",
                    div { class: "gl-box",
                        ToolCallCard { name: "lookup_account", summary: "Read the Northwind account", status: ToolCallStatus::Success, result: "{{\"ok\": true, \"seats\": 42}}", duration: 400u64 }
                    }
                }
                Ex { label: "Running",
                    div { class: "gl-box",
                        ToolCallCard { name: "draft_reply", summary: "Drafting a reply to Jonas", status: ToolCallStatus::Running }
                    }
                }
                Ex { label: "Error",
                    div { class: "gl-box",
                        ToolCallCard { name: "search_web", summary: "Searched the web for 'CRM pricing'", status: ToolCallStatus::Error, error: "Timed out after 5s", duration: 5012u64 }
                    }
                }
                Ex { label: "Grouped, calls listed",
                    div { class: "gl-box",
                        ToolCallGroup {
                            calls: vec![
                                GroupCall { name: "search_web".into(), icon: None, status: ToolCallStatus::Success, duration: Some(1500), started_at: None },
                                GroupCall { name: "fetch_page".into(), icon: None, status: ToolCallStatus::Running, duration: None, started_at: None },
                            ],
                            default_open: true,
                            ToolCallCard { name: "search_web", summary: "Searched the web for 'CRM pricing'", status: ToolCallStatus::Success, duration: 1500u64 }
                            ToolCallCard { name: "fetch_page", summary: "Reading acme.example/pricing", status: ToolCallStatus::Running }
                        }
                    }
                }
            }
            Item { name: "ToolCallGroup",
                Ex { label: "Finished, one failed",
                    div { class: "gl-box",
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
                Ex { label: "Still running",
                    div { class: "gl-box",
                        ToolCallGroup {
                            calls: vec![
                                GroupCall { name: "search_crm".into(), icon: None, status: ToolCallStatus::Success, duration: Some(180), started_at: None },
                                GroupCall { name: "pull_invoices".into(), icon: None, status: ToolCallStatus::Running, duration: None, started_at: None },
                            ],
                            span { "Pulling invoices for Northwind." }
                        }
                    }
                }
            }
        }
    }
}
