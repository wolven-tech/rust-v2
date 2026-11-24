use dioxus::prelude::*;

use super::{Item, Section};

#[component]
pub fn Inputs() -> Element {
    rsx! {
        Section { id: "inputs", title: "Inputs",
            ModelSelectorDemo {}
            MultiSelectLookupDemo {}
            MultiStepWizardFormDemo {}
            PeriodPickerDemo {}
            TrustLevelSelectorDemo {}
            FillInPromptTemplatesDemo {}
            RunControlComposerDemo {}
            FileDropAreaDemo {}
            ConnectorPickerDemo {}
            ConnectorSettingsCardDemo {}
            OnboardingChecklistPanelDemo {}
            ChargePhaseBuilderDemo {}
        }
    }
}

fn model_demo_options() -> Vec<rv2_kit::components::model_selector::ModelOption> {
    use rv2_kit::components::model_selector::ModelOption;
    let rated =
        |id: &str, name: &str, note: &str, ranks: [u8; 3], seconds: f64, price: f64| ModelOption {
            option_note: Some(note.into()),
            speed_rank: Some(ranks[0]),
            quality_rank: Some(ranks[1]),
            cost_rank: Some(ranks[2]),
            first_token_time: Some(seconds),
            price_per_million: Some(price),
            ..ModelOption::new(id, name)
        };
    vec![
        rated("fast", "Model Fast", "Quick replies", [5, 2, 2], 0.6, 0.5),
        rated(
            "deep",
            "Model Deep",
            "Careful reasoning",
            [2, 5, 4],
            1.9,
            15.0,
        ),
        rated("long", "Model Long", "Long documents", [1, 4, 5], 3.1, 30.0),
        ModelOption {
            option_note: Some("Routes each message".into()),
            ..ModelOption::new("auto", "Model Auto")
        },
    ]
}

#[component]
fn ModelSelectorDemo() -> Element {
    use rv2_kit::components::model_selector::{MenuSide, ModelSelector, ThinkingEffort};
    let mut last = use_signal(String::new);
    let mut model = use_signal(|| Some("deep".to_string()));
    let mut thinking = use_signal(|| true);
    let mut web_search = use_signal(|| false);
    let mut effort = use_signal(|| ThinkingEffort::Medium);
    rsx! {
        Item { slug: "model-selector", name: "ModelSelector",
            ModelSelector {
                model_options: model_demo_options(),
                selected_model_id: model(),
                on_model_change: move |id: String| {
                    last.set(format!("model {id}"));
                    model.set(Some(id));
                },
                thinking_on: Some(thinking()),
                on_thinking_change: move |on: bool| {
                    last.set(format!("thinking {on}"));
                    thinking.set(on);
                },
                web_search_on: Some(web_search()),
                on_web_search_change: move |on: bool| {
                    last.set(format!("web search {on}"));
                    web_search.set(on);
                },
                thinking_effort: Some(effort()),
                on_effort_change: move |e: ThinkingEffort| {
                    last.set(format!("effort {e:?}"));
                    effort.set(e);
                },
            }
            div { class: "gl-out", "data-out": "model-selector", "{last()}" }
            div { class: "gl-open-demo", style: "min-height: 640px;",
                div { style: "display: flex; flex-direction: column; justify-content: space-between; gap: var(--kit-space-3); min-height: 600px; padding: var(--kit-space-3); border: 1px solid var(--kit-rule-soft); border-radius: var(--kit-radius-card); background: var(--kit-surface); box-sizing: border-box;",
                    p { style: "margin: 0; color: var(--kit-ink-muted); font-size: var(--kit-text-lg);", "Ask a follow-up about the renewal terms" }
                    div { style: "display: flex; align-items: center;",
                        ModelSelector {
                            model_options: model_demo_options(),
                            selected_model_id: Some("deep".to_string()),
                            initial_highlight: Some("fast".to_string()),
                            initial_open: true,
                            menu_side: MenuSide::Top,
                            thinking_on: Some(true),
                            web_search_on: Some(false),
                            thinking_effort: Some(ThinkingEffort::Medium),
                        }
                    }
                }
            }
        }
    }
}

fn approver_options() -> Vec<rv2_kit::components::multi_select_lookup::LookupOption> {
    use rv2_kit::components::multi_select_lookup::{LookupOption, OptionImage};
    let person = |value: &str, label: &str, role: &str, group: &str| LookupOption {
        option_hint: Some(role.into()),
        option_image: Some(OptionImage::Initials),
        option_group: Some(group.into()),
        match_terms: vec![label.to_lowercase()],
        ..LookupOption::new(value, label)
    };
    vec![
        person("ada", "Ada Lovelace", "Staff engineer", "Platform"),
        person("ben", "Ben Carter", "Engineer", "Platform"),
        person("cy", "Cy Mora", "Platform lead", "Platform"),
        person("dana", "Dana Ruiz", "Support lead", "Support"),
        person("eli", "Eli Brandt", "Support agent", "Support"),
        person("farah", "Farah Nasser", "Operations manager", "Operations"),
        person("gus", "Gus Holm", "Operations analyst", "Operations"),
    ]
}

#[component]
fn MultiSelectLookupDemo() -> Element {
    use rv2_kit::components::multi_select_lookup::{LookupOption, MultiSelectLookup};
    let mut last = use_signal(String::new);
    let mut picked = use_signal(|| vec!["ben".to_string()]);
    let options = approver_options;
    rsx! {
        Item { slug: "multi-select-lookup", name: "MultiSelectLookup",
            MultiSelectLookup {
                option_list: options(),
                selected_values: Some(picked()),
                field_caption: Some("Approvers".to_string()),
                empty_hint: "Add approvers",
                on_selection_change: move |(vals, _): (Vec<String>, Vec<LookupOption>)| {
                    last.set(format!("selected {}", vals.join(",")));
                    picked.set(vals);
                },
            }
            div { class: "gl-out", "data-out": "multi-select-lookup", "{last()}" }
            div { class: "gl-open-demo", style: "min-height: 560px;",
                MultiSelectLookup {
                    option_list: options(),
                    selected_values: Some(vec!["ben".to_string(), "dana".to_string()]),
                    field_caption: Some("Approvers".to_string()),
                    empty_hint: "Add approvers",
                    start_open: true,
                    on_selection_change: move |(vals, _): (Vec<String>, Vec<LookupOption>)| last.set(format!("open selected {}", vals.join(","))),
                }
            }
        }
    }
}

#[component]
fn MultiStepWizardFormDemo() -> Element {
    use rv2_affect::text_field::TextField;
    use rv2_kit::components::multi_step_wizard_form::{
        FieldMessage, FormValues, MultiStepWizardForm, ReviewRow, StepContext, Task, WizardField,
        WizardReview, WizardStep,
    };
    let mut last = use_signal(String::new);
    let steps = vec![
        WizardStep {
            step_id: "workspace".into(),
            step_title: "Workspace".into(),
            step_heading: Some("Who is this workspace for?".into()),
            step_description: Some("Name it and choose its address.".into()),
            render_step: Callback::new(move |ctx: StepContext| {
                let mut values = ctx.set_form_values;
                let name = ctx.form_values.get("name").cloned().unwrap_or_default();
                let address = ctx.form_values.get("address").cloned().unwrap_or_default();
                rsx! {
                    TextField {
                        id: "wizard-name",
                        name: "name",
                        label: rsx! { "Name" },
                        value: name,
                        on_input: move |e: FormEvent| {
                            values.write().insert("name".to_string(), e.value());
                        },
                    }
                    WizardField {
                        field_id: "wizard-address",
                        field_name: "address",
                        label: "Address",
                        prefix: "workspace.example/",
                        value: address,
                        hint: "Lowercase letters and dashes.",
                        on_input: move |v: String| {
                            values.write().insert("address".to_string(), v);
                        },
                    }
                }
            }),
            check_step: Some(Callback::new(|v: FormValues| {
                if v.get("name").is_none_or(|s| s.trim().is_empty()) {
                    vec![FieldMessage {
                        field: "name".into(),
                        text: "Name is required".into(),
                    }]
                } else {
                    vec![]
                }
            })),
        },
        WizardStep {
            step_id: "team".into(),
            step_title: "Team".into(),
            step_heading: Some("Who else will work here?".into()),
            step_description: None,
            render_step: Callback::new(
                |_ctx: StepContext| rsx! { p { "Invite teammates from settings after setup." } },
            ),
            check_step: None,
        },
        WizardStep {
            step_id: "plan".into(),
            step_title: "Plan".into(),
            step_heading: Some("Pick a plan".into()),
            step_description: None,
            render_step: Callback::new(
                |_ctx: StepContext| rsx! { p { "Plans can change later." } },
            ),
            check_step: None,
        },
        WizardStep {
            step_id: "review".into(),
            step_title: "Review".into(),
            step_heading: Some("Check the details".into()),
            step_description: None,
            render_step: Callback::new(move |ctx: StepContext| {
                let name = ctx.form_values.get("name").cloned().unwrap_or_default();
                let address = ctx.form_values.get("address").cloned().unwrap_or_default();
                rsx! {
                    WizardReview {
                        rows: vec![
                            ReviewRow { row_label: "Name".into(), row_value: name, edit_step: Some(0) },
                            ReviewRow { row_label: "Address".into(), row_value: address, edit_step: Some(0) },
                            ReviewRow { row_label: "Plan".into(), row_value: "Starter".into(), edit_step: Some(2) },
                        ],
                        go_to: ctx.go_to,
                    }
                }
            }),
            check_step: None,
        },
    ];
    rsx! {
        Item { slug: "multi-step-wizard-form", name: "MultiStepWizardForm",
            MultiStepWizardForm {
                step_list: steps,
                final_label: "Create workspace",
                on_complete: move |v: FormValues| {
                    last.set(format!("complete {}", v.get("name").cloned().unwrap_or_default()));
                    let task: Task = Box::pin(async { Ok(()) });
                    task
                },
            }
            div { class: "gl-out", "data-out": "multi-step-wizard-form", "{last()}" }
        }
    }
}

#[component]
fn PeriodPickerDemo() -> Element {
    use rv2_affect::date_picker::Day;
    use rv2_kit::components::period_picker::{
        ComparisonMode, DateRange, PeriodPicker, resolve_builtin,
    };
    let mut last = use_signal(String::new);
    let day = |d: u8| Day {
        year: 2026,
        month: 11,
        day: d,
    };
    rsx! {
        Item { slug: "period-picker", name: "PeriodPicker",
            div { class: "gl-out", "data-out": "period-picker", "{last()}" }
            div { class: "gl-open-demo", style: "min-height: 560px;",
                PeriodPicker {
                    selected_range: Some(resolve_builtin("last_30", day(21))),
                    anchor_today: Some(day(21)),
                    starting_comparison: ComparisonMode::PreviousPeriod,
                    on_apply: move |(r, cmp, mode): (DateRange, Option<DateRange>, ComparisonMode)| {
                        last.set(format!("apply {}-{} vs {} {mode:?}", r.from.day, r.to.day, cmp.is_some()));
                    },
                    start_expanded: true,
                }
            }
        }
    }
}

#[component]
fn TrustLevelSelectorDemo() -> Element {
    use rv2_kit::components::trust_level_selector::{Capability, RiskClass, TrustLevelSelector};
    let mut last = use_signal(String::new);
    let mut level = use_signal(|| Some(1u8));
    let cap = |id: &str, name: &str, risk: RiskClass| Capability {
        id: id.into(),
        capability_name: name.into(),
        capability_note: None,
        risk_class: risk,
        capability_mark: name.chars().next().map(|c| c.to_string()),
    };
    rsx! {
        Item { slug: "trust-level-selector", name: "TrustLevelSelector",
            TrustLevelSelector {
                capabilities: vec![
                    cap("read", "Read files", RiskClass::Routine),
                    cap("search", "Search the web", RiskClass::Routine),
                    cap("draft", "Draft replies", RiskClass::Routine),
                    cap("send", "Send email", RiskClass::Sensitive),
                    cap("spend", "Spend credits", RiskClass::Sensitive),
                    cap("delete", "Delete data", RiskClass::Irreversible),
                ],
                policy_level: level(),
                on_policy_change: move |l: u8| {
                    last.set(format!("level {l}"));
                    level.set(Some(l));
                },
                on_always_ask_change: move |ids: Vec<String>| last.set(format!("always ask {}", ids.len())),
            }
            div { class: "gl-out", "data-out": "trust-level-selector", "{last()}" }
        }
    }
}

#[component]
fn FillInPromptTemplatesDemo() -> Element {
    use rv2_kit::components::fill_in_prompt_templates::{
        BlankSpec, FillInPromptTemplates, Template,
    };
    use std::collections::BTreeMap;
    let mut last = use_signal(String::new);
    let mut selected = use_signal(|| Some("t1".to_string()));
    let blanks = BTreeMap::from([
        (
            "client".to_string(),
            BlankSpec {
                hint_text: Some("Client name".into()),
                fallback_value: None,
            },
        ),
        (
            "account".to_string(),
            BlankSpec {
                hint_text: Some("Account owner".into()),
                fallback_value: None,
            },
        ),
        (
            "tone".to_string(),
            BlankSpec {
                hint_text: Some("Tone".into()),
                fallback_value: Some("friendly".into()),
            },
        ),
        (
            "topic".to_string(),
            BlankSpec {
                hint_text: Some("Topic".into()),
                fallback_value: None,
            },
        ),
    ]);
    let templates = vec![
        Template {
            template_id: "t1".into(),
            template_title: "Client summary".into(),
            group_label: Some("Sales".into()),
            group_key: Some("sales".into()),
            template_body: "Summarise the last call with {client} for {account} in a {tone} tone, covering {topic}.".into(),
            blank_specs: blanks,
        },
        Template {
            template_id: "t4".into(),
            template_title: "Ticket reply".into(),
            group_label: Some("Support".into()),
            group_key: Some("support".into()),
            template_body: "Reply to {customer} about ticket {ticket}.".into(),
            blank_specs: BTreeMap::new(),
        },
        Template {
            template_id: "t2".into(),
            template_title: "Release note".into(),
            group_label: Some("Product".into()),
            group_key: None,
            template_body: "Write a release note for version 2.".into(),
            blank_specs: BTreeMap::new(),
        },
        Template {
            template_id: "t3".into(),
            template_title: "Follow-up email".into(),
            group_label: Some("Sales".into()),
            group_key: Some("sales".into()),
            template_body: "Draft a follow-up to {client} about {topic}.".into(),
            blank_specs: BTreeMap::from([
                (
                    "client".to_string(),
                    BlankSpec {
                        hint_text: Some("Client name".into()),
                        fallback_value: None,
                    },
                ),
                (
                    "topic".to_string(),
                    BlankSpec {
                        hint_text: Some("Topic".into()),
                        fallback_value: None,
                    },
                ),
            ]),
        },
    ];
    rsx! {
        Item { slug: "fill-in-prompt-templates", name: "FillInPromptTemplates",
            FillInPromptTemplates {
                template_list: templates,
                selected_id: selected(),
                on_select: move |id: String| {
                    last.set(format!("select {id}"));
                    selected.set(Some(id));
                },
                on_submit: move |(id, _, fills): (String, Template, BTreeMap<String, String>)| last.set(format!("submit {id} {fills:?}")),
                on_copy: move |t: String| last.set(format!("copy {} chars", t.len())),
            }
            div { class: "gl-out", "data-out": "fill-in-prompt-templates", "{last()}" }
        }
    }
}

#[component]
fn RunControlComposerDemo() -> Element {
    use rv2_kit::components::run_control_composer::{
        AgentIdentity, QueuedMessage, RunControlComposer, RunState, SteerNote, WorkMode,
    };
    let mut last = use_signal(String::new);
    let mut run_state = use_signal(|| RunState::Running);
    let mut work_mode = use_signal(|| WorkMode::Act);
    rsx! {
        Item { slug: "run-control-composer", name: "RunControlComposer",
            RunControlComposer {
                agent_identity: AgentIdentity { agent_id: "itinerary".into(), agent_name: "Itinerary agent".into() },
                request_text: Some("Plan a three-day trip to Lisbon with a focus on food.".to_string()),
                answer_text: Some("Day one: settle into Alfama, then lunch at a tasca by the river. Day two: the tram to Belém, a morning at the monastery and custard tarts by the water.".to_string()),
                run_state: run_state(),
                steer_notes: vec![SteerNote { at_char: 12, note_text: "Prefer primary sources".into() }],
                queued_messages: vec![QueuedMessage { queue_id: "q1".into(), message_text: "Then add a ferry ride on day three".into() }],
                work_mode: Some(work_mode()),
                on_work_mode_change: move |m: WorkMode| {
                    last.set(format!("mode {m:?}"));
                    work_mode.set(m);
                },
                context_share: Some(0.42),
                elapsed_secs: Some(75),
                input_hint: "Steer the run",
                on_submit: move |t: String| last.set(format!("submit {t}")),
                on_steer: move |t: String| last.set(format!("steer {t}")),
                on_queue: move |t: String| last.set(format!("queue {t}")),
                on_dequeue: move |id: String| last.set(format!("dequeue {id}")),
                on_stop: move |_| {
                    last.set("stop".into());
                    run_state.set(RunState::Stopped);
                },
                on_resume: move |_| {
                    last.set("resume".into());
                    run_state.set(RunState::Running);
                },
                on_retry: move |_| last.set("retry".into()),
                on_dismiss: move |_| last.set("dismiss".into()),
            }
            div { class: "gl-out", "data-out": "run-control-composer", "{last()}" }
        }
    }
}

#[component]
fn FileDropAreaDemo() -> Element {
    use rv2_kit::components::file_drop_area::{FileDropArea, FileItem, ItemState};
    let mut last = use_signal(String::new);
    let rows = vec![
        FileItem {
            item_id: "fda-close-pack".into(),
            file_name: "close-pack.csv".into(),
            byte_size: 3_984_589,
            item_state: ItemState::Uploading,
            progress: 0.4,
            error_text: None,
            seconds_left: Some(6),
        },
        FileItem {
            item_id: "fda-brief".into(),
            file_name: "brief.pdf".into(),
            byte_size: 239_616,
            item_state: ItemState::Done,
            progress: 1.0,
            error_text: None,
            seconds_left: None,
        },
        FileItem {
            item_id: "fda-setup".into(),
            file_name: "setup.exe".into(),
            byte_size: 12_582_912,
            item_state: ItemState::Rejected,
            progress: 0.0,
            error_text: Some("Type not permitted".into()),
            seconds_left: None,
        },
    ];
    rsx! {
        Item { slug: "file-drop-area", name: "FileDropArea",
            FileDropArea {
                accepted_extensions: vec![".pdf".to_string(), ".csv".to_string()],
                title_text: Some("Add files to this list".to_string()),
                hint_text: Some("Accepts PDF and CSV".to_string()),
                initial_items: rows,
                on_list_change: move |items: Vec<FileItem>| {
                    let done = items.iter().filter(|i| i.item_state == ItemState::Done).count();
                    last.set(format!("{} files, {done} done", items.len()));
                },
            }
            div { class: "gl-out", "data-out": "file-drop-area", "{last()}" }
        }
    }
}

#[component]
fn ConnectorPickerDemo() -> Element {
    use rv2_kit::components::connector_picker::{Connector, ConnectorLabels, ConnectorPicker};
    let mut last = use_signal(String::new);
    let conn = |id: &str, name: &str, category: &str, connected: bool| Connector {
        connector_id: id.into(),
        connector_name: name.into(),
        connector_domain: format!("{id}.example.com"),
        connector_category: category.into(),
        connector_summary: format!("Keep {name} records in step with the workspace."),
        connector_features: vec![
            "Two-way sync".into(),
            "Scheduled refresh".into(),
            "Field mapping".into(),
        ],
        connector_mark: None,
        is_connected: connected,
    };
    let mut connectors = use_signal(|| {
        vec![
            conn("files", "Files", "Storage", true),
            conn("calendar", "Calendar", "Planning", true),
            conn("mail", "Mail", "Messaging", true),
            conn("warehouse", "Warehouse", "Data", true),
            conn("drive", "Drive", "Storage", false),
            conn("planner", "Planner", "Planning", false),
            conn("chat", "Chat", "Messaging", false),
            conn("sheets", "Sheets", "Data", false),
            conn("notes", "Notes", "Storage", false),
            conn("board", "Board", "Planning", false),
        ]
    });
    rsx! {
        Item { slug: "connector-picker", name: "ConnectorPicker",
            ConnectorPicker {
                connector_list: Some(connectors()),
                product_name: "Harbor",
                label_text: ConnectorLabels {
                    explore: Some("Open settings".into()),
                    ..ConnectorLabels::default()
                },
                on_connection_change: move |(id, on): (String, bool)| {
                    last.set(format!("connection {id} {on}"));
                    connectors.with_mut(|list| {
                        for connector in list.iter_mut().filter(|c| c.connector_id == id) {
                            connector.is_connected = on;
                        }
                    });
                },
                on_explore: move |id: String| last.set(format!("explore {id}")),
            }
            div { class: "gl-out", "data-out": "connector-picker", "{last()}" }
        }
    }
}

#[component]
fn ConnectorSettingsCardDemo() -> Element {
    use rv2_kit::components::connector_settings_card::{ConnectorSettingsCard, ConnectorValues};
    let mut last = use_signal(String::new);
    rsx! {
        Item { slug: "connector-settings-card", name: "ConnectorSettingsCard",
            ConnectorSettingsCard {
                service_name: "Analytics",
                item_count: Some(3),
                subtitle_text: "Sends events to the analytics workspace",
                site_url_hint: "https://your-site.example",
                time_zone_options: vec!["UTC".to_string(), "Europe/Berlin".to_string()],
                initial_values: ConnectorValues { is_enabled: true, site_url: "https://app.example".into(), time_zone: "UTC".into(), api_key: String::new() },
                on_save: move |v: ConnectorValues| last.set(format!("save {} enabled={}", v.site_url, v.is_enabled)),
                on_enabled_change: move |on: bool| last.set(format!("enabled {on}")),
            }
            div { class: "gl-out", "data-out": "connector-settings-card", "{last()}" }
        }
    }
}

#[component]
fn OnboardingChecklistPanelDemo() -> Element {
    use rv2_kit::components::onboarding_checklist_panel::{
        CheckGroup, CheckItem, OnboardingChecklistPanel,
    };
    let mut last = use_signal(String::new);
    let item = |id: &str, text: &str, done: bool, locked: bool| CheckItem {
        item_id: id.into(),
        item_text: text.into(),
        is_done: done,
        is_locked: locked,
    };
    let groups = vec![
        CheckGroup {
            group_id: "setup".into(),
            group_title: "Set up".into(),
            task_list: vec![
                item("a", "Connect a model", true, false),
                item("b", "Invite a teammate", true, false),
                item("c", "Import the first document set", false, false),
            ],
        },
        CheckGroup {
            group_id: "safety".into(),
            group_title: "Safety".into(),
            task_list: vec![
                item("d", "Review permissions", false, true),
                item("e", "Set a spend limit", false, false),
            ],
        },
        CheckGroup {
            group_id: "team".into(),
            group_title: "Team".into(),
            task_list: vec![item("f", "Name the workspace owner", true, false)],
        },
    ];
    rsx! {
        Item { slug: "onboarding-checklist-panel", name: "OnboardingChecklistPanel",
            OnboardingChecklistPanel {
                group_list: groups,
                on_change: move |ids: Vec<String>| last.set(format!("checked {}", ids.join(","))),
                on_edit: move |_| last.set("edit".into()),
                on_dismiss: move |_| last.set("dismiss".into()),
            }
            div { class: "gl-out", "data-out": "onboarding-checklist-panel", "{last()}" }
        }
    }
}

#[component]
fn ChargePhaseBuilderDemo() -> Element {
    use rv2_kit::components::charge_phase_builder::{
        CatalogItem, ChargePhaseBuilder, Line, LineKind, ProductLine, Recurrence, Schedule, Timing,
    };
    let mut last = use_signal(String::new);
    let kickoff = Line {
        line_id: "k1".into(),
        kind: LineKind::Product(ProductLine {
            name: "Kickoff workshop".into(),
            recurrence: Recurrence::Once,
            timing: Timing::UpFront,
            amount: "300".into(),
            quantity: "1".into(),
        }),
    };
    let catalog = vec![CatalogItem {
        option_name: "Platform seat".into(),
        recurrence: Recurrence::Monthly,
        unit_amount: 120.0,
    }];
    rsx! {
        Item { slug: "charge-phase-builder", name: "ChargePhaseBuilder",
            ChargePhaseBuilder {
                heading_text: "Contract phase",
                phase_label: "Rollout phase",
                item_options: catalog,
                starting_lines: vec![kickoff],
                starting_date: "2026-03-01",
                on_schedule_change: move |s: Schedule| last.set(format!("schedule {} lines {}", s.duration, s.lines.len())),
            }
            div { class: "gl-out", "data-out": "charge-phase-builder", "{last()}" }
        }
    }
}
