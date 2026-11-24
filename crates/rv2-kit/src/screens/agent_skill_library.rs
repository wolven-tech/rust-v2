//! Agent skill library: the reusable skills an agent can draw on. A person turns
//! skills on or off, installs them from a catalogue, writes their own, and reviews
//! what each one does, when it applies and what it needs.
//!
//! Design choice: install shows progress for at least `MIN_INSTALL_MS`, so an
//! instant install is still visible. `on_install` is a plain callback; the host
//! sets `is_installed` when it is done, and this module treats the install as
//! complete once both the minimum time and the host's flag have passed.
//! Design choice: a skill created here is kept locally and opens in the panel. The
//! host should add it to `skill_records` when it saves the skill.
//! Design choice: the New skill dialog is an inline panel with four entry points
//! (describe, write, upload, browse). Ctrl or Cmd+Enter creates the skill.
//! Design choice: describing drafts locally: the first sentence names and describes
//! the skill, later sentences are its steps. The host can replace the draft by
//! saving its own. Upload reads a Markdown file in the browser and reads `# ` as the
//! name and list items as steps.
//! Design choice: the fourteen-day chart is bars sized by `daily_run_counts`.
//! Browser checks: a switch calls `on_toggle` with the new state; a built-in skill
//! has no Remove; Install shows progress and then the card is under Installed; a
//! category chip narrows both sections; a created skill appears under Own and opens
//! in the panel; "/" focuses search and Escape clears it; one column and no
//! horizontal scroll at 320 px.

use dioxus::prelude::*;
use rv2_affect::switch::Switch;

use crate::internal::{KIT_CSS, focus_after_frame, next_id, sleep};

const CSS: Asset = asset!("/assets/agent-skill-library.css");

/// The shortest time an install shows progress.
pub const MIN_INSTALL_MS: u32 = 700;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillSource {
    BuiltIn,
    Own,
    Community,
}

impl SkillSource {
    fn label(self) -> &'static str {
        match self {
            Self::BuiltIn => "Built-in",
            Self::Own => "Own",
            Self::Community => "Community",
        }
    }

    /// Own and community skills can be removed; built-in skills can only be switched off.
    pub fn removable(self) -> bool {
        !matches!(self, Self::BuiltIn)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Requirement {
    pub entry_label: String,
    pub permissions_link: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkillUsage {
    pub task_name: String,
    pub occurred_at: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkillRecord {
    pub id: String,
    pub entry_name: String,
    pub entry_description: String,
    pub category_name: String,
    pub source_kind: SkillSource,
    pub author_name: Option<String>,
    pub version_label: Option<String>,
    pub is_installed: bool,
    pub is_enabled: bool,
    pub weekly_use_count: Option<u32>,
    pub install_count: Option<u32>,
    pub trigger_phrases: Vec<String>,
    pub step_texts: Vec<String>,
    pub requirement_list: Vec<Requirement>,
    pub usage_log: Vec<SkillUsage>,
    /// Fourteen values, oldest first.
    pub daily_run_counts: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewSkill {
    pub entry_name: String,
    pub entry_description: String,
    pub category_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryPoint {
    Describe,
    Write,
    Upload,
    Browse,
}

impl EntryPoint {
    fn label(self) -> &'static str {
        match self {
            Self::Describe => "Describe it and let the agent draft it",
            Self::Write => "Write it by hand",
            Self::Upload => "Upload a definition file",
            Self::Browse => "Browse the catalogue",
        }
    }
}

/// Whether a skill passes the category filter; `None` keeps every skill.
pub fn in_category(skill: &SkillRecord, category: Option<&str>) -> bool {
    category.is_none_or(|c| skill.category_name == c)
}

/// Whether a skill matches the search text in its name or description.
pub fn matches_search(skill: &SkillRecord, query: &str) -> bool {
    let needle = query.trim().to_lowercase();
    needle.is_empty()
        || skill.entry_name.to_lowercase().contains(&needle)
        || skill.entry_description.to_lowercase().contains(&needle)
}

/// Skills in the Installed section, then in Discover, each in list order.
pub fn split_sections(skills: &[SkillRecord]) -> (Vec<SkillRecord>, Vec<SkillRecord>) {
    let installed = skills.iter().filter(|s| s.is_installed).cloned().collect();
    let discover = skills.iter().filter(|s| !s.is_installed).cloned().collect();
    (installed, discover)
}

/// The accessible name of a skill card: description, source, weekly use and off state.
pub fn card_name(skill: &SkillRecord) -> String {
    let uses = skill
        .weekly_use_count
        .map_or_else(String::new, |n| format!(", used {n} times this week"));
    let off = if skill.is_enabled { "" } else { ", off" };
    format!(
        "{}, {}{}{}",
        skill.entry_name,
        skill.source_kind.label(),
        uses,
        off
    )
}

/// A skill drafted from text: a name, a description and its steps.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DraftSkill {
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
}

/// Drafts a skill from a plain description: the first sentence describes it, and
/// each later sentence becomes a step.
pub fn draft_from_description(text: &str) -> DraftSkill {
    let sentences: Vec<String> = text
        .split(['.', '\n', ';'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let Some(first) = sentences.first() else {
        return DraftSkill::default();
    };
    let steps = if sentences.len() > 1 {
        sentences[1..].to_vec()
    } else {
        vec![first.clone()]
    };
    DraftSkill {
        name: first
            .split_whitespace()
            .take(6)
            .collect::<Vec<_>>()
            .join(" "),
        description: first.clone(),
        steps,
    }
}

/// Reads a Markdown skill definition: the first `# ` heading names it, the first
/// other line describes it, and list items are steps.
pub fn parse_definition(text: &str) -> DraftSkill {
    let mut draft = DraftSkill::default();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let numbered = line
            .split_once(". ")
            .filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
            .map(|(_, rest)| rest);
        if let Some(title) = line.strip_prefix("# ") {
            if draft.name.is_empty() {
                draft.name = title.trim().to_string();
            }
        } else if let Some(step) = line
            .strip_prefix("- ")
            .or_else(|| line.strip_prefix("* "))
            .or(numbered)
        {
            draft.steps.push(step.trim().to_string());
        } else if draft.description.is_empty() && !line.starts_with('#') {
            draft.description = line.to_string();
        }
    }
    if draft.name.is_empty() {
        draft.name = draft.description.clone();
    }
    draft
}

/// DOM id of the chip for `current`: All for `None`, else its category's chip.
fn chip_dom(uid: impl std::fmt::Display, categories: &[String], current: Option<&str>) -> String {
    match current.and_then(|c| categories.iter().position(|x| x == c)) {
        Some(index) => format!("kit-asl-{uid}-chip-{index}"),
        None => format!("kit-asl-{uid}-chip-all"),
    }
}

/// Moves along the chips, All first, then each category, wrapping at both ends.
/// `None` is All.
pub fn cycle_chip(categories: &[String], current: Option<&str>, delta: isize) -> Option<String> {
    let count = categories.len() as isize + 1;
    let at = match current {
        None => 0,
        Some(name) => categories
            .iter()
            .position(|c| c == name)
            .map_or(0, |i| i as isize + 1),
    };
    match (at + delta).rem_euclid(count) {
        0 => None,
        next => Some(categories[next as usize - 1].clone()),
    }
}

/// Moves to the next or previous category chip, wrapping at both ends.
pub fn cycle_category(
    categories: &[String],
    current: Option<&str>,
    delta: isize,
) -> Option<String> {
    if categories.is_empty() {
        return None;
    }
    let last = categories.len() as isize - 1;
    let next = match current.and_then(|c| categories.iter().position(|x| x == c)) {
        Some(index) => (index as isize + delta).clamp(0, last),
        None if delta >= 0 => 0,
        None => last,
    };
    Some(categories[next as usize].clone())
}

/// Installed and Discover sections with a panel for the open skill and a New
/// skill dialog.
#[component]
pub fn AgentSkillLibrary(
    #[props(default)] skill_records: Vec<SkillRecord>,
    #[props(default)] category_names: Vec<String>,
    #[props(default)] initial_skill_id: Option<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_install: Option<EventHandler<String>>,
    #[props(default)] on_toggle: Option<EventHandler<(String, bool)>>,
    #[props(default)] on_remove: Option<EventHandler<String>>,
    #[props(default)] on_create: Option<EventHandler<NewSkill>>,
    #[props(default)] on_select: Option<EventHandler<String>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
) -> Element {
    let uid = use_hook(next_id);
    let mut category = use_signal(|| None::<String>);
    let mut query = use_signal(String::new);
    let mut open_id = use_signal(move || initial_skill_id.clone());
    let mut installing = use_signal(Vec::<String>::new);
    let mut installed_now = use_signal(Vec::<String>::new);
    let mut toggled = use_signal(Vec::<(String, bool)>::new);
    let mut created = use_signal(Vec::<SkillRecord>::new);
    let mut removing = use_signal(|| None::<String>);
    let mut dialog_open = use_signal(|| false);
    let mut entry_point = use_signal(|| EntryPoint::Describe);
    let mut draft_name = use_signal(String::new);
    let mut name_error = use_signal(|| false);
    let mut announce = use_signal(String::new);
    let mut describe = use_signal(String::new);
    let mut draft = use_signal(DraftSkill::default);
    let mut upload_note = use_signal(String::new);

    let mut all: Vec<SkillRecord> = skill_records.clone();
    all.extend(created());
    let installed_local = installed_now();
    let toggled_local = toggled();
    let mut skills: Vec<SkillRecord> = all
        .into_iter()
        .map(|mut s| {
            if installed_local.contains(&s.id) {
                s.is_installed = true;
            }
            if let Some((_, enabled)) = toggled_local.iter().find(|(id, _)| *id == s.id) {
                s.is_enabled = *enabled;
            }
            s
        })
        .collect();
    skills.retain(|s| in_category(s, category().as_deref()) && matches_search(s, &query()));
    let (installed, discover) = split_sections(&skills);
    let order: Vec<String> = installed
        .iter()
        .chain(discover.iter())
        .map(|s| s.id.clone())
        .collect();
    let open_skill = open_id().and_then(|id| skills.iter().find(|s| s.id == id).cloned());
    let installed_count = installed.len();
    let count_text = format!("{installed_count} installed");
    let categories = category_names.clone();
    let search_id = format!("kit-asl-{uid}-search");
    let chip_all_id = format!("kit-asl-{uid}-chip-all");
    let installing_now = installing();
    let removing_now = removing();
    let dialog_now = dialog_open();
    let point_now = entry_point();
    let describe_now = describe();
    let staged_now = draft();
    let upload_note_now = upload_note();
    let file_id = format!("kit-asl-{uid}-file");
    let file_id_read = file_id.clone();
    let draft_now = draft_name();
    let name_invalid = name_error();
    let name_id = format!("kit-asl-{uid}-name");
    let name_error_id = format!("kit-asl-{uid}-name-error");
    let name_error_text = if name_invalid {
        "Enter a name for the skill."
    } else {
        ""
    };
    let key_order = order.clone();
    let focus_search = search_id.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-asl",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            onkeydown: move |event: KeyboardEvent| {
                if dialog_open() {
                    return;
                }
                match event.key() {
                    Key::Escape => query.set(String::new()),
                    Key::Character(c) if c == "/" => {
                        event.prevent_default();
                        focus_after_frame(&focus_search);
                    }
                    Key::Character(c) if c == "j" || c == "k" => {
                        let delta: isize = if c == "j" { 1 } else { -1 };
                        let at = open_id().as_deref().and_then(|id| key_order.iter().position(|o| o == id));
                        let next = match at {
                            Some(i) => (i as isize + delta).clamp(0, key_order.len() as isize - 1) as usize,
                            None => 0,
                        };
                        if let Some(id) = key_order.get(next) {
                            open_id.set(Some(id.clone()));
                            if let Some(handler) = on_select {
                                handler.call(id.clone());
                            }
                        }
                    }
                    _ => {}
                }
            },
            header { class: "kit-asl__head",
                div { class: "kit-asl__heading",
                    h2 { class: "kit-asl__title", "Skills" }
                    p { class: "kit-asl__meta", "{count_text}" }
                }
                div { class: "kit-asl__head-actions",
                    input {
                        id: "{search_id}",
                        class: "kit-asl__search",
                        r#type: "search",
                        placeholder: "Search skills",
                        "aria-label": "Search skills",
                        value: "{query}",
                        oninput: move |event| query.set(event.value()),
                    }
                    button {
                        r#type: "button",
                        class: "kit-asl__primary",
                        "aria-expanded": dialog_now.to_string(),
                        onclick: move |_| dialog_open.toggle(),
                        "New skill"
                    }
                }
            }

            div { class: "kit-asl__filters",
                div { class: "kit-asl__chips", role: "radiogroup", "aria-label": "Category",
                    button {
                        id: "{chip_all_id}",
                        r#type: "button",
                        role: "radio",
                        class: "kit-asl__chip",
                        "aria-checked": (category().is_none()).to_string(),
                        tabindex: if category().is_none() { "0" } else { "-1" },
                        onclick: move |_| category.set(None),
                        onkeydown: {
                            let chips = categories.clone();
                            move |event: KeyboardEvent| {
                                let delta = match event.key() {
                                    Key::ArrowRight => 1,
                                    Key::ArrowLeft => -1,
                                    _ => return,
                                };
                                let next = cycle_chip(&chips, None, delta);
                                category.set(next.clone());
                                focus_after_frame(&chip_dom(uid, &chips, next.as_deref()));
                            }
                        },
                        "All"
                    }
                    for (index, name) in categories.iter().cloned().enumerate() {
                        {
                            let chip_key = name.clone();
                            let chip_name = name.clone();
                            let chip_checked = (category().as_deref() == Some(name.as_str())).to_string();
                            let chip_tab = if category().as_deref() == Some(name.as_str()) { "0" } else { "-1" };
                            let chip_id = format!("kit-asl-{uid}-chip-{index}");
                            let chip_list = categories.clone();
                            let own_name = name.clone();
                            rsx! {
                                button {
                                    key: "{chip_key}",
                                    id: "{chip_id}",
                                    r#type: "button",
                                    role: "radio",
                                    class: "kit-asl__chip",
                                    "aria-checked": "{chip_checked}",
                                    tabindex: "{chip_tab}",
                                    onclick: move |_| category.set(Some(chip_name.clone())),
                                    onkeydown: move |event: KeyboardEvent| {
                                        let delta = match event.key() {
                                            Key::ArrowRight => 1,
                                            Key::ArrowLeft => -1,
                                            _ => return,
                                        };
                                        let next = cycle_chip(&chip_list, Some(own_name.as_str()), delta);
                                        category.set(next.clone());
                                        focus_after_frame(&chip_dom(uid, &chip_list, next.as_deref()));
                                    },
                                    "{chip_key}"
                                }
                            }
                        }
                    }
                }
            }

            if dialog_now {
                section { class: "kit-asl__dialog", "aria-label": "New skill",
                    div { class: "kit-asl__entries", role: "radiogroup", "aria-label": "How to start",
                        for point in [EntryPoint::Describe, EntryPoint::Write, EntryPoint::Upload, EntryPoint::Browse] {
                            {
                                let point_key = point.label();
                                let point_checked = (point_now == point).to_string();
                                rsx! {
                                    button {
                                        key: "{point_key}",
                                        r#type: "button",
                                        role: "radio",
                                        class: "kit-asl__entry",
                                        "aria-checked": "{point_checked}",
                                        onclick: move |_| entry_point.set(point),
                                        "{point_key}"
                                    }
                                }
                            }
                        }
                    }
                    if point_now == EntryPoint::Browse {
                        p { class: "kit-asl__meta", "The catalogue lists skills you can install from the Discover section." }
                    } else {
                        if point_now == EntryPoint::Describe {
                            form {
                                class: "kit-asl__describe",
                                "aria-label": "Describe the skill",
                                onsubmit: move |event: FormEvent| {
                                    event.prevent_default();
                                    let next = draft_from_description(&describe.peek());
                                    draft_name.set(next.name.clone());
                                    draft.set(next);
                                },
                                label { class: "kit-asl__field",
                                    span { "What should it do?" }
                                    textarea {
                                        class: "kit-asl__input",
                                        rows: "3",
                                        value: "{describe_now}",
                                        oninput: move |event| describe.set(event.value()),
                                    }
                                }
                                button { r#type: "submit", class: "kit-asl__ghost", "Draft it" }
                            }
                        }
                        if point_now == EntryPoint::Upload {
                            div { class: "kit-asl__upload",
                                label { class: "kit-asl__field",
                                    span { "Definition file (.md)" }
                                    input {
                                        id: "{file_id}",
                                        class: "kit-asl__input",
                                        r#type: "file",
                                        accept: ".md,.txt",
                                        onchange: move |_| {
                                            let id = file_id_read.clone();
                                            spawn(async move {
                                                let read = document::eval(&format!(
                                                    "const f = document.getElementById({id:?}).files[0]; return f ? [f.name, await f.text()] : null;"
                                                ))
                                                .await;
                                                let Ok(value) = read else {
                                                    return;
                                                };
                                                let Some(parts) = value.as_array() else {
                                                    return;
                                                };
                                                let file = parts.first().and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                                let text = parts.get(1).and_then(|v| v.as_str()).unwrap_or_default();
                                                let parsed = parse_definition(text);
                                                draft_name.set(parsed.name.clone());
                                                upload_note.set(format!("Loaded {file}: {} steps", parsed.steps.len()));
                                                draft.set(parsed);
                                            });
                                        },
                                    }
                                }
                                p { class: "kit-asl__meta", role: "status", "{upload_note_now}" }
                            }
                        }
                        if !staged_now.steps.is_empty() || !staged_now.description.is_empty() {
                            section { class: "kit-asl__draft", "aria-label": "Draft",
                                p { class: "kit-asl__meta", "{staged_now.description}" }
                                ol { class: "kit-asl__steps",
                                    for (index, step) in staged_now.steps.iter().cloned().enumerate() {
                                        li { key: "{index}", "{step}" }
                                    }
                                }
                            }
                        }
                        form {
                            class: "kit-asl__create",
                            "aria-label": "Skill name",
                            onsubmit: move |event: FormEvent| {
                                event.prevent_default();
                                let name = draft_name().trim().to_string();
                                if name.is_empty() {
                                    name_error.set(true);
                                    focus_after_frame(&format!("kit-asl-{uid}-name"));
                                    return;
                                }
                                let staged = draft.peek().clone();
                                let next = NewSkill {
                                    entry_name: name.clone(),
                                    entry_description: staged.description.clone(),
                                    category_name: category().unwrap_or_else(|| "General".to_string()),
                                };
                                let record = SkillRecord {
                                    id: format!("own-{name}"),
                                    entry_name: name.clone(),
                                    entry_description: staged.description.clone(),
                                    category_name: next.category_name.clone(),
                                    source_kind: SkillSource::Own,
                                    author_name: None,
                                    version_label: Some("draft".to_string()),
                                    is_installed: true,
                                    is_enabled: true,
                                    weekly_use_count: None,
                                    install_count: None,
                                    trigger_phrases: Vec::new(),
                                    step_texts: staged.steps.clone(),
                                    requirement_list: Vec::new(),
                                    usage_log: Vec::new(),
                                    daily_run_counts: vec![0; 14],
                                };
                                let open = record.id.clone();
                                created.with_mut(|list| list.push(record));
                                open_id.set(Some(open));
                                if let Some(handler) = on_create {
                                    handler.call(next);
                                }
                                announce.set(format!("Created {name}"));
                                draft_name.set(String::new());
                                draft.set(DraftSkill::default());
                                describe.set(String::new());
                                upload_note.set(String::new());
                                dialog_open.set(false);
                            },
                            label { class: "kit-asl__field",
                                span { "Name" }
                                input {
                                    id: "{name_id}",
                                    class: "kit-asl__input",
                                    r#type: "text",
                                    value: "{draft_now}",
                                    "aria-invalid": "{name_invalid}",
                                    "aria-describedby": if name_invalid { name_error_id.clone() } else { String::new() },
                                    oninput: move |event| {
                                        name_error.set(false);
                                        draft_name.set(event.value());
                                    },
                                }
                                span { id: "{name_error_id}", class: "kit-asl__error", role: "alert", "{name_error_text}" }
                            }
                            button { r#type: "submit", class: "kit-asl__primary", "Create skill" }
                        }
                    }
                }
            }

            if is_empty && !is_loading {
                div { class: "kit-asl__empty",
                    p { "No skills yet. Add one to give the agent a repeatable way of working." }
                    button {
                        r#type: "button",
                        class: "kit-asl__primary",
                        onclick: move |_| dialog_open.set(true),
                        "New skill"
                    }
                }
            } else if is_loading {
                div { class: "kit-asl__grid", "aria-hidden": "true",
                    for n in 0..3 {
                        div { key: "{n}", class: "kit-asl__ph" }
                    }
                }
            } else {
                div { class: "kit-asl__body",
                    div { class: "kit-asl__sections",
                        section { class: "kit-asl__section", "aria-label": "Installed",
                            h3 { class: "kit-asl__section-title", "Installed · {installed_count}" }
                            ul { class: "kit-asl__grid", role: "list",
                                for skill in installed.iter().cloned() {
                                    {
                                        let card_key = skill.id.clone();
                                        let card_label = card_name(&skill);
                                        let switch_label = format!("{} enabled", skill.entry_name);
                                        let state_word = if skill.is_enabled { "On" } else { "Off" };
                                        let enabled = skill.is_enabled;
                                        let open_pick = skill.id.clone();
                                        let toggle_id = skill.id.clone();
                                        let is_open = open_id().as_deref() == Some(skill.id.as_str());
                                        let description = skill.entry_description.clone();
                                        let source = match (skill.source_kind, skill.author_name.as_deref()) {
                                            (SkillSource::Community, Some(handle)) => handle.to_string(),
                                            (kind, _) => kind.label().to_string(),
                                        };
                                        let card_name_text = skill.entry_name.clone();
                                        rsx! {
                                            li { key: "{card_key}", class: "kit-asl__item", "data-enabled": "{enabled}",
                                                button {
                                                    r#type: "button",
                                                    class: "kit-asl__card",
                                                    "aria-label": "{card_label}",
                                                    "aria-current": is_open.then_some("true"),
                                                    onclick: move |_| {
                                                        open_id.set(Some(open_pick.clone()));
                                                        if let Some(handler) = on_select {
                                                            handler.call(open_pick.clone());
                                                        }
                                                    },
                                                    span { class: "kit-asl__card-name", "{card_name_text}" }
                                                    span { class: "kit-asl__meta", "{description}" }
                                                    span { class: "kit-asl__tag", "{source}" }
                                                }
                                                Switch {
                                                    class: "kit-asl__switch",
                                                    label: rsx! { "{state_word}" },
                                                    checked: enabled,
                                                    "aria-label": "{switch_label}",
                                                    onchange: move |event: FormEvent| {
                                                        let next = event.checked();
                                                        toggled.with_mut(|list| {
                                                            list.retain(|(id, _)| *id != toggle_id);
                                                            list.push((toggle_id.clone(), next));
                                                        });
                                                        if let Some(handler) = on_toggle {
                                                            handler.call((toggle_id.clone(), next));
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        section { class: "kit-asl__section", "aria-label": "Discover",
                            h3 { class: "kit-asl__section-title", "Discover · {discover.len()}" }
                            p { class: "kit-asl__meta", "Skills from the catalogue that are not installed yet." }
                            ul { class: "kit-asl__grid", role: "list",
                                for skill in discover.iter().cloned() {
                                    {
                                        let card_key = skill.id.clone();
                                        let card_label = card_name(&skill);
                                        let install_id = skill.id.clone();
                                        let install_name = skill.entry_name.clone();
                                        let installing_this = installing_now.contains(&skill.id);
                                        let install_label = if installing_this { "Installing" } else { "Install" };
                                        let install_aria = format!("Install {}", skill.entry_name);
                                        let uses = skill.install_count.map_or_else(String::new, |n| format!("{n} use it"));
                                        let description = skill.entry_description.clone();
                                        let card_name_text = skill.entry_name.clone();
                                        rsx! {
                                            li { key: "{card_key}", class: "kit-asl__item",
                                                div { class: "kit-asl__card kit-asl__card--discover", "aria-label": "{card_label}",
                                                    span { class: "kit-asl__card-name", "{card_name_text}" }
                                                    span { class: "kit-asl__meta", "{description}" }
                                                    span { class: "kit-asl__meta", "{uses}" }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "kit-asl__primary",
                                                    "aria-label": "{install_aria}",
                                                    disabled: installing_this,
                                                    onclick: move |_| {
                                                        let id = install_id.clone();
                                                        let name = install_name.clone();
                                                        installing.with_mut(|list| list.push(id.clone()));
                                                        announce.set(format!("Installing {name}"));
                                                        if let Some(handler) = on_install {
                                                            handler.call(id.clone());
                                                        }
                                                        spawn(async move {
                                                            sleep(MIN_INSTALL_MS).await;
                                                            installing.with_mut(|list| list.retain(|v| *v != id));
                                                            installed_now.with_mut(|list| list.push(id.clone()));
                                                            announce.set(format!("{name}: installed"));
                                                        });
                                                    },
                                                    "{install_label}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(skill) = open_skill.clone() {
                        {
                            let title = skill.entry_name.clone();
                            let source = skill.source_kind.label();
                            let removable = skill.source_kind.removable();
                            let remove_id = skill.id.clone();
                            let confirming = removing_now.as_deref() == Some(skill.id.as_str());
                            let triggers = skill.trigger_phrases.clone();
                            let steps = skill.step_texts.clone();
                            let requirements = skill.requirement_list.clone();
                            let bars = skill.daily_run_counts.clone();
                            let peak = bars.iter().copied().max().unwrap_or(1).max(1);
                            let week_runs: u32 = bars.iter().rev().take(7).sum();
                            let switch_text = if skill.is_enabled { "Turn off" } else { "Turn on" };
                            let version = skill.version_label.clone().unwrap_or_default();
                            let usage = skill.usage_log.clone();
                            let panel_label = format!("Skill: {title}");
                            let enabled = skill.is_enabled;
                            let toggle_id = skill.id.clone();
                            let remove_target = skill.id.clone();
                            rsx! {
                                aside { class: "kit-asl__panel", "aria-label": "{panel_label}",
                                    div { class: "kit-asl__panel-head",
                                        h3 { class: "kit-asl__panel-title", "{title}" }
                                        span { class: "kit-asl__tag", "{source}" }
                                        if !version.is_empty() {
                                            span { class: "kit-asl__meta", "v{version}" }
                                        }
                                    }
                                    Switch {
                                        class: "kit-asl__switch kit-asl__switch--panel",
                                        label: rsx! { "{switch_text}" },
                                        checked: enabled,
                                        "aria-label": "{panel_label} enabled",
                                        onchange: move |event: FormEvent| {
                                            let next = event.checked();
                                            toggled.with_mut(|list| {
                                                list.retain(|(id, _)| *id != toggle_id);
                                                list.push((toggle_id.clone(), next));
                                            });
                                            if let Some(handler) = on_toggle {
                                                handler.call((toggle_id.clone(), next));
                                            }
                                        },
                                    }
                                    if removable {
                                        if confirming {
                                            div { class: "kit-asl__confirm",
                                                span { "Remove this skill?" }
                                                button {
                                                    r#type: "button",
                                                    class: "kit-asl__danger",
                                                    onclick: move |_| {
                                                        removing.set(None);
                                                        if let Some(handler) = on_remove {
                                                            handler.call(remove_target.clone());
                                                        }
                                                    },
                                                    "Remove"
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "kit-asl__ghost",
                                                    onclick: move |_| removing.set(None),
                                                    "Keep"
                                                }
                                            }
                                        } else {
                                            button {
                                                r#type: "button",
                                                class: "kit-asl__ghost",
                                                onclick: {
                                                    let id = remove_id.clone();
                                                    move |_| removing.set(Some(id.clone()))
                                                },
                                                "Remove"
                                            }
                                        }
                                    }
                                    section { class: "kit-asl__block", "aria-label": "Last 14 days",
                                        h4 { class: "kit-asl__section-title", "Last 14 days" }
                                        div { class: "kit-asl__bars", role: "img", "aria-label": "Runs over the last 14 days",
                                            for (index, value) in bars.iter().enumerate() {
                                                {
                                                    let bar_key = format!("{index}");
                                                    let height = format!("{:.0}%", f64::from(*value) / f64::from(peak) * 100.0);
                                                    rsx! { span { key: "{bar_key}", class: "kit-asl__bar", style: "height: {height};" } }
                                                }
                                            }
                                        }
                                        p { class: "kit-asl__meta", "{week_runs} runs this week" }
                                    }
                                    section { class: "kit-asl__block", "aria-label": "When it applies",
                                        h4 { class: "kit-asl__section-title", "When it applies" }
                                        ul { class: "kit-asl__list",
                                            for (index, phrase) in triggers.iter().enumerate() {
                                                li { key: "{index}", "{phrase}" }
                                            }
                                        }
                                    }
                                    section { class: "kit-asl__block", "aria-label": "Steps",
                                        h4 { class: "kit-asl__section-title", "Steps" }
                                        ol { class: "kit-asl__list",
                                            for (index, step) in steps.iter().enumerate() {
                                                li { key: "{index}", "{step}" }
                                            }
                                        }
                                    }
                                    section { class: "kit-asl__block", "aria-label": "Requirements",
                                        h4 { class: "kit-asl__section-title", "Needs" }
                                        ul { class: "kit-asl__list",
                                            for (index, req) in requirements.iter().enumerate() {
                                                {
                                                    let req_key = format!("{index}");
                                                    let req_text = req.entry_label.clone();
                                                    let has_link = req.permissions_link;
                                                    rsx! {
                                                        li { key: "{req_key}",
                                                            span { "{req_text}" }
                                                            if has_link {
                                                                button {
                                                                    r#type: "button",
                                                                    class: "kit-asl__link",
                                                                    onclick: move |_| {
                                                                        if let Some(handler) = on_navigate {
                                                                            handler.call("Permissions".to_string());
                                                                        }
                                                                    },
                                                                    "Permissions"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    section { class: "kit-asl__block", "aria-label": "Recent use",
                                        h4 { class: "kit-asl__section-title", "Recent use" }
                                        if usage.is_empty() {
                                            p { class: "kit-asl__meta", "Not used yet." }
                                        }
                                        ul { class: "kit-asl__list",
                                            for (index, entry) in usage.iter().enumerate() {
                                                {
                                                    let use_key = format!("{index}");
                                                    let use_task = entry.task_name.clone();
                                                    rsx! { li { key: "{use_key}", "{use_task}" } }
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

            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
