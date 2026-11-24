//! The first minutes in a new workspace: name it, connect tools, invite teammates, create a first
//! agent. Four steps can be moved between in any order, a checklist shows progress, and Skip for now
//! moves on without ticking a step.
//!
//! Design choice: the invite field is its own component. Enter, comma or space adds an address, and
//! an address that is not valid is refused, which `WizardChips` cannot do.
//! Design choice: the address check is `handover_email_valid` from the assistant persona step.
//! Design choice: the greeting names the account's first name only. Time to finish is not stated.
//! Not implemented: the sidebar and the workspace-frame chrome; this screen renders page content.
//! Browser checks: default step 3 shows steps 1 and 2 done; comma adds an invite chip; Backspace on the
//! empty field removes the last one; a template chooses with the arrow keys.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};
use rv2_affect::switch::Switch;

use super::assistant_persona_step::handover_email_valid;
use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/workspace-setup-checklist.css");

/// Steps in the checklist.
pub const STEP_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TeamSize {
    #[default]
    One,
    TwoToTen,
    ElevenToFifty,
    FiftyOnePlus,
}

impl TeamSize {
    pub const ALL: [Self; 4] = [
        Self::One,
        Self::TwoToTen,
        Self::ElevenToFifty,
        Self::FiftyOnePlus,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::TwoToTen => "2-10",
            Self::ElevenToFifty => "11-50",
            Self::FiftyOnePlus => "51+",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InviteRole {
    Admin,
    #[default]
    Member,
}

impl InviteRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectableTool {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub connected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Values handed to `on_complete` once the first agent is created.
#[derive(Clone, Debug, PartialEq)]
pub struct SetupValues {
    pub workspace_name: String,
    pub url_slug: String,
    pub team_size: TeamSize,
    pub connected_tools: Vec<String>,
    pub invite_emails: Vec<String>,
    pub invite_role: InviteRole,
    pub agent_template: String,
}

/// The workspace address derived from its name.
pub fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut dash = false;
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            dash = false;
        } else if !dash && !slug.is_empty() {
            slug.push('-');
            dash = true;
        }
    }
    slug.trim_end_matches('-').to_string()
}

/// "N of 4 done".
pub fn progress_line(done: usize) -> String {
    format!("{done} of {STEP_COUNT} done")
}

/// The hint under an unfinished step's title.
pub fn step_hint(number: usize) -> &'static str {
    match number {
        1 => "Name the workspace",
        2 => "Pick the tools to connect",
        3 => "Add teammates by email",
        _ => "Create the first agent",
    }
}

/// The one-line status under a step's title: the value entered on a finished step, else its hint.
pub fn step_status(number: usize, state: &str, value: &str) -> String {
    if state == "done" && !value.is_empty() {
        value.to_string()
    } else {
        step_hint(number).to_string()
    }
}

/// The word that states a step's progress.
pub fn step_word(state: &str) -> &'static str {
    match state {
        "done" => "Done",
        "current" => "In progress",
        _ => "Not started",
    }
}

/// The letter that marks a connectable tool in its row.
pub fn tool_mark(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default()
}

/// Continue is off until the step's required field is valid.
pub fn step_valid(step: usize, name: &str) -> bool {
    match step {
        1 => !name.trim().is_empty(),
        _ => true,
    }
}

/// Splits a comma, space or Enter separated entry into addresses, keeping only valid ones.
pub fn valid_invites(text: &str) -> (Vec<String>, Vec<String>) {
    let mut valid = Vec::new();
    let mut rejected = Vec::new();
    for part in text
        .split([',', ' ', '\n'])
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        if handover_email_valid(part) {
            valid.push(part.to_string());
        } else {
            rejected.push(part.to_string());
        }
    }
    (valid, rejected)
}

/// The checklist's first-minutes flow.
#[component]
pub fn WorkspaceSetupChecklist(
    #[props(default, into)] workspace_name: String,
    #[props(default, into)] account_first_name: String,
    #[props(default = 2)] default_step: usize,
    #[props(default)] default_invites: Vec<String>,
    #[props(default)] connected_tools: Vec<ConnectableTool>,
    #[props(default)] templates: Vec<AgentTemplate>,
    #[props(default)] team_size: TeamSize,
    #[props(default)] crumbs: Vec<String>,
    on_complete: EventHandler<SetupValues>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
) -> Element {
    let id = use_hook(|| format!("kit-wsc-{}", next_id()));
    let start = default_step.clamp(1, STEP_COUNT);
    let mut current = use_signal(move || start);
    let mut done = use_signal(move || {
        (1..=STEP_COUNT)
            .map(|step| step < start)
            .collect::<Vec<bool>>()
    });
    let mut name = use_signal(move || workspace_name);
    let mut size = use_signal(move || team_size);
    let tool_seed: Vec<String> = connected_tools
        .iter()
        .filter(|t| t.connected)
        .map(|t| t.id.clone())
        .collect();
    let mut tools = use_signal(move || tool_seed);
    let mut invites = use_signal(move || default_invites);
    let mut invite_draft = use_signal(String::new);
    let mut invite_error = use_signal(|| None::<String>);
    let mut role = use_signal(InviteRole::default);
    let first_template = templates.first().map(|t| t.id.clone()).unwrap_or_default();
    let mut template = use_signal(move || first_template);
    let mut agent_name = use_signal(String::new);

    let step = current();
    let done_now = done();
    let done_count = done_now.iter().filter(|d| **d).count();
    let slug = slugify(&name());
    let name_now = name();
    let can_continue = step_valid(step, &name_now);
    let line = progress_line(done_count);
    let first = account_first_name.clone();
    let tools_list = connected_tools.clone();
    let templates_list = templates.clone();
    let invite_chip_list = invites();
    let error = invite_error();
    let invited = invite_chip_list.len();
    let last = step == STEP_COUNT;
    let continue_label = if last { "Create agent" } else { "Continue" };
    let step_titles = [
        "Workspace",
        "Connect your tools",
        "Invite your team",
        "Your first agent",
    ];
    let title_now = step_titles[step - 1];
    let mut direction = use_signal(|| "forward");
    let mut go_to = move |target: usize| {
        let now = *current.peek();
        direction.set(if target < now { "back" } else { "forward" });
        current.set(target);
    };
    let mut advance = move || {
        done.write()[step - 1] = true;
        if last {
            on_complete.call(SetupValues {
                workspace_name: name.peek().trim().to_string(),
                url_slug: slugify(&name.peek()),
                team_size: size(),
                connected_tools: tools.peek().clone(),
                invite_emails: invites.peek().clone(),
                invite_role: role(),
                agent_template: template.peek().clone(),
            });
        } else {
            go_to(step + 1);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-wsc", "data-step": "{step}",
            if !crumbs.is_empty() {
                nav { class: "kit-wsc__crumbs", "aria-label": "Breadcrumb",
                    ol {
                        for (index, crumb) in crumbs.iter().enumerate() {
                            li { key: "{index}", "{crumb}" }
                        }
                    }
                }
            }
            aside { class: "kit-wsc__aside",
                p { class: "kit-wsc__greeting",
                    if first.is_empty() { "Welcome" } else { "Welcome, {first}" }
                }
                p { class: "kit-wsc__lead", "A few short steps set up the workspace. Skip any of them and come back later." }
                p { class: "kit-wsc__progress", role: "status", "aria-live": "polite", "{line}" }
                ol { class: "kit-wsc__steps", "aria-label": "Setup steps",
                    for (index, title) in step_titles.iter().enumerate() {
                        {
                            let number = index + 1;
                            let state = if done_now[index] { "done" } else if number == step { "current" } else { "upcoming" };
                            let word = step_word(state);
                            let value = match number {
                                1 => name_now.trim().to_string(),
                                2 => format!("{} connected", tools().len()),
                                3 => format!("{invited} invited"),
                                _ => agent_name().trim().to_string(),
                            };
                            let status = step_status(number, state, &value);
                            let key = number;
                            rsx! {
                                li { key: "{key}", class: "kit-wsc__step-item", "data-state": state,
                                    button {
                                        r#type: "button",
                                        class: "kit-wsc__step",
                                        "aria-current": (number == step).then_some("step"),
                                        onclick: move |_| go_to(number),
                                        span { class: "kit-wsc__step-title", "{title}" }
                                        span { class: "kit-wsc__step-word", "{word}" }
                                        span { class: "kit-wsc__step-status", "{status}" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-wsc__help",
                    p { class: "kit-wsc__help-text", "Want help with setup?" }
                    button {
                        r#type: "button",
                        class: "kit-wsc__help-link",
                        onclick: move |_| {
                            if let Some(nav) = on_navigate {
                                nav.call("Book a setup call".to_string());
                            }
                        },
                        "Book a setup call"
                    }
                }
            }
            section { class: "kit-wsc__form", "aria-labelledby": "{id}-title",
                div { key: "{step}", class: "kit-wsc__stage", "data-dir": "{direction}",
                h1 { id: "{id}-title", class: "kit-wsc__title", "{title_now}" }
                if step == 1 {
                    div { class: "kit-wsc__field",
                        label { class: "kit-wsc__label", r#for: "{id}-name", "Workspace name" }
                        input {
                            id: "{id}-name",
                            class: "kit-wsc__input",
                            r#type: "text",
                            autofocus: true,
                            value: "{name}",
                            oninput: move |event: FormEvent| name.set(event.value()),
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Enter && step_valid(1, &name.peek()) {
                                    event.prevent_default();
                                    done.write()[0] = true;
                                    go_to(2);
                                }
                            },
                        }
                        p { class: "kit-wsc__hint", "Workspace address: {slug}" }
                    }
                    fieldset { class: "kit-wsc__field",
                        legend { class: "kit-wsc__label", "Team size" }
                        div { class: "kit-wsc__options",
                            for option in TeamSize::ALL {
                                {
                                    let value = option.as_str();
                                    rsx! {
                                        label { key: "{value}", class: "kit-wsc__option",
                                            input {
                                                r#type: "radio",
                                                name: "{id}-size",
                                                value: "{value}",
                                                checked: size() == option,
                                                onchange: move |_| size.set(option),
                                            }
                                            span { "{value}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if step == 2 {
                    ul { class: "kit-wsc__tools",
                        for tool in tools_list.iter() {
                            {
                                let key = tool.id.clone();
                                let tool_id = tool.id.clone();
                                let name_text = tool.name.clone();
                                let summary = tool.summary.clone();
                                let tool_mark_letter = tool_mark(&tool.name);
                                let on = tools().contains(&tool.id);
                                rsx! {
                                    li { key: "{key}", class: "kit-wsc__tool",
                                        span { class: "kit-wsc__tool-mark", "aria-hidden": "true", "{tool_mark_letter}" }
                                        Switch {
                                            id: format!("{id}-tool-{key}"),
                                            label: rsx! { "{name_text}" },
                                            description: rsx! { "{summary}" },
                                            checked: on,
                                            onchange: move |event: FormEvent| {
                                                let mut next = tools.peek().clone();
                                                next.retain(|t| *t != tool_id);
                                                if event.checked() {
                                                    next.push(tool_id.clone());
                                                }
                                                tools.set(next);
                                            },
                                        }
                                        if on {
                                            span { class: "kit-wsc__badge", "Connected" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if step == 3 {
                    div { class: "kit-wsc__field",
                        label { class: "kit-wsc__label", r#for: "{id}-invite", "Invite by email" }
                        div { class: "kit-wsc__box", "data-invalid": error.is_some().then_some("true"),
                            for address in invite_chip_list.iter() {
                                {
                                    let remove = address.clone();
                                    let key = address.clone();
                                    let label = format!("Remove {address}");
                                    rsx! {
                                        span { key: "{key}", class: "kit-wsc__chip",
                                            "{key}"
                                            button {
                                                r#type: "button",
                                                class: "kit-wsc__chip-remove",
                                                "aria-label": "{label}",
                                                onclick: move |_| invites.write().retain(|a| *a != remove),
                                                "×"
                                            }
                                        }
                                    }
                                }
                            }
                            input {
                                id: "{id}-invite",
                                class: "kit-wsc__input-inline",
                                r#type: "email",
                                value: "{invite_draft}",
                                "aria-invalid": error.is_some().then_some("true"),
                                oninput: move |event: FormEvent| {
                                    let value = event.value();
                                    if value.ends_with([',', ' ']) {
                                        let (valid, rejected) = valid_invites(&value);
                                        let mut next = invites.peek().clone();
                                        for address in valid {
                                            if !next.contains(&address) {
                                                next.push(address);
                                            }
                                        }
                                        invites.set(next);
                                        invite_error.set(rejected.first().cloned());
                                        invite_draft.set(rejected.first().cloned().unwrap_or_default());
                                    } else {
                                        invite_draft.set(value);
                                        invite_error.set(None);
                                    }
                                },
                                onkeydown: move |event: KeyboardEvent| {
                                    if event.key() == Key::Backspace && invite_draft.peek().is_empty() {
                                        invites.write().pop();
                                    } else if event.key() == Key::Enter {
                                        event.prevent_default();
                                        let typed = invite_draft.peek().clone();
                                        let (valid, rejected) = valid_invites(&typed);
                                        if !valid.is_empty() {
                                            let mut next = invites.peek().clone();
                                            for address in valid {
                                                if !next.contains(&address) {
                                                    next.push(address);
                                                }
                                            }
                                            invites.set(next);
                                            invite_draft.set(String::new());
                                            invite_error.set(None);
                                        } else if !rejected.is_empty() {
                                            invite_error.set(rejected.first().cloned());
                                        }
                                    }
                                },
                            }
                        }
                        p { class: "kit-wsc__hint", role: "status", "aria-live": "polite",
                            if let Some(bad) = error.clone() {
                                "“{bad}” is not an email address."
                            } else {
                                "{invited} invited."
                            }
                        }
                    }
                    fieldset { class: "kit-wsc__field",
                        legend { class: "kit-wsc__label", "Role" }
                        div { class: "kit-wsc__options",
                            for option in [InviteRole::Admin, InviteRole::Member] {
                                {
                                    let value = option.as_str();
                                    let text = if option == InviteRole::Admin { "Admin" } else { "Member" };
                                    rsx! {
                                        label { key: "{value}", class: "kit-wsc__option",
                                            input {
                                                r#type: "radio",
                                                name: "{id}-role",
                                                value: "{value}",
                                                checked: role() == option,
                                                onchange: move |_| role.set(option),
                                            }
                                            span { "{text}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    fieldset { class: "kit-wsc__field",
                        legend { class: "kit-wsc__label", "Template" }
                        div { class: "kit-wsc__templates",
                            for entry in templates_list.iter() {
                                {
                                    let value = entry.id.clone();
                                    let title = entry.name.clone();
                                    let detail = entry.description.clone();
                                    let checked = template() == entry.id;
                                    let pick = entry.id.clone();
                                    rsx! {
                                        label { key: "{value}", class: "kit-wsc__option kit-wsc__option--block",
                                            input {
                                                class: "kit-wsc__radio",
                                                r#type: "radio",
                                                name: "{id}-template",
                                                value: "{value}",
                                                checked,
                                                onchange: move |_| template.set(pick.clone()),
                                                onkeydown: move |event: KeyboardEvent| {
                                                    if event.key() == Key::Enter && can_continue {
                                                        event.prevent_default();
                                                        advance();
                                                    }
                                                },
                                            }
                                            span { class: "kit-wsc__option-title", "{title}" }
                                            span { class: "kit-wsc__option-detail", "{detail}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-wsc__field",
                        label { class: "kit-wsc__label", r#for: "{id}-agent", "Agent name" }
                        input {
                            id: "{id}-agent",
                            class: "kit-wsc__input",
                            r#type: "text",
                            value: "{agent_name}",
                            oninput: move |event: FormEvent| agent_name.set(event.value()),
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Enter && can_continue {
                                    event.prevent_default();
                                    advance();
                                }
                            },
                        }
                    }
                }
                }
                div { class: "kit-wsc__actions",
                    button {
                        r#type: "button",
                        class: "kit-wsc__skip",
                        onclick: move |_| {
                            if step < STEP_COUNT {
                                go_to(step + 1);
                            }
                        },
                        "Skip for now"
                    }
                    if step > 1 {
                        Button {
                            variant: ButtonVariant::Secondary,
                            label: "Back",
                            onclick: move |_| go_to(step - 1),
                        }
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        label: continue_label,
                        disabled: !can_continue,
                        onclick: move |_| advance(),
                    }
                }
            }
        }
    }
}
