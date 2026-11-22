use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};
use rv2_affect::context_menu::ContextMenu;
use rv2_affect::dialog::{AlertDialog, Dialog, DialogSize};
use rv2_affect::dropdown_menu::{DropdownMenu, DropdownTrigger, MenuItem};
use rv2_affect::hover_card::HoverCard;
use rv2_affect::popover::{Popover, Side};
use rv2_affect::select::{Select, SelectOption, SelectSize};
use rv2_affect::tooltip::Tooltip;

use crate::gallery::Ex;
use crate::{Item, Section};

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
    let mut picked = use_signal(|| "none".to_string());
    let ctx_last = use_signal(|| "nothing yet".to_string());

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
                                strong { "Northwind Traders" }
                                p { "Renewal due 14 Jan · 42 seats" }
                            },
                            a { class: "gl-link", href: "#overlays", "Northwind" }
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
                        title: rsx! { "Deal stage" },
                        description: rsx! { "Show deals in" },
                        p { "Open, Won, Lost and Archived" }
                    }
                }
                div { style: "height: 140px" }
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
                    div { class: "gl-space",
                        Dialog {
                            open: true,
                            contained: true,
                            onopenchange: move |_| {},
                            title: rsx! { "Workspace settings" },
                            description: rsx! { "Changes apply to every member" },
                            size: DialogSize::Md,
                            footer: rsx! {
                                Button { variant: ButtonVariant::Secondary, label: "Cancel", size: rv2_affect::button::ButtonSize::Sm }
                                Button { variant: ButtonVariant::Primary, label: "Save", size: rv2_affect::button::ButtonSize::Sm }
                            },
                            p { "Weekly digest goes to admins on Mondays at 09:00." }
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
                    div { class: "gl-space",
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
                    DropdownMenu { trigger: row_trigger(), items: menu(wrap(), last, wrap) }
                }
                Ex { label: "Open, checkbox and submenu",
                    div { class: "gl-space",
                        DropdownMenu {
                            trigger: row_trigger(),
                            open: true,
                            onopenchange: move |_| {},
                            items: menu(wrap(), last, wrap),
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
                p { class: "gl-out", "Last menu action: {last}. Wrap: {wrap}" }
            }
            Item { name: "ContextMenu",
                Ex { label: "Right-click area",
                    ContextMenu { items: menu(wrap(), ctx_last, wrap),
                        div { class: "gl-canvas",
                            "Right-click here, or focus and press Shift+F10."
                        }
                    }
                }
                Ex { label: "Titled menu",
                    ContextMenu { title: "Message", items: menu(wrap(), ctx_last, wrap),
                        div { class: "gl-canvas",
                            "Right-click this message for actions."
                        }
                    }
                }
                p { class: "gl-out", "Last context action: {ctx_last}" }
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
