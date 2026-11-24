//! Task branch scene: a client's task splits into a subtask card on one branch and a role chain on
//! the other. A side node hangs off the task's right edge. It shows how one piece of client work
//! becomes a set of owned pieces.
//!
//! Design choice: every node and connector sits at a fixed fraction of the card, so the tree keeps
//! its shape at every width; below 520px the labels are hidden rather than shrunk.
//! Design choice: the fade is a mask over the scene layer, so the rings stay outside it and every
//! node inside a band fades together.
//! Design choice: connectors are one-pixel boxes drawn with `scale`, so hairlines stay one pixel
//! whatever the card's aspect.
//! Design choice: a hover sets `data-hot` on the scene; one selector set lights a node and the
//! connectors listed in its `data-touch`.
//! Browser checks: connectors draw client, task, side, subtasks, roles, ghost; hovering a node lights
//! it and its connectors; reduced motion places every element at once; nothing scrolls sideways at
//! 390 and 320.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/task-branch-scene.css");

#[derive(Clone, PartialEq)]
pub struct ClientNode {
    pub label: String,
    pub connection_status: String,
}

#[derive(Clone, PartialEq)]
pub struct Assignee {
    pub id: String,
    pub avatar: String,
}

#[derive(Clone, PartialEq)]
pub struct TaskCard {
    pub title: String,
    pub assignees: Vec<Assignee>,
    pub tool_icons: Vec<Element>,
}

/// The side node hanging off the task's right edge.
#[derive(Clone, PartialEq)]
pub struct SideNode {
    pub label: String,
    pub meta: String,
}

#[derive(Clone, PartialEq)]
pub struct NestedCard {
    pub title: String,
    pub assignees: Vec<Assignee>,
    pub tool_icons: Vec<Element>,
}

#[derive(Clone, PartialEq)]
pub struct SubtaskCard {
    pub title: String,
    pub assignees: Vec<Assignee>,
    /// Drawn in front of the lower half of the subtasks card.
    pub nested: Option<NestedCard>,
}

#[derive(Clone, PartialEq)]
pub struct Person {
    pub name: String,
    pub avatar: String,
}

#[derive(Clone, PartialEq)]
pub struct Role {
    pub id: String,
    pub title: String,
    pub subline: String,
    pub person: Person,
    pub tool_icons: Vec<Element>,
}

/// The nodes a hover can light.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchNode {
    Client,
    Task,
    Side,
    Subtasks,
    Nested,
    Role,
    Ghost,
}

impl BranchNode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Task => "task",
            Self::Side => "side",
            Self::Subtasks => "subtasks",
            Self::Nested => "nested",
            Self::Role => "role",
            Self::Ghost => "ghost",
        }
    }
}

/// Up to two letters for an avatar: the first letter of the first two words.
pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

fn tool_row(tag: &str, icons: &[Element]) -> Element {
    rsx! {
        if !icons.is_empty() {
            div { class: "kit-branch__tools",
                span { class: "kit-branch__label kit-branch__tag", {tag.to_string()} }
                for icon in icons.iter() {
                    span { class: "kit-branch__tool", {icon.clone()} }
                }
            }
        }
    }
}

fn owners_row(assignees: &[Assignee]) -> Element {
    rsx! {
        div { class: "kit-branch__avatars",
            for assignee in assignees.iter() {
                span { class: "kit-branch__avatar", "data-id": assignee.id.clone(), {assignee.avatar.clone()} }
            }
        }
    }
}

#[component]
pub fn TaskBranchScene(
    client: ClientNode,
    task: TaskCard,
    side: SideNode,
    subtasks: SubtaskCard,
    split_label: String,
    role: Role,
    #[props(default)] ghost_role: Option<Role>,
    #[props(default, into)] tool_tag: String,
    #[props(default)] class: String,
) -> Element {
    let mut hot = use_signal(|| None::<BranchNode>);
    let hot_attr = hot().map(BranchNode::as_str);
    let class = class_list("kit-branch", &class);
    let hover = move |node: BranchNode| move |_| hot.set(Some(node));
    let leave = move |_| hot.set(None);
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class, "aria-hidden": "true", "data-hot": hot_attr,
            span { class: "kit-branch__ring kit-branch__ring--a" }
            span { class: "kit-branch__ring kit-branch__ring--b" }
            span { class: "kit-branch__ring kit-branch__ring--c" }
            div { class: "kit-branch__scene",
                div { class: "kit-branch__node kit-branch__node--client", "data-node": "client",
                    onmouseenter: hover(BranchNode::Client),
                    onmouseleave: leave,
                    span { class: "kit-branch__label kit-branch__kicker", {client.label.clone()} }
                    span { class: "kit-branch__label kit-branch__status", {client.connection_status.clone()} }
                }
                span { class: "kit-branch__link kit-branch__link--client-task", "data-connector": "client-task", "data-touch": "client task" }
                div { class: "kit-branch__node kit-branch__node--task", "data-node": "task",
                    onmouseenter: hover(BranchNode::Task),
                    onmouseleave: leave,
                    span { class: "kit-branch__label kit-branch__title", {task.title.clone()} }
                    {owners_row(&task.assignees)}
                    {tool_row(&tool_tag, &task.tool_icons)}
                }
                span { class: "kit-branch__link kit-branch__link--task-side", "data-connector": "task-side", "data-touch": "task side" }
                div { class: "kit-branch__node kit-branch__node--side", "data-node": "side",
                    onmouseenter: hover(BranchNode::Side),
                    onmouseleave: leave,
                    span { class: "kit-branch__side-icon", "aria-hidden": "true" }
                    span { class: "kit-branch__label kit-branch__side-label", {side.label.clone()} }
                    span { class: "kit-branch__label kit-branch__meta", {side.meta.clone()} }
                }
                span { class: "kit-branch__link kit-branch__link--stem", "data-connector": "stem", "data-touch": "task subtasks roles" }
                span { class: "kit-branch__link kit-branch__link--bar", "data-connector": "stem", "data-touch": "task subtasks roles" }
                span { class: "kit-branch__link kit-branch__link--subtasks", "data-connector": "subtasks", "data-touch": "subtasks" }
                div { class: "kit-branch__node kit-branch__node--subtasks", "data-node": "subtasks",
                    onmouseenter: hover(BranchNode::Subtasks),
                    onmouseleave: leave,
                    span { class: "kit-branch__label kit-branch__title", {subtasks.title.clone()} }
                    {owners_row(&subtasks.assignees)}
                }
                if let Some(nested) = subtasks.nested.as_ref() {
                    div { class: "kit-branch__node kit-branch__node--nested", "data-node": "nested",
                        onmouseenter: hover(BranchNode::Nested),
                        onmouseleave: leave,
                        span { class: "kit-branch__label kit-branch__title", {nested.title.clone()} }
                        {owners_row(&nested.assignees)}
                        {tool_row(&tool_tag, &nested.tool_icons)}
                    }
                }
                span { class: "kit-branch__link kit-branch__link--roles", "data-connector": "roles", "data-touch": "roles" }
                span { class: "kit-branch__chip kit-branch__label", "data-touch": "roles", {split_label.clone()} }
                div { class: "kit-branch__node kit-branch__node--role", "data-node": "role",
                    onmouseenter: hover(BranchNode::Role),
                    onmouseleave: leave,
                    span { class: "kit-branch__avatar kit-branch__avatar--portrait", {role.person.avatar.clone()} }
                    div { class: "kit-branch__role-text",
                        span { class: "kit-branch__label kit-branch__role-title", {role.title.clone()} }
                        span { class: "kit-branch__label kit-branch__subline", {role.subline.clone()} }
                    }
                    {tool_row(&tool_tag, &role.tool_icons)}
                }
                if let Some(ghost) = ghost_role.as_ref() {
                    span { class: "kit-branch__link kit-branch__link--ghost", "data-connector": "ghost", "data-touch": "roles ghost" }
                    div { class: "kit-branch__node kit-branch__node--ghost", "data-node": "ghost",
                        onmouseenter: hover(BranchNode::Ghost),
                        onmouseleave: leave,
                        span { class: "kit-branch__avatar kit-branch__avatar--portrait", {ghost.person.avatar.clone()} }
                        div { class: "kit-branch__role-text",
                            span { class: "kit-branch__label kit-branch__role-title", {ghost.title.clone()} }
                            span { class: "kit-branch__label kit-branch__subline", {ghost.subline.clone()} }
                        }
                        {tool_row(&tool_tag, &ghost.tool_icons)}
                    }
                }
                span { class: "kit-branch__dot", style: "left: 49.5%; top: 24%;", "data-touch": "client task" }
                span { class: "kit-branch__dot", style: "left: 49.5%; top: 27%;", "data-touch": "client task" }
                span { class: "kit-branch__dot", style: "left: 58%; top: 34%;", "data-touch": "task side" }
                span { class: "kit-branch__dot", style: "left: 60%; top: 34%;", "data-touch": "task side" }
                span { class: "kit-branch__dot", style: "left: 49.5%; top: 41%;", "data-touch": "task subtasks roles" }
                span { class: "kit-branch__dot", style: "left: 34%; top: 50%;", "data-touch": "subtasks" }
                span { class: "kit-branch__dot", style: "left: 63.5%; top: 52%;", "data-touch": "roles" }
                span { class: "kit-branch__dot", style: "left: 63.5%; top: 55%;", "data-touch": "roles" }
                span { class: "kit-branch__dot", style: "left: 63.5%; top: 69%;", "data-touch": "roles ghost" }
                if ghost_role.is_some() {
                    span { class: "kit-branch__dot", style: "left: 63.5%; top: 71%;", "data-touch": "roles ghost" }
                }
            }
        }
    }
}
