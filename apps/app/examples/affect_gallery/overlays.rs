use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};
use rv2_affect::context_menu::ContextMenu;
use rv2_affect::dialog::{AlertDialog, Dialog, DialogSize};
use rv2_affect::dropdown_menu::{DropdownMenu, DropdownTrigger, MenuItem};
use rv2_affect::hover_card::HoverCard;
use rv2_affect::popover::{Popover, Side};
use rv2_affect::select::{Select, SelectOption};
use rv2_affect::tooltip::Tooltip;

use crate::{Item, Section};

fn options() -> Vec<SelectOption> {
    vec![
        SelectOption {
            value: "alpha".into(),
            label: "Alpha".into(),
            description: Some("First letter".into()),
            icon: None,
            disabled: false,
        },
        SelectOption {
            value: "beta".into(),
            label: "Beta".into(),
            description: None,
            icon: None,
            disabled: false,
        },
        SelectOption {
            value: "gamma".into(),
            label: "Gamma (archived)".into(),
            description: None,
            icon: None,
            disabled: true,
        },
        SelectOption {
            value: "delta".into(),
            label: "Delta".into(),
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
                div { class: "gl-row",
                    Tooltip { content: "Save", shortcut: "⌘S", button { "Hover or focus me" } }
                    Tooltip { content: "Opens in a new tab", button { "Plain label" } }
                }
            }
            Item { name: "HoverCard",
                div { class: "gl-row",
                    HoverCard { content: rsx! { "Ada Lovelace, analytical engine" }, a { href: "#overlays", "Hover Ada" } }
                }
            }
            Item { name: "Popover",
                div { class: "gl-row",
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
            }
            Item { name: "Dialog",
                div { class: "gl-row",
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
            }
            Item { name: "AlertDialog",
                div { class: "gl-row",
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
                p { class: "gl-out", "Last alert result: {alert_result}" }
            }
            Item { name: "DropdownMenu",
                div { class: "gl-row",
                    DropdownMenu {
                        trigger: Callback::new(|t: DropdownTrigger| rsx! {
                            button { id: "{t.id}", r#type: "button", "aria-expanded": "{t.expanded}", onclick: move |e| t.toggle.call(e), "Row actions" }
                        }),
                        items: menu(wrap(), last, wrap),
                    }
                }
                p { class: "gl-out", "Last menu action: {last}. Wrap: {wrap}" }
            }
            Item { name: "ContextMenu",
                ContextMenu { items: menu(wrap(), ctx_last, wrap),
                    div { class: "gl-canvas",
                        "Right-click here, or focus and press Shift+F10."
                    }
                }
                p { class: "gl-out", "Last context action: {ctx_last}" }
            }
            Item { name: "Select",
                div { class: "gl-box",
                    Select {
                        options: options(),
                        placeholder: "Pick one",
                        name: "letter",
                        onchange: move |o: SelectOption| picked.set(o.value),
                    }
                }
                p { class: "gl-out", "Selected: {picked}" }
            }
        }
    }
}
