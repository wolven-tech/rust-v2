//! Assistant memory manager: what the assistant remembers about a person,
//! grouped, with edit, pause, forget (undoable) and a master "Use memory" switch.
//!
//! Design choice: when `on_list_change` is set the panel shows exactly the
//! list it is given; otherwise edits stay in the panel's own state.
//! Design choice: `today_label` stands in for "today" on new memories
//! (no clock on wasm32). An empty list shows the same "No memories here" line
//! as an empty group.
//! Design choice: the new-memory group choice is a native `select`, so arrow
//! keys are the platform's; the chip row carries the Left/Right radio behaviour.
//! Design choice: Tab order follows the DOM, which matches the spec order.
//! Browser checks: the fold and fade of filtered rows, the undo window timing,
//! the widening New memory editor, the narrow switch track, and the chip
//! scroll with edge fade at narrow widths.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/assistant-memory-manager.css");

#[derive(Clone, PartialEq, Debug)]
pub struct Memory {
    pub memory_id: String,
    pub memory_text: String,
    pub memory_group: String,
    pub source_label: Option<String>,
    pub date_label: Option<String>,
    pub is_paused: bool,
}

/// A button in the footer row; Undo takes its place for the undo window.
#[derive(Clone, PartialEq)]
pub struct FooterAction {
    pub action_label: String,
    pub on_action: EventHandler<()>,
}

/// Group names in `order` first (when present), then any others by first appearance.
pub fn group_names(list: &[Memory], order: &[String]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for group in order {
        if list.iter().any(|m| &m.memory_group == group) && !names.contains(group) {
            names.push(group.clone());
        }
    }
    for memory in list {
        if !names.contains(&memory.memory_group) {
            names.push(memory.memory_group.clone());
        }
    }
    names
}

/// `8 memories · 1 paused`; the paused part appears only when something is paused.
pub fn summary_line(list: &[Memory]) -> String {
    let total = list.len();
    let paused = list.iter().filter(|m| m.is_paused).count();
    let noun = if total == 1 { "memory" } else { "memories" };
    if paused == 0 {
        format!("{total} {noun}")
    } else {
        format!("{total} {noun} · {paused} paused")
    }
}

/// Removes a memory, returning its former index for Undo.
pub fn remove_memory(list: &[Memory], id: &str) -> Option<(usize, Memory, Vec<Memory>)> {
    let index = list.iter().position(|m| m.memory_id == id)?;
    let mut next = list.to_vec();
    let removed = next.remove(index);
    Some((index, removed, next))
}

/// Reinstates a memory at `index`, clamped to the list length.
pub fn insert_memory(list: &[Memory], index: usize, memory: Memory) -> Vec<Memory> {
    let mut next = list.to_vec();
    let at = index.min(next.len());
    next.insert(at, memory);
    next
}

/// Applies `change` to the memory with `id`; other memories are untouched.
pub fn map_memory(list: &[Memory], id: &str, change: impl FnOnce(&mut Memory)) -> Vec<Memory> {
    let mut next = list.to_vec();
    if let Some(memory) = next.iter_mut().find(|m| m.memory_id == id) {
        change(memory);
    }
    next
}

/// Stable tint slot (0 to 5) for a group, so one group keeps one colour.
fn tone(seed: &str) -> u32 {
    seed.bytes().fold(0_u32, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u32::from(b))
    }) % 6
}

/// Case-insensitive text match; an empty query matches everything.
pub fn matches_query(memory: &Memory, query: &str) -> bool {
    let needle = query.trim().to_lowercase();
    needle.is_empty() || memory.memory_text.to_lowercase().contains(&needle)
}

fn focus_id(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// The panel. Memories are listed by group; each can be edited, paused or forgotten.
#[component]
pub fn AssistantMemoryManager(
    #[props(default)] memory_items: Vec<Memory>,
    #[props(default)] on_list_change: Option<EventHandler<Vec<Memory>>>,
    #[props(default)] on_add: Option<EventHandler<Memory>>,
    #[props(default)] on_text_edit: Option<EventHandler<(String, String)>>,
    #[props(default)] on_pause_change: Option<EventHandler<(String, bool)>>,
    #[props(default)] on_forget: Option<EventHandler<String>>,
    #[props(default)] on_restore: Option<EventHandler<Memory>>,
    #[props(default)] use_memory_on: Option<bool>,
    #[props(default = true)] initial_use_on: bool,
    #[props(default)] on_use_change: Option<EventHandler<bool>>,
    #[props(default)] group_order: Vec<String>,
    #[props(default = "Memory".to_string(), into)] heading_text: String,
    #[props(default = 360.0)] list_max_height: f64,
    #[props(default = 5_000)] undo_window_ms: u32,
    #[props(default = "You added".to_string(), into)] added_source_label: String,
    #[props(default = "Today".to_string(), into)] today_label: String,
    #[props(default)] footer_action: Option<FooterAction>,
    #[props(default, into)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-memory-{}", next_id()));
    let controlled_list = on_list_change.is_some();
    let mut items = use_signal(|| memory_items.clone());
    let mut use_on = use_signal(|| initial_use_on);
    let mut group_filter = use_signal(|| Option::<String>::None);
    let mut query = use_signal(String::new);
    let mut editing = use_signal(|| Option::<(String, String)>::None);
    let mut composing = use_signal(|| Option::<(String, String)>::None);
    let mut undo = use_signal(|| Option::<(u64, usize, Memory)>::None);
    let mut undo_token = use_signal(|| 0_u64);
    let mut status = use_signal(String::new);

    let list: Vec<Memory> = if controlled_list {
        memory_items.clone()
    } else {
        items()
    };
    let groups = group_names(&list, &group_order);
    let using = use_memory_on.unwrap_or(use_on());
    let selected_group = group_filter();
    let search = query();
    let edit_state = editing();
    let compose_state = composing();
    let undo_state = undo();

    // Commits a new list: the host gets it when it owns the list, otherwise the panel keeps it.
    let mut commit = move |next: Vec<Memory>| {
        if let Some(handler) = on_list_change {
            handler.call(next);
        } else {
            items.set(next);
        }
    };

    let mut set_use = move |enabled: bool| {
        if use_memory_on.is_none() {
            use_on.set(enabled);
        }
        if let Some(handler) = on_use_change {
            handler.call(enabled);
        }
    };

    let forget = {
        let list = list.clone();
        move |id: String| {
            let Some((index, memory, next)) = remove_memory(&list, &id) else {
                return;
            };
            commit(next);
            if let Some(handler) = on_forget {
                handler.call(id.clone());
            }
            let token = undo_token() + 1;
            undo_token.set(token);
            undo.set(Some((token, index, memory.clone())));
            status.set(format!("Forgot {}", memory.memory_text));
            spawn(async move {
                sleep(undo_window_ms).await;
                if undo_token() == token {
                    undo.set(None);
                }
            });
        }
    };

    let restore = {
        let list = list.clone();
        move |_| {
            let Some((_, index, memory)) = undo() else {
                return;
            };
            undo.set(None);
            let next = insert_memory(&list, index, memory.clone());
            commit(next);
            status.set(format!("Restored {}", memory.memory_text));
            if let Some(handler) = on_restore {
                handler.call(memory);
            }
        }
    };

    let save_edit = {
        let list = list.clone();
        move |id: String, text: String| {
            let trimmed = text.trim().to_string();
            editing.set(None);
            if trimmed.is_empty() {
                return;
            }
            let next = map_memory(&list, &id, |m| m.memory_text = trimmed.clone());
            commit(next);
            if let Some(handler) = on_text_edit {
                handler.call((id, trimmed));
            }
            status.set("Memory saved".to_string());
        }
    };

    let toggle_pause = {
        let list = list.clone();
        move |id: String| {
            let Some(current) = list.iter().find(|m| m.memory_id == id) else {
                return;
            };
            let paused = !current.is_paused;
            let next = map_memory(&list, &id, |m| m.is_paused = paused);
            commit(next);
            if let Some(handler) = on_pause_change {
                handler.call((id, paused));
            }
        }
    };

    let mut save_new = {
        let list = list.clone();
        let added_source = added_source_label.clone();
        let today = today_label.clone();
        let id_prefix = base.clone();
        move |text: String, group: String| {
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                return;
            }
            let memory = Memory {
                memory_id: format!("{id_prefix}-new-{}", next_id()),
                memory_text: trimmed,
                memory_group: group,
                source_label: Some(added_source.clone()),
                date_label: Some(today.clone()),
                is_paused: false,
            };
            let mut next = list.clone();
            next.push(memory.clone());
            commit(next);
            composing.set(None);
            status.set("Memory added".to_string());
            if let Some(handler) = on_add {
                handler.call(memory);
            }
        }
    };

    let total = list.len();
    let paused_total = list.iter().filter(|m| m.is_paused).count();
    let summary = summary_line(&list);
    let status_text = status();
    let note_hidden = if using { "true" } else { "false" };
    let switch_checked = if using { "true" } else { "false" };
    let list_class = if using {
        "kit-memory__list"
    } else {
        "kit-memory__list is-off"
    };
    let root_cls = class_list("kit-memory", &extra_class);
    let max_height = format!("max-height: {list_max_height}px;");
    let undo_visible = undo_state.is_some();
    let undo_text = undo_state
        .as_ref()
        .map(|(_, _, m)| format!("Undo “{}”", m.memory_text));
    let chip_names: Vec<(Option<String>, String, usize)> =
        std::iter::once((None, "All".to_string(), total))
            .chain(groups.iter().map(|g| {
                (
                    Some(g.clone()),
                    g.clone(),
                    list.iter().filter(|m| &m.memory_group == g).count(),
                )
            }))
            .collect();
    let chip_ids: Vec<String> = (0..chip_names.len())
        .map(|index| format!("{base}-chip-{index}"))
        .collect();
    let chip_values: Vec<Option<String>> = chip_names.iter().map(|(v, _, _)| v.clone()).collect();
    let group_views: Vec<(String, Vec<Memory>, bool)> = groups
        .iter()
        .map(|g| {
            let members: Vec<Memory> = list
                .iter()
                .filter(|m| &m.memory_group == g && matches_query(m, &search))
                .cloned()
                .collect();
            let folded = selected_group.as_ref().is_some_and(|sel| sel != g);
            (g.clone(), members, folded)
        })
        .collect();
    let group_choices: Vec<String> = groups.clone();
    let new_group_default = groups.first().cloned().unwrap_or_default();
    let compose_value = compose_text(&compose_state);
    let compose_group_value = compose_group(&compose_state, &new_group_default);

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: root_cls, role: "group", "aria-label": "{heading_text}",
            header { class: "kit-memory__head",
                h2 { class: "kit-memory__title", "{heading_text}" }
                label { class: "kit-memory__switch",
                    input {
                        r#type: "checkbox",
                        role: "switch",
                        "aria-checked": "{switch_checked}",
                        checked: using,
                        onchange: move |_| set_use(!using),
                    }
                    span { "Use memory" }
                }
                p { class: "kit-memory__note", "data-shown": "{note_hidden}",
                    "Memories are not used or stored in new conversations. They stay listed."
                }
            }
            div { class: "kit-memory__filters",
                div { class: "kit-memory__chips", role: "radiogroup", "aria-label": "Groups",
                    for (index, (value, label, count)) in chip_names.into_iter().enumerate() {
                        {
                            let chip_id = chip_ids[index].clone();
                            let selected = match (&value, &selected_group) {
                                (None, None) => true,
                                (Some(a), Some(b)) => a == b,
                                _ => false,
                            };
                            let checked = if selected { "true" } else { "false" };
                            let tab_index = if selected { "0" } else { "-1" };
                            let chip_text = format!("{label} {count}");
                            let pick = value.clone();
                            let keys_values = chip_values.clone();
                            let keys_ids = chip_ids.clone();
                            rsx! {
                                button {
                                    key: "{chip_id}",
                                    id: "{chip_id}",
                                    r#type: "button",
                                    role: "radio",
                                    "aria-checked": "{checked}",
                                    tabindex: "{tab_index}",
                                    class: "kit-memory__chip",
                                    onclick: move |_| group_filter.set(pick.clone()),
                                    onkeydown: move |event: KeyboardEvent| {
                                        let step: isize = match event.key() {
                                            Key::ArrowRight => 1,
                                            Key::ArrowLeft => -1,
                                            _ => return,
                                        };
                                        event.prevent_default();
                                        let current = keys_values
                                            .iter()
                                            .position(|v| v == &group_filter())
                                            .unwrap_or(0) as isize;
                                        let len = keys_values.len() as isize;
                                        let next = (current + step).rem_euclid(len) as usize;
                                        group_filter.set(keys_values[next].clone());
                                        focus_id(&keys_ids[next]);
                                    },
                                    "{chip_text}"
                                }
                            }
                        }
                    }
                }
                input {
                    r#type: "search",
                    class: "kit-memory__search",
                    "aria-label": "Search memories",
                    placeholder: "Search",
                    value: "{search}",
                    oninput: move |event: FormEvent| query.set(event.value()),
                    onkeydown: move |event: KeyboardEvent| {
                        if matches!(event.key(), Key::Escape) {
                            query.set(String::new());
                        }
                    },
                }
            }
            div { class: "{list_class}", style: "{max_height}",
                if total == 0 {
                    p { class: "kit-memory__empty", "No memories here" }
                }
                for (group, members, folded) in group_views {
                    {
                        let folded_attr = if folded { "true" } else { "false" };
                        let group_label = group.clone();
                        let group_style = format!("--t: var(--kit-id-{})", tone(&group) + 1);
                        let empty_here = members.is_empty();
                        rsx! {
                            section {
                                key: "{group}",
                                class: "kit-memory__group",
                                role: "region",
                                "aria-label": "{group_label}",
                                "data-folded": "{folded_attr}",
                                h3 { class: "kit-memory__group-title", "{group_label}" }
                                if empty_here {
                                    p { class: "kit-memory__empty", "No memories here" }
                                }
                                ul { class: "kit-memory__rows",
                                    for memory in members {
                                        {
                                            let id = memory.memory_id.clone();
                                            let text = memory.memory_text.clone();
                                            let is_editing = edit_state.as_ref().is_some_and(|(eid, _)| eid == &id);
                                            let edit_value = edit_state
                                                .as_ref()
                                                .filter(|(eid, _)| eid == &id)
                                                .map(|(_, t)| t.clone())
                                                .unwrap_or_default();
                                            let source = memory.source_label.clone().unwrap_or_default();
                                            let date = memory.date_label.clone().unwrap_or_default();
                                            let paused = memory.is_paused;
                                            let row_class = if paused { "kit-memory__row is-paused" } else { "kit-memory__row" };
                                            let pause_label = if paused { "Resume" } else { "Pause" };
                                            let edit_id = id.clone();
                                            let edit_text = text.clone();
                                            let pause_id = id.clone();
                                            let forget_id = id.clone();
                                            let save_id = id.clone();
                                            let save_fn = save_edit.clone();
                                            let mut forget_fn = forget.clone();
                                            let mut pause_fn = toggle_pause.clone();
                                            let forget_label = format!("Forget {text}");
                                            let edit_label = format!("Edit {text}");
                                            let pause_aria = format!("{pause_label} {text}");
                                            rsx! {
                                                li { key: "{id}", class: row_class, "data-memory-id": "{id}",
                                                    span { class: "kit-memory__tile", style: "{group_style}", aria_hidden: "true",
                                                        svg { view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round",
                                                            path { d: "M4 2.2h8a.8.8 0 0 1 .8.8v10.8L8 11.6l-4.8 2.2V3a.8.8 0 0 1 .8-.8z" }
                                                        }
                                                    }
                                                    if is_editing {
                                                        div { class: "kit-memory__editor",
                                                            textarea {
                                                                class: "kit-memory__input",
                                                                "aria-label": "Edit memory",
                                                                value: "{edit_value}",
                                                                oninput: {
                                                                    let save_id = save_id.clone();
                                                                    move |event: FormEvent| {
                                                                        editing.set(Some((save_id.clone(), event.value())));
                                                                    }
                                                                },
                                                                onkeydown: {
                                                                    let mut save_fn = save_fn.clone();
                                                                    let save_id = save_id.clone();
                                                                    move |event: KeyboardEvent| {
                                                                        if matches!(event.key(), Key::Escape) {
                                                                            editing.set(None);
                                                                        } else if matches!(event.key(), Key::Enter) && !event.modifiers().shift() {
                                                                            event.prevent_default();
                                                                            let text = editing().map(|(_, t)| t).unwrap_or_default();
                                                                            save_fn(save_id.clone(), text);
                                                                        }
                                                                    }
                                                                },
                                                            }
                                                            button {
                                                                r#type: "button",
                                                                class: "kit-memory__btn",
                                                                onclick: {
                                                                    let mut save_fn = save_fn.clone();
                                                                    let save_id = edit_id.clone();
                                                                    move |_| {
                                                                        let text = editing().map(|(_, t)| t).unwrap_or_default();
                                                                        save_fn(save_id.clone(), text);
                                                                    }
                                                                },
                                                                "Save"
                                                            }
                                                            button {
                                                                r#type: "button",
                                                                class: "kit-memory__btn",
                                                                onclick: move |_| editing.set(None),
                                                                "Cancel"
                                                            }
                                                        }
                                                    } else {
                                                        p { class: "kit-memory__text", "{text}" }
                                                        p { class: "kit-memory__meta",
                                                            if paused {
                                                                span { class: "kit-memory__tag",
                                                                    svg { view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", "aria-hidden": "true",
                                                                        path { d: "M6 4.5v7M10 4.5v7" }
                                                                    }
                                                                    "Paused"
                                                                }
                                                            }
                                                            span { class: "kit-memory__source", "{source}" }
                                                            span { class: "kit-memory__date", "{date}" }
                                                        }
                                                        div { class: "kit-memory__actions",
                                                            button {
                                                                r#type: "button",
                                                                class: "kit-memory__btn",
                                                                "aria-label": "{edit_label}",
                                                                onclick: move |_| editing.set(Some((edit_id.clone(), edit_text.clone()))),
                                                                "Edit"
                                                            }
                                                            button {
                                                                r#type: "button",
                                                                class: "kit-memory__btn",
                                                                "aria-label": "{pause_aria}",
                                                                "aria-pressed": if paused { "true" } else { "false" },
                                                                onclick: move |_| pause_fn(pause_id.clone()),
                                                                "{pause_label}"
                                                            }
                                                            button {
                                                                r#type: "button",
                                                                class: "kit-memory__btn is-danger",
                                                                "aria-label": "{forget_label}",
                                                                onclick: move |_| forget_fn(forget_id.clone()),
                                                                "Forget"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if compose_state.is_some() {
                div { class: "kit-memory__editor kit-memory__new",
                    textarea {
                        class: "kit-memory__input",
                        "aria-label": "New memory text",
                        value: "{compose_value}",
                        oninput: move |event: FormEvent| {
                            let group = composing().map(|(_, g)| g).unwrap_or_default();
                            composing.set(Some((event.value(), group)));
                        },
                    }
                    select {
                        class: "kit-memory__select",
                        "aria-label": "Group for new memory",
                        onchange: move |event: FormEvent| {
                            let text = composing().map(|(t, _)| t).unwrap_or_default();
                            composing.set(Some((text, event.value())));
                        },
                        for choice in group_choices.clone() {
                            {
                                let is_selected = choice == compose_group_value;
                                rsx! {
                                    option { key: "{choice}", value: "{choice}", selected: is_selected, "{choice}" }
                                }
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "kit-memory__btn",
                        onclick: move |_| {
                            let (text, group) = composing().unwrap_or_default();
                            save_new(text, group);
                        },
                        "Save"
                    }
                    button {
                        r#type: "button",
                        class: "kit-memory__btn",
                        onclick: move |_| composing.set(None),
                        "Cancel"
                    }
                }
            } else {
                button {
                    r#type: "button",
                    class: "kit-memory__btn kit-memory__new-btn",
                    onclick: {
                        let default_group = new_group_default.clone();
                        move |_| composing.set(Some((String::new(), default_group.clone())))
                    },
                    svg { view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", "aria-hidden": "true",
                        path { d: "M8 3.2v9.6M3.2 8h9.6" }
                    }
                    "New memory"
                }
            }
            footer { class: "kit-memory__foot",
                p { class: "kit-memory__summary", "{summary}" }
                if paused_total > 0 {
                    span { class: "kit-memory__sr", "{paused_total} paused" }
                }
                if undo_visible {
                    button {
                        r#type: "button",
                        class: "kit-memory__btn",
                        onclick: restore,
                        {undo_text.clone().unwrap_or_default()}
                    }
                } else if let Some(action) = footer_action {
                    button {
                        r#type: "button",
                        class: "kit-memory__btn",
                        onclick: move |_| action.on_action.call(()),
                        "{action.action_label}"
                    }
                }
            }
            div { class: "kit-memory__sr", role: "status", "aria-live": "polite", "{status_text}" }
        }
    }
}

fn compose_text(state: &Option<(String, String)>) -> String {
    state.as_ref().map(|(t, _)| t.clone()).unwrap_or_default()
}

fn compose_group(state: &Option<(String, String)>, fallback: &str) -> String {
    state
        .as_ref()
        .map(|(_, g)| g.clone())
        .filter(|g| !g.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}
