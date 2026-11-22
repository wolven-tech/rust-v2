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
            "lead",
            "Planner",
            "Opus",
            None,
            AgentStatus::Running,
            "Splitting the release into three checks",
            4.82,
            1260,
        ),
        agent(
            "api",
            "API tests",
            "Sonnet",
            Some("lead"),
            AgentStatus::Running,
            "Running contract tests",
            0.91,
            420,
        ),
        agent(
            "ui",
            "Layout check",
            "Sonnet",
            Some("lead"),
            AgentStatus::Failed,
            "Screenshot diff failed at 390 px",
            0.37,
            610,
        ),
        agent(
            "docs",
            "Changelog",
            "Haiku",
            None,
            AgentStatus::Waiting,
            "Waiting for a release note",
            0.08,
            95,
        ),
        agent(
            "triage",
            "Triage",
            "Haiku",
            None,
            AgentStatus::Done,
            "Labelled 14 issues",
            0.22,
            300,
        ),
    ];
    let inbox = vec![
        InboxItem {
            id: "i1".into(),
            agent_id: "docs".into(),
            kind: InboxKind::Decision,
            title: "Publish the release note?".into(),
            detail: "Draft is ready to publish under v2.4.".into(),
            waiting_secs: 240,
            actions: vec![
                InboxAction {
                    id: "yes".into(),
                    label: "Publish".into(),
                },
                InboxAction {
                    id: "edit".into(),
                    label: "Edit first".into(),
                },
            ],
        },
        InboxItem {
            id: "i2".into(),
            agent_id: "ui".into(),
            kind: InboxKind::Failure,
            title: "Layout broke at 390 px".into(),
            detail: "The pricing grid overflows by 14 px.".into(),
            waiting_secs: 610,
            actions: vec![InboxAction {
                id: "retry".into(),
                label: "Retry".into(),
            }],
        },
    ];
    let activity = vec![
        ActivityPoint {
            at: NOW_MS - 50 * MIN,
            agent_id: "triage".into(),
            outcome: Outcome::Done,
        },
        ActivityPoint {
            at: NOW_MS - 34 * MIN,
            agent_id: "ui".into(),
            outcome: Outcome::Failed,
        },
        ActivityPoint {
            at: NOW_MS - 20 * MIN,
            agent_id: "docs".into(),
            outcome: Outcome::NeedsYou,
        },
        ActivityPoint {
            at: NOW_MS - 9 * MIN,
            agent_id: "api".into(),
            outcome: Outcome::Done,
        },
    ];
    let mut agents_sig = use_signal(|| agents.clone());
    let mut inbox_sig = use_signal(|| inbox.clone());
    rsx! {
        Item { slug: "agent-fleet-overview", name: "AgentFleetOverview",
            AgentFleetOverview {
                agents: agents_sig(),
                inbox_items: inbox_sig(),
                activity: activity,
                last_seen_at: NOW_MS - 60 * MIN,
                now_at: gallery_now(),
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
    let agents = vec![
        TimelineAgent {
            id: "lead".into(),
            name: "Planner".into(),
            model_label: "Opus".into(),
            status: AgentStatus::Running,
        },
        TimelineAgent {
            id: "api".into(),
            name: "API tests".into(),
            model_label: "Sonnet".into(),
            status: AgentStatus::Running,
        },
        TimelineAgent {
            id: "ui".into(),
            name: "Layout check".into(),
            model_label: "Sonnet".into(),
            status: AgentStatus::Failed,
        },
        TimelineAgent {
            id: "docs".into(),
            name: "Changelog".into(),
            model_label: "Haiku".into(),
            status: AgentStatus::Waiting,
        },
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
            "lead",
            "Plan release",
            NOW_MS - 28 * MIN,
            Some(NOW_MS - 20 * MIN),
            AgentStatus::Done,
            None,
        ),
        span(
            "b",
            "lead",
            "Split work",
            NOW_MS - 19 * MIN,
            None,
            AgentStatus::Running,
            None,
        ),
        span(
            "c",
            "api",
            "Contract tests",
            NOW_MS - 22 * MIN,
            None,
            AgentStatus::Running,
            None,
        ),
        span(
            "d",
            "ui",
            "Screenshot diff",
            NOW_MS - 24 * MIN,
            Some(NOW_MS - 14 * MIN),
            AgentStatus::Failed,
            Some("Pricing grid overflows by 14 px at 390 px"),
        ),
        span(
            "e",
            "docs",
            "Draft changelog",
            NOW_MS - 16 * MIN,
            Some(NOW_MS - 6 * MIN),
            AgentStatus::Waiting,
            None,
        ),
    ];
    let inbox = vec![InboxItem {
        id: "i1".into(),
        agent_id: "docs".into(),
        kind: InboxKind::Decision,
        title: "Publish the release note?".into(),
        detail: "Draft ready under v2.4".into(),
        waiting_secs: 240,
        actions: vec![InboxAction {
            id: "yes".into(),
            label: "Publish".into(),
        }],
    }];
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
    let trend: Vec<TrendPoint> = (0..8)
        .map(|i| TrendPoint {
            x_label: format!("W{}", i + 1),
            value: 40.0 + (i as f64) * 6.5,
            comparison_value: Some(36.0 + (i as f64) * 4.0),
        })
        .collect();
    let messages = vec![
        ChatMessage {
            id: "q1".into(),
            role: Role::Person,
            text: Some("How did revenue move this quarter?".into()),
            blocks: vec![],
            failed: false,
        },
        ChatMessage {
            id: "a1".into(),
            role: Role::Assistant,
            text: Some("Revenue is up 18% on last quarter.".into()),
            blocks: vec![
                AnswerBlock::Figures(vec![
                    Figure {
                        label: "Revenue".into(),
                        value: "$482k".into(),
                        delta_percent: 18.0,
                    },
                    Figure {
                        label: "Refunds".into(),
                        value: "$9.1k".into(),
                        delta_percent: -4.2,
                    },
                ]),
                AnswerBlock::Trend {
                    block_id: "t1".into(),
                    series_label: "Revenue".into(),
                    comparison_label: "Last quarter".into(),
                    points: trend,
                },
                AnswerBlock::Breakdown {
                    block_id: "b1".into(),
                    groups: vec![
                        BreakdownGroup {
                            label: "Subscriptions".into(),
                            value: 310.0,
                        },
                        BreakdownGroup {
                            label: "Services".into(),
                            value: 120.0,
                        },
                        BreakdownGroup {
                            label: "Add-ons".into(),
                            value: 52.0,
                        },
                    ],
                },
            ],
            failed: false,
        },
        ChatMessage {
            id: "q2".into(),
            role: Role::Person,
            text: Some("Where will revenue land by week 12?".into()),
            blocks: vec![],
            failed: false,
        },
        ChatMessage {
            id: "a2".into(),
            role: Role::Assistant,
            text: Some("Forecast: about 640k by week 12, with a band of 600k to 680k.".into()),
            blocks: vec![AnswerBlock::Forecast {
                block_id: "f1".into(),
                points: [
                    (482.0, 482.0, 482.0),
                    (498.0, 490.0, 506.0),
                    (510.0, 496.0, 524.0),
                    (530.0, 510.0, 550.0),
                    (552.0, 528.0, 576.0),
                    (571.0, 540.0, 602.0),
                    (590.0, 552.0, 628.0),
                    (612.0, 568.0, 656.0),
                    (640.0, 600.0, 680.0),
                ]
                .iter()
                .enumerate()
                .map(|(i, (v, lo, hi))| ForecastPoint {
                    x_label: format!("W{}", i + 4),
                    value: *v,
                    band_low: Some(*lo),
                    band_high: Some(*hi),
                })
                .collect(),
                actual_until_index: 3,
            }],
            failed: false,
        },
    ];
    let prompts = vec![Prompt {
        id: "p1".into(),
        label: "Show revenue by month".into(),
    }];
    rsx! {
        Item { slug: "chart-answer-chat", name: "ChartAnswerChat",
            ChartAnswerChat { messages: messages, suggested_prompts: prompts, on_ask: move |_: AskSubmit| {} }
        }
    }
}

#[component]
fn WorkspaceItem() -> Element {
    use rv2_kit::blocks::client_workspace::*;
    let mut section = use_signal(|| "context".to_string());
    let mut kind = use_signal(|| None::<ItemKind>);
    let items = vec![
        TimelineItem {
            id: "g1".into(),
            kind: ItemKind::Goal,
            author: "Ana".into(),
            at_ms: NOW_MS - 90 * MIN,
            title: None,
            text: "Renew the annual audit before March.".into(),
            references: vec![],
        },
        TimelineItem {
            id: "r1".into(),
            kind: ItemKind::Report,
            author: "Sam".into(),
            at_ms: NOW_MS - 50 * MIN,
            title: Some("Q3 cash report".into()),
            text: "Cash runway is 14 months at current spend.".into(),
            references: vec![],
        },
        TimelineItem {
            id: "n1".into(),
            kind: ItemKind::Note,
            author: "Ana".into(),
            at_ms: NOW_MS - 10 * MIN,
            title: None,
            text: "Call moved to Thursday; see [Q3 cash report].".into(),
            references: vec![Reference {
                label: "Q3 cash report".into(),
                target: "r1".into(),
            }],
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
            "Acme Ltd",
            vec!["Retainer".to_string(), "EUR".to_string()],
            vec![
                StatCard {
                    id: "rev".into(),
                    label: "Revenue".into(),
                    value: "€42.1k".into(),
                    delta_percent: Some(8.2),
                    series: vec![3.0, 4.0, 3.5, 5.0, 6.0],
                },
                StatCard {
                    id: "margin".into(),
                    label: "Margin".into(),
                    value: "31%".into(),
                    delta_percent: Some(-1.4),
                    series: vec![33.0, 32.0, 32.5, 31.0, 31.0],
                },
            ],
        )
    };
    rsx! {
        Item { slug: "client-workspace", name: "ClientWorkspace",
            ClientWorkspace {
                workspace: WorkspaceInfo { id: ws_id.into(), name: ws_name.into(), meta: ws_meta, synced_at_ms: NOW_MS - 4 * MIN },
                workspace_options: vec![WorkspaceOption { id: "acme".into(), name: "Acme Ltd".into() }, WorkspaceOption { id: "globex".into(), name: "Globex".into() }],
                stat_cards: cards,
                sections: vec![
                    WorkspaceSection { id: "context".into(), label: "Context".into(), count: Some(3) },
                    WorkspaceSection { id: "reports".into(), label: "Reports".into(), count: Some(1) },
                    WorkspaceSection { id: "settings".into(), label: "Settings".into(), count: None },
                ],
                active_section: section(),
                on_section_change: move |s: String| section.set(s),
                timeline_items: items_by_client().get(&client()).cloned().unwrap_or_default(),
                kind_options: vec![ItemKind::Goal, ItemKind::Note, ItemKind::Report],
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
    use rv2_kit::blocks::expandable_panel_strip::*;
    let mut open = use_signal(|| Some("reports".to_string()));
    let preview = |title: &str| rsx! { div { style: "padding: 24px; font: 15px system-ui", "{title} preview" } };
    let panels = vec![
        StripPanel {
            id: "reports".into(),
            title: "Reports".into(),
            caption: "Scheduled reports".into(),
            preview: preview("Reports"),
        },
        StripPanel {
            id: "alerts".into(),
            title: "Alerts".into(),
            caption: "Threshold alerts".into(),
            preview: preview("Alerts"),
        },
        StripPanel {
            id: "forecast".into(),
            title: "Forecast".into(),
            caption: "Revenue forecast".into(),
            preview: preview("Forecast"),
        },
    ];
    rsx! {
        Item { slug: "expandable-panel-strip", name: "ExpandablePanelStrip",
            ExpandablePanelStrip { panels: panels, open_id: open(), on_open_change: move |id: Option<String>| open.set(id), preview_label: "Feature" }
        }
    }
}

#[component]
fn CatalogItem() -> Element {
    use rv2_kit::blocks::integration_catalog::*;
    let mut search = use_signal(String::new);
    let mut connected = use_signal(|| vec!["slack".to_string()]);
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
            "slack",
            "Slack",
            "Slack",
            "Post summaries and approvals to channels.",
            &["chat"],
            &["Free"],
        ),
        mk(
            "gcal",
            "Calendar",
            "Google",
            "Read meetings to plan the week.",
            &["calendar"],
            &["Free"],
        ),
        mk(
            "stripe",
            "Stripe",
            "Stripe",
            "Read invoices and payouts.",
            &["finance"],
            &["Team"],
        ),
        mk(
            "github",
            "GitHub",
            "GitHub",
            "Track pull requests and issues.",
            &["dev"],
            &["Team"],
        ),
    ];
    rsx! {
        Item { slug: "integration-catalog", name: "IntegrationCatalog",
            IntegrationCatalog {
                integrations: items,
                categories: vec![CategoryOption { id: "chat".into(), label: "Chat".into() }, CategoryOption { id: "dev".into(), label: "Developer".into() }, CategoryOption { id: "finance".into(), label: "Finance".into() }],
                featured_ids: vec!["slack".into(), "gcal".into(), "stripe".into()],
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
        number: "INV-2041".into(),
        issued_on: "1 Nov 2026".into(),
        due_on: "30 Nov 2026".into(),
        period_from: "1 Oct".into(),
        period_to: "31 Oct".into(),
        currency: "EUR".into(),
        issuer: party("Harbour Labs"),
        recipient: party("Acme Ltd"),
        tax_percent: Some(23.0),
        status: InvoiceStatus::Open,
        lines: vec![
            Line {
                id: "l1".into(),
                name: "Platform seats".into(),
                quantity: 12.0,
                unit_amount: 40.0,
                tax_percent: 23.0,
                total: 480.0,
                kind: LineKind::Fixed,
            },
            Line {
                id: "l2".into(),
                name: "API calls".into(),
                quantity: 81_000.0,
                unit_amount: 0.001,
                tax_percent: 23.0,
                total: 81.0,
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
            label: "To do".into(),
        },
        StatusColumn {
            id: "doing".into(),
            label: "In progress".into(),
        },
        StatusColumn {
            id: "done".into(),
            label: "Done".into(),
        },
    ];
    let card = |id: &str, title: &str, status: &str, prio: u8, due: Option<i64>| WorkItem {
        id: id.into(),
        title: title.into(),
        status_id: status.into(),
        priority: prio,
        label_ids: vec!["bug".into()],
        assignee_id: None,
        due_in_days: due,
        subtasks_done: 1,
        subtasks_total: 3,
        pull_request_ref: None,
    };
    let items = vec![
        card(
            "T-1",
            "Fix VAT validation on empty input",
            "todo",
            3,
            Some(-2),
        ),
        card(
            "T-2",
            "Pricing grid overflow at 390 px",
            "doing",
            2,
            Some(0),
        ),
        card("T-3", "Export invoices as CSV", "done", 1, Some(4)),
    ];
    rsx! {
        Item { slug: "kanban-task-board", name: "KanbanTaskBoard",
            KanbanTaskBoard {
                breadcrumb_segments: vec!["Acme".into(), "Billing".into(), "Board".into()],
                board_name: "Billing sprint",
                status_columns: columns,
                work_items: items,
                tag_options: vec![TagOption { id: "bug".into(), label: "Bug".into() }],
                starred_ids: vec!["T-2".into()],
                on_items_update: move |_: Vec<WorkItem>| {},
            }
        }
    }
}

#[component]
fn OverviewItem() -> Element {
    use rv2_kit::blocks::metric_overview_board::*;
    let mut period = use_signal(|| "week".to_string());
    let mut segment = use_signal(|| ALL_SEGMENTS.to_string());
    let axis: Vec<String> = (1..=7).map(|d| format!("Day {d}")).collect();
    let base_cards = vec![
        StatMetric {
            id: "rev".into(),
            label: "Revenue".into(),
            total: 42_100.0,
            current_series: vec![4.0, 5.0, 4.5, 6.0, 6.5, 7.0, 7.5],
            previous_series: vec![3.5, 4.0, 4.2, 4.4, 4.8, 5.0, 5.2],
            lower_is_better: false,
        },
        StatMetric {
            id: "churn".into(),
            label: "Churn".into(),
            total: 2.3,
            current_series: vec![2.8, 2.6, 2.5, 2.4, 2.2, 2.3, 2.3],
            previous_series: vec![2.4, 2.5, 2.6, 2.7, 2.6, 2.5, 2.4],
            lower_is_better: true,
        },
        StatMetric {
            id: "cost".into(),
            label: "Support cost".into(),
            total: 6_400.0,
            current_series: vec![900.0, 950.0, 880.0, 940.0, 910.0, 920.0, 900.0],
            previous_series: vec![860.0, 880.0, 870.0, 900.0, 890.0, 880.0, 870.0],
            lower_is_better: true,
        },
    ];
    let base_overview = vec![
        OverviewSeries {
            label: "Enterprise".into(),
            values: vec![10.0, 12.0, 11.0, 13.0, 14.0, 15.0, 16.0],
        },
        OverviewSeries {
            label: "SMB".into(),
            values: vec![6.0, 6.5, 7.0, 6.8, 7.2, 7.5, 7.8],
        },
    ];
    let (scale, previous_scale) = if period() == "month" {
        (4.0, 3.6)
    } else {
        (1.0, 1.0)
    };
    let cards: Vec<StatMetric> = base_cards
        .into_iter()
        .map(|c| StatMetric {
            total: c.total * scale,
            current_series: c.current_series.iter().map(|v| v * scale).collect(),
            previous_series: c
                .previous_series
                .iter()
                .map(|v| v * previous_scale)
                .collect(),
            ..c
        })
        .collect();
    let overview: Vec<OverviewSeries> = base_overview
        .into_iter()
        .map(|s| OverviewSeries {
            values: s.values.iter().map(|v| v * scale).collect(),
            ..s
        })
        .collect();
    rsx! {
        Item { slug: "metric-overview-board", name: "MetricOverviewBoard",
            MetricOverviewBoard {
                stat_cards: cards,
                overview_series: overview,
                axis_labels: axis,
                period_options: vec![PickOption { id: "week".into(), label: "Last 7 days".into() }, PickOption { id: "month".into(), label: "Last 30 days".into() }],
                selected_period: period(),
                on_period_change: move |p: String| period.set(p),
                segment_options: vec![PickOption { id: ALL_SEGMENTS.into(), label: "All".into() }, PickOption { id: "Enterprise".into(), label: "Enterprise".into() }, PickOption { id: "SMB".into(), label: "SMB".into() }],
                selected_segment: segment(),
                on_segment_change: move |s: String| segment.set(s),
                heading: "Overview",
                format_amount: rv2_kit::components::series_trend_chart::compact_number,
                setup_action: ("Set a target".to_string(), EventHandler::new(|_| {})),
            }
        }
    }
}

#[component]
fn PlanItem() -> Element {
    use rv2_kit::blocks::plan_comparison_section::*;
    let mut billing = use_signal(|| MONTHLY.to_string());
    let plan =
        |id: &str, name: &str, m: f64, y: f64, highlighted: bool, features: &[(&str, bool)]| Plan {
            id: id.into(),
            name: name.into(),
            price_monthly: m,
            price_yearly: y,
            billing_note_monthly: "Billed monthly".into(),
            billing_note_yearly: "Billed yearly".into(),
            cta_label: if highlighted {
                "Start trial".into()
            } else {
                "Choose".into()
            },
            cta_href: "#".into(),
            highlighted,
            badge: highlighted.then(|| "Most popular".to_string()),
            limits: vec![
                PlanLimit {
                    label: "Seats".into(),
                    value: "12".into(),
                },
                PlanLimit {
                    label: "API calls".into(),
                    value: "100k".into(),
                },
            ],
            integrations: PlanIntegrations {
                names: vec!["Slack".into(), "Calendar".into()],
                sync_label: "Sync hourly".into(),
                max_selectable: 3,
            },
            features: features
                .iter()
                .map(|(l, i)| PlanFeature {
                    label: l.to_string(),
                    included: *i,
                })
                .collect(),
        };
    let plans = vec![
        plan(
            "starter",
            "Starter",
            19.0,
            190.0,
            false,
            &[("Email support", true), ("Audit log", false)],
        ),
        plan(
            "team",
            "Team",
            49.0,
            470.0,
            true,
            &[("Email support", true), ("Audit log", true)],
        ),
        plan(
            "scale",
            "Scale",
            129.0,
            1_240.0,
            false,
            &[("Priority support", true), ("SSO", true)],
        ),
    ];
    rsx! {
        Item { slug: "plan-comparison-section", name: "PlanComparisonSection",
            PlanComparisonSection {
                plans: plans,
                heading: "Plans",
                intro: "Compare what each plan includes.",
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
        title: "Release check run".into(),
        status: RunStatus::Running,
        duration_secs: 312.4,
        cost_usd: 6.12,
        token_count: 184_000,
    };
    let agents = vec![
        RunAgent {
            id: "lead".into(),
            name: "Planner".into(),
            model_label: "Opus".into(),
            status: RunStatus::Running,
            parent_id: None,
            cost_usd: 3.9,
            elapsed_secs: 312.0,
            current_step: Some("Waiting on layout result".into()),
        },
        RunAgent {
            id: "ui".into(),
            name: "Layout check".into(),
            model_label: "Sonnet".into(),
            status: RunStatus::Error,
            parent_id: Some("lead".into()),
            cost_usd: 1.4,
            elapsed_secs: 140.0,
            current_step: None,
        },
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
            "lead",
            CallKind::Model,
            "Plan checks",
            CallStatus::Ok,
            12.4,
            None,
        ),
        call(
            "c2",
            "lead",
            CallKind::Tool,
            "dispatch",
            CallStatus::Ok,
            0.3,
            None,
        ),
        call(
            "c3",
            "ui",
            CallKind::Model,
            "Read diff",
            CallStatus::Ok,
            8.1,
            None,
        ),
        call(
            "c4",
            "ui",
            CallKind::Tool,
            "screenshot",
            CallStatus::Error,
            4.6,
            Some("Viewport 390 px: pricing grid overflows by 14 px"),
        ),
        call(
            "c5",
            "lead",
            CallKind::Tool,
            "Await layout",
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
    let mut period = use_signal(|| "week".to_string());
    let month = period() == "month";
    let mut style = use_signal(|| ChartStyle::Area);
    let mut compare = use_signal(|| false);
    let mut exported = use_signal(String::new);
    let cards = vec![
        SalesMetric {
            id: "rev".into(),
            label: "Revenue".into(),
            value: (if month { "$190.4k" } else { "$48.2k" }).into(),
            delta_percent: 12.4,
            series: vec![3.0, 4.0, 3.8, 5.0, 6.0, 5.5, 7.0],
        },
        SalesMetric {
            id: "orders".into(),
            label: "Orders".into(),
            value: (if month { "2,410" } else { "612" }).into(),
            delta_percent: -3.1,
            series: vec![90.0, 84.0, 88.0, 80.0, 86.0, 82.0, 79.0],
        },
        SalesMetric {
            id: "aov".into(),
            label: "Avg order".into(),
            value: (if month { "$79.0" } else { "$78.8" }).into(),
            delta_percent: 2.2,
            series: vec![76.0, 77.0, 77.5, 78.0, 78.2, 78.5, 78.8],
        },
    ];
    let trend: Vec<TrendPoint> = (1..=7)
        .map(|d| TrendPoint {
            date: format!("Nov {d}"),
            value: 5.0 + d as f64 * 0.8,
        })
        .collect();
    let previous: Vec<f64> = (1..=7).map(|d| 4.5 + d as f64 * 0.6).collect();
    rsx! {
        Item { slug: "sales-metrics-panel", name: "SalesMetricsPanel",
            div {
            SalesMetricsPanel {
                title: "Sales",
                subtitle: "1 Nov to 7 Nov 2026",
                metric_cards: cards,
                selected_metric: metric(),
                on_metric_change: move |m: String| metric.set(Some(m)),
                periods: vec![PeriodOption { id: "week".into(), label: "This week".into() }, PeriodOption { id: "month".into(), label: "This month".into() }],
                selected_period: period(),
                on_period_change: move |p: String| period.set(p),
                trend_series: trend,
                previous_series: previous,
                chart_style: style(),
                on_chart_style_change: move |s: ChartStyle| style.set(s),
                compare_enabled: compare(),
                on_compare_change: move |c: bool| compare.set(c),
                top_products: vec![Ranked { id: "p1".into(), name: "Team plan".into(), value: 21_400.0 }, Ranked { id: "p2".into(), name: "Starter".into(), value: 9_800.0 }],
                channels: vec![Ranked { id: "c1".into(), name: "Direct".into(), value: 18_000.0 }, Ranked { id: "c2".into(), name: "Partners".into(), value: 12_600.0 }],
                on_export: move |f: ExportFormat| exported.set(format!("Exported as {}", f.label())),
            }
            p { role: "status", "aria-live": "polite", "{exported}" }
            }
        }
    }
}
