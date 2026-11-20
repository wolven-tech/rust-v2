//! Tree view: a nested list of folders and files, or any hierarchy. Follows the WAI-ARIA tree pattern: one tab stop,
//! ↑ ↓ move, → opens or goes to the first child, ← closes or goes to the parent, Home and End go to the ends, `*`
//! opens every sibling branch, and typing a name jumps to the next row that starts with it.
//!
//! Port losses: `ref` is not forwarded. `onSelect` receives the node (its id is `node.id`), not the id and the node
//! as two arguments. `selected` takes `Option<String>`, so a controlled tree cannot be controlled at "no selection".
//! Closing a branch from outside the tree leaves the focus where it was; the React original moves it to the nearest
//! row still shown.

use std::collections::HashSet;

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/tree-view.css");

/// Focuses the row whose `data-id` is `__ID__` inside the tree `__TREE__`.
const FOCUS_ROW: &str = r#"
const tree = document.getElementById("__TREE__");
const row = tree && [...tree.querySelectorAll('[role="treeitem"]')].find((el) => el.dataset.id === __ID__);
if (row) row.focus();
"#;

const RTL: &str = r#"
const tree = document.getElementById("__TREE__");
return !!tree && getComputedStyle(tree).direction === "rtl";
"#;

/// A node of the tree. Its `children` make it a branch: any list, even an empty one, opens and closes.
#[derive(Clone, PartialEq)]
pub struct TreeNode {
    /// Unique in the whole tree. A repeated id keeps its first occurrence and drops the rest.
    pub id: String,
    /// The name: one line, cut short with … when long, and what typing jumps to.
    pub label: String,
    /// A glyph before the name, 16px.
    pub icon: Option<Element>,
    /// A glyph for while the branch is open. Falls back to `icon`.
    pub open_icon: Option<Element>,
    /// Quiet text at the end of the row, e.g. a count.
    pub meta: Option<Element>,
    pub disabled: bool,
    pub children: Option<Vec<TreeNode>>,
}

/// A row on screen, with what the keys need to know about it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub label: String,
    pub depth: usize,
    pub parent: Option<String>,
    /// 1-based place among the siblings, and how many there are.
    pub pos: usize,
    pub size: usize,
    pub branch: bool,
    pub expanded: bool,
    pub disabled: bool,
}

/// What a key asks for, before the component applies it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeKey {
    Down,
    Up,
    Home,
    End,
    /// → in a left-to-right row: open a closed branch, else go to its first child.
    Inward,
    /// ← in a left-to-right row: close an open branch, else go to the parent.
    Outward,
    Enter,
    Space,
    Star,
    Char(char),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeAction {
    Focus(String),
    Expand { id: String, open: bool },
    Select(String),
    ExpandAll(Vec<String>),
    Type(char),
}

/// Drops empty and repeated ids, keeping the first occurrence. Each level's siblings are kept before the children of
/// any of them are looked at, as the React original does.
pub fn normalize(items: &[TreeNode]) -> Vec<TreeNode> {
    let mut seen = HashSet::new();
    normalize_level(items, &mut seen)
}

fn normalize_level(items: &[TreeNode], seen: &mut HashSet<String>) -> Vec<TreeNode> {
    let mut kept: Vec<TreeNode> = items
        .iter()
        .filter(|node| seen.insert(node.id.clone()))
        .cloned()
        .collect();
    for node in &mut kept {
        if let Some(children) = node.children.take() {
            node.children = Some(normalize_level(&children, seen));
        }
    }
    kept
}

/// The rows on screen, top to bottom, for a normalized tree. A branch shows its children only when it is in `open`.
pub fn visible_rows(items: &[TreeNode], open: &[String]) -> Vec<Row> {
    let mut out = Vec::new();
    push_rows(items, 0, None, open, &mut out);
    out
}

fn push_rows(
    list: &[TreeNode],
    depth: usize,
    parent: Option<&str>,
    open: &[String],
    out: &mut Vec<Row>,
) {
    let size = list.len();
    for (i, node) in list.iter().enumerate() {
        let expanded = node.children.is_some() && open.contains(&node.id);
        out.push(Row {
            id: node.id.clone(),
            label: node.label.clone(),
            depth,
            parent: parent.map(str::to_string),
            pos: i + 1,
            size,
            branch: node.children.is_some(),
            expanded,
            disabled: node.disabled,
        });
        if let Some(children) = node.children.as_deref().filter(|_| expanded) {
            push_rows(children, depth + 1, Some(&node.id), open, out);
        }
    }
}

/// The node with `id` anywhere in the tree.
pub fn find_node<'a>(items: &'a [TreeNode], id: &str) -> Option<&'a TreeNode> {
    for node in items {
        if node.id == id {
            return Some(node);
        }
        if let Some(found) = node
            .children
            .as_deref()
            .and_then(|children| find_node(children, id))
        {
            return Some(found);
        }
    }
    None
}

/// The actions for `key` pressed on row `current`. `rows` are the visible rows; disabled rows take no keys and are
/// skipped by the moves. A key on a row that is not focusable does nothing.
pub fn key_actions(key: TreeKey, rows: &[Row], current: &str) -> Vec<TreeAction> {
    let nav: Vec<&Row> = rows.iter().filter(|r| !r.disabled).collect();
    let Some(at) = nav.iter().position(|r| r.id == current) else {
        return Vec::new();
    };
    let row = nav[at];
    match key {
        TreeKey::Down => nav
            .get(at + 1)
            .map(|r| TreeAction::Focus(r.id.clone()))
            .into_iter()
            .collect(),
        TreeKey::Up => at
            .checked_sub(1)
            .map(|i| TreeAction::Focus(nav[i].id.clone()))
            .into_iter()
            .collect(),
        TreeKey::Home => nav
            .first()
            .map(|r| TreeAction::Focus(r.id.clone()))
            .into_iter()
            .collect(),
        TreeKey::End => nav
            .last()
            .map(|r| TreeAction::Focus(r.id.clone()))
            .into_iter()
            .collect(),
        TreeKey::Inward if row.branch => {
            if !row.expanded {
                vec![TreeAction::Expand {
                    id: row.id.clone(),
                    open: true,
                }]
            } else {
                nav.iter()
                    .find(|r| r.parent.as_deref() == Some(row.id.as_str()))
                    .map(|r| TreeAction::Focus(r.id.clone()))
                    .into_iter()
                    .collect()
            }
        }
        TreeKey::Outward => {
            if row.expanded {
                vec![TreeAction::Expand {
                    id: row.id.clone(),
                    open: false,
                }]
            } else {
                row.parent
                    .clone()
                    .map(TreeAction::Focus)
                    .into_iter()
                    .collect()
            }
        }
        TreeKey::Enter => {
            let mut out = vec![TreeAction::Select(row.id.clone())];
            if row.branch {
                out.push(TreeAction::Expand {
                    id: row.id.clone(),
                    open: !row.expanded,
                });
            }
            out
        }
        TreeKey::Space => vec![TreeAction::Select(row.id.clone())],
        TreeKey::Star => {
            let branches: Vec<String> = rows
                .iter()
                .filter(|r| r.parent == row.parent && r.branch && !r.disabled && !r.expanded)
                .map(|r| r.id.clone())
                .collect();
            if branches.is_empty() {
                Vec::new()
            } else {
                vec![TreeAction::ExpandAll(branches)]
            }
        }
        TreeKey::Char(c) => vec![TreeAction::Type(c)],
        TreeKey::Inward => Vec::new(),
    }
}

/// The type-ahead buffer. Keys within 700ms of each other build one query; a pause starts a new one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TypeAhead {
    text: String,
    at: f64,
}

impl TypeAhead {
    /// Adds `key` at time `now` (ms) and returns the lowercase query so far.
    pub fn push(&mut self, key: char, now: f64) -> String {
        let fresh = now - self.at > 700.0;
        if fresh {
            self.text.clear();
        }
        self.text.push(key);
        self.at = now;
        self.text.to_lowercase()
    }
}

/// Index of the row to jump to for `query`, searching on from `from` and wrapping. A query of one letter repeated
/// (`aaa`) steps through the rows that start with that letter, moving past `from`.
pub fn type_ahead_target(rows: &[Row], query: &str, from: usize) -> Option<usize> {
    let first = query.chars().next()?;
    let repeated = query.chars().all(|c| c == first);
    let needle = if repeated {
        first.to_string()
    } else {
        query.to_string()
    };
    let start = if repeated { from + 1 } else { from };
    let n = rows.len();
    (0..n).map(|k| (start + k) % n).find(|&i| {
        rows[i]
            .label
            .trim_start()
            .to_lowercase()
            .starts_with(&needle)
    })
}

/// A tree of folders and files. A row opens and closes with its chevron or with a click, and is selected by a click or
/// Enter. Controlled when `expanded` or `selected` is set; otherwise `default_expanded` and `default_selected` start it.
#[component]
pub fn TreeView(
    items: Vec<TreeNode>,
    #[props(default)] expanded: Option<Vec<String>>,
    #[props(default)] default_expanded: Vec<String>,
    #[props(default)] on_expanded_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] selected: Option<String>,
    #[props(default)] default_selected: Option<String>,
    #[props(default)] on_select: Option<EventHandler<TreeNode>>,
    #[props(default = true)] guides: bool,
    #[props(default = "Empty".to_string(), into)] empty_text: String,
    #[props(default, into)] aria_label: Option<String>,
    #[props(default, into)] aria_labelledby: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let uid = use_hook(next_id);
    let tree_id = format!("ui-tv-{uid}");
    let roots = normalize(&items);

    let inner_open = use_signal(|| default_expanded);
    let open_controlled = expanded.is_some();
    let open = match &expanded {
        Some(open) => open.clone(),
        None => inner_open(),
    };
    let inner_selected = use_signal(|| default_selected);
    let selected_controlled = selected.is_some();
    let selected_now = match &selected {
        Some(id) => Some(id.clone()),
        None => inner_selected(),
    };

    let focused = use_signal(|| None::<String>);
    let typed = use_signal(TypeAhead::default);
    let mut seen = use_signal(Vec::<String>::new);
    let mut rtl = use_signal(|| false);

    let rtl_script = RTL.replace("__TREE__", &tree_id);
    use_effect(move || {
        let script = rtl_script.clone();
        spawn(async move {
            let is_rtl = document::eval(&script)
                .await
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            rtl.set(is_rtl);
        });
    });

    let open_for_seen = open.clone();
    use_effect(use_reactive((&open_for_seen,), move |(open,)| {
        let mut known = seen.write();
        for id in open {
            if !known.contains(&id) {
                known.push(id);
            }
        }
    }));

    let rows = visible_rows(&roots, &open);
    let nav: Vec<Row> = rows.iter().filter(|r| !r.disabled).cloned().collect();
    let stop = focused()
        .filter(|id| nav.iter().any(|r| &r.id == id))
        .or_else(|| {
            selected_now
                .clone()
                .filter(|id| nav.iter().any(|r| &r.id == id))
        })
        .or_else(|| nav.first().map(|r| r.id.clone()));

    let ctx = Ctx {
        roots: roots.clone(),
        open: open.clone(),
        open_controlled,
        inner_open,
        selected_controlled,
        inner_selected,
        on_expanded_change,
        on_select,
        tree_id: tree_id.clone(),
        focused,
        typed,
    };

    let key_ctx = ctx.clone();
    let key_nav = nav.clone();
    let key_rows = rows.clone();
    let on_keys = move |event: KeyboardEvent| {
        let mods = event.modifiers();
        if mods.contains(Modifiers::CONTROL)
            || mods.contains(Modifiers::META)
            || mods.contains(Modifiers::ALT)
        {
            return;
        }
        let Some(current) = key_ctx.focused.peek().clone() else {
            return;
        };
        let is_rtl = rtl();
        let key = match event.key() {
            Key::ArrowDown => TreeKey::Down,
            Key::ArrowUp => TreeKey::Up,
            Key::Home => TreeKey::Home,
            Key::End => TreeKey::End,
            Key::ArrowRight if is_rtl => TreeKey::Outward,
            Key::ArrowRight => TreeKey::Inward,
            Key::ArrowLeft if is_rtl => TreeKey::Inward,
            Key::ArrowLeft => TreeKey::Outward,
            Key::Enter => TreeKey::Enter,
            Key::Character(ref c) if c.as_str() == " " => TreeKey::Space,
            Key::Character(ref c) if c.as_str() == "*" => TreeKey::Star,
            Key::Character(ref c) => {
                let mut chars = c.chars();
                match (chars.next(), chars.next()) {
                    (Some(ch), None) if !ch.is_whitespace() => TreeKey::Char(ch),
                    _ => return,
                }
            }
            _ => return,
        };
        event.prevent_default();
        let at = key_nav.iter().position(|r| r.id == current);
        for action in key_actions(key, &key_rows, &current) {
            match action {
                TreeAction::Focus(id) => key_ctx.focus_row(&id),
                TreeAction::Expand { id, open } => key_ctx.set_open(&id, open),
                TreeAction::Select(id) => key_ctx.select(&id),
                TreeAction::ExpandAll(ids) => {
                    for id in ids {
                        key_ctx.set_open(&id, true);
                    }
                }
                TreeAction::Type(ch) => {
                    if let Some(from) = at {
                        key_ctx.type_ahead(ch, key_nav.clone(), from);
                    }
                }
            }
        }
    };

    let class = class_list("ui-tv", &class);
    let body = render_level(
        &ctx,
        &roots,
        0,
        &mut 0,
        &open,
        &seen(),
        selected_now.as_deref(),
        stop.as_deref(),
        &empty_text,
    );

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        ul {
            id: "{tree_id}",
            class,
            role: "tree",
            aria_label,
            aria_labelledby,
            "data-guides": guides.then_some(""),
            onkeydown: on_keys,
            for child in body {
                {child}
            }
        }
    }
}

#[derive(Clone)]
struct Ctx {
    roots: Vec<TreeNode>,
    open: Vec<String>,
    open_controlled: bool,
    inner_open: Signal<Vec<String>>,
    selected_controlled: bool,
    inner_selected: Signal<Option<String>>,
    on_expanded_change: Option<EventHandler<Vec<String>>>,
    on_select: Option<EventHandler<TreeNode>>,
    tree_id: String,
    focused: Signal<Option<String>>,
    typed: Signal<TypeAhead>,
}

impl Ctx {
    fn set_open(&self, id: &str, to: bool) {
        if self.open.iter().any(|x| x == id) == to {
            return;
        }
        let next: Vec<String> = if to {
            let mut out = self.open.clone();
            out.push(id.to_string());
            out
        } else {
            self.open
                .iter()
                .filter(|x| x.as_str() != id)
                .cloned()
                .collect()
        };
        if !self.open_controlled {
            let mut inner = self.inner_open;
            inner.set(next.clone());
        }
        if let Some(handler) = self.on_expanded_change {
            handler.call(next);
        }
    }

    fn select(&self, id: &str) {
        if !self.selected_controlled {
            let mut inner = self.inner_selected;
            inner.set(Some(id.to_string()));
        }
        if let (Some(handler), Some(node)) = (self.on_select, find_node(&self.roots, id)) {
            handler.call(node.clone());
        }
    }

    fn focus_row(&self, id: &str) {
        let script = FOCUS_ROW
            .replace("__TREE__", &self.tree_id)
            .replace("__ID__", &format!("{id:?}"));
        document::eval(&script);
    }

    fn type_ahead(&self, key: char, nav: Vec<Row>, from: usize) {
        let mut typed = self.typed;
        let ctx = self.clone();
        spawn(async move {
            let now = document::eval("return Date.now();")
                .await
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let query = typed.write().push(key, now);
            if let Some(i) = type_ahead_target(&nav, &query, from) {
                ctx.focus_row(&nav[i].id);
            }
        });
    }
}

/// The rows of one level, each with its open branch below it. `n` numbers the rendered rows for their label ids.
#[allow(clippy::too_many_arguments)]
fn render_level(
    ctx: &Ctx,
    list: &[TreeNode],
    depth: usize,
    n: &mut usize,
    open: &[String],
    seen: &[String],
    selected: Option<&str>,
    stop: Option<&str>,
    empty_text: &str,
) -> Vec<Element> {
    let size = list.len();
    let mut out = Vec::with_capacity(size);
    for (i, node) in list.iter().enumerate() {
        *n += 1;
        let number = *n;
        let branch = node.children.is_some();
        let is_open = branch && open.iter().any(|x| x == &node.id);
        let shows_children = branch && (is_open || seen.iter().any(|x| x == &node.id));
        let children = if shows_children {
            match node.children.as_deref() {
                Some(kids) if !kids.is_empty() => render_level(
                    ctx,
                    kids,
                    depth + 1,
                    n,
                    open,
                    seen,
                    selected,
                    stop,
                    empty_text,
                ),
                _ => vec![empty_row(depth + 1, empty_text)],
            }
        } else {
            Vec::new()
        };
        out.push(item_view(
            ctx,
            node,
            depth,
            i + 1,
            size,
            number,
            branch,
            is_open,
            selected == Some(node.id.as_str()),
            stop == Some(node.id.as_str()),
            children,
        ));
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn item_view(
    ctx: &Ctx,
    node: &TreeNode,
    depth: usize,
    pos: usize,
    size: usize,
    number: usize,
    branch: bool,
    is_open: bool,
    is_selected: bool,
    is_stop: bool,
    children: Vec<Element>,
) -> Element {
    let id = node.id.clone();
    let tree = ctx.tree_id.clone();
    let label_id = format!("{tree}-l{number}");
    let meta_id = format!("{tree}-m{number}");
    let disabled = node.disabled;
    let icon = if is_open {
        node.open_icon.clone().or_else(|| node.icon.clone())
    } else {
        node.icon.clone()
    };
    let meta = node.meta.clone();
    let described_by = meta.is_some().then(|| meta_id.clone());
    let label = node.label.clone();
    let tab = if is_stop { "0" } else { "-1" };
    let depth_style = format!("--ui-tv-d: {depth};");
    let group_style = format!("--ui-tv-d: {};", depth + 1);

    let focus_id = id.clone();
    let focus_ctx = ctx.clone();
    let row_ctx = ctx.clone();
    let row_id = id.clone();
    let chev_ctx = ctx.clone();
    let chev_id = id.clone();

    rsx! {
        li {
            key: "{id}",
            role: "treeitem",
            class: "ui-tv__item",
            "data-id": "{id}",
            aria_level: "{depth + 1}",
            aria_posinset: "{pos}",
            aria_setsize: "{size}",
            aria_expanded: branch.then_some(if is_open { "true" } else { "false" }),
            aria_selected: if is_selected { "true" } else { "false" },
            aria_disabled: disabled.then_some("true"),
            aria_labelledby: "{label_id}",
            aria_describedby: described_by,
            "tabindex": "{tab}",
            onfocus: move |event: FocusEvent| {
                event.stop_propagation();
                let mut focused = focus_ctx.focused;
                focused.set(Some(focus_id.clone()));
            },
            div {
                class: "ui-tv__row",
                style: "{depth_style}",
                onmousedown: move |event: MouseEvent| {
                    if disabled {
                        event.prevent_default();
                    }
                },
                onclick: move |_| {
                    if disabled {
                        return;
                    }
                    row_ctx.select(&row_id);
                    if branch {
                        row_ctx.set_open(&row_id, !is_open);
                    }
                },
                span {
                    class: "ui-tv__chev",
                    "aria-hidden": "true",
                    onclick: move |event: MouseEvent| {
                        if branch {
                            event.stop_propagation();
                            if !disabled {
                                chev_ctx.set_open(&chev_id, !is_open);
                            }
                        }
                    },
                    if branch {
                        {chevron()}
                    }
                }
                if let Some(icon) = icon {
                    span { class: "ui-tv__icon", "aria-hidden": "true", {icon} }
                }
                span { id: "{label_id}", class: "ui-tv__label", dir: "auto", title: "{label}", "{label}" }
                if let Some(meta) = meta {
                    span { id: "{meta_id}", class: "ui-tv__meta", dir: "auto", {meta} }
                }
            }
            if branch {
                div { class: "ui-tv__fold", "data-open": is_open.then_some(""),
                    ul { role: "group", class: "ui-tv__group", style: "{group_style}",
                        for child in children {
                            {child}
                        }
                    }
                }
            }
        }
    }
}

fn empty_row(depth: usize, text: &str) -> Element {
    let style = format!("--ui-tv-d: {depth};");
    rsx! {
        li { role: "none", class: "ui-tv__empty", style: "{style}", "{text}" }
    }
}

/// Hugeicons Free (MIT, © Hugeicons, hugeicons.com), ArrowRight01.
fn chevron() -> Element {
    rsx! {
        svg { width: "12", height: "12", view_box: "0 0 24 24", fill: "none", "aria-hidden": "true",
            path { d: "M9 6c0 0 6 4.42 6 6s-6 6-6 6", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round" }
        }
    }
}
