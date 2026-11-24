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
const FOLDER_OPEN: &str = "M2 4h4l1.5 1.5H14v1.5H4.5L2 12.5zM4.5 7.5H15l-2 5H2.5z";
const FILE: &str = "M4.5 2h5l3.5 3.5V14h-8.5zM9.5 2v3.5H13";
/// Handle lines in the ink colour, so they show on the card in both themes.
const LINES: &str = "--ui-rszc-line: color-mix(in srgb, currentColor 16%, transparent); --ui-rszc-line-active: color-mix(in srgb, currentColor 36%, transparent); --ui-rszc-grip-edge: color-mix(in srgb, currentColor 24%, transparent);";

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

fn meta(text: &str) -> Option<Element> {
    Some(rsx! { span { class: "gl-tree-meta", "{text}" } })
}

fn file(id: &str, label: &str, size: &str) -> TreeNode {
    TreeNode {
        icon: Some(glyph(FILE)),
        meta: meta(size),
        ..leaf(id, label)
    }
}

fn folder(id: &str, label: &str, count: Option<&str>, children: Vec<TreeNode>) -> TreeNode {
    TreeNode {
        icon: Some(glyph(FOLDER)),
        open_icon: Some(glyph(FOLDER_OPEN)),
        meta: count.and_then(meta),
        ..branch(id, label, children)
    }
}

fn tree() -> Vec<TreeNode> {
    vec![
        folder(
            "prompts",
            "prompts",
            Some("3"),
            vec![
                file("digest", "digest.md", "2 KB"),
                file("triage", "triage.md", "1 KB"),
                file("reply", "reply.md", "3 KB"),
            ],
        ),
        folder(
            "runs",
            "runs",
            None,
            vec![
                folder(
                    "run-418",
                    "run-418",
                    Some("Today"),
                    vec![
                        file("report", "report.md", "12 KB"),
                        file("sources", "sources.csv", "48 rows"),
                        folder(
                            "charts",
                            "charts",
                            None,
                            vec![file("plot", "plot.svg", "4 KB")],
                        ),
                    ],
                ),
                folder(
                    "run-417",
                    "run-417",
                    Some("Yesterday"),
                    vec![file("report-417", "report.md", "9 KB")],
                ),
                TreeNode {
                    disabled: true,
                    ..folder("run-409", "run-409 · archived", None, Vec::new())
                },
            ],
        ),
        folder(
            "memory",
            "memory",
            None,
            vec![file("notes", "notes.md", "1 KB")],
        ),
        file("agent", "agent.yaml", "1 KB"),
    ]
}

#[component]
pub fn Navigation() -> Element {
    let mut page = use_signal(|| 1usize);
    let mut size = use_signal(|| 25usize);
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
                    div { class: "gl-box",
                        FilterMenu {
                            title: "Deals",
                            groups: filter_groups(),
                            value: picked(),
                            on_change: move |v: FilterValue| picked.set(v),
                        }
                    }
                }
                Ex { label: "Nothing selected",
                    div { class: "gl-box",
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
                            page_sizes: vec![10, 25, 50],
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
                        ResizablePanelGroup { direction: Direction::Horizontal, style: LINES,
                            ResizablePanel { id: "list", default_size: 36.0,
                                div { style: "padding: 10px 8px; display: grid; gap: 2px; font-size: 13px;",
                                    span { style: "font-size: 12px; opacity: 0.6; padding: 0 8px 6px;", "Runs · 5" }
                                    for (name, meta, fill) in [("Summarise churn interviews", "Running · 1m 12s", "#0000000d"), ("Draft Q3 pricing memo", "Done · 3m 40s", "transparent"), ("Tag 214 support tickets", "Done · 58s", "transparent"), ("Sync invoices to billing", "Failed · 12s", "transparent")] {
                                        div { style: "display: grid; padding: 8px; border-radius: 8px; background: {fill};",
                                            span { style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{name}" }
                                            span { style: "font-size: 12px; opacity: 0.6;", "{meta}" }
                                        }
                                    }
                                }
                            }
                            ResizableHandle { with_handle: true }
                            ResizablePanel { id: "detail", default_size: 64.0,
                                div { style: "padding: 10px 14px; display: grid; gap: 8px; font-size: 13px;",
                                    span { style: "font-size: 12px; opacity: 0.6;", "Output" }
                                    strong { "Summarise churn interviews" }
                                    p { style: "margin: 0;", "Most customers who left in Q3 did so within two weeks of a price change. Five themes came up again and again: surprise overage fees, unclear credit packs, missing annual discounts, slow support replies and seats they no longer used." }
                                }
                            }
                        }
                    }
                }
                Ex { label: "Stacked, no grip",
                    div { class: "gl-canvas", style: "height: 320px;",
                        ResizablePanelGroup { direction: Direction::Vertical, style: LINES,
                            ResizablePanel { id: "top", default_size: 50.0,
                                div { style: "padding: 10px 14px; display: grid; gap: 4px; font-size: 12px; font-family: ui-monospace, monospace;",
                                    span { "14:02:07  Read 18 interview notes" }
                                    span { "14:02:19  Grouped quotes into 5 themes" }
                                    span { "14:02:44  Asked billing-api for plan changes" }
                                }
                            }
                            ResizableHandle { with_handle: false }
                            ResizablePanel { id: "bottom", default_size: 50.0,
                                div { style: "padding: 10px 14px; font-size: 13px;",
                                    strong { "Comments" }
                                    p { style: "margin: 6px 0 0;", "Ada: the 14% figure needs the enterprise cohort split out." }
                                }
                            }
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
                Ex { label: "Workspace, expanded to a file",
                    div { class: "gl-box",
                        div { style: "border: 1px solid #0000000f; border-radius: 12px; overflow: hidden;",
                            div { style: "display: flex; gap: 8px; align-items: baseline; padding: 12px 14px 8px; font-size: 13px; font-weight: 500;",
                                "Workspace"
                                span { style: "font-size: 12px; font-weight: 400; opacity: 0.55;", "research-agent" }
                            }
                            div { class: "gl-tree",
                                TreeView {
                                    items: tree(),
                                    default_expanded: vec!["runs".to_string(), "run-418".to_string()],
                                    default_selected: "report".to_string(),
                                    aria_label: "Workspace files",
                                    on_select: move |n: TreeNode| selected.set(n.label),
                                }
                            }
                            div { style: "padding: 10px 14px; border-top: 1px solid #0000000f; font-size: 12px; opacity: 0.6;", "Pick a file" }
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
