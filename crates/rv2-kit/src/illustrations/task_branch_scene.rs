//! Task branch scene: one client's task splitting into linked notes and the roles assigned to it.
//! It shows how one piece of client work becomes a set of owned pieces.
//!
//! Design choice: the nodes sit in a flow column, not absolute positions, so the scene reflows at
//! phone width without measuring anything.
//! Design choice: connectors draw in by `scaleX` or `scaleY`, and nodes rise in by opacity and
//! transform, in the order client, task, notes, roles.
//! Design choice: a role card carries its own tool icons; the spec gives roles no tool row, so the
//! caller supplies them.
//! Design choice: the hover highlight is a `data-hot` attribute on the scene, so one selector set
//! lights a node and the connectors that touch it.
//! Browser checks: connectors draw in client, task, notes, roles order; hovering a node highlights it
//! and its connectors; reduced motion places every node at once; avatars match the assignee list;
//! nothing scrolls sideways at 390 and 320.

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

#[derive(Clone, PartialEq)]
pub struct Note {
    pub id: String,
    pub label: String,
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
    pub person: Person,
    pub tool_icons: Vec<Element>,
}

/// The nodes a hover can light, in build order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchNode {
    Client,
    Task,
    Notes,
    Roles,
}

impl BranchNode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Task => "task",
            Self::Notes => "notes",
            Self::Roles => "roles",
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

#[component]
pub fn TaskBranchScene(
    client: ClientNode,
    task: TaskCard,
    notes: Vec<Note>,
    roles: Vec<Role>,
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
            div { class: "kit-branch__node kit-branch__node--client",
                onmouseenter: hover(BranchNode::Client),
                onmouseleave: leave,
                span { class: "kit-branch__kicker", {client.label.clone()} }
                span { class: "kit-branch__status", {client.connection_status.clone()} }
            }
            span { class: "kit-branch__link kit-branch__link--client", "data-from": "client" }
            div { class: "kit-branch__row",
                div { class: "kit-branch__node kit-branch__node--task",
                    onmouseenter: hover(BranchNode::Task),
                    onmouseleave: leave,
                    span { class: "kit-branch__title", {task.title.clone()} }
                    div { class: "kit-branch__avatars",
                        for assignee in task.assignees.iter() {
                            span { class: "kit-branch__avatar", "data-id": assignee.id.clone(), {assignee.avatar.clone()} }
                        }
                    }
                    div { class: "kit-branch__tools",
                        for icon in task.tool_icons.iter() {
                            span { class: "kit-branch__tool", {icon.clone()} }
                        }
                    }
                }
                span { class: "kit-branch__link kit-branch__link--notes", "data-from": "notes" }
                div { class: "kit-branch__node kit-branch__node--notes",
                    onmouseenter: hover(BranchNode::Notes),
                    onmouseleave: leave,
                    for note in notes.iter() {
                        span { class: "kit-branch__note", "data-id": note.id.clone(), {note.label.clone()} }
                    }
                }
            }
            span { class: "kit-branch__link kit-branch__link--roles", "data-from": "roles" }
            div { class: "kit-branch__roles",
                onmouseenter: hover(BranchNode::Roles),
                onmouseleave: leave,
                for role in roles.iter() {
                    div { class: "kit-branch__node kit-branch__node--role", "data-id": role.id.clone(),
                        span { class: "kit-branch__role-title", {role.title.clone()} }
                        span { class: "kit-branch__person",
                            span { class: "kit-branch__avatar", {role.person.avatar.clone()} }
                            span { {role.person.name.clone()} }
                        }
                        div { class: "kit-branch__tools",
                            for icon in role.tool_icons.iter() {
                                span { class: "kit-branch__tool", {icon.clone()} }
                            }
                        }
                    }
                }
            }
            span { class: "kit-branch__ring kit-branch__ring--a" }
            span { class: "kit-branch__ring kit-branch__ring--b" }
        }
    }
}
