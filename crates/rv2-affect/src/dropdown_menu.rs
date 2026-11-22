//! Dropdown Menu: actions or choices that open from any trigger, with submenus that open to the side.
//!
//! `MenuLevel` and `MenuItem` are shared with Context Menu, which passes its own class prefix.
//!
//! Port losses: Hugeicons data objects are not accepted; pass an SVG `Element` as the icon. A pointer
//! click always focuses the first row (the React version focuses the menu itself). The trigger is a
//! render callback that receives `DropdownTrigger` and returns the element to draw. Focus and
//! geometry of rows are read through eval by `data-index`, not through per-row `MountedEvent`s.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};
use crate::popover::{
    Align, Placement, Side, listen, listen_js, placed_style, refresh, release_js,
};

const CSS: Asset = asset!("/assets/dropdown-menu.css");

/// Pause before a submenu opens on hover, and before the open one closes when the pointer moves on.
const SUB_OPEN_MS: u32 = 90;
const SUB_CLOSE_MS: u32 = 220;

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem {
    pub label: String,
    /// A second line in a lighter color.
    pub description: Option<String>,
    /// A 16px glyph before the label.
    pub icon: Option<Element>,
    /// A brand mark in place of the icon, in its own colors.
    pub logo: Option<Element>,
    /// A check: the item becomes a choice. Leave it out for a plain action.
    pub checked: Option<bool>,
    /// Checkbox semantics for a choice; radio when false.
    pub checkbox: bool,
    /// Red text and icon, e.g. Delete.
    pub danger: bool,
    /// Shown only; wire the keys up yourself.
    pub shortcut: Option<String>,
    /// A line above this item.
    pub divider: bool,
    pub disabled: bool,
    /// The menu stays open after a pick.
    pub keep_open: bool,
    /// Items that open to the side on hover, →, Enter or Space.
    pub submenu: Option<Vec<MenuItem>>,
    pub onselect: Option<EventHandler<()>>,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            icon: None,
            logo: None,
            checked: None,
            checkbox: false,
            danger: false,
            shortcut: None,
            divider: false,
            disabled: false,
            keep_open: false,
            submenu: None,
            onselect: None,
        }
    }
}

/// What the menu does with focus when it closes: keep it, move it to the trigger after a beat, or now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusBack {
    Keep,
    Later,
    Now,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RovingStep {
    Next,
    Previous,
    First,
    Last,
}

/// The next position in a list of `len` focusable rows. Arrows wrap; from nothing, Next is the first
/// and Previous is the last.
pub fn roving_index(len: usize, current: Option<usize>, step: RovingStep) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match step {
        RovingStep::Next => current.map_or(0, |c| (c + 1) % len),
        RovingStep::Previous => current.map_or(len - 1, |c| (c + len - 1) % len),
        RovingStep::First => 0,
        RovingStep::Last => len - 1,
    })
}

/// The next focusable row whose label starts with `query` (case-insensitive). A single letter starts
/// after `at`, so repeating it cycles; a longer query starts at `at`.
pub fn menu_typeahead(labels: &[&str], query: &str, at: Option<usize>) -> Option<usize> {
    let n = labels.len() as isize;
    if n == 0 {
        return None;
    }
    let q = query.to_lowercase();
    let at = at.map_or(-1, |a| a as isize);
    let from = if query.chars().count() == 1 {
        at + 1
    } else {
        at.max(0)
    };
    (0..n)
        .map(|k| (from + k).rem_euclid(n) as usize)
        .find(|&i| labels[i].to_lowercase().starts_with(&q))
}

/// Shows a menu panel and moves the focus to its first row, or its last after ArrowUp. Visibility is
/// set first: a `visibility: hidden` element cannot take focus, and it is hidden until placed.
pub(crate) fn open_focus_script(panel_id: &str, menu_id: &str, last: bool) -> String {
    let row = if last { "rows.at(-1)" } else { "rows[0]" };
    format!(
        "const panel = document.getElementById({panel_id:?}); \
         if (panel) {{ panel.style.visibility = 'visible'; panel.showPopover?.(); }} \
         const rows = [...(document.getElementById({menu_id:?})?.querySelectorAll(':scope > [role^=\"menuitem\"]:not([aria-disabled=\"true\"])') ?? [])]; \
         {row}?.focus({{ preventScroll: true }});"
    )
}

/// What the render callback of a Dropdown Menu receives to draw its trigger.
#[derive(Clone, PartialEq)]
pub struct DropdownTrigger {
    pub id: String,
    pub expanded: bool,
    pub controls: Option<String>,
    pub toggle: EventHandler<MouseEvent>,
}

#[component]
pub fn DropdownMenu(
    trigger: Callback<DropdownTrigger, Element>,
    items: Vec<MenuItem>,
    #[props(default)] align: Align,
    #[props(default)] side: Side,
    #[props(default)] title: Option<String>,
    #[props(default)] header: Option<Element>,
    #[props(default)] open: Option<bool>,
    #[props(default)] default_open: bool,
    #[props(default)] onopenchange: Option<EventHandler<bool>>,
    #[props(default)] menu_class: String,
    #[props(default, into)] width: Option<String>,
    #[props(default = "Nothing here yet".to_string(), into)] empty: String,
    #[props(default)] class: String,
    #[props(default, into)] aria_label: Option<String>,
) -> Element {
    let uid = use_hook(next_id);
    let trigger_id = format!("ui-dmc-{uid}-trigger");
    let menu_id = format!("ui-dmc-{uid}-menu");
    let title_id = format!("ui-dmc-{uid}-title");
    let anchor_id = format!("ui-dmc-{uid}-anchor");
    let panel_id = format!("ui-dmc-{uid}-panel");
    let mut own = use_signal(|| default_open);
    let is_open = open.unwrap_or(own());
    let mut placed = use_signal(|| None::<Placement>);
    let mut card = use_signal(|| None::<MountedEvent>);
    let mut start_at_last = use_signal(|| false);
    let anchor = format!("document.getElementById({anchor_id:?})");

    let mut set_open = move |next: bool| {
        if open.is_none() {
            own.set(next);
        }
        if next != is_open
            && let Some(handler) = onopenchange
        {
            handler.call(next);
        }
    };

    let close_menu = {
        let trigger_id = trigger_id.clone();
        move |back: FocusBack| {
            set_open(false);
            match back {
                FocusBack::Keep => {}
                FocusBack::Now => {
                    document::eval(&format!(
                        "document.getElementById({trigger_id:?})?.focus({{ preventScroll: true }});"
                    ));
                }
                FocusBack::Later => {
                    let trigger_id = trigger_id.clone();
                    spawn(async move {
                        sleep(0).await;
                        document::eval(&format!(
                            "const t = document.getElementById({trigger_id:?}); const at = document.activeElement; \
                             if (t && (!at || at === document.body)) t.focus({{ preventScroll: true }});"
                        ));
                    });
                }
            }
        }
    };

    let close_handler = EventHandler::new(close_menu);
    let trigger_props = DropdownTrigger {
        id: trigger_id.clone(),
        expanded: is_open,
        controls: is_open.then(|| menu_id.clone()),
        toggle: EventHandler::new(move |_: MouseEvent| {
            start_at_last.set(false);
            set_open(!is_open);
        }),
    };

    let setup = format!(
        r#"const anchor = () => document.getElementById({anchor_id:?});
const panel = () => document.getElementById({panel_id:?});
on(document, "pointerdown", (e) => {{ if (!anchor()?.contains(e.target)) send("outside"); }}, true);
on(document, "focusout", (e) => {{
  const inside = (n) => !!n && (!!anchor()?.contains(n) || !!panel()?.contains(n));
  if (inside(e.target) && e.relatedTarget && !inside(e.relatedTarget)) send("blur");
}});
on(window, "resize", () => send("move"));
on(window, "scroll", () => send("move"), true);"#
    );

    use_effect(use_reactive((&is_open,), {
        let panel_id = panel_id.clone();
        let anchor = anchor.clone();
        move |(now_open,)| {
            if !now_open {
                placed.set(None);
                document::eval(&release_js(&panel_id));
                return;
            }
            let anchor = anchor.clone();
            listen(listen_js(&panel_id, &setup), move |message| {
                match message.as_str() {
                    "move" => refresh(card, placed, anchor.clone(), side, align, 6.0),
                    "outside" | "blur" => set_open(false),
                    _ => {}
                }
            });
        }
    }));

    let open_from_key = move |event: Event<KeyboardData>| {
        if is_open || !matches!(event.key(), Key::ArrowDown | Key::ArrowUp) {
            return;
        }
        event.prevent_default();
        start_at_last.set(event.key() == Key::ArrowUp);
        set_open(true);
    };

    let style = placed_style(placed(), width.as_deref());
    let placed_now = placed();
    let shown_side = placed_now.map_or(side, |p| p.side);
    let shown_align = placed_now.map_or(align, |p| p.align);
    let last = start_at_last();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        span {
            id: "{anchor_id}",
            class: class_list("ui-dmc", &class),
            onkeydown: open_from_key,
            {trigger.call(trigger_props)}
            if is_open {
                div {
                    id: "{panel_id}",
                    class: class_list("ui-dmc__panel", &menu_class),
                    popover: "manual",
                    "data-side": shown_side.as_str(),
                    "data-align": shown_align.as_str(),
                    style: "{style}",
                    onmounted: {
                        let anchor = anchor.clone();
                        let focus = open_focus_script(&panel_id, &menu_id, last);
                        move |event: MountedEvent| {
                            card.set(Some(event.clone()));
                            refresh(card, placed, anchor.clone(), side, align, 6.0);
                            document::eval(&focus);
                        }
                    },
                    {header}
                    if let Some(title) = title.clone() {
                        span { class: "ui-dmc__title", id: "{title_id}", "{title}" }
                    }
                    MenuLevel {
                        prefix: "ui-dmc",
                        id: menu_id.clone(),
                        items: items.clone(),
                        empty: empty.clone(),
                        labelled_by: title.as_ref().map(|_| title_id.clone()).or_else(|| aria_label.is_none().then(|| trigger_id.clone())),
                        label: if title.is_some() { None } else { aria_label.clone() },
                        start_last: last,
                        on_pick: move |_| close_handler.call(FocusBack::Later),
                        on_close: close_handler,
                    }
                }
            }
        }
    }
}

/// One level of a menu: the root, or a submenu. Arrow keys move through the focusable rows, typing
/// letters jumps to a row, → or Enter opens a submenu, ← or Escape goes back out of one.
#[component]
pub fn MenuLevel(
    prefix: &'static str,
    id: String,
    items: Vec<MenuItem>,
    #[props(default = "Nothing here yet".to_string(), into)] empty: String,
    #[props(default)] labelled_by: Option<String>,
    #[props(default)] label: Option<String>,
    #[props(default)] start_last: bool,
    on_pick: EventHandler<()>,
    on_close: EventHandler<FocusBack>,
    #[props(default)] on_back: Option<EventHandler<()>>,
) -> Element {
    let sub_panel_id = format!("{id}-sub");
    let sub_list_id = format!("{id}-sub-list");
    let mut glow = use_signal(|| None::<(f64, f64, bool)>);
    let mut sub = use_signal(|| None::<(usize, bool)>);
    let mut focused = use_signal(|| None::<usize>);
    let mut pending = use_signal(|| None::<isize>);
    let mut generation = use_signal(|| 0u32);
    let mut typed = use_signal(String::new);
    let mut typed_token = use_signal(|| 0u32);

    let focusable: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, it)| !it.disabled)
        .map(|(i, _)| i)
        .collect();
    let labels: Vec<String> = focusable.iter().map(|&i| items[i].label.clone()).collect();
    let list_id = id.clone();
    let row_js = move |index: usize| {
        format!(
            "document.getElementById({list_id:?})?.querySelector(':scope > [data-index=\"{index}\"]')"
        )
    };

    let focus_row = {
        let row_js = row_js.clone();
        move |index: usize| {
            document::eval(&format!(
                "{}?.focus({{ preventScroll: true }});",
                row_js(index)
            ));
        }
    };

    let mut cancel_later = move || {
        generation.set(generation() + 1);
        pending.set(None);
    };

    let mut open_sub = move |index: usize, focus: bool| {
        cancel_later();
        if sub().is_some_and(|(i, _)| i == index) && !focus {
            return;
        }
        sub.set(Some((index, focus)));
    };

    let mut back = {
        let focus_row = focus_row.clone();
        move || {
            let at = sub().map(|(i, _)| i);
            sub.set(None);
            if let Some(at) = at {
                focus_row(at);
            }
        }
    };

    let mut activate = {
        let items = items.clone();
        move |index: usize, keyboard: bool| {
            let Some(item) = items.get(index).cloned() else {
                return;
            };
            if item.disabled {
                return;
            }
            if item.submenu.is_some() {
                let is_open = sub().is_some_and(|(i, _)| i == index);
                if !keyboard && is_open && !sub().is_some_and(|(_, f)| f) {
                    sub.set(None);
                } else {
                    open_sub(index, keyboard);
                }
                return;
            }
            if let Some(onselect) = item.onselect {
                onselect.call(());
            }
            if !item.keep_open {
                on_pick.call(());
            }
        }
    };

    let mut later = move |ms: u32, what: isize, fire: Box<dyn FnOnce()>| {
        let token = generation() + 1;
        generation.set(token);
        pending.set(Some(what));
        spawn(async move {
            sleep(ms).await;
            if generation() == token {
                pending.set(None);
                fire();
            }
        });
    };

    let typeahead = {
        let labels = labels.clone();
        let focusable = focusable.clone();
        let focus_row = focus_row.clone();
        move |key: String| {
            let token = typed_token() + 1;
            typed_token.set(token);
            let text = if typed().is_empty() {
                key
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
            let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
            let at = focused().and_then(|f| focusable.iter().position(|&i| i == f));
            if let Some(hit) = menu_typeahead(&refs, &text, at) {
                focus_row(focusable[hit]);
            }
        }
    };

    let on_key = {
        let focusable = focusable.clone();
        let items = items.clone();
        let focus_row = focus_row.clone();
        let mut activate = activate.clone();
        let mut back = back.clone();
        let mut typeahead = typeahead.clone();
        let mut open_sub = open_sub;
        move |event: Event<KeyboardData>| {
            let key = event.key();
            let modifiers = event.data().modifiers();
            let at = focused().and_then(|f| focusable.iter().position(|&i| i == f));
            let index = focused();
            let stop = || {
                event.prevent_default();
                event.stop_propagation();
            };
            let mut step = |s: RovingStep| {
                stop();
                sub.set(None);
                if let Some(pos) = roving_index(focusable.len(), at, s) {
                    focus_row(focusable[pos]);
                }
            };
            match key {
                Key::ArrowDown => step(RovingStep::Next),
                Key::ArrowUp => step(RovingStep::Previous),
                Key::Home | Key::PageUp => step(RovingStep::First),
                Key::End | Key::PageDown => step(RovingStep::Last),
                Key::ArrowRight => {
                    if let Some(i) = index.filter(|&i| items[i].submenu.is_some()) {
                        stop();
                        open_sub(i, true);
                    }
                }
                Key::ArrowLeft => {
                    if on_back.is_some() {
                        stop();
                        back();
                    }
                }
                Key::Escape => {
                    stop();
                    match on_back {
                        Some(_) => back(),
                        None => on_close.call(FocusBack::Now),
                    }
                }
                Key::Enter => {
                    stop();
                    if let Some(i) = index {
                        activate(i, true);
                    }
                }
                Key::Tab => on_close.call(FocusBack::Now),
                Key::Character(c) if c == " " => {
                    stop();
                    if !typed().is_empty() {
                        typeahead(" ".to_string());
                    } else if let Some(i) = index {
                        activate(i, true);
                    }
                }
                Key::Character(c)
                    if c.chars().count() == 1
                        && !modifiers.meta()
                        && !modifiers.ctrl()
                        && !modifiers.alt() =>
                {
                    stop();
                    typeahead(c);
                }
                _ => {}
            }
        }
    };

    let on_row_focus = EventHandler::new(move |i: usize| focused.set(Some(i)));
    let on_row_activate = EventHandler::new(move |i: usize| activate(i, false));
    let hover_items = items.clone();
    let on_row_hover = EventHandler::new(move |i: usize| {
        let Some(item) = hover_items.get(i) else {
            return;
        };
        if item.disabled {
            return;
        }
        if item.submenu.is_some() {
            if sub().is_some_and(|(s, _)| s == i) {
                cancel_later();
            } else if pending() != Some(i as isize) {
                later(
                    SUB_OPEN_MS,
                    i as isize,
                    Box::new(move || open_sub(i, false)),
                );
            }
        } else if sub().is_none() {
            cancel_later();
        } else if pending() != Some(-1) {
            later(SUB_CLOSE_MS, -1, Box::new(move || sub.set(None)));
        }
    });
    let glow_items = items.clone();
    let glow_row_js = row_js.clone();
    use_effect(use_reactive((&focused(),), move |(at,)| {
        let Some(i) = at else {
            glow.set(None);
            return;
        };
        let danger = glow_items.get(i).is_some_and(|it| it.danger);
        let script = format!(
            "const r = {}; return r ? [r.offsetTop, r.offsetHeight] : null;",
            glow_row_js(i)
        );
        spawn(async move {
            if let Ok(v) = document::eval(&script).await
                && let (Some(top), Some(height)) = (
                    v.get(0).and_then(|x| x.as_f64()),
                    v.get(1).and_then(|x| x.as_f64()),
                )
            {
                glow.set(Some((top, height, danger)));
            }
        });
    }));

    let glow_style =
        glow().map(|(top, height, _)| format!("transform:translateY({top}px);height:{height}px;"));
    let glow_danger = glow().is_some_and(|(_, _, d)| d);
    let sub_open = sub();
    let sub_items = sub_open.and_then(|(i, _)| items[i].submenu.clone());
    let sub_label = sub_open.map(|(i, _)| items[i].label.clone());
    let rows: Vec<(usize, MenuItem)> = items.iter().cloned().enumerate().collect();
    let empty_now = items.is_empty();

    rsx! {
        div {
            id: "{id}",
            class: "{prefix}__list",
            role: "menu",
            aria_orientation: "vertical",
            aria_labelledby: labelled_by,
            aria_label: label,
            tabindex: "-1",
            onkeydown: on_key,
            onfocusout: move |_| glow.set(None),
            onpointerleave: move |_| cancel_later(),
            if let Some(style) = glow_style {
                span { class: "{prefix}__glow", "aria-hidden": "true", "data-danger": glow_danger.then_some("true"), style: "{style}" }
            }
            if empty_now {
                span { class: "{prefix}__empty", "{empty}" }
            }
            for (i, item) in rows {
                if item.divider && i > 0 {
                    span { class: "{prefix}__sep", role: "separator" }
                }
                MenuRow {
                    prefix,
                    index: i,
                    item: item.clone(),
                    sub_open: sub_open.is_some_and(|(s, _)| s == i),
                    sub_list_id: sub_list_id.clone(),
                    on_focus: on_row_focus,
                    on_hover: on_row_hover,
                    on_activate: on_row_activate,
                }
            }
            if let (Some(sub_items), Some(sub_label)) = (sub_items, sub_label) {
                SubMenu {
                    prefix,
                    id: sub_panel_id.clone(),
                    list_id: sub_list_id.clone(),
                    anchor: row_js(sub_open.map_or(0, |(i, _)| i)),
                    items: sub_items,
                    focus: sub_open.is_some_and(|(_, f)| f),
                    label: sub_label,
                    on_enter: move |_| cancel_later(),
                    on_back: move |_| back(),
                    on_pick,
                    on_close,
                }
            }
        }
    }
}

#[component]
fn MenuRow(
    prefix: &'static str,
    index: usize,
    item: MenuItem,
    sub_open: bool,
    sub_list_id: String,
    on_focus: EventHandler<usize>,
    on_hover: EventHandler<usize>,
    on_activate: EventHandler<usize>,
) -> Element {
    let choice = item.checked.is_some() && item.submenu.is_none();
    let role = if !choice {
        "menuitem"
    } else if item.checkbox {
        "menuitemcheckbox"
    } else {
        "menuitemradio"
    };
    let checked = item.checked.unwrap_or(false);
    rsx! {
        div {
            class: "{prefix}__item",
            role,
            tabindex: "-1",
            "data-index": "{index}",
            aria_checked: choice.then_some(if checked { "true" } else { "false" }),
            aria_disabled: item.disabled.then_some("true"),
            aria_haspopup: item.submenu.is_some().then_some("menu"),
            aria_expanded: item.submenu.is_some().then(|| sub_open.to_string()),
            aria_controls: (item.submenu.is_some() && sub_open).then_some(sub_list_id),
            "data-danger": item.danger.then_some("true"),
            "data-rich": item.description.is_some().then_some(""),
            "data-open": sub_open.then_some("true"),
            onpointerdown: move |event| event.prevent_default(),
            onfocus: move |_| on_focus.call(index),
            onpointermove: move |_| on_hover.call(index),
            onclick: move |_| on_activate.call(index),
            if item.logo.is_some() || item.icon.is_some() {
                span { class: "{prefix}__icon", "data-logo": item.logo.is_some().then_some(""), "aria-hidden": "true",
                    {item.logo.clone().or(item.icon.clone())}
                }
            }
            span { class: "{prefix}__label",
                span { class: "{prefix}__text", title: "{item.label}", "{item.label}" }
                if let Some(description) = item.description.clone() {
                    span { class: "{prefix}__desc", title: "{description}", "{description}" }
                }
            }
            if let Some(shortcut) = item.shortcut.clone() {
                kbd { class: "{prefix}__kbd", "{shortcut}" }
            }
            if item.submenu.is_some() {
                span { class: "{prefix}__more", "aria-hidden": "true",
                    svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                        path { d: "M9 6s6 4.42 6 6-6 6-6 6" }
                    }
                }
            }
            if choice {
                span { class: "{prefix}__check", "aria-hidden": "true",
                    if checked {
                        svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                            path { class: "{prefix}__tick", d: "M5 14 8.5 17.5 19 6.5", "pathLength": "1" }
                        }
                    }
                }
            }
        }
    }
}

/// A submenu: its own panel beside the row that opened it, or on the other side when there is no room.
#[component]
fn SubMenu(
    prefix: &'static str,
    id: String,
    list_id: String,
    anchor: String,
    items: Vec<MenuItem>,
    focus: bool,
    label: String,
    on_enter: EventHandler<()>,
    on_back: EventHandler<()>,
    on_pick: EventHandler<()>,
    on_close: EventHandler<FocusBack>,
) -> Element {
    let placed = use_signal(|| None::<Placement>);
    let mut card = use_signal(|| None::<MountedEvent>);
    let style = placed_style(placed(), None);
    let first_row = format!(
        "document.getElementById({list_id:?})?.querySelector(':scope > [role^=\"menuitem\"]:not([aria-disabled=\"true\"])')"
    );
    let placed_side = placed().map_or(Side::Right, |p| p.side);
    rsx! {
        div {
            id: "{id}",
            class: "{prefix}__panel",
            "data-sub": "",
            popover: "manual",
            "data-side": placed_side.as_str(),
            style: "{style}",
            onpointerenter: move |_| on_enter.call(()),
            onmounted: {
                let anchor = anchor.clone();
                let first_row = first_row.clone();
                let show = format!("const p = document.getElementById({id:?}); if (p) {{ p.style.visibility = 'visible'; p.showPopover?.(); }}");
                move |event: MountedEvent| {
                    card.set(Some(event.clone()));
                    refresh(card, placed, anchor.clone(), Side::Right, Align::Start, 4.0);
                    document::eval(&show);
                    if focus {
                        document::eval(&format!("{first_row}?.focus({{ preventScroll: true }});"));
                    }
                }
            },
            MenuLevel {
                prefix,
                id: list_id.clone(),
                items,
                label,
                on_pick,
                on_close,
                on_back,
            }
        }
    }
}
