//! Branch choice map: a fork an agent should not settle alone, every option with its consequences set beside it.
//!
//! Design choice: `on_select` is a `Callback` returning an optional answer, shown back under the fork. The spec's
//! promise form is not supported; a synchronous answer keeps the component free of async plumbing.
//! Design choice: a follow-up's branches are one level deep and cannot open a further follow-up.
//! Design choice: the pager walks `pending_forks` minus the forks decided here, so a fork appended by the caller
//! appears at the end without resetting the current position.
//! Design choice: the elbow connectors are CSS, not measured. The branch node is the connector anchor, so the
//! connectors hold their place at any text length, and a follow-up's rail sits above its node's consequences.
//! Browser checks: fork grow-in, path emphasis on hover, text-field growth to three lines, tree layout below the
//! container breakpoint, connector joints at each node.

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/branch-choice-map.css");

/// Rows the custom-instruction field shows before it scrolls.
pub const CUSTOM_MAX_ROWS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Risk {
    High,
    Medium,
    Low,
}

impl Risk {
    pub fn word(self) -> &'static str {
        match self {
            Self::High => "High risk",
            Self::Medium => "Medium risk",
            Self::Low => "Low risk",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Consequence {
    pub text: String,
    /// Set on a flagged consequence; its note shows on click, and on the F key.
    pub flag_explanation: Option<String>,
    pub reference_url: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Branch {
    pub branch_label: String,
    pub branch_mark: Option<String>,
    pub consequences: Vec<Consequence>,
    pub touched_apps: Vec<String>,
    pub agent_preferred: bool,
    pub follow_up: Option<FollowUp>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FollowUp {
    pub question_text: String,
    pub branches: Vec<Branch>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Fork {
    pub fork_id: String,
    pub question_text: String,
    pub context_text: Option<String>,
    pub agent_name: String,
    pub model_name: Option<String>,
    pub agent_mark: Option<String>,
    pub risk_level: Option<Risk>,
    pub branches: Vec<Branch>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FollowUpPick {
    pub index: usize,
    pub branch: Branch,
}

#[derive(Clone, PartialEq, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Selection {
    Branch {
        index: usize,
        branch: Branch,
        follow_up_selection: Option<FollowUpPick>,
    },
    Custom {
        custom_text: String,
    },
}

/// What a digit key does on a fork with `branch_count` branches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyPick {
    Branch(usize),
    Custom,
}

/// Digit `1..=branch_count` picks a branch; the digit after the last branch opens the custom field when allowed.
pub fn digit_pick(digit: u32, branch_count: usize, allow_custom: bool) -> Option<KeyPick> {
    let d = digit as usize;
    if d == 0 {
        None
    } else if d <= branch_count {
        Some(KeyPick::Branch(d - 1))
    } else if d == branch_count + 1 && allow_custom {
        Some(KeyPick::Custom)
    } else {
        None
    }
}

/// Every flagged consequence on a fork as (branch index, consequence index), in reading order.
pub fn flagged_consequences(fork: &Fork) -> Vec<(usize, usize)> {
    fork.branches
        .iter()
        .enumerate()
        .flat_map(|(b, branch)| {
            branch
                .consequences
                .iter()
                .enumerate()
                .filter(|(_, c)| c.flag_explanation.is_some())
                .map(move |(c, _)| (b, c))
        })
        .collect()
}

/// Next flagged consequence after `current`, wrapping to the first one.
pub fn next_flagged(count: usize, current: Option<usize>) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match current {
        Some(i) => (i + 1) % count,
        None => 0,
    })
}

/// Pager move, clamped to the list.
pub fn pager_step(len: usize, current: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }
    (current as isize + delta).clamp(0, len as isize - 1) as usize
}

/// A branch set may mark at most one agent preference.
pub fn preference_ok(branches: &[Branch]) -> bool {
    branches.iter().filter(|b| b.agent_preferred).count() <= 1
}

/// Accessible name of a branch button, with the preference spelled out.
pub fn branch_label_for_screen_reader(branch: &Branch) -> String {
    if branch.agent_preferred {
        format!("{}, agent's preference", branch.branch_label)
    } else {
        branch.branch_label.clone()
    }
}

/// Forks still awaiting a choice, in their given order.
pub fn pending_after(forks: &[Fork], decided: &[String]) -> Vec<Fork> {
    forks
        .iter()
        .filter(|f| !decided.contains(&f.fork_id))
        .cloned()
        .collect()
}

/// Forks awaiting a choice, one at a time, with branches and their consequences beside them.
#[component]
pub fn BranchChoiceMap(
    pending_forks: Vec<Fork>,
    #[props(default)] on_select: Option<Callback<(Fork, Selection), Option<String>>>,
    #[props(default = true)] allow_custom: bool,
    #[props(default)] initial_pick: Option<usize>,
    #[props(default = "Decisions".to_string(), into)] heading_text: String,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-bcm-{}", next_id()));
    let mut decided = use_signal(Vec::<String>::new);
    let mut answers = use_signal(Vec::<(String, String)>::new);
    let mut position = use_signal(|| 0usize);
    let mut picked = use_signal(|| initial_pick);
    let mut custom_open = use_signal(|| false);
    let mut custom_text = use_signal(String::new);
    let mut note_open = use_signal(|| None::<(usize, usize)>);
    let mut flag_cursor = use_signal(|| None::<usize>);

    // Copy-only captures, so each handler below takes its own copy.
    let commit = use_callback(move |(fork, selection): (Fork, Selection)| {
        let fork_id = fork.fork_id.clone();
        let reply = on_select.and_then(|h| h.call((fork, selection)));
        decided.write().push(fork_id.clone());
        answers.write().push((fork_id, reply.unwrap_or_default()));
        picked.set(None);
        custom_open.set(false);
        custom_text.set(String::new());
        note_open.set(None);
        flag_cursor.set(None);
        position.set(0);
    });
    let choose = use_callback(move |(fork, index): (Fork, usize)| {
        let Some(branch) = fork.branches.get(index).cloned() else {
            return;
        };
        if branch.follow_up.is_some() && picked().is_none() {
            picked.set(Some(index));
            return;
        }
        commit.call((
            fork,
            Selection::Branch {
                index,
                branch,
                follow_up_selection: None,
            },
        ));
    });

    let forks = pending_after(&pending_forks, &decided());
    let total = forks.len();
    let class = class_list("kit-bcm", &extra_class);
    let Some(fork) = forks.get(position().min(total.saturating_sub(1))).cloned() else {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            section { class: "kit-bcm kit-bcm--empty", "aria-label": "{heading_text}" }
        };
    };
    let idx = position().min(total - 1);
    let branch_count = fork.branches.len();
    let flagged = flagged_consequences(&fork);
    let answer = answers()
        .iter()
        .rev()
        .find(|(id, _)| *id == fork.fork_id)
        .map(|(_, text)| text.clone());
    let picked_index = picked();
    let pager_label = format!("{} of {total}", idx + 1);
    let count_text = format!("{total} waiting");
    let any_follow = fork.branches.iter().any(|b| b.follow_up.is_some());
    let follow_attr = if any_follow { "true" } else { "false" };
    let risk_attr = fork.risk_level.map(|r| match r {
        Risk::High => "high",
        Risk::Medium => "medium",
        Risk::Low => "low",
    });

    let fork_keys = fork.clone();
    let fork_custom_submit = fork.clone();
    let fork_custom_key = fork.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class: "{class}", "aria-labelledby": "{base}-title", "data-risk": risk_attr,
            onkeydown: move |event| {
                let key = event.key();
                match key {
                    Key::ArrowRight | Key::ArrowLeft => {
                        let delta = if key == Key::ArrowRight { 1 } else { -1 };
                        position.set(pager_step(total, idx, delta));
                        picked.set(None);
                    }
                    Key::Escape => {
                        if note_open().is_some() {
                            note_open.set(None);
                        } else {
                            custom_open.set(false);
                        }
                    }
                    Key::Character(c) => match c.as_str() {
                        "j" | "J" => {
                            position.set(pager_step(total, idx, -1));
                            picked.set(None);
                        }
                        "k" | "K" => {
                            position.set(pager_step(total, idx, 1));
                            picked.set(None);
                        }
                        "f" | "F" => {
                            let next = next_flagged(flagged.len(), flag_cursor());
                            flag_cursor.set(next);
                            if let Some(n) = next {
                                note_open.set(Some(flagged[n]));
                            }
                        }
                        digit => {
                            if let Some(pick) = digit.parse::<u32>().ok().and_then(|d| digit_pick(d, branch_count, allow_custom)) {
                                event.prevent_default();
                                match pick {
                                    KeyPick::Branch(i) => choose.call((fork_keys.clone(), i)),
                                    KeyPick::Custom => custom_open.set(true),
                                }
                            }
                        }
                    },
                    _ => {}
                }
            },
            header { class: "kit-bcm__head",
                div { class: "kit-bcm__heading",
                    h2 { id: "{base}-title", class: "kit-bcm__title", "{heading_text}" }
                    span { class: "kit-bcm__count", "{count_text}" }
                }
                div { class: "kit-bcm__pager-wrap",
                    span { class: "kit-bcm__dots", "aria-hidden": "true",
                        for n in 0..total {
                            span { key: "{n}", class: "kit-bcm__dot", "data-current": (n == idx).then_some("true") }
                        }
                    }
                    p { class: "kit-bcm__pager", "{pager_label}" }
                }
            }
            article { class: "kit-bcm__fork", "aria-label": "{fork.question_text}", "data-follow": "{follow_attr}",
                div { class: "kit-bcm__question-card",
                    div { class: "kit-bcm__meta",
                        if let Some(risk) = fork.risk_level {
                            span { class: "kit-bcm__risk", "{risk.word()}" }
                        }
                        span { class: "kit-bcm__agent", "{fork.agent_name}" }
                        if let Some(model) = fork.model_name.clone() { span { class: "kit-bcm__model", "{model}" } }
                    }
                    p { class: "kit-bcm__question", "{fork.question_text}" }
                    if let Some(context) = fork.context_text.clone() {
                        p { class: "kit-bcm__context", "{context}" }
                    }
                }
                div { class: "kit-bcm__tree",
                    ol { class: "kit-bcm__tree-list", "aria-label": "Options",
                        for (i, branch) in fork.branches.iter().enumerate() {
                            {
                                let label = branch_label_for_screen_reader(branch);
                                let is_picked = picked_index == Some(i);
                                let dimmed = picked_index.is_some() && !is_picked;
                                let fork_row = fork.clone();
                                let branch_label = branch.branch_label.clone();
                                let apps = branch.touched_apps.join(", ");
                                let follow = branch.follow_up.clone();
                                let has_follow = follow.is_some();
                                let fork_follow = fork.clone();
                                let parent_branch = branch.clone();
                                rsx! {
                                    li { key: "{i}", class: "kit-bcm__row", "data-dimmed": dimmed.then_some("true"), "data-follow": has_follow.then_some("true"),
                                        button {
                                            r#type: "button", class: "kit-bcm__branch", "aria-label": "{label}",
                                            "aria-pressed": "{is_picked}",
                                            onclick: move |_| choose.call((fork_row.clone(), i)),
                                            span { class: "kit-bcm__key", "aria-hidden": "true", "{i + 1}" }
                                            if let Some(mark) = branch.branch_mark.clone() { span { class: "kit-bcm__mark", "aria-hidden": "true", "{mark}" } }
                                            span { class: "kit-bcm__label", "{branch_label}" }
                                            if branch.agent_preferred { span { class: "kit-bcm__tag", "Agent's pick" } }
                                            if branch.follow_up.is_some() { span { class: "kit-bcm__tag", "Follow-up" } }
                                            if !apps.is_empty() { span { class: "kit-bcm__apps", "{apps}" } }
                                        }
                                        ul { class: "kit-bcm__consequences",
                                            for (c, cons) in branch.consequences.iter().enumerate() {
                                                {
                                                    let note_id = format!("{base}-note-{i}-{c}");
                                                    let showing = note_open() == Some((i, c));
                                                    let text = cons.text.clone();
                                                    let note = cons.flag_explanation.clone();
                                                    rsx! {
                                                        li { key: "{c}", class: "kit-bcm__consequence",
                                                            span { "{text}" }
                                                            if let Some(explanation) = note {
                                                                button {
                                                                    r#type: "button", class: "kit-bcm__flag", "aria-describedby": "{note_id}",
                                                                    "aria-expanded": "{showing}", "aria-label": "Flagged: {text}",
                                                                    onfocus: move |_| note_open.set(Some((i, c))),
                                                                    onclick: move |_| note_open.set(Some((i, c))),
                                                                    "!"
                                                                }
                                                                if showing {
                                                                    p { id: "{note_id}", role: "note", class: "kit-bcm__note", "{explanation}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if let Some(follow) = follow {
                                            section { class: "kit-bcm__follow", "data-live": "{is_picked}", "aria-label": "{follow.question_text}",
                                                p { class: "kit-bcm__question", "{follow.question_text}" }
                                                if !is_picked {
                                                    span { class: "kit-bcm__unchosen", "Not chosen" }
                                                }
                                                ol { class: "kit-bcm__options", "aria-label": "Follow-up options",
                                                    for (j, sub) in follow.branches.iter().enumerate() {
                                                        {
                                                            let sub_label = sub.branch_label.clone();
                                                            let fork_sub = fork_follow.clone();
                                                            let parent_sub = parent_branch.clone();
                                                            let sub_branch = sub.clone();
                                                            rsx! {
                                                                li { key: "{j}", class: "kit-bcm__sub",
                                                                    button {
                                                                        r#type: "button", class: "kit-bcm__branch", disabled: !is_picked,
                                                                        onclick: move |_| {
                                                                            commit.call((fork_sub.clone(), Selection::Branch {
                                                                                index: i,
                                                                                branch: parent_sub.clone(),
                                                                                follow_up_selection: Some(FollowUpPick { index: j, branch: sub_branch.clone() }),
                                                                            }));
                                                                        },
                                                                        span { class: "kit-bcm__key", "aria-hidden": "true", "{j + 1}" }
                                                                        span { class: "kit-bcm__label", "{sub_label}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if has_follow {
                                            span { class: "kit-bcm__rail", "aria-hidden": "true" }
                                        }
                                    }
                                }
                            }
                        }
                        if allow_custom {
                            li { class: "kit-bcm__row",
                                button {
                                    r#type: "button", class: "kit-bcm__branch kit-bcm__branch--custom",
                                    "aria-expanded": "{custom_open()}",
                                    onclick: move |_| custom_open.set(!custom_open()),
                                    span { class: "kit-bcm__key", "aria-hidden": "true", "{branch_count + 1}" }
                                    span { class: "kit-bcm__label", "Write your own instruction" }
                                }
                            }
                        }
                    }
                    if custom_open() {
                        form {
                            class: "kit-bcm__custom",
                            onsubmit: move |event| {
                                event.prevent_default();
                                let text = custom_text();
                                if !text.trim().is_empty() {
                                    commit.call((fork_custom_submit.clone(), Selection::Custom { custom_text: text }));
                                }
                            },
                            label { class: "kit-bcm__label", r#for: "{base}-custom", "Your instruction" }
                            textarea {
                                id: "{base}-custom", class: "kit-bcm__textarea", rows: "{CUSTOM_MAX_ROWS}",
                                value: "{custom_text}",
                                oninput: move |event| custom_text.set(event.value()),
                                onkeydown: move |event| {
                                    if event.key() == Key::Enter && !event.modifiers().shift() {
                                        event.prevent_default();
                                        let text = custom_text();
                                        if !text.trim().is_empty() {
                                            commit.call((fork_custom_key.clone(), Selection::Custom { custom_text: text }));
                                        }
                                    }
                                },
                            }
                            button { r#type: "submit", class: "kit-bcm__send", "Send" }
                        }
                    }
                }
                if let Some(text) = answer.filter(|text| !text.is_empty()) {
                    p { class: "kit-bcm__outcome", role: "status", "aria-live": "polite", "{text}" }
                }
            }
        }
    }
}
