//! Automation flow editor: a top-down flow of steps, with branch steps that split into named paths.
//!
//! Design choice: the flow is internal state seeded from `default_flow_steps`; every change is reported through
//! `on_flow_change`. A controlled `flow_steps` prop is not offered.
//! Design choice: edits address steps by id, recursively through branch paths, so a step keeps its identity when a
//! neighbour moves. Undo keeps whole-flow snapshots, capped at `UNDO_LIMIT`.
//! Design choice: a path reads as skipped when every step in it has run result `Skipped`.
//! Browser checks: canvas pan and drag, pulse travel into a running step, add-menu flip above the anchor, touch targets.

use dioxus::prelude::*;

use crate::components::agent_context_map::focus_on_mount;
use crate::internal::*;

const CSS: Asset = asset!("/assets/automation-flow-editor.css");

/// Most snapshots the undo stack keeps.
pub const UNDO_LIMIT: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    Trigger,
    Action,
    Agent,
    Branch,
    Wait,
}

impl StepKind {
    pub fn word(self) -> &'static str {
        match self {
            Self::Trigger => "Trigger",
            Self::Action => "Action",
            Self::Agent => "Agent",
            Self::Branch => "Branch",
            Self::Wait => "Wait",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepIcon {
    Webhook,
    Schedule,
    Form,
    Mail,
    Message,
    Chat,
    Database,
    Http,
    Wait,
    Branch,
    User,
    Task,
    Flash,
}

impl StepIcon {
    pub fn name(self) -> &'static str {
        match self {
            Self::Webhook => "webhook",
            Self::Schedule => "schedule",
            Self::Form => "form",
            Self::Mail => "mail",
            Self::Message => "message",
            Self::Chat => "chat",
            Self::Database => "database",
            Self::Http => "http",
            Self::Wait => "wait",
            Self::Branch => "branch",
            Self::User => "user",
            Self::Task => "task",
            Self::Flash => "flash",
        }
    }

    /// Single-character glyph shown in the icon tile.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Webhook => "W",
            Self::Schedule => "S",
            Self::Form => "F",
            Self::Mail => "@",
            Self::Message => "M",
            Self::Chat => "C",
            Self::Database => "D",
            Self::Http => "H",
            Self::Wait => "…",
            Self::Branch => "⑂",
            Self::User => "U",
            Self::Task => "T",
            Self::Flash => "⚡",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RunResult {
    #[default]
    Idle,
    InProgress,
    Done,
    Failed,
    Skipped,
}

impl RunResult {
    pub fn word(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::InProgress => "Running",
            Self::Done => "Passed",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Step {
    pub step_id: String,
    pub step_kind: StepKind,
    pub step_title: String,
    pub step_detail: Option<String>,
    pub step_note: Option<String>,
    pub step_icon: Option<StepIcon>,
    pub branch_paths: Vec<Path>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Path {
    pub path_name: String,
    pub path_steps: Vec<Step>,
}

/// What the add menu creates. A branch template carries the names of its paths.
#[derive(Clone, PartialEq, Debug)]
pub struct Template {
    pub step_kind: StepKind,
    pub step_title: String,
    pub step_detail: Option<String>,
    pub step_icon: Option<StepIcon>,
    pub branch_paths: Vec<String>,
}

fn build_step(template: &Template, id: String) -> Step {
    Step {
        step_id: id,
        step_kind: template.step_kind,
        step_title: template.step_title.clone(),
        step_detail: template.step_detail.clone(),
        step_note: None,
        step_icon: template.step_icon,
        branch_paths: template
            .branch_paths
            .iter()
            .map(|name| Path {
                path_name: name.clone(),
                path_steps: Vec::new(),
            })
            .collect(),
    }
}

/// Inserts `new` directly after the step with `after_id`, searching branch paths too. Returns whether it was found.
pub fn insert_after(steps: &mut Vec<Step>, after_id: &str, new: Step) -> bool {
    if let Some(pos) = steps.iter().position(|s| s.step_id == after_id) {
        steps.insert(pos + 1, new);
        return true;
    }
    for step in steps.iter_mut() {
        for path in step.branch_paths.iter_mut() {
            if insert_after(&mut path.path_steps, after_id, new.clone()) {
                return true;
            }
        }
    }
    false
}

/// Removes a step. Removing a branch hands its first path's steps back to the flow at the same place.
pub fn remove_step(steps: &mut Vec<Step>, id: &str) -> bool {
    if let Some(pos) = steps.iter().position(|s| s.step_id == id) {
        let removed = steps.remove(pos);
        let mut handed_back = removed
            .branch_paths
            .into_iter()
            .next()
            .map(|p| p.path_steps)
            .unwrap_or_default();
        for (offset, step) in handed_back.drain(..).enumerate() {
            steps.insert(pos + offset, step);
        }
        return true;
    }
    for step in steps.iter_mut() {
        for path in step.branch_paths.iter_mut() {
            if remove_step(&mut path.path_steps, id) {
                return true;
            }
        }
    }
    false
}

/// Moves a step up (`-1`) or down (`1`) within its own list. Returns whether it moved.
pub fn move_step(steps: &mut [Step], id: &str, delta: isize) -> bool {
    if let Some(pos) = steps.iter().position(|s| s.step_id == id) {
        let target = pos as isize + delta;
        if target < 0 || target >= steps.len() as isize {
            return false;
        }
        steps.swap(pos, target as usize);
        return true;
    }
    for step in steps.iter_mut() {
        for path in step.branch_paths.iter_mut() {
            if move_step(&mut path.path_steps, id, delta) {
                return true;
            }
        }
    }
    false
}

/// Every step id in reading order: each step, then its paths top to bottom.
pub fn reading_order(steps: &[Step]) -> Vec<String> {
    let mut ids = Vec::new();
    for step in steps {
        ids.push(step.step_id.clone());
        for path in &step.branch_paths {
            ids.extend(reading_order(&path.path_steps));
        }
    }
    ids
}

/// Total step count, branch paths included.
pub fn count_steps(steps: &[Step]) -> usize {
    steps
        .iter()
        .map(|s| {
            1 + s
                .branch_paths
                .iter()
                .map(|p| count_steps(&p.path_steps))
                .sum::<usize>()
        })
        .sum()
}

/// A flow runs as a test only with at least two steps.
pub fn can_run(steps: &[Step]) -> bool {
    count_steps(steps) >= 2
}

/// Run result for a step, `Idle` when the map has no entry.
pub fn result_for(results: &[(String, RunResult)], id: &str) -> RunResult {
    results
        .iter()
        .find(|(k, _)| k == id)
        .map(|(_, r)| *r)
        .unwrap_or_default()
}

/// A path is skipped when it has steps and every one of them was skipped.
pub fn path_skipped(path: &Path, results: &[(String, RunResult)]) -> bool {
    !path.path_steps.is_empty()
        && path
            .path_steps
            .iter()
            .all(|s| result_for(results, &s.step_id) == RunResult::Skipped)
}

/// Snapshot stack for undo, oldest first, capped at `UNDO_LIMIT`.
pub fn push_snapshot(stack: &mut Vec<Vec<Step>>, snapshot: Vec<Step>) {
    stack.push(snapshot);
    if stack.len() > UNDO_LIMIT {
        stack.remove(0);
    }
}

/// One list of steps, recursive through branch paths. Owns no state; reports every action upward.
#[component]
fn FlowSteps(
    steps: Vec<Step>,
    results: Vec<(String, RunResult)>,
    selected: Option<String>,
    menu_at: Option<String>,
    templates: Vec<Template>,
    base: String,
    on_pick: EventHandler<String>,
    on_menu: EventHandler<String>,
    on_insert: EventHandler<(String, Template)>,
    on_focus: EventHandler<Option<String>>,
) -> Element {
    rsx! {
        for step in steps.iter() {
            {
                let result = result_for(&results, &step.step_id);
                let result_word = result.word();
                let is_selected = selected.as_deref() == Some(step.step_id.as_str());
                let id_pick = step.step_id.clone();
                let id_focus = step.step_id.clone();
                let id_menu = step.step_id.clone();
                let btn_id = format!("{}-btn", step.step_id);
                let plus_id = format!("{}-plus", step.step_id);
                let menu_id = format!("{}-menu", step.step_id);
                let icon = step.step_icon.map(|i| i.glyph()).unwrap_or("·");
                let title = step.step_title.clone();
                let detail = step.step_detail.clone();
                let note = if result == RunResult::InProgress { step.step_note.clone() } else { None };
                let skipped = result == RunResult::Skipped;
                let kind = step.step_kind.word();
                let menu_open = menu_at.as_deref() == Some(step.step_id.as_str());
                let label = format!("{title}, {kind}, {result_word}");
                let after_label = format!("Add a step after {title}");
                let paths = step.branch_paths.clone();
                let templates_here = templates.clone();
                let base_here = base.clone();
                rsx! {
                    div { key: "{step.step_id}", class: "kit-afe__node", "data-kind": "{kind}", "data-result": "{result_word}", "data-skipped": skipped.then_some("true"),
                        button {
                            id: "{btn_id}", r#type: "button", class: "kit-afe__step", "aria-pressed": "{is_selected}", "aria-label": "{label}",
                            onclick: move |_| on_pick.call(id_pick.clone()),
                            onfocus: move |_| on_focus.call(Some(id_focus.clone())),
                            onblur: move |_| on_focus.call(None),
                            span { class: "kit-afe__icon", "aria-hidden": "true", "{icon}" }
                            span { class: "kit-afe__body",
                                span { class: "kit-afe__title", "{title}" }
                                if let Some(d) = detail { span { class: "kit-afe__detail", "{d}" } }
                                if let Some(n) = note { span { class: "kit-afe__note", "{n}" } }
                            }
                            span { class: "kit-afe__tag", "{result_word}" }
                        }
                        if !paths.is_empty() {
                            div { class: "kit-afe__paths",
                                for path in paths.iter() {
                                    {
                                        let path_name = path.path_name.clone();
                                        let path_skip = path_skipped(path, &results);
                                        let path_steps = path.path_steps.clone();
                                        let path_templates = templates_here.clone();
                                        let base_here = base_here.clone();
                                        rsx! {
                                            section { key: "{path_name}", class: "kit-afe__path", "data-skipped": path_skip.then_some("true"), "aria-label": "{path_name}",
                                                h4 { class: "kit-afe__path-name", "{path_name}" }
                                                FlowSteps {
                                                    steps: path_steps,
                                                    results: results.clone(),
                                                    selected: selected.clone(),
                                                    menu_at: menu_at.clone(),
                                                    templates: path_templates,
                                                    base: base_here.clone(),
                                                    on_pick,
                                                    on_menu,
                                                    on_insert,
                                                    on_focus,
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "kit-afe__join",
                            button {
                                id: "{plus_id}", r#type: "button", class: "kit-afe__plus", "aria-label": "{after_label}", "aria-haspopup": "menu",
                                "aria-expanded": "{menu_open}",
                                onclick: move |_| on_menu.call(id_menu.clone()),
                                "+"
                            }
                            if menu_open {
                                ul { id: "{menu_id}", class: "kit-afe__menu", role: "menu", "aria-label": "Insert a step", tabindex: "-1",
                                    onmounted: focus_on_mount,
                                    if templates_here.is_empty() {
                                        li { role: "none", span { class: "kit-afe__menu-empty", "No steps to insert" } }
                                    }
                                    for template in templates_here.iter() {
                                        {
                                            let t = template.clone();
                                            let anchor = step.step_id.clone();
                                            let name = template.step_title.clone();
                                            rsx! {
                                                li { key: "{name}", role: "none",
                                                    button {
                                                        r#type: "button", role: "menuitem", class: "kit-afe__menu-item",
                                                        onclick: move |_| on_insert.call((anchor.clone(), t.clone())),
                                                        "{name}"
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
}

/// Automation flow editor. Edits are undoable; a test run marks each step with its run result.
#[component]
pub fn AutomationFlowEditor(
    #[props(default = "Untitled workflow".to_string(), into)] flow_name: String,
    #[props(default)] default_flow_steps: Vec<Step>,
    #[props(default)] insert_templates: Vec<Template>,
    #[props(default)] trigger_templates: Vec<Template>,
    #[props(default)] run_results: Vec<(String, RunResult)>,
    #[props(default)] on_flow_change: Option<EventHandler<Vec<Step>>>,
    #[props(default)] on_select: Option<EventHandler<Option<String>>>,
    #[props(default)] on_run: Option<EventHandler<()>>,
    #[props(default)] on_stop: Option<EventHandler<()>>,
    #[props(default)] running: bool,
    #[props(default = 420)] canvas_height: u32,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-afe-{}", next_id()));
    let mut steps = use_signal(|| default_flow_steps.clone());
    let mut history = use_signal(Vec::<Vec<Step>>::new);
    let mut selected = use_signal(|| None::<String>);
    let mut menu_at = use_signal(|| None::<String>);
    let mut focused = use_signal(|| None::<String>);
    let mut announce = use_signal(String::new);

    // Copy-only captures, so every handler below owns its own copy.
    let apply = use_callback(move |next: Vec<Step>| {
        let previous = steps();
        push_snapshot(&mut history.write(), previous);
        steps.set(next.clone());
        if let Some(h) = on_flow_change {
            h.call(next);
        }
    });

    let current = steps();
    let count = count_steps(&current);
    let runnable = can_run(&current);
    let not_runnable = !runnable;
    let class = class_list("kit-afe", &extra_class);
    let is_empty = current.is_empty();
    let triggers = trigger_templates.clone();
    let inserts = insert_templates.clone();
    let style = format!("height: {canvas_height}px;");
    let running_now = running;
    let base_insert = base.clone();

    rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            section {
                class: "{class}", "aria-labelledby": "{base}-title",
                onkeydown: move |event| {
                    let mods = event.modifiers();
                    match event.key() {
                        Key::Escape => {
                            if let Some(anchor) = menu_at() {
                                menu_at.set(None);
                                document::eval(&format!("document.getElementById('{anchor}-plus')?.focus();"));
                            } else {
                                selected.set(None);
                                if let Some(h) = on_select {
                                    h.call(None);
                                }
                            }
                        }
                        Key::Backspace | Key::Delete => {
                            if let Some(id) = focused() {
                                event.prevent_default();
                                let index = reading_order(&steps()).iter().position(|x| *x == id).unwrap_or(0);
                                let mut next = steps();
                                if remove_step(&mut next, &id) {
                                    apply.call(next);
                                    let ids = reading_order(&steps());
                                    let previous = index.checked_sub(1).and_then(|p| ids.get(p).cloned());
                                    let focus_target = previous.clone().or_else(|| ids.get(index).cloned());
                                    selected.set(previous.clone());
                                    if let Some(h) = on_select {
                                        h.call(previous);
                                    }
                                    if let Some(target) = focus_target {
                                        focused.set(Some(target.clone()));
                                        document::eval(&format!("document.getElementById('{target}-btn')?.focus();"));
                                    }
                                    announce.set("Step removed".to_string());
                                }
                            }
                        }
                        Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End => {
                            let ids = reading_order(&steps());
                            if ids.is_empty() {
                                return;
                            }
                            let here = selected().and_then(|s| ids.iter().position(|x| *x == s));
                            if mods.alt() && matches!(event.key(), Key::ArrowUp | Key::ArrowDown) {
                                if let Some(id) = selected() {
                                    event.prevent_default();
                                    let delta = if event.key() == Key::ArrowUp { -1 } else { 1 };
                                    let mut next = steps();
                                    if move_step(&mut next, &id, delta) {
                                        apply.call(next);
                                    }
                                }
                                return;
                            }
                            event.prevent_default();
                            let target = match event.key() {
                                Key::Home => 0,
                                Key::End => ids.len() - 1,
                                Key::ArrowUp => here.map(|h| h.saturating_sub(1)).unwrap_or(0),
                                _ => here.map(|h| (h + 1).min(ids.len() - 1)).unwrap_or(0),
                            };
                            let next = ids[target].clone();
                            selected.set(Some(next.clone()));
                            if let Some(h) = on_select {
                                h.call(Some(next.clone()));
                            }
                            document::eval(&format!("document.getElementById('{next}-btn')?.focus();"));
                        }
                        Key::Character(c) if (mods.meta() || mods.ctrl()) && (c == "z" || c == "Z") => {
                            event.prevent_default();
                            let mut stack = history.write();
                            if let Some(prev) = stack.pop() {
                                drop(stack);
                                steps.set(prev.clone());
                                if let Some(h) = on_flow_change {
                                    h.call(prev);
                                }
                                announce.set("Undone".to_string());
                            }
                        }
                        _ => {}
                    }
                },
                header { class: "kit-afe__head",
                    div {
                        h2 { id: "{base}-title", class: "kit-afe__title", "{flow_name}" }
                        p { class: "kit-afe__count", "{count} steps" }
                    }
                    div { class: "kit-afe__actions",
                        if running_now {
                            button { r#type: "button", class: "kit-afe__run", onclick: move |_| if let Some(h) = on_stop { h.call(()) }, "Stop" }
                        } else {
                            button {
                                r#type: "button", class: "kit-afe__run", "aria-disabled": "{not_runnable}", disabled: not_runnable,
                                onclick: move |_| if let Some(h) = on_run { h.call(()) },
                                "Run test"
                            }
                        }
                        button {
                            r#type: "button", class: "kit-afe__undo", "aria-label": "Undo",
                            disabled: history().is_empty(),
                            onclick: move |_| {
                                let mut stack = history.write();
                                if let Some(prev) = stack.pop() {
                                    drop(stack);
                                    steps.set(prev.clone());
                                    if let Some(h) = on_flow_change {
                                        h.call(prev);
                                    }
                                }
                            },
                            "Undo"
                        }
                    }
                }
                div { class: "kit-afe__canvas", style: "{style}", role: "group", "aria-label": "Flow steps",
                    if is_empty {
                        div { class: "kit-afe__empty",
                            p { "Start the flow with a trigger." }
                            ul { class: "kit-afe__menu", role: "menu", "aria-label": "Choose a trigger",
                                for template in triggers.iter() {
                                    {
                                        let t = template.clone();
                                        let name = template.step_title.clone();
                                        let base_t = base.clone();
                                        rsx! {
                                            li { key: "{name}", role: "none",
                                                button {
                                                    r#type: "button", role: "menuitem", class: "kit-afe__menu-item",
                                                    onclick: move |_| {
                                                        let id = format!("{base_t}-step-{}", next_id());
                                                        let mut next = steps();
                                                        next.push(build_step(&t, id.clone()));
                                                        apply.call(next);
                                                        selected.set(Some(id.clone()));
                                                        if let Some(h) = on_select {
        h.call(Some(id.clone()));
    }
                                                    },
                                                    "{name}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        FlowSteps {
                            steps: current.clone(),
                            results: run_results.clone(),
                            selected: selected(),
                            menu_at: menu_at(),
                            templates: inserts.clone(),
                            base: base.clone(),
                            on_pick: move |id: String| {
                                selected.set(Some(id.clone()));
                                if let Some(h) = on_select {
        h.call(Some(id.clone()));
    }
                            },
                            on_menu: move |id: String| {
                                menu_at.set(if menu_at() == Some(id.clone()) { None } else { Some(id) });
                            },
                            on_insert: move |(after, template): (String, Template)| {
                                let id = format!("{base_insert}-step-{}", next_id());
                                let mut next = steps();
                                if insert_after(&mut next, &after, build_step(&template, id.clone())) {
                                    apply.call(next);
                                    selected.set(Some(id.clone()));
                                    if let Some(h) = on_select {
        h.call(Some(id.clone()));
    }
                                    let script = format!("document.getElementById('{id}-btn')?.focus();");
                                    spawn(async move {
                                        sleep(16).await;
                                        document::eval(&script);
                                    });
                                }
                                menu_at.set(None);
                            },
                            on_focus: move |id: Option<String>| focused.set(id),
                        }
                    }
                }
                p { class: "kit-afe__live", role: "status", "aria-live": "polite", "{announce}" }
            }
        }
}
