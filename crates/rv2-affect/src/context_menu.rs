//! Context Menu: the menu a right-click, a long press or Shift+F10 opens at the pointer or the focused element.
//!
//! Rows, submenus and typeahead are the Dropdown Menu's `MenuLevel` with the `ui-ctxc` class prefix.
//!
//! Port losses: `items` is a list, not a function of the element under the pointer; build the list in the
//! parent. The browser menu shows when the list is empty. A scroll of the area itself is not told apart
//! from a scroll of the page: any scroll outside the menu closes a gesture menu. The long-press timer is
//! 500ms and the contextmenu event that follows a touch or a key open is swallowed for 600ms.

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

/// A point given relative to an area's top-left corner, in viewport coordinates.
pub fn anchored_point(area: (f64, f64), point: (f64, f64)) -> (f64, f64) {
    (area.0 + point.0, area.1 + point.1)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Opener {
    Pointer,
    Keyboard,
    /// Opened at rest by `open_at`, with the point relative to the area. Takes no focus, and
    /// follows the area through scroll and resize until it is used.
    Rest((f64, f64)),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Opening {
    x: f64,
    y: f64,
    by: Opener,
    n: u32,
}

/// What the menu measures from the page, in one script so the crate's eval count does not grow
/// (`tooling/xtask/kit-eval-ratchet.txt`; AF5 keeps web-sys out of this crate).
struct PageLayout {
    viewport: (f64, f64),
    rtl: bool,
    /// The top-left of the area's first element, when an area was named and has one.
    origin: Option<(f64, f64)>,
}

/// The JS that [`page_layout`] runs. `area_id` names the area whose origin is read, if any.
pub fn page_layout_script(area_id: Option<&str>) -> String {
    let area = area_id.map_or_else(|| "null".to_string(), |id| format!("{id:?}"));
    format!(
        "const id = {area}; \
         const r = id === null ? null : document.getElementById(id)?.firstElementChild?.getBoundingClientRect(); \
         return [window.innerWidth, window.innerHeight, getComputedStyle(document.documentElement).direction === 'rtl' ? 1 : 0, r ? r.left : null, r ? r.top : null];"
    )
}

async fn page_layout(area_id: Option<&str>) -> Option<PageLayout> {
    let v = document::eval(&page_layout_script(area_id)).await.ok()?;
    Some(PageLayout {
        viewport: (v.get(0)?.as_f64()?, v.get(1)?.as_f64()?),
        rtl: v.get(2)?.as_f64() == Some(1.0),
        origin: v.get(3)?.as_f64().zip(v.get(4)?.as_f64()),
    })
}

/// The top-left of the area's first element. The `display: contents` wrapper has no box of its own.
async fn area_origin(area_id: String) -> Option<(f64, f64)> {
    page_layout(Some(&area_id)).await?.origin
}

/// Places a card of its own size with its top-left at `pointer`, kept inside the viewport.
async fn place_card(card: Option<MountedEvent>, pointer: (f64, f64)) -> Option<Placement> {
    let node = card?;
    let rect = node.get_client_rect().await.ok()?;
    let layout = page_layout(None).await?;
    let (w, h) = layout.viewport;
    let rtl = layout.rtl;
    let card_size = Size {
        width: rect.size.width,
        height: rect.size.height,
    };
    let at = Rect {
        left: pointer.0,
        top: pointer.1,
        width: 0.0,
        height: 0.0,
    };
    let align = if rtl { Align::End } else { Align::Start };
    Some(place(
        at,
        card_size,
        Size {
            width: w,
            height: h,
        },
        Side::Bottom,
        align,
        0.0,
        rtl,
    ))
}

/// Wraps `children` and opens `items` where the user right-clicks, long-presses or presses Shift+F10.
///
/// `open_at` opens the menu once, on mount, at that point relative to the top-left of the first element
/// of `children`. Until the user presses or focuses inside it, scroll and resize move it with the area
/// instead of closing it. Outside press, Escape, a pick and window blur close it in both modes.
#[component]
pub fn ContextMenu(
    children: Element,
    items: Vec<MenuItem>,
    #[props(default, into)] title: Option<String>,
    #[props(default = "Actions".to_string(), into)] aria_label: String,
    #[props(default)] disabled: bool,
    #[props(default)] open_at: Option<(f64, f64)>,
    #[props(default)] onopenchange: Option<EventHandler<bool>>,
    #[props(default, into)] width: Option<String>,
    #[props(default)] menu_class: String,
) -> Element {
    let uid = use_hook(next_id);
    let area_id = format!("ui-ctxc-{uid}-area");
    let has_items = !items.is_empty();
    let mut at = use_signal(|| None::<Opening>);
    let mut count = use_signal(|| 0u32);
    let mut press = use_signal(|| None::<(f64, f64, u32)>);
    let mut press_token = use_signal(|| 0u32);
    let mut suppress = use_signal(|| false);
    let mut area = use_signal(|| None::<MountedEvent>);

    let mut show_at = move |x: f64, y: f64, by: Opener| {
        if disabled || !has_items {
            return false;
        }
        let n = count() + 1;
        count.set(n);
        at.set(Some(Opening { x, y, by, n }));
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
            if show_at(x, y, Opener::Keyboard) {
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
        if show_at(point.x, point.y, Opener::Pointer) {
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
                if show_at(point.x, point.y, Opener::Pointer) {
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
    let mount_area = area_id.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{area_id}",
            style: "display: contents; -webkit-touch-callout: none;",
            onmounted: move |event: MountedEvent| {
                area.set(Some(event));
                if let Some(point) = open_at {
                    let mount_area = mount_area.clone();
                    spawn(async move {
                        if let Some(origin) = area_origin(mount_area).await {
                            let (x, y) = anchored_point(origin, point);
                            show_at(x, y, Opener::Rest(point));
                        }
                    });
                }
            },
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
                id: format!("ui-ctxc-{uid}-menu-{}", opened.n),
                area_id: area_id.clone(),
                x: opened.x,
                y: opened.y,
                by: opened.by,
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
    area_id: String,
    x: f64,
    y: f64,
    by: Opener,
    items: Vec<MenuItem>,
    title: Option<String>,
    label: String,
    width: Option<String>,
    menu_class: String,
    on_close: EventHandler<FocusBack>,
) -> Element {
    let mut placed = use_signal(|| None::<Placement>);
    let mut card = use_signal(|| None::<MountedEvent>);
    let title_id = format!("{id}-title");
    let list_id = format!("{id}-list");
    let style = placed_style(placed(), width.as_deref());
    let panel_id = id.clone();
    let resting = matches!(by, Opener::Rest(_));

    let setup = format!(
        r#"const panel = () => document.getElementById({panel_id:?});
let resting = {resting};
on(document, "pointerdown", (e) => {{ if (panel()?.contains(e.target)) resting = false; else send("close"); }}, true);
on(document, "focusin", (e) => {{ if (panel()?.contains(e.target)) resting = false; }});
// Focusing the panel on open can scroll the page; that scroll must not close it.
const opened = performance.now();
on(window, "scroll", (e) => {{
  if (panel()?.contains(e.target)) return;
  if (resting) send("reflow");
  else if (performance.now() - opened > 250) send("close");
}}, true);
on(window, "resize", () => send(resting ? "reflow" : "close"));
on(window, "blur", () => send("close"));
on(window, "keydown", (e) => {{ if (resting && e.key === "Escape") send("close"); }}, true);"#
    );
    use_effect(use_reactive((&panel_id,), {
        let panel_id = panel_id.clone();
        let area_id = area_id.clone();
        move |(_,)| {
            let area_id = area_id.clone();
            listen(listen_js(&panel_id, &setup), move |message| {
                match message.as_str() {
                    "close" => on_close.call(FocusBack::Keep),
                    "reflow" => {
                        if let Opener::Rest(point) = by {
                            let area_id = area_id.clone();
                            spawn(async move {
                                let Some(origin) = area_origin(area_id).await else {
                                    return;
                                };
                                if let Some(next) =
                                    place_card(card(), anchored_point(origin, point)).await
                                {
                                    placed.set(Some(next));
                                }
                            });
                        }
                    }
                    _ => {}
                }
            });
        }
    }));

    let focus_script = match by {
        Opener::Keyboard => format!(
            "document.getElementById({list_id:?})?.querySelector(':scope > [role^=\"menuitem\"]:not([aria-disabled=\"true\"])')?.focus({{ preventScroll: true }});"
        ),
        Opener::Pointer => {
            format!("document.getElementById({list_id:?})?.focus({{ preventScroll: true }});")
        }
        Opener::Rest(_) => String::new(),
    };
    let shown_side = placed().map_or(Side::Bottom, |p| p.side);
    let shown_align = placed().map_or(Align::Start, |p| p.align);
    let labelled_by = title.as_ref().map(|_| title_id.clone());
    let menu_label = if title.is_some() { None } else { Some(label) };

    rsx! {
        div {
            id: "{panel_id}",
            class: class_list("ui-ctxc ui-ctxc__panel", &menu_class),
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
                card.set(Some(event));
                spawn(async move {
                    if let Some(next) = place_card(card(), (x, y)).await {
                        placed.set(Some(next));
                    }
                });
                let show = format!(
                    "const p = document.getElementById({panel_id:?}); if (p) {{ p.style.visibility = 'visible'; p.showPopover?.(); }}"
                );
                document::eval(&format!("{show} {focus_script}"));
            },
            if let Some(title) = title {
                span { class: "ui-ctxc__title", id: "{title_id}", "{title}" }
            }
            MenuLevel {
                prefix: "ui-ctxc",
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
