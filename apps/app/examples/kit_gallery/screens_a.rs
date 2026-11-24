use dioxus::prelude::*;
use rv2_kit::components::grounded_reply::SourceEntry;
use rv2_kit::components::prompt_input_bar::ApprovalPolicy;
use rv2_kit::screens::ai_spend_chat_planner::{
    AiSpendChatPlanner, Answer, AskRequest, Chart, Metric, SplitGroup, SplitPart, Turn,
};
use rv2_kit::screens::ai_workspace_home::{
    ActivityDay, AiWorkspaceHome, FileEntry, MonthStat, MonthStats, RecentChat, UsageMeter,
};
use rv2_kit::screens::assistant_chat_progress_panel::{
    AssistantChatProgressPanel, CallState, Message, Output, OutputKind, Part, Step, StepState,
    ToolCall, UsageRecord,
};
use rv2_kit::screens::assistant_chat_sidebars::{
    ApprovalMode, AssistantChatSidebars, ChatItem, ChatMessage, OutputFile,
    OutputKind as SidebarOutputKind, Project, SearchState, SearchStep, Section as RailSection,
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
    FileKind, Product, WorkspaceChat,
};
use rv2_kit::screens::workspace_frame_inset::{
    Artifact as FileArtifact, ArtifactKind as FileArtifactKind,
};
use rv2_kit::screens::workspace_setup_checklist::{
    AgentTemplate, ConnectableTool, SetupValues, WorkspaceSetupChecklist,
};

use super::screens_b::{Mode, ModePick};
use super::{Item, Section};

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
                    default_domains: domains(&["harbor-lane.com"]),
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
    let mut known = use_signal(|| true);
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
                    website_domain: "northwind.com",
                    default_names: if known_on { domains(&["Northwind", "Northwind Co."]) } else { Vec::new() },
                    on_continue: move |names: Vec<String>| last.set(format!("continue {}", names.join(" | "))),
                    on_back: move |_| last.set("back".to_string()),
                    on_logout: move |_| last.set("log out".to_string()),
                    account_email: "ana@northwind.com",
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
    let tools = [
        (
            "company",
            "Company knowledge",
            "Search company knowledge for ",
        ),
        ("doc", "Document", "Write a document about "),
        ("slides", "Presentation", "Make a presentation about "),
        ("table", "Table", "Build a table of "),
        ("image", "Image", "Create an image of "),
        ("research", "Deep research", "Research "),
        ("chart", "Data visualisation", "Chart "),
    ]
    .into_iter()
    .map(|(id, label, template)| ToolChip {
        id: id.into(),
        entry_label: label.into(),
        prompt_template: template.into(),
    })
    .collect::<Vec<_>>();
    let apps = ["Mail", "Drive", "Calendar"]
        .into_iter()
        .enumerate()
        .map(|(index, name)| ConnectedApp {
            id: format!("app-{index}"),
            entry_name: name.into(),
            app_mark: Some(name.chars().take(1).collect()),
        })
        .collect::<Vec<_>>();
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
                    invitation_text: "What should we work on today?",
                    working_folder: "Harbor lane",
                    approval_mode: "Automatic",
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
            {frame(720, rsx! {
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

/// A tool call that finished, with its arguments and result.
fn finished_call(id: &str, tool: &str, label: &str, result: &str, duration: u64) -> Part {
    Part::Tool(ToolCall {
        id: id.into(),
        tool_name: tool.into(),
        app_label: None,
        entry_label: label.into(),
        started_at: None,
        duration: Some(duration),
        arguments_data: format!("{{\"step\": \"{label}\"}}"),
        result_data: Some(result.into()),
        call_state: CallState::Done,
        error: None,
    })
}

#[component]
fn ProgressDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "none".to_string());
    let chat = vec![
        Message::User {
            id: "u1".into(),
            text: "Compare last quarter's hosting costs and draft a reply.".into(),
        },
        Message::Assistant {
            id: "a1".into(),
            parts: vec![
                finished_call("t1", "read_sheet", "Read the hosting ledger", "40 rows read", 2_100),
                Part::Text("The ledger shows the vendor raised the Pro plan by 18% in October.".into()),
                finished_call("t2", "search_web", "Check the vendor's price page", "Price page read", 1_400),
                Part::Text("Three points for the reply: the new price, the renewal date, and the seat limit that changed.".into()),
            ],
        },
        Message::User {
            id: "u2".into(),
            text: "Keep it under 120 words and add the renewal date.".into(),
        },
        Message::Assistant {
            id: "a2".into(),
            parts: vec![
                finished_call("t3", "write_draft", "Draft the reply", "Draft saved, 96 words", 900),
                Part::Text("The draft is saved. It runs 96 words and names the renewal date, 30 March.".into()),
            ],
        },
    ];
    let steps = [
        "Read the ledger",
        "Collect price pages",
        "Check the seat limits",
        "Draft the reply",
        "Save the draft",
    ]
    .iter()
    .enumerate()
    .map(|(n, label)| Step {
        id: format!("s{}", n + 1),
        entry_label: label.to_string(),
        step_state: StepState::Done,
    })
    .collect::<Vec<_>>();
    let outputs = vec![
        Output {
            entry_name: "hosting-reply-november.docx".into(),
            output_kind: OutputKind::Document,
            app_name: Some("Docs".into()),
            preview_lines: vec![
                "Hi Harbor team, the Pro plan rises 18% on 1 December.".into(),
                "Reply by Friday to keep your current seats.".into(),
            ],
        },
        Output {
            entry_name: "hosting-ledger-q3.xlsx".into(),
            output_kind: OutputKind::Sheet,
            app_name: Some("Sheets".into()),
            preview_lines: vec![],
        },
        Output {
            entry_name: "vendor-terms-summary.docx".into(),
            output_kind: OutputKind::Document,
            app_name: Some("Word".into()),
            preview_lines: vec![],
        },
        Output {
            entry_name: "vendor-reply-draft.md".into(),
            output_kind: OutputKind::Markdown,
            app_name: None,
            preview_lines: vec![],
        },
    ];
    let usage = UsageRecord {
        upload_names: vec![
            "ledger-q3.csv".into(),
            "terms-2025.pdf".into(),
            "invoice-oct.pdf".into(),
        ],
        app_names: vec!["Sheets".into(), "Docs".into(), "Mail".into()],
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
                    approval_policy: ApprovalPolicy::AutoSafe,
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
    let mut open_chat = use_signal(|| "c0".to_string());
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
        RailSection {
            id: "integrations".into(),
            entry_label: "Integrations".into(),
            icon_key: "plug".into(),
        },
        RailSection {
            id: "settings".into(),
            entry_label: "Settings".into(),
            icon_key: "gear".into(),
        },
    ];
    let projects = vec![
        Project {
            id: "p1".into(),
            entry_name: "Growth".into(),
        },
        Project {
            id: "p2".into(),
            entry_name: "Operations".into(),
        },
        Project {
            id: "p3".into(),
            entry_name: "Finance".into(),
        },
    ];
    let chat = |id: &str, title: &str, project: Option<&str>, pinned: bool| ChatItem {
        id: id.into(),
        entry_title: title.into(),
        project_id: project.map(str::to_string),
        is_pinned: pinned,
        updated_at: "Mon".into(),
    };
    let chats = vec![
        chat("c0", "Hosting shortlist", None, true),
        chat("c1", "Hiring plan Q1", Some("p1"), false),
        chat("c2", "Churn analysis", Some("p1"), false),
        chat("c3", "Pricing tests", Some("p1"), false),
        chat("c4", "Vendor renewals", Some("p2"), false),
        chat("c5", "Office move", Some("p2"), false),
        chat("c6", "Budget review", Some("p3"), false),
        chat("c7", "Forecast notes", Some("p3"), false),
    ];
    let messages = vec![
        ChatMessage::User {
            id: "u1".into(),
            text: "Compare hosting plans for a ten-seat team.".into(),
        },
        ChatMessage::Assistant {
            id: "a1".into(),
            text: "Which region do you run in? I can narrow the list.".into(),
            searches: vec![],
            outputs: vec![],
        },
        ChatMessage::User {
            id: "u2".into(),
            text: "Europe, and we need single sign-on.".into(),
        },
        ChatMessage::Assistant {
            id: "a2".into(),
            text: "Two plans fit. The team plan costs 79 a month and includes single sign-on; the starter plan does not.".into(),
            searches: vec![
                SearchStep {
                    query: "hosting plans EU region price".into(),
                    state: SearchState::Done,
                    pages: 3,
                },
                SearchStep {
                    query: "single sign-on team plan".into(),
                    state: SearchState::Done,
                    pages: 2,
                },
                SearchStep {
                    query: "starter plan seat limits".into(),
                    state: SearchState::Done,
                    pages: 2,
                },
            ],
            outputs: vec!["hosting-summary.md".into(), "pricing-sheet.xlsx".into()],
        },
    ];
    let mut shown_messages = messages;
    shown_messages.extend(extra());
    let files = vec![
        OutputFile {
            entry_name: "hosting-summary.md".into(),
            output_kind: SidebarOutputKind::Document,
            content: "Plan A: 49 a month for up to five seats.\n\nPlan B: 79 a month for teams of up to 25, with single sign-on.".into(),
        },
        OutputFile {
            entry_name: "pricing-sheet.xlsx".into(),
            output_kind: SidebarOutputKind::Sheet,
            content: "Plan, Monthly, Seats, Region\nStarter, 49, 5, EU\nTeam, 79, 25, EU\nBusiness, 149, 80, EU\nEnterprise, 399, 500, EU".into(),
        },
    ];
    let open_tabs = vec![
        "pricing-sheet.xlsx".to_string(),
        "hosting-summary.md".to_string(),
    ];
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
                    open_tab_list: open_tabs,
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
    let agents = vec![
        AgentStatus {
            id: "r".into(),
            entry_name: "Researcher".into(),
            node_role: "Worker".into(),
            entry_state: AgentState::Working,
            current_step: Some("Reading three price pages".into()),
        },
        AgentStatus {
            id: "w".into(),
            entry_name: "Writer".into(),
            node_role: "Worker".into(),
            entry_state: AgentState::Working,
            current_step: Some("Drafting the channel copy".into()),
        },
        AgentStatus {
            id: "a".into(),
            entry_name: "Analyst".into(),
            node_role: "Worker".into(),
            entry_state: AgentState::Working,
            current_step: Some("Checking the margins".into()),
        },
        AgentStatus {
            id: "b".into(),
            entry_name: "Budget analyst".into(),
            node_role: "Worker".into(),
            entry_state: AgentState::Done,
            current_step: Some("Budget table delivered".into()),
        },
        AgentStatus {
            id: "d".into(),
            entry_name: "Designer".into(),
            node_role: "Worker".into(),
            entry_state: AgentState::Waiting,
            current_step: Some("Needs a palette choice".into()),
        },
    ];
    let messages = vec![
        AgentMessage::User {
            id: "m1".into(),
            text: "Plan the Q1 launch and draft the copy for each channel.".into(),
        },
        AgentMessage::Agent {
            id: "m2".into(),
            author: "Orchestrator".into(),
            text: "Understood. I will split the launch across five workers.".into(),
            work_time: Some("Worked for 2 min".into()),
            step_list: Vec::new(),
        },
        AgentMessage::Agent {
            id: "m3".into(),
            author: "Orchestrator".into(),
            text: "Which selling angle should the copy lead with: price, speed or support?".into(),
            work_time: None,
            step_list: Vec::new(),
        },
    ];
    let initial_brief = Brief {
        entry_title: "Q1 launch brief".into(),
        product: Some(Product {
            entry_name: "Harbor Pro".into(),
            entry_price: "49 a month".into(),
            sales_line: "312 sold in October, 41 of them to small teams".into(),
            is_top_seller: true,
        }),
        field_list: vec![
            BriefField {
                id: "angle".into(),
                entry_label: "Selling angle".into(),
                entry_value: "Speed".into(),
                field_type: FieldType::Choice,
                option_list: domains(&["Price", "Speed", "Support"]),
            },
            BriefField {
                id: "audience".into(),
                entry_label: "Audience".into(),
                entry_value: "Small teams".into(),
                field_type: FieldType::Chips,
                option_list: domains(&["Small teams", "Agencies", "Solo founders", "Enterprise"]),
            },
            BriefField {
                id: "channels".into(),
                entry_label: "Channels".into(),
                entry_value: "Email, Social".into(),
                field_type: FieldType::Switches,
                option_list: domains(&["Email", "Social", "Search"]),
            },
            BriefField {
                id: "palette".into(),
                entry_label: "Palette".into(),
                entry_value: String::new(),
                field_type: FieldType::Swatches,
                option_list: domains(&["Harbor", "Dune", "Fern", "Ink"]),
            },
            BriefField {
                id: "formats".into(),
                entry_label: "Formats".into(),
                entry_value: "Long form".into(),
                field_type: FieldType::Choice,
                option_list: domains(&["Long form", "Short form", "Carousel"]),
            },
            BriefField {
                id: "launch".into(),
                entry_label: "Launch date".into(),
                entry_value: "2027-01-12".into(),
                field_type: FieldType::Date,
                option_list: Vec::new(),
            },
            BriefField {
                id: "markets".into(),
                entry_label: "Markets".into(),
                entry_value: "DACH".into(),
                field_type: FieldType::MultiChoice,
                option_list: domains(&["DACH", "UK", "Nordics"]),
            },
            BriefField {
                id: "budget".into(),
                entry_label: "Budget".into(),
                entry_value: "6000".into(),
                field_type: FieldType::Number,
                option_list: Vec::new(),
            },
        ],
        file_list: vec![
            BriefFile {
                entry_name: "launch-plan.md".into(),
                file_kind: FileKind::Document,
            },
            BriefFile {
                entry_name: "pricing-sheet.xlsx".into(),
                file_kind: FileKind::Sheet,
            },
            BriefFile {
                entry_name: "channel-copy.docx".into(),
                file_kind: FileKind::Document,
            },
        ],
        activity_log: vec![
            ActivityEntry {
                at: "09:42".into(),
                text: "Budget analyst delivered the table".into(),
            },
            ActivityEntry {
                at: "09:38".into(),
                text: "Researcher read three pricing pages".into(),
            },
            ActivityEntry {
                at: "09:31".into(),
                text: "Orchestrator split the brief".into(),
            },
        ],
    };
    let mut fields = use_signal(|| initial_brief.field_list.clone());
    let brief = Brief {
        field_list: fields(),
        ..initial_brief.clone()
    };
    let artifacts = vec![
        FileArtifact {
            name: "launch-plan.md".into(),
            kind: FileArtifactKind::Document,
            body: "Goal: ten trials in the first month.\n\nAngle: speed for small teams.".into(),
        },
        FileArtifact {
            name: "pricing-sheet.xlsx".into(),
            kind: FileArtifactKind::Sheet,
            body: "Plan, Monthly, Seats\nStarter, 49, 5\nTeam, 79, 12".into(),
        },
    ];
    rsx! {
        Item { slug: "workspace-chat", name: "WorkspaceChat",
            ModePick { slug: "workspace-chat", mode, last: last() }
            {frame(720, rsx! {
                WorkspaceChat {
                    message_list: messages,
                    agent_status_list: agents,
                    orchestrator_status: "Orchestrator: splits the goal, checks each worker's output and asks you the open questions.",
                    brief_record: Some(brief),
                    artifact_list: artifacts,
                    generate_eta: "Usually takes a few minutes.".to_string(),
                    approval_mode: ApprovalMode::Auto,
                    is_loading: mode() == Mode::Loading,
                    is_empty: mode() == Mode::Empty,
                    on_send: move |text: String| last.set(format!("send: {text}")),
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
                    value_dollars: 2400.0,
                },
                SplitPart {
                    entry_name: "Small".into(),
                    value_dollars: 1100.0,
                },
            ],
        },
        SplitGroup {
            entry_name: "Nimbus".into(),
            parts: vec![
                SplitPart {
                    entry_name: "Large".into(),
                    value_dollars: 1800.0,
                },
                SplitPart {
                    entry_name: "Mid".into(),
                    value_dollars: 900.0,
                },
                SplitPart {
                    entry_name: "Small".into(),
                    value_dollars: 600.0,
                },
            ],
        },
        SplitGroup {
            entry_name: "Quill".into(),
            parts: vec![
                SplitPart {
                    entry_name: "Mid".into(),
                    value_dollars: 1200.0,
                },
                SplitPart {
                    entry_name: "Small".into(),
                    value_dollars: 500.0,
                },
            ],
        },
    ];
    let answers = vec![
        Turn::Question {
            id: "q1".into(),
            text: "Where does the spend go this month?".into(),
        },
        Turn::Answer(Answer {
            id: "a1".into(),
            text: "Atlas carries 41 percent of the 8,500 dollars spent this month, and its Large model is the biggest single line. Spend is up 15 percent on last month.".into(),
            metric_cards: vec![
                Metric {
                    metric_label: "Spend".into(),
                    value: 8500.0,
                    change_percent: Some(15.0),
                },
                Metric {
                    metric_label: "Requests".into(),
                    value: 152000.0,
                    change_percent: Some(-2.0),
                },
                Metric {
                    metric_label: "Cost per 1,000".into(),
                    value: 55.9,
                    change_percent: Some(18.0),
                },
            ],
            chart_block: Some(Chart::Split {
                title_text: "Spend by provider and model".into(),
                previous_total: Some(7400.0),
                groups,
            }),
            caption_text: "Spend this month by provider and model. Click a bar to ask about it.".into(),
            source_list: vec![
                SourceEntry {
                    entry_id: "s1".into(),
                    chip_text: "Usage export".into(),
                    link_target: None,
                    leading_mark: None,
                    snippet: Some("Monthly totals by model.".into()),
                },
                SourceEntry {
                    entry_id: "s2".into(),
                    chip_text: "Billing ledger".into(),
                    link_target: None,
                    leading_mark: None,
                    snippet: Some("Invoices for each provider.".into()),
                },
            ],
            follow_up_list: vec![
                "What would batch jobs save?".into(),
                "Which requests drive the Large model?".into(),
                "Compare with last month".into(),
            ],
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
        ConnectableTool {
            id: "calendar".into(),
            name: "Calendar".into(),
            summary: "Schedule meetings and read notes.".into(),
            connected: false,
        },
        ConnectableTool {
            id: "mail".into(),
            name: "Mail".into(),
            summary: "Read and send from shared inboxes.".into(),
            connected: false,
        },
        ConnectableTool {
            id: "forms".into(),
            name: "Forms".into(),
            summary: "Collect answers in a sheet.".into(),
            connected: false,
        },
        ConnectableTool {
            id: "invoices".into(),
            name: "Invoices".into(),
            summary: "Issue and track invoices.".into(),
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
            {frame(720, rsx! {
                WorkspaceSetupChecklist {
                    key: "{start_step}",
                    workspace_name: "Harbor lane",
                    account_first_name: "Ana",
                    default_step: start_step,
                    default_invites: domains(&["ben@harbor-lane.com"]),
                    connected_tools: tools,
                    templates,
                    crumbs: domains(&["Workspace home", "Set up Harbor lane"]),
                    on_complete: move |values: SetupValues| {
                        last.set(format!("complete {} with {}", values.workspace_name, values.agent_template));
                    },
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            })}
        }
    }
}
