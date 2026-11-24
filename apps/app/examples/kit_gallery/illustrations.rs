use dioxus::prelude::*;

use super::{Item, Section};

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
            div { style: "width: min(100%, 560px); margin: 0 auto;",
                FrostedFolderMark { size_token: SizeStep::Md }
            }
            p { style: "margin: 8px 0 0; text-align: center;", "Hover the folder to open it" }
        }
    }
}

#[component]
fn PillDemo() -> Element {
    use rv2_kit::illustrations::progress_pill_scene::ProgressPillScene;
    let mut progress = use_signal(|| 0.33f64);
    rsx! {
        Item { slug: "progress-pill-scene", name: "ProgressPillScene",
            div { class: "gl-row",
                button { r#type: "button", "data-action": "advance", onclick: move |_| progress.set((progress() + 0.1).min(1.0)), "Advance 10%" }
                button { r#type: "button", "data-action": "reset", onclick: move |_| progress.set(0.0), "Reset" }
            }
            ProgressPillScene {
                title: "Q3 revenue report",
                agent_label: "Analyst",
                source_line: "24 sources read",
                progress: progress(),
            }
        }
    }
}

#[component]
fn DictationDemo() -> Element {
    use rv2_kit::illustrations::dictation_capture_panel::{DictationCapturePanel, DictationState};
    let mut state = use_signal(|| DictationState::Listening);
    let mut out = use_signal(String::new);
    // Fixed levels, all inside the standard glow band, so the light rests at mid strength.
    let samples: Vec<f64> = (0..40)
        .map(|i| 0.2 + ((i * 37) % 41) as f64 / 41.0 * 0.4)
        .collect();
    rsx! {
        Item { slug: "dictation-capture-panel", name: "DictationCapturePanel",
            div { class: "gl-row",
                button { r#type: "button", "data-action": "listen", onclick: move |_| state.set(DictationState::Listening), "Listening" }
                button { r#type: "button", "data-action": "pause", onclick: move |_| state.set(DictationState::Paused), "Pause" }
                button { r#type: "button", "data-action": "idle", onclick: move |_| state.set(DictationState::Idle), "Idle" }
            }
            div { class: "gl-stack",
                DictationCapturePanel {
                    state: state(),
                    level_samples: samples,
                    elapsed_secs: 42,
                    finish_label: "Done",
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
    use rv2_kit::illustrations::voice_assistant_orb::{OrbState, VoiceAssistantOrb, state_label};
    let mut state = use_signal(|| OrbState::Listening);
    rsx! {
        Item { slug: "voice-assistant-orb", name: "VoiceAssistantOrb",
            div { style: "width: min(100%, 560px); margin: 0 auto;",
                VoiceAssistantOrb {
                    state: state(),
                    level: 0.5,
                    on_demo_state: move |next: OrbState| state.set(next),
                }
            }
            div { class: "gl-out", "data-out": "voice-assistant-orb", {state_label(state())} }
        }
    }
}

/// Our own placeholder shapes for model rows, one per index: circle, diamond, triangle, square,
/// star, hexagon, ring. They are not vendor marks.
fn shape_mark(index: usize) -> Element {
    let (d, rule) = match index % 7 {
        0 => ("M8 3a5 5 0 1 0 0.01 0Z", "nonzero"),
        1 => ("M8 2.5 13.5 8 8 13.5 2.5 8Z", "nonzero"),
        2 => ("M8 2.5 13.5 13 2.5 13Z", "nonzero"),
        3 => ("M3 3h10v10H3Z", "nonzero"),
        4 => (
            "M8 2 9.8 6.1 14.2 6.5 10.9 9.4 11.9 13.8 8 11.5 4.1 13.8 5.1 9.4 1.8 6.5 6.2 6.1Z",
            "nonzero",
        ),
        5 => (
            "M8 2.2 13.3 5.1 13.3 10.9 8 13.8 2.7 10.9 2.7 5.1Z",
            "nonzero",
        ),
        _ => (
            "M8 3a5 5 0 1 0 0.01 0Z M8 5.5a2.5 2.5 0 1 1 0.01 0Z",
            "evenodd",
        ),
    };
    rsx! {
        svg { class: "gl-glyph", "viewBox": "0 0 16 16", width: "14", height: "14", "aria-hidden": "true",
            path { d, fill: "currentColor", "fill-rule": rule }
        }
    }
}

/// Generic kit icons standing in for the caller's destination marks.
fn icon_envelope() -> Element {
    rsx! {
        svg { class: "gl-glyph", "viewBox": "0 0 16 16", width: "18", height: "18", "aria-hidden": "true",
            fill: "none", stroke: "currentColor", "stroke-width": "1.3",
            rect { x: "2", y: "3.5", width: "12", height: "9", rx: "1.5" }
            path { d: "M2.5 4.5 8 9l5.5-4.5" }
        }
    }
}

fn icon_bubble() -> Element {
    rsx! {
        svg { class: "gl-glyph", "viewBox": "0 0 16 16", width: "18", height: "18", "aria-hidden": "true",
            fill: "none", stroke: "currentColor", "stroke-width": "1.3",
            path { d: "M3 3.5h10a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H7l-3 2.5V10.5H3a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1z" }
        }
    }
}

fn icon_document() -> Element {
    rsx! {
        svg { class: "gl-glyph", "viewBox": "0 0 16 16", width: "18", height: "18", "aria-hidden": "true",
            fill: "none", stroke: "currentColor", "stroke-width": "1.3",
            path { d: "M4 2.5h5l3 3v8H4z" }
            path { d: "M6.5 8.5h3M6.5 11h3" }
        }
    }
}

#[component]
fn FanoutDemo() -> Element {
    use rv2_kit::illustrations::outreach_fanout_lines::{Endpoint, OutreachFanoutLines};
    let mut active = use_signal(|| None::<String>);
    let mut picked = use_signal(String::new);
    let destinations = vec![
        Endpoint {
            id: "mail".into(),
            label: "Email".into(),
            icon: Some(icon_envelope()),
        },
        Endpoint {
            id: "chat".into(),
            label: "Chat".into(),
            icon: Some(icon_bubble()),
        },
        Endpoint {
            id: "docs".into(),
            label: "Documents".into(),
            icon: Some(icon_document()),
        },
    ];
    rsx! {
        Item { slug: "outreach-fanout-lines", name: "OutreachFanoutLines",
            OutreachFanoutLines {
                destinations: destinations.clone(),
                active_destination_id: active(),
                on_select: move |id: String| {
                    picked.set(id.clone());
                    active.set(Some(id));
                },
            }
            div { class: "gl-out", "data-out": "outreach-fanout-lines", {picked()} }
        }
    }
}

#[component]
fn TaskBranchDemo() -> Element {
    use rv2_kit::illustrations::task_branch_scene::{
        Assignee, ClientNode, NestedCard, Person, Role, SideNode, SubtaskCard, TaskBranchScene,
        TaskCard,
    };
    let owner = |id: &str, avatar: &str| Assignee {
        id: id.to_string(),
        avatar: avatar.to_string(),
    };
    let icons = |count: usize| (0..count).map(|_| glyph()).collect::<Vec<_>>();
    rsx! {
        Item { slug: "task-branch-scene", name: "TaskBranchScene",
            div { style: "width: min(100%, 880px); margin: 0 auto;",
                TaskBranchScene {
                    client: ClientNode {
                        label: "Client: Northwind Foods".to_string(),
                        connection_status: "Connected".to_string(),
                    },
                    task: TaskCard {
                        title: "Task: Q4 onboarding deck".to_string(),
                        assignees: vec![owner("al", "AL"), owner("sr", "SR")],
                        tool_icons: icons(5),
                    },
                    side: SideNode {
                        label: "Client notes".to_string(),
                        meta: "3 notes".to_string(),
                    },
                    subtasks: SubtaskCard {
                        title: "Subtasks: draft the outline".to_string(),
                        assignees: vec![owner("dm", "DM"), owner("ml", "ML"), owner("jk", "JK")],
                        nested: Some(NestedCard {
                            title: "Subtask 1: draft the outline".to_string(),
                            assignees: vec![owner("al", "AL")],
                            tool_icons: icons(4),
                        }),
                    },
                    split_label: "Role split".to_string(),
                    role: Role {
                        id: "pm".to_string(),
                        title: "Project manager".to_string(),
                        subline: "Owns the timeline".to_string(),
                        person: Person { name: "Dana Miller".to_string(), avatar: "DM".to_string() },
                        tool_icons: icons(3),
                    },
                    ghost_role: Some(Role {
                        id: "design".to_string(),
                        title: "Design lead".to_string(),
                        subline: "Owns the look".to_string(),
                        person: Person { name: "Mia Lee".to_string(), avatar: "ML".to_string() },
                        tool_icons: icons(3),
                    }),
                    tool_tag: "Stack",
                }
            }
        }
    }
}

#[component]
fn ModelPickerDemo() -> Element {
    use rv2_kit::illustrations::model_picker_scene::{ModelGroup, ModelOption, ModelPickerScene};
    let names = [
        ("flash", "Flash 2"),
        ("deep", "Deep reason"),
        ("large", "Large context"),
        ("compact", "Compact 3"),
        ("vision", "Vision pro"),
        ("code", "Code mini"),
        ("legacy", "Legacy v1"),
    ];
    let groups = vec![ModelGroup {
        label: "Fast models".to_string(),
        models: names
            .iter()
            .enumerate()
            .map(|(index, (id, name))| ModelOption {
                id: id.to_string(),
                name: name.to_string(),
                vendor_mark: Some(shape_mark(index)),
                available: true,
            })
            .collect(),
    }];
    let mut chosen = use_signal(|| "deep".to_string());
    rsx! {
        Item { slug: "model-picker-scene", name: "ModelPickerScene",
            div { style: "width: min(100%, 560px); margin: 0 auto;",
                ModelPickerScene {
                    groups: groups.clone(),
                    default_id: "deep",
                    on_select: move |id: String| chosen.set(id),
                }
            }
            div { class: "gl-row",
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
            id: "north-america".into(),
            label: "North America".into(),
            geometry_ref: "north-america".into(),
        },
        MapRegion {
            id: "south-america".into(),
            label: "South America".into(),
            geometry_ref: "south-america".into(),
        },
        MapRegion {
            id: "oceania".into(),
            label: "Oceania".into(),
            geometry_ref: "oceania".into(),
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
        (176.0, 66.0),
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
    let item = |id: &str, label: &str| Integration {
        id: id.to_string(),
        label: label.to_string(),
        status: status(id),
        logo: Some(glyph()),
    };
    let integrations = vec![
        item("helpdesk", "Helpdesk"),
        item("warehouse", "Warehouse"),
        item("chat", "Chat"),
        item("tracker", "Tracker"),
        item("payments", "Payments"),
    ];
    let hub = || Hub {
        title: "Analysis hub".to_string(),
        subtitle: "Every source in one view".to_string(),
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
        }
    }
}

#[component]
fn PromptDemo() -> Element {
    use rv2_kit::illustrations::keyboard_prompt_input::KeyboardPromptInput;
    let mut out = use_signal(String::new);
    // Starts paused so the gallery's first paint is the resting frame; Play runs the rotation.
    let mut paused = use_signal(|| true);
    rsx! {
        Item { slug: "keyboard-prompt-input", name: "KeyboardPromptInput",
            div { class: "gl-row",
                button { r#type: "button", onclick: move |_| paused.toggle(), if paused() { "Play" } else { "Pause" } }
            }
            KeyboardPromptInput {
                sample_prompts: vec![
                    "Summarize last week's support tickets".to_string(),
                    "Draft a reply to the open invoice".to_string(),
                    "Compare two plans for a team of ten".to_string(),
                ],
                paused: paused(),
            }
        }
        Item { slug: "keyboard-prompt-input-interactive", name: "KeyboardPromptInput (interactive)",
            KeyboardPromptInput {
                value: "Compare the two pricing tiers",
                on_submit: move |text: String| out.set(text),
            }
            div { class: "gl-out", "data-out": "keyboard-prompt-input", {out()} }
        }
    }
}

fn sample_card() -> rv2_kit::illustrations::inspected_agent_card_scene::AgentCard {
    use rv2_kit::illustrations::inspected_agent_card_scene::{AgentCard, CardFooter};
    AgentCard {
        title: "Patch writer".to_string(),
        model_label: "Large model".to_string(),
        status_label: "Running".to_string(),
        task_line: "Fix the broken link in the setup guide".to_string(),
        footer: CardFooter {
            cost_label: "$0.42".to_string(),
            time_label: "9 min".to_string(),
        },
    }
}

#[component]
fn CardDemo() -> Element {
    use rv2_kit::illustrations::inspected_agent_card_scene::{
        CardFooter, GhostCard, InspectStep, InspectedAgentCardScene,
    };
    let steps = vec![
        InspectStep {
            row_index: 0,
            note: "Header: name, model and status".to_string(),
        },
        InspectStep {
            row_index: 1,
            note: "Task: one sentence".to_string(),
        },
        InspectStep {
            row_index: 2,
            note: "Footer: cost and elapsed time".to_string(),
        },
    ];
    let ghost = GhostCard {
        title: "Link checker".to_string(),
        model_label: "Small model".to_string(),
        status_label: "Done".to_string(),
        task_line: "Check every link on the docs site".to_string(),
        footer: CardFooter {
            cost_label: "$0.18".to_string(),
            time_label: "3 min".to_string(),
        },
    };
    rsx! {
        Item { slug: "inspected-agent-card-scene", name: "InspectedAgentCardScene",
            InspectedAgentCardScene {
                card: sample_card(),
                ghost: Some(ghost),
                inspect_steps: steps,
                interactive: true,
            }
        }
    }
}

fn composer() -> rv2_kit::illustrations::inspected_composer_scene::Composer {
    rv2_kit::illustrations::inspected_composer_scene::Composer {
        message: "Draft the release notes for version 3".to_string(),
        mode_label: "Auto".to_string(),
    }
}

#[component]
fn ComposerDemo() -> Element {
    use rv2_kit::illustrations::inspected_composer_scene::InspectedComposerScene;
    rsx! {
        Item { slug: "inspected-composer-scene", name: "InspectedComposerScene",
            InspectedComposerScene {
                composer: composer(),
                connected_apps: vec![glyph(), glyph(), glyph()],
                interactive: true,
            }
        }
    }
}
