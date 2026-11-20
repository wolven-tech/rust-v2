//! Popover: a small card beside its trigger that flips to the other side when there is no room.
//!
//! Shared by Tooltip, Hover Card, Select and the menus: `place`, `flip_side`, `listen`.
//!
//! Port losses: the React version reads the anchor through `getBoundingClientRect` on every
//! `ResizeObserver` tick; here the card is repositioned on open, on window resize and on scroll.
//! The card is in the top layer only where the browser supports `showPopover`.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/popover.css");

/// Pixels kept between a card and the window edge.
pub(crate) const PAD: f64 = 8.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Bottom,
    Top,
    Right,
    Left,
}

impl Side {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bottom => "bottom",
            Self::Top => "top",
            Self::Right => "right",
            Self::Left => "left",
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Bottom => Self::Top,
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Left => Self::Right,
        }
    }

    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

impl Align {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn right(&self) -> f64 {
        self.left + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.top + self.height
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// What is left on each side of the trigger for the card, after the gap and `PAD`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Room {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

impl Room {
    pub fn of(&self, side: Side) -> f64 {
        match side {
            Side::Top => self.top,
            Side::Bottom => self.bottom,
            Side::Left => self.left,
            Side::Right => self.right,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub side: Side,
    pub align: Align,
    pub top: f64,
    pub left: f64,
    pub max_height: Option<f64>,
}

/// The room around `trigger` inside a window of `view`.
pub fn room(trigger: Rect, view: Size, gap: f64) -> Room {
    Room {
        top: trigger.top - gap - PAD,
        bottom: view.height - trigger.bottom() - gap - PAD,
        left: trigger.left - gap - PAD,
        right: view.width - trigger.right() - gap - PAD,
    }
}

/// The side a card opens on: the preferred one, unless it has less room than the card needs
/// and the opposite side has more.
pub fn flip_side(preferred: Side, room: Room, card: Size) -> Side {
    let need = if preferred.is_vertical() {
        card.height
    } else {
        card.width
    };
    if room.of(preferred) < need && room.of(preferred.opposite()) > room.of(preferred) {
        preferred.opposite()
    } else {
        preferred
    }
}

/// Puts a card of `card` size next to `trigger`. On a phone with no room beside it, the card
/// goes under or over the trigger instead. The edge it lines up with switches when it would
/// stick out; the card is pushed inside the window as a last resort. Too tall for its room:
/// `max_height` is set and the card scrolls. `rtl` swaps start and end above and below.
pub fn place(
    trigger: Rect,
    card: Size,
    view: Size,
    side: Side,
    align: Align,
    gap: f64,
    rtl: bool,
) -> Placement {
    let room = room(trigger, view, gap);
    let mut side = flip_side(side, room, card);
    if !side.is_vertical() && room.of(side) < card.width {
        side = if room.bottom >= room.top {
            Side::Bottom
        } else {
            Side::Top
        };
    }
    let mut align = if side.is_vertical() && rtl {
        match align {
            Align::Start => Align::End,
            Align::End => Align::Start,
            Align::Center => Align::Center,
        }
    } else {
        align
    };
    let (top, left, max_height);
    if side.is_vertical() {
        let fit = card.height.min(room.of(side).max(120.0));
        top = if side == Side::Bottom {
            trigger.bottom() + gap
        } else {
            trigger.top - gap - fit
        };
        let mut x = match align {
            Align::Start => trigger.left,
            Align::End => trigger.right() - card.width,
            Align::Center => trigger.left + trigger.width / 2.0 - card.width / 2.0,
        };
        if align == Align::Start
            && x + card.width > view.width - PAD
            && trigger.right() - card.width >= PAD
        {
            align = Align::End;
            x = trigger.right() - card.width;
        } else if align == Align::End && x < PAD && trigger.left + card.width <= view.width - PAD {
            align = Align::Start;
            x = trigger.left;
        }
        left = x.max(PAD).min(view.width - PAD - card.width);
        max_height = (card.height > fit).then_some(fit);
    } else {
        left = if side == Side::Right {
            trigger.right() + gap
        } else {
            trigger.left - gap - card.width
        };
        let mut y = match align {
            Align::Start => trigger.top,
            Align::End => trigger.bottom() - card.height,
            Align::Center => trigger.top + trigger.height / 2.0 - card.height / 2.0,
        };
        if align == Align::Start
            && y + card.height > view.height - PAD
            && trigger.bottom() - card.height >= PAD
        {
            align = Align::End;
            y = trigger.bottom() - card.height;
        } else if align == Align::End && y < PAD && trigger.top + card.height <= view.height - PAD {
            align = Align::Start;
            y = trigger.top;
        }
        top = y.max(PAD).min((view.height - PAD - card.height).max(PAD));
        max_height = (card.height > view.height - 2.0 * PAD).then_some(view.height - 2.0 * PAD);
    }
    Placement {
        side,
        align,
        top,
        left,
        max_height,
    }
}

/// Runs `script` in the page and calls `on_message` with every string it sends.
pub(crate) fn listen(script: String, mut on_message: impl FnMut(String) + 'static) {
    spawn(async move {
        let mut events = document::eval(&script);
        while let Ok(message) = events.recv::<String>().await {
            on_message(message);
        }
    });
}

/// Listener setup for `owner`. Installing again replaces the previous set; `release_js` removes it.
/// Scripts call `on(target, type, fn, options)` to register and `send(message)` to report.
pub(crate) fn listen_js(owner: &str, setup: &str) -> String {
    format!(
        r#"const owners = (window.__affectOwners ||= {{}});
owners[{owner:?}]?.remove();
const handlers = [];
const on = (target, type, fn, options) => {{ target.addEventListener(type, fn, options); handlers.push(() => target.removeEventListener(type, fn, options)); }};
const send = (message) => {{ try {{ dioxus.send(message); }} catch (_) {{}} }};
owners[{owner:?}] = {{ remove: () => handlers.forEach((undo) => undo()) }};
{setup}"#
    )
}

pub(crate) fn release_js(owner: &str) -> String {
    format!("window.__affectOwners?.[{owner:?}]?.remove();")
}

/// The JS expression for the anchor: the element with `anchor_id`, or the card's parent.
pub(crate) fn anchor_expression(panel_id: &str, anchor_id: Option<&str>) -> String {
    match anchor_id {
        Some(id) => format!("document.getElementById({id:?})"),
        None => format!("document.getElementById({panel_id:?})?.parentElement"),
    }
}

/// Reads the anchor and the viewport through eval, and the card's size from its mounted node.
pub(crate) async fn position(
    card: Option<MountedEvent>,
    anchor: &str,
    side: Side,
    align: Align,
    gap: f64,
) -> Option<Placement> {
    let node = card?;
    let rect = node.get_client_rect().await.ok()?;
    let card_size = Size {
        width: rect.size.width,
        height: rect.size.height,
    };
    let metrics = document::eval(&format!(
        "const a = {anchor}; if (!a) return null; const r = a.getBoundingClientRect(); \
         return [r.left, r.top, r.width, r.height, window.innerWidth, window.innerHeight, getComputedStyle(a).direction === 'rtl' ? 1 : 0];"
    ))
    .await
    .ok()?;
    let n = |i: usize| metrics.get(i).and_then(|v| v.as_f64());
    let trigger = Rect {
        left: n(0)?,
        top: n(1)?,
        width: n(2)?,
        height: n(3)?,
    };
    let view = Size {
        width: n(4)?,
        height: n(5)?,
    };
    Some(place(
        trigger,
        card_size,
        view,
        side,
        align,
        gap,
        n(6)? == 1.0,
    ))
}

/// Recomputes the placement into `placed`. Failure (not mounted, not in the page) keeps the last one.
pub(crate) fn refresh(
    card: Signal<Option<MountedEvent>>,
    mut placed: Signal<Option<Placement>>,
    anchor: String,
    side: Side,
    align: Align,
    gap: f64,
) {
    let node = card();
    spawn(async move {
        if let Some(next) = position(node, &anchor, side, align, gap).await {
            placed.set(Some(next));
        }
    });
}

/// The inline style that puts a card where `placed` says, hidden until it has been placed.
pub fn placed_style(placed: Option<Placement>, width: Option<&str>) -> String {
    let mut style = match placed {
        Some(p) => format!(
            "visibility:visible;top:{}px;left:{}px;max-height:{};",
            p.top.round(),
            p.left.round(),
            p.max_height
                .map_or("none".to_string(), |h| format!("{}px", h.round())),
        ),
        None => "visibility:hidden;".to_string(),
    };
    if let Some(width) = width {
        style.push_str(&format!("width:{width};max-width:calc(100vw - 16px);"));
    }
    style
}

#[component]
pub fn Popover(
    open: bool,
    /// Called on a press outside, Escape, or when the focus leaves.
    onclose: EventHandler<()>,
    #[props(default)] side: Side,
    #[props(default)] align: Align,
    #[props(default = 6.0)] offset: f64,
    /// The element it is attached to; by default its parent element.
    #[props(default, into)]
    anchor_id: Option<String>,
    #[props(default)] title: Option<Element>,
    #[props(default)] description: Option<Element>,
    /// A CSS width, e.g. "280px". By default the card fits its content.
    #[props(default, into)]
    width: Option<String>,
    #[props(default = true)] focus_on_open: bool,
    #[props(default)] unstyled: bool,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let uid = use_hook(next_id);
    let panel_id = format!("ui-pop-{uid}");
    let title_id = format!("{panel_id}-title");
    let desc_id = format!("{panel_id}-desc");
    let anchor = anchor_expression(&panel_id, anchor_id.as_deref());
    let mut card = use_signal(|| None::<MountedEvent>);
    let mut placed = use_signal(|| None::<Placement>);
    // Only an opening after a close moves the focus; a card open from the start leaves it alone.
    let mut armed = use_signal(|| !open);

    let setup = {
        let panel = format!("document.getElementById({panel_id:?})");
        let trigger_of = format!(
            "(() => {{ const a = {anchor}; if (!a) return null; if (a.matches('button, [href], input, [tabindex]')) return a; \
             return [...a.querySelectorAll('button, [href], input, [tabindex]')].find((el) => !{panel}?.contains(el)) ?? null; }})()"
        );
        format!(
            r#"const panel = () => {panel};
const anchor = () => {anchor};
const inside = (node) => !!node && (!!panel()?.contains(node) || !!anchor()?.contains(node));
on(document, "pointerdown", (e) => {{ if (!inside(e.target)) send("close"); }}, true);
on(window, "keydown", (e) => {{
  if (e.key !== "Escape" || e.defaultPrevented) return;
  const layer = e.target?.closest?.("[popover]");
  if (layer && layer !== panel() && panel()?.contains(layer)) return;
  e.preventDefault();
  e.stopPropagation();
  const back = inside(document.activeElement);
  send("close");
  if (back) {trigger_of}?.focus({{ preventScroll: true }});
}}, true);
on(document, "focusout", (e) => {{ if (inside(e.target) && e.relatedTarget && !inside(e.relatedTarget)) send("close"); }});
on(window, "resize", () => send("move"));
on(window, "scroll", () => send("move"), true);"#
        )
    };

    use_effect(use_reactive((&open,), {
        let panel_id = panel_id.clone();
        let anchor = anchor.clone();
        move |(open,)| {
            if !open {
                armed.set(true);
                placed.set(None);
                document::eval(&release_js(&panel_id));
                return;
            }
            let anchor = anchor.clone();
            listen(listen_js(&panel_id, &setup), move |message| {
                match message.as_str() {
                    "move" => refresh(card, placed, anchor.clone(), side, align, offset),
                    "close" => onclose.call(()),
                    _ => {}
                }
            });
        }
    }));

    if !open {
        return rsx! {};
    }
    let style = placed_style(placed(), width.as_deref());
    let (shown_side, shown_align) = placed().map_or((side, align), |p| (p.side, p.align));
    let class = class_list("ui-pop", &class);
    let head = title.is_some() || description.is_some();
    let describedby = description.is_some().then(|| desc_id.clone());
    let labelledby = title.is_some().then(|| title_id.clone());
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{panel_id}",
            class,
            "data-unstyled": unstyled.then_some("true"),
            role: "dialog",
            aria_labelledby: labelledby,
            aria_describedby: describedby,
            tabindex: "-1",
            "data-side": shown_side.as_str(),
            "data-align": shown_align.as_str(),
            style: "{style}",
            onmounted: {
                let anchor = anchor.clone();
                let panel_id = panel_id.clone();
                move |event: MountedEvent| {
                    card.set(Some(event.clone()));
                    refresh(card, placed, anchor.clone(), side, align, offset);
                    if focus_on_open && armed() {
                        let script = format!(
                            "const el = document.getElementById({panel_id:?}); if (el) el.style.visibility = 'visible'; (el?.querySelector('input, textarea, select, button, [href], [tabindex]:not([tabindex=\"-1\"])') ?? el)?.focus({{ preventScroll: true }});"
                        );
                        document::eval(&script);
                    }
                }
            },
            ..attributes,
            if head {
                div { class: "ui-pop__head",
                    if let Some(title) = title {
                        p { class: "ui-pop__title", id: "{title_id}", {title} }
                    }
                    if let Some(description) = description {
                        p { class: "ui-pop__desc", id: "{desc_id}", {description} }
                    }
                }
            }
            {children}
        }
    }
}
