use dioxus::prelude::*;
use rv2_affect::avatar::{Avatar, AvatarKind};
use rv2_affect::button::{Button, ButtonSize, ButtonVariant};
use rv2_affect::context_menu::ContextMenu;
use rv2_affect::dialog::{AlertDialog, Dialog, DialogSize};
use rv2_affect::dropdown_menu::{DropdownMenu, DropdownTrigger, MenuItem};
use rv2_affect::hover_card::HoverCard;
use rv2_affect::icon_button::{IconButton, IconButtonVariant};
use rv2_affect::popover::{Popover, Side};
use rv2_affect::select::{Select, SelectOption, SelectSize};
use rv2_affect::text_area::TextArea;
use rv2_affect::text_field::TextField;
use rv2_affect::tooltip::Tooltip;

use crate::gallery::Ex;
use crate::{Item, Section, glyph};

const SEARCH: &str = "M7 12a5 5 0 1 0 0-10 5 5 0 0 0 0 10zM14 14l-3.5-3.5";
const FILTER: &str = "M2.5 3.5h11M4.5 8h7M6.5 12.5h3";
const RERUN: &str = "M13 8a5 5 0 1 1-1.5-3.5M13 2.5v3h-3";
const BELL: &str = "M4 11.5V7a4 4 0 0 1 8 0v4.5l1 1H3zM6.5 13.5a1.5 1.5 0 0 0 3 0";

fn plans() -> Vec<SelectOption> {
    vec![
        SelectOption {
            value: "starter".into(),
            label: "Starter".into(),
            description: Some("Up to five seats".into()),
            icon: None,
            disabled: false,
        },
        SelectOption {
            value: "team".into(),
            label: "Team".into(),
            description: Some("Shared views and audit log".into()),
            icon: None,
            disabled: false,
        },
        SelectOption {
            value: "legacy".into(),
            label: "Legacy (retired)".into(),
            description: None,
            icon: None,
            disabled: true,
        },
        SelectOption {
            value: "enterprise".into(),
            label: "Enterprise".into(),
            description: None,
            icon: None,
            disabled: false,
        },
    ]
}

fn menu(wrap: bool, mut last: Signal<String>, mut wrap_set: Signal<bool>) -> Vec<MenuItem> {
    let mut copy = MenuItem::new("Copy");
    copy.shortcut = Some("⌘C".to_string());
    copy.onselect = Some(EventHandler::new(move |_| last.set("Copy".to_string())));

    let mut wrap_item = MenuItem::new("Wrap lines");
    wrap_item.checked = Some(wrap);
    wrap_item.checkbox = true;
    wrap_item.onselect = Some(EventHandler::new(move |_| wrap_set.set(!wrap)));

    let mut delete = MenuItem::new("Delete");
    delete.danger = true;
    delete.divider = true;
    delete.onselect = Some(EventHandler::new(move |_| last.set("Delete".to_string())));

    let mut share = MenuItem::new("Share");
    let mut email = MenuItem::new("Email");
    email.onselect = Some(EventHandler::new(move |_| {
        last.set("Share by email".to_string())
    }));
    share.submenu = Some(vec![email]);

    vec![copy, wrap_item, delete, share]
}

const EYE: &str = "M1.5 8s2.4-4.5 6.5-4.5S14.5 8 14.5 8s-2.4 4.5-6.5 4.5S1.5 8 1.5 8zM8 6.5a1.5 1.5 0 1 0 0 3 1.5 1.5 0 0 0 0-3z";
const DOWNLOAD: &str = "M8 2.5v7.5M5 7l3 3 3-3M3 13.5h10";
const LINK: &str = "M6.5 9.5a2.5 2.5 0 0 0 3.5 0l2-2a2.5 2.5 0 0 0-3.5-3.5l-.6.6M9.5 6.5a2.5 2.5 0 0 0-3.5 0l-2 2a2.5 2.5 0 0 0 3.5 3.5l.6-.6";
const DUPLICATE: &str = "M5.5 5.5h7v8h-7zM3.5 10.5v-8h7";
const FOLDER: &str =
    "M2 4.5a1 1 0 0 1 1-1h3l1.5 1.5H13a1 1 0 0 1 1 1v6.5a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1z";
const PIN: &str = "M6 2.5h4l-.5 4 2 2v1H4.5v-1l2-2zM8 9.5v4";
const TRASH: &str = "M2.5 4.5h11M6 4.5V3h4v1.5M4 4.5l.7 8.5h6.6l.7-8.5";
const FILE: &str = "M4.5 2h5l3.5 3.5V14h-8.5zM9.5 2v3.5H13";
const HIGHLIGHT: &str = "color-mix(in srgb, currentColor 6%, transparent)";

/// Name, folder, size, age, kind, pinned, background.
const FILE_ROWS: [(&str, &str, &str, &str, &str, bool, &str); 4] = [
    (
        "Launch notes, draft v3.pdf",
        "Docs",
        "412 KB",
        "1h ago",
        "pdf",
        true,
        "transparent",
    ),
    (
        "sync-worker.ts",
        "Code",
        "3 KB",
        "2m ago",
        "ts",
        false,
        HIGHLIGHT,
    ),
    (
        "pipeline-costs-w45.csv",
        "Finance",
        "48 KB",
        "14m ago",
        "csv",
        false,
        "transparent",
    ),
    (
        "onboarding-funnel.png",
        "Design",
        "96 KB",
        "1d ago",
        "img",
        false,
        "transparent",
    ),
];

fn file_tile(kind: &'static str) -> Element {
    let tint = match kind {
        "pdf" => "#f97316",
        "ts" => "#3b82f6",
        "csv" => "#16a34a",
        _ => "#9333ea",
    };
    rsx! {
        span { style: "display: grid; place-items: center; flex: none; width: 32px; height: 32px; border-radius: 8px; background: {HIGHLIGHT}; color: {tint};",
            {glyph(FILE)}
        }
    }
}

fn file_action(
    label: &'static str,
    path: &'static str,
    shortcut: Option<&str>,
    mut last: Signal<String>,
) -> MenuItem {
    let mut item = MenuItem::new(label);
    item.icon = Some(glyph(path));
    item.shortcut = shortcut.map(str::to_string);
    item.onselect = Some(EventHandler::new(move |_| last.set(label.to_string())));
    item
}

fn move_to_item(mut last: Signal<String>) -> MenuItem {
    let folders = ["Docs", "Code", "Finance"].map(|folder| {
        let mut item = MenuItem::new(folder);
        item.onselect = Some(EventHandler::new(move |_| {
            last.set(format!("Moved to {folder}"))
        }));
        item
    });
    let mut move_to = MenuItem::new("Move to");
    move_to.icon = Some(glyph(FOLDER));
    move_to.divider = true;
    move_to.submenu = Some(folders.to_vec());
    move_to
}

fn file_menu(last: Signal<String>) -> Vec<MenuItem> {
    let mut delete = file_action("Delete file", TRASH, None, last);
    delete.danger = true;
    delete.divider = true;

    vec![
        file_action("Open", EYE, Some("⌘O"), last),
        file_action("Download", DOWNLOAD, None, last),
        file_action("Copy link", LINK, Some("⌘L"), last),
        file_action("Duplicate", DUPLICATE, Some("⌘D"), last),
        move_to_item(last),
        file_action("Pin to top", PIN, None, last),
        delete,
    ]
}

fn run_menu(notify: bool, last: Signal<String>, mut notify_set: Signal<bool>) -> Vec<MenuItem> {
    let mut notify_item = MenuItem::new("Notify when done");
    notify_item.icon = Some(glyph(BELL));
    notify_item.checkbox = true;
    notify_item.checked = Some(notify);
    notify_item.onselect = Some(EventHandler::new(move |_| notify_set.set(!notify)));

    let mut delete = file_action("Delete run", TRASH, None, last);
    delete.danger = true;
    delete.divider = true;

    vec![
        file_action("Rerun", RERUN, Some("⌘R"), last),
        file_action("Duplicate", DUPLICATE, Some("⌘D"), last),
        file_action("Copy link", LINK, Some("⌘L"), last),
        move_to_item(last),
        notify_item,
        delete,
    ]
}

fn row_trigger() -> Callback<DropdownTrigger, Element> {
    Callback::new(|t: DropdownTrigger| {
        rsx! {
            button { id: "{t.id}", r#type: "button", "aria-expanded": "{t.expanded}", onclick: move |e| t.toggle.call(e), "Row actions" }
        }
    })
}

#[component]
pub fn Overlays() -> Element {
    let mut pop_open = use_signal(|| false);
    let mut dialog_open = use_signal(|| false);
    let mut alert_open = use_signal(|| false);
    let mut alert_result = use_signal(|| "none".to_string());
    let last = use_signal(|| "nothing yet".to_string());
    let wrap = use_signal(|| true);
    let notify = use_signal(|| true);
    let mut picked = use_signal(|| "none".to_string());
    let file_last = use_signal(|| "nothing yet".to_string());

    rsx! {
        Section { id: "overlays", title: "Overlays",
            Item { name: "Tooltip",
                Ex { label: "Shortcut",
                    Tooltip { content: "Save", shortcut: "⌘S", button { "Save draft" } }
                }
                Ex { label: "Plain, below",
                    Tooltip { content: "Opens in a new tab", side: Side::Bottom, button { "Plain label" } }
                }
                Ex { label: "Hover or focus the trigger to show",
                    Tooltip { content: "Hidden until hover", button { "Hover me" } }
                }
                Ex { label: "Open above a toolbar button",
                    div { class: "gl-box",
                        div { class: "gl-toolbar",
                            span { style: "font-size: 13px; font-weight: 500;", "Runs" }
                            div { class: "gl-toolbar-tools",
                                Tooltip { content: "Search runs", shortcut: "⌘K", open: true,
                                    IconButton { icon: glyph(SEARCH), label: "Search runs", variant: IconButtonVariant::Secondary }
                                }
                                IconButton { icon: glyph(FILTER), label: "Filter runs", variant: IconButtonVariant::Secondary }
                                IconButton { icon: glyph(PIN), label: "Pin runs", variant: IconButtonVariant::Secondary }
                            }
                        }
                    }
                }
            }
            Item { name: "HoverCard",
                Ex { label: "Closed, hover to open",
                    HoverCard { content: rsx! { "Ada Lovelace, analytical engine" }, a { class: "gl-link", href: "#overlays", "Hover Ada" } }
                }
                Ex { label: "Open",
                    div { class: "gl-space",
                        HoverCard {
                            open: true,
                            content: rsx! {
                                div { style: "display: grid; gap: 10px; min-width: 240px;",
                                    div { style: "display: flex; gap: 10px; align-items: center;",
                                        Avatar { name: "Scout", kind: AvatarKind::Agent, color: "#0060e2" }
                                        div { style: "display: grid;",
                                            strong { "Scout" }
                                            span { style: "font-size: 12px; opacity: 0.6;", "Research agent · claude-sonnet-4.5" }
                                        }
                                    }
                                    p { style: "margin: 0;", "Reads new issues, finds the cause and drafts a fix for the billing team." }
                                    div { style: "display: grid; gap: 4px; font-size: 12px; padding-top: 10px; border-top: 1px solid #0000000f;",
                                        div { style: "display: flex; justify-content: space-between;", span { "Status" } span { "Running · 2 of 3 tasks" } }
                                        div { style: "display: flex; justify-content: space-between;", span { "This week" } span { "41 runs · $12.40" } }
                                    }
                                    Button { variant: ButtonVariant::Secondary, label: "Open agent", size: ButtonSize::Sm }
                                }
                            },
                            a { class: "gl-link", href: "#overlays", "Scout" }
                        }
                    }
                }
            }
            Item { name: "Popover",
                Ex { label: "Closed, toggled by its trigger",
                    Button { variant: ButtonVariant::Secondary, label: "Filters", onclick: move |_| pop_open.set(!pop_open()) }
                    Popover {
                        open: pop_open(),
                        onclose: move |_| pop_open.set(false),
                        side: Side::Bottom,
                        title: rsx! { "Filters" },
                        description: rsx! { "Narrow the list" },
                        p { "Escape or outside click closes." }
                    }
                }
                Ex { label: "Open, below the trigger",
                    Button { variant: ButtonVariant::Secondary, label: "Deal stage" }
                    Popover {
                        open: true,
                        onclose: move |_| {},
                        side: Side::Bottom,
                        title: rsx! { "Schedule" },
                        description: rsx! { "The digest runs on its own and posts when it is done." },
                        div { style: "display: grid; gap: 2px; margin: 0 -4px;",
                            for (label, hint, chosen, fill) in [("Every day", "At 9:00", false, "transparent"), ("Weekdays", "Monday to Friday, 9:00", false, "transparent"), ("Every Monday", "Covers the week before", true, "#0000000d")] {
                                button {
                                    r#type: "button",
                                    "aria-pressed": "{chosen}",
                                    style: "display: flex; justify-content: space-between; align-items: center; gap: 10px; min-height: 0; padding: 8px; border: 0; border-radius: 8px; background: {fill}; color: inherit; font: inherit; text-align: start; cursor: pointer;",
                                    span { style: "display: grid;",
                                        span { "{label}" }
                                        span { style: "font-size: 12px; opacity: 0.6;", "{hint}" }
                                    }
                                    if chosen {
                                        span { "aria-hidden": "true", "•" }
                                    }
                                }
                            }
                        }
                        div { style: "display: flex; justify-content: flex-end; gap: 8px; margin-top: 12px;",
                            Button { variant: ButtonVariant::Secondary, label: "Cancel", size: ButtonSize::Sm }
                            Button { variant: ButtonVariant::Primary, label: "Save", size: ButtonSize::Sm }
                        }
                    }
                }
                div { style: "height: 300px" }
            }
            Item { name: "Dialog",
                Ex { label: "Trigger",
                    Button { label: "Open settings", onclick: move |_| dialog_open.set(true) }
                    Dialog {
                        open: dialog_open(),
                        onopenchange: move |v: bool| dialog_open.set(v),
                        title: rsx! { "Settings" },
                        description: rsx! { "Local to this browser" },
                        size: DialogSize::Md,
                        p { "Press Escape to close; focus returns to the trigger." }
                    }
                }
                Ex { label: "Open, medium",
                    div { class: "gl-space gl-space-wide",
                        div { class: "gl-page", "aria-hidden": "true",
                            span { style: "font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; opacity: 0.6;", "Agents 2" }
                            div { class: "gl-page-row", span { class: "gl-tile", {glyph(FILTER)} } span { "Release notes writer" } }
                            div { class: "gl-page-row", span { class: "gl-tile", {glyph(FILTER)} } span { "Churn digest" } }
                        }
                        Dialog {
                            open: true,
                            contained: true,
                            onopenchange: move |_| {},
                            title: rsx! { "Edit agent" },
                            description: rsx! { "Changes apply from its next run." },
                            size: DialogSize::Md,
                            footer: rsx! {
                                Button { variant: ButtonVariant::Secondary, label: "Cancel", size: ButtonSize::Sm }
                                Button { variant: ButtonVariant::Primary, label: "Save", size: ButtonSize::Sm }
                            },
                            TextField { label: rsx! { "Name" }, default_value: "Release notes writer".to_string() }
                            TextArea { label: rsx! { "Instructions" }, min_rows: 3, default_value: "Sum up last week's merged pull requests in plain words. Group them by area.".to_string() }
                        }
                    }
                }
            }
            Item { name: "AlertDialog",
                Ex { label: "Trigger",
                    Button { variant: ButtonVariant::Destructive, label: "Delete agent", onclick: move |_| alert_open.set(true) }
                    AlertDialog {
                        open: alert_open(),
                        onopenchange: move |v: bool| alert_open.set(v),
                        title: rsx! { "Delete agent?" },
                        danger: true,
                        confirm_label: "Delete agent",
                        onconfirm: move |_| alert_result.set("confirmed".to_string()),
                        oncancel: move |_| alert_result.set("cancelled".to_string()),
                        p { "This removes its history. It cannot be undone." }
                    }
                }
                Ex { label: "Open, danger",
                    div { class: "gl-space gl-space-wide",
                        AlertDialog {
                            open: true,
                            contained: true,
                            onopenchange: move |_| {},
                            title: rsx! { "Remove Northwind from the pipeline?" },
                            danger: true,
                            confirm_label: "Remove",
                            onconfirm: move |_| alert_result.set("confirmed".to_string()),
                            oncancel: move |_| alert_result.set("cancelled".to_string()),
                            p { "Its 14 open tasks move to the archive." }
                        }
                    }
                }
                p { class: "gl-out", "Last alert result: {alert_result}" }
            }
            Item { name: "DropdownMenu",
                Ex { label: "Closed",
                    DropdownMenu { trigger: row_trigger(), items: run_menu(notify(), last, notify) }
                }
                Ex { label: "Open, checkbox and submenu",
                    div { class: "gl-space",
                        DropdownMenu {
                            trigger: row_trigger(),
                            open: true,
                            onopenchange: move |_| {},
                            items: run_menu(notify(), last, notify),
                        }
                    }
                }
                Ex { label: "Open, empty",
                    div { class: "gl-space",
                        DropdownMenu {
                            trigger: row_trigger(),
                            open: true,
                            onopenchange: move |_| {},
                            items: Vec::<MenuItem>::new(),
                            empty: "No saved views yet",
                        }
                    }
                }
                p { class: "gl-out", "Last menu action: {last}. Notify: {notify}" }
            }
            Item { name: "ContextMenu",
                Ex { label: "Open at rest, beside the highlighted file. Scroll keeps it in place",
                    ContextMenu { items: file_menu(file_last), open_at: (169.0, 126.0),
                        div { style: "width: 100%; max-width: 400px; box-sizing: border-box; padding: 14px 8px 0; border: 1px solid color-mix(in srgb, currentColor 12%, transparent); border-radius: 12px; font-size: 13px;",
                            div { style: "display: flex; gap: 6px; align-items: center; height: 26px; padding: 0 8px; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; opacity: 0.6;",
                                span { "Files" }
                                span { "4" }
                            }
                            div { style: "display: grid; gap: 2px; padding: 4px 0 8px;",
                                for (name, folder, size, age, kind, pinned, fill) in FILE_ROWS {
                                    div { style: "display: flex; align-items: center; gap: 12px; padding: 8px; border-radius: 8px; background: {fill};",
                                        {file_tile(kind)}
                                        span { style: "display: grid; flex: 1; min-width: 0;",
                                            span { style: "display: flex; align-items: center; gap: 6px; min-width: 0;",
                                                span { style: "overflow: hidden; white-space: nowrap; text-overflow: ellipsis;", "{name}" }
                                                if pinned {
                                                    span { style: "display: grid; flex: none; opacity: 0.5;", {glyph(PIN)} }
                                                }
                                            }
                                            span { style: "font-size: 12px; opacity: 0.6; white-space: nowrap;", "{folder} · {size} · {age}" }
                                        }
                                    }
                                }
                            }
                            div { style: "padding: 12px 8px; border-top: 1px solid color-mix(in srgb, currentColor 10%, transparent); font-size: 12px; opacity: 0.6; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                "Right-click a file or the empty space for more."
                            }
                        }
                    }
                }
                Ex { label: "Right-click, titled",
                    ContextMenu { title: "Message", items: menu(wrap(), file_last, wrap),
                        p { style: "margin: 0; padding: 8px 0; font-size: 13px;", "Right-click this message for actions." }
                    }
                }
                p { class: "gl-out", "Last context action: {file_last}" }
            }
            Item { name: "Select",
                Ex { label: "Placeholder",
                    div { class: "gl-box",
                        Select {
                            options: plans(),
                            placeholder: "Pick a plan",
                            name: "plan",
                            onchange: move |o: SelectOption| picked.set(o.value),
                        }
                    }
                }
                Ex { label: "Sizes",
                    div { class: "gl-box",
                        Select { options: plans(), placeholder: "Small", size: SelectSize::Sm }
                        Select { options: plans(), placeholder: "Medium", size: SelectSize::Md }
                        Select { options: plans(), placeholder: "Large", size: SelectSize::Lg }
                    }
                }
                Ex { label: "Labelled, with hint",
                    div { class: "gl-box",
                        Select {
                            options: plans(),
                            placeholder: "Choose",
                            label: rsx! { "Plan" },
                            hint: rsx! { "You can change it later" },
                            optional: true,
                        }
                    }
                }
                Ex { label: "Error",
                    div { class: "gl-box",
                        Select {
                            options: plans(),
                            placeholder: "Choose",
                            label: rsx! { "Plan" },
                            error: rsx! { "Pick a plan to continue" },
                            required: true,
                        }
                    }
                }
                Ex { label: "Default value, disabled",
                    div { class: "gl-box",
                        Select { options: plans(), default_value: "team", label: rsx! { "Current plan" }, disabled: true }
                    }
                }
                p { class: "gl-out", "Selected: {picked}" }
            }
        }
    }
}
