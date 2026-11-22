use dioxus::prelude::*;
use rv2_kit::components::grounded_reply::SourceEntry;
use rv2_kit::components::series_trend_chart::{SeriesTone, TrendSeries};
use rv2_kit::screens::ai_spend_chat_planner::{
    AiSpendChatPlanner, Answer, AskRequest, Chart, Lever, LeverKind, Metric, SplitGroup, SplitPart,
    Turn,
};
use rv2_kit::screens::ai_workspace_home::{
    ActivityDay, AiWorkspaceHome, FileEntry, MonthStat, MonthStats, RecentChat, UsageMeter,
};
use rv2_kit::screens::assistant_chat_progress_panel::{
    AssistantChatProgressPanel, CallState, Message, Output, OutputKind, Part, Step, StepState,
    ToolCall, UsageRecord,
};
use rv2_kit::screens::assistant_chat_sidebars::{
    AssistantChatSidebars, ChatItem, ChatMessage, OutputFile, OutputKind as SidebarOutputKind,
    Project, SearchState, SearchStep, Section as RailSection,
};
use rv2_kit::screens::assistant_naming_step::AssistantNamingStep;
use rv2_kit::screens::assistant_persona_step::AssistantPersonaStep;
use rv2_kit::screens::assistant_ready_step::AssistantReadyStep;
use rv2_kit::screens::assistant_start_page::{
    Artifact, AssistantStartPage, ConnectedApp, OutputKind as StartOutputKind, SendRequest,
    Suggestion, ToolChip,
};
use rv2_kit::screens::knowledge_source_step::KnowledgeSourceStep;
use rv2_kit::screens::workspace_chat::{
    ActivityEntry, AgentMessage, AgentState, AgentStatus, Brief, BriefField, BriefFile, FieldType,
    FileKind, WorkspaceChat,
};
use rv2_kit::screens::workspace_frame_inset::{
    Artifact as FileArtifact, ArtifactKind as FileArtifactKind,
};
use rv2_kit::screens::workspace_setup_checklist::{
    AgentTemplate, ConnectableTool, SetupValues, WorkspaceSetupChecklist,
};

use super::screens_b::{Mode, ModePick};
use super::{Item, Section, gallery_now};

#[component]
pub fn ScreensA() -> Element {
    rsx! {
        Section { id: "screens_a", title: "Screens: assistant and workspace",
            KnowledgeDemo {}
            NamingDemo {}
            PersonaDemo {}
            ReadyDemo {}
            StartDemo {}
            HomeDemo {}
            ProgressDemo {}
            SidebarsDemo {}
            WorkspaceChatDemo {}
            SpendDemo {}
            SetupDemo {}
        }
    }
}

/// A fixed-height frame: screens render as pages, so the demo gives them a page's height.
fn frame(height: u32, children: Element) -> Element {
    rsx! {
        div { class: "gl-canvas", style: "height: {height}px; overflow: auto; padding: 0;", {children} }
    }
}

fn domains(list: &[&str]) -> Vec<String> {
    list.iter().map(|d| d.to_string()).collect()
}

#[component]
fn KnowledgeDemo() -> Element {
    let mut last = use_signal(|| "no action yet".to_string());
    rsx! {
        Item { slug: "knowledge-source-step", name: "KnowledgeSourceStep",
            div { class: "gl-out", "data-out": "knowledge-source-step", "{last}" }
            {frame(760, rsx! {
                KnowledgeSourceStep {
                    default_domains: domains(&["harbor-lane.com", "docs.harbor-lane.com"]),
                    on_continue: move |list: Vec<String>| last.set(format!("continue {}", list.join(", "))),
                    on_logout: move |_| last.set("log out".to_string()),
                    account_email: "ana@harbor-lane.com",
                    auto_focus: true,
                }
            })}
        }
    }
}

#[component]
fn NamingDemo() -> Element {
    let mut known = use_signal(|| false);
    let mut last = use_signal(|| "no action yet".to_string());
    let known_on = known();
    let proposed_on = !known_on;
    rsx! {
        Item { slug: "assistant-naming-step", name: "AssistantNamingStep",
            div { style: "display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin-bottom: 8px;",
                button { r#type: "button", "aria-pressed": "{proposed_on}", onclick: move |_| known.set(false), "Proposed name" }
                button { r#type: "button", "aria-pressed": "{known_on}", onclick: move |_| known.set(true), "Known names" }
                div { class: "gl-out", "data-out": "assistant-naming-step", "{last}" }
            }
            {frame(700, rsx! {
                AssistantNamingStep {
                    key: "{known_on}",
                    website_domain: "harbor-lane.com",
                    default_names: if known_on { domains(&["Harbor Lane", "HL"]) } else { Vec::new() },
                    on_continue: move |names: Vec<String>| last.set(format!("continue {}", names.join(" | "))),
                    account_email: "ana@harbor-lane.com",
                }
            })}
        }
    }
}

#[component]
fn PersonaDemo() -> Element {
    rsx! {
        Item { slug: "assistant-persona-step", name: "AssistantPersonaStep",
            {frame(760, rsx! {
                AssistantPersonaStep {
                    website_domain: "harbor-lane.com",
                    brand_name: "Harbor lane",
                    short_forms: domains(&["HL"]),
                    on_continue: move |_| {},
                    account_email: "ana@harbor-lane.com",
                }
            })}
        }
    }
}

#[component]
fn ReadyDemo() -> Element {
    let mut no_handover = use_signal(|| false);
    let mut last = use_signal(|| "no action yet".to_string());
    let no_handover_on = no_handover();
    let handover_on = !no_handover_on;
    let handover_to = if no_handover_on {
        String::new()
    } else {
        "support@harbor-lane.com".to_string()
    };
    rsx! {
        Item { slug: "assistant-ready-step", name: "AssistantReadyStep",
            div { style: "display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin-bottom: 8px;",
                button { r#type: "button", "aria-pressed": "{handover_on}", onclick: move |_| no_handover.set(false), "Hands over" }
                button { r#type: "button", "aria-pressed": "{no_handover_on}", onclick: move |_| no_handover.set(true), "Nobody takes over" }
                div { class: "gl-out", "data-out": "assistant-ready-step", "{last}" }
            }
            {frame(760, rsx! {
                AssistantReadyStep {
                    source_domains: domains(&["harbor-lane.com", "docs.harbor-lane.com"]),
                    brand: "Harbor lane",
                    aliases: domains(&["HL"]),
                    assistant_name: "Harbor lane Assistant",
                    handover_to,
                    on_finish: move |_| last.set("open dashboard".to_string()),
                    on_edit: move |step: u8| last.set(format!("edit step {step}")),
                    on_back: move |_| last.set("back".to_string()),
                    account_email: "ana@harbor-lane.com",
                }
            })}
        }
    }
}

#[component]
fn StartDemo() -> Element {
    let tools = vec![
        ToolChip {
            id: "doc".into(),
            entry_label: "Document".into(),
            prompt_template: "Write a document about ".into(),
        },
        ToolChip {
            id: "sheet".into(),
            entry_label: "Table".into(),
            prompt_template: "Build a table of ".into(),
        },
        ToolChip {
            id: "research".into(),
            entry_label: "Deep research".into(),
            prompt_template: "Research ".into(),
        },
    ];
    let apps = vec![ConnectedApp {
        id: "drive".into(),
        entry_name: "Shared drive".into(),
        app_mark: None,
    }];
    let suggestions = vec![
        Suggestion {
            id: "s1".into(),
            entry_label: "Summarise last week's orders".into(),
            prompt_text: "Summarise last week's orders".into(),
            artifact: None,
        },
        Suggestion {
            id: "s2".into(),
            entry_label: "Q4 pricing plan".into(),
            prompt_text: String::new(),
            artifact: Some("q4-pricing-plan.md".into()),
        },
    ];
    let artifacts = vec![Artifact {
        entry_name: "q4-pricing-plan.md".into(),
        output_kind: StartOutputKind::Document,
        preview_text: "Tier one holds at 49. Tier two moves to 79 for teams above ten seats."
            .into(),
    }];
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let current = mode();
    let loading = current == Mode::Loading;
    let empty = current == Mode::Empty;
    let apps = if empty { Vec::new() } else { apps };
    rsx! {
        Item { slug: "assistant-start-page", name: "AssistantStartPage",
            ModePick { slug: "assistant-start-page", mode, last: last() }
            {frame(720, rsx! {
                AssistantStartPage {
                    greeting_text: "Good afternoon, Ana",
                    tool_chip_list: tools,
                    connected_app_list: apps,
                    suggestion_list: suggestions,
                    suggested_artifacts: artifacts,
                    is_loading: loading,
                    is_empty: empty,
                    on_send: move |request: SendRequest| last.set(format!("send {}", request.text)),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                    on_attach: move |_| last.set("attach".to_string()),
                }
            })}
        }
    }
}

#[component]
fn HomeDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "none".to_string());
    let days: Vec<ActivityDay> = (0..63)
        .map(|i| {
            let (month, day) = match i {
                0..=29 => (9, i + 1),
                30..=60 => (10, i - 29),
                _ => (11, i - 60),
            };
            ActivityDay {
                date: format!("2026-{month:02}-{day:02}"),
                chat_count: ((i * 7) % 9) as u32,
            }
        })
        .collect();
    let recent = vec![
        RecentChat {
            id: "c1".into(),
            entry_title: "Pricing for the spring launch".into(),
            updated_at: "09:12".into(),
        },
        RecentChat {
            id: "c2".into(),
            entry_title: "Draft reply to the hosting vendor".into(),
            updated_at: "Yesterday".into(),
        },
        RecentChat {
            id: "c3".into(),
            entry_title: "Weekly churn summary".into(),
            updated_at: "Mon".into(),
        },
        RecentChat {
            id: "c4".into(),
            entry_title: "Onboarding copy review".into(),
            updated_at: "Sun".into(),
        },
    ];
    let files = vec![
        FileEntry {
            entry_name: "quarterly-board-deck-final-version-seven.pptx".into(),
            extension: "pptx".into(),
            created_at: "09:40".into(),
        },
        FileEntry {
            entry_name: "churn-by-cohort.xlsx".into(),
            extension: "xlsx".into(),
            created_at: "Tue".into(),
        },
        FileEntry {
            entry_name: "launch-notes.md".into(),
            extension: "md".into(),
            created_at: "Mon".into(),
        },
        FileEntry {
            entry_name: "invoice-review.pdf".into(),
            extension: "pdf".into(),
            created_at: "Fri".into(),
        },
    ];
    let stats = MonthStats {
        chats: MonthStat {
            entry_value: 42,
            change_vs_last: 12,
        },
        files_made: MonthStat {
            entry_value: 9,
            change_vs_last: -3,
        },
    };
    rsx! {
        Item { slug: "ai-workspace-home", name: "AiWorkspaceHome",
            ModePick { slug: "ai-workspace-home", mode, last: last() }
            {frame(980, rsx! {
                AiWorkspaceHome {
                    workspace_name: "Harbor lane",
                    account_name: "Harbor Co",
                    activity_days: days,
                    recent_chat_list: recent,
                    hidden_chat_count: 6,
                    usage_meter: UsageMeter { used_units: 2140, unit_limit: 2000, resets_on: "1 Dec".into() },
                    file_list: files,
                    hidden_file_count: 2,
                    month_stats: stats,
                    is_loading: mode() == Mode::Loading,
                    is_empty: mode() == Mode::Empty,
                    on_send: move |text: String| last.set(format!("send: {text}")),
                    on_navigate: move |label: String| last.set(format!("navigate: {label}")),
                    on_open_chat: move |id: String| last.set(format!("open chat: {id}")),
                    on_open_file: move |name: String| last.set(format!("open file: {name}")),
                }
            })}
        }
    }
}

#[component]
fn ProgressDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "none".to_string());
    // Running calls count from the browser's clock, so their start is read from it too.
    let mut wall = use_signal(|| None::<u64>);
    use_effect(move || {
        spawn(async move {
            let ms = document::eval("return Date.now();")
                .await
                .ok()
                .and_then(|value| value.as_f64())
                .unwrap_or(0.0);
            wall.set(Some(ms as u64));
        });
    });
    let now = wall().unwrap_or_else(gallery_now);
    let chat = vec![
        Message::User {
            id: "u1".into(),
            text: "Compare last quarter's hosting costs and draft a reply.".into(),
        },
        Message::Assistant {
            id: "a1".into(),
            parts: vec![
                Part::Tool(ToolCall {
                    id: "t1".into(),
                    tool_name: "read_sheet".into(),
                    app_label: Some("Sheets".into()),
                    entry_label: "Read the hosting ledger".into(),
                    started_at: Some(now - 8_000),
                    duration: Some(2_100),
                    arguments_data: "{\"range\": \"Q3!A1:F40\"}".into(),
                    result_data: Some("40 rows read".into()),
                    call_state: CallState::Done,
                    error: None,
                }),
                Part::Tool(ToolCall {
                    id: "t2".into(),
                    tool_name: "search_web".into(),
                    app_label: None,
                    entry_label: "Check the vendor's price page".into(),
                    started_at: Some(now - 3_000),
                    duration: None,
                    arguments_data: "{\"query\": \"hosting price 2026\"}".into(),
                    result_data: None,
                    call_state: CallState::Running,
                    error: None,
                }),
                Part::Tool(ToolCall {
                    id: "t3".into(),
                    tool_name: "send_draft".into(),
                    app_label: None,
                    entry_label: "Send the reply".into(),
                    started_at: None,
                    duration: Some(400),
                    arguments_data: "{\"to\": \"sales@vendor.example\"}".into(),
                    result_data: None,
                    call_state: CallState::Failed,
                    error: Some("Mail service timed out".into()),
                }),
            ],
        },
    ];
    let steps = vec![
        Step {
            id: "s1".into(),
            entry_label: "Read the ledger".into(),
            step_state: StepState::Done,
        },
        Step {
            id: "s2".into(),
            entry_label: "Collect price pages".into(),
            step_state: StepState::Done,
        },
        Step {
            id: "s3".into(),
            entry_label: "Draft the reply".into(),
            step_state: StepState::Done,
        },
        Step {
            id: "s4".into(),
            entry_label: "Check the vendor's price page".into(),
            step_state: StepState::Running,
        },
    ];
    let outputs = vec![Output {
        entry_name: "vendor-reply-draft-november-revision-two.md".into(),
        output_kind: OutputKind::Markdown,
        app_name: Some("Docs".into()),
    }];
    let usage = UsageRecord {
        upload_names: vec!["ledger-q3.csv".into()],
        app_names: vec!["Sheets".into()],
        web_search_pages: Some(4),
        skill_names: vec!["Vendor letters".into()],
    };
    rsx! {
        Item { slug: "assistant-chat-progress-panel", name: "AssistantChatProgressPanel",
            ModePick { slug: "assistant-chat-progress-panel", mode, last: last() }
            {frame(720, rsx! {
                AssistantChatProgressPanel {
                    active_chat: chat,
                    step_list: steps,
                    output_list: outputs,
                    usage_record: usage,
                    is_loading: mode() == Mode::Loading,
                    is_empty: mode() == Mode::Empty,
                    on_send: move |text: String| last.set(format!("send: {text}")),
                    on_navigate: move |label: String| last.set(format!("navigate: {label}")),
                    on_open_output: move |name: String| last.set(format!("open output: {name}")),
                }
            })}
        }
    }
}

#[component]
fn SidebarsDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "none".to_string());
    let mut project = use_signal(|| None::<String>);
    let mut open_chat = use_signal(|| "c1".to_string());
    let mut extra = use_signal(Vec::<ChatMessage>::new);
    let sections = vec![
        RailSection {
            id: "chat".into(),
            entry_label: "Chat".into(),
            icon_key: "chat".into(),
        },
        RailSection {
            id: "folders".into(),
            entry_label: "Folders".into(),
            icon_key: "folder".into(),
        },
        RailSection {
            id: "reports".into(),
            entry_label: "Reports".into(),
            icon_key: "reports".into(),
        },
    ];
    let projects = vec![
        Project {
            id: "p1".into(),
            entry_name: "Spring launch".into(),
        },
        Project {
            id: "p2".into(),
            entry_name: "Vendors".into(),
        },
    ];
    let chats = vec![
        ChatItem {
            id: "c1".into(),
            entry_title: "Launch pricing".into(),
            project_id: Some("p1".into()),
            is_pinned: true,
            updated_at: "09:00".into(),
        },
        ChatItem {
            id: "c2".into(),
            entry_title: "Press list".into(),
            project_id: Some("p1".into()),
            is_pinned: false,
            updated_at: "Mon".into(),
        },
        ChatItem {
            id: "c3".into(),
            entry_title: "Hosting renewal".into(),
            project_id: Some("p2".into()),
            is_pinned: false,
            updated_at: "Tue".into(),
        },
        ChatItem {
            id: "c4".into(),
            entry_title: "Loose notes".into(),
            project_id: None,
            is_pinned: false,
            updated_at: "Fri".into(),
        },
    ];
    let messages = vec![
        ChatMessage::User {
            id: "u1".into(),
            text: "Find the current price of the hosting plans and draft a summary.".into(),
        },
        ChatMessage::Assistant {
            id: "a1".into(),
            text: "Here is the comparison. The draft is ready to review.".into(),
            searches: vec![
                SearchStep {
                    query: "hosting plans 2026 price".into(),
                    state: SearchState::Done,
                    pages: 3,
                },
                SearchStep {
                    query: "vendor enterprise tier".into(),
                    state: SearchState::Done,
                    pages: 2,
                },
            ],
            outputs: vec!["hosting-summary.md".into()],
        },
    ];
    let other_messages = vec![
        ChatMessage::User {
            id: "u2".into(),
            text: "Draft the press list intro.".into(),
        },
        ChatMessage::Assistant {
            id: "a2".into(),
            text: "Here is a three-line intro.".into(),
            searches: vec![],
            outputs: vec![],
        },
    ];
    let shown_messages = if open_chat() == "c2" {
        other_messages
    } else {
        let mut all = messages;
        all.extend(extra());
        all
    };
    let files = vec![OutputFile {
        entry_name: "hosting-summary.md".into(),
        output_kind: SidebarOutputKind::Document,
        content: "Plan A: 49 a month.\nPlan B: 79 a month for teams above ten seats.".into(),
    }];
    rsx! {
        Item { slug: "assistant-chat-sidebars", name: "AssistantChatSidebars",
            ModePick { slug: "assistant-chat-sidebars", mode, last: last() }
            {frame(760, rsx! {
                AssistantChatSidebars {
                    workspace_name: "Harbor lane",
                    account_name: "Ana",
                    section_list: sections,
                    active_section_id: "chat".to_string(),
                    project_list: projects,
                    selected_project_id: project(),
                    chat_list: chats,
                    active_chat_id: open_chat(),
                    active_chat: shown_messages,
                    output_list: files,
                    is_loading: mode() == Mode::Loading,
                    is_empty: mode() == Mode::Empty,
                    is_generating: false,
                    on_send: move |text: String| {
                        last.set(format!("send: {text}"));
                        let n = extra.peek().len();
                        extra.with_mut(|list| {
                            list.push(ChatMessage::User {
                                id: format!("q{n}"),
                                text: text.clone(),
                            });
                            list.push(ChatMessage::Assistant {
                                id: format!("r{n}"),
                                text: format!("Here is a reply to: {text}. It types out as it arrives."),
                                searches: vec![],
                                outputs: vec![],
                            });
                        });
                    },
                    on_stop: move |_: ()| last.set("stop".to_string()),
                    on_navigate: move |label: String| last.set(format!("navigate: {label}")),
                    on_select_chat: move |id: String| {
                        last.set(format!("select chat: {id}"));
                        open_chat.set(id);
                    },
                    on_select_project: move |pick: Option<String>| {
                        last.set(format!("project: {}", pick.clone().unwrap_or_else(|| "all".into())));
                        project.set(pick);
                    },
                    on_open_output: move |name: String| last.set(format!("open output: {name}")),
                }
            })}
        }
    }
}

#[component]
fn WorkspaceChatDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "none".to_string());
    let mut open_chat = use_signal(|| "c1".to_string());
    let agents = vec![
        AgentStatus {
            id: "a1".into(),
            entry_name: "Researcher".into(),
            node_role: "Lead".into(),
            entry_state: AgentState::Working,
            current_step: Some("Reading three price pages".into()),
        },
        AgentStatus {
            id: "a2".into(),
            entry_name: "Analyst".into(),
            node_role: "Support".into(),
            entry_state: AgentState::Waiting,
            current_step: None,
        },
    ];
    let messages = vec![
        AgentMessage::User {
            id: "m1".into(),
            text: "Plan the Q1 vendor review and draft the brief.".into(),
        },
        AgentMessage::Agent {
            id: "m2".into(),
            author: "Lead".into(),
            text: "I will gather prices first, then fill the brief.".into(),
        },
    ];
    let other_messages = vec![AgentMessage::User {
        id: "m3".into(),
        text: "Outline the hiring plan for Q2.".into(),
    }];
    let shown_messages = if open_chat() == "c2" {
        other_messages
    } else {
        messages
    };
    let initial_brief = Brief {
        entry_title: "Q1 vendor review".into(),
        field_list: vec![
            BriefField {
                id: "goal".into(),
                entry_label: "Goal".into(),
                entry_value: "Cut hosting spend by 15 percent".into(),
                field_type: FieldType::Text,
            },
            BriefField {
                id: "budget".into(),
                entry_label: "Budget".into(),
                entry_value: "6000".into(),
                field_type: FieldType::Number,
            },
        ],
        file_list: vec![BriefFile {
            entry_name: "vendor-scorecard.xlsx".into(),
            file_kind: FileKind::Sheet,
        }],
        activity_log: vec![ActivityEntry {
            at: "09:14".into(),
            text: "Lead started the research".into(),
        }],
    };
    let mut fields = use_signal(|| initial_brief.field_list.clone());
    let brief = Brief {
        field_list: fields(),
        ..initial_brief.clone()
    };
    let artifacts = vec![FileArtifact {
        name: "vendor-scorecard.xlsx".into(),
        kind: FileArtifactKind::Sheet,
        body: "Vendor, Monthly, Seats\nAtlas, 4200, 12\nBrightline, 2100, 8".into(),
    }];
    let chats = vec![
        ChatItem {
            id: "c1".into(),
            entry_title: "Q1 vendor review".into(),
            project_id: None,
            is_pinned: true,
            updated_at: "09:14".into(),
        },
        ChatItem {
            id: "c2".into(),
            entry_title: "Hiring plan".into(),
            project_id: None,
            is_pinned: false,
            updated_at: "Mon".into(),
        },
    ];
    rsx! {
        Item { slug: "workspace-chat", name: "WorkspaceChat",
            ModePick { slug: "workspace-chat", mode, last: last() }
            {frame(760, rsx! {
                WorkspaceChat {
                    workspace_name: "Harbor lane",
                    account_name: "Ana",
                    chat_list: chats,
                    active_chat_id: open_chat(),
                    message_list: shown_messages,
                    agent_status_list: agents,
                    orchestrator_status: "Waiting for the analyst's scorecard".to_string(),
                    brief_record: Some(brief),
                    artifact_list: artifacts,
                    generate_eta: "Usually takes a few minutes.".to_string(),
                    is_loading: mode() == Mode::Loading,
                    is_empty: mode() == Mode::Empty,
                    on_send: move |text: String| last.set(format!("send: {text}")),
                    on_select_chat: move |id: String| {
                        last.set(format!("select chat: {id}"));
                        open_chat.set(id);
                    },
                    on_field_change: move |(key, value): (String, String)| {
                        let mut list = fields.peek().clone();
                        for field in list.iter_mut().filter(|f| f.id == key) {
                            field.entry_value = value.clone();
                        }
                        fields.set(list);
                        last.set(format!("field {key}: {value}"));
                    },
                    on_generate: move |_| last.set("generate".to_string()),
                    on_open_file: move |name: String| last.set(format!("open file: {name}")),
                    on_navigate: move |label: String| last.set(format!("navigate: {label}")),
                }
            })}
        }
    }
}

#[component]
fn SpendDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut working = use_signal(|| false);
    let mut last = use_signal(|| "none".to_string());
    let groups = vec![
        SplitGroup {
            entry_name: "Atlas".into(),
            parts: vec![
                SplitPart {
                    entry_name: "Large".into(),
                    value_dollars: 4200.0,
                },
                SplitPart {
                    entry_name: "Small".into(),
                    value_dollars: 1300.0,
                },
            ],
        },
        SplitGroup {
            entry_name: "Brightline".into(),
            parts: vec![
                SplitPart {
                    entry_name: "Large".into(),
                    value_dollars: 2100.0,
                },
                SplitPart {
                    entry_name: "Batch".into(),
                    value_dollars: 900.0,
                },
            ],
        },
        SplitGroup {
            entry_name: "Cobalt".into(),
            parts: vec![SplitPart {
                entry_name: "Small".into(),
                value_dollars: 600.0,
            }],
        },
    ];
    let levers = vec![
        Lever {
            id: "batch".into(),
            entry_label: "Batch jobs".into(),
            detail_text: "Run non-urgent work overnight.".into(),
            lever_kind: LeverKind::Range,
            monthly_base_dollars: 1200.0,
            max_saving_share: 0.5,
            max_percent: 100.0,
            entry_value: 40.0,
        },
        Lever {
            id: "cache".into(),
            entry_label: "Response cache".into(),
            detail_text: "Reuse answers to repeated questions.".into(),
            lever_kind: LeverKind::Switch,
            monthly_base_dollars: 2100.0,
            max_saving_share: 0.2,
            max_percent: 0.0,
            entry_value: 1.0,
        },
    ];
    let months: Vec<String> = (1..=6).map(|m| format!("M{m}")).collect();
    let trend = vec![TrendSeries {
        name: "Spend".into(),
        values: vec![
            Some(7.1),
            Some(7.9),
            Some(8.4),
            Some(9.0),
            Some(9.8),
            Some(10.2),
        ],
        tone: SeriesTone::Ink,
        emphasis: true,
        is_earlier: false,
    }];
    let answers = vec![
        Turn::Question {
            id: "q1".into(),
            text: "Where does the spend go this month?".into(),
        },
        Turn::Answer(Answer {
            id: "a1".into(),
            metric_cards: vec![
                Metric {
                    metric_label: "Spend".into(),
                    value: 9100.0,
                    change_percent: Some(23.0),
                },
                Metric {
                    metric_label: "Requests".into(),
                    value: 152000.0,
                    change_percent: Some(-2.0),
                },
            ],
            chart_block: Some(Chart::Split {
                title_text: "Spend by provider and model".into(),
                previous_total: Some(7400.0),
                groups,
            }),
            caption_text:
                "Atlas carries 60 percent of the spend. The Large model is the biggest single line [1]."
                    .into(),
            source_list: vec![SourceEntry {
                entry_id: "s1".into(),
                chip_text: "Usage export".into(),
                link_target: None,
                leading_mark: None,
                snippet: Some("Monthly totals by model.".into()),
            }],
            follow_up_list: vec!["Which requests drive the Large model?".into()],
        }),
        Turn::Answer(Answer {
            id: "a2".into(),
            metric_cards: vec![],
            chart_block: Some(Chart::Trend {
                title_text: "Weekly spend, thousands".into(),
                x_labels: months.clone(),
                series: trend,
            }),
            caption_text: "Spend grows about 4 percent a month.".into(),
            source_list: vec![],
            follow_up_list: vec![],
        }),
        Turn::Answer(Answer {
            id: "a3".into(),
            metric_cards: vec![],
            chart_block: Some(Chart::Plan {
                current_monthly: 9100.0,
                months: 6,
                growth_rate: 0.04,
                lever_list: levers,
            }),
            caption_text: "Two levers could save part of the monthly spend.".into(),
            source_list: vec![],
            follow_up_list: vec![],
        }),
    ];
    rsx! {
        Item { slug: "ai-spend-chat-planner", name: "AiSpendChatPlanner",
            div { class: "gl-row",
                ModePick { slug: "ai-spend-chat-planner", mode, last: last() }
                button { r#type: "button", "aria-pressed": "{working()}", onclick: move |_| working.toggle(), "Working" }
            }
            {frame(1000, rsx! {
                AiSpendChatPlanner {
                    turn_list: answers,
                    is_loading: mode() == Mode::Loading,
                    is_empty: mode() == Mode::Empty,
                    is_working: working(),
                    starter_questions: domains(&["Which provider costs the most?", "What would batch jobs save?"]),
                    on_ask: move |req: AskRequest| last.set(format!("ask: {}", req.question)),
                }
            })}
        }
    }
}

#[component]
fn SetupDemo() -> Element {
    let tools = vec![
        ConnectableTool {
            id: "drive".into(),
            name: "Shared drive".into(),
            summary: "Read and write team documents.".into(),
            connected: true,
        },
        ConnectableTool {
            id: "crm".into(),
            name: "Customer records".into(),
            summary: "Look up accounts and orders.".into(),
            connected: false,
        },
    ];
    let templates = vec![
        AgentTemplate {
            id: "support".into(),
            name: "Support".into(),
            description: "Answers customer questions from your docs.".into(),
        },
        AgentTemplate {
            id: "research".into(),
            name: "Research".into(),
            description: "Gathers sources and writes briefs.".into(),
        },
    ];
    let mut from_step_three = use_signal(|| false);
    let mut last = use_signal(|| "no action yet".to_string());
    let step_three_on = from_step_three();
    let step_two_on = !step_three_on;
    let start_step = if step_three_on { 3usize } else { 2usize };
    rsx! {
        Item { slug: "workspace-setup-checklist", name: "WorkspaceSetupChecklist",
            div { style: "display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin-bottom: 8px;",
                button { r#type: "button", "aria-pressed": "{step_two_on}", onclick: move |_| from_step_three.set(false), "Start at step 2" }
                button { r#type: "button", "aria-pressed": "{step_three_on}", onclick: move |_| from_step_three.set(true), "Start at step 3" }
                div { class: "gl-out", "data-out": "workspace-setup-checklist", "{last}" }
            }
            {frame(760, rsx! {
                WorkspaceSetupChecklist {
                    key: "{start_step}",
                    workspace_name: "Harbor lane",
                    account_first_name: "Ana",
                    default_step: start_step,
                    default_invites: domains(&["ben@harbor-lane.com"]),
                    connected_tools: tools,
                    templates,
                    on_complete: move |values: SetupValues| {
                        last.set(format!("complete {} with {}", values.workspace_name, values.agent_template));
                    },
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            })}
        }
    }
}
