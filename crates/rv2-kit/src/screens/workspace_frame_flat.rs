//! Workspace frame, flat. The same frame as `workspace_frame_inset` with
//! `frame_style` set to flat: the page runs edge to edge, separated from the sidebar
//! by a hairline, with no rounded panel.
//!
//! Design choice: this module is a wrapper, not a copy. It forwards the flat spec's
//! data to `WorkspaceFrame` and fixes `FrameStyle::Flat`; the splitters, drawers, tabs
//! and chat rows are the shared implementation.
//! The default header is the open page's name, a share control and a more menu; the
//! `top_bar_title` and `top_bar_actions` props replace them.
//! Not forwarded: sidebar and panel widths, `side_cards`, `default_panel_open`, and the
//! inset-only props. Use `WorkspaceFrame` directly to set them.

use dioxus::prelude::*;

use super::workspace_frame_inset::{
    Artifact, ArtifactKind, ChatRow, FrameAccount, FrameStyle, FrameTabs, MenuGroup, MenuItem,
    WorkspaceFrame,
};

const FLAT_CSS: Asset = asset!("/assets/workspace-frame-flat.css");

/// One row of the Start tab: its name and the line that says where it comes from.
#[derive(Clone, Debug, PartialEq)]
pub struct StartRow {
    pub title: String,
    pub source: String,
}

/// The Start tab body: a Tools group, then a Suggested group, each under its heading in the
/// order given. Activating a row opens it as a work-area tab and then calls `on_select`.
#[component]
pub fn FlatStartPanel(
    tools: Vec<StartRow>,
    suggested: Vec<StartRow>,
    #[props(default)] on_select: Option<EventHandler<StartRow>>,
) -> Element {
    rsx! {
        div { class: "kit-flat-start",
            StartGroup { heading: "Tools", rows: tools, on_select }
            StartGroup { heading: "Suggested", rows: suggested, on_select }
        }
    }
}

#[component]
fn StartGroup(
    heading: &'static str,
    rows: Vec<StartRow>,
    on_select: Option<EventHandler<StartRow>>,
) -> Element {
    let frame_tabs = try_use_context::<FrameTabs>();
    rsx! {
        section { class: "kit-flat-start__group",
            h3 { class: "kit-flat-start__heading", "{heading}" }
            ul { class: "kit-flat-start__list",
                for row in rows {
                    li { key: "{row.title}", class: "kit-flat-start__item",
                        button {
                            r#type: "button",
                            class: "kit-flat-start__row",
                            onclick: {
                                let picked = row.clone();
                                move |_| {
                                    if let Some(tabs) = frame_tabs {
                                        tabs.open_artifact(Artifact {
                                            name: picked.title.clone(),
                                            kind: ArtifactKind::Document,
                                            body: format!("{}\n\n{}", picked.title, picked.source),
                                        });
                                    }
                                    if let Some(select) = on_select {
                                        select.call(picked.clone());
                                    }
                                }
                            },
                            span { class: "kit-flat-start__title", "{row.title}" }
                            span { class: "kit-flat-start__source", "{row.source}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn WorkspaceFrameFlat(
    content: Element,
    #[props(default = true)] top_bar: bool,
    #[props(default)] top_bar_title: Option<Element>,
    #[props(default)] top_bar_actions: Option<Element>,
    #[props(default)] current_item: Option<String>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default = "Workspace".to_string(), into)] workspace_name: String,
    #[props(default)] account: Option<FrameAccount>,
    #[props(default)] menu_items: Option<Vec<MenuItem>>,
    #[props(default)] menu_groups: Vec<MenuGroup>,
    #[props(default)] project_list: Vec<String>,
    #[props(default)] chat_list: Vec<ChatRow>,
    #[props(default)] on_open_chat: Option<EventHandler<ChatRow>>,
    #[props(default)] work_area: bool,
    #[props(default)] side_panel: Option<Element>,
    #[props(default)] panel_heading: Option<String>,
    #[props(default)] artifacts: Vec<Artifact>,
    #[props(default)] on_open_artifact: Option<EventHandler<String>>,
    #[props(default)] initial_tabs: Vec<Artifact>,
    #[props(default, into)] class_name: String,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: FLAT_CSS }
        WorkspaceFrame {
            content,
            frame_style: FrameStyle::Flat,
            top_bar,
            top_bar_title,
            top_bar_actions,
            current_item,
            on_navigate,
            workspace_name,
            account,
            menu_items,
            menu_groups,
            project_list,
            chat_list,
            on_open_chat,
            work_area,
            side_panel,
            panel_heading,
            artifacts,
            on_open_artifact,
            initial_tabs,
            class_name,
        }
    }
}
