use dioxus::prelude::*;
use rv2_affect::breadcrumb::{Breadcrumb, BreadcrumbItem};
use rv2_affect::filter_menu::{FilterGroup, FilterMenu, FilterOption, FilterValue};
use rv2_affect::pagination::Pagination;
use rv2_affect::profile_menu::{ProfileMenu, ProfileSetup};
use rv2_affect::resizable::{Direction, ResizableHandle, ResizablePanel, ResizablePanelGroup};
use rv2_affect::scroll_area::{ScrollArea, ScrollOrientation, ScrollbarVisibility};
use rv2_affect::tree_view::{TreeNode, TreeView};

use crate::gallery::Ex;
use crate::{Item, Section, glyph};

const FOLDER: &str = "M2 4h4l1.5 1.5H14v7H2z";

fn crumb(label: &str, href: Option<&str>) -> BreadcrumbItem {
    BreadcrumbItem {
        label: label.to_string(),
        href: href.map(str::to_string),
        onclick: None,
        icon: None,
    }
}

fn filter_groups() -> Vec<FilterGroup> {
    vec![
        FilterGroup {
            id: "score".into(),
            label: "Score".into(),
            options: vec![
                FilterOption {
                    id: "80".into(),
                    label: "80–90".into(),
                },
                FilterOption {
                    id: "90".into(),
                    label: "90+".into(),
                },
            ],
            chip_prefix: Some("Score: ".into()),
        },
        FilterGroup {
            id: "stage".into(),
            label: "Stage".into(),
            options: vec![
                FilterOption {
                    id: "open".into(),
                    label: "Open".into(),
                },
                FilterOption {
                    id: "won".into(),
                    label: "Won".into(),
                },
            ],
            chip_prefix: None,
        },
    ]
}

fn leaf(id: &str, label: &str) -> TreeNode {
    TreeNode {
        id: id.into(),
        label: label.into(),
        icon: None,
        open_icon: None,
        meta: None,
        disabled: false,
        children: None,
    }
}

fn branch(id: &str, label: &str, children: Vec<TreeNode>) -> TreeNode {
    TreeNode {
        children: Some(children),
        ..leaf(id, label)
    }
}

fn tree() -> Vec<TreeNode> {
    vec![
        branch(
            "src",
            "src",
            vec![
                leaf("main", "main.rs"),
                branch(
                    "affect",
                    "affect",
                    vec![leaf("lib", "lib.rs"), leaf("button", "button.rs")],
                ),
            ],
        ),
        branch("docs", "docs", vec![leaf("readme", "README.md")]),
        leaf("cargo", "Cargo.toml"),
    ]
}

#[component]
pub fn Navigation() -> Element {
    let mut page = use_signal(|| 3usize);
    let mut size = use_signal(|| 20usize);
    let mut picked =
        use_signal(|| FilterValue::from([("score".to_string(), vec!["90".to_string()])]));
    let mut selected = use_signal(|| "none".to_string());

    let rows: Vec<String> = (1..=24)
        .map(|i| format!("Deal {i:02} · Northwind renewal"))
        .collect();

    rsx! {
        Section { id: "navigation", title: "Navigation",
            Item { name: "Breadcrumb",
                Ex { label: "Deep path",
                    Breadcrumb {
                        items: vec![
                            crumb("Workspace", Some("#navigation")),
                            crumb("Reports", Some("#navigation")),
                            crumb("Folder A", None),
                            crumb("Folder B", None),
                            crumb("Folder C", None),
                            crumb("Q3 pipeline", None),
                        ],
                    }
                }
                Ex { label: "Two levels, with icon",
                    Breadcrumb {
                        items: vec![
                            crumb("Workspace", Some("#navigation")),
                            BreadcrumbItem { icon: Some(glyph(FOLDER)), ..crumb("Northwind", None) },
                        ],
                    }
                }
            }
            Item { name: "FilterMenu",
                Ex { label: "Two groups active",
                    div { class: "gl-row",
                        FilterMenu {
                            title: "Deals",
                            groups: filter_groups(),
                            value: picked(),
                            on_change: move |v: FilterValue| picked.set(v),
                        }
                    }
                }
                Ex { label: "Nothing selected",
                    div { class: "gl-row",
                        FilterMenu {
                            title: "Deals",
                            groups: filter_groups(),
                            value: FilterValue::default(),
                            on_change: move |_: FilterValue| {},
                        }
                    }
                }
                p { class: "gl-out", "Picked: {picked:?}" }
            }
            Item { name: "Pagination",
                Ex { label: "Many pages, with page size",
                    div { class: "gl-box",
                        Pagination {
                            page: page(),
                            total: 1284usize,
                            page_size: size(),
                            on_page_change: move |p: usize| page.set(p),
                            on_page_size_change: move |s: usize| size.set(s),
                        }
                    }
                }
                Ex { label: "First of three pages",
                    div { class: "gl-box",
                        Pagination { page: 1usize, total: 60usize, page_size: 20usize, on_page_change: move |_| {} }
                    }
                }
                Ex { label: "Compact",
                    div { class: "gl-box",
                        Pagination {
                            page: 2usize,
                            total: 200usize,
                            page_size: 10usize,
                            compact: true,
                            on_page_change: move |_| {},
                        }
                    }
                }
                Ex { label: "Disabled",
                    div { class: "gl-box",
                        Pagination { page: 1usize, total: 60usize, page_size: 20usize, disabled: true, on_page_change: move |_| {} }
                    }
                }
                p { class: "gl-out", "Page {page}, size {size}" }
            }
            Item { name: "ProfileMenu",
                Ex { label: "Setup in progress",
                    div { class: "gl-box",
                        ProfileMenu {
                            name: "Ada Lovelace",
                            email: "ada@example.com",
                            setup: ProfileSetup {
                                done: 2,
                                total: 5,
                                next: Some("Add a photo".into()),
                                on_continue: Some(EventHandler::new(move |_| selected.set("setup continue".to_string()))),
                            },
                        }
                    }
                }
                Ex { label: "Setup complete",
                    div { class: "gl-box",
                        ProfileMenu {
                            name: "Ben Ortiz",
                            email: "ben@northwind.example",
                            setup: ProfileSetup { done: 5, total: 5, next: None, on_continue: None },
                        }
                    }
                }
                p { class: "gl-out", "Last profile action: {selected}" }
            }
            Item { name: "Resizable",
                Ex { label: "Side by side, with grip",
                    div { class: "gl-canvas gl-canvas-tall",
                        ResizablePanelGroup { direction: Direction::Horizontal,
                            ResizablePanel { id: "list", default_size: 30.0, div { style: "padding: 8px", "List pane" } }
                            ResizableHandle { with_handle: true }
                            ResizablePanel { id: "detail", default_size: 70.0, div { style: "padding: 8px", "Detail pane" } }
                        }
                    }
                }
                Ex { label: "Stacked, no grip",
                    div { class: "gl-canvas gl-canvas-tall",
                        ResizablePanelGroup { direction: Direction::Vertical,
                            ResizablePanel { id: "top", default_size: 40.0, div { style: "padding: 8px", "Activity" } }
                            ResizableHandle { with_handle: false }
                            ResizablePanel { id: "bottom", default_size: 60.0, div { style: "padding: 8px", "Comments" } }
                        }
                    }
                }
                p { class: "gl-out", "Drag the grip, or focus it and use arrow keys." }
            }
            Item { name: "ScrollArea",
                Ex { label: "Vertical, scrollbar on hover, faded",
                    div { class: "gl-box",
                        ScrollArea { max_height: "200px", orientation: ScrollOrientation::Vertical, visibility: ScrollbarVisibility::Hover, fade: true,
                            ul { class: "gl-list",
                                for row in rows.iter() {
                                    li { "{row}" }
                                }
                            }
                        }
                    }
                }
                Ex { label: "Horizontal, scrollbar while scrolling",
                    div { class: "gl-box",
                        ScrollArea { max_height: "120px", orientation: ScrollOrientation::Horizontal, visibility: ScrollbarVisibility::Scroll,
                            div { style: "display: flex; gap: 8px; width: max-content; padding: 4px",
                                for chip in ["Northwind", "Globex", "Initech", "Umbrella", "Hooli", "Vandelay"] {
                                    span { class: "gl-tag", style: "padding: 6px 12px; border: 1px solid #0000001f; border-radius: 999px; opacity: 1; font-size: 13px", "{chip}" }
                                }
                            }
                        }
                    }
                }
            }
            Item { name: "TreeView",
                Ex { label: "Expanded to a file",
                    div { class: "gl-box",
                        TreeView {
                            items: tree(),
                            default_expanded: vec!["src".to_string()],
                            default_selected: "main".to_string(),
                            aria_label: "Project files",
                            on_select: move |n: TreeNode| selected.set(n.label),
                        }
                    }
                }
                Ex { label: "Collapsed, one node disabled",
                    div { class: "gl-box",
                        TreeView {
                            items: vec![
                                branch("exports", "exports", vec![leaf("q2", "Q2 deck.pdf")]),
                                TreeNode { disabled: true, ..leaf("archive", "Archive (locked)") },
                            ],
                            default_expanded: Vec::<String>::new(),
                            default_selected: "cargo".to_string(),
                            aria_label: "Exports",
                            on_select: move |n: TreeNode| selected.set(n.label),
                        }
                    }
                }
                p { class: "gl-out", "Selected node: {selected}" }
            }
        }
    }
}
