use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_kit::screens::autonomy_settings::{
    ActionClass, ActionRule, AmountLimit, AppEntry, AutonomyLevel, AutonomySettings, Outcome,
    RecentEvent, Setting, WeekCounts,
};
use rv2_kit::screens::company_register::{
    Account as RegisterAccount, CompanyRecord, CompanyRegister, NavItem, NavSection,
};
use rv2_kit::screens::connected_apps_catalogue::{App, ConnectedAppsCatalogue};
use rv2_kit::screens::decision_queue::{
    AnswerRecord, AnswerValue, Artifact, DecisionQueue, Paragraph, ParagraphKind, Request,
    RequestKind, RequestOption, Run, TaskDetail, TaskStep,
};
use rv2_kit::screens::file_library::{
    Account as LibraryAccount, FileLibrary, Folder, Kind, Scope, Template, TemplateAction,
};
use rv2_kit::screens::folder_detail::{
    Account as FolderAccount, FileItem, FileKind, FolderDetail, ShareState,
};
use rv2_kit::screens::issue_board_screen::{
    BoardColumn, Draft, Issue, IssueBoardScreen, IssuePatch, IssueStatus, PriorityTone,
};
use rv2_kit::screens::prompt_test_board::{
    Account as PromptAccount, CaseStatus, ChangeKind, KeyFigure, PassPoint, PromptTestBoard,
    ProposedChange, TestCase, Trend, Version,
};
use rv2_kit::screens::revenue_chat_analyst::{
    AnswerBody, ChartBlock, FocusRange, MetricCard, RevenueChatAnalyst, SplitGroup, SplitPart, Turn,
};
use rv2_kit::screens::revenue_overview::{
    Account, AverageValue, BreakdownRow, ChangeSign, ChannelSeries, KeyFigure as RevenueFigure,
    RankedItem, RevenueOverview, SessionPeriod, SessionsSeries,
};

use super::screens_b_canvas::AutomationCanvasDemo;
use super::{Item, Section};

#[component]
pub fn ScreensB() -> Element {
    rsx! {
        Section { id: "screens_b", title: "Screens: data and boards",
            RevenueOverviewDemo {}
            PromptTestBoardDemo {}
            RevenueChatAnalystDemo {}
            CompanyRegisterDemo {}
            ConnectedAppsCatalogueDemo {}
            FileLibraryDemo {}
            FolderDetailDemo {}
            AutonomySettingsDemo {}
            DecisionQueueDemo {}
            IssueBoardScreenDemo {}
            AutomationCanvasDemo {}
        }
    }
}

/// Which state the demo host passes: the data, the loading skeleton or the empty state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Data,
    Loading,
    Empty,
}

/// Picks the demo state and shows the last callback the screen fired.
#[component]
pub fn ModePick(slug: &'static str, mode: Signal<Mode>, last: String) -> Element {
    let current = mode();
    let data_on = current == Mode::Data;
    let loading_on = current == Mode::Loading;
    let empty_on = current == Mode::Empty;
    rsx! {
        div { style: "display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin-bottom: 8px;",
            button { r#type: "button", "aria-pressed": "{data_on}", onclick: move |_| mode.set(Mode::Data), "Data" }
            button { r#type: "button", "aria-pressed": "{loading_on}", onclick: move |_| mode.set(Mode::Loading), "Loading" }
            button { r#type: "button", "aria-pressed": "{empty_on}", onclick: move |_| mode.set(Mode::Empty), "Empty" }
            div { class: "gl-out", "data-out": "{slug}", "{last}" }
        }
    }
}

fn answer_label(answer: &AnswerValue) -> String {
    match answer {
        AnswerValue::Option { option_id } => format!("option {option_id}"),
        AnswerValue::Text { free_text } => format!("text {free_text}"),
        AnswerValue::Approve => "approve".to_string(),
        AnswerValue::Decline => "decline".to_string(),
    }
}

#[component]
fn RevenueOverviewDemo() -> Element {
    let days = |a: &[f64]| a.to_vec();
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    rsx! {
        Item { slug: "revenue-overview", name: "RevenueOverview",
            ModePick { slug: "revenue-overview", mode, last: last() }
            div { style: "border: 1px solid var(--kit-rule-soft);",
                RevenueOverview {
                    account: Account { name: "Ada Lovelace".into(), avatar_url: None },
                    is_loading: loading,
                    is_empty: empty,
                    key_figures: vec![
                        RevenueFigure { item_name: "Revenue".into(), item_value: "$48,210".into(), change_text: "12.4% vs last period".into(), change_sign: ChangeSign::Up, good_when: ChangeSign::Up, trend_bars: vec![3.0, 5.0, 4.0, 8.0, 6.0] },
                        RevenueFigure { item_name: "Refund rate".into(), item_value: "2.1%".into(), change_text: "0.4 pts vs last period".into(), change_sign: ChangeSign::Up, good_when: ChangeSign::Down, trend_bars: vec![2.0, 2.0, 3.0, 4.0] },
                        RevenueFigure { item_name: "Orders".into(), item_value: "412".into(), change_text: "3.1% vs last period".into(), change_sign: ChangeSign::Down, good_when: ChangeSign::Up, trend_bars: vec![6.0, 5.0, 5.0, 4.0] },
                        RevenueFigure { item_name: "Churn".into(), item_value: "1.8%".into(), change_text: "no change".into(), change_sign: ChangeSign::Flat, good_when: ChangeSign::Down, trend_bars: vec![2.0, 2.0, 2.0] },
                    ],
                    day_labels: vec!["Mon".into(), "Tue".into(), "Wed".into(), "Thu".into(), "Fri".into(), "Sat".into(), "Sun".into()],
                    channel_series: vec![
                        ChannelSeries { name: "Web".into(), daily: days(&[6200.0, 6900.0, 7100.0, 6800.0, 7400.0, 8100.0, 7700.0]) },
                        ChannelSeries { name: "Partners".into(), daily: days(&[2100.0, 2400.0, 1900.0, 2600.0, 2800.0, 3100.0, 2950.0]) },
                    ],
                    chart_total: "$48,210",
                    chart_change_text: "12.4%",
                    chart_change_sign: ChangeSign::Up,
                    breakdown_rows: vec![
                        BreakdownRow { item_name: "Subscriptions".into(), item_value: "$31,000".into(), trend_points: vec![1.0, 2.0, 2.0, 3.0], trend_direction: ChangeSign::Up },
                        BreakdownRow { item_name: "One-time".into(), item_value: "$9,800".into(), trend_points: vec![3.0, 2.0, 2.0, 1.0], trend_direction: ChangeSign::Down },
                        BreakdownRow { item_name: "Add-ons".into(), item_value: "$7,410".into(), trend_points: vec![2.0, 2.0, 2.0, 2.0], trend_direction: ChangeSign::Flat },
                    ],
                    ranked_items: vec![
                        RankedItem { item_name: "Pro plan".into(), item_value: "$20,000".into(), share_pct: 100.0 },
                        RankedItem { item_name: "Team add-on".into(), item_value: "$9,000".into(), share_pct: 45.0 },
                        RankedItem { item_name: "Support pack".into(), item_value: "$3,500".into(), share_pct: 17.5 },
                    ],
                    sessions_series: vec![
                        SessionsSeries { item_name: SessionPeriod::This, daily: vec![310.0, 342.0, 330.0, 360.0, 390.0, 401.0, 388.0] },
                        SessionsSeries { item_name: SessionPeriod::Previous, daily: vec![280.0, 300.0, 296.0, 310.0, 322.0, 340.0, 333.0] },
                    ],
                    average_value: Some(AverageValue { current: "$42.10".into(), previous: "$39.80".into(), daily: vec![40.0, 44.0, 42.0, 43.0, 41.0, 45.0, 42.0] }),
                    period_label: "Last 7 days",
                    on_connect: move |_| last.set("connect payment account".into()),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                    on_export: move |_| last.set("export".into()),
                    on_save_report: move |_| last.set("save as report".into()),
                    on_period_change: move |period: String| last.set(format!("period {period}")),
                }
            }
        }
    }
}

#[component]
fn PromptTestBoardDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let mut running = use_signal(|| false);
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    rsx! {
        Item { slug: "prompt-test-board", name: "PromptTestBoard",
            ModePick { slug: "prompt-test-board", mode, last: last() }
            div { style: "height: 980px; overflow: hidden; border: 1px solid var(--kit-rule-soft);",
                PromptTestBoard {
                    account: PromptAccount { name: "Mina Okafor".into(), avatar_url: None },
                    prompt_name: "Support reply",
                    prompt_names: vec!["Support reply".into(), "Refund summary".into()],
                    versions: vec![
                        Version { id: "v1".into(), label: "v1".into(), created_at: "Nov 2".into(), pass_rate: 62.0 },
                        Version { id: "v2".into(), label: "v2".into(), created_at: "Nov 12".into(), pass_rate: 74.0 },
                        Version { id: "v3".into(), label: "v3".into(), created_at: "Nov 19".into(), pass_rate: 81.0 },
                    ],
                    current_version: "v3",
                    model: "Model A",
                    last_run_text: "last run two hours ago",
                    is_loading: loading,
                    is_empty: empty,
                    running: running(),
                    key_figures: vec![
                        KeyFigure { label: "Pass rate".into(), value: "81%".into(), delta: "+7 pts".into(), trend: Trend::Up, good_when: Trend::Up },
                        KeyFigure { label: "Average score".into(), value: "4.2".into(), delta: "+0.3".into(), trend: Trend::Up, good_when: Trend::Up },
                        KeyFigure { label: "Median latency".into(), value: "1.2 s".into(), delta: "−0.3 s".into(), trend: Trend::Down, good_when: Trend::Down },
                        KeyFigure { label: "Cost per 1,000".into(), value: "$2.40".into(), delta: "+$0.20".into(), trend: Trend::Up, good_when: Trend::Down },
                    ],
                    pass_rate_series: vec![
                        PassPoint { version_label: "v1".into(), pass_rate: 62.0 },
                        PassPoint { version_label: "v2".into(), pass_rate: 74.0 },
                        PassPoint { version_label: "v3".into(), pass_rate: 81.0 },
                    ],
                    prompt_text: "Answer the customer in plain English. Be brief and cite the refund policy. Escalate billing disputes to a person.",
                    proposed_changes: vec![
                        ProposedChange { id: "p1".into(), kind: ChangeKind::Replace, before: "Be brief".into(), after: "Be concise".into(), author: "Mina".into() },
                        ProposedChange { id: "p2".into(), kind: ChangeKind::Add, before: String::new(), after: " Use the customer's first name.".into(), author: "Sam".into() },
                    ],
                    test_cases: vec![
                        TestCase { id: "c1".into(), input: "Where is my refund?".into(), expected: "Cites the 5-day window".into(), output: Some("Refunds take up to five days.".into()), score: Some(5.0), max_score: 5.0, latency_ms: Some(820.0), status: CaseStatus::Pass },
                        TestCase { id: "c2".into(), input: "I was charged twice".into(), expected: "Escalates to a person".into(), output: Some("Please check your statement.".into()), score: Some(2.0), max_score: 5.0, latency_ms: Some(1340.0), status: CaseStatus::Partial },
                        TestCase { id: "c3".into(), input: "Cancel my plan".into(), expected: "Explains the cancel steps".into(), output: None, score: Some(0.0), max_score: 5.0, latency_ms: Some(910.0), status: CaseStatus::Fail },
                        TestCase { id: "c4".into(), input: "Change my address".into(), expected: "Asks for the new address".into(), output: Some("Sure, what is the new address?".into()), score: Some(5.0), max_score: 5.0, latency_ms: None, status: CaseStatus::Pass },
                    ],
                    on_run_eval: move |_| {
                        last.set("run evaluation".into());
                        running.set(true);
                        spawn(async move {
                            let _ = document::eval("await new Promise((resolve) => setTimeout(resolve, 1500)); return true;").await;
                            running.set(false);
                        });
                    },
                    on_select_version: move |id: String| last.set(format!("version {id}")),
                    on_accept_change: move |id: String| last.set(format!("accept {id}")),
                    on_reject_change: move |id: String| last.set(format!("reject {id}")),
                    on_accept_all: move |_| last.set("accept all".into()),
                    on_reject_all: move |_| last.set("reject all".into()),
                    on_add_case: move |_| last.set("add case".into()),
                    on_open_case: move |id: String| last.set(format!("open case {id}")),
                }
            }
        }
    }
}

#[component]
fn RevenueChatAnalystDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    let this_year = vec![
        4.6, 4.9, 4.8, 5.2, 5.1, 5.3, 5.6, 5.5, 5.9, 5.8, 6.2, 6.1, 6.4,
    ];
    let last_year = vec![
        4.2, 4.4, 4.4, 4.7, 4.5, 4.8, 4.9, 5.0, 5.1, 5.0, 5.3, 5.2, 5.4,
    ];
    let total = this_year.iter().sum::<f64>() * 1000.0;
    let prior_total = last_year.iter().sum::<f64>() * 1000.0;
    let change = (total / prior_total - 1.0) * 100.0;
    let weeks: Vec<String> = (1..=13).map(|w| format!("Wk {w}")).collect();
    rsx! {
        Item { slug: "revenue-chat-analyst", name: "RevenueChatAnalyst",
            ModePick { slug: "revenue-chat-analyst", mode, last: last() }
            div { style: "height: 920px; overflow: hidden; border: 1px solid var(--kit-rule-soft);",
                RevenueChatAnalyst {
                    chat_list: vec!["Q4 revenue".into(), "Refund review".into()],
                    is_loading: loading,
                    is_empty: empty,
                    turn_list: vec![
                        Turn::Question { id: "q1".into(), text: "How did revenue move over the last 13 weeks?".into(), quoted_focus: None },
                        Turn::Answer {
                            id: "a1".into(),
                            body: AnswerBody {
                                text: format!("Revenue reached {total:.0} over 13 weeks, {change:.1}% above the same weeks last year. The gain held after the price change in week 9."),
                                metric_cards: vec![
                                    MetricCard { entry_label: "Revenue, 13 weeks".into(), value: total, change_text: Some(format!("+{change:.1}% vs last year")) },
                                    MetricCard { entry_label: "Orders".into(), value: 412.0, change_text: Some("−3.1%".into()) },
                                    MetricCard { entry_label: "Weekly average".into(), value: total / 13.0, change_text: Some("+2.4%".into()) },
                                ],
                                chart_block: Some(ChartBlock::Trend {
                                    category_labels: weeks,
                                    current_values: this_year,
                                    previous_values: last_year,
                                    marked_point: Some(8),
                                }),
                                caption_text: Some("Weekly revenue in thousands, this year against last year.".into()),
                                source_list: vec!["Ledger".into(), "Payments".into()],
                                follow_up_list: Vec::new(),
                            },
                        },
                        Turn::Question { id: "q2".into(), text: "Split that by region.".into(), quoted_focus: None },
                        Turn::Answer {
                            id: "a2".into(),
                            body: AnswerBody {
                                text: "EMEA carries the most growth, 31,000 of 65,000 this quarter. Web is the largest channel in every region.".into(),
                                metric_cards: Vec::new(),
                                chart_block: Some(ChartBlock::Split {
                                    title_text: "Revenue by region and channel, this quarter".into(),
                                    previous_total: Some(58000.0),
                                    groups: vec![
                                        SplitGroup { entry_name: "EMEA".into(), parts: vec![SplitPart { entry_name: "Web".into(), entry_value: 18000.0 }, SplitPart { entry_name: "Partners".into(), entry_value: 9000.0 }, SplitPart { entry_name: "Direct".into(), entry_value: 4000.0 }] },
                                        SplitGroup { entry_name: "Americas".into(), parts: vec![SplitPart { entry_name: "Web".into(), entry_value: 12000.0 }, SplitPart { entry_name: "Partners".into(), entry_value: 4000.0 }, SplitPart { entry_name: "Direct".into(), entry_value: 3000.0 }] },
                                        SplitGroup { entry_name: "APAC".into(), parts: vec![SplitPart { entry_name: "Web".into(), entry_value: 6000.0 }, SplitPart { entry_name: "Partners".into(), entry_value: 2400.0 }, SplitPart { entry_name: "Direct".into(), entry_value: 1600.0 }] },
                                        SplitGroup { entry_name: "LATAM".into(), parts: vec![SplitPart { entry_name: "Web".into(), entry_value: 3000.0 }, SplitPart { entry_name: "Partners".into(), entry_value: 1200.0 }, SplitPart { entry_name: "Direct".into(), entry_value: 800.0 }] },
                                    ],
                                }),
                                caption_text: Some("Each column adds its three channels, in dollars.".into()),
                                source_list: Vec::new(),
                                follow_up_list: vec!["Show Americas only".into(), "Compare with last year".into(), "Which channel grew most?".into()],
                            },
                        },
                    ],
                    starter_questions: vec!["How did revenue change this quarter?".into(), "Which region grew fastest?".into(), "What will next month look like?".into()],
                    on_ask: move |(question, _focus): (String, Option<FocusRange>)| last.set(format!("ask {question}")),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
        }
    }
}

#[component]
fn CompanyRegisterDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    let record = |id: &str,
                  name: &str,
                  category: &str,
                  value: f64,
                  stage: &str,
                  owner: &str,
                  contacted: &str| {
        CompanyRecord {
            id: id.into(),
            company_name: name.into(),
            company_category: category.into(),
            website_domain: format!("{id}.example"),
            account_owner: owner.into(),
            deal_value: value,
            pipeline_stage: stage.into(),
            primary_contact: "Sam Reyes".into(),
            last_contact: contacted.into(),
            logo_url: None,
        }
    };
    rsx! {
        Item { slug: "company-register", name: "CompanyRegister",
            ModePick { slug: "company-register", mode, last: last() }
            div { style: "height: 860px; overflow: hidden; border: 1px solid var(--kit-rule-soft);",
                CompanyRegister {
                    account: RegisterAccount { company_name: "Juniper Works".into(), avatar_url: None },
                    is_loading: loading,
                    is_empty: empty,
                    sections: vec![
                        NavSection {
                            label: String::new(),
                            items: vec![
                                NavItem { label: "Home".into(), count: Some(4) },
                                NavItem { label: "Companies".into(), count: Some(10) },
                                NavItem { label: "Contacts".into(), count: Some(23) },
                                NavItem { label: "Reports".into(), count: Some(2) },
                            ],
                        },
                        NavSection {
                            label: "Pipelines".into(),
                            items: vec![
                                NavItem { label: "New business".into(), count: None },
                                NavItem { label: "Expansion".into(), count: None },
                                NavItem { label: "Partners".into(), count: None },
                                NavItem { label: "Renewals".into(), count: None },
                            ],
                        },
                    ],
                    company_records: vec![
                        record("acme", "Acme Labs", "Software", 12400.0, "Proposal", "Ada", "Nov 18"),
                        record("birch", "Birch & Co", "Retail", 0.0, "Renewal due", "Lin", "Nov 04"),
                        record("cobalt", "Cobalt Health", "Health", 38000.0, "Won", "Ada", "Oct 29"),
                        record("delta", "Delta Freight", "Logistics", 7600.0, "Qualified", "Sam", "Nov 21"),
                        record("ember", "Ember Foods", "Food", 5200.0, "Lead", "Lin", "Oct 14"),
                        record("fjord", "Fjord Design", "Creative", 9800.0, "Proposal", "Sam", "Nov 12"),
                        record("granite", "Granite Mining", "Industrial", 61000.0, "Won", "Ada", "Sep 30"),
                        record("harbor", "Harbor Legal", "Services", 0.0, "Renewal due", "Lin", "Nov 09"),
                        record("iris", "Iris Analytics", "Software", 14300.0, "Qualified", "Sam", "Nov 15"),
                        record("juno", "Juno Retail", "Retail", 22700.0, "Lead", "Ada", "Oct 22"),
                    ],
                    on_add: move |name: String| last.set(format!("add {name}")),
                    on_new_view: move |_| last.set("new view".into()),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                    on_cell_change: move |(id, field, value): (String, String, String)| last.set(format!("cell {id} {field} = {value}")),
                }
            }
        }
    }
}

#[component]
fn ConnectedAppsCatalogueDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    let app =
        |id: &str, name: &str, category: &str, summary: &str, connected: bool, popular: bool| App {
            id: id.into(),
            name: name.into(),
            summary: summary.into(),
            category: category.into(),
            connected,
            popular,
            logo_url: None,
            bundle: Vec::new(),
        };
    let docs = "Documents and design";
    let comms = "Communication";
    let sched = "Scheduling and analytics";
    let data = "Data and payments";
    rsx! {
        Item { slug: "connected-apps-catalogue", name: "ConnectedAppsCatalogue",
            ModePick { slug: "connected-apps-catalogue", mode, last: last() }
            div { style: "height: 760px; overflow: hidden; border: 1px solid var(--kit-rule-soft);",
                ConnectedAppsCatalogue {
                    is_loading: loading,
                    is_empty: empty,
                    categories: vec![docs.into(), comms.into(), sched.into(), data.into()],
                    apps: vec![
                        app("slack", "Slack", comms, "Post updates and read team threads.", true, true),
                        app("drive", "Google Drive", docs, "Open and share team files.", true, true),
                        app("gmail", "Gmail", comms, "Read and send email from the assistant.", false, true),
                        app("stripe", "Stripe", data, "Read payments, refunds and payouts.", false, true),
                        app("notion", "Notion", docs, "Search and update pages and databases.", true, false),
                        App { bundle: vec!["Word".into(), "Excel".into(), "PowerPoint".into()], ..app("office", "Office suite", docs, "Documents, sheets and slides.", false, false) },
                        app("figma", "Figma", docs, "Read frames and design comments.", false, false),
                        app("dropbox", "Dropbox", docs, "Find and open shared folders.", false, false),
                        app("zoom", "Zoom", comms, "Start meetings and read recordings.", true, false),
                        app("teams", "Microsoft Teams", comms, "Post to channels and read messages.", false, false),
                        app("intercom", "Intercom", comms, "Read support conversations.", false, false),
                        app("mailchimp", "Mailchimp", comms, "Read campaign results.", false, false),
                        App { bundle: vec!["Google Calendar".into(), "Outlook Calendar".into()], ..app("calendars", "Calendars", sched, "Check availability across calendars.", false, false) },
                        app("looker", "Looker Studio", sched, "Read dashboards and report figures.", true, false),
                        app("xero", "Xero", data, "Read invoices and bank lines.", false, false),
                        app("plaid", "Plaid", data, "Read linked bank balances.", false, false),
                    ],
                    on_connect: move |id: String| last.set(format!("connect {id}")),
                    on_disconnect: move |id: String| last.set(format!("disconnect {id}")),
                    on_request: move |(id, note): (String, String)| last.set(format!("request {id}: {note}")),
                    on_open_app: move |id: String| last.set(format!("open {id}")),
                }
            }
        }
    }
}

#[component]
fn FileLibraryDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    rsx! {
        Item { slug: "file-library", name: "FileLibrary",
            ModePick { slug: "file-library", mode, last: last() }
            div { style: "height: 760px; overflow: hidden; border: 1px solid var(--kit-rule-soft);",
                FileLibrary {
                    account: LibraryAccount { name: "Ada Lovelace".into(), avatar_url: None },
                    is_loading: loading,
                    is_empty: empty,
                    folders: vec![
                        Folder { id: "f1".into(), name: "Q3 Campaign".into(), scope: Scope::Shared, file_count: 12 },
                        Folder { id: "f2".into(), name: "Drafts".into(), scope: Scope::Private, file_count: 3 },
                        Folder { id: "f3".into(), name: "Board packs".into(), scope: Scope::Shared, file_count: 7 },
                        Folder { id: "f4".into(), name: "Hiring".into(), scope: Scope::Private, file_count: 5 },
                    ],
                    templates: vec![
                        Template { id: "t1".into(), name: "Brand guidelines".into(), kind: Kind::Doc, owner: "Ada".into(), scope: Scope::Shared },
                        Template { id: "t2".into(), name: "Campaign budget".into(), kind: Kind::Sheet, owner: "Lin".into(), scope: Scope::Mine },
                        Template { id: "t3".into(), name: "Client onboarding kit".into(), kind: Kind::Pdf, owner: "Sam".into(), scope: Scope::Shared },
                        Template { id: "t4".into(), name: "Weekly status report".into(), kind: Kind::Doc, owner: "Ada".into(), scope: Scope::Mine },
                    ],
                    on_create_folder: move |_| last.set("create folder".into()),
                    on_add_template: move |_| last.set("add template".into()),
                    on_open_folder: move |id: String| last.set(format!("open folder {id}")),
                    on_use_template: move |id: String| last.set(format!("use template {id}")),
                    on_template_action: move |(id, action): (String, TemplateAction)| last.set(format!("{action:?} {id}")),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
        }
    }
}

#[component]
fn FolderDetailDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    let file = |id: &str, name: &str, kind: FileKind| FileItem {
        id: id.into(),
        name: name.into(),
        kind,
        updated_at: "Nov 18".into(),
        subtitle: "Ada · Nov 18".into(),
        preview_url: None,
    };
    rsx! {
        Item { slug: "folder-detail", name: "FolderDetail",
            ModePick { slug: "folder-detail", mode, last: last() }
            div { style: "height: 720px; overflow: hidden; border: 1px solid var(--kit-rule-soft);",
                FolderDetail {
                    folder_name: "Q3 Campaign",
                    crumbs: vec!["Clients".to_string(), "Juniper".to_string(), "Q3 Campaign".to_string()],
                    account: FolderAccount { name: "Ada Lovelace".into(), avatar_url: None },
                    is_loading: loading,
                    is_empty: empty,
                    files: vec![
                        file("a", "Brief.pdf", FileKind::Pdf),
                        file("b", "Budget.xlsx", FileKind::Sheet),
                        file("c", "Brand notes", FileKind::Doc),
                        file("d", "Hero shot.png", FileKind::Image),
                        file("e", "Pricing.pdf", FileKind::Pdf),
                        file("f", "Pitch deck.docx", FileKind::Doc),
                        file("g", "Reach model.xlsx", FileKind::Sheet),
                        file("h", "Logo pack.png", FileKind::Image),
                        file("i", "Press kit.pdf", FileKind::Pdf),
                    ],
                    share_state: ShareState::Shared,
                    on_upload: move |names: Vec<String>| last.set(format!("upload {}", names.join(", "))),
                    on_share: move |_| last.set("share".into()),
                    on_open_file: move |id: String| last.set(format!("open file {id}")),
                    on_remove: move |ids: Vec<String>| last.set(format!("remove {}", ids.join(", "))),
                    on_start_chat: move |_| last.set("start chat".into()),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
        }
    }
}

#[component]
fn AutonomySettingsDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    let rule =
        |app: &str, id: &str, class: ActionClass, setting: Setting, fixed: bool, name: &str| {
            ActionRule {
                id: id.into(),
                app_id: app.into(),
                action_name: name.into(),
                detail_text: None,
                mark: None,
                rule_setting: setting,
                default_setting: setting,
                action_class: class,
                is_fixed: fixed,
                amount_limit: None,
                week_counts: WeekCounts {
                    done: 3,
                    asked: 1,
                    blocked: 0,
                },
                recent_events: vec![RecentEvent {
                    what: "Filed reply".into(),
                    outcome: Outcome::Done,
                    occurred_at: "Today, 9:14".into(),
                }],
            }
        };
    let event = |what: &str, outcome: Outcome, when: &str| RecentEvent {
        what: what.into(),
        outcome,
        occurred_at: when.into(),
    };
    let mut credit = rule(
        "payments",
        "credit",
        ActionClass::Risky,
        Setting::Allow,
        false,
        "Issue credit notes",
    );
    credit.amount_limit = Some(AmountLimit {
        amount: 200.0,
        currency: "USD".into(),
    });
    credit.recent_events = vec![event("Credit note 1042", Outcome::Approved, "Today, 8:02")];
    let mut charges = rule(
        "payments",
        "charges",
        ActionClass::Read,
        Setting::Allow,
        false,
        "Read charges",
    );
    charges.recent_events = vec![event("Read charges", Outcome::Done, "Today, 9:14")];
    let mut prices = rule(
        "payments",
        "prices",
        ActionClass::Risky,
        Setting::Never,
        false,
        "Change plan prices",
    );
    prices.recent_events = vec![event("Price change", Outcome::Declined, "Yesterday, 17:40")];
    let app = |id: &str, name: &str, account: &str, mark: &str| AppEntry {
        id: id.into(),
        entry_name: name.into(),
        account: Some(account.into()),
        mark: Some(mark.into()),
    };
    rsx! {
        Item { slug: "autonomy-settings", name: "AutonomySettings",
            ModePick { slug: "autonomy-settings", mode, last: last() }
            div { style: "height: 720px; overflow: auto; border: 1px solid var(--kit-rule-soft);",
                AutonomySettings {
                    initial_level: AutonomyLevel::Balanced,
                    is_loading: loading,
                    is_empty: empty,
                    app_list: vec![
                        app("mail", "Mail", "ops@juniper.example", "Ma"),
                        app("chat", "Team chat", "Juniper Works", "Tc"),
                        app("code", "Code host", "juniper-works", "Co"),
                        app("drive", "Shared drive", "drive@juniper.example", "Dr"),
                        app("payments", "Payments", "finance@juniper.example", "Pa"),
                        app("calendar", "Calendar", "ada@juniper.example", "Ca"),
                        app("browser", "Web browser", "ada@juniper.example", "Wb"),
                    ],
                    action_rules: vec![
                        rule("mail", "read-inbox", ActionClass::Read, Setting::Allow, false, "Read inbox"),
                        rule("mail", "search", ActionClass::Read, Setting::Allow, false, "Search threads"),
                        rule("mail", "file", ActionClass::Write, Setting::Allow, false, "File replies"),
                        rule("mail", "refund", ActionClass::Risky, Setting::Ask, false, "Issue refunds"),
                        rule("mail", "bulk", ActionClass::Risky, Setting::Never, false, "Send bulk email"),
                        rule("chat", "read-channels", ActionClass::Read, Setting::Allow, false, "Read channels"),
                        rule("chat", "post", ActionClass::Write, Setting::Allow, false, "Post messages"),
                        rule("chat", "share-out", ActionClass::Risky, Setting::Ask, false, "Share files outside the team"),
                        rule("chat", "delete-history", ActionClass::Risky, Setting::Never, false, "Delete channel history"),
                        rule("code", "read-repos", ActionClass::Read, Setting::Allow, false, "Read repositories"),
                        rule("code", "open-prs", ActionClass::Write, Setting::Allow, false, "Open pull requests"),
                        rule("code", "merge", ActionClass::Risky, Setting::Ask, false, "Merge to main"),
                        rule("code", "force-push", ActionClass::Risky, Setting::Never, false, "Force-push"),
                        rule("drive", "list", ActionClass::Read, Setting::Allow, false, "List files"),
                        rule("drive", "move", ActionClass::Write, Setting::Allow, false, "Move files"),
                        rule("drive", "sharing", ActionClass::Risky, Setting::Ask, false, "Change sharing settings"),
                        rule("drive", "delete-folders", ActionClass::Risky, Setting::Never, false, "Delete shared folders"),
                        charges,
                        credit,
                        prices,
                        rule("calendar", "read-calendar", ActionClass::Read, Setting::Allow, false, "Read calendar"),
                        rule("calendar", "create-events", ActionClass::Write, Setting::Allow, false, "Create events"),
                        rule("calendar", "outside-invites", ActionClass::Risky, Setting::Ask, false, "Accept invitations from outside"),
                        rule("browser", "read-pages", ActionClass::Read, Setting::Allow, false, "Read pages"),
                        rule("browser", "submit-forms", ActionClass::Risky, Setting::Ask, false, "Submit forms"),
                        rule("browser", "downloads", ActionClass::Risky, Setting::Never, false, "Download files"),
                        rule("mail", "delete-threads", ActionClass::Risky, Setting::Never, true, "Delete threads"),
                        rule("payments", "move-money", ActionClass::Risky, Setting::Never, true, "Move money between accounts"),
                        rule("drive", "share-outside", ActionClass::Risky, Setting::Ask, true, "Share outside the workspace"),
                        rule("browser", "publish", ActionClass::Risky, Setting::Never, true, "Publish to the public web"),
                        rule("code", "delete-repos", ActionClass::Risky, Setting::Never, true, "Delete repositories"),
                        rule("chat", "admin-roles", ActionClass::Risky, Setting::Never, true, "Change admin roles"),
                        rule("drive", "sign", ActionClass::Risky, Setting::Never, true, "Sign contracts"),
                    ],
                    initial_app_id: Some("payments".to_string()),
                    on_change: move |changed: ActionRule| last.set(format!("rule {}", changed.id)),
                    on_preset: move |_: AutonomyLevel| last.set("preset".into()),
                    on_select: move |id: String| last.set(format!("select {id}")),
                    on_connect: move |_| last.set("connect".into()),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
        }
    }
}

#[component]
fn DecisionQueueDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    let option = |id: &str, title: &str, recommended: bool| RequestOption {
        id: id.into(),
        entry_title: title.into(),
        summary: Some(format!("{title} for this order.")),
        is_recommended: recommended,
        advantages: vec!["Fast to settle".into()],
        drawbacks: vec!["Costs more".into()],
        preview_text: None,
    };
    let request = |id: &str,
                   kind: RequestKind,
                   question: &str,
                   options: Vec<RequestOption>,
                   answered: bool| Request {
        id: id.into(),
        request_kind: kind,
        question_text: question.into(),
        context_text: Some("The customer disputes a late delivery.".into()),
        task_name: "Refund batch".into(),
        asked_at: "12 min ago".into(),
        is_unread: !answered,
        option_list: options,
        fact_rows: Vec::new(),
        suggested_answers: vec!["Yes, go ahead".into()],
        answer_record: answered.then(|| AnswerRecord {
            answer_text: "Refund in full".into(),
            answered_at: "5 min ago".into(),
            entry_label: None,
        }),
    };
    let task_details = BTreeMap::from([(
        "Refund batch".to_string(),
        TaskDetail {
            goal: Some("Settle the late delivery claim".into()),
            project_name: Some("Refund batch".into()),
            task_state: Some("Waiting on you".into()),
            step_list: vec![
                TaskStep {
                    label: "Check the tracking record".into(),
                    done: true,
                },
                TaskStep {
                    label: "Pick a refund amount".into(),
                    done: false,
                },
            ],
            artifact_names: vec!["Claim summary".into(), "Refund sheet".into()],
        },
    )]);
    let artifact_store = BTreeMap::from([
        (
            "Claim summary".to_string(),
            Artifact::Document {
                paragraphs: vec![
                    Paragraph {
                        kind: ParagraphKind::Plain,
                        runs: vec![
                            Run {
                                text: "Order 1042 arrived ".into(),
                                ..Default::default()
                            },
                            Run {
                                text: "nine days".into(),
                                bold: true,
                                ..Default::default()
                            },
                            Run {
                                text: " late.".into(),
                                ..Default::default()
                            },
                        ],
                    },
                    Paragraph::plain("The customer asks for a full refund."),
                    Paragraph {
                        kind: ParagraphKind::Bullet,
                        runs: vec![Run {
                            text: "Policy: refunds over $500 need a second approver.".into(),
                            italic: true,
                            link: Some("https://example.com/refund-policy".into()),
                            ..Default::default()
                        }],
                    },
                    Paragraph {
                        kind: ParagraphKind::Checklist { done: true },
                        runs: vec![Run {
                            text: "Confirm the tracking record".into(),
                            ..Default::default()
                        }],
                    },
                    Paragraph {
                        kind: ParagraphKind::Checklist { done: false },
                        runs: vec![Run {
                            text: "Send the revised amount".into(),
                            ..Default::default()
                        }],
                    },
                ],
            },
        ),
        (
            "Refund sheet".to_string(),
            Artifact::Sheet {
                columns: vec!["Item".into(), "Amount".into()],
                rows: vec![vec!["Order 1042".into(), "$640".into()]],
            },
        ),
        (
            "Customer email".to_string(),
            Artifact::Document {
                paragraphs: vec![Paragraph::plain(
                    "Hi, the parcel reached us nine days after the promised date.",
                )],
            },
        ),
        (
            "Attachments".to_string(),
            Artifact::Folder {
                items: vec!["Receipt.pdf".into(), "Parcel photo.jpg".into()],
            },
        ),
    ]);
    rsx! {
        Item { slug: "decision-queue", name: "DecisionQueue",
            ModePick { slug: "decision-queue", mode, last: last() }
            div { style: "height: 760px; overflow: hidden; border: 1px solid var(--kit-rule-soft);",
                DecisionQueue {
                    request_list: vec![
                        request("q1", RequestKind::Decision, "Refund in full or partially?", vec![option("full", "Refund in full", true), option("part", "Partial refund", false)], false),
                        request("q2", RequestKind::Question, "Which address should the invoice use?", Vec::new(), false),
                        request("q3", RequestKind::Approval, "Approve the replacement shipment?", Vec::new(), true),
                    ],
                    task_details,
                    artifact_store,
                    running_count: 2,
                    is_loading: loading,
                    is_empty: empty,
                    on_answer: move |(id, answer): (String, AnswerValue)| last.set(format!("answer {id}: {}", answer_label(&answer))),
                    on_later: move |id: String| last.set(format!("later {id}")),
                    on_reopen: move |id: String| last.set(format!("reopen {id}")),
                    on_select: move |id: Option<String>| last.set(format!("select {}", id.unwrap_or_default())),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                    on_artifact_change: move |(name, artifact): (String, Artifact)| {
                        let detail = match &artifact {
                            Artifact::Document { paragraphs } => format!(
                                ", {} paragraphs, {} bold runs",
                                paragraphs.len(),
                                paragraphs.iter().flat_map(|p| p.runs.iter()).filter(|r| r.bold).count()
                            ),
                            _ => String::new(),
                        };
                        last.set(format!("artifact {name}{detail}"));
                    },
                }
            }
        }
    }
}

#[component]
fn IssueBoardScreenDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    let issue = |id: &str,
                 column: &str,
                 title: &str,
                 text: &str,
                 status: IssueStatus,
                 counts: (u32, u32),
                 who: &str| Issue {
        id: id.into(),
        column_id: column.into(),
        title: title.into(),
        description: Some(text.into()),
        issue_status: Some(status),
        comment_count: counts.0,
        attachment_count: counts.1,
        assignee: Some(who.into()),
        labels: Vec::new(),
    };
    rsx! {
        Item { slug: "issue-board-screen", name: "IssueBoardScreen",
            ModePick { slug: "issue-board-screen", mode, last: last() }
            div { style: "height: 760px; overflow: hidden; border: 1px solid var(--kit-rule-soft);",
                IssueBoardScreen {
                    board_columns: vec![
                        BoardColumn { id: "low".into(), column_label: "Low".into(), priority_tone: PriorityTone::Low },
                        BoardColumn { id: "medium".into(), column_label: "Medium".into(), priority_tone: PriorityTone::Medium },
                        BoardColumn { id: "high".into(), column_label: "High".into(), priority_tone: PriorityTone::High },
                        BoardColumn { id: "done".into(), column_label: "Done".into(), priority_tone: PriorityTone::Done },
                    ],
                    issues: vec![
                        issue("l1", "low", "Tidy invoice email footer", "Align the footer links with the rest of the template.", IssueStatus::Review, (0, 0), "Sam"),
                        issue("l2", "low", "Shorten onboarding tooltips", "Cut each tip to one sentence and link the guide.", IssueStatus::Progress, (1, 0), "Lin"),
                        issue("l3", "low", "Rename legacy export column", "Match the label used in the CSV template.", IssueStatus::Fixed, (0, 1), "Ada"),
                        issue("m1", "medium", "Onboarding copy review", "Check the welcome steps read well for new admins.", IssueStatus::Review, (2, 0), "Sam"),
                        issue("m2", "medium", "Empty state for inbox", "Add a line on what appears here and a next step.", IssueStatus::Progress, (0, 2), "Lin"),
                        issue("m3", "medium", "Pricing page FAQ", "Answer the three questions sales hears most.", IssueStatus::Attention, (3, 1), "Ada"),
                        issue("h1", "high", "Refund webhook fails", "Retries stop after the second attempt on staging.", IssueStatus::Attention, (4, 1), "Ada"),
                        issue("h2", "high", "Invoice PDF layout", "Line totals overflow the page on long descriptions.", IssueStatus::Progress, (2, 0), "Lin"),
                        issue("h3", "high", "Duplicate charge on retry", "Idempotency key is not sent on the second request.", IssueStatus::Attention, (5, 2), "Sam"),
                        issue("d1", "done", "Retire old export", "Old CSV endpoint removed; links redirect to the new one.", IssueStatus::Fixed, (1, 0), "Ada"),
                        issue("d2", "done", "Login rate limit", "Limit lowered to ten attempts per minute.", IssueStatus::Fixed, (0, 1), "Lin"),
                    ],
                    is_loading: loading,
                    is_empty: empty,
                    on_create: move |draft: Draft| last.set(format!("create {} in {}", draft.title, draft.column_id)),
                    on_move: move |(id, column, before): (String, String, Option<String>)| last.set(format!("move {id} to {column} before {}", before.unwrap_or_default())),
                    on_update: move |(id, patch): (String, IssuePatch)| last.set(format!("update {id}: {}", patch.title.unwrap_or_default())),
                    on_open: move |id: String| last.set(format!("open {id}")),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
        }
    }
}
