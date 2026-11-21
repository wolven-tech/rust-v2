//! Toggle group: a row of toggle buttons. `Multiple` lets any number be on;
//! `Single` keeps at most one, and pressing it again turns it off.
//!
//! Port losses: the React `type="single"` callback receives `T | null`; here
//! both kinds report a `Vec<String>` (zero or one item in single mode). The
//! arrow-key focus move runs in the browser through `document::eval`, so the
//! arrow keys are handled in JS rather than by a Rust match on the target.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/toggle-group.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleKind {
    /// Any number of items can be on.
    #[default]
    Multiple,
    /// At most one item is on; pressing it again turns it off.
    Single,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleSize {
    /// 26px high.
    Sm,
    /// 30px high.
    #[default]
    Md,
    /// 36px high.
    Lg,
}

impl ToggleSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleVariant {
    /// A light-gray track with pressed items raised as white pills.
    #[default]
    Tray,
    /// Separate chips with an edge that wrap onto a new line.
    Buttons,
}

impl ToggleVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Tray => "tray",
            Self::Buttons => "buttons",
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct ToggleGroupItem {
    pub value: String,
    /// Shown as text. Without a label or icon the value is shown.
    pub label: Option<Element>,
    pub icon: Option<Element>,
    /// Accessible name and tooltip. Needed when the item shows only an icon.
    pub aria_label: Option<String>,
    pub disabled: bool,
}

/// Drops items with a repeated value; the first one wins.
pub fn dedupe_items(items: Vec<ToggleGroupItem>) -> Vec<ToggleGroupItem> {
    let mut seen: Vec<String> = Vec::new();
    items
        .into_iter()
        .filter(|item| {
            if seen.contains(&item.value) {
                false
            } else {
                seen.push(item.value.clone());
                true
            }
        })
        .collect()
}

/// The selection after pressing `value`, or `None` when that item is missing
/// or disabled. Multiple keeps the items' order, whatever order they were
/// pressed in.
pub fn next_selection(
    items: &[ToggleGroupItem],
    on: &[String],
    value: &str,
    single: bool,
) -> Option<Vec<String>> {
    let item = items.iter().find(|item| item.value == value)?;
    if item.disabled {
        return None;
    }
    let pressed = on.iter().any(|v| v == value);
    if single {
        return Some(if pressed {
            Vec::new()
        } else {
            vec![value.to_string()]
        });
    }
    Some(
        items
            .iter()
            .filter_map(|item| {
                let keep = if item.value == value {
                    !pressed
                } else {
                    on.contains(&item.value)
                };
                keep.then(|| item.value.clone())
            })
            .collect(),
    )
}

#[component]
pub fn ToggleGroup(
    items: Vec<ToggleGroupItem>,
    #[props(default)] kind: ToggleKind,
    #[props(default)] size: ToggleSize,
    #[props(default)] variant: ToggleVariant,
    #[props(default)] disabled: bool,
    #[props(default, into)] aria_label: Option<String>,
    #[props(default, into)] aria_labelledby: Option<String>,
    #[props(default)] class: String,
    #[props(default)] value: Option<Vec<String>>,
    #[props(default)] default_value: Vec<String>,
    #[props(default)] on_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let id = use_hook(|| format!("ui-tgl-{}", next_id()));
    let mut inner = use_signal(|| default_value.clone());
    let mut focused = use_signal(|| None::<String>);

    let items = Rc::new(dedupe_items(items));
    let single = kind == ToggleKind::Single;
    let controlled = value.is_some();
    let on: Vec<String> = value.unwrap_or(inner());
    let class = class_list("ui-tgl", &class);

    let enabled: Vec<String> = items
        .iter()
        .filter(|item| !item.disabled && !disabled)
        .map(|item| item.value.clone())
        .collect();
    // Roving tab stop: Tab reaches exactly one item in the group.
    let stop = focused()
        .filter(|f| enabled.contains(f))
        .or_else(|| enabled.iter().find(|v| on.contains(v)).cloned())
        .or_else(|| enabled.first().cloned());

    let group_id = id.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            role: "group",
            aria_label,
            aria_labelledby,
            aria_disabled: disabled.then_some("true"),
            class,
            "data-size": size.as_str(),
            "data-variant": variant.as_str(),
            "data-disabled": disabled.then_some("true"),
            onkeydown: move |event: KeyboardEvent| {
                'navigate: {
                    if event.modifiers().alt() || event.modifiers().ctrl() || event.modifiers().meta() {
                        break 'navigate;
                    }
                    let name = match event.key() {
                        Key::ArrowLeft => "ArrowLeft",
                        Key::ArrowRight => "ArrowRight",
                        Key::ArrowUp => "ArrowUp",
                        Key::ArrowDown => "ArrowDown",
                        Key::Home => "Home",
                        Key::End => "End",
                        _ => break 'navigate,
                    };
                    event.prevent_default();
                    document::eval(&format!(
                    "const g = document.getElementById({group_id:?}); if (!g) return;
                     const buttons = Array.from(g.querySelectorAll(':scope > .ui-tgl__item:not(:disabled)'));
                     const at = buttons.indexOf(document.activeElement);
                     if (at < 0) return;
                     const rtl = getComputedStyle(g).direction === 'rtl';
                     const fwd = rtl ? 'ArrowLeft' : 'ArrowRight';
                     const back = rtl ? 'ArrowRight' : 'ArrowLeft';
                     const key = {name:?};
                     let to = -1;
                     if (key === fwd || key === 'ArrowDown') to = (at + 1) % buttons.length;
                     else if (key === back || key === 'ArrowUp') to = (at - 1 + buttons.length) % buttons.length;
                     else if (key === 'Home') to = 0;
                     else if (key === 'End') to = buttons.length - 1;
                     if (to >= 0) buttons[to].focus();"
                    ));
                }
                if let Some(handler) = &onkeydown {
                    handler.call(event);
                }
            },
            ..attributes,
            for item in items.iter() {
                {
                    let value = item.value.clone();
                    let pressed = on.contains(&value);
                    let label = match (&item.label, &item.icon) {
                        (Some(label), _) => Some(label.clone()),
                        (None, None) => Some(rsx! { "{value}" }),
                        (None, Some(_)) => None,
                    };
                    let icon_only = label.is_none();
                    let title = if icon_only { item.aria_label.clone() } else { None };
                    let items_for_toggle = items.clone();
                    let on_now = on.clone();
                    let toggle_value = value.clone();
                    let focus_value = value.clone();
                    let is_stop = stop.as_deref() == Some(value.as_str());
                    rsx! {
                        button {
                            key: "{value}",
                            r#type: "button",
                            class: "ui-tgl__item",
                            aria_pressed: if pressed { "true" } else { "false" },
                            aria_label: item.aria_label.clone(),
                            title,
                            "data-icon-only": icon_only.then_some("true"),
                            disabled: item.disabled || disabled,
                            tabindex: if is_stop { "0" } else { "-1" },
                            onfocus: move |_| focused.set(Some(focus_value.clone())),
                            onclick: move |_| {
                                if let Some(next) = next_selection(&items_for_toggle, &on_now, &toggle_value, single) {
                                    if !controlled {
                                        inner.set(next.clone());
                                    }
                                    if let Some(handler) = &on_change {
                                        handler.call(next);
                                    }
                                }
                            },
                            if let Some(icon) = item.icon.clone() {
                                span { class: "ui-tgl__icon", "aria-hidden": "true", {icon} }
                            }
                            if let Some(label) = label {
                                span { class: "ui-tgl__label", dir: "auto", {label} }
                            }
                        }
                    }
                }
            }
        }
    }
}
