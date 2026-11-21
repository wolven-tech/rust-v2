//! Multi-select lookup: a searchable, grouped combobox whose picks become removable chips.
//!
//! Design choice: the create row is always last; a typed text that equals an existing label or
//! value (ignoring case) offers no create row. Options are re-read from the props on each event,
//! so a changed `option_list` applies to the next interaction.
//!
//! Browser checks: focus stays in the input while a row or chip is clicked; the Backspace
//! highlight then delete; the list matches the field width; chips wrap on a phone.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/multi-select-lookup.css");

#[derive(Clone, Debug, PartialEq)]
pub enum OptionImage {
    /// An image URL.
    Url(String),
    /// The first letter of the label, on a tile.
    Initials,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LookupOption {
    pub option_value: String,
    pub option_label: String,
    pub option_hint: Option<String>,
    pub option_image: Option<OptionImage>,
    /// A CSS colour, for example `var(--kit-ok)`.
    pub option_swatch: Option<String>,
    pub option_mark: Option<Element>,
    pub option_group: Option<String>,
    pub match_terms: Vec<String>,
}

impl LookupOption {
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            option_value: value.to_string(),
            option_label: label.to_string(),
            option_hint: None,
            option_image: None,
            option_swatch: None,
            option_mark: None,
            option_group: None,
            match_terms: Vec::new(),
        }
    }
}

/// Splits `label` around the first case-insensitive match of `query`: (before, matched, after).
/// The match is empty when there is no query or no hit.
pub fn highlight_parts(label: &str, query: &str) -> (String, String, String) {
    let chars: Vec<char> = label.chars().collect();
    let needle: Vec<char> = query
        .trim()
        .chars()
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if needle.is_empty() || needle.len() > chars.len() {
        return (label.to_string(), String::new(), String::new());
    }
    for start in 0..=chars.len() - needle.len() {
        let end = start + needle.len();
        if chars[start..end]
            .iter()
            .zip(&needle)
            .all(|(a, b)| a.to_ascii_lowercase() == *b)
        {
            return (
                chars[..start].iter().collect(),
                chars[start..end].iter().collect(),
                chars[end..].iter().collect(),
            );
        }
    }
    (label.to_string(), String::new(), String::new())
}

/// Options whose label or a match term contains `query`, in list order. An empty query matches all.
pub fn matching_indices(options: &[LookupOption], query: &str) -> Vec<usize> {
    options
        .iter()
        .enumerate()
        .filter(|(_, option)| {
            query.trim().is_empty()
                || std::iter::once(&option.option_label)
                    .chain(option.match_terms.iter())
                    .any(|term| !highlight_parts(term, query).1.is_empty())
        })
        .map(|(index, _)| index)
        .collect()
}

/// Groups in order of first appearance; options keep the order they were given in.
pub fn group_order(
    options: &[LookupOption],
    indices: &[usize],
) -> Vec<(Option<String>, Vec<usize>)> {
    let mut groups: Vec<(Option<String>, Vec<usize>)> = Vec::new();
    for &index in indices {
        let key = options[index].option_group.clone();
        match groups.iter_mut().find(|(group, _)| *group == key) {
            Some((_, members)) => members.push(index),
            None => groups.push((key, vec![index])),
        }
    }
    groups
}

/// True when the typed text matches no label or value and the component may create one.
pub fn create_offered(options: &[LookupOption], query: &str, allow_create: bool) -> bool {
    let text = query.trim();
    allow_create
        && !text.is_empty()
        && !options.iter().any(|option| {
            option.option_label.eq_ignore_ascii_case(text)
                || option.option_value.eq_ignore_ascii_case(text)
        })
}

/// Keyboard order of the popover: option rows in group order, then the create row as `None`.
pub fn visible_rows(
    options: &[LookupOption],
    query: &str,
    allow_create: bool,
) -> Vec<Option<usize>> {
    let mut rows: Vec<Option<usize>> = group_order(options, &matching_indices(options, query))
        .into_iter()
        .flat_map(|(_, members)| members)
        .map(Some)
        .collect();
    if create_offered(options, query, allow_create) {
        rows.push(None);
    }
    rows
}

/// Picks or unpicks `value`. Single mode replaces the selection.
pub fn toggle_pick(selected: &[String], value: &str, allow_many: bool) -> Vec<String> {
    if selected.iter().any(|picked| picked == value) {
        return selected
            .iter()
            .filter(|picked| *picked != value)
            .cloned()
            .collect();
    }
    if allow_many {
        let mut next = selected.to_vec();
        next.push(value.to_string());
        next
    } else {
        vec![value.to_string()]
    }
}

/// Visible chips and the count hidden behind "+N" for `total` picks.
pub fn overflow(total: usize, max_chips: Option<usize>) -> (usize, usize) {
    let visible = max_chips.map_or(total, |max| max.min(total));
    (visible, total - visible)
}

/// Next active row, clamped to the list. A `None` current starts at the first row (down) or the last (up).
pub fn step_active(current: Option<usize>, len: usize, dir: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match current {
        None if dir > 0 => 0,
        None => len - 1,
        Some(index) => (index as isize + dir).clamp(0, len as isize - 1) as usize,
    })
}

fn build_all(list: &[LookupOption], created: &[LookupOption]) -> Vec<LookupOption> {
    list.iter().chain(created.iter()).cloned().collect()
}

fn resolve_values(values: &[String], all: &[LookupOption]) -> Vec<LookupOption> {
    values
        .iter()
        .map(|value| {
            all.iter()
                .find(|option| &option.option_value == value)
                .cloned()
                .unwrap_or_else(|| LookupOption::new(value, value))
        })
        .collect()
}

fn commit(
    mut internal: Signal<Vec<String>>,
    controlled: bool,
    on_change: EventHandler<(Vec<String>, Vec<LookupOption>)>,
    next: Vec<String>,
    all: &[LookupOption],
) {
    let resolved = resolve_values(&next, all);
    if !controlled {
        internal.set(next.clone());
    }
    on_change.call((next, resolved));
}

/// The value a pick produces, and the option it created when the create row was used.
fn pick_next(
    all: &[LookupOption],
    row: Option<usize>,
    query: &str,
    picked: &[String],
    allow_many: bool,
    make: Option<Callback<String, LookupOption>>,
) -> (Vec<String>, Option<LookupOption>) {
    match row {
        Some(index) => (
            toggle_pick(picked, &all[index].option_value, allow_many),
            None,
        ),
        None => {
            let text = query.trim().to_string();
            let created = match make {
                Some(build) => build.call(text),
                None => LookupOption::new(&text, &text),
            };
            (
                toggle_pick(picked, &created.option_value, allow_many),
                Some(created),
            )
        }
    }
}

fn initial_of(label: &str) -> String {
    label
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default()
}

#[component]
pub fn MultiSelectLookup(
    #[props(default)] option_list: Vec<LookupOption>,
    #[props(default = true)] allow_many: bool,
    /// Controlled selection. When set, the prop decides the chips and picks report through `on_selection_change`.
    #[props(default)]
    selected_values: Option<Vec<String>>,
    #[props(default)] initial_selection: Vec<String>,
    #[props(default)] on_selection_change: EventHandler<(Vec<String>, Vec<LookupOption>)>,
    #[props(default = "Choose…".to_string(), into)] empty_hint: String,
    #[props(default)] allow_create: bool,
    #[props(default)] build_new_option: Option<Callback<String, LookupOption>>,
    #[props(default)] create_row_text: Option<Callback<String, String>>,
    #[props(default)] max_chip_count: Option<usize>,
    #[props(default, into)] field_caption: Option<String>,
    #[props(default)] start_open: bool,
    #[props(default = "No matches".to_string(), into)] no_match_text: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-msl-{}", next_id()));
    let controlled = selected_values.is_some();
    let internal = use_signal(|| initial_selection.clone());
    let mut created = use_signal(Vec::<LookupOption>::new);
    let mut query = use_signal(String::new);
    let mut open = use_signal(|| start_open);
    let mut active = use_signal(|| None::<usize>);
    let mut chip_focus = use_signal(|| None::<usize>);

    let all = build_all(&option_list, &created());
    let picked: Vec<String> = selected_values.clone().unwrap_or_else(|| internal());
    let picked_options = resolve_values(&picked, &all);
    let rows = visible_rows(&all, &query(), allow_create);
    let groups = group_order(&all, &matching_indices(&all, &query()));
    let offer_create = create_offered(&all, &query(), allow_create);
    let (shown_chips, hidden_chips) = overflow(picked.len(), max_chip_count);
    let is_open = open();
    let active_row = active().filter(|index| *index < rows.len());
    let active_id = active_row.map(|index| format!("{id}-row-{index}"));
    let create_text = query().trim().to_string();
    let create_label = match create_row_text {
        Some(text) => text.call(create_text.clone()),
        None => format!("Create “{create_text}”"),
    };
    let caption = field_caption
        .clone()
        .unwrap_or_else(|| "Lookup".to_string());
    let create_index = rows.len().saturating_sub(1);
    let picked_count = picked.len();

    let list_keys = option_list.clone();
    let picked_keys = picked.clone();
    let list_rows = option_list.clone();
    let picked_rows = picked.clone();
    let list_chips = option_list.clone();
    let picked_chips = picked.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: class_list("kit-msl", &root_class),
            "data-open": is_open.then_some("true"),
            "data-mode": if allow_many { "many" } else { "one" },
            if field_caption.is_some() {
                label { class: "kit-msl__caption", id: "{id}-caption", "{caption}" }
            }
            div { class: "kit-msl__field",
                if allow_many {
                    for (index, value) in picked_options.iter().take(shown_chips).enumerate() {
                        span {
                            key: "{value.option_value}",
                            class: "kit-msl__chip",
                            "data-highlight": (chip_focus() == Some(index)).then_some("true"),
                            "{value.option_label}"
                            button {
                                r#type: "button",
                                class: "kit-msl__chip-remove",
                                tabindex: "-1",
                                "aria-label": "Remove {value.option_label}",
                                onmousedown: move |event| event.prevent_default(),
                                onclick: {
                                    let list = list_chips.clone();
                                    let picked = picked_chips.clone();
                                    let value = value.option_value.clone();
                                    move |_| {
                                        let all = build_all(&list, &created());
                                        let next: Vec<String> = picked.iter().filter(|p| **p != value).cloned().collect();
                                        commit(internal, controlled, on_selection_change, next, &all);
                                    }
                                },
                                "×"
                            }
                        }
                    }
                    if hidden_chips > 0 {
                        span { class: "kit-msl__chip kit-msl__chip--more", "+{hidden_chips}" }
                    }
                } else if let Some(first) = picked_options.first() {
                    span { class: "kit-msl__single", "{first.option_label}" }
                }
                input {
                    id: "{id}-input",
                    class: "kit-msl__input",
                    role: "combobox",
                    r#type: "text",
                    autocomplete: "off",
                    "aria-label": "{caption}",
                    "aria-expanded": "{is_open}",
                    "aria-controls": "{id}-list",
                    "aria-autocomplete": "list",
                    "aria-activedescendant": active_id,
                    placeholder: if picked.is_empty() { empty_hint.clone() } else { String::new() },
                    value: "{query}",
                    oninput: move |event| {
                        query.set(event.value());
                        open.set(true);
                        active.set(None);
                    },
                    onfocus: move |_| open.set(true),
                    onkeydown: move |event: KeyboardEvent| {
                        let all = build_all(&list_keys, &created());
                        let rows = visible_rows(&all, &query(), allow_create);
                        let text_empty = query().is_empty();
                        match event.key() {
                            Key::ArrowDown | Key::ArrowUp => {
                                event.prevent_default();
                                open.set(true);
                                let dir = if event.key() == Key::ArrowDown { 1 } else { -1 };
                                active.set(step_active(active(), rows.len(), dir));
                            }
                            Key::Enter => {
                                event.prevent_default();
                                if let Some(row) = active().and_then(|index| rows.get(index).copied()) {
                                    let (next, new_option) = pick_next(&all, row, &query(), &picked_keys, allow_many, build_new_option);
                                    if let Some(option) = new_option {
                                        created.write().push(option);
                                    }
                                    query.set(String::new());
                                    let all_after = build_all(&list_keys, &created());
                                    commit(internal, controlled, on_selection_change, next, &all_after);
                                    if !allow_many {
                                        open.set(false);
                                    }
                                }
                            }
                            Key::Backspace if text_empty && allow_many && !picked_keys.is_empty() => {
                                let last = picked_keys.len() - 1;
                                if chip_focus() == Some(last) {
                                    chip_focus.set(None);
                                    let next: Vec<String> = picked_keys[..last].to_vec();
                                    commit(internal, controlled, on_selection_change, next, &all);
                                } else {
                                    chip_focus.set(Some(last));
                                }
                            }
                            Key::Escape => {
                                if open() {
                                    open.set(false);
                                } else {
                                    query.set(String::new());
                                }
                            }
                            Key::Tab => open.set(false),
                            _ => chip_focus.set(None),
                        }
                    },
                }
            }
            if is_open {
                div {
                    id: "{id}-list",
                    class: "kit-msl__list",
                    role: "listbox",
                    "aria-multiselectable": allow_many.then_some("true"),
                    "aria-label": "{caption}",
                    if rows.is_empty() {
                        div { class: "kit-msl__empty", "{no_match_text}" }
                    }
                    for (group, members) in groups.iter() {
                        if let Some(name) = group.clone() {
                            div { key: "{name}", class: "kit-msl__group", role: "presentation", "{name}" }
                        }
                        for index in members.iter().copied() {
                            {
                                let option = all[index].clone();
                                let row_index = rows.iter().position(|row| *row == Some(index)).unwrap_or(0);
                                let is_picked = picked.contains(&option.option_value);
                                let (pre, hit, post) = highlight_parts(&option.option_label, &query());
                                let avatar: Element = match option.option_image.clone() {
                                    Some(OptionImage::Url(url)) => rsx! { img { class: "kit-msl__img", src: "{url}", alt: "" } },
                                    Some(OptionImage::Initials) => {
                                        let initial = initial_of(&option.option_label);
                                        rsx! { span { class: "kit-msl__initials", "{initial}" } }
                                    }
                                    None => match option.option_swatch.clone() {
                                        Some(swatch) => rsx! { span { class: "kit-msl__swatch", style: "background: {swatch}" } },
                                        None => rsx! {},
                                    },
                                };
                                let list = list_rows.clone();
                                let picked_row = picked_rows.clone();
                                rsx! {
                                    div {
                                        key: "{option.option_value}",
                                        id: "{id}-row-{row_index}",
                                        class: "kit-msl__row",
                                        role: "option",
                                        "aria-selected": "{is_picked}",
                                        "data-active": (active_row == Some(row_index)).then_some("true"),
                                        onmousedown: move |event| {
                                            event.prevent_default();
                                            let all = build_all(&list, &created());
                                            let (next, _) = pick_next(&all, Some(index), &query(), &picked_row, allow_many, None);
                                            commit(internal, controlled, on_selection_change, next, &all);
                                            query.set(String::new());
                                            if !allow_many {
                                                open.set(false);
                                            }
                                        },
                                        onmouseenter: move |_| active.set(Some(row_index)),
                                        span { class: "kit-msl__avatar", "aria-hidden": "true", {avatar} }
                                        span { class: "kit-msl__text",
                                            span { class: "kit-msl__label", "{pre}"
                                                em { class: "kit-msl__hit", "{hit}" }
                                                "{post}"
                                            }
                                            if let Some(hint) = option.option_hint.clone() {
                                                span { class: "kit-msl__hint", "{hint}" }
                                            }
                                        }
                                        if is_picked {
                                            span { class: "kit-msl__check", "aria-hidden": "true", "✓" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if offer_create {
                        div {
                            class: "kit-msl__row kit-msl__row--create",
                            id: "{id}-row-{create_index}",
                            role: "option",
                            "aria-selected": "false",
                            "data-active": (active_row == Some(rows.len() - 1)).then_some("true"),
                            onmousedown: {
                                let list = list_rows.clone();
                                let picked_row = picked_rows.clone();
                                move |event| {
                                    event.prevent_default();
                                    let (next, new_option) = pick_next(&build_all(&list, &created()), None, &query(), &picked_row, allow_many, build_new_option);
                                    if let Some(option) = new_option {
                                        created.write().push(option);
                                        query.set(String::new());
                                    }
                                    let all_after = build_all(&list, &created());
                                    commit(internal, controlled, on_selection_change, next, &all_after);
                                }
                            },
                            "{create_label}"
                        }
                    }
                    div { class: "kit-msl__foot", role: "status", "aria-live": "polite", "{picked_count} selected" }
                }
            }
        }
    }
}
