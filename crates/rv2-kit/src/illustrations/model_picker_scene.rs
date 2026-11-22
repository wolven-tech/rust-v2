//! Model picker scene: a list of model names, each with its vendor mark. The person picks one and
//! a highlight glides to the chosen row. The list fades at its lower edge to suggest more models.
//!
//! Design choice: rows have a fixed height and group headings a fixed height, so the highlight's
//! offset is pure arithmetic (`row_offset`) and moves by transform only.
//! Design choice: `popover` shows a trigger that opens the listbox; Escape closes it without a
//! change and returns focus to the trigger. A press outside the picker root closes it through a
//! capture-phase document listener that is removed on close.
//! Design choice: the listbox keeps real focus and points at the active option with
//! `aria-activedescendant`; unavailable options are skipped by the arrow keys.
//! Browser checks: a click moves the highlight and check; arrows and Enter select; an unavailable
//! row cannot be picked and reads "unavailable"; Escape closes the popover unchanged; reduced motion
//! jumps the highlight; nothing scrolls sideways at 390 and 320.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/model-picker-scene.css");

/// Height of one option row, px.
pub const ROW_PX: u32 = 40;
/// Height of a group heading, px.
pub const HEADING_PX: u32 = 28;

#[derive(Clone, PartialEq)]
pub struct ModelOption {
    pub id: String,
    pub name: String,
    pub vendor_mark: Option<Element>,
    pub available: bool,
}

#[derive(Clone, PartialEq)]
pub struct ModelGroup {
    pub label: String,
    pub models: Vec<ModelOption>,
}

/// A keyboard move through the available options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Next,
    Previous,
    First,
    Last,
}

/// Distance from the top of the list to the row of `id`, px. `None` if no option has that id.
pub fn row_offset(groups: &[ModelGroup], id: &str) -> Option<u32> {
    let mut top = 0;
    for group in groups {
        top += HEADING_PX;
        for (index, model) in group.models.iter().enumerate() {
            if model.id == id {
                return Some(top + index as u32 * ROW_PX);
            }
        }
        top += group.models.len() as u32 * ROW_PX;
    }
    None
}

/// The option one step away from `current` among `available` ids, clamped at the ends.
pub fn step_active(available: &[String], current: Option<&str>, step: Step) -> Option<String> {
    if available.is_empty() {
        return None;
    }
    let position = current.and_then(|id| available.iter().position(|a| a == id));
    let last = available.len() - 1;
    let index = match (step, position) {
        (Step::First, _) => 0,
        (Step::Last, _) => last,
        (Step::Next, Some(p)) => (p + 1).min(last),
        (Step::Previous, Some(p)) => p.saturating_sub(1),
        (Step::Next, None) => 0,
        (Step::Previous, None) => last,
    };
    Some(available[index].clone())
}

/// Ids of every available option, in list order.
pub fn available_ids(groups: &[ModelGroup]) -> Vec<String> {
    groups
        .iter()
        .flat_map(|g| g.models.iter())
        .filter(|m| m.available)
        .map(|m| m.id.clone())
        .collect()
}

/// Reports `outside` on a press outside the picker root. Re-running replaces the previous listener.
fn outside_js(root_id: &str) -> String {
    format!(
        r#"const owners = (window.__kitOutside ||= {{}});
owners[{root_id:?}]?.();
const press = (e) => {{
  const root = document.getElementById({root_id:?});
  if (root && !root.contains(e.target)) {{ try {{ dioxus.send("outside"); }} catch (_) {{}} }}
}};
document.addEventListener("pointerdown", press, true);
owners[{root_id:?}] = () => document.removeEventListener("pointerdown", press, true);"#
    )
}

fn release_outside_js(root_id: &str) -> String {
    format!("window.__kitOutside?.[{root_id:?}]?.();")
}

#[component]
pub fn ModelPickerScene(
    groups: Vec<ModelGroup>,
    #[props(default, into)] selected_id: Option<String>,
    #[props(default, into)] default_id: Option<String>,
    #[props(default)] on_select: Option<EventHandler<String>>,
    #[props(default)] popover: bool,
    #[props(default)] class: String,
) -> Element {
    let uid = use_hook(|| format!("kit-model-{}", next_id()));
    let first_available = available_ids(&groups).into_iter().next();
    let mut internal = use_signal(|| default_id.clone().or(first_available));
    let mut open = use_signal(|| !popover);
    let mut active = use_signal(|| None::<String>);
    let selected = selected_id.clone().or_else(|| internal());
    let shown_active = active().or_else(|| selected.clone());
    let list_id = format!("{uid}-list");
    let trigger_id = format!("{uid}-trigger");
    let trigger_label = groups
        .iter()
        .flat_map(|g| g.models.iter())
        .find(|m| Some(&m.id) == selected.as_ref())
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "Choose a model".to_string());
    let band_y = selected
        .as_deref()
        .and_then(|id| row_offset(&groups, id))
        .unwrap_or(0);
    let band_style = format!("transform: translateY({band_y}px);");
    let available = available_ids(&groups);
    let class = class_list("kit-model", &class);
    let is_open = open();

    let return_target = trigger_id.clone();
    let choose = use_callback(move |id: String| {
        internal.set(Some(id.clone()));
        if let Some(handler) = on_select {
            handler.call(id);
        }
        if popover {
            open.set(false);
            focus_after_frame(&return_target);
        }
    });
    let escape_target = trigger_id.clone();
    let root_id = format!("{uid}-root");
    let outside_open = popover && is_open;
    use_effect(use_reactive((&outside_open,), {
        let root_id = root_id.clone();
        move |(now_open,)| {
            if !now_open {
                document::eval(&release_outside_js(&root_id));
                return;
            }
            let root_id = root_id.clone();
            spawn(async move {
                let mut events = document::eval(&outside_js(&root_id));
                while let Ok(message) = events.recv::<String>().await {
                    if message == "outside" {
                        open.set(false);
                    }
                }
            });
        }
    }));

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class, id: "{root_id}", "data-popover": popover.then_some("true"),
            if popover {
                button {
                    r#type: "button",
                    id: "{trigger_id}",
                    class: "kit-model__trigger",
                    "aria-haspopup": "listbox",
                    "aria-expanded": "{is_open}",
                    "aria-controls": "{list_id}",
                    onclick: {
                        let list_for_focus = list_id.clone();
                        move |_| {
                            let next = !open();
                            open.set(next);
                            if next {
                                focus_after_frame(&list_for_focus);
                            }
                        }
                    },
                    span { class: "kit-model__trigger-label", "{trigger_label}" }
                }
            }
            if is_open {
                div { class: "kit-model__panel",
                    div { class: "kit-model__list",
                        div {
                            id: "{list_id}",
                            class: "kit-model__options",
                            role: "listbox",
                            tabindex: "0",
                            "aria-label": "Models",
                            "aria-activedescendant": shown_active.as_ref().map(|id| format!("{uid}-{id}")),
                            onkeydown: {
                                let available = available.clone();
                                let selected = selected.clone();
                                move |event: KeyboardEvent| {
                                    let step = match event.key() {
                                        Key::ArrowDown => Some(Step::Next),
                                        Key::ArrowUp => Some(Step::Previous),
                                        Key::Home => Some(Step::First),
                                        Key::End => Some(Step::Last),
                                        _ => None,
                                    };
                                    if let Some(step) = step {
                                        event.prevent_default();
                                        let current = active().or_else(|| selected.clone());
                                        active.set(step_active(&available, current.as_deref(), step));
                                    } else if event.key() == Key::Enter || event.key() == Key::Character(" ".into()) {
                                        event.prevent_default();
                                        if let Some(id) = active().or_else(|| selected.clone()) {
                                            choose.call(id);
                                        }
                                    } else if event.key() == Key::Escape && popover {
                                        event.prevent_default();
                                        open.set(false);
                                        focus_after_frame(&escape_target);
                                    }
                                }
                            },
                            div { class: "kit-model__band", style: "{band_style}", "aria-hidden": "true" }
                            for group in groups.iter() {
                                div { class: "kit-model__group", role: "group", "aria-label": group.label.clone(),
                                    div { class: "kit-model__heading", "aria-hidden": "true", {group.label.clone()} }
                                    for model in group.models.iter() {
                                        {
                                            let is_selected = selected.as_deref() == Some(model.id.as_str());
                                            let is_active = shown_active.as_deref() == Some(model.id.as_str());
                                            let option_id = format!("{uid}-{}", model.id);
                                            let id = model.id.clone();
                                            let usable = model.available;
                                            let row_class = if usable { "kit-model__row" } else { "kit-model__row is-unavailable" };
                                            rsx! {
                                                div {
                                                    id: "{option_id}",
                                                    class: row_class,
                                                    role: "option",
                                                    "aria-selected": "{is_selected}",
                                                    "aria-disabled": (!usable).then_some("true"),
                                                    "data-active": is_active.then_some("true"),
                                                    onmouseenter: {
                                                        let id = id.clone();
                                                        move |_| {
                                                            if usable {
                                                                active.set(Some(id.clone()));
                                                            }
                                                        }
                                                    },
                                                    onclick: {
                                                        let id = id.clone();
                                                        move |_| {
                                                            if usable {
                                                                choose.call(id.clone());
                                                            }
                                                        }
                                                    },
                                                    span { class: "kit-model__mark", "aria-hidden": "true",
                                                        {model.vendor_mark.clone()}
                                                    }
                                                    span { class: "kit-model__name", {model.name.clone()} }
                                                    if !model.available {
                                                        span { class: "kit-model__note", "unavailable" }
                                                    }
                                                    if is_selected {
                                                        svg { class: "kit-model__check", "viewBox": "0 0 16 16", "aria-hidden": "true",
                                                            path { "d": "M3 8.5l3 3 7-7" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        span { class: "kit-model__fade", "aria-hidden": "true" }
                    }
                }
            }
        }
    }
}
