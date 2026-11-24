use dioxus::prelude::*;

use super::{Item, NOW_MS, Section, gallery_now};

const MIN: u64 = 60_000;

#[component]
pub fn Blocks() -> Element {
    rsx! {
        Section { id: "blocks", title: "Blocks",
            FleetItem {}
            TeamItem {}
            TimelineItem {}
            ChatItem {}
            WorkspaceItem {}
            WorkspaceLoadingItem {}
            StripItem {}
            CatalogItem {}
            InvoiceItem {}
            KanbanItem {}
            OverviewItem {}
            PlanItem {}
            RunItem {}
            SalesItem {}
        }
    }
}

#[component]
fn FleetItem() -> Element {
    use rv2_kit::blocks::agent_fleet_overview::*;
    let mut filter = use_signal(StatusFilter::default);
    let mut spot = use_signal(|| None::<String>);
    let agent = |id: &str,
                 name: &str,
                 model: &str,
                 parent: Option<&str>,
                 status: AgentStatus,
                 task: &str,
                 cost: f64,
                 secs: u64| FleetAgent {
        id: id.into(),
        name: name.into(),
        model_label: model.into(),
        parent_id: parent.map(Into::into),
        status,
        cost_usd: cost,
        elapsed_secs: secs,
        current_task: task.into(),
    };
    let agents = vec![
        agent(
            "planner",
            "Planner",
            "Large model",
            None,
            AgentStatus::Running,
            "Splitting the launch into three checks",
            1.42,
            1260,
        ),
        agent(
            "support",
            "Support lead",
            "Mid model",
            None,
            AgentStatus::Running,
            "Answering open tickets in the queue",
            1.02,
            1740,
        ),
        agent(
            "clerk",
            "Data clerk",
            "Small model",
            None,
            AgentStatus::Waiting,
            "Rotating the webhook secret",
            0.05,
            660,
        ),
        agent(
            "research",
            "Researcher",
            "Mid model",
            Some("planner"),
            AgentStatus::Done,
            "Comparing four rival listings",
            0.31,
            840,
        ),
        agent(
            "drafter",
            "Drafter",
            "Mid model",
            Some("planner"),
            AgentStatus::Running,
            "Writing three plan descriptions",
            0.22,
            540,
        ),
        agent(
            "publisher",
            "Publisher",
            "Small model",
            Some("planner"),
            AgentStatus::Waiting,
            "Waiting for approval to send",
            0.08,
            360,
        ),
        agent(
            "reply",
            "Reply writer",
            "Mid model",
            Some("support"),
            AgentStatus::Failed,
            "Reply draft blocked by policy check",
            0.12,
            240,
        ),
    ];
    let inbox = vec![
        InboxItem {
            id: "i1".into(),
            agent_id: "publisher".into(),
            kind: InboxKind::Decision,
            title: "Send the launch email to 240 accounts?".into(),
            detail: "The list is final and the send window opens at 10 AM.".into(),
            waiting_secs: 840,
            actions: vec![
                InboxAction {
                    id: "reject".into(),
                    label: "Reject".into(),
                },
                InboxAction {
                    id: "approve".into(),
                    label: "Approve".into(),
                },
            ],
        },
        InboxItem {
            id: "i2".into(),
            agent_id: "clerk".into(),
            kind: InboxKind::Decision,
            title: "Rotate the webhook secret now?".into(),
            detail: "The old secret stays valid for 24 hours after rotation.".into(),
            waiting_secs: 660,
            actions: vec![
                InboxAction {
                    id: "reject".into(),
                    label: "Reject".into(),
                },
                InboxAction {
                    id: "approve".into(),
                    label: "Approve".into(),
                },
            ],
        },
        InboxItem {
            id: "i3".into(),
            agent_id: "reply".into(),
            kind: InboxKind::Failure,
            title: "Reply draft blocked by policy check".into(),
            detail: "The draft quotes a price that is not in the approved list.".into(),
            waiting_secs: 360,
            actions: vec![
                InboxAction {
                    id: "retry".into(),
                    label: "Retry".into(),
                },
                InboxAction {
                    id: "dismiss".into(),
                    label: "Dismiss".into(),
                },
            ],
        },
    ];
    let event = |minutes: u64, agent: &str, outcome: Outcome| ActivityPoint {
        at: NOW_MS - minutes * MIN,
        agent_id: agent.into(),
        outcome,
    };
    let activity = vec![
        event(100, "research", Outcome::Done),
        event(80, "clerk", Outcome::NeedsYou),
        event(70, "reply", Outcome::Failed),
        event(40, "publisher", Outcome::NeedsYou),
        event(30, "support", Outcome::Done),
        event(20, "research", Outcome::Done),
        event(10, "support", Outcome::Done),
    ];
    let mut agents_sig = use_signal(|| agents.clone());
    let mut inbox_sig = use_signal(|| inbox.clone());
    rsx! {
        Item { slug: "agent-fleet-overview", name: "AgentFleetOverview",
            AgentFleetOverview {
                agents: agents_sig(),
                inbox_items: inbox_sig(),
                activity: activity,
                last_seen_at: NOW_MS - 120 * MIN,
                now_at: gallery_now(),
                heading_text: "Mission control",
                since_label: "Since 9:10 AM · 2h",
                range_labels: ("9:10 AM".to_string(), "Now".to_string()),
                status_filter: filter(),
                on_status_filter_change: move |f: StatusFilter| filter.set(f),
                spotlight_agent_id: spot(),
                on_spotlight_change: move |s: Option<String>| spot.set(s),
                on_inbox_answer: move |(item_id, _action): (String, String)| {
                    let answered = inbox_sig().into_iter().find(|i| i.id == item_id);
                    inbox_sig.write().retain(|i| i.id != item_id);
                    if let Some(item) = answered {
                        for a in agents_sig.write().iter_mut().filter(|a| a.id == item.agent_id) {
                            a.status = AgentStatus::Running;
                        }
                    }
                },
                on_clear_failed: move |id: String| {
                    agents_sig.write().retain(|a| a.id != id);
                    inbox_sig.write().retain(|i| i.agent_id != id);
                    if spot().as_deref() == Some(id.as_str()) {
                        spot.set(None);
                    }
                },
            }
        }
    }
}

#[component]
fn TeamItem() -> Element {
    use rv2_kit::blocks::agent_team_session::*;
    let mut spot = use_signal(|| None::<String>);
    let mut paused = use_signal(|| false);
    let mut sent = use_signal(Vec::<TeamMessage>::new);
    let agents = vec![
        TeamAgent {
            id: "pm".into(),
            name: "Priya".into(),
            model_label: "Opus".into(),
            role: "Plans".into(),
        },
        TeamAgent {
            id: "dev".into(),
            name: "Dev".into(),
            model_label: "Sonnet".into(),
            role: "Builds".into(),
        },
        TeamAgent {
            id: "qa".into(),
            name: "Quinn".into(),
            model_label: "Sonnet".into(),
            role: "Tests".into(),
        },
    ];
    let mut messages = vec![
        TeamMessage {
            id: "m1".into(),
            author_id: "pm".into(),
            to: Recipients::Everyone,
            reply_to_id: None,
            at_label: "09:02".into(),
            text:
                "Ship the billing screen behind a flag. Dev takes the form, Quinn the edge cases."
                    .into(),
            tool_calls: vec![],
        },
        TeamMessage {
            id: "m2".into(),
            author_id: "dev".into(),
            to: Recipients::Ids(vec!["pm".into()]),
            reply_to_id: Some("m1".into()),
            at_label: "09:05".into(),
            text: "Form is in. Two validation rules still open.".into(),
            tool_calls: vec![ToolCallCount {
                name: "read_file".into(),
                count: 4,
            }],
        },
        TeamMessage {
            id: "m3".into(),
            author_id: "qa".into(),
            to: Recipients::Ids(vec!["dev".into()]),
            reply_to_id: None,
            at_label: "09:11".into(),
            text: "Empty VAT number passes validation. Please reject it.".into(),
            tool_calls: vec![ToolCallCount {
                name: "run_tests".into(),
                count: 1,
            }],
        },
    ];
    messages.extend(sent());
    let plan = vec![
        PlanStep {
            id: "s1".into(),
            title: "Form layout".into(),
            owner_id: Some("dev".into()),
            status: StepStatus::Done,
            started_at_ms: Some(NOW_MS - 30 * MIN),
            finished_at_ms: Some(NOW_MS - 12 * MIN),
            blocked_reason: None,
        },
        PlanStep {
            id: "s2".into(),
            title: "Validation rules".into(),
            owner_id: Some("dev".into()),
            status: StepStatus::Running,
            started_at_ms: Some(NOW_MS - 12 * MIN),
            finished_at_ms: None,
            blocked_reason: None,
        },
        PlanStep {
            id: "s3".into(),
            title: "Edge-case tests".into(),
            owner_id: Some("qa".into()),
            status: StepStatus::Blocked,
            started_at_ms: None,
            finished_at_ms: None,
            blocked_reason: Some("Waiting on the VAT fixture".into()),
        },
        PlanStep {
            id: "s4".into(),
            title: "Copy review".into(),
            owner_id: Some("pm".into()),
            status: StepStatus::Pending,
            started_at_ms: None,
            finished_at_ms: None,
            blocked_reason: None,
        },
    ];
    let usage = ContextUsage {
        used_tokens: 128_000.0,
        limit_tokens: 200_000.0,
        segments: vec![
            ContextSegment {
                label: "Brief".into(),
                tokens: 12_000.0,
            },
            ContextSegment {
                label: "Files".into(),
                tokens: 96_000.0,
            },
            ContextSegment {
                label: "Chat".into(),
                tokens: 20_000.0,
            },
        ],
    };
    rsx! {
        Item { slug: "agent-team-session", name: "AgentTeamSession",
            AgentTeamSession {
                task_title: "Billing screen behind a flag",
                team_name: "Checkout team",
                agents: agents,
                messages: messages,
                plan_steps: plan,
                context_usage: usage,
                clock_ms: gallery_now(),
                is_paused: paused(),
                on_toggle_pause: move |p: bool| paused.set(p),
                spotlight_agent_id: spot(),
                on_spotlight_change: move |s: Option<String>| spot.set(s),
                on_message_submit: move |s: ComposeSubmit| {
                    sent.with_mut(|list| list.push(TeamMessage { id: format!("sent-{}", list.len()), author_id: "pm".into(), to: if s.to_ids.is_empty() { Recipients::Everyone } else { Recipients::Ids(s.to_ids) }, reply_to_id: s.reply_to_id, at_label: "now".into(), text: s.text, tool_calls: vec![] }));
                },
                stream_height_cap: 360.0,
            }
        }
    }
}

#[component]
fn TimelineItem() -> Element {
    use rv2_kit::blocks::agent_fleet_overview::{
        AgentStatus, InboxAction, InboxItem, InboxKind, StatusFilter,
    };
    use rv2_kit::blocks::agent_timeline_overview::*;
    let mut filter = use_signal(StatusFilter::default);
    let mut spot = use_signal(|| None::<String>);
    let now = gallery_now();
    let mut window = use_signal(|| None::<(u64, u64)>);
    let (from, to) = window().unwrap_or((now - 30 * MIN, now));
    let agent = |id: &str, name: &str, model: &str, status: AgentStatus| TimelineAgent {
        id: id.into(),
        name: name.into(),
        model_label: model.into(),
        status,
    };
    let agents = vec![
        agent("planner", "Planner", "Large model", AgentStatus::Running),
        agent("scout", "Scout", "Mid model", AgentStatus::Running),
        agent("drafter", "Drafter", "Mid model", AgentStatus::Done),
        agent("publisher", "Publisher", "Mid model", AgentStatus::Waiting),
        agent("support", "Support", "Small model", AgentStatus::Running),
        agent("invoicer", "Invoicer", "Small model", AgentStatus::Failed),
        agent("monitor", "Monitor", "Small model", AgentStatus::Waiting),
    ];
    let span = |id: &str,
                agent: &str,
                label: &str,
                start: u64,
                end: Option<u64>,
                status: AgentStatus,
                error: Option<&str>| TimelineSpan {
        id: id.into(),
        agent_id: agent.into(),
        label: label.into(),
        start_at: start,
        end_at: end,
        status,
        error_line: error.map(Into::into),
    };
    let spans = vec![
        span(
            "a",
            "planner",
            "Outline the launch",
            NOW_MS - 25 * MIN,
            None,
            AgentStatus::Running,
            None,
        ),
        span(
            "b",
            "scout",
            "Compare vendor prices",
            NOW_MS - 18 * MIN,
            None,
            AgentStatus::Running,
            None,
        ),
        span(
            "c",
            "drafter",
            "Draft launch copy",
            NOW_MS - 20 * MIN,
            Some(NOW_MS - 8 * MIN),
            AgentStatus::Done,
            None,
        ),
        span(
            "d",
            "publisher",
            "Stage the release",
            NOW_MS - 9 * MIN,
            None,
            AgentStatus::Waiting,
            None,
        ),
        span(
            "e",
            "support",
            "Answer 38 tickets",
            NOW_MS - 25 * MIN,
            None,
            AgentStatus::Running,
            None,
        ),
        span(
            "f",
            "invoicer",
            "Send invoice batch",
            NOW_MS - 12 * MIN,
            Some(NOW_MS - 6 * MIN),
            AgentStatus::Failed,
            Some("Ledger rejected row 212: tax code missing"),
        ),
        span(
            "g",
            "monitor",
            "Watch the error budget",
            NOW_MS - 5 * MIN,
            None,
            AgentStatus::Waiting,
            None,
        ),
    ];
    let activity = vec![
        TimelineEvent {
            at_ms: NOW_MS - 3 * MIN,
            status: AgentStatus::Done,
        },
        TimelineEvent {
            at_ms: NOW_MS - 10 * MIN,
            status: AgentStatus::Done,
        },
        TimelineEvent {
            at_ms: NOW_MS - 8 * MIN,
            status: AgentStatus::Done,
        },
        TimelineEvent {
            at_ms: NOW_MS - 15 * MIN,
            status: AgentStatus::Done,
        },
        TimelineEvent {
            at_ms: NOW_MS - 9 * MIN,
            status: AgentStatus::Waiting,
        },
        TimelineEvent {
            at_ms: NOW_MS - 5 * MIN,
            status: AgentStatus::Waiting,
        },
        TimelineEvent {
            at_ms: NOW_MS - 6 * MIN,
            status: AgentStatus::Failed,
        },
    ];
    let inbox = vec![
        InboxItem {
            id: "i1".into(),
            agent_id: "publisher".into(),
            kind: InboxKind::Decision,
            title: "Ship the release notes?".into(),
            detail: "Copy passed review. Publish to the blog?".into(),
            waiting_secs: 14 * 60,
            actions: vec![
                InboxAction {
                    id: "approve".into(),
                    label: "Approve".into(),
                },
                InboxAction {
                    id: "reject".into(),
                    label: "Reject".into(),
                },
            ],
        },
        InboxItem {
            id: "i2".into(),
            agent_id: "invoicer".into(),
            kind: InboxKind::Failure,
            title: "Invoice batch stopped".into(),
            detail: "Ledger rejected row 212: tax code missing".into(),
            waiting_secs: 6 * 60,
            actions: vec![InboxAction {
                id: "retry".into(),
                label: "Retry".into(),
            }],
        },
        InboxItem {
            id: "i3".into(),
            agent_id: "monitor".into(),
            kind: InboxKind::Decision,
            title: "Raise the alert threshold?".into(),
            detail: "The error budget is at 80% for the week.".into(),
            waiting_secs: 11 * 60,
            actions: vec![
                InboxAction {
                    id: "approve".into(),
                    label: "Approve".into(),
                },
                InboxAction {
                    id: "reject".into(),
                    label: "Reject".into(),
                },
            ],
        },
    ];
    let mut agents_sig = use_signal(|| agents.clone());
    let mut spans_sig = use_signal(|| spans.clone());
    let mut inbox_sig = use_signal(|| inbox.clone());
    rsx! {
        Item { slug: "agent-timeline-overview", name: "AgentTimelineOverview",
            AgentTimelineOverview {
                agents: agents_sig(),
                spans: spans_sig(),
                inbox_items: inbox_sig(),
                window_from: from,
                window_to: to,
                now_ms: now,
                last_seen_at: NOW_MS - 25 * MIN,
                activity: activity.clone(),
                status_filter: filter(),
                on_status_filter_change: move |f: StatusFilter| filter.set(f),
                spotlight_agent_id: spot(),
                on_spotlight_change: move |s: Option<String>| spot.set(s),
                on_inbox_answer: move |(item_id, _action): (String, String)| {
                    let owner = inbox_sig().into_iter().find(|i| i.id == item_id).map(|i| i.agent_id);
                    inbox_sig.write().retain(|i| i.id != item_id);
                    if let Some(agent_id) = owner {
                        for s in spans_sig.write().iter_mut().filter(|s| s.agent_id == agent_id && s.status == AgentStatus::Waiting) {
                            s.status = AgentStatus::Done;
                            s.end_at = Some(now);
                        }
                        for a in agents_sig.write().iter_mut().filter(|a| a.id == agent_id) {
                            a.status = AgentStatus::Done;
                        }
                    }
                },
                on_window_change: move |w: (u64, u64)| window.set(Some(w)),
            }
        }
    }
}

#[component]
fn ChatItem() -> Element {
    use rv2_kit::blocks::chart_answer_chat::*;
    let this_period: [f64; 13] = [
        1080.0, 1120.0, 1160.0, 1190.0, 1230.0, 1270.0, 1310.0, 1350.0, 1420.0, 1640.0, 1500.0,
        1480.0, 1520.0,
    ];
    let last_period: [f64; 13] = [
        1000.0, 1020.0, 1040.0, 1060.0, 1080.0, 1100.0, 1120.0, 1150.0, 1170.0, 1190.0, 1210.0,
        1230.0, 1250.0,
    ];
    let trend: Vec<TrendPoint> = this_period
        .iter()
        .zip(last_period.iter())
        .enumerate()
        .map(|(i, (value, last))| TrendPoint {
            x_label: format!("W{}", 27 + i),
            value: *value,
            comparison_value: Some(*last),
        })
        .collect();
    let messages = vec![
        ChatMessage {
            id: "q1".into(),
            role: Role::Person,
            text: Some("How did signups change over the quarter?".into()),
            blocks: vec![],
            failed: false,
        },
        ChatMessage {
            id: "a1".into(),
            role: Role::Assistant,
            text: Some("Signups closed at 18.4k, 11% ahead of last year.".into()),
            blocks: vec![
                AnswerBlock::Figures(vec![
                    Figure {
                        label: "Signups".into(),
                        value: "18.4k".into(),
                        delta_percent: 11.0,
                    },
                    Figure {
                        label: "New MRR".into(),
                        value: "$86K".into(),
                        delta_percent: 9.4,
                    },
                    Figure {
                        label: "Churn".into(),
                        value: "2.4%".into(),
                        delta_percent: -0.6,
                    },
                ]),
                AnswerBlock::Trend {
                    block_id: "t1".into(),
                    series_label: "This period".into(),
                    comparison_label: "Last period".into(),
                    points: trend,
                    hint: Some("Drag across the chart to ask about a span of weeks.".into()),
                },
            ],
            failed: false,
        },
    ];
    let prompts = [
        "Break it down by region",
        "Forecast next quarter",
        "Why did week 36 jump?",
    ]
    .iter()
    .enumerate()
    .map(|(i, label)| Prompt {
        id: format!("p{i}"),
        label: label.to_string(),
    })
    .collect::<Vec<_>>();
    rsx! {
        Item { slug: "chart-answer-chat", name: "ChartAnswerChat",
            div { style: "width: min(100%, 560px); margin: 0 auto;",
                ChartAnswerChat {
                    messages: messages,
                    suggested_prompts: prompts,
                    sources: vec!["CRM".to_string(), "Billing".to_string()],
                    on_ask: move |_: AskSubmit| {},
                }
            }
        }
    }
}

#[component]
fn WorkspaceItem() -> Element {
    use rv2_kit::blocks::client_workspace::*;
    let mut section = use_signal(|| "context".to_string());
    let mut kind = use_signal(|| None::<ItemKind>);
    const DAY: u64 = 24 * 60 * MIN;
    let items = vec![
        TimelineItem {
            id: "n1".into(),
            kind: ItemKind::Note,
            author: "Ana Ruiz".into(),
            at_ms: NOW_MS - 40 * MIN,
            title: None,
            text: "Bank lines for March are in; the Q3 cash report needs a new total.".into(),
            references: vec![Reference {
                label: "Q3 cash report".into(),
                target: "r1".into(),
            }],
        },
        TimelineItem {
            id: "r1".into(),
            kind: ItemKind::Report,
            author: "Ben Koch".into(),
            at_ms: NOW_MS - DAY,
            title: Some("Q3 cash report".into()),
            text: "Cash runway is 14 months at current spend.".into(),
            references: vec![],
        },
        TimelineItem {
            id: "f1".into(),
            kind: ItemKind::File,
            author: "Ana Ruiz".into(),
            at_ms: NOW_MS - 2 * DAY,
            title: None,
            text: "March export and bank statement.".into(),
            references: vec![
                Reference {
                    label: "march-export.csv".into(),
                    target: "file-1".into(),
                },
                Reference {
                    label: "bank-statement.pdf".into(),
                    target: "file-2".into(),
                },
            ],
        },
        TimelineItem {
            id: "n2".into(),
            kind: ItemKind::Note,
            author: "Ben Koch".into(),
            at_ms: NOW_MS - 4 * DAY,
            title: None,
            text: "Payment terms agreed; see the renewal terms.".into(),
            references: vec![Reference {
                label: "Renewal terms".into(),
                target: "r2".into(),
            }],
        },
        TimelineItem {
            id: "a1".into(),
            kind: ItemKind::App,
            author: "".into(),
            at_ms: NOW_MS - 7 * DAY,
            title: None,
            text: "Connected ledger app".into(),
            references: vec![],
        },
    ];
    let globex = vec![TimelineItem {
        id: "x1".into(),
        kind: ItemKind::Report,
        author: "Lee".into(),
        at_ms: NOW_MS - 30 * MIN,
        title: Some("Pipeline review".into()),
        text: "Two renewals slip into Q1.".into(),
        references: vec![],
    }];
    let mut client = use_signal(|| "acme".to_string());
    let mut items_by_client = use_signal(|| {
        std::collections::BTreeMap::from([
            ("acme".to_string(), items.clone()),
            ("globex".to_string(), globex),
        ])
    });
    let (ws_id, ws_name, ws_meta, cards) = if client() == "globex" {
        (
            "globex",
            "Globex",
            vec!["Project".to_string(), "USD".to_string()],
            vec![StatCard {
                id: "pipe".into(),
                label: "Pipeline".into(),
                value: "$1.2M".into(),
                delta_percent: Some(3.1),
                series: vec![1.0, 1.1, 1.3, 1.2, 1.4],
            }],
        )
    } else {
        (
            "acme",
            "Larch & Co Ltd",
            vec![
                "Berlin".to_string(),
                "EUR".to_string(),
                "year closes 31 Dec".to_string(),
            ],
            vec![
                StatCard {
                    id: "receivables".into(),
                    label: "Receivables".into(),
                    value: "€48.2k".into(),
                    delta_percent: Some(4.2),
                    series: vec![41.0, 43.5, 42.0, 45.5, 47.0, 48.2],
                },
                StatCard {
                    id: "bank".into(),
                    label: "Bank balance".into(),
                    value: "€612k".into(),
                    delta_percent: Some(-1.8),
                    series: vec![640.0, 628.0, 634.0, 620.0, 615.0, 612.0],
                },
                StatCard {
                    id: "margin".into(),
                    label: "Gross margin".into(),
                    value: "64.1%".into(),
                    delta_percent: Some(0.9),
                    series: vec![62.8, 63.1, 63.0, 63.6, 63.9, 64.1],
                },
                StatCard {
                    id: "burn".into(),
                    label: "Monthly burn".into(),
                    value: "€36.0k".into(),
                    delta_percent: Some(-3.1),
                    series: vec![38.5, 37.9, 37.2, 37.6, 36.8, 36.0],
                },
            ],
        )
    };
    rsx! {
        Item { slug: "client-workspace", name: "ClientWorkspace",
            ClientWorkspace {
                workspace: WorkspaceInfo { id: ws_id.into(), name: ws_name.into(), meta: ws_meta, synced_at_ms: NOW_MS - 3 * MIN },
                workspace_options: vec![WorkspaceOption { id: "acme".into(), name: "Larch & Co Ltd".into() }, WorkspaceOption { id: "globex".into(), name: "Globex".into() }],
                stat_cards: cards,
                sections: vec![
                    WorkspaceSection { id: "context".into(), label: "Context".into(), count: None },
                    WorkspaceSection { id: "reports".into(), label: "Reports".into(), count: None },
                    WorkspaceSection { id: "financials".into(), label: "Financials".into(), count: None },
                    WorkspaceSection { id: "settings".into(), label: "Settings".into(), count: None },
                ],
                active_section: section(),
                on_section_change: move |s: String| section.set(s),
                timeline_items: items_by_client().get(&client()).cloned().unwrap_or_default(),
                kind_options: ItemKind::ALL.to_vec(),
                selected_kind: kind(),
                on_kind_change: move |k: Option<ItemKind>| kind.set(k),
                now_ms: gallery_now(),
                on_switch_workspace: move |id: String| client.set(id),
                on_submit_note: move |note: NoteSubmit| {
                    let key = client();
                    let mut map = items_by_client.write();
                    let list = map.entry(key).or_default();
                    let next = list.len();
                    list.push(TimelineItem {
                        id: format!("s{next}"),
                        kind: ItemKind::Note,
                        author: "You".into(),
                        at_ms: gallery_now(),
                        title: None,
                        text: note.text,
                        references: note.references,
                    });
                },
            }
        }
    }
}

#[component]
fn WorkspaceLoadingItem() -> Element {
    use rv2_kit::blocks::client_workspace::*;
    rsx! {
        Item { slug: "client-workspace-loading", name: "ClientWorkspace (loading)",
            ClientWorkspace {
                workspace: WorkspaceInfo { id: "acme".into(), name: "Acme Ltd".into(), meta: vec![], synced_at_ms: NOW_MS },
                workspace_options: vec![WorkspaceOption { id: "acme".into(), name: "Acme Ltd".into() }],
                stat_cards: vec![],
                sections: vec![WorkspaceSection { id: "context".into(), label: "Context".into(), count: None }],
                timeline_items: vec![],
                kind_options: vec![ItemKind::Goal],
                now_ms: gallery_now(),
                is_loading: true,
            }
        }
    }
}

#[component]
fn StripItem() -> Element {
    use rv2_affect::button::{Button, ButtonSize, ButtonVariant};
    use rv2_kit::blocks::expandable_panel_strip::*;
    let mut open = use_signal(|| Some("renewal".to_string()));
    let preview = |title: &str| rsx! { div { style: "padding: 24px; font: 15px system-ui", "{title} preview" } };
    let sparkline = |points: &str| {
        rsx! {
            svg { width: "60", height: "20", view_box: "0 0 60 20", "aria-hidden": "true",
                polyline { points: "{points}", fill: "none", stroke: "currentColor", "stroke-width": "1.5" }
            }
        }
    };
    let renewal = rsx! {
        div { style: "display: grid; gap: 14px; padding: 18px",
            div { style: "display: flex; align-items: center; gap: 8px; flex-wrap: wrap",
                span { style: "width: 12px; height: 12px; border-radius: 3px; background: currentColor", "aria-hidden": "true" }
                span { style: "font-weight: 600", "Renewal watch" }
                Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, label: "New agent", onclick: move |_| {} }
                span { style: "margin-left: auto; font-size: 12px", "SO" }
            }
            div { style: "display: grid; gap: 4px",
                h3 { style: "margin: 0; font-size: 20px", "Renewal watch" }
                p { style: "margin: 0; font-size: 13px", "Updated 3 min ago" }
                p { style: "margin: 0; font-size: 13px", "Starts when a plan has 30 days left · Owner Sam Ortega" }
            }
            div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(110px, 1fr)); gap: 8px",
                div { span { style: "display: block; font-size: 12px", "Runs" } strong { "126" } {sparkline("0,15 12,12 24,13 36,8 48,9 60,4")} }
                div { span { style: "display: block; font-size: 12px", "Success" } strong { "94%" } {sparkline("0,8 12,9 24,6 36,7 48,4 60,5")} }
                div { span { style: "display: block; font-size: 12px", "Hours saved" } strong { "2.8" } {sparkline("0,16 12,13 24,12 36,9 48,7 60,6")} }
                div { span { style: "display: block; font-size: 12px", "Escalated" } strong { "9" } {sparkline("0,5 12,8 24,6 36,11 48,10 60,14")} }
            }
            div { role: "tablist", "aria-label": "Agent sections", style: "display: flex; gap: 6px; flex-wrap: wrap",
                button { role: "tab", "aria-selected": "true", "Context" }
                button { role: "tab", "aria-selected": "false", "Steps" }
                button { role: "tab", "aria-selected": "false", "Tools" }
                button { role: "tab", "aria-selected": "false", "Limits" }
            }
            section { style: "display: grid; gap: 8px",
                h4 { style: "margin: 0; font-size: 15px", "Agent context" }
                p { style: "margin: 0; font-size: 13px", "Rules and facts the agent reads before each run." }
                div { style: "display: flex; gap: 8px; flex-wrap: wrap",
                    input { r#type: "search", "aria-label": "Search context", placeholder: "Search context", style: "flex: 1 1 180px; min-width: 0" }
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, label: "Filter", onclick: move |_| {} }
                }
                div { style: "display: grid; gap: 6px",
                    p { style: "margin: 0; font-size: 13px", strong { "SO " } "Renewals above the plan threshold go to the account owner first." }
                    p { style: "margin: 0; font-size: 13px", strong { "PN " } "Sam added the regional price list on 2 Nov." }
                }
                div { style: "display: flex; gap: 8px; flex-wrap: wrap",
                    input { r#type: "text", "aria-label": "New context", placeholder: "Add a note", style: "flex: 1 1 180px; min-width: 0" }
                    Button { variant: ButtonVariant::Primary, size: ButtonSize::Sm, label: "Add context", onclick: move |_| {} }
                }
            }
        }
    };
    let panels = vec![
        StripPanel {
            id: "triage".into(),
            title: "Ticket triage".into(),
            caption: "Sorts new tickets by urgency and routes them.".into(),
            preview: preview("Ticket triage"),
        },
        StripPanel {
            id: "leads".into(),
            title: "Lead finder".into(),
            caption: "Finds accounts that match the ideal customer profile.".into(),
            preview: preview("Lead finder"),
        },
        StripPanel {
            id: "notes".into(),
            title: "Call notes".into(),
            caption: "Writes a summary after each sales call.".into(),
            preview: preview("Call notes"),
        },
        StripPanel {
            id: "renewal".into(),
            title: "Renewal watch".into(),
            caption: "Flags quiet accounts before their contract ends and drafts the first call."
                .into(),
            preview: renewal,
        },
        StripPanel {
            id: "onboarding".into(),
            title: "Onboarding guide".into(),
            caption: "Walks new customers through setup, step by step.".into(),
            preview: preview("Onboarding guide"),
        },
        StripPanel {
            id: "copy".into(),
            title: "Copy check".into(),
            caption: "Checks product copy against the style guide.".into(),
            preview: preview("Copy check"),
        },
        StripPanel {
            id: "changelog".into(),
            title: "Changelog writer".into(),
            caption: "Drafts release notes from merged changes.".into(),
            preview: preview("Changelog writer"),
        },
    ];
    rsx! {
        Item { slug: "expandable-panel-strip", name: "ExpandablePanelStrip",
            ExpandablePanelStrip {
                panels: panels,
                heading: "Specialist agents, one at a time".to_string(),
                intro: "Each agent has its own steps, tools and limits.".to_string(),
                open_id: open(),
                on_open_change: move |id: Option<String>| open.set(id),
                preview_label: "Agent",
            }
        }
    }
}

#[component]
fn CatalogItem() -> Element {
    use rv2_kit::blocks::integration_catalog::*;
    let mut search = use_signal(String::new);
    let mut connected = use_signal(|| vec!["chatroom".to_string()]);
    let mut pending = use_signal(|| None::<String>);
    let mk = |id: &str, name: &str, vendor: &str, desc: &str, cats: &[&str], badges: &[&str]| {
        Integration {
            id: id.into(),
            name: name.into(),
            vendor: vendor.into(),
            description: desc.into(),
            category_ids: cats.iter().map(|c| c.to_string()).collect(),
            connected: connected().iter().any(|c| c == id),
            logo: rsx! { span { "{name.chars().next().unwrap_or('?')}" } },
            badges: badges.iter().map(|b| b.to_string()).collect(),
            error: None,
        }
    };
    let items = vec![
        mk(
            "chatroom",
            "Chatroom",
            "Ardent",
            "Get alerts in your channels and reply without leaving the thread.",
            &["communication"],
            &[],
        ),
        mk(
            "docs",
            "Docs hub",
            "Quillo",
            "Turn records into pages and keep project notes in step.",
            &["docs"],
            &[],
        ),
        mk(
            "tracker",
            "Tracker",
            "Brightline",
            "Turn customer feedback into work items and follow them to release.",
            &["product"],
            &[],
        ),
        mk(
            "design",
            "Design board",
            "Kestrel",
            "Preview mockups inline and pin each frame to its ticket.",
            &["design"],
            &[],
        ),
        mk(
            "codehost",
            "Code host",
            "Halden",
            "Link changes and show build status beside each task.",
            &["development"],
            &[],
        ),
        mk(
            "payments",
            "Payments",
            "Oriel",
            "Sync customers, invoices and renewals as they change.",
            &["billing"],
            &[],
        ),
        mk(
            "automation",
            "Automation hub",
            "Veyra",
            "Connect thousands of apps with no-code triggers.",
            &["automation"],
            &[],
        ),
        mk(
            "contacts",
            "Contacts",
            "Lumen",
            "Keep people and companies matched across both tools.",
            &["crm"],
            &[],
        ),
        mk(
            "metrics",
            "Metrics Lab",
            "Fenwick",
            "Chart product usage by team and by week.",
            &["analytics"],
            &[],
        ),
        mk(
            "pulse",
            "Pulse Board",
            "Garnet",
            "Watch key numbers update live on one shared page.",
            &["analytics"],
            &[],
        ),
        mk(
            "funnel",
            "Funnel Map",
            "Ostrava",
            "See where visitors drop out between two steps.",
            &["analytics", "product"],
            &[],
        ),
        mk(
            "relay",
            "Relay Hub",
            "Tallow",
            "Forward messages between tools and keep replies in one place.",
            &["communication", "scheduling"],
            &[],
        ),
        mk(
            "notify",
            "Notify Hub",
            "Brook",
            "Send alerts to the right people on their preferred channel.",
            &["communication"],
            &[],
        ),
        mk(
            "thread",
            "Thread Notes",
            "Marlow",
            "Turn long threads into a shared summary every morning.",
            &["communication"],
            &[],
        ),
        mk(
            "pager",
            "Pager Bridge",
            "Quarry",
            "Escalate urgent messages to on-call staff by phone.",
            &["communication", "scheduling"],
            &[],
        ),
        mk(
            "slots",
            "Slot Booker",
            "Heron",
            "Let customers book times that match your team's calendars.",
            &["scheduling"],
            &["Free"],
        ),
        mk(
            "leads",
            "Lead Kit",
            "Vesper",
            "Capture new leads from forms and assign them automatically.",
            &["crm"],
            &[],
        ),
        mk(
            "deals",
            "Deal Tracker",
            "Cobalt",
            "Follow deals through each stage and forecast revenue.",
            &["crm", "billing"],
            &[],
        ),
        mk(
            "mockups",
            "Mock Tool",
            "Pale",
            "Sketch screens together and share a live prototype link.",
            &["design"],
            &[],
        ),
        mk(
            "builds",
            "Build Runner",
            "Anvil",
            "Run tests on every change and report results in the review.",
            &["development"],
            &["Team"],
        ),
        mk(
            "packages",
            "Package Hub",
            "Lattice",
            "Publish builds and keep versioned files in one library.",
            &["development", "storage"],
            &[],
        ),
        mk(
            "vault",
            "Drive Vault",
            "Sable",
            "Store large files with access rules and version history.",
            &["storage", "development"],
            &[],
        ),
    ];
    rsx! {
        Item { slug: "integration-catalog", name: "IntegrationCatalog",
            IntegrationCatalog {
                integrations: items,
                categories: [
                    ("analytics", "Analytics"), ("automation", "Automation"), ("billing", "Billing"),
                    ("communication", "Communication"), ("crm", "CRM"), ("design", "Design"),
                    ("development", "Development"), ("docs", "Docs"), ("product", "Product"),
                    ("scheduling", "Scheduling"), ("storage", "Storage"),
                ].into_iter().map(|(id, label)| CategoryOption { id: id.into(), label: label.into() }).collect::<Vec<_>>(),
                featured_ids: vec!["chatroom".into(), "docs".into(), "tracker".into(), "design".into(), "codehost".into(), "payments".into(), "automation".into(), "contacts".into()],
                read_more_href: "#".to_string(),
                pending_id: pending(),
                on_connect: move |id: String| {
                    pending.set(Some(id.clone()));
                    spawn(async move {
                        let _ = document::eval("await new Promise((resolve) => setTimeout(resolve, 1600)); return true;").await;
                        connected.with_mut(|c| c.push(id));
                        pending.set(None);
                    });
                },
                on_disconnect: move |id: String| connected.with_mut(|c| c.retain(|x| *x != id)),
                filter_options: vec![FilterOption { id: "Free".into(), label: "Free".into() }, FilterOption { id: "Team".into(), label: "Team plan".into() }],
                search_text: search(),
                on_search_change: move |s: String| search.set(s),
            }
        }
    }
}

#[component]
fn InvoiceItem() -> Element {
    use rv2_kit::blocks::invoice_credit_pair::*;
    let mut side = use_signal(|| Side::Invoice);
    let mut credit = use_signal(|| None::<CreditDocument>);
    let party = |name: &str| Party {
        name: name.into(),
        address: vec!["12 Harbour Road".into(), "Dublin 2".into()],
        registration_id: Some("IE 4411".into()),
    };
    let invoice = Invoice {
        number: "INV-2107".into(),
        issued_on: "1 Oct 2026".into(),
        due_on: "31 Oct 2026".into(),
        period_from: "1 Sep 2026".into(),
        period_to: "30 Sep 2026".into(),
        currency: "EUR".into(),
        issuer: party("Marlow Works Ltd"),
        recipient: party("Larch & Co Ltd"),
        tax_percent: Some(20.0),
        status: InvoiceStatus::Open,
        lines: vec![
            Line {
                id: "l1".into(),
                name: "Team plan".into(),
                quantity: 1.0,
                unit_amount: 980.0,
                tax_percent: 20.0,
                total: 980.0,
                kind: LineKind::Fixed,
            },
            Line {
                id: "l2".into(),
                name: "Extra storage".into(),
                quantity: 4.0,
                unit_amount: 25.0,
                tax_percent: 20.0,
                total: 100.0,
                kind: LineKind::Fixed,
            },
            Line {
                id: "l3".into(),
                name: "Image renders".into(),
                quantity: 9_640.0,
                unit_amount: 0.30,
                tax_percent: 20.0,
                total: 2_892.0,
                kind: LineKind::Metered,
            },
        ],
    };
    let doc = credit();
    rsx! {
        Item { slug: "invoice-credit-pair", name: "InvoiceCreditPair",
            InvoiceCreditPair {
                invoice: invoice,
                credit_document: doc,
                active_side: side(),
                on_side_change: move |s: Side| side.set(s),
                on_credit_change: move |_: Vec<CreditedLine>| {},
                on_issue: move |d: CreditDocument| credit.set(Some(d)),
                on_download: move |_: ()| {},
            }
        }
    }
}

#[component]
fn KanbanItem() -> Element {
    use rv2_kit::blocks::kanban_task_board::*;
    let columns = vec![
        StatusColumn {
            id: "todo".into(),
            label: "Todo".into(),
        },
        StatusColumn {
            id: "review".into(),
            label: "In review".into(),
        },
        StatusColumn {
            id: "progress".into(),
            label: "In progress".into(),
        },
        StatusColumn {
            id: "done".into(),
            label: "Done".into(),
        },
    ];
    let card = |id: &str,
                identifier: &str,
                title: &str,
                status: &str,
                labels: &[&str],
                due: Option<i64>,
                subtasks: (u32, u32),
                pr: Option<&str>,
                assignee: Option<&str>|
     -> WorkItem {
        WorkItem {
            id: id.into(),
            identifier_line: identifier.into(),
            title: title.into(),
            status_id: status.into(),
            priority: 2,
            label_ids: labels.iter().map(|l| l.to_string()).collect(),
            assignee_id: assignee.map(Into::into),
            due_in_days: due,
            subtasks_done: subtasks.0,
            subtasks_total: subtasks.1,
            pull_request_ref: pr.map(Into::into),
        }
    };
    let items = vec![
        card(
            "ST-214",
            "ST-214 · Size M · Created 3 Oct",
            "Add the spring banner to the home page",
            "todo",
            &["web"],
            Some(2),
            (0, 0),
            None,
            Some("ana"),
        ),
        card(
            "ST-209",
            "ST-209 · Created 1 Oct",
            "Fix rounding on annual invoices",
            "todo",
            &["payments", "bug"],
            None,
            (0, 0),
            None,
            None,
        ),
        card(
            "ST-197",
            "ST-197 · Created 28 Sep",
            "Ask beta shops about the new tier",
            "todo",
            &[],
            Some(2),
            (2, 4),
            None,
            Some("ben"),
        ),
        card(
            "ST-221",
            "ST-221 · Created 5 Oct",
            "Tighten the onboarding copy",
            "review",
            &["web"],
            Some(1),
            (0, 0),
            None,
            Some("ana"),
        ),
        card(
            "ST-205",
            "ST-205 · Size L · Created 2 Oct",
            "Cap stock alerts at one per day",
            "review",
            &["web", "feature"],
            None,
            (0, 0),
            None,
            None,
        ),
        card(
            "ST-188",
            "ST-188 · Created 25 Sep",
            "Push opt-in on iOS",
            "review",
            &["app"],
            None,
            (0, 0),
            Some("#412"),
            Some("ben"),
        ),
        card(
            "ST-230",
            "ST-230 · Created 6 Oct",
            "Write the help-centre export guide",
            "progress",
            &["web"],
            None,
            (1, 3),
            None,
            Some("ana"),
        ),
        card(
            "ST-201",
            "ST-201 · Created 30 Sep",
            "Prorate seat changes mid-month",
            "progress",
            &["payments"],
            Some(4),
            (0, 0),
            None,
            None,
        ),
        card(
            "ST-176",
            "ST-176 · Size XL · Created 20 Sep",
            "Offline mode for the task list",
            "progress",
            &["app"],
            None,
            (3, 5),
            Some("#398"),
            Some("ben"),
        ),
        card(
            "ST-160",
            "ST-160 · Created 12 Sep",
            "Dark mode for the dashboard",
            "done",
            &["web", "feature"],
            None,
            (0, 0),
            None,
            Some("ana"),
        ),
        card(
            "ST-171",
            "ST-171 · Created 14 Sep",
            "Crash on large avatar upload",
            "done",
            &["app", "bug"],
            None,
            (0, 0),
            None,
            None,
        ),
    ];
    rsx! {
        Item { slug: "kanban-task-board", name: "KanbanTaskBoard",
            KanbanTaskBoard {
                breadcrumb_segments: vec!["Meridian".into(), "Storefront".into(), "Sprint 9".into()],
                board_name: "Storefront",
                status_columns: columns,
                work_items: items,
                tag_options: vec![
                    TagOption { id: "web".into(), label: "Web".into() },
                    TagOption { id: "payments".into(), label: "Payments".into() },
                    TagOption { id: "app".into(), label: "App".into() },
                    TagOption { id: "bug".into(), label: "Bug".into() },
                    TagOption { id: "feature".into(), label: "Feature".into() },
                ],
                member_options: vec![
                    MemberOption { id: "ana".into(), name: "Ana Lopez".into(), avatar: "AL".into() },
                    MemberOption { id: "ben".into(), name: "Ben Ortiz".into(), avatar: "BO".into() },
                ],
                on_items_update: move |_: Vec<WorkItem>| {},
            }
        }
    }
}

#[component]
fn OverviewItem() -> Element {
    use rv2_kit::blocks::metric_overview_board::*;
    let mut period = use_signal(|| "30d".to_string());
    let mut segment = use_signal(|| ALL_SEGMENTS.to_string());
    let axis: Vec<String> = (1..=30)
        .map(|d| match d {
            1 => "1 Nov".to_string(),
            30 => "Today".to_string(),
            _ => format!("{d} Nov"),
        })
        .collect();
    // A rising daily shape with a small wave, scaled so its sum is the period total.
    let fit = |start: f64, end: f64, phase: f64, total: f64| -> Vec<f64> {
        let raw: Vec<f64> = (0..30)
            .map(|i| {
                let t = i as f64 / 29.0;
                start + (end - start) * t + 0.04 * start * (i as f64 * 0.9 + phase).sin()
            })
            .collect();
        let sum: f64 = raw.iter().sum();
        raw.iter().map(|v| v * total / sum).collect()
    };
    let cards = vec![
        StatMetric {
            id: "rev".into(),
            label: "Revenue".into(),
            total: 42_100.0,
            format_total: rv2_kit::blocks::sales_metrics_panel::dollars_compact,
            current_series: fit(1_100.0, 1_500.0, 0.0, 42_100.0),
            previous_series: fit(1_000.0, 1_350.0, 0.0, 42_100.0 / 1.084),
            lower_is_better: false,
        },
        StatMetric {
            id: "signups".into(),
            label: "Sign-ups".into(),
            total: 1_914.0,
            format_total: rv2_kit::components::series_trend_chart::group_digits,
            current_series: fit(52.0, 75.0, 1.0, 1_914.0),
            previous_series: fit(47.0, 68.0, 1.0, 1_914.0 / 1.099),
            lower_is_better: false,
        },
        StatMetric {
            id: "refunds".into(),
            label: "Refunds".into(),
            total: 107.0,
            format_total: rv2_kit::components::series_trend_chart::group_digits,
            current_series: fit(3.0, 4.0, 2.0, 107.0),
            previous_series: fit(4.5, 5.5, 2.0, 107.0 / 0.743),
            lower_is_better: true,
        },
    ];
    let overview = vec![
        OverviewSeries {
            label: "Desk license".into(),
            values: fit(330.0, 420.0, 0.5, 11_200.0),
        },
        OverviewSeries {
            label: "Add-ons".into(),
            values: fit(230.0, 280.0, 2.5, 7_400.0),
        },
    ];
    let overview_previous = fit(520.0, 600.0, 1.5, 18_600.0 / 1.062);
    let product = ProductCard {
        name: "Desk license".into(),
        tag: "One-time".into(),
        figure_label: "Seats sold".into(),
        figure_value: "438".into(),
        delta_percent: 6.3,
        series: (0..30).map(|i| 400.0 + 38.0 * i as f64 / 29.0).collect(),
        comparisons: vec![
            ("Last period".into(), "412".into()),
            ("This period".into(), "438".into()),
        ],
    };
    rsx! {
        Item { slug: "metric-overview-board", name: "MetricOverviewBoard",
            MetricOverviewBoard {
                stat_cards: cards,
                overview_series: overview,
                overview_previous,
                overview_title: "Sales overview",
                axis_labels: axis,
                period_options: vec![
                    PickOption { id: "7d".into(), label: "Last 7 days".into() },
                    PickOption { id: "30d".into(), label: "Last 30 days".into() },
                    PickOption { id: "90d".into(), label: "Last 90 days".into() },
                ],
                selected_period: period(),
                on_period_change: move |p: String| period.set(p),
                period_mark: rsx! {
                    svg { width: "14", height: "14", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", "aria-hidden": "true",
                        rect { x: "2", y: "3", width: "12", height: "11", rx: "2" }
                        path { d: "M2 6.5h12M5.5 1.5v3M10.5 1.5v3" }
                    }
                },
                segment_options: vec![
                    PickOption { id: ALL_SEGMENTS.into(), label: "All items".into() },
                    PickOption { id: "Desk license".into(), label: "Desk license".into() },
                    PickOption { id: "Add-ons".into(), label: "Add-ons".into() },
                ],
                selected_segment: segment(),
                on_segment_change: move |s: String| segment.set(s),
                heading: "Dashboard".to_string(),
                format_amount: rv2_kit::blocks::sales_metrics_panel::dollars_compact,
                product_card: product,
                setup_action: ("Set up".to_string(), EventHandler::new(|_| {})),
                footer_action: ("See full analytics".to_string(), EventHandler::new(|_| {})),
            }
        }
    }
}

#[component]
fn PlanItem() -> Element {
    use rv2_kit::blocks::plan_comparison_section::*;
    let mut billing = use_signal(|| MONTHLY.to_string());
    // Feature rows with inclusion for Solo, Team, Studio.
    const FEATURES: [(&str, [bool; 3]); 7] = [
        ("Unlimited boards", [false, true, true]),
        ("Custom domain", [false, true, true]),
        ("API access", [false, true, true]),
        ("Scheduled exports", [true, true, true]),
        ("Single sign-on", [false, false, true]),
        ("Dedicated onboarding", [false, false, true]),
        ("Email support", [true, true, true]),
    ];
    let features = |col: usize| -> Vec<PlanFeature> {
        FEATURES
            .iter()
            .map(|(label, included)| PlanFeature {
                label: label.to_string(),
                included: included[col],
            })
            .collect()
    };
    let limits = |readers: &str, seats: &str, workspaces: &str| {
        vec![
            PlanLimit {
                label: "Monthly readers".into(),
                value: readers.into(),
            },
            PlanLimit {
                label: "Seats".into(),
                value: seats.into(),
            },
            PlanLimit {
                label: "Workspaces".into(),
                value: workspaces.into(),
            },
        ]
    };
    let integrations = |sync: &str, selection: &str| PlanIntegrations {
        names: vec!["Chatroom".into(), "Tracker".into(), "Docs hub".into()],
        sync_label: sync.into(),
        selection_label: selection.into(),
    };
    let plans = vec![
        Plan {
            id: "solo".into(),
            name: "Solo".into(),
            price_monthly: 19.0,
            price_yearly: 15.0,
            billing_note_monthly: "Billed monthly".into(),
            billing_note_yearly: "Billed yearly at $180".into(),
            cta_label: "Start free trial".into(),
            cta_href: "#".into(),
            highlighted: false,
            badge: None,
            limits: limits("5k", "2", "1"),
            integrations: integrations("Sync daily", "Choose 2 of 8"),
            features: features(0),
        },
        Plan {
            id: "team".into(),
            name: "Team".into(),
            price_monthly: 59.0,
            price_yearly: 47.0,
            billing_note_monthly: "Billed monthly".into(),
            billing_note_yearly: "Billed yearly at $564".into(),
            cta_label: "Start free trial".into(),
            cta_href: "#".into(),
            highlighted: true,
            badge: Some("Most chosen".into()),
            limits: limits("50k", "10", "3"),
            integrations: integrations("Sync hourly", "Choose 4 of 8"),
            features: features(1),
        },
        Plan {
            id: "studio".into(),
            name: "Studio".into(),
            price_monthly: 149.0,
            price_yearly: 119.0,
            billing_note_monthly: "Billed monthly".into(),
            billing_note_yearly: "Billed yearly at $1,428".into(),
            cta_label: "Start free trial".into(),
            cta_href: "#".into(),
            highlighted: false,
            badge: None,
            limits: limits("500k", "Unlimited", "10"),
            integrations: integrations("Sync live", "Choose 8 of 8"),
            features: features(2),
        },
    ];
    rsx! {
        Item { slug: "plan-comparison-section", name: "PlanComparisonSection",
            PlanComparisonSection {
                plans: plans,
                heading: "Pick the plan that fits your team",
                intro: "Every plan includes scheduled exports and email support.",
                yearly_saving_label: "Save 15%".to_string(),
                selected_billing: billing(),
                on_billing_change: move |b: String| billing.set(b),
            }
        }
    }
}

#[component]
fn RunItem() -> Element {
    use rv2_kit::blocks::run_tree_inspector::*;
    let mut selected = use_signal(|| None::<String>);
    let mut running_secs = use_signal(|| 8.0_f64);
    use_hook(move || {
        spawn(async move {
            loop {
                let _ = document::eval(
                    "await new Promise((resolve) => setTimeout(resolve, 1000)); return true;",
                )
                .await;
                running_secs.with_mut(|s| *s += 1.0);
            }
        });
    });
    let run = RunInfo {
        id: "run-41".into(),
        title: "Ship the spring newsletter".into(),
        status: RunStatus::Running,
        duration_secs: 520.0,
        cost_usd: 0.31,
        token_count: 96_000,
        model_call_count: 4,
        tool_call_count: 6,
    };
    let agent = |id: &str,
                 name: &str,
                 model: &str,
                 parent: Option<&str>,
                 cost: f64,
                 elapsed: f64,
                 faded: bool| RunAgent {
        id: id.into(),
        name: name.into(),
        model_label: model.into(),
        status: RunStatus::Running,
        parent_id: parent.map(Into::into),
        cost_usd: cost,
        elapsed_secs: elapsed,
        current_step: None,
        faded,
    };
    let agents = vec![
        agent(
            "planner",
            "Planner",
            "Large model",
            None,
            0.19,
            520.0,
            false,
        ),
        agent(
            "budget",
            "Budget checker",
            "Small model",
            None,
            0.03,
            520.0,
            false,
        ),
        agent(
            "research",
            "Researcher",
            "Mid model",
            Some("planner"),
            0.09,
            300.0,
            false,
        ),
        agent(
            "writer",
            "Writer",
            "Mid model",
            Some("planner"),
            0.06,
            240.0,
            true,
        ),
    ];
    let call = |id: &str,
                agent: &str,
                kind: CallKind,
                name: &str,
                status: CallStatus,
                secs: f64,
                error: Option<&str>| RunCall {
        id: id.into(),
        agent_id: agent.into(),
        kind,
        name: name.into(),
        status,
        duration_secs: secs,
        error_message: error.map(Into::into),
    };
    let calls = vec![
        call(
            "c1",
            "planner",
            CallKind::Model,
            "Draft the send plan",
            CallStatus::Ok,
            96.0,
            None,
        ),
        call(
            "c2",
            "budget",
            CallKind::Tool,
            "Read the spend limit",
            CallStatus::Ok,
            3.2,
            None,
        ),
        call(
            "c3",
            "research",
            CallKind::Tool,
            "Search price pages",
            CallStatus::Running,
            running_secs(),
            None,
        ),
        call(
            "c4",
            "research",
            CallKind::Tool,
            "Search partner terms",
            CallStatus::Running,
            running_secs(),
            None,
        ),
        call(
            "c5",
            "research",
            CallKind::Tool,
            "Query price history",
            CallStatus::Running,
            running_secs(),
            None,
        ),
        call(
            "c6",
            "writer",
            CallKind::Tool,
            "Read the style guide",
            CallStatus::Running,
            running_secs(),
            None,
        ),
        call(
            "c7",
            "writer",
            CallKind::Tool,
            "Read the price frames",
            CallStatus::Running,
            running_secs(),
            None,
        ),
    ];
    rsx! {
        Item { slug: "run-tree-inspector", name: "RunTreeInspector",
            RunTreeInspector {
                run: run,
                agents: agents,
                calls: calls,
                selected_call_id: selected(),
                on_select_call: move |id: Option<String>| selected.set(id),
            }
        }
    }
}

#[component]
fn SalesItem() -> Element {
    use rv2_kit::blocks::sales_metrics_panel::*;
    let mut metric = use_signal(|| None::<String>);
    let mut period = use_signal(|| "month".to_string());
    let mut style = use_signal(|| ChartStyle::Line);
    let mut compare = use_signal(|| false);
    let mut exported = use_signal(String::new);
    let cards = vec![
        SalesMetric {
            id: "rev".into(),
            label: "Revenue".into(),
            value: "$48.2k".into(),
            delta_percent: 12.4,
            delta_in_points: false,
            lower_is_better: false,
            series: vec![3.0, 4.0, 3.8, 5.0, 6.0, 5.5, 7.0],
        },
        SalesMetric {
            id: "orders".into(),
            label: "Orders".into(),
            value: "612".into(),
            delta_percent: -3.1,
            delta_in_points: false,
            lower_is_better: false,
            series: vec![90.0, 84.0, 88.0, 80.0, 86.0, 82.0, 79.0],
        },
        SalesMetric {
            id: "aov".into(),
            label: "Average order".into(),
            value: "$78.80".into(),
            delta_percent: 2.2,
            delta_in_points: false,
            lower_is_better: false,
            series: vec![76.0, 77.0, 77.5, 78.0, 78.2, 78.5, 78.8],
        },
        SalesMetric {
            id: "refunds".into(),
            label: "Refund rate".into(),
            value: "1.8%".into(),
            delta_percent: -0.3,
            delta_in_points: true,
            lower_is_better: true,
            series: vec![2.4, 2.2, 2.3, 2.1, 2.0, 1.9, 1.8],
        },
    ];
    // Daily revenue in dollars, 1 to 21 Nov; sums to the Revenue card's $48.2k.
    let daily: [f64; 21] = [
        1800.0, 1950.0, 2100.0, 2650.0, 2200.0, 2000.0, 1800.0, 1780.0, 2000.0, 2300.0, 2900.0,
        3620.0, 3000.0, 2600.0, 2250.0, 2200.0, 2150.0, 2250.0, 2550.0, 2300.0, 1800.0,
    ];
    let trend: Vec<TrendPoint> = daily
        .iter()
        .enumerate()
        .map(|(i, value)| TrendPoint {
            date: format!("Nov {}", i + 1),
            value: *value,
        })
        .collect();
    let previous: Vec<f64> = daily.iter().map(|v| v * 0.9).collect();
    rsx! {
        Item { slug: "sales-metrics-panel", name: "SalesMetricsPanel",
            div {
            SalesMetricsPanel {
                title: "Sales",
                subtitle: "Kiosk network · Last 30 days",
                metric_cards: cards,
                selected_metric: metric(),
                on_metric_change: move |m: String| metric.set(Some(m)),
                periods: vec![
                    PeriodOption { id: "week".into(), label: "Week".into() },
                    PeriodOption { id: "month".into(), label: "Month".into() },
                    PeriodOption { id: "quarter".into(), label: "Quarter".into() },
                ],
                selected_period: period(),
                on_period_change: move |p: String| period.set(p),
                trend_series: trend,
                previous_series: previous,
                chart_style: style(),
                on_chart_style_change: move |s: ChartStyle| style.set(s),
                compare_enabled: compare(),
                on_compare_change: move |c: bool| compare.set(c),
                top_products: vec![
                    Ranked { id: "trail".into(), name: "Trail jacket".into(), value: 12_100.0 },
                    Ranked { id: "rain".into(), name: "Rain shell".into(), value: 9_400.0 },
                    Ranked { id: "fleece".into(), name: "Fleece vest".into(), value: 8_200.0 },
                    Ranked { id: "socks".into(), name: "Wool socks".into(), value: 4_300.0 },
                    Ranked { id: "cap".into(), name: "Canvas cap".into(), value: 2_900.0 },
                ],
                channels: vec![
                    Ranked { id: "web".into(), name: "Web shop".into(), value: 21_700.0 },
                    Ranked { id: "partner".into(), name: "Partner stores".into(), value: 11_400.0 },
                    Ranked { id: "market".into(), name: "Marketplace".into(), value: 8_900.0 },
                    Ranked { id: "phone".into(), name: "Phone orders".into(), value: 3_600.0 },
                    Ranked { id: "events".into(), name: "Events".into(), value: 2_600.0 },
                ],
                on_export: move |f: ExportFormat| exported.set(format!("Exported as {}", f.label())),
            }
            p { role: "status", "aria-live": "polite", "{exported}" }
            }
        }
    }
}
