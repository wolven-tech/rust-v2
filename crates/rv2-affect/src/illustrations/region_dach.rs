//! Map of Germany, Austria and Switzerland, with the outlines of the neighbouring countries
//! fading out towards the edges. A soft pastel band travels along the outer border in a loop.
//! A menu in the top-left corner switches to the United States, China, India, Brazil or Japan;
//! the old map fades out while the new one fades in. Drawn on a 540 × 440 viewBox from Natural
//! Earth country shapes (public domain).
//!
//! Port losses: `label` takes a string only (the original also takes a function of the region).
//! The menu keeps its fixed size and does not scale down in small previews. Outside clicks close
//! the menu when focus leaves it. Path data is read from `assets/region-dach.svg`.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/region-dach.css");
/// Path data for every map, one `<path id="{region}-{kind}-{n}">` per shape.
const MAP_DATA: &str = include_str!("../../assets/region-dach.svg");
/// Height of one menu row, and the panel's inner padding above the first row.
const ROW_HEIGHT: f64 = 32.0;
const PANEL_PADDING: f64 = 5.0;
/// How long the old map takes to fade out, in ms.
const EXIT_MS: u32 = 480;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Region {
    #[default]
    Dach,
    Us,
    Cn,
    In,
    Br,
    Jp,
}

const OPTIONS: [(Region, &str, &str); 6] = [
    (
        Region::Dach,
        "DACH",
        "Map of the DACH region: Germany, Austria and Switzerland",
    ),
    (Region::Us, "United States", "Map of the United States"),
    (Region::Cn, "China", "Map of China"),
    (Region::In, "India", "Map of India"),
    (Region::Br, "Brazil", "Map of Brazil"),
    (Region::Jp, "Japan", "Map of Japan"),
];

impl Region {
    fn key(self) -> &'static str {
        match self {
            Self::Dach => "dach",
            Self::Us => "us",
            Self::Cn => "cn",
            Self::In => "in",
            Self::Br => "br",
            Self::Jp => "jp",
        }
    }

    fn index(self) -> usize {
        OPTIONS
            .iter()
            .position(|(region, _, _)| *region == self)
            .unwrap_or(0)
    }

    /// Centre and radius of the neighbours' fade, in viewBox units.
    fn fade(self) -> (f64, f64, f64) {
        match self {
            Self::Dach => (260.0, 220.0, 220.0),
            _ => (270.0, 220.0, 240.0),
        }
    }
}

/// The path data of one shape, by id.
fn shape(id: &str) -> Option<&'static str> {
    let needle = format!("id=\"{id}\" d=\"");
    let start = MAP_DATA.find(&needle)? + needle.len();
    let end = MAP_DATA[start..].find('"')? + start;
    Some(&MAP_DATA[start..end])
}

fn shapes(prefix: &str, kind: &str) -> Vec<&'static str> {
    (0..)
        .map_while(|i| shape(&format!("{prefix}-{kind}-{i}")))
        .collect()
}

fn perimeter(prefix: &str) -> &'static str {
    shape(&format!("{prefix}-perimeter"))
        .or_else(|| shape(&format!("{prefix}-perimeter-0")))
        .unwrap_or("")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    Enter,
    Exit,
}

impl Phase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Enter => "enter",
            Self::Exit => "exit",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Layer {
    key: u32,
    region: Region,
    phase: Phase,
}

#[component]
pub fn RegionDach(
    /// The shown region, when the parent controls it.
    #[props(default)]
    region: Option<Region>,
    #[props(default)] default_region: Region,
    #[props(default)] on_region_change: Option<EventHandler<Region>>,
    #[props(default = true)] show_switcher: bool,
    #[props(default, into)] class: String,
    #[props(default)] label: Option<String>,
) -> Element {
    let root = use_hook(|| format!("ui-rd-{}", next_id()));
    let mut own = use_signal(|| default_region);
    let current = region.unwrap_or(own());
    let mut layers = use_signal(|| {
        vec![Layer {
            key: 0,
            region: current,
            phase: Phase::Idle,
        }]
    });

    use_effect(use_reactive((&current,), move |(current,)| {
        let top = layers.peek().last().copied();
        if top.is_none_or(|top| top.region == current) {
            return;
        }
        let next_key = top.map_or(0, |top| top.key + 1);
        layers.with_mut(|all| {
            for layer in all.iter_mut() {
                layer.phase = Phase::Exit;
            }
            all.push(Layer {
                key: next_key,
                region: current,
                phase: Phase::Enter,
            });
        });
        spawn(async move {
            sleep(EXIT_MS).await;
            layers.with_mut(|all| all.retain(|layer| layer.phase != Phase::Exit));
        });
    }));

    let select = move |next: Region| {
        if region.is_none() {
            own.set(next);
        }
        if let Some(handler) = on_region_change.filter(|_| next != current) {
            handler.call(next);
        }
    };

    let option = OPTIONS[current.index()];
    let aria = label.unwrap_or_else(|| option.2.to_string());
    let class = class_list("ui-rd", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class,
            div { class: "ui-rd-map", role: "img", "aria-label": "{aria}",
                svg { class: "ui-rd-art", "viewBox": "0 0 540 440", "aria-hidden": "true",
                    defs {
                        filter { id: "{root}-shadow", x: "-15%", y: "-15%", width: "130%", height: "130%",
                            feDropShadow { class: "ui-rd-drop", dx: "0", dy: "3", "stdDeviation": "3.5", "floodColor": "#1F2128", "floodOpacity": "0.07" }
                        }
                        linearGradient { id: "{root}-shine", "gradientUnits": "userSpaceOnUse", x1: "0", y1: "0", x2: "540", y2: "220",
                            stop { offset: "0%", "stopColor": "#FFA0AB" }
                            stop { offset: "33%", "stopColor": "#FFB89E" }
                            stop { offset: "66%", "stopColor": "#C8A8E5" }
                            stop { offset: "100%", "stopColor": "#A8B5FF" }
                        }
                        filter { id: "{root}-blur", x: "-20%", y: "-20%", width: "140%", height: "140%",
                            feGaussianBlur { "stdDeviation": "4" }
                        }
                    }
                    for layer in layers() {
                        MapLayer { key: "{layer.key}", root: root.clone(), region: layer.region, phase: layer.phase }
                    }
                }
            }
            if show_switcher {
                Switcher { value: current, on_pick: select }
            }
        }
    }
}

#[component]
fn MapLayer(root: String, region: Region, phase: Phase) -> Element {
    let key = region.key();
    let lid = format!("{root}-{key}-{}", phase.as_str());
    let land = shapes(key, "land");
    let neighbors = shapes(key, "nb");
    let outline = perimeter(key);
    let (cx, cy, r) = region.fade();
    let fade_url = format!("url(#{lid}-fade)");
    let outside_url = format!("url(#{lid}-outside)");
    let neighbors_url = format!("url(#{lid}-neighbors)");
    let shadow_url = format!("url(#{root}-shadow)");
    let shine_url = format!("url(#{root}-shine)");
    let blur_url = format!("url(#{root}-blur)");
    rsx! {
        g { class: "ui-rd-layer", "data-phase": phase.as_str(),
            defs {
                mask { id: "{lid}-outside", "maskUnits": "userSpaceOnUse", x: "0", y: "0", width: "540", height: "440",
                    rect { x: "0", y: "0", width: "540", height: "440", fill: "#FFFFFF" }
                    for d in land.iter().copied() {
                        path { d, fill: "#000000" }
                    }
                }
                radialGradient { id: "{lid}-fade", cx: "{cx}", cy: "{cy}", r: "{r}", "gradientUnits": "userSpaceOnUse",
                    stop { offset: "0%", "stopColor": "#FFFFFF", "stopOpacity": "0.9" }
                    stop { offset: "65%", "stopColor": "#FFFFFF", "stopOpacity": "0.6" }
                    stop { offset: "90%", "stopColor": "#FFFFFF", "stopOpacity": "0.18" }
                    stop { offset: "100%", "stopColor": "#FFFFFF", "stopOpacity": "0" }
                }
                mask { id: "{lid}-neighbors", "maskUnits": "userSpaceOnUse", x: "0", y: "0", width: "540", height: "440",
                    rect { x: "0", y: "0", width: "540", height: "440", fill: "{fade_url}" }
                }
            }
            g { mask: "{neighbors_url}", class: "ui-rd-neighbors", fill: "none", stroke: "#A6ABB3", "strokeWidth": "0.7", "strokeLinejoin": "round", "strokeLinecap": "round",
                for d in neighbors.iter().copied() {
                    path { d }
                }
            }
            g { class: "ui-rd-land", filter: "{shadow_url}", fill: "#F8F8F8", stroke: "#D6D9DE", "strokeWidth": "0.9", "strokeLinejoin": "round",
                for d in land.iter().copied() {
                    path { d }
                }
            }
            g { fill: "none", stroke: "{shine_url}", "strokeLinecap": "round", "strokeLinejoin": "round",
                path { class: "ui-rd-band", d: outline, "pathLength": "1000", "strokeWidth": "6", "strokeDasharray": "180 820", filter: "{blur_url}", mask: "{outside_url}", opacity: "0.45" }
                path { class: "ui-rd-band", d: outline, "pathLength": "1000", "strokeWidth": "1.4", "strokeDasharray": "200 800", opacity: "0.32" }
                path { class: "ui-rd-band ui-rd-band--1", d: outline, "pathLength": "1000", "strokeWidth": "1.4", "strokeDasharray": "140 860", "strokeDashoffset": "-30", opacity: "0.55" }
                path { class: "ui-rd-band ui-rd-band--2", d: outline, "pathLength": "1000", "strokeWidth": "1.4", "strokeDasharray": "80 920", "strokeDashoffset": "-60" }
            }
        }
    }
}

/// Reports the choice, closes the menu and returns focus to its trigger.
fn pick_option(index: usize, on_pick: EventHandler<Region>, mut open: Signal<bool>, trigger: &str) {
    on_pick.call(OPTIONS[index].0);
    open.set(false);
    document::eval(&format!("document.getElementById({trigger:?})?.focus();"));
}

#[component]
fn Switcher(value: Region, on_pick: EventHandler<Region>) -> Element {
    let root = use_hook(|| format!("ui-rd-menu-{}", next_id()));
    let mut open = use_signal(|| false);
    let mut active = use_signal(|| value.index());
    let mut shown = use_signal(|| false);
    let selected = value.index();
    let list_id = format!("{root}-list");

    use_effect(use_reactive((&open(),), {
        let list_id = list_id.clone();
        move |(is_open,)| {
            if is_open {
                document::eval(&format!(
                    "document.getElementById({list_id:?})?.focus({{ preventScroll: true }});"
                ));
            }
        }
    }));

    let fill_top = PANEL_PADDING + ROW_HEIGHT * active() as f64;
    let fill_style = format!(
        "transform: translateY({fill_top}px); height: {ROW_HEIGHT}px; opacity: {}",
        if shown() { 1 } else { 0 }
    );
    let trigger_id = format!("{root}-trigger");
    let panel_trigger = trigger_id.clone();
    let menu_inert = (!open()).then_some("");
    let open_attr = if open() { "true" } else { "false" };
    let expanded = if open() { "true" } else { "false" };
    let selected_name = OPTIONS[selected].1;
    let trigger_label = format!("Region: {selected_name}");
    let active_option = format!("{root}-{}", active());

    rsx! {
        div { class: "ui-rd-switch",
            onfocusout: move |_| {
                open.set(false);
            },
            button {
                id: "{trigger_id}",
                r#type: "button",
                class: "ui-rd-trigger",
                "aria-haspopup": "listbox",
                "aria-expanded": expanded,
                "aria-controls": "{list_id}",
                "aria-label": "{trigger_label}",
                onclick: move |_| {
                    if open() {
                        open.set(false);
                    } else {
                        active.set(selected);
                        shown.set(false);
                        open.set(true);
                    }
                },
                onkeydown: move |event| {
                    if matches!(event.key().to_string().as_str(), "ArrowDown" | "ArrowUp") {
                        event.prevent_default();
                        active.set(selected);
                        shown.set(false);
                        open.set(true);
                    }
                },
                span { "{selected_name}" }
                svg { class: "ui-rd-chevron", "viewBox": "0 0 24 24", fill: "none", "aria-hidden": "true",
                    path { class: "ui-rd-chevron-ink", d: "M18 9.00005C18 9.00005 13.5811 15 12 15C10.4188 15 6 9 6 9", stroke: "#737373", "strokeWidth": "1.5", "strokeLinecap": "round", "strokeLinejoin": "round" }
                }
            }
            div { class: "ui-rd-menu", "data-open": open_attr, "inert": menu_inert,
                div { id: "{list_id}", class: "ui-rd-panel", role: "listbox", tabindex: "-1", "aria-label": "Select a region",
                    "aria-activedescendant": "{active_option}",
                    onkeydown: move |event| {
                        let last = OPTIONS.len() - 1;
                        let key = event.key().to_string();
                        let moved = match key.as_str() {
                            "ArrowDown" => Some((active() + 1).min(last)),
                            "ArrowUp" => Some(active().saturating_sub(1)),
                            "Home" => Some(0),
                            "End" => Some(last),
                            _ => None,
                        };
                        if let Some(next) = moved {
                            event.prevent_default();
                            active.set(next);
                            shown.set(true);
                        } else if matches!(key.as_str(), "Enter" | " ") {
                            event.prevent_default();
                            pick_option(active(), on_pick, open, &panel_trigger);
                        } else if key == "Escape" {
                            event.prevent_default();
                            event.stop_propagation();
                            open.set(false);
                            document::eval(&format!("document.getElementById({panel_trigger:?})?.focus();"));
                        } else if key == "Tab" {
                            open.set(false);
                        }
                    },
                    onpointerleave: move |_| shown.set(false),
                    span { class: "ui-rd-fill", "aria-hidden": "true", style: "{fill_style}" }
                    for (i, (_, name, _)) in OPTIONS.iter().copied().enumerate() {
                        div {
                            key: "{i}",
                            id: "{root}-{i}",
                            role: "option",
                            "aria-selected": "{i == selected}",
                            class: "ui-rd-option",
                            onpointerenter: move |_| {
                                active.set(i);
                                shown.set(true);
                            },
                            onclick: {
                                let option_trigger = trigger_id.clone();
                                move |_| pick_option(i, on_pick, open, &option_trigger)
                            },
                            span { class: "ui-rd-option-name", "{name}" }
                            span { class: "ui-rd-check", "aria-hidden": "true",
                                svg { "viewBox": "0 0 24 24", fill: "none", stroke: "currentColor", "strokeWidth": "1.75", "strokeLinecap": "round", "strokeLinejoin": "round", "aria-hidden": "true",
                                    path { d: "M5 14L8.5 17.5L19 6.5" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
