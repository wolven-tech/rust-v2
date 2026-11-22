//! Expandable panel strip: a row of feature tiles. Opening one reveals its larger preview in a
//! framed card below the row, with its caption; the other tiles collapse to slim tiles.
//!
//! Design choice: the open preview sits under the row rather than in its middle. A strip
//! that holds a preview in the middle of a narrow row has no room for its neighbours.
//! Design choice: selection is controlled when `on_open_change` is given (`open_id` is then
//! the truth, `None` closes all). Without it the strip keeps its own open tile, starting at
//! `initial_open_id` or the first panel.
//! Design choice: at narrow widths the open tile fills the row and its two neighbours peek
//! as slim tiles; the pager and swipe move the selection and wrap at the ends.
//! Not implemented: the width transition is a CSS flex transition, not a shared motion value.
//! Browser checks: click opens and collapses the previous tile, Left/Right move focus without
//! opening, Enter opens, Escape closes and returns focus, hover does not open, pager wraps,
//! one neighbour per side at narrow width, no sideways scroll.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonSize, ButtonVariant};

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/expandable-panel-strip.css");

#[derive(Clone, Debug)]
pub struct StripPanel {
    pub id: String,
    pub title: String,
    pub caption: String,
    pub preview: Element,
}

impl PartialEq for StripPanel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.title == other.title && self.caption == other.caption
    }
}

/// Index after moving `step` places from `current`, wrapping at both ends.
pub fn wrap_index(len: usize, current: usize, step: i64) -> usize {
    if len == 0 {
        return 0;
    }
    (current as i64 + step).rem_euclid(len as i64) as usize
}

/// Index that `Home` (`end == false`) or `End` reaches.
pub fn edge_index(len: usize, end: bool) -> usize {
    if end { len.saturating_sub(1) } else { 0 }
}

/// Index of the open panel, if the id is in the list.
pub fn open_index(ids: &[String], open: Option<&str>) -> Option<usize> {
    let open = open?;
    ids.iter().position(|id| id == open)
}

/// How a click or key reaches the open tile: through the host when controlled, else locally.
#[derive(Clone, Copy)]
struct PanelSelect {
    disabled: bool,
    controlled: bool,
    on_change: Option<EventHandler<Option<String>>>,
    local: Signal<Option<String>>,
}

impl PanelSelect {
    fn pick(self, id: Option<String>) {
        if self.disabled {
            return;
        }
        if self.controlled {
            if let Some(handler) = self.on_change {
                handler.call(id);
            }
        } else {
            let mut local = self.local;
            local.set(id);
        }
    }
}

fn tile_id(strip: &str, index: usize) -> String {
    format!("{strip}-tile-{index}")
}

fn focus_tile(id: &str) {
    document::eval(&format!(
        "document.getElementById({id:?})?.focus({{ preventScroll: true }});"
    ));
}

#[component]
pub fn ExpandablePanelStrip(
    panels: Vec<StripPanel>,
    #[props(default)] open_id: Option<String>,
    #[props(default)] initial_open_id: Option<String>,
    #[props(default)] on_open_change: Option<EventHandler<Option<String>>>,
    #[props(default = "Preview".to_string(), into)] preview_label: String,
    #[props(default)] disabled: bool,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let strip_id = use_hook(|| format!("kit-strip-{}", next_id()));
    let ids: Vec<String> = panels.iter().map(|p| p.id.clone()).collect();
    let local_open = use_signal(|| initial_open_id.or_else(|| ids.first().cloned()));
    let mut swipe_start = use_signal(|| None::<f64>);
    let controlled = on_open_change.is_some();
    let current_open: Option<String> = if controlled { open_id } else { local_open() };
    let open_at = open_index(&ids, current_open.as_deref());
    let len = panels.len();
    let root_class = class_list("kit-strip", &extra_classes);
    let swipe_ids = ids.clone();
    let prev_ids = ids.clone();
    let next_ids = ids.clone();

    let select = PanelSelect {
        disabled,
        controlled,
        on_change: on_open_change,
        local: local_open,
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "{root_class}", "data-disabled": disabled.then_some("true"),
            div { class: "kit-strip__row", role: "tablist", "aria-label": "{preview_label} panels",
                for (index, panel) in panels.iter().enumerate() {
                    {
                        let open = open_at == Some(index);
                        let near = open_at.is_some_and(|o| o.abs_diff(index) == 1);
                        let tid = tile_id(&strip_id, index);
                        let pick = panel.id.clone();
                        let key_strip = strip_id.clone();
                        let focus_key = tid.clone();
                        let title = panel.title.clone();
                        let caption = panel.caption.clone();
                        rsx! {
                            button {
                                key: "{panel.id}",
                                id: "{tid}",
                                r#type: "button",
                                role: "tab",
                                class: "kit-strip__tile",
                                "aria-selected": if open { "true" } else { "false" },
                                "aria-label": "{title}",
                                "aria-controls": "{key_strip}-panel",
                                tabindex: "0",
                                disabled: disabled && !open,
                                "data-open": open.then_some("true"),
                                "data-peek": near.then_some("true"),
                                onclick: move |_| {
                                    if !open {
                                        select.pick(Some(pick.clone()));
                                    }
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    let target = match e.key() {
                                        Key::ArrowRight => Some(wrap_index(len, index, 1)),
                                        Key::ArrowLeft => Some(wrap_index(len, index, -1)),
                                        Key::Home => Some(edge_index(len, false)),
                                        Key::End => Some(edge_index(len, true)),
                                        _ => None,
                                    };
                                    if let Some(target) = target {
                                        e.prevent_default();
                                        focus_tile(&tile_id(&key_strip, target));
                                    }
                                    if e.key() == Key::Escape && open {
                                        select.pick(None);
                                        focus_after_frame(&focus_key);
                                    }
                                },
                                span { class: "kit-strip__artwork", "aria-hidden": "true" }
                                span { class: "kit-strip__title", "{title}" }
                                span { class: "kit-strip__caption", "{caption}" }
                            }
                        }
                    }
                }
            }
            div { class: "kit-strip__stage",
                if let Some(index) = open_at {
                    {
                        let panel = panels[index].clone();
                        let labelled = tile_id(&strip_id, index);
                        let panel_id = format!("{strip_id}-panel");
                        rsx! {
                            section {
                                id: "{panel_id}",
                                class: "kit-strip__preview",
                                role: "tabpanel",
                                "aria-labelledby": "{labelled}",
                                "aria-label": "{preview_label}: {panel.title}",
                                onpointerdown: move |e: PointerEvent| swipe_start.set(Some(e.client_coordinates().x)),
                                onpointerup: move |e: PointerEvent| {
                                    let Some(start) = swipe_start() else { return };
                                    swipe_start.set(None);
                                    let dx = e.client_coordinates().x - start;
                                    if dx.abs() > 40.0 && len > 1 {
                                        let step = if dx < 0.0 { 1 } else { -1 };
                                        select.pick(swipe_ids.get(wrap_index(len, index, step)).cloned());
                                    }
                                },
                                div { class: "kit-strip__frame", {panel.preview.clone()} }
                                p { class: "kit-strip__figcaption", "{panel.caption}" }
                            }
                        }
                    }
                }
                if len > 1 {
                    div { class: "kit-strip__pager",
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            label: "Previous",
                            disabled: disabled,
                            onclick: move |_| {
                                let at = open_at.unwrap_or(0);
                                select.pick(prev_ids.get(wrap_index(len, at, -1)).cloned());
                            },
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            label: "Next",
                            disabled: disabled,
                            onclick: move |_| {
                                let at = open_at.unwrap_or(0);
                                select.pick(next_ids.get(wrap_index(len, at, 1)).cloned());
                            },
                        }
                    }
                }
            }
        }
    }
}
