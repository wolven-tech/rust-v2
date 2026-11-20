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

#[component]
fn ModelSelectorDemo() -> Element {
    use rv2_kit::components::model_selector::{ModelOption, ModelSelector, ThinkingEffort};
    let mut last = use_signal(String::new);
    let mut model = use_signal(|| Some("fast".to_string()));
    let mut thinking = use_signal(|| true);
    let mut web_search = use_signal(|| false);
    let mut effort = use_signal(|| ThinkingEffort::Medium);
    let opt = |id: &str, name: &str, speed: u8, quality: u8, price: f64| ModelOption {
        option_id: id.into(),
        option_name: name.into(),
        option_note: Some("Balanced".into()),
        option_icon: None,
        speed_rank: Some(speed),
        quality_rank: Some(quality),
        cost_rank: Some(3),
        first_token_time: Some(0.6),
        price_per_million: Some(price),
    };
    rsx! {
        Item { slug: "model-selector", name: "ModelSelector",
            ModelSelector {
                model_options: vec![opt("fast", "Model Fast", 5, 2, 0.5), opt("deep", "Model Deep", 2, 5, 15.0)],
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
        }
    }
}

#[component]
fn MultiSelectLookupDemo() -> Element {
    use rv2_kit::components::multi_select_lookup::{LookupOption, MultiSelectLookup, OptionImage};
    let mut last = use_signal(String::new);
    let mut picked = use_signal(|| vec!["ada".to_string()]);
    let opt = |v: &str, label: &str, group: &str| LookupOption {
        option_value: v.into(),
        option_label: label.into(),
        option_hint: Some("Member".into()),
        option_image: Some(OptionImage::Initials),
        option_swatch: None,
        option_mark: None,
        option_group: Some(group.into()),
        match_terms: vec![label.to_lowercase()],
    };
    let options = vec![
        opt("ada", "Ada Lovelace", "Engineering"),
        opt("ben", "Ben Carter", "Engineering"),
        opt("cy", "Cy Mora", "Design"),
    ];
    rsx! {
        Item { slug: "multi-select-lookup", name: "MultiSelectLookup",
            MultiSelectLookup {
                option_list: options,
                selected_values: Some(picked()),
                field_caption: Some("Reviewers".to_string()),
                empty_hint: "Add reviewers",
                on_selection_change: move |(vals, _): (Vec<String>, Vec<LookupOption>)| {
                    last.set(format!("selected {}", vals.join(",")));
                    picked.set(vals);
                },
            }
            div { class: "gl-out", "data-out": "multi-select-lookup", "{last()}" }
        }
    }
}

#[component]
fn MultiStepWizardFormDemo() -> Element {
    use rv2_affect::text_field::TextField;
    use rv2_kit::components::multi_step_wizard_form::{
        FieldMessage, FormValues, MultiStepWizardForm, StepContext, Task, WizardStep,
    };
    let mut last = use_signal(String::new);
    let steps = vec![
        WizardStep {
            step_id: "who".into(),
            step_title: "Who".into(),
            step_heading: Some("Who is this for?".into()),
            step_description: Some("Name the workspace owner.".into()),
            render_step: Callback::new(move |ctx: StepContext| {
                let mut values = ctx.set_form_values;
                let current = ctx.form_values.get("name").cloned().unwrap_or_default();
                rsx! {
                    TextField {
                        id: "wizard-name",
                        name: "name",
                        label: rsx! { "Name" },
                        value: current,
                        on_input: move |e: FormEvent| {
                            values.write().insert("name".to_string(), e.value());
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
            step_id: "what".into(),
            step_title: "What".into(),
            step_heading: Some("What will it do?".into()),
            step_description: None,
            render_step: Callback::new(
                |_ctx: StepContext| rsx! { p { "Pick a plan on the next step." } },
            ),
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
    use rv2_kit::components::period_picker::{ComparisonMode, DateRange, PeriodPicker};
    let mut last = use_signal(String::new);
    let day = |d: u8| Day {
        year: 2026,
        month: 11,
        day: d,
    };
    rsx! {
        Item { slug: "period-picker", name: "PeriodPicker",
            PeriodPicker {
                starting_range: Some(DateRange { from: day(3), to: day(14) }),
                anchor_today: Some(day(21)),
                starting_comparison: ComparisonMode::PreviousPeriod,
                on_apply: move |(r, cmp, mode): (DateRange, Option<DateRange>, ComparisonMode)| {
                    last.set(format!("apply {}-{} vs {} {mode:?}", r.from.day, r.to.day, cmp.is_some()));
                },
                start_expanded: false,
            }
            div { class: "gl-out", "data-out": "period-picker", "{last()}" }
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
        capability_mark: None,
    };
    rsx! {
        Item { slug: "trust-level-selector", name: "TrustLevelSelector",
            TrustLevelSelector {
                capabilities: vec![cap("read", "Read files", RiskClass::Routine), cap("send", "Send email", RiskClass::Sensitive), cap("delete", "Delete data", RiskClass::Irreversible)],
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
    let blanks = BTreeMap::from([(
        "client".to_string(),
        BlankSpec {
            hint_text: Some("Client name".into()),
            fallback_value: None,
        },
    )]);
    let templates = vec![
        Template {
            template_id: "t1".into(),
            template_title: "Client summary".into(),
            group_label: Some("Sales".into()),
            group_key: Some("sales".into()),
            template_body: "Summarise the last call with {client} in three bullets.".into(),
            blank_specs: blanks,
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
                agent_identity: AgentIdentity { agent_id: "a".into(), agent_name: "Research agent".into() },
                run_state: run_state(),
                steer_notes: vec![SteerNote { at_char: 12, note_text: "Prefer primary sources".into() }],
                queued_messages: vec![QueuedMessage { queue_id: "q1".into(), message_text: "Then check the 2024 numbers".into() }],
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
            }
            div { class: "gl-out", "data-out": "run-control-composer", "{last()}" }
        }
    }
}

#[component]
fn FileDropAreaDemo() -> Element {
    use rv2_kit::components::file_drop_area::{FileDropArea, FileItem, ItemState};
    let mut last = use_signal(String::new);
    rsx! {
        Item { slug: "file-drop-area", name: "FileDropArea",
            FileDropArea {
                accepted_extensions: vec![".pdf".to_string(), ".csv".to_string()],
                hint_text: Some("Drop PDF or CSV files here".to_string()),
                demo_run: true,
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
    use rv2_kit::components::connector_picker::{ActionMode, Connector, ConnectorPicker};
    let mut last = use_signal(String::new);
    let conn = |id: &str, name: &str, connected: bool| Connector {
        connector_id: id.into(),
        connector_name: name.into(),
        connector_domain: format!("{id}.example.com"),
        connector_category: "Storage".into(),
        connector_summary: format!("Sync {name} documents"),
        connector_features: vec!["Two-way sync".into()],
        connector_mark: None,
        is_connected: connected,
    };
    let mut connectors = use_signal(|| {
        vec![
            conn("drive", "Drive", true),
            conn("notion", "Notion", false),
        ]
    });
    rsx! {
        Item { slug: "connector-picker", name: "ConnectorPicker",
            ConnectorPicker {
                connector_list: Some(connectors()),
                product_name: "Workspace",
                action_mode: ActionMode::Connect,
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
                item("b", "Invite a teammate", false, false),
            ],
        },
        CheckGroup {
            group_id: "safety".into(),
            group_title: "Safety".into(),
            task_list: vec![item("c", "Review permissions", false, true)],
        },
    ];
    rsx! {
        Item { slug: "onboarding-checklist-panel", name: "OnboardingChecklistPanel",
            OnboardingChecklistPanel {
                group_list: groups,
                on_change: move |ids: Vec<String>| last.set(format!("checked {}", ids.join(","))),
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
    let lines = vec![Line {
        line_id: "l1".into(),
        kind: LineKind::Product(ProductLine {
            name: "Seat".into(),
            recurrence: Recurrence::Monthly,
            timing: Timing::UpFront,
            amount: "120".into(),
            quantity: "2".into(),
        }),
    }];
    rsx! {
        Item { slug: "charge-phase-builder", name: "ChargePhaseBuilder",
            ChargePhaseBuilder {
                item_options: vec![CatalogItem { option_name: "Seat".into(), recurrence: Recurrence::Monthly, unit_amount: 120.0 }],
                starting_lines: lines,
                starting_date: "2026-12-01",
                on_schedule_change: move |s: Schedule| last.set(format!("schedule {} lines {}", s.duration, s.lines.len())),
            }
            div { class: "gl-out", "data-out": "charge-phase-builder", "{last()}" }
        }
    }
}
