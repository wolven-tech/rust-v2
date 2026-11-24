//! Workspace frame, inset and flat. One implementation: a collapsible sidebar, a
//! header, the page, and an optional resizable work area on the right. `frame_style`
//! picks the inset look (rounded page panel) or the flat look (edge-to-edge page,
//! hairlines). `workspace_frame_flat` is a thin wrapper that sets `Flat`.
//!
//! Design choice: the frame measures its own width. Below 720px the sidebar and the
//! work area become drawers that trap Tab and close on Escape, and the page takes the
//! full width. Above it they fold to a rail or a header icon.
//! Design choice: the spec's `top_bar` (`{title, actions} | false`) is two props,
//! `top_bar` (bool) and `top_bar_title` / `top_bar_actions`.
//! Design choice: a folded sidebar rail names each icon with a `title` tooltip and an
//! accessible name; no separate tooltip component is used.
//! Design choice: a folded work area is `hidden`, so no focusable element sits in it.
//! Design choice: the work area's first tab is the `side_panel`; "+" opens a tab with
//! start options (tools, sources, open action), not a second copy of the panel.
//! Design choice: `initial_tabs` opens artifact tabs at mount, after the side panel tab.
//! Design choice: below 720px the flat frame moves its header actions into the page
//! more menu; the inset frame keeps them inline.
//! Design choice: the workspace name and the account block are the only places the
//! frame reads account data; the other screens do not depend on the frame.
//! Browser checks: splitter drag and keys within min and max, double-click reset,
//! fold leaves the header icon, "+" opens an active tab, last close folds and focuses
//! the header icon, arrow and Delete on tabs, drawers at 390 and 320, Escape order,
//! side cards toggle, menu navigation, chat row menu and move to project, no
//! horizontal scroll at three widths.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, focus_after_frame, next_id, use_element_width};

const FRAME_CSS: Asset = asset!("/assets/workspace-frame-inset.css");

/// Below this width (CSS px) the sidebar and the work area are drawers.
pub const NARROW_PX: f64 = 720.0;

/// Look of the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FrameStyle {
    #[default]
    Inset,
    Flat,
}

impl FrameStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inset => "inset",
            Self::Flat => "flat",
        }
    }
}

/// A resizable width: its default and its bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidthRange {
    pub default: f64,
    pub min: f64,
    pub max: f64,
}

impl WidthRange {
    pub const fn new(default: f64, min: f64, max: f64) -> Self {
        Self { default, min, max }
    }
}

impl Default for WidthRange {
    fn default() -> Self {
        Self::new(264.0, 200.0, 420.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuIcon {
    #[default]
    Dot,
    Chat,
    Folder,
    Plug,
}

impl MenuIcon {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dot => "dot",
            Self::Chat => "chat",
            Self::Folder => "folder",
            Self::Plug => "plug",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem {
    pub item_label: String,
    pub item_icon: MenuIcon,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuGroup {
    pub title: String,
    pub items: Vec<MenuItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrameAccount {
    pub name: String,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatRow {
    pub id: String,
    pub title: String,
    pub pinned: bool,
    pub unread: bool,
    pub project_id: Option<String>,
    pub section: Option<String>,
}

/// Chats split for the sidebar: pinned first, then the unpinned chats that belong to
/// no project, then the chats in each project in `project_list` order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChatGroups {
    pub pinned: Vec<ChatRow>,
    pub recent: Vec<ChatRow>,
    pub by_project: Vec<(String, Vec<ChatRow>)>,
}

pub fn group_chats(chats: &[ChatRow], projects: &[String]) -> ChatGroups {
    let pinned: Vec<ChatRow> = chats.iter().filter(|c| c.pinned).cloned().collect();
    let recent: Vec<ChatRow> = chats
        .iter()
        .filter(|c| !c.pinned && c.project_id.is_none())
        .cloned()
        .collect();
    let by_project = projects
        .iter()
        .map(|p| {
            let rows = chats
                .iter()
                .filter(|c| !c.pinned && c.project_id.as_deref() == Some(p.as_str()))
                .cloned()
                .collect();
            (p.clone(), rows)
        })
        .collect();
    ChatGroups {
        pinned,
        recent,
        by_project,
    }
}

/// Clamps a width into `[min, max]`.
pub fn clamp_width(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

/// Width after a pointer drag. A work-area edge is dragged leftwards to grow, so it
/// is `invert`ed.
pub fn drag_width(start_width: f64, start_x: f64, x: f64, range: WidthRange, invert: bool) -> f64 {
    let delta = if invert { start_x - x } else { x - start_x };
    clamp_width(start_width + delta, range.min, range.max)
}

/// Width after a key on a focused splitter: arrows step by 16px (48px with Shift),
/// Home and End jump to the bounds. `None` for keys the splitter does not use.
/// `grow_key` is the arrow that grows the pane (`ArrowRight` for the sidebar).
pub fn resize_key(
    current: f64,
    key: &str,
    shift: bool,
    grow_key: &str,
    range: WidthRange,
) -> Option<f64> {
    let step = if shift { 48.0 } else { 16.0 };
    let next = match key {
        "Home" => range.min,
        "End" => range.max,
        k if k == grow_key => current + step,
        k if (k == "ArrowLeft" || k == "ArrowRight") && k != grow_key => current - step,
        _ => return None,
    };
    Some(clamp_width(next, range.min, range.max))
}

/// Width after a double-click on a splitter: back to the default.
pub fn reset_width(range: WidthRange) -> f64 {
    range.default
}

/// DOM id of a frame element: `kit-frame-<uid><suffix>`. Event handlers build ids
/// from `uid` so they capture a copy, not the strings.
fn dom_id(uid: usize, suffix: &str) -> String {
    format!("kit-frame-{uid}{suffix}")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Drawer {
    Sidebar,
    Work,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Splitter {
    Sidebar,
    Work,
}

#[derive(Clone, Debug, PartialEq)]
struct WorkTab {
    id: usize,
    title: String,
    artifact: Option<Artifact>,
}

/// Index of the open tab whose artifact is named `title`. `names` holds each tab's artifact name,
/// `None` for a tab with no artifact. Opening an artifact that is already open activates that tab.
pub fn open_tab_index(names: &[Option<String>], title: &str) -> Option<usize> {
    names.iter().position(|name| name.as_deref() == Some(title))
}

/// Lets content inside the frame, such as the flat Start tab, open an artifact as a work-area tab.
#[derive(Clone, Copy)]
pub struct FrameTabs {
    tabs: Signal<Vec<WorkTab>>,
    active_tab: Signal<usize>,
    next_tab: Signal<usize>,
    work_open: Signal<bool>,
}

impl FrameTabs {
    /// Adds `artifact` as a tab after the open ones and makes it current; an artifact already open
    /// is made current instead of being added twice.
    pub fn open_artifact(self, artifact: Artifact) {
        let mut tabs = self.tabs;
        let mut active_tab = self.active_tab;
        let mut next_tab = self.next_tab;
        let mut work_open = self.work_open;
        let names: Vec<Option<String>> = tabs
            .peek()
            .iter()
            .map(|tab| tab.artifact.as_ref().map(|a| a.name.clone()))
            .collect();
        let id = match open_tab_index(&names, &artifact.name) {
            Some(index) => tabs.peek()[index].id,
            None => {
                let id = next_tab();
                next_tab.set(id + 1);
                let title = artifact.name.clone();
                tabs.with_mut(|list| {
                    list.push(WorkTab {
                        id,
                        title,
                        artifact: Some(artifact),
                    })
                });
                id
            }
        };
        active_tab.set(id);
        work_open.set(true);
    }
}

/// The frame. `content` is the page; `side_panel` is the work area's first tab.
#[component]
pub fn WorkspaceFrame(
    content: Element,
    #[props(default)] frame_style: FrameStyle,
    #[props(default)] current_item: Option<String>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default = "Workspace".to_string(), into)] workspace_name: String,
    #[props(default)] account: Option<FrameAccount>,
    #[props(default)] menu_items: Option<Vec<MenuItem>>,
    #[props(default)] menu_groups: Vec<MenuGroup>,
    #[props(default)] project_list: Vec<String>,
    #[props(default)] chat_list: Vec<ChatRow>,
    #[props(default)] on_open_chat: Option<EventHandler<ChatRow>>,
    #[props(default)] sidebar_width: WidthRange,
    #[props(default = true)] top_bar: bool,
    #[props(default)] top_bar_title: Option<Element>,
    #[props(default)] top_bar_actions: Option<Element>,
    #[props(default = true)] side_cards: bool,
    #[props(default)] work_area: bool,
    #[props(default)] side_panel: Option<Element>,
    #[props(default = "Workspace".to_string(), into)] panel_title: String,
    #[props(default)] panel_heading: Option<String>,
    #[props(default = true)] default_panel_open: bool,
    #[props(default = WidthRange::new(340.0, 260.0, 560.0))] panel_width: WidthRange,
    #[props(default)] artifacts: Vec<Artifact>,
    #[props(default)] on_open_artifact: Option<EventHandler<String>>,
    #[props(default)] initial_tabs: Vec<Artifact>,
    #[props(default, into)] class_name: String,
) -> Element {
    let uid = use_hook(|| next_id());
    let frame_id = format!("kit-frame-{uid}");
    let sidebar_id = format!("{frame_id}-sidebar");
    let work_id = format!("{frame_id}-work");
    let sidebar_toggle_id = format!("{frame_id}-sidebar-toggle");
    let work_open_id = format!("{frame_id}-work-open");

    let width_now = use_element_width(frame_id.clone());
    let narrow = width_now.is_some_and(|w| w < NARROW_PX);

    let mut sidebar_open = use_signal(|| true);
    let mut work_open = use_signal(|| default_panel_open);
    let mut drawer = use_signal(|| None::<Drawer>);
    let mut sidebar_w = use_signal(|| sidebar_width.default);
    let mut work_w = use_signal(|| panel_width.default);
    let mut splitter_drag = use_signal(|| None::<(Splitter, f64, f64)>);
    let mut account_open = use_signal(|| false);
    let mut folded = use_signal(Vec::<String>::new);
    let mut chats = use_signal(|| chat_list.clone());
    let mut row_menu = use_signal(|| None::<String>);
    let mut move_open = use_signal(|| false);
    let mut dragging = use_signal(|| None::<String>);
    let mut selected = use_signal(|| current_item.clone());
    let mut tabs = use_signal(|| {
        let mut list = vec![WorkTab {
            id: 0,
            title: panel_heading.clone().unwrap_or_else(|| panel_title.clone()),
            artifact: None,
        }];
        list.extend(initial_tabs.iter().enumerate().map(|(i, art)| WorkTab {
            id: i + 1,
            title: art.name.clone(),
            artifact: Some(art.clone()),
        }));
        list
    });
    let mut active_tab = use_signal(|| if initial_tabs.is_empty() { 0usize } else { 1 });
    let mut next_tab = use_signal(|| initial_tabs.len() + 1);
    use_context_provider(|| FrameTabs {
        tabs,
        active_tab,
        next_tab,
        work_open,
    });

    {
        let sidebar_id = sidebar_id.clone();
        let work_id = work_id.clone();
        use_effect(move || {
            let target = match drawer() {
                Some(Drawer::Sidebar) => Some(sidebar_id.clone()),
                Some(Drawer::Work) => Some(work_id.clone()),
                None => None,
            };
            match target {
                Some(id) => {
                    document::eval(&trap_script(&id));
                }
                None => {
                    document::eval("window.__kitTrapRemove?.(); window.__kitTrapRemove = null;");
                }
            }
        });
    }

    let style_now = frame_style;
    let collapse_actions = narrow && style_now == FrameStyle::Flat;
    let sidebar_mode = if sidebar_open() { "open" } else { "folded" };
    let work_mode = if work_open() { "open" } else { "folded" };
    let drawer_mode = match drawer() {
        Some(Drawer::Sidebar) => "sidebar",
        Some(Drawer::Work) => "work",
        None => "none",
    };
    let work_hidden = if narrow {
        drawer() != Some(Drawer::Work)
    } else {
        !work_open()
    };
    let sidebar_inert = narrow && drawer() != Some(Drawer::Sidebar);
    let sidebar_style = format!(
        "--kit-frame-sidebar: {}px; --kit-frame-sidebar-min: {}px; --kit-frame-sidebar-max: {}px;",
        sidebar_w(),
        sidebar_width.min,
        sidebar_width.max
    );
    let scale = width_now.map_or(1.0, |w| (w / 1280.0).clamp(1.0, 1.5));
    let work_style = format!("--kit-frame-work: {}px;", work_w() * scale);
    let root_style = format!("{sidebar_style} {work_style}");
    let class = if class_name.is_empty() {
        "kit-frame".to_string()
    } else {
        format!("kit-frame {class_name}")
    };

    let groups = group_chats(&chats(), &project_list);
    let menu: Vec<MenuItem> = menu_items.clone().unwrap_or_else(|| {
        vec![
            MenuItem {
                item_label: "New chat".to_string(),
                item_icon: MenuIcon::Chat,
            },
            MenuItem {
                item_label: "Folders".to_string(),
                item_icon: MenuIcon::Folder,
            },
            MenuItem {
                item_label: "Integrations".to_string(),
                item_icon: MenuIcon::Plug,
            },
        ]
    });
    let letter = workspace_name
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect::<String>())
        .unwrap_or_default();
    let account_name = account.as_ref().map(|a| a.name.clone()).unwrap_or_default();
    let account_initial = account_name
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect::<String>())
        .unwrap_or_default();

    let work_title = panel_heading.clone().unwrap_or_else(|| panel_title.clone());
    let reset_title = work_title.clone();
    let open_label = format!("Open {panel_title}");
    let toggle_sidebar_label = if sidebar_open() {
        "Fold sidebar"
    } else {
        "Open sidebar"
    };
    let page_name = current_item.clone().unwrap_or_default();
    let page_title = top_bar_title
        .clone()
        .or_else(|| current_item.clone().map(|name| rsx! { "{name}" }));
    let mut page_menu = use_signal(|| false);
    let more_label = format!("More actions for {page_name}");

    let mut go = move |label: String| {
        selected.set(Some(label.clone()));
        if let Some(h) = &on_navigate {
            h.call(label);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: FRAME_CSS }
        div {
            id: "{frame_id}",
            class,
            "data-style": style_now.as_str(),
            "data-sidebar": sidebar_mode,
            "data-work": work_mode,
            "data-drawer": drawer_mode,
            style: "{root_style}",
            onpointermove: move |e: PointerEvent| {
                if let Some((which, start_x, start_w)) = splitter_drag() {
                    let x = e.client_coordinates().x;
                    match which {
                        Splitter::Sidebar => sidebar_w.set(drag_width(start_w, start_x, x, sidebar_width, false)),
                        Splitter::Work => work_w.set(drag_width(start_w, start_x / scale, x / scale, panel_width, true)),
                    }
                }
            },
            onpointerup: move |_| splitter_drag.set(None),
            onpointerleave: move |_| splitter_drag.set(None),
            onkeydown: move |e: KeyboardEvent| {
                if e.key() != Key::Escape {
                    return;
                }
                if account_open() {
                    e.stop_propagation();
                    account_open.set(false);
                    focus_after_frame(&dom_id(uid, "-account"));
                } else if narrow && drawer().is_some() {
                    e.stop_propagation();
                    let target = if drawer() == Some(Drawer::Work) {
                        dom_id(uid, "-work-open")
                    } else {
                        dom_id(uid, "-sidebar-toggle")
                    };
                    drawer.set(None);
                    focus_after_frame(&target);
                }
            },

            if top_bar {
                header { class: "kit-frame__bar",
                    button {
                        id: "{sidebar_toggle_id}",
                        r#type: "button",
                        class: "kit-frame__icon-btn",
                        "aria-label": toggle_sidebar_label,
                        "aria-expanded": (sidebar_open() || drawer() == Some(Drawer::Sidebar)).to_string(),
                        "aria-controls": "{sidebar_id}",
                        onclick: move |_| {
                            if narrow {
                                drawer.set(if drawer() == Some(Drawer::Sidebar) { None } else { Some(Drawer::Sidebar) });
                            } else {
                                sidebar_open.set(!sidebar_open());
                            }
                        },
                        FrameIcon { name: "rail" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-frame__icon-btn",
                        "aria-label": "Search",
                        FrameIcon { name: "search" }
                    }
                    if let Some(title) = page_title {
                        h1 { class: "kit-frame__title", {title} }
                    }
                    div { class: "kit-frame__bar-end",
                        div { class: "kit-frame__actions",
                            if !collapse_actions {
                                if let Some(actions) = top_bar_actions.clone() {
                                    {actions}
                                } else {
                                    button {
                                        r#type: "button",
                                        class: "kit-frame__text-btn",
                                        "aria-label": "Share",
                                        FrameIcon { name: "share" }
                                        span { "Share" }
                                    }
                                }
                            }
                            if collapse_actions || top_bar_actions.is_none() {
                                button {
                                    id: "{frame_id}-page-more",
                                    r#type: "button",
                                    class: "kit-frame__icon-btn",
                                    "aria-label": "{more_label}",
                                    "aria-haspopup": "menu",
                                    "aria-expanded": page_menu().to_string(),
                                    onclick: move |_| page_menu.set(!page_menu()),
                                    FrameIcon { name: "more" }
                                }
                                if page_menu() {
                                    div {
                                        class: "kit-frame__menu kit-frame__page-menu",
                                        role: "menu",
                                        aria_label: "{page_name} actions",
                                        onkeydown: move |e: KeyboardEvent| {
                                            if e.key() == Key::Escape {
                                                e.stop_propagation();
                                                page_menu.set(false);
                                                focus_after_frame(&dom_id(uid, "-page-more"));
                                            }
                                        },
                                        if collapse_actions {
                                            if let Some(actions) = top_bar_actions.clone() {
                                                div { class: "kit-frame__menu-actions", {actions} }
                                            } else {
                                                button {
                                                    r#type: "button",
                                                    role: "menuitem",
                                                    class: "kit-frame__menu-item",
                                                    onclick: move |_| {
                                                        page_menu.set(false);
                                                        if let Some(h) = &on_navigate {
                                                            h.call("Share".to_string());
                                                        }
                                                    },
                                                    "Share"
                                                }
                                            }
                                        }
                                        if top_bar_actions.is_none() {
                                            for entry in ["Rename", "Duplicate", "Delete"] {
                                                button {
                                                    key: "{entry}",
                                                    r#type: "button",
                                                    role: "menuitem",
                                                    class: "kit-frame__menu-item",
                                                    onclick: move |_| {
                                                        page_menu.set(false);
                                                        if let Some(h) = &on_navigate {
                                                            h.call(entry.to_string());
                                                        }
                                                    },
                                                    "{entry}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if work_area && !(narrow && drawer() == Some(Drawer::Work)) && (!work_open() || narrow) {
                            button {
                                id: "{work_open_id}",
                                r#type: "button",
                                class: "kit-frame__icon-btn kit-frame__work-open",
                                "aria-label": open_label.clone(),
                                "aria-expanded": (work_open() || drawer() == Some(Drawer::Work)).to_string(),
                                "aria-controls": "{work_id}",
                                onclick: move |_| {
                                    if narrow {
                                        drawer.set(Some(Drawer::Work));
                                        document::eval(&format!(
                                            "setTimeout(() => document.getElementById({:?})?.focus({{ preventScroll: true }}), 30);",
                                            dom_id(uid, &format!("-tab-{}", active_tab()))
                                        ));
                                    } else {
                                        work_open.set(true);
                                        if tabs().is_empty() {
                                            tabs.set(vec![WorkTab { id: 0, title: reset_title.clone(), artifact: None }]);
                                            active_tab.set(0);
                                        }
                                    }
                                },
                                FrameIcon { name: "panel" }
                            }
                        }
                    }
                }
            }

            if narrow && drawer().is_some() {
                div {
                    class: "kit-frame__scrim",
                    onclick: move |_| {
                        let was = drawer();
                        drawer.set(None);
                        let target = match was {
                            Some(Drawer::Work) => dom_id(uid, "-work-open"),
                            _ => dom_id(uid, "-sidebar-toggle"),
                        };
                        focus_after_frame(&target);
                    },
                }
            }

            div { class: "kit-frame__body",
                aside {
                    id: "{sidebar_id}",
                    class: "kit-frame__sidebar",
                    "aria-label": "Sidebar",
                    "inert": sidebar_inert.then_some(""),
                    div { class: "kit-frame__ws",
                        span { class: "kit-frame__mark", aria_hidden: "true", "{letter}" }
                        span { class: "kit-frame__label kit-frame__ws-name", "{workspace_name}" }
                    }
                    nav { class: "kit-frame__nav", aria_label: "Main menu",
                        ul { class: "kit-frame__list",
                            for item in menu.clone() {
                                li { key: "{item.item_label}",
                                    button {
                                        r#type: "button",
                                        class: "kit-frame__item",
                                        title: "{item.item_label}",
                                        aria_current: (selected().as_deref() == Some(item.item_label.as_str())).then_some("true"),
                                        onclick: {
                                            let label = item.item_label.clone();
                                            move |_| go(label.clone())
                                        },
                                        FrameIcon { name: item.item_icon.as_str() }
                                        span { class: "kit-frame__label", "{item.item_label}" }
                                    }
                                }
                            }
                        }
                    }
                    for group in menu_groups.clone() {
                        section { key: "{group.title}", class: "kit-frame__section", aria_label: "{group.title}",
                            h2 { class: "kit-frame__heading kit-frame__label", "{group.title}" }
                            ul { class: "kit-frame__list",
                                for item in group.items.clone() {
                                    li { key: "{item.item_label}",
                                        button {
                                            r#type: "button",
                                            class: "kit-frame__item",
                                            title: "{item.item_label}",
                                            aria_current: (selected().as_deref() == Some(item.item_label.as_str())).then_some("true"),
                                            onclick: {
                                                let label = item.item_label.clone();
                                                move |_| go(label.clone())
                                            },
                                            FrameIcon { name: item.item_icon.as_str() }
                                            span { class: "kit-frame__label", "{item.item_label}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if menu_groups.is_empty() && !project_list.is_empty() && chat_list.is_empty() {
                        section { class: "kit-frame__section", aria_label: "Projects",
                            h2 { class: "kit-frame__heading kit-frame__label", "Projects" }
                            ul { class: "kit-frame__list",
                                for project in project_list.clone() {
                                    li { key: "{project}",
                                        button {
                                            r#type: "button",
                                            class: "kit-frame__item",
                                            title: "{project}",
                                            onclick: {
                                                let label = project.clone();
                                                move |_| go(label.clone())
                                            },
                                            FrameIcon { name: "folder" }
                                            span { class: "kit-frame__label", "{project}" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !chat_list.is_empty() {
                        if !groups.pinned.is_empty() {
                            section { class: "kit-frame__section", aria_label: "Pinned",
                                h2 { class: "kit-frame__heading kit-frame__label", "Pinned" }
                                ul { class: "kit-frame__list",
                                    for chat in groups.pinned.clone() {
                                        ChatRowView {
                                            key: "{chat.id}",
                                            chat: chat.clone(),
                                            menu_open: row_menu() == Some(chat.id.clone()),
                                            projects: project_list.clone(),
                                            on_toggle_menu: move |id: String| {
                                                row_menu.set(if row_menu() == Some(id.clone()) { None } else { Some(id) });
                                                move_open.set(false);
                                            },
                                            on_close_menu: move |id: String| {
                                                row_menu.set(None);
                                                move_open.set(false);
                                                focus_after_frame(&dom_id(uid, &format!("-more-{id}")));
                                            },
                                            on_open: move |chat: ChatRow| {
                                                selected.set(Some(chat.title.clone()));
                                                if let Some(h) = &on_open_chat {
                                                    h.call(chat);
                                                } else if let Some(h) = &on_navigate {
                                                    h.call(chat.title.clone());
                                                }
                                            },
                                            on_action: move |(id, action): (String, String)| {
                                                row_menu.set(None);
                                                move_open.set(false);
                                                chats.with_mut(|list| {
                                                    if let Some(c) = list.iter_mut().find(|c| c.id == id) {
                                                        match action.as_str() {
                                                            "pin" => c.pinned = !c.pinned,
                                                            "unread" => c.unread = !c.unread,
                                                            _ => {}
                                                        }
                                                    }
                                                });
                                            },
                                            on_move: move |(id, project): (String, Option<String>)| {
                                                row_menu.set(None);
                                                move_open.set(false);
                                                chats.with_mut(|list| {
                                                    if let Some(c) = list.iter_mut().find(|c| c.id == id) {
                                                        c.project_id = project.clone();
                                                    }
                                                });
                                            },
                                            frame_prefix: frame_id.clone(),
                                            move_open: move_open(),
                                            on_move_open: move |open: bool| move_open.set(open),
                                            on_drag_start: move |id: String| dragging.set(Some(id)),
                                        }
                                    }
                                }
                            }
                        }
                        for (project, rows) in groups.by_project.clone() {
                            {
                                let is_open = !folded().contains(&project);
                                let folder_id = format!("{frame_id}-folder-{project}");
                                let toggle_project = project.clone();
                                let drop_project = project.clone();
                                rsx! {
                                    section { key: "{project}", class: "kit-frame__section kit-frame__folder", aria_label: "{project} folder",
                                        ondragover: move |e: DragEvent| e.prevent_default(),
                                        ondrop: move |e: DragEvent| {
                                            e.prevent_default();
                                            if let Some(id) = dragging() {
                                                let target = drop_project.clone();
                                                chats.with_mut(|list| {
                                                    if let Some(c) = list.iter_mut().find(|c| c.id == id) {
                                                        c.project_id = Some(target.clone());
                                                    }
                                                });
                                            }
                                            dragging.set(None);
                                        },
                                        button {
                                            r#type: "button",
                                            class: "kit-frame__item kit-frame__folder-toggle",
                                            id: "{folder_id}",
                                            aria_expanded: is_open.to_string(),
                                            aria_controls: "{folder_id}-list",
                                            onclick: move |_| {
                                                let name = toggle_project.clone();
                                                folded.with_mut(|list| {
                                                    if let Some(pos) = list.iter().position(|x| *x == name) {
                                                        list.remove(pos);
                                                    } else {
                                                        list.push(name);
                                                    }
                                                });
                                            },
                                            FrameIcon { name: "folder" }
                                            span { class: "kit-frame__label", "{project}" }
                                        }
                                        if is_open {
                                            ul { id: "{folder_id}-list", class: "kit-frame__list kit-frame__children",
                                                for chat in rows.clone() {
                                                    ChatRowView {
                                                        key: "{chat.id}",
                                                        chat: chat.clone(),
                                                        menu_open: row_menu() == Some(chat.id.clone()),
                                                        projects: project_list.clone(),
                                                        on_toggle_menu: move |id: String| {
                                                            row_menu.set(if row_menu() == Some(id.clone()) { None } else { Some(id) });
                                                            move_open.set(false);
                                                        },
                                                        on_close_menu: move |id: String| {
                                                            row_menu.set(None);
                                                            move_open.set(false);
                                                            focus_after_frame(&dom_id(uid, &format!("-more-{id}")));
                                                        },
                                                        on_open: move |chat: ChatRow| {
                                                            selected.set(Some(chat.title.clone()));
                                                            if let Some(h) = &on_open_chat {
                                                                h.call(chat);
                                                            } else if let Some(h) = &on_navigate {
                                                                h.call(chat.title.clone());
                                                            }
                                                        },
                                                        on_action: move |(id, action): (String, String)| {
                                                            row_menu.set(None);
                                                            move_open.set(false);
                                                            chats.with_mut(|list| {
                                                                if let Some(c) = list.iter_mut().find(|c| c.id == id) {
                                                                    match action.as_str() {
                                                                        "pin" => c.pinned = !c.pinned,
                                                                        "unread" => c.unread = !c.unread,
                                                                        _ => {}
                                                                    }
                                                                }
                                                            });
                                                        },
                                                        on_move: move |(id, project): (String, Option<String>)| {
                                                            row_menu.set(None);
                                                            move_open.set(false);
                                                            chats.with_mut(|list| {
                                                                if let Some(c) = list.iter_mut().find(|c| c.id == id) {
                                                                    c.project_id = project.clone();
                                                                }
                                                            });
                                                        },
                                                        frame_prefix: frame_id.clone(),
                                                        move_open: move_open(),
                                                        on_move_open: move |open: bool| move_open.set(open),
                                                        on_drag_start: move |id: String| dragging.set(Some(id)),
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if !groups.recent.is_empty() {
                            section { class: "kit-frame__section", aria_label: "Chats",
                                h2 { class: "kit-frame__heading kit-frame__label", "Chats" }
                                ul { class: "kit-frame__list",
                                    for chat in groups.recent.clone() {
                                        ChatRowView {
                                            key: "{chat.id}",
                                            chat: chat.clone(),
                                            menu_open: row_menu() == Some(chat.id.clone()),
                                            projects: project_list.clone(),
                                            on_toggle_menu: move |id: String| {
                                                row_menu.set(if row_menu() == Some(id.clone()) { None } else { Some(id) });
                                                move_open.set(false);
                                            },
                                            on_close_menu: move |id: String| {
                                                row_menu.set(None);
                                                move_open.set(false);
                                                focus_after_frame(&dom_id(uid, &format!("-more-{id}")));
                                            },
                                            on_open: move |chat: ChatRow| {
                                                selected.set(Some(chat.title.clone()));
                                                if let Some(h) = &on_open_chat {
                                                    h.call(chat);
                                                } else if let Some(h) = &on_navigate {
                                                    h.call(chat.title.clone());
                                                }
                                            },
                                            on_action: move |(id, action): (String, String)| {
                                                row_menu.set(None);
                                                move_open.set(false);
                                                chats.with_mut(|list| {
                                                    if let Some(c) = list.iter_mut().find(|c| c.id == id) {
                                                        match action.as_str() {
                                                            "pin" => c.pinned = !c.pinned,
                                                            "unread" => c.unread = !c.unread,
                                                            _ => {}
                                                        }
                                                    }
                                                });
                                            },
                                            on_move: move |(id, project): (String, Option<String>)| {
                                                row_menu.set(None);
                                                move_open.set(false);
                                                chats.with_mut(|list| {
                                                    if let Some(c) = list.iter_mut().find(|c| c.id == id) {
                                                        c.project_id = project.clone();
                                                    }
                                                });
                                            },
                                            frame_prefix: frame_id.clone(),
                                            move_open: move_open(),
                                            on_move_open: move |open: bool| move_open.set(open),
                                            on_drag_start: move |id: String| dragging.set(Some(id)),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if side_cards {
                        div { class: "kit-frame__cards",
                            section { class: "kit-frame__card", aria_label: "What's new",
                                h2 { class: "kit-frame__card-title", "What's new" }
                                p { class: "kit-frame__card-body", "Shared chats now keep their folder when you move them." }
                            }
                            section { class: "kit-frame__card", aria_label: "Getting started",
                                h2 { class: "kit-frame__card-title", "Getting started" }
                                p { class: "kit-frame__card-body", "Add a project, invite a teammate, and connect one service." }
                            }
                        }
                    }

                    div { class: "kit-frame__account",
                        button {
                            id: "{frame_id}-account",
                            r#type: "button",
                            class: "kit-frame__account-btn",
                            "aria-haspopup": "menu",
                            "aria-expanded": account_open().to_string(),
                            "aria-label": "Account menu for {account_name}",
                            onclick: move |_| account_open.set(!account_open()),
                            span { class: "kit-frame__avatar", aria_hidden: "true",
                                if let Some(url) = account.as_ref().and_then(|a| a.avatar_url.clone()) {
                                    img { src: "{url}", alt: "" }
                                } else {
                                    "{account_initial}"
                                }
                            }
                            span { class: "kit-frame__label", "{account_name}" }
                        }
                        if account_open() {
                            div { class: "kit-frame__menu", role: "menu", aria_label: "Account",
                                if let Some(acc) = account.clone() {
                                    div { class: "kit-frame__menu-head",
                                        span { class: "kit-frame__menu-name", "{acc.name}" }
                                        if let Some(email) = acc.email.clone() {
                                            span { class: "kit-frame__menu-email", "{email}" }
                                        }
                                    }
                                }
                                for entry in ["Language", "Theme", "Connected services", "Preferences", "Installed apps", "Release notes", "Help", "Sign out"] {
                                    button {
                                        key: "{entry}",
                                        r#type: "button",
                                        role: "menuitem",
                                        class: "kit-frame__menu-item",
                                        onclick: move |_| {
                                            account_open.set(false);
                                            if let Some(h) = &on_navigate {
                                                h.call(entry.to_string());
                                            }
                                        },
                                        "{entry}"
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "kit-frame__split kit-frame__split--sidebar",
                    role: "separator",
                    tabindex: "0",
                    aria_orientation: "vertical",
                    aria_label: "Resize sidebar",
                    aria_valuemin: "{sidebar_width.min}",
                    aria_valuemax: "{sidebar_width.max}",
                    aria_valuenow: "{sidebar_w()}",
                    onpointerdown: move |e: PointerEvent| {
                        splitter_drag.set(Some((Splitter::Sidebar, e.client_coordinates().x, sidebar_w())));
                    },
                    ondoubleclick: move |_| sidebar_w.set(reset_width(sidebar_width)),
                    onkeydown: move |e: KeyboardEvent| {
                        if let Some(next) = resize_key(sidebar_w(), &e.key().to_string(), e.modifiers().shift(), "ArrowRight", sidebar_width) {
                            sidebar_w.set(next);
                        }
                    },
                }

                main { class: "kit-frame__page", "data-folded-work": (!work_open() && !narrow).then_some("true"),
                    div { class: "kit-frame__page-inner",
                        {content}
                    }
                }

                if work_area {
                    div {
                        class: "kit-frame__split kit-frame__split--work",
                        role: "separator",
                        tabindex: if work_hidden { "-1" } else { "0" },
                        aria_orientation: "vertical",
                        aria_label: "Resize work area",
                        aria_valuemin: "{panel_width.min}",
                        aria_valuemax: "{panel_width.max}",
                        aria_valuenow: "{work_w()}",
                        hidden: work_hidden,
                        onpointerdown: move |e: PointerEvent| {
                            splitter_drag.set(Some((Splitter::Work, e.client_coordinates().x, work_w())));
                        },
                        ondoubleclick: move |_| work_w.set(reset_width(panel_width)),
                        onkeydown: move |e: KeyboardEvent| {
                            if let Some(next) = resize_key(work_w(), &e.key().to_string(), e.modifiers().shift(), "ArrowLeft", panel_width) {
                                work_w.set(next);
                            }
                        },
                    }
                    aside {
                        id: "{work_id}",
                        class: "kit-frame__work",
                        role: "region",
                        aria_label: "{work_title}",
                        hidden: work_hidden,
                        div { class: "kit-frame__work-head",
                            div {
                                class: "kit-frame__tablist",
                                role: "tablist",
                                aria_label: "Open tabs",
                                onkeydown: move |e: KeyboardEvent| {
                                    let list = tabs();
                                    if list.is_empty() {
                                        return;
                                    }
                                    let at = list.iter().position(|t| t.id == active_tab()).unwrap_or(0);
                                    match e.key() {
                                        Key::ArrowRight | Key::ArrowLeft => {
                                            let step: isize = if e.key() == Key::ArrowRight { 1 } else { -1 };
                                            let next = (at as isize + step).rem_euclid(list.len() as isize) as usize;
                                            active_tab.set(list[next].id);
                                            focus_after_frame(&dom_id(uid, &format!("-tab-{}", list[next].id)));
                                        }
                                        Key::Delete | Key::Backspace => {
                                            let closing = list[at].id;
                                            let neighbour = if at + 1 < list.len() { at + 1 } else { at.saturating_sub(1) };
                                            let remaining: Vec<WorkTab> = list.iter().filter(|t| t.id != closing).cloned().collect();
                                            if remaining.is_empty() {
                                                tabs.set(Vec::new());
                                                work_open.set(false);
                                                focus_after_frame(&dom_id(uid, "-work-open"));
                                            } else {
                                                let target = list[neighbour].id;
                                                tabs.set(remaining);
                                                active_tab.set(target);
                                                focus_after_frame(&dom_id(uid, &format!("-tab-{target}")));
                                            }
                                        }
                                        _ => {}
                                    }
                                },
                                for tab in tabs() {
                                    {
                                        let tab_id = tab.id;
                                        let is_active = active_tab() == tab_id;
                                        let close_label = format!("Close {}", tab.title);
                                        rsx! {
                                            div { key: "{tab.id}", class: "kit-frame__tab", "data-active": is_active.then_some("true"),
                                                button {
                                                    id: "{frame_id}-tab-{tab_id}",
                                                    r#type: "button",
                                                    role: "tab",
                                                    class: "kit-frame__tab-btn",
                                                    aria_selected: is_active.to_string(),
                                                    aria_controls: "{frame_id}-panel",
                                                    tabindex: if is_active { "0" } else { "-1" },
                                                    onclick: move |_| active_tab.set(tab_id),
                                                    span { class: "kit-frame__tab-title", "{tab.title}" }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "kit-frame__tab-close",
                                                    "aria-label": "{close_label}",
                                                    onclick: move |_| {
                                                        let list = tabs();
                                                        let remaining: Vec<WorkTab> = list.iter().filter(|t| t.id != tab_id).cloned().collect();
                                                        if remaining.is_empty() {
                                                            tabs.set(Vec::new());
                                                            work_open.set(false);
                                                            focus_after_frame(&dom_id(uid, "-work-open"));
                                                        } else {
                                                            if active_tab() == tab_id {
                                                                active_tab.set(remaining[0].id);
                                                            }
                                                            tabs.set(remaining);
                                                        }
                                                    },
                                                    span { aria_hidden: "true", "×" }
                                                }
                                            }
                                        }
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "kit-frame__icon-btn kit-frame__new-tab",
                                    "aria-label": "New tab",
                                    onclick: move |_| {
                                        let id = next_tab();
                                        next_tab.set(id + 1);
                                        tabs.with_mut(|list| list.push(WorkTab { id, title: format!("New tab {id}"), artifact: None }));
                                        active_tab.set(id);
                                    },
                                    span { aria_hidden: "true", "+" }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "kit-frame__icon-btn kit-frame__work-fold",
                                "aria-label": "Fold {panel_title}",
                                onclick: move |_| {
                                    if narrow {
                                        drawer.set(None);
                                        focus_after_frame(&dom_id(uid, "-work-open"));
                                    } else {
                                        work_open.set(false);
                                        focus_after_frame(&dom_id(uid, "-work-open"));
                                    }
                                },
                                FrameIcon { name: "close" }
                            }
                        }
                        div { id: "{frame_id}-panel", role: "tabpanel", class: "kit-frame__work-body",
                            if tabs().is_empty() {
                                div { class: "kit-frame__work-empty",
                                    p { class: "kit-frame__card-body", "No tabs are open." }
                                }
                            }
                            for tab in tabs() {
                                if tab.id == active_tab() {
                                    if let Some(art) = tab.artifact.clone() {
                                        div { key: "{tab.id}", class: "kit-frame__panel-content",
                                            ArtifactView { artifact: art }
                                        }
                                    } else if tab.id == 0 {
                                        div { key: "{tab.id}", class: "kit-frame__panel-content", {side_panel.clone()} }
                                    } else {
                                        div { key: "{tab.id}", class: "kit-frame__start",
                                            h2 { class: "kit-frame__card-title", "Start a tab" }
                                            button { r#type: "button", class: "kit-frame__item", "Tools" }
                                            button { r#type: "button", class: "kit-frame__item", "Sources" }
                                            button { r#type: "button", class: "kit-frame__item", "Open an action" }
                                            for art in artifacts.clone() {
                                                button {
                                                    key: "{art.name}",
                                                    r#type: "button",
                                                    class: "kit-frame__item",
                                                    onclick: {
                                                        let tab_id = tab.id;
                                                        let art = art.clone();
                                                        move |_| {
                                                            let title = art.name.clone();
                                                            if let Some(h) = &on_open_artifact {
                                                                h.call(title.clone());
                                                            }
                                                            let opened = art.clone();
                                                            tabs.with_mut(|list| {
                                                                if let Some(t) = list.iter_mut().find(|t| t.id == tab_id) {
                                                                    t.title = title.clone();
                                                                    t.artifact = Some(opened.clone());
                                                                }
                                                            });
                                                        }
                                                    },
                                                    "{art.name}"
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

/// Outline icon for the frame's controls. Decorative: the control it sits in carries the name.
#[component]
fn FrameIcon(name: &'static str) -> Element {
    let d = match name {
        "chat" => {
            "M2.5 4.5a2 2 0 0 1 2-2h7a2 2 0 0 1 2 2v4.5a2 2 0 0 1-2 2H7l-3 2.5v-2.5h-.5a2 2 0 0 1-2-2z"
        }
        "folder" => {
            "M2 4.5a1.5 1.5 0 0 1 1.5-1.5h2.7l1.3 1.6h5a1.5 1.5 0 0 1 1.5 1.5v5.4a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z"
        }
        "plug" => "M6 2v3M10 2v3M4.5 5h7v2.5a3.5 3.5 0 0 1-7 0zM8 11v3",
        "rail" | "panel" => "M2.5 3h11v10h-11zM6.5 3v10",
        "search" => "M7 2.5a4.5 4.5 0 1 0 0 9 4.5 4.5 0 0 0 0-9zM10.5 10.5 14 14",
        "close" => "M4 4l8 8M12 4l-8 8",
        "share" => "M8 2v8M5 5l3-3 3 3M3 9v4h10V9",
        "more" => "M3.5 8h.01M8 8h.01M12.5 8h.01",
        "dot" => "M8 6.5a1.5 1.5 0 1 0 0 3 1.5 1.5 0 0 0 0-3z",
        _ => "",
    };
    rsx! {
        svg {
            class: "kit-frame__glyph",
            "data-icon": name,
            view_box: "0 0 16 16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "{d}" }
        }
    }
}

/// A file opened as an artifact tab. `kind` picks the viewer; `body` is the content
/// (paragraphs, rows, lines, code, or an image URL).
#[derive(Clone, Debug, PartialEq)]
pub struct Artifact {
    pub name: String,
    pub kind: ArtifactKind,
    pub body: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ArtifactKind {
    #[default]
    Document,
    Sheet,
    Folder,
    Code,
    Image,
}

impl ArtifactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Sheet => "sheet",
            Self::Folder => "folder",
            Self::Code => "code",
            Self::Image => "image",
        }
    }
}

/// The viewer for an artifact tab, chosen by its kind.
#[component]
pub(crate) fn ArtifactView(artifact: Artifact) -> Element {
    let kind = artifact.kind.as_str();
    rsx! {
        article { class: "kit-frame__artifact", "data-kind": kind, aria_label: "{artifact.name}",
            {match artifact.kind {
                ArtifactKind::Document => rsx! {
                    for para in artifact.body.split("\n\n").map(str::to_string).collect::<Vec<_>>() {
                        p { key: "{para}", class: "kit-frame__card-body", "{para}" }
                    }
                },
                ArtifactKind::Sheet => rsx! {
                    table { class: "kit-frame__sheet",
                        tbody {
                            for (row, line) in artifact.body.lines().enumerate() {
                                tr { key: "{row}",
                                    for cell in line.split(',').map(|c| c.trim().to_string()).collect::<Vec<_>>() {
                                        td { "{cell}" }
                                    }
                                }
                            }
                        }
                    }
                },
                ArtifactKind::Folder => rsx! {
                    ul { class: "kit-frame__list",
                        for line in artifact.body.lines().map(str::to_string).collect::<Vec<_>>() {
                            li { key: "{line}", class: "kit-frame__item", "{line}" }
                        }
                    }
                },
                ArtifactKind::Code => rsx! {
                    pre { class: "kit-frame__code", code { "{artifact.body}" } }
                },
                ArtifactKind::Image => rsx! {
                    img { class: "kit-frame__image", src: "{artifact.body}", alt: "{artifact.name}" }
                },
            }}
        }
    }
}

/// A chat row: open, pin, unread dot, a more menu with its own control, draggable.
#[component]
fn ChatRowView(
    chat: ChatRow,
    menu_open: bool,
    projects: Vec<String>,
    on_toggle_menu: EventHandler<String>,
    on_close_menu: EventHandler<String>,
    on_open: EventHandler<ChatRow>,
    on_action: EventHandler<(String, String)>,
    on_move: EventHandler<(String, Option<String>)>,
    frame_prefix: String,
    move_open: bool,
    on_move_open: EventHandler<bool>,
    on_drag_start: EventHandler<String>,
) -> Element {
    let id = chat.id.clone();
    let more_id = format!("{frame_prefix}-more-{id}");
    let menu_id = format!("{frame_prefix}-menu-{id}");
    let more_label = format!("More actions for {}", chat.title);
    let pin_label = if chat.pinned {
        format!("Unpin {}", chat.title)
    } else {
        format!("Pin {}", chat.title)
    };
    let open_chat = chat.clone();
    let close_id = id.clone();
    rsx! {
        li { class: "kit-frame__row", "data-unread": chat.unread.then_some("true"),
            div {
                class: "kit-frame__row-main",
                draggable: "true",
                ondragstart: {
                    let id = id.clone();
                    move |_| on_drag_start.call(id.clone())
                },
                button {
                    r#type: "button",
                    class: "kit-frame__row-open",
                    onclick: move |_| on_open.call(open_chat.clone()),
                    if chat.unread { span { class: "kit-frame__dot", aria_hidden: "true" } }
                    span { class: "kit-frame__label", "{chat.title}" }
                }
                button {
                    r#type: "button",
                    class: "kit-frame__row-pin",
                    "aria-label": "{pin_label}",
                    "aria-pressed": chat.pinned.to_string(),
                    onclick: {
                        let id = id.clone();
                        move |_| on_action.call((id.clone(), "pin".to_string()))
                    },
                    span { aria_hidden: "true", if chat.pinned { "●" } else { "○" } }
                }
                button {
                    id: "{more_id}",
                    r#type: "button",
                    class: "kit-frame__row-more",
                    "aria-label": "{more_label}",
                    "aria-haspopup": "menu",
                    "aria-expanded": menu_open.to_string(),
                    "aria-controls": "{menu_id}",
                    onclick: {
                        let id = id.clone();
                        let menu_id = menu_id.clone();
                        move |_| {
                            on_toggle_menu.call(id.clone());
                            document::eval(&format!(
                                "requestAnimationFrame(() => document.getElementById({menu_id:?})?.scrollIntoView({{ block: 'nearest' }}));"
                            ));
                        }
                    },
                    span { aria_hidden: "true", "…" }
                }
            }
            if menu_open {
                div {
                    id: "{menu_id}",
                    class: "kit-frame__menu kit-frame__row-menu",
                    role: "menu",
                    aria_label: "Actions for {chat.title}",
                    onkeydown: {
                        let close_id = close_id.clone();
                        move |e: KeyboardEvent| {
                            if e.key() == Key::Escape {
                                e.prevent_default();
                                e.stop_propagation();
                                on_close_menu.call(close_id.clone());
                            }
                        }
                    },
                    for action in ["Rename", "Pin", "Mark unread", "Move to project", "Share", "Copy link", "Open in new window", "Archive"] {
                        if action == "Move to project" {
                            button {
                                key: "{action}",
                                r#type: "button",
                                role: "menuitem",
                                class: "kit-frame__menu-item",
                                "aria-expanded": move_open.to_string(),
                                onclick: move |_| on_move_open.call(!move_open),
                                "{action}"
                            }
                            if move_open {
                                div { class: "kit-frame__submenu", role: "menu", aria_label: "Move to project",
                                    for project in projects.clone() {
                                        button {
                                            key: "{project}",
                                            r#type: "button",
                                            role: "menuitem",
                                            class: "kit-frame__menu-item",
                                            onclick: {
                                                let id = close_id.clone();
                                                move |_| on_move.call((id.clone(), Some(project.clone())))
                                            },
                                            "{project}"
                                        }
                                    }
                                }
                            }
                        } else {
                            button {
                                key: "{action}",
                                r#type: "button",
                                role: "menuitem",
                                class: "kit-frame__menu-item",
                                onclick: {
                                    let id = close_id.clone();
                                    let action_key = match action {
                                        "Pin" => "pin",
                                        "Mark unread" => "unread",
                                        _ => "other",
                                    };
                                    move |_| {
                                        if action_key == "other" {
                                            on_close_menu.call(id.clone());
                                        } else {
                                            on_action.call((id.clone(), action_key.to_string()));
                                        }
                                    }
                                },
                                "{action}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Installs a Tab trap on the element `id`: focus cycles inside it, and the first
/// focusable element takes focus on open.
fn trap_script(id: &str) -> String {
    format!(
        "(() => {{\
           window.__kitTrapRemove?.();\
           const el = document.getElementById({id:?});\
           if (!el) return;\
           const focusables = () => [...el.querySelectorAll('button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),a[href],[tabindex=\"0\"]')].filter((x) => x.offsetParent !== null);\
           const onKey = (e) => {{\
             if (e.key !== 'Tab') return;\
             const f = focusables();\
             if (!f.length) return;\
             const first = f[0], last = f[f.length - 1];\
             if (e.shiftKey && (document.activeElement === first || !el.contains(document.activeElement))) {{ e.preventDefault(); last.focus(); }}\
             else if (!e.shiftKey && (document.activeElement === last || !el.contains(document.activeElement))) {{ e.preventDefault(); first.focus(); }}\
           }};\
           document.addEventListener('keydown', onKey, true);\
           window.__kitTrapRemove = () => document.removeEventListener('keydown', onKey, true);\
           requestAnimationFrame(() => focusables()[0]?.focus({{ preventScroll: true }}));\
         }})();"
    )
}
