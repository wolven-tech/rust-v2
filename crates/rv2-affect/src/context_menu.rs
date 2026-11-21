//! Context Menu: the menu a right-click, a long press or Shift+F10 opens at the pointer or the focused element.
//!
//! Rows, submenus and typeahead are the Dropdown Menu's `MenuLevel` with the `ui-ctx` class prefix.
//!
//! Port losses: `items` is a list, not a function of the element under the pointer; build the list in the
//! parent. The browser menu shows when the list is empty. A scroll of the area itself is not told apart
//! from a scroll of the page: any scroll outside the menu closes it. The long-press timer is 500ms and the
//! contextmenu event that follows a touch or a key open is swallowed for 600ms.

use dioxus::prelude::*;

use crate::dropdown_menu::{FocusBack, MenuItem, MenuLevel};
use crate::internal::{class_list, next_id, sleep};
use crate::popover::{Align, Placement, Rect, Side, Size, listen, listen_js, place, placed_style};

const CSS: Asset = asset!("/assets/context-menu.css");

/// How long a finger rests before the menu opens, and how far it may drift meanwhile.
const PRESS_MS: u32 = 500;
const SLOP: f64 = 10.0;
/// The contextmenu event the platform sends after a touch or a key already opened the menu is swallowed.
const SUPPRESS_MS: u32 = 600;

/// Shift+F10 and the menu key.
pub fn is_menu_key(key: &Key, shift: bool) -> bool {
    matches!(key, Key::ContextMenu) || (shift && *key == Key::F10)
}

/// Where the menu opens from the focused element: a little in from its start edge, under its top.
pub fn keyboard_origin(rect: Rect, rtl: bool) -> (f64, f64) {
    let inset = 12f64.min(rect.width / 2.0);
    let x = if rtl {
        rect.right() - inset
    } else {
        rect.left + inset
    };
    (x, rect.top + rect.height.min(26.0))
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Opening {
    x: f64,
    y: f64,
    keyboard: bool,
    n: u32,
}

#[component]
pub fn ContextMenu(
    children: Element,
    items: Vec<MenuItem>,
    #[props(default, into)] title: Option<String>,
    #[props(default = "Actions".to_string(), into)] aria_label: String,
    #[props(default)] disabled: bool,
    #[props(default)] onopenchange: Option<EventHandler<bool>>,
    #[props(default, into)] width: Option<String>,
    #[props(default)] menu_class: String,
) -> Element {
    let uid = use_hook(next_id);
    let area_id = format!("ui-ctx-{uid}-area");
    let has_items = !items.is_empty();
    let mut at = use_signal(|| None::<Opening>);
    let mut count = use_signal(|| 0u32);
    let mut press = use_signal(|| None::<(f64, f64, u32)>);
    let mut press_token = use_signal(|| 0u32);
    let mut suppress = use_signal(|| false);
    let mut area = use_signal(|| None::<MountedEvent>);

    let mut open_at = move |x: f64, y: f64, keyboard: bool| {
        if disabled || !has_items {
            return false;
        }
        let n = count() + 1;
        count.set(n);
        at.set(Some(Opening { x, y, keyboard, n }));
        if let Some(handler) = onopenchange {
            handler.call(true);
        }
        true
    };

    let mut arm_suppress = move || {
        suppress.set(true);
        spawn(async move {
            sleep(SUPPRESS_MS).await;
            suppress.set(false);
        });
    };

    let close = EventHandler::new({
        let area_id = area_id.clone();
        move |back: FocusBack| {
            at.set(None);
            if let Some(handler) = onopenchange {
                handler.call(false);
            }
            match back {
                FocusBack::Keep => {}
                FocusBack::Now => {
                    document::eval(&format!(
                        "document.getElementById({area_id:?})?.querySelector('button, [href], input, [tabindex]')?.focus({{ preventScroll: true }});"
                    ));
                }
                FocusBack::Later => {
                    let area_id = area_id.clone();
                    spawn(async move {
                        sleep(0).await;
                        document::eval(&format!(
                            "const a = document.getElementById({area_id:?}); const at = document.activeElement; \
                             if (a && (!at || at === document.body)) a.querySelector('button, [href], input, [tabindex]')?.focus({{ preventScroll: true }});"
                        ));
                    });
                }
            }
        }
    });

    let open_from_key = move |event: Event<KeyboardData>| {
        if disabled || !is_menu_key(&event.key(), event.modifiers().shift()) {
            return;
        }
        event.prevent_default();
        let Some(node) = area() else {
            return;
        };
        spawn(async move {
            let Ok(rect) = node.get_client_rect().await else {
                return;
            };
            let rect = Rect {
                left: rect.origin.x,
                top: rect.origin.y,
                width: rect.size.width,
                height: rect.size.height,
            };
            let rtl = document::eval(
                "return getComputedStyle(document.documentElement).direction === 'rtl';",
            )
            .await
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            let (x, y) = keyboard_origin(rect, rtl);
            if open_at(x, y, true) {
                arm_suppress();
            }
        });
    };

    let on_context = move |event: Event<MouseData>| {
        if suppress() {
            event.prevent_default();
            return;
        }
        let point = event.data().client_coordinates();
        if open_at(point.x, point.y, false) {
            event.prevent_default();
        }
    };

    let on_down = move |event: Event<PointerData>| {
        if event.data().pointer_type() != "touch" || disabled {
            return;
        }
        let point = event.data().client_coordinates();
        let token = press_token() + 1;
        press_token.set(token);
        press.set(Some((point.x, point.y, token)));
        spawn(async move {
            sleep(PRESS_MS).await;
            if press().is_some_and(|(_, _, t)| t == token) {
                press.set(None);
                if open_at(point.x, point.y, false) {
                    arm_suppress();
                }
            }
        });
    };
    let on_move = move |event: Event<PointerData>| {
        if let Some((x, y, _)) = press() {
            let point = event.data().client_coordinates();
            if ((point.x - x).powi(2) + (point.y - y).powi(2)).sqrt() > SLOP {
                press.set(None);
            }
        }
    };
    let on_up = move |_: Event<PointerData>| press.set(None);

    let opening = at();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{area_id}",
            style: "display: contents; -webkit-touch-callout: none;",
            onmounted: move |event: MountedEvent| area.set(Some(event)),
            oncontextmenu: on_context,
            onkeydown: open_from_key,
            onpointerdown: on_down,
            onpointermove: on_move,
            onpointerup: on_up,
            onpointercancel: on_up,
            {children}
        }
        for opened in opening {
            ContextPanel {
                key: "{opened.n}",
                id: format!("ui-ctx-{uid}-menu-{}", opened.n),
                x: opened.x,
                y: opened.y,
                keyboard: opened.keyboard,
                items: items.clone(),
                title: title.clone(),
                label: aria_label.clone(),
                width: width.clone(),
                menu_class: menu_class.clone(),
                on_close: close,
            }
        }
    }
}

#[component]
fn ContextPanel(
    id: String,
    x: f64,
    y: f64,
    keyboard: bool,
    items: Vec<MenuItem>,
    title: Option<String>,
    label: String,
    width: Option<String>,
    menu_class: String,
    on_close: EventHandler<FocusBack>,
) -> Element {
    let mut placed = use_signal(|| None::<Placement>);
    let title_id = format!("{id}-title");
    let list_id = format!("{id}-list");
    let style = placed_style(placed(), width.as_deref());
    let panel_id = id.clone();

    let setup = format!(
        r#"const panel = () => document.getElementById({panel_id:?});
on(document, "pointerdown", (e) => {{ if (!panel()?.contains(e.target)) send("close"); }}, true);
// Focusing the panel on open can scroll the page; that scroll must not close it.
const opened = performance.now();
on(window, "scroll", (e) => {{ if (performance.now() - opened > 250 && !panel()?.contains(e.target)) send("close"); }}, true);
on(window, "resize", () => send("close"));
on(window, "blur", () => send("close"));"#
    );
    use_effect(use_reactive((&panel_id,), {
        let panel_id = panel_id.clone();
        move |(_,)| {
            listen(listen_js(&panel_id, &setup), move |message| {
                if message == "close" {
                    on_close.call(FocusBack::Keep);
                }
            });
        }
    }));

    let focus_script = if keyboard {
        format!(
            "document.getElementById({list_id:?})?.querySelector(':scope > [role^=\"menuitem\"]:not([aria-disabled=\"true\"])')?.focus({{ preventScroll: true }});"
        )
    } else {
        format!("document.getElementById({list_id:?})?.focus({{ preventScroll: true }});")
    };
    let shown_side = placed().map_or(Side::Bottom, |p| p.side);
    let shown_align = placed().map_or(Align::Start, |p| p.align);
    let labelled_by = title.as_ref().map(|_| title_id.clone());
    let menu_label = if title.is_some() { None } else { Some(label) };

    rsx! {
        div {
            id: "{panel_id}",
            class: class_list("ui-ctx ui-ctx__panel", &menu_class),
            popover: "manual",
            "data-side": shown_side.as_str(),
            "data-align": shown_align.as_str(),
            style: "{style}",
            onclick: move |event| event.stop_propagation(),
            ondoubleclick: move |event| event.stop_propagation(),
            onpointerdown: move |event| event.stop_propagation(),
            onmousedown: move |event| event.stop_propagation(),
            oncontextmenu: move |event| {
                event.prevent_default();
                event.stop_propagation();
            },
            onmounted: move |event: MountedEvent| {
                let node = event.clone();
                spawn(async move {
                    let Ok(rect) = node.get_client_rect().await else {
                        return;
                    };
                    let Ok(view) = document::eval("return [window.innerWidth, window.innerHeight, getComputedStyle(document.documentElement).direction === 'rtl' ? 1 : 0];").await else {
                        return;
                    };
                    let (Some(w), Some(h)) = (view.get(0).and_then(|v| v.as_f64()), view.get(1).and_then(|v| v.as_f64())) else {
                        return;
                    };
                    let rtl = view.get(2).and_then(|v| v.as_f64()) == Some(1.0);
                    let card_size = Size { width: rect.size.width, height: rect.size.height };
                    let pointer = Rect { left: x, top: y, width: 0.0, height: 0.0 };
                    let align = if rtl { Align::End } else { Align::Start };
                    placed.set(Some(place(pointer, card_size, Size { width: w, height: h }, Side::Bottom, align, 0.0, rtl)));
                });
                document::eval(&format!(
                    "const p = document.getElementById({panel_id:?}); if (p) {{ p.style.visibility = 'visible'; p.showPopover?.(); }} {focus_script}"
                ));
            },
            if let Some(title) = title {
                span { class: "ui-ctx__title", id: "{title_id}", "{title}" }
            }
            MenuLevel {
                prefix: "ui-ctx",
                id: list_id.clone(),
                items,
                labelled_by,
                label: menu_label,
                on_pick: move |_| on_close.call(FocusBack::Later),
                on_close,
            }
        }
    }
}
