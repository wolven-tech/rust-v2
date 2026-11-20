//! Select: a field that opens a list of options under it (or above it when there is more room there).
//!
//! Port losses: the list is placed on open and on window resize or scroll, not on every layout change.
//! The list's height is measured as its cap (288px), not from its content, so a short list keeps the
//! same top edge as a long one. Typing letters jumps to an option through `typeahead_index`, with the
//! 700ms reset run by a timer.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};
use crate::popover::{Rect, Size, listen, listen_js, release_js};

const CSS: Asset = asset!("/assets/select.css");

/// Gap between the field and the list, and the margin kept from the window edge.
const GAP: f64 = 4.0;
const EDGE: f64 = 8.0;
/// The list's height cap; shorter lists are not stretched to it.
const LIST_CAP: f64 = 288.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl SelectSize {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    /// A quiet second line under the label.
    pub description: Option<String>,
    /// A 16px icon before the label.
    pub icon: Option<Element>,
    pub disabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListPlacement {
    pub up: bool,
    pub top: f64,
    pub left: f64,
    pub width: f64,
    pub max_height: f64,
    pub view_height: f64,
}

/// The next enabled index from `from` in `dir` (1 or -1). Stays at `from` when nothing is enabled that way.
pub fn select_step(from: isize, enabled: &[bool], dir: isize) -> isize {
    let mut i = from + dir;
    while i >= 0 && (i as usize) < enabled.len() {
        if enabled[i as usize] {
            return i;
        }
        i += dir;
    }
    from
}

/// The first enabled option whose label starts with `query` (case-insensitive), searching from `start`.
/// A single letter moves past `start`, so repeating it cycles; a longer query matches from `start` itself.
pub fn typeahead_index(labels: &[(&str, bool)], query: &str, start: isize) -> Option<usize> {
    let n = labels.len() as isize;
    if n == 0 {
        return None;
    }
    let q = query.to_lowercase();
    let bump = if query.chars().count() == 1 { 1 } else { 0 };
    (0..n)
        .map(|k| (k + start + bump).rem_euclid(n) as usize)
        .find(|&i| labels[i].1 && labels[i].0.to_lowercase().starts_with(&q))
}

/// Where the list goes: under the field, or above it when there is more room there and the room
/// below is short; as wide as the field (at least 180px), kept inside the window horizontally.
pub fn list_placement(trigger: Rect, want: f64, view: Size) -> ListPlacement {
    let below = view.height - trigger.bottom() - GAP - EDGE;
    let above = trigger.top - GAP - EDGE;
    let up = below < want.min(160.0) && above > below;
    let width = trigger.width.max(180.0);
    let left = trigger.left.max(EDGE).min(view.width - width - EDGE);
    ListPlacement {
        up,
        top: if up {
            trigger.top - GAP
        } else {
            trigger.bottom() + GAP
        },
        left,
        width,
        max_height: 120f64.max(LIST_CAP.min(if up { above } else { below })),
        view_height: view.height,
    }
}

#[component]
pub fn Select(
    options: Vec<SelectOption>,
    /// The chosen value; makes the select controlled.
    #[props(default, into)]
    value: Option<String>,
    #[props(default, into)] default_value: Option<String>,
    #[props(default)] onchange: Option<EventHandler<SelectOption>>,
    #[props(default = "Choose…".to_string(), into)] placeholder: String,
    #[props(default)] label: Option<Element>,
    #[props(default)] hint: Option<Element>,
    #[props(default)] error: Option<Element>,
    #[props(default)] optional: bool,
    #[props(default)] size: SelectSize,
    #[props(default)] disabled: bool,
    #[props(default, into)] name: Option<String>,
    #[props(default)] required: bool,
    #[props(default, into)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default, into)] aria_label: Option<String>,
) -> Element {
    let uid = use_hook(next_id);
    let trigger_id = id
        .clone()
        .unwrap_or_else(|| format!("ui-sel-{uid}-trigger"));
    let list_id = format!("ui-sel-{uid}-list");
    let label_id = format!("ui-sel-{uid}-label");
    let message_id = format!("ui-sel-{uid}-message");
    let option_id = |i: usize| format!("ui-sel-{uid}-option-{i}");

    let mut own = use_signal(|| default_value.clone());
    let value_prop = value.clone();
    let shown_value = value.clone().or(own());
    let chosen = options
        .iter()
        .find(|o| Some(&o.value) == shown_value.as_ref())
        .cloned();
    let mut open = use_signal(|| false);
    let mut active = use_signal(|| -1isize);
    let mut place = use_signal(|| None::<ListPlacement>);
    let mut glow = use_signal(|| None::<(f64, f64)>);
    let mut trigger = use_signal(|| None::<MountedEvent>);
    let mut typed = use_signal(String::new);
    let mut typed_token = use_signal(|| 0u32);

    let invalid = error.is_some();
    let message = if error.is_some() {
        error.clone()
    } else {
        hint.clone()
    };
    let enabled: Vec<bool> = options.iter().map(|o| !o.disabled).collect();
    let chosen_index = options
        .iter()
        .position(|o| Some(&o.value) == shown_value.as_ref());

    let mut show = {
        let enabled = enabled.clone();
        move |at: Option<isize>| {
            if disabled || enabled.is_empty() {
                return;
            }
            let first = enabled.iter().position(|e| *e).map_or(-1, |i| i as isize);
            let chosen = chosen_index.map_or(-1, |i| i as isize);
            let start = at.unwrap_or(if chosen >= 0 && enabled[chosen as usize] {
                chosen
            } else {
                first
            });
            active.set(start);
            open.set(true);
        }
    };
    let mut hide = move |focus_back: bool| {
        open.set(false);
        glow.set(None);
        place.set(None);
        if focus_back && let Some(node) = trigger() {
            spawn(async move {
                let _ = node.set_focus(true).await;
            });
        }
    };
    let pick = {
        let options = options.clone();
        let value = shown_value.clone();
        let controlled = value_prop.is_some();
        move |i: usize| {
            let Some(option) = options.get(i).filter(|o| !o.disabled).cloned() else {
                return;
            };
            if !controlled {
                own.set(Some(option.value.clone()));
            }
            if value.as_deref() != Some(option.value.as_str())
                && let Some(handler) = onchange
            {
                handler.call(option);
            }
            hide(true);
        }
    };

    let measure = move || {
        let Some(node) = trigger() else {
            return;
        };
        spawn(async move {
            let Ok(rect) = node.get_client_rect().await else {
                return;
            };
            let trigger_rect = Rect {
                left: rect.origin.x,
                top: rect.origin.y,
                width: rect.size.width,
                height: rect.size.height,
            };
            let view = document::eval("return [window.innerWidth, window.innerHeight];")
                .await
                .ok();
            let view = view.and_then(|v| {
                Some(Size {
                    width: v.get(0)?.as_f64()?,
                    height: v.get(1)?.as_f64()?,
                })
            });
            if let Some(view) = view {
                place.set(Some(list_placement(trigger_rect, LIST_CAP, view)));
            }
        });
    };

    let setup = format!(
        r#"on(document, "pointerdown", (e) => {{
  const list = document.getElementById({list_id:?});
  const trigger = document.getElementById({trigger_id:?});
  if (!trigger?.contains(e.target) && !list?.contains(e.target)) send("close");
}}, true);
on(window, "resize", () => send("move"));
on(window, "scroll", () => send("move"), true);"#
    );

    use_effect(use_reactive((&open(),), {
        let list_id = list_id.clone();
        move |(is_open,)| {
            if !is_open {
                document::eval(&release_js(&list_id));
                return;
            }
            listen(listen_js(&list_id, &setup), move |message| {
                match message.as_str() {
                    "move" => measure(),
                    "close" => hide(false),
                    _ => {}
                }
            });
        }
    }));

    use_effect(use_reactive((&active(), &open()), {
        let list_id = list_id.clone();
        move |(at, is_open)| {
            if !is_open || at < 0 {
                return;
            }
            let script = format!(
                "const row = document.getElementById({list_id:?})?.querySelector('[data-index=\"{at}\"]'); \
                 if (!row) return null; row.scrollIntoView({{ block: 'nearest' }}); return [row.offsetTop, row.offsetHeight];"
            );
            spawn(async move {
                if let Ok(v) = document::eval(&script).await
                    && let (Some(top), Some(height)) = (
                        v.get(0).and_then(|x| x.as_f64()),
                        v.get(1).and_then(|x| x.as_f64()),
                    )
                {
                    glow.set(Some((top, height)));
                }
            });
        }
    }));

    let typeahead = {
        let labels: Vec<(String, bool)> = options
            .iter()
            .map(|o| (o.label.clone(), !o.disabled))
            .collect();
        let mut pick = pick.clone();
        move |key: String| {
            let token = typed_token() + 1;
            typed_token.set(token);
            let text = if typed().is_empty() {
                key.clone()
            } else {
                format!("{}{key}", typed())
            };
            typed.set(text.clone());
            spawn(async move {
                sleep(700).await;
                if typed_token() == token {
                    typed.set(String::new());
                }
            });
            let refs: Vec<(&str, bool)> = labels.iter().map(|(l, e)| (l.as_str(), *e)).collect();
            let start = if open() {
                active()
            } else {
                chosen_index.map_or(-1, |i| i as isize)
            };
            if let Some(hit) = typeahead_index(&refs, &text, start) {
                if open() {
                    active.set(hit as isize);
                } else {
                    pick(hit);
                }
            }
        }
    };

    let on_key = {
        let enabled = enabled.clone();
        let mut pick = pick.clone();
        let mut typeahead = typeahead.clone();
        let mut show = show.clone();
        move |event: Event<KeyboardData>| {
            if disabled {
                return;
            }
            let key = event.key();
            let plain_char = matches!(&key, Key::Character(c) if c.chars().count() == 1)
                && !event.modifiers().meta()
                && !event.modifiers().ctrl()
                && !event.modifiers().alt();
            if !open() {
                if matches!(key, Key::ArrowDown | Key::ArrowUp | Key::Enter)
                    || key == Key::Character(" ".into())
                {
                    event.prevent_default();
                    show(None);
                } else if let (true, Key::Character(c)) = (plain_char, &key) {
                    typeahead(c.clone());
                }
                return;
            }
            let at = active();
            match key {
                Key::ArrowDown => {
                    event.prevent_default();
                    active.set(select_step(at, &enabled, 1));
                }
                Key::ArrowUp => {
                    event.prevent_default();
                    active.set(select_step(at, &enabled, -1));
                }
                Key::Home | Key::PageUp => {
                    event.prevent_default();
                    active.set(enabled.iter().position(|e| *e).map_or(-1, |i| i as isize));
                }
                Key::End | Key::PageDown => {
                    event.prevent_default();
                    active.set(enabled.iter().rposition(|e| *e).map_or(-1, |i| i as isize));
                }
                Key::Enter => {
                    event.prevent_default();
                    pick(at.max(0) as usize);
                }
                Key::Escape => {
                    event.prevent_default();
                    hide(true);
                }
                Key::Tab => hide(false),
                Key::Character(c) if c == " " => {
                    event.prevent_default();
                    if !typed().is_empty() {
                        typeahead(" ".into());
                    } else {
                        pick(at.max(0) as usize);
                    }
                }
                Key::Character(c) if plain_char => typeahead(c),
                _ => {}
            }
        }
    };

    let shown_side = place().map(|p| p.up);
    let list_style = place().map(|p| {
        let vertical = if p.up {
            format!("bottom:{}px;", (p.view_height - p.top).round())
        } else {
            format!("top:{}px;", p.top.round())
        };
        format!(
            "{vertical}left:{}px;width:{}px;max-height:{}px;",
            p.left.round(),
            p.width.round(),
            p.max_height.round()
        )
    });
    let glow_style =
        glow().map(|(top, height)| format!("transform:translateY({top}px);height:{height}px;"));
    let is_open = open();
    let has_label = label.is_some();
    let label_for_list = has_label.then(|| label_id.clone());
    let trig_labelled = if aria_label.is_some() {
        None
    } else if label.is_some() {
        Some(format!("{label_id} {trigger_id}"))
    } else {
        None
    };
    let described = message.is_some().then(|| message_id.clone());
    let chosen_icon = chosen.as_ref().and_then(|c| c.icon.clone());
    let chosen_label = chosen.as_ref().map(|c| c.label.clone());
    let selected_value = shown_value.clone().unwrap_or_default();
    let active_descendant = (is_open && active() >= 0).then(|| option_id(active() as usize));
    let names: Vec<(usize, SelectOption)> = options.iter().cloned().enumerate().collect();
    let pick_for_rows = pick.clone();
    let shown_value_for_rows = shown_value.clone();
    let open_now = is_open;
    let mount_list = list_id.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: class_list("ui-sel", &class),
            "data-size": size.as_str(),
            "data-open": open_now.then_some("true"),
            "data-invalid": invalid.then_some("true"),
            "data-disabled": disabled.then_some("true"),
            if let Some(label) = label {
                div { class: "ui-sel__head",
                    label { class: "ui-sel__label", id: "{label_id}", r#for: "{trigger_id}", {label} }
                    if optional {
                        span { class: "ui-sel__optional", "Optional" }
                    }
                }
            }
            button {
                id: "{trigger_id}",
                r#type: "button",
                class: "ui-sel__trigger",
                role: "combobox",
                aria_haspopup: "listbox",
                aria_expanded: "{open_now}",
                aria_controls: "{list_id}",
                aria_activedescendant: active_descendant,
                aria_label: aria_label.clone(),
                aria_labelledby: trig_labelled,
                aria_describedby: described.clone(),
                aria_invalid: invalid.then_some("true"),
                aria_required: required.then_some("true"),
                disabled,
                onmounted: move |node: MountedEvent| trigger.set(Some(node)),
                onclick: move |_| if open() { hide(true) } else { show(None) },
                onkeydown: on_key,
                span { class: "ui-sel__value", "data-empty": chosen.is_none().then_some(""), title: chosen_label.clone().unwrap_or_default(),
                    if let Some(icon) = chosen_icon {
                        span { class: "ui-sel__icon", "aria-hidden": "true", {icon} }
                    }
                    {chosen_label.clone().unwrap_or(placeholder.clone())}
                }
                span { class: "ui-sel__chevron", "aria-hidden": "true",
                    svg { view_box: "0 0 24 24", width: "14", height: "14", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                        path { d: "m7 10 5 5 5-5" }
                    }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name, value: "{selected_value}" }
            }
            if is_open {
                div {
                    id: "{list_id}",
                    class: "ui-sel__list",
                    role: "listbox",
                    aria_labelledby: label_for_list,
                    aria_label: if !has_label { aria_label.clone().or(Some(placeholder.clone())) } else { None },
                    popover: "manual",
                    "data-up": shown_side.filter(|up| *up).map(|_| "true"),
                    style: "{list_style.clone().unwrap_or_default()}",
                    onpointerleave: move |_| active.set(-1),
                    onmounted: move |_| {
                        document::eval(&format!("document.getElementById({mount_list:?})?.showPopover?.();"));
                        measure();
                    },
                    if let Some(glow_style) = glow_style {
                        span { class: "ui-sel__glow", "aria-hidden": "true", style: "{glow_style}" }
                    }
                    for (i, option) in names {
                        div {
                            key: "{option.value}",
                            id: "{option_id(i)}",
                            class: "ui-sel__option",
                            role: "option",
                            "data-index": "{i}",
                            aria_selected: "{shown_value_for_rows.as_deref() == Some(option.value.as_str())}",
                            aria_disabled: option.disabled.then_some("true"),
                            "data-active": (active() == i as isize).then_some("true"),
                            onpointermove: move |_| {
                                if !option.disabled && active() != i as isize {
                                    active.set(i as isize);
                                }
                            },
                            onpointerdown: move |event| event.prevent_default(),
                            onclick: {
                                let mut pick = pick_for_rows.clone();
                                move |_| pick(i)
                            },
                            if let Some(icon) = option.icon.clone() {
                                span { class: "ui-sel__icon", "aria-hidden": "true", {icon} }
                            }
                            span { class: "ui-sel__text",
                                span { class: "ui-sel__option-label", "{option.label}" }
                                if let Some(description) = option.description.clone() {
                                    span { class: "ui-sel__option-hint", "{description}" }
                                }
                            }
                            span { class: "ui-sel__check", "aria-hidden": "true",
                                if shown_value_for_rows.as_deref() == Some(option.value.as_str()) {
                                    svg { view_box: "0 0 24 24", width: "14", height: "14", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                                        path { class: "ui-sel__tick", d: "M5 12.5 9.5 17 19 7", "pathLength": "1" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(message) = message {
                p {
                    class: "ui-sel__message",
                    id: "{message_id}",
                    aria_live: if invalid { Some("polite") } else { None },
                    {message}
                }
            }
        }
    }
}
