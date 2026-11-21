//! Quick action palette: one search field that runs any command, with an assistant fallback.
//!
//! Design choice: when nothing matches, Enter sends the reserved `ASSISTANT_ID` with the typed
//! text as `typed_text`. The `suggested` group takes the Mod+1 to Mod+5 quick slots.
//!
//! Browser checks: Mod+digit runs at once; Escape with text clears it; focus on mount; the footer
//! confirmation after a run; no horizontal scroll at phone width.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/quick-action-palette.css");

/// The reserved id that `on_command` receives for the assistant fallback.
pub const ASSISTANT_ID: &str = "assistant:ask";
/// The group id whose first five commands get the Mod+1 to Mod+5 slots.
pub const SUGGESTED_GROUP: &str = "suggested";

#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub command_id: String,
    pub command_name: String,
    pub command_note: Option<String>,
    pub command_mark: Option<Element>,
    pub key_combo: Option<String>,
    pub match_terms: Vec<String>,
}

impl Command {
    pub fn new(command_id: &str, command_name: &str) -> Self {
        Self {
            command_id: command_id.to_string(),
            command_name: command_name.to_string(),
            command_note: None,
            command_mark: None,
            key_combo: None,
            match_terms: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandGroup {
    pub group_id: String,
    pub group_name: String,
    pub commands: Vec<Command>,
}

/// A row of the palette: (group index, command index within that group).
pub type Row = (usize, usize);

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack
        .to_lowercase()
        .contains(&needle.trim().to_lowercase())
}

/// Rows in display order, keeping only commands whose name or match term contains `query`.
/// Groups with no match are dropped.
pub fn filtered_rows(groups: &[CommandGroup], query: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    for (g, group) in groups.iter().enumerate() {
        for (c, command) in group.commands.iter().enumerate() {
            let hit = query.trim().is_empty()
                || contains_ci(&command.command_name, query)
                || command
                    .match_terms
                    .iter()
                    .any(|term| contains_ci(term, query));
            if hit {
                rows.push((g, c));
            }
        }
    }
    rows
}

/// The row to move to when Tab (`dir` 1) or Shift+Tab (`dir` -1) jumps a section. Stays put at the ends.
pub fn section_jump(rows: &[Row], current: usize, dir: isize) -> usize {
    let Some(&(group, _)) = rows.get(current) else {
        return current;
    };
    if dir > 0 {
        return rows
            .iter()
            .enumerate()
            .skip(current)
            .find(|(_, row)| row.0 != group)
            .map_or(current, |(index, _)| index);
    }
    let start = rows
        .iter()
        .position(|row| row.0 == group)
        .unwrap_or(current);
    if start < current {
        return start;
    }
    if start == 0 {
        return current;
    }
    let previous = rows[start - 1].0;
    rows.iter()
        .position(|row| row.0 == previous)
        .unwrap_or(current)
}

/// Next active row, clamped to the list. A `None` current starts at the first row (down) or the last (up).
fn step_index(current: Option<usize>, len: usize, dir: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match current {
        None if dir > 0 => 0,
        None => len - 1,
        Some(index) => (index as isize + dir).clamp(0, len as isize - 1) as usize,
    })
}

/// The command id in quick slot `slot` (1 to 5) of the suggested group.
pub fn quick_slot(groups: &[CommandGroup], slot: usize) -> Option<String> {
    let group = groups.iter().find(|g| g.group_id == SUGGESTED_GROUP)?;
    group
        .commands
        .get(slot.checked_sub(1)?)
        .map(|command| command.command_id.clone())
}

/// Digit keys 1 to 5 map to quick slots.
pub fn slot_for_key(key: &str) -> Option<usize> {
    match key {
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        "4" => Some(4),
        "5" => Some(5),
        _ => None,
    }
}

#[component]
pub fn QuickActionPalette(
    #[props(default)] command_groups: Vec<CommandGroup>,
    #[props(default = "Type a command…".to_string(), into)] empty_hint: String,
    /// Called with the command id and, for the assistant fallback, the typed text.
    #[props(default)]
    on_command: EventHandler<(String, Option<String>)>,
    #[props(default)] on_dismiss: EventHandler<()>,
    #[props(default)] focus_on_mount: bool,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-qap-{}", next_id()));
    let mut query = use_signal(String::new);
    let mut active = use_signal(|| Some(0_usize));
    let mut last_run = use_signal(|| None::<String>);

    let rows = filtered_rows(&command_groups, &query());
    let text = query().trim().to_string();
    let fallback = rows.is_empty() && !text.is_empty();
    let count_text = match rows.len() {
        0 if fallback => "Ask the assistant".to_string(),
        0 => "No matching commands".to_string(),
        1 => "1 command".to_string(),
        n => format!("{n} commands"),
    };
    let active_row = active().filter(|index| *index < rows.len());
    let groups_for_keys = command_groups.clone();

    let mut run = move |command_id: String, typed: Option<String>, name: String| {
        last_run.set(Some(name));
        on_command.call((command_id, typed));
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: class_list("kit-qap", &root_class),
            role: "dialog",
            "aria-label": "Quick actions",
            div { class: "kit-qap__search",
                span { class: "kit-qap__glyph", "aria-hidden": "true", "⌕" }
                input {
                    id: "{id}-input",
                    class: "kit-qap__input",
                    role: "combobox",
                    r#type: "text",
                    autocomplete: "off",
                    autofocus: focus_on_mount,
                    "aria-label": "Search commands",
                    "aria-expanded": "true",
                    "aria-controls": "{id}-list",
                    "aria-activedescendant": active_row.map(|index| format!("{id}-row-{index}")),
                    placeholder: "{empty_hint}",
                    value: "{query}",
                    oninput: move |event| {
                        query.set(event.value());
                        active.set(Some(0));
                    },
                    onkeydown: move |event: KeyboardEvent| {
                        let rows = filtered_rows(&groups_for_keys, &query());
                        let modified = event.modifiers().ctrl() || event.modifiers().meta();
                        if modified {
                            if let Key::Character(key) = event.key() {
                                if let Some(command_id) = slot_for_key(&key).and_then(|slot| quick_slot(&groups_for_keys, slot)) {
                                    event.prevent_default();
                                    let name = groups_for_keys
                                        .iter()
                                        .flat_map(|g| g.commands.iter())
                                        .find(|c| c.command_id == command_id)
                                        .map(|c| c.command_name.clone())
                                        .unwrap_or_default();
                                    run(command_id, None, name);
                                }
                            }
                            return;
                        }
                        match event.key() {
                            Key::ArrowDown | Key::ArrowUp => {
                                event.prevent_default();
                                let dir = if event.key() == Key::ArrowDown { 1 } else { -1 };
                                active.set(step_index(active(), rows.len(), dir));
                            }
                            Key::Tab => {
                                event.prevent_default();
                                let dir = if event.modifiers().shift() { -1 } else { 1 };
                                let current = active().unwrap_or(0);
                                active.set(Some(section_jump(&rows, current, dir)));
                            }
                            Key::Enter => {
                                event.prevent_default();
                                if rows.is_empty() && !query().trim().is_empty() {
                                    run(ASSISTANT_ID.to_string(), Some(query().trim().to_string()), "Ask the assistant".to_string());
                                } else if let Some(&(g, c)) = active().and_then(|i| rows.get(i)) {
                                    let command = &groups_for_keys[g].commands[c];
                                    run(command.command_id.clone(), None, command.command_name.clone());
                                }
                            }
                            Key::Escape => {
                                if query().is_empty() {
                                    on_dismiss.call(());
                                } else {
                                    query.set(String::new());
                                    active.set(Some(0));
                                }
                            }
                            _ => {}
                        }
                    },
                }
            }
            div { id: "{id}-list", class: "kit-qap__list", role: "listbox", "aria-label": "Commands",
                if fallback {
                    div {
                        id: "{id}-row-0",
                        class: "kit-qap__row",
                        role: "option",
                        "aria-selected": "true",
                        "data-active": "true",
                        onmousedown: move |event| {
                            event.prevent_default();
                            run(ASSISTANT_ID.to_string(), Some(text.clone()), "Ask the assistant".to_string());
                        },
                        span { class: "kit-qap__name", "Ask the assistant: {text}" }
                    }
                } else if rows.is_empty() {
                    div { class: "kit-qap__empty", "No matching commands" }
                }
                for (g, group) in command_groups.iter().enumerate() {
                    if rows.iter().any(|r| r.0 == g) {
                        div { key: "{group.group_id}", class: "kit-qap__group", role: "presentation",
                            span { class: "kit-qap__heading", id: "{id}-group-{g}", "{group.group_name}" }
                            for (c, command) in group.commands.iter().enumerate() {
                                if let Some(row_index) = rows.iter().position(|r| *r == (g, c)) {
                                    {
                                        let full_name = match &command.key_combo {
                                            Some(combo) => format!("{} {combo}", command.command_name),
                                            None => command.command_name.clone(),
                                        };
                                        rsx! {
                                    div {
                                        key: "{command.command_id}",
                                        id: "{id}-row-{row_index}",
                                        class: "kit-qap__row",
                                        role: "option",
                                        "aria-selected": if active_row == Some(row_index) { "true" } else { "false" },
                                        "aria-label": "{full_name}",
                                        "data-active": (active_row == Some(row_index)).then_some("true"),
                                        onmouseenter: move |_| active.set(Some(row_index)),
                                        onmousedown: {
                                            let command_id = command.command_id.clone();
                                            let name = command.command_name.clone();
                                            move |event| {
                                                event.prevent_default();
                                                run(command_id.clone(), None, name.clone());
                                            }
                                        },
                                        if let Some(mark) = command.command_mark.clone() {
                                            span { class: "kit-qap__mark", "aria-hidden": "true", {mark} }
                                        }
                                        span { class: "kit-qap__name", "{command.command_name}" }
                                        if let Some(note) = command.command_note.clone() {
                                            span { class: "kit-qap__note", "{note}" }
                                        }
                                        if let Some(combo) = command.key_combo.clone() {
                                            kbd { class: "kit-qap__key", "aria-hidden": "true", "{combo}" }
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
            div { class: "kit-qap__foot",
                span { class: "kit-qap__hints", "↑↓ move · Tab section · Enter run · Esc close" }
                span { class: "kit-qap__count", role: "status", "aria-live": "polite", "{count_text}" }
                span { class: "kit-qap__ran", role: "status", "aria-live": "polite",
                    if let Some(name) = last_run() {
                        "✓ {name}"
                    }
                }
            }
        }
    }
}
