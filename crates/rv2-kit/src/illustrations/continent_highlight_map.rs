//! Continent highlight map: a world map with one region picked out, to show where a business
//! operates or where a region's data comes from. The person switches the region with a menu button.
//!
//! Design choice: the component ships no land shapes. The caller passes `land` pieces in a 1000 x 500
//! drawing space, and each region names one piece through `geometry_ref`. A grouping (for example
//! "Americas") is its own piece, drawn beneath the continents, so a click on land reaches the
//! continent first.
//! Design choice: every region's highlight is always mounted and fades by opacity, so the old
//! highlight fades out as the new one fades in with no unmount.
//! Design choice: the coastal band is the highlight's own outline stroked on top of it. Its dash
//! offset moves along the coastline, so the band draws in along the new coast as the highlight moves.
//! Design choice: the switch and its listbox follow the menu-button pattern; Escape closes without a
//! change and returns focus to the switch.
//! Browser checks: choosing a region highlights exactly its land; Escape closes unchanged; hovering a
//! non-selected region outlines it without changing the selection; reduced motion applies the change
//! in one step; the map fills the width at 390 and 320 with no sideways scroll.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/continent-highlight-map.css");

/// Drawing space of the map, in units.
pub const MAP_WIDTH: f64 = 1000.0;
pub const MAP_HEIGHT: f64 = 500.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BandStyle {
    #[default]
    Coastal,
    None,
}

impl BandStyle {
    fn as_str(self) -> &'static str {
        match self {
            Self::Coastal => "coastal",
            Self::None => "none",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct MapRegion {
    pub id: String,
    pub label: String,
    pub geometry_ref: String,
}

/// One piece of land: polygons in drawing units, named by `geometry_ref`.
#[derive(Clone, PartialEq)]
pub struct LandPiece {
    pub geometry_ref: String,
    pub polygons: Vec<Vec<(f64, f64)>>,
}

/// SVG path data for a set of polygons.
pub fn polygons_path(polygons: &[Vec<(f64, f64)>]) -> String {
    let mut d = String::new();
    for polygon in polygons {
        for (i, (x, y)) in polygon.iter().enumerate() {
            let verb = if i == 0 { 'M' } else { 'L' };
            d.push_str(&format!("{verb}{x:.1} {y:.1} "));
        }
        d.push('Z');
    }
    d
}

/// The land piece a region is drawn from.
pub fn region_land<'a>(
    regions: &[MapRegion],
    land: &'a [LandPiece],
    region_id: &str,
) -> Option<&'a LandPiece> {
    let region = regions.iter().find(|r| r.id == region_id)?;
    land.iter()
        .find(|piece| piece.geometry_ref == region.geometry_ref)
}

/// The first region whose land is the piece `geometry_ref`, if any.
pub fn region_for_land<'a>(regions: &'a [MapRegion], geometry_ref: &str) -> Option<&'a MapRegion> {
    regions.iter().find(|r| r.geometry_ref == geometry_ref)
}

/// The region one arrow or Home/End step from `current` (an index), clamped to the list.
pub fn step_region_index(len: usize, current: usize, step: RegionStep) -> usize {
    let last = len.saturating_sub(1);
    match step {
        RegionStep::Next => (current + 1).min(last),
        RegionStep::Previous => current.saturating_sub(1),
        RegionStep::First => 0,
        RegionStep::Last => last,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionStep {
    Next,
    Previous,
    First,
    Last,
}

#[component]
pub fn ContinentHighlightMap(
    regions: Vec<MapRegion>,
    land: Vec<LandPiece>,
    #[props(default, into)] selected_region_id: Option<String>,
    #[props(default, into)] default_region_id: Option<String>,
    #[props(default)] on_region_change: Option<EventHandler<String>>,
    #[props(default)] band_style: BandStyle,
    #[props(default)] show_legend: bool,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
) -> Element {
    let uid = use_hook(|| format!("kit-map-{}", next_id()));
    let first = regions.first().map(|r| r.id.clone());
    let mut internal = use_signal(|| default_region_id.clone().or(first));
    let mut open = use_signal(|| false);
    let mut hovered = use_signal(|| None::<String>);
    let mut active = use_signal(|| 0usize);
    let selected = selected_region_id.clone().or_else(|| internal());
    let selected_label = regions
        .iter()
        .find(|r| Some(&r.id) == selected.as_ref())
        .map(|r| r.label.clone())
        .unwrap_or_else(|| "None".to_string());
    let class = class_list("kit-map", &class);
    let trigger_id = format!("{uid}-switch");
    let list_id = format!("{uid}-regions");
    let return_target = trigger_id.clone();
    let choose = use_callback(move |id: String| {
        internal.set(Some(id.clone()));
        open.set(false);
        if let Some(handler) = on_region_change {
            handler.call(id);
        }
        focus_after_frame(&return_target);
    });
    let escape_target = trigger_id.clone();
    let listbox_regions = regions.clone();
    let regions_for_open = regions.clone();
    let selected_for_open = selected.clone();
    let list_for_focus = list_id.clone();
    let land_for_paths = land.clone();
    let is_open = open() && !disabled;
    let hover_ref = hovered();
    let outline_ref = hover_ref
        .as_deref()
        .and_then(|id| regions.iter().find(|r| r.id == id).cloned());
    let outline_ref = outline_ref.map(|r| r.geometry_ref);
    let outline_d = outline_ref
        .as_ref()
        .and_then(|g| land.iter().find(|p| &p.geometry_ref == g))
        .map(|p| polygons_path(&p.polygons));
    let legend = show_legend;
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class, "data-band": band_style.as_str(), "data-disabled": disabled.then_some("true"),
            div { class: "kit-map__toolbar",
                button {
                    r#type: "button",
                    id: "{trigger_id}",
                    class: "kit-map__switch",
                    disabled,
                    "aria-haspopup": "listbox",
                    "aria-expanded": "{is_open}",
                    "aria-controls": "{list_id}",
                    onclick: move |_| {
                        let next = !open();
                        open.set(next);
                        if next {
                            active.set(
                                selected_for_open
                                    .as_ref()
                                    .and_then(|id| regions_for_open.iter().position(|r| &r.id == id))
                                    .unwrap_or(0),
                            );
                            focus_after_frame(&list_for_focus);
                        }
                    },
                    span { class: "kit-map__switch-name", "Region" }
                    span { class: "kit-map__switch-value", "{selected_label}" }
                }
                if is_open {
                    div {
                        id: "{list_id}",
                        class: "kit-map__list",
                        role: "listbox",
                        tabindex: "0",
                        "aria-label": "Regions",
                        "aria-activedescendant": format!("{uid}-opt-{}", active()),
                        onkeydown: move |event: KeyboardEvent| {
                            let len = listbox_regions.len();
                            let step = match event.key() {
                                Key::ArrowDown => Some(RegionStep::Next),
                                Key::ArrowUp => Some(RegionStep::Previous),
                                Key::Home => Some(RegionStep::First),
                                Key::End => Some(RegionStep::Last),
                                _ => None,
                            };
                            if let Some(step) = step {
                                event.prevent_default();
                                active.set(step_region_index(len, active(), step));
                            } else if event.key() == Key::Enter || event.key() == Key::Character(" ".into()) {
                                event.prevent_default();
                                if let Some(region) = listbox_regions.get(active()) {
                                    choose.call(region.id.clone());
                                }
                            } else if event.key() == Key::Escape {
                                event.prevent_default();
                                open.set(false);
                                focus_after_frame(&escape_target);
                            }
                        },
                        for (index, region) in regions.iter().enumerate() {
                            {
                                let id = region.id.clone();
                                let label = region.label.clone();
                                let is_selected = Some(&id) == selected.as_ref();
                                let is_active = index == active();
                                rsx! {
                                    div {
                                        id: "{uid}-opt-{index}",
                                        class: "kit-map__option",
                                        role: "option",
                                        "aria-selected": "{is_selected}",
                                        "data-active": is_active.then_some("true"),
                                        onclick: move |_| choose.call(id.clone()),
                                        {label}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "kit-map__frame", role: "img",
                "aria-label": "World map, {selected_label} highlighted",
                svg { class: "kit-map__svg", "viewBox": "0 0 1000 500", "aria-hidden": "true",
                    for piece in land_for_paths.iter() {
                        {
                            let d = polygons_path(&piece.polygons);
                            let geometry_ref = piece.geometry_ref.clone();
                            let land_for_hit = regions.clone();
                            rsx! {
                                path { class: "kit-map__land", d: "{d}",
                                    "data-ref": "{geometry_ref}",
                                    onmouseenter: {
                                        let geometry_ref = geometry_ref.clone();
                                        let land_for_hit = land_for_hit.clone();
                                        move |_| {
                                            if !disabled {
                                                hovered.set(region_for_land(&land_for_hit, &geometry_ref).map(|r| r.id.clone()));
                                            }
                                        }
                                    },
                                    onmouseleave: move |_| hovered.set(None),
                                    onclick: {
                                        let geometry_ref = geometry_ref.clone();
                                        let land_for_hit = land_for_hit.clone();
                                        move |_| {
                                            if disabled {
                                                return;
                                            }
                                            if let Some(region) = region_for_land(&land_for_hit, &geometry_ref) {
                                                choose.call(region.id.clone());
                                            }
                                        }
                                    },
                                }
                            }
                        }
                    }
                    for region in regions.iter() {
                        {
                            let is_selected = Some(&region.id) == selected.as_ref();
                            let piece = region_land(&regions, &land_for_paths, &region.id);
                            let d = piece.map(|p| polygons_path(&p.polygons)).unwrap_or_default();
                            let state = if is_selected { "true" } else { "false" };
                            let region_id = region.id.clone();
                            rsx! {
                                path { class: "kit-map__highlight", d: "{d}", "data-selected": state, "data-region": region_id.clone() }
                                path { class: "kit-map__band", d: "{d}", "pathLength": "1", "data-selected": state, "data-region": region_id }
                            }
                        }
                    }
                    if let Some(d) = outline_d.clone() {
                        path { class: "kit-map__outline", d: "{d}" }
                    }
                }
                if legend {
                    ul { class: "kit-map__legend",
                        for region in regions.iter() {
                            li { "data-selected": (Some(&region.id) == selected.as_ref()).then_some("true"), {region.label.clone()} }
                        }
                    }
                }
            }
            p { class: "kit-map__caption", "Highlighted: {selected_label}" }
        }
    }
}
