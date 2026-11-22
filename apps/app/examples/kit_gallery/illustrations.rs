use dioxus::prelude::*;

use super::{Item, Section, gallery_now};

#[component]
pub fn Illustrations() -> Element {
    rsx! {
        Section { id: "illustrations", title: "Illustrations",
            FolderDemo {}
            PillDemo {}
            DictationDemo {}
            OrbDemo {}
            FanoutDemo {}
            TaskBranchDemo {}
            ModelPickerDemo {}
            MapDemo {}
            ArcDemo {}
            PromptDemo {}
            CardDemo {}
            ComposerDemo {}
        }
    }
}

/// Our own glyph, used wherever a demo needs an icon or a mark.
fn glyph() -> Element {
    rsx! {
        svg { class: "gl-glyph", "viewBox": "0 0 16 16", width: "16", height: "16", "aria-hidden": "true",
            circle { "cx": "8", "cy": "8", "r": "5", fill: "currentColor" }
        }
    }
}

#[component]
fn FolderDemo() -> Element {
    use rv2_kit::illustrations::frosted_folder_mark::{FrostedFolderMark, SizeStep};
    rsx! {
        Item { slug: "frosted-folder-mark", name: "FrostedFolderMark",
            div { class: "gl-row",
                FrostedFolderMark { size_token: SizeStep::Sm }
                FrostedFolderMark { size_token: SizeStep::Md }
                FrostedFolderMark { size_token: SizeStep::Lg, label: "Client documents" }
                FrostedFolderMark { size_token: SizeStep::Md, disabled: true }
            }
            div { class: "kit-folder-scope gl-canvas gl-row", tabindex: "0", "data-scope": "folder",
                FrostedFolderMark { size_token: SizeStep::Md }
                span { "Focus this card to open the folder" }
            }
        }
    }
}

#[component]
fn PillDemo() -> Element {
    use rv2_kit::illustrations::progress_pill_scene::{PillState, ProgressPillScene};
    let mut progress = use_signal(|| 0.42f64);
    rsx! {
        Item { slug: "progress-pill-scene", name: "ProgressPillScene",
            div { class: "gl-row",
                button { r#type: "button", "data-action": "advance", onclick: move |_| progress.set((progress() + 0.1).min(1.0)), "Advance 10%" }
                button { r#type: "button", "data-action": "reset", onclick: move |_| progress.set(0.0), "Reset" }
            }
            div { class: "gl-stack",
                ProgressPillScene {
                    title: "Q3 pricing memo",
                    agent_label: "Research agent",
                    source_line: "12 sources",
                    progress: progress(),
                    step_label: "Comparing competitor tiers",
                }
                ProgressPillScene {
                    title: "Pipeline audit trail export for the Northwind account",
                    agent_label: "Export agent",
                    source_line: "3 sources",
                    progress: 0.6,
                    step_label: "Waiting for approval",
                    state: PillState::Paused,
                }
                ProgressPillScene {
                    title: "Launch checklist",
                    agent_label: "Ops agent",
                    source_line: "3 sources",
                    progress: 1.0,
                    step_label: "Delivered",
                    state: PillState::Done,
                    interactive: true,
                    on_open: |_| {},
                }
            }
        }
    }
}

#[component]
fn DictationDemo() -> Element {
    use rv2_kit::illustrations::dictation_capture_panel::{DictationCapturePanel, DictationState};
    let mut state = use_signal(|| DictationState::Listening);
    let mut listened_ms = use_signal(|| 0u64);
    let mut last = use_signal(gallery_now);
    let mut out = use_signal(String::new);
    use_effect(move || {
        let now = gallery_now();
        let previous = *last.peek();
        if *state.peek() == DictationState::Listening && now > previous {
            let total = *listened_ms.peek() + (now - previous);
            listened_ms.set(total);
        }
        last.set(now);
    });
    let ticks = listened_ms() as f64 / 250.0;
    let samples: Vec<f64> = (0..40)
        .map(|i| (0.5 + 0.4 * ((ticks - (39 - i) as f64) / 2.5).sin()).clamp(0.0, 1.0))
        .collect();
    let elapsed = listened_ms() / 1000;
    rsx! {
        Item { slug: "dictation-capture-panel", name: "DictationCapturePanel",
            div { class: "gl-row",
                button { r#type: "button", "data-action": "listen", onclick: move |_| state.set(DictationState::Listening), "Listening" }
                button { r#type: "button", "data-action": "pause", onclick: move |_| state.set(DictationState::Paused), "Pause" }
                button { r#type: "button", "data-action": "idle", onclick: move |_| {
                    state.set(DictationState::Idle);
                    listened_ms.set(0);
                }, "Idle" }
            }
            div { class: "gl-stack",
                DictationCapturePanel {
                    state: state(),
                    level_samples: samples,
                    elapsed_secs: elapsed,
                    on_finish: move |_| out.set("finished".to_string()),
                    on_cancel: move |_| out.set("cancelled".to_string()),
                }
            }
            div { class: "gl-out", "data-out": "dictation-capture-panel", {out()} }
        }
    }
}

#[component]
fn OrbDemo() -> Element {
    use rv2_kit::illustrations::voice_assistant_orb::{OrbState, VoiceAssistantOrb};
    let mut state = use_signal(|| OrbState::Idle);
    let mut level = use_signal(|| 0.2f64);
    let mut presses = use_signal(|| 0u32);
    let count = presses();
    rsx! {
        Item { slug: "voice-assistant-orb", name: "VoiceAssistantOrb",
            div { class: "gl-row",
                button { r#type: "button", "data-action": "idle", onclick: move |_| state.set(OrbState::Idle), "Idle" }
                button { r#type: "button", "data-action": "listening", onclick: move |_| state.set(OrbState::Listening), "Listening" }
                button { r#type: "button", "data-action": "thinking", onclick: move |_| state.set(OrbState::Thinking), "Thinking" }
                button { r#type: "button", "data-action": "speaking", onclick: move |_| state.set(OrbState::Speaking), "Speaking" }
                button { r#type: "button", "data-action": "loud", onclick: move |_| level.set(0.9), "Loud" }
                button { r#type: "button", "data-action": "quiet", onclick: move |_| level.set(0.15), "Quiet" }
            }
            div { class: "gl-row",
                VoiceAssistantOrb { state: state(), level: level(), label: "Assistant" }
                VoiceAssistantOrb {
                    state: state(),
                    level: level(),
                    interactive: true,
                    on_press: move |_| {
                        let next = if state() == OrbState::Listening { OrbState::Idle } else { OrbState::Listening };
                        state.set(next);
                        presses += 1;
                    },
                }
            }
            div { class: "gl-out", "data-out": "voice-assistant-orb", "presses {count}" }
        }
    }
}

fn endpoint(id: &str, label: &str) -> rv2_kit::illustrations::outreach_fanout_lines::Endpoint {
    rv2_kit::illustrations::outreach_fanout_lines::Endpoint {
        id: id.to_string(),
        label: label.to_string(),
        icon: Some(glyph()),
    }
}

#[component]
fn FanoutDemo() -> Element {
    use rv2_kit::illustrations::outreach_fanout_lines::OutreachFanoutLines;
    let mut active = use_signal(|| None::<String>);
    let mut picked = use_signal(String::new);
    let destinations = vec![
        endpoint("mail", "Mail"),
        endpoint("chat", "Chat"),
        endpoint("crm", "CRM"),
        endpoint("sms", "Text"),
    ];
    rsx! {
        Item { slug: "outreach-fanout-lines", name: "OutreachFanoutLines",
            div { class: "gl-stack",
                OutreachFanoutLines {
                    source: endpoint("outbox", "Outbox"),
                    destinations: destinations.clone(),
                    active_destination_id: active(),
                }
                OutreachFanoutLines {
                    source: endpoint("outbox", "Outbox"),
                    destinations: destinations.clone(),
                    active_destination_id: active(),
                    on_select: move |id: String| {
                        picked.set(id.clone());
                        active.set(Some(id));
                    },
                }
            }
            div { class: "gl-out", "data-out": "outreach-fanout-lines", {picked()} }
        }
    }
}

#[component]
fn TaskBranchDemo() -> Element {
    use rv2_kit::illustrations::task_branch_scene::{
        Assignee, ClientNode, Note, Person, Role, TaskBranchScene, TaskCard,
    };
    rsx! {
        Item { slug: "task-branch-scene", name: "TaskBranchScene",
            div { class: "gl-stack",
                TaskBranchScene {
                    client: ClientNode {
                        label: "Northwind Foods".to_string(),
                        connection_status: "Connected".to_string(),
                    },
                    task: TaskCard {
                        title: "Q4 onboarding deck".to_string(),
                        assignees: vec![
                            Assignee { id: "ada".to_string(), avatar: "AL".to_string() },
                            Assignee { id: "sam".to_string(), avatar: "SR".to_string() },
                        ],
                        tool_icons: vec![glyph(), glyph()],
                    },
                    notes: vec![
                        Note { id: "kickoff".to_string(), label: "Kickoff call".to_string() },
                        Note { id: "brief".to_string(), label: "Brief v2".to_string() },
                    ],
                    roles: vec![Role {
                        id: "design".to_string(),
                        title: "Design lead".to_string(),
                        person: Person { name: "Ada Lovelace".to_string(), avatar: "AL".to_string() },
                        tool_icons: vec![glyph()],
                    }],
                }
            }
        }
    }
}

#[component]
fn ModelPickerDemo() -> Element {
    use rv2_kit::illustrations::model_picker_scene::{ModelGroup, ModelOption, ModelPickerScene};
    let option = |id: &str, name: &str, available: bool| ModelOption {
        id: id.to_string(),
        name: name.to_string(),
        vendor_mark: Some(glyph()),
        available,
    };
    let groups = vec![
        ModelGroup {
            label: "Recommended".to_string(),
            models: vec![
                option("fast", "Flash 2", true),
                option("deep", "Deep reason", true),
            ],
        },
        ModelGroup {
            label: "More".to_string(),
            models: vec![
                option("large", "Large context", true),
                option("legacy", "Legacy v1", false),
                option("mini", "Mini", true),
                option("edge", "Edge", true),
            ],
        },
    ];
    let mut chosen = use_signal(|| "fast".to_string());
    rsx! {
        Item { slug: "model-picker-scene", name: "ModelPickerScene",
            div { class: "gl-row",
                ModelPickerScene {
                    groups: groups.clone(),
                    default_id: "deep",
                    on_select: move |id: String| chosen.set(id),
                }
                ModelPickerScene {
                    groups: groups.clone(),
                    popover: true,
                    on_select: move |id: String| chosen.set(id),
                }
            }
            div { class: "gl-out", "data-out": "model-picker-scene", {chosen()} }
        }
    }
}

#[component]
fn MapDemo() -> Element {
    use rv2_kit::illustrations::continent_highlight_map::{ContinentHighlightMap, MapRegion};
    let mut chosen = use_signal(|| "europe".to_string());
    let regions = vec![
        MapRegion {
            id: "north-america".into(),
            label: "North America".into(),
            geometry_ref: "north-america".into(),
        },
        MapRegion {
            id: "europe".into(),
            label: "Europe".into(),
            geometry_ref: "europe".into(),
        },
        MapRegion {
            id: "africa".into(),
            label: "Africa".into(),
            geometry_ref: "africa".into(),
        },
        MapRegion {
            id: "asia".into(),
            label: "Asia".into(),
            geometry_ref: "asia".into(),
        },
        MapRegion {
            id: "americas".into(),
            label: "Americas".into(),
            geometry_ref: "americas".into(),
        },
        MapRegion {
            id: "europe-africa".into(),
            label: "Europe and Africa".into(),
            geometry_ref: "europe-africa".into(),
        },
    ];
    let land = sample_land();
    rsx! {
        Item { slug: "continent-highlight-map", name: "ContinentHighlightMap",
            ContinentHighlightMap {
                regions: regions.clone(),
                land: land.clone(),
                selected_region_id: chosen(),
                on_region_change: move |id: String| chosen.set(id),
                show_legend: true,
            }
            div { class: "gl-out", "data-out": "continent-highlight-map", {chosen()} }
        }
    }
}

/// Simplified original outlines, as (longitude, latitude) in degrees; drawn to the map's 1000 x 500 space.
fn sample_land() -> Vec<rv2_kit::illustrations::continent_highlight_map::LandPiece> {
    use rv2_kit::illustrations::continent_highlight_map::LandPiece;
    let geo = |points: &[(f64, f64)]| -> Vec<(f64, f64)> {
        points
            .iter()
            .map(|(lon, lat)| ((lon + 180.0) / 360.0 * 1000.0, (90.0 - lat) / 180.0 * 500.0))
            .collect()
    };
    let north_america = geo(&[
        (-168.0, 66.0),
        (-141.0, 70.0),
        (-120.0, 72.0),
        (-95.0, 73.0),
        (-82.0, 69.0),
        (-72.0, 62.0),
        (-62.0, 58.0),
        (-56.0, 52.0),
        (-65.0, 44.0),
        (-70.0, 42.0),
        (-76.0, 35.0),
        (-81.0, 31.0),
        (-80.0, 25.0),
        (-83.0, 29.0),
        (-89.0, 30.0),
        (-97.0, 27.0),
        (-97.0, 19.0),
        (-92.0, 18.0),
        (-88.0, 21.0),
        (-84.0, 15.0),
        (-80.0, 9.0),
        (-86.0, 13.0),
        (-95.0, 16.0),
        (-105.0, 20.0),
        (-110.0, 24.0),
        (-114.0, 31.0),
        (-118.0, 34.0),
        (-124.0, 40.0),
        (-124.0, 48.0),
        (-130.0, 55.0),
        (-141.0, 60.0),
        (-152.0, 59.0),
        (-165.0, 61.0),
    ]);
    let greenland = geo(&[
        (-73.0, 78.0),
        (-60.0, 82.0),
        (-30.0, 83.0),
        (-20.0, 75.0),
        (-22.0, 70.0),
        (-40.0, 64.0),
        (-50.0, 62.0),
        (-55.0, 70.0),
        (-68.0, 76.0),
    ]);
    let south_america = geo(&[
        (-80.0, 9.0),
        (-72.0, 12.0),
        (-62.0, 11.0),
        (-50.0, 0.0),
        (-35.0, -5.0),
        (-39.0, -14.0),
        (-41.0, -22.0),
        (-48.0, -26.0),
        (-58.0, -35.0),
        (-65.0, -42.0),
        (-68.0, -52.0),
        (-72.0, -54.0),
        (-75.0, -47.0),
        (-73.0, -38.0),
        (-71.0, -27.0),
        (-70.0, -18.0),
        (-76.0, -14.0),
        (-81.0, -5.0),
    ]);
    let europe = geo(&[
        (-9.0, 43.0),
        (-9.0, 37.0),
        (-2.0, 36.0),
        (3.0, 42.0),
        (8.0, 44.0),
        (12.0, 44.0),
        (16.0, 38.0),
        (18.0, 40.0),
        (14.0, 46.0),
        (23.0, 36.0),
        (26.0, 41.0),
        (29.0, 45.0),
        (33.0, 46.0),
        (40.0, 47.0),
        (38.0, 50.0),
        (30.0, 56.0),
        (28.0, 60.0),
        (22.0, 60.0),
        (18.0, 59.0),
        (12.0, 56.0),
        (9.0, 58.0),
        (5.0, 60.0),
        (14.0, 67.0),
        (25.0, 71.0),
        (40.0, 67.0),
        (40.0, 62.0),
        (30.0, 60.0),
    ]);
    let africa = geo(&[
        (-17.0, 21.0),
        (-13.0, 28.0),
        (-6.0, 36.0),
        (10.0, 37.0),
        (20.0, 32.0),
        (32.0, 31.0),
        (35.0, 28.0),
        (43.0, 12.0),
        (51.0, 12.0),
        (40.0, -3.0),
        (40.0, -15.0),
        (35.0, -24.0),
        (27.0, -34.0),
        (18.0, -35.0),
        (12.0, -18.0),
        (13.0, -5.0),
        (9.0, 4.0),
        (-8.0, 4.0),
        (-17.0, 14.0),
    ]);
    let asia = geo(&[
        (26.0, 41.0),
        (29.0, 45.0),
        (40.0, 47.0),
        (60.0, 70.0),
        (100.0, 78.0),
        (140.0, 72.0),
        (180.0, 68.0),
        (170.0, 60.0),
        (155.0, 56.0),
        (140.0, 48.0),
        (130.0, 42.0),
        (122.0, 40.0),
        (122.0, 30.0),
        (108.0, 21.0),
        (100.0, 13.0),
        (103.0, 2.0),
        (98.0, 8.0),
        (92.0, 22.0),
        (80.0, 8.0),
        (72.0, 20.0),
        (58.0, 25.0),
        (52.0, 26.0),
        (45.0, 40.0),
        (36.0, 36.0),
        (30.0, 36.0),
        (26.0, 39.0),
    ]);
    let oceania = geo(&[
        (114.0, -22.0),
        (122.0, -18.0),
        (131.0, -12.0),
        (137.0, -12.0),
        (142.0, -11.0),
        (146.0, -19.0),
        (153.0, -26.0),
        (150.0, -37.0),
        (141.0, -38.0),
        (131.0, -31.0),
        (115.0, -34.0),
    ]);
    vec![
        LandPiece {
            geometry_ref: "americas".into(),
            polygons: vec![
                north_america.clone(),
                greenland.clone(),
                south_america.clone(),
            ],
        },
        LandPiece {
            geometry_ref: "europe-africa".into(),
            polygons: vec![europe.clone(), africa.clone()],
        },
        LandPiece {
            geometry_ref: "south-america".into(),
            polygons: vec![south_america],
        },
        LandPiece {
            geometry_ref: "north-america".into(),
            polygons: vec![north_america, greenland],
        },
        LandPiece {
            geometry_ref: "europe".into(),
            polygons: vec![europe],
        },
        LandPiece {
            geometry_ref: "africa".into(),
            polygons: vec![africa],
        },
        LandPiece {
            geometry_ref: "asia".into(),
            polygons: vec![asia],
        },
        LandPiece {
            geometry_ref: "oceania".into(),
            polygons: vec![oceania],
        },
    ]
}

#[component]
fn ArcDemo() -> Element {
    use rv2_kit::illustrations::integration_arc_flow::{
        Hub, Integration, IntegrationArcFlow, IntegrationStatus,
    };
    let mut paused = use_signal(Vec::<String>::new);
    let status = |id: &str| {
        if paused().iter().any(|p| p == id) {
            IntegrationStatus::Paused
        } else {
            IntegrationStatus::Live
        }
    };
    let item = |id: &str, label: &str, sub: &str| Integration {
        id: id.to_string(),
        label: label.to_string(),
        sub_label: sub.to_string(),
        status: status(id),
        logo: Some(glyph()),
    };
    let integrations = vec![
        item("crm", "CRM", "Synced 2 min ago"),
        item("billing", "Billing", "Synced 1 h ago"),
        item("support", "Support", "Synced 5 min ago"),
        item("warehouse", "Warehouse", "Synced 9 min ago"),
    ];
    let hub = || Hub {
        title: "Revenue analysis".to_string(),
        subtitle: "One view across tools".to_string(),
    };
    rsx! {
        Item { slug: "integration-arc-flow", name: "IntegrationArcFlow",
            IntegrationArcFlow {
                hub: hub(),
                integrations: integrations.clone(),
                on_toggle: move |id: String| {
                    paused.with_mut(|list| {
                        if let Some(at) = list.iter().position(|p| *p == id) {
                            list.remove(at);
                        } else {
                            list.push(id);
                        }
                    });
                },
            }
            IntegrationArcFlow { hub: hub(), integrations: integrations.clone() }
        }
    }
}

#[component]
fn PromptDemo() -> Element {
    use rv2_kit::illustrations::keyboard_prompt_input::KeyboardPromptInput;
    let mut out = use_signal(String::new);
    rsx! {
        Item { slug: "keyboard-prompt-input", name: "KeyboardPromptInput",
            div { class: "gl-stack",
                KeyboardPromptInput {
                    sample_prompts: vec![
                        "Summarise the Q3 churn drivers".to_string(),
                        "Draft a reply to the Northwind renewal".to_string(),
                    ],
                }
                KeyboardPromptInput {
                    value: "Compare the two pricing tiers",
                    on_submit: move |text: String| out.set(text),
                }
            }
            div { class: "gl-out", "data-out": "keyboard-prompt-input", {out()} }
        }
    }
}

fn sample_card() -> rv2_kit::illustrations::inspected_agent_card_scene::AgentCard {
    use rv2_kit::illustrations::inspected_agent_card_scene::{AgentCard, CardFooter, CardRow};
    AgentCard {
        title: "Invoice triage agent".to_string(),
        model_label: "Model: Sonnet".to_string(),
        status_label: "Running".to_string(),
        task_line: "Sorting 42 invoices".to_string(),
        rows: vec![
            CardRow {
                label: "Tokens".to_string(),
                value: "12.4k".to_string(),
                bar_fraction: Some(0.4),
            },
            CardRow {
                label: "Tools".to_string(),
                value: "3".to_string(),
                bar_fraction: None,
            },
            CardRow {
                label: "Latency".to_string(),
                value: "0.8 s".to_string(),
                bar_fraction: Some(0.2),
            },
            CardRow {
                label: "Cost".to_string(),
                value: "$0.08".to_string(),
                bar_fraction: None,
            },
        ],
        footer: CardFooter {
            cost_label: "$0.08 spent".to_string(),
            time_label: "1m 12s".to_string(),
        },
    }
}

#[component]
fn CardDemo() -> Element {
    use rv2_kit::illustrations::inspected_agent_card_scene::{
        InspectSpeed, InspectStep, InspectedAgentCardScene,
    };
    let steps = vec![
        InspectStep {
            row_index: 0,
            note: "Token use grows with the batch".to_string(),
        },
        InspectStep {
            row_index: 1,
            note: "Three tools are open".to_string(),
        },
        InspectStep {
            row_index: 2,
            note: "Latency stays under a second".to_string(),
        },
        InspectStep {
            row_index: 3,
            note: "Spend is shown last".to_string(),
        },
    ];
    rsx! {
        Item { slug: "inspected-agent-card-scene", name: "InspectedAgentCardScene",
            InspectedAgentCardScene { card: sample_card(), inspect_steps: steps.clone(), speed: InspectSpeed::Normal }
            InspectedAgentCardScene { card: sample_card(), inspect_steps: steps, interactive: true }
        }
    }
}

fn composer() -> rv2_kit::illustrations::inspected_composer_scene::Composer {
    rv2_kit::illustrations::inspected_composer_scene::Composer {
        placeholder_text: "Ask about your pipeline".to_string(),
        mode_label: "Chat".to_string(),
        tools: vec!["Search".to_string(), "Files".to_string()],
        send_label: "Send".to_string(),
    }
}

#[component]
fn ComposerDemo() -> Element {
    use rv2_kit::illustrations::inspected_composer_scene::InspectedComposerScene;
    rsx! {
        Item { slug: "inspected-composer-scene", name: "InspectedComposerScene",
            InspectedComposerScene {
                composer: composer(),
                connected_apps: vec![glyph(), glyph()],
            }
            InspectedComposerScene {
                composer: composer(),
                connected_apps: vec![glyph(), glyph()],
                interactive: true,
            }
        }
    }
}
