//! Hover Card: a preview that opens when the pointer rests on its trigger or the keyboard focuses it.
//!
//! The pointer may travel from the trigger to the card through the convex hull of the exit point and the
//! card's corners (`corridor`); a pointer that leaves the hull starts the close delay.
//!
//! Port losses: `children` is wrapped in a `display: contents` span, which takes the pointer and focus
//! events. Focus leaving the trigger into the card is tracked by focus events, not by `relatedTarget`.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};
use crate::popover::{
    Align, Placement, Rect, Side, listen, listen_js, placed_style, refresh, release_js,
};

const CSS: Asset = asset!("/assets/hover-card.css");

/// Slack in px for a shaky hand at the corridor's edge.
const SLACK: f64 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Convex hull of `points` (monotone chain), counter-clockwise.
pub fn hull(points: &[Point]) -> Vec<Point> {
    let mut p = points.to_vec();
    p.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    let cross =
        |o: Point, a: Point, b: Point| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut lower: Vec<Point> = Vec::new();
    for &q in &p {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], q) <= 0.0 {
            lower.pop();
        }
        lower.push(q);
    }
    let mut upper: Vec<Point> = Vec::new();
    for &q in p.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], q) <= 0.0 {
            upper.pop();
        }
        upper.push(q);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Whether (x, y) is inside the counter-clockwise polygon `poly`, within `SLACK` px.
pub fn inside_hull(poly: &[Point], x: f64, y: f64) -> bool {
    if poly.len() < 3 {
        return false;
    }
    poly.iter().enumerate().all(|(i, a)| {
        let b = poly[(i + 1) % poly.len()];
        let len = (b.x - a.x).hypot(b.y - a.y).max(1.0);
        ((b.x - a.x) * (y - a.y) - (b.y - a.y) * (x - a.x)) / len >= -SLACK
    })
}

/// The corridor from where the pointer left the trigger to the card.
pub fn corridor(exit: Point, card: Rect) -> Vec<Point> {
    let (l, t, r, b) = (card.left, card.top, card.right(), card.bottom());
    hull(&[
        exit,
        Point { x: l, y: t },
        Point { x: r, y: t },
        Point { x: r, y: b },
        Point { x: l, y: b },
    ])
}

fn card_dom_id(uid: usize) -> String {
    format!("ui-hvc-{uid}")
}

fn corridor_js(uid: usize, on: bool) -> String {
    let key = card_dom_id(uid);
    if on {
        format!("(window.__affectCorridor ||= {{}})[{key:?}] = true;")
    } else {
        format!("delete window.__affectCorridor?.[{key:?}];")
    }
}

#[component]
pub fn HoverCard(
    /// What the card shows. Without content it never opens.
    #[props(default)]
    content: Option<Element>,
    #[props(default)] side: Side,
    #[props(default)] align: Align,
    #[props(default = 8.0)] offset: f64,
    #[props(default = 600)] open_delay: u32,
    #[props(default = 300)] close_delay: u32,
    /// A CSS width, e.g. "300px". By default the card fits its content, up to 320px.
    #[props(default, into)]
    width: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] open: Option<bool>,
    #[props(default)] onopenchange: Option<EventHandler<bool>>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let uid = use_hook(next_id);
    let card_id = card_dom_id(uid);
    let anchor_id = format!("ui-hvc-anchor-{uid}");
    let anchor = format!("document.getElementById({anchor_id:?})?.firstElementChild");
    let empty = content.is_none();
    let mut shown = use_signal(|| false);
    let mut blocked = use_signal(|| false);
    let mut pointer_focus = use_signal(|| false);
    let mut generation = use_signal(|| 0u32);
    let mut card = use_signal(|| None::<MountedEvent>);
    let mut placed = use_signal(|| None::<Placement>);
    let mut corridor_on = use_signal(|| None::<Vec<Point>>);
    let open_now = !disabled && !empty && open.unwrap_or(shown());

    let mut change = move |next: bool| {
        generation.set(generation() + 1);
        corridor_on.set(None);
        document::eval(&corridor_js(uid, false));
        if open.is_none() {
            shown.set(next);
        }
        if next != open_now
            && let Some(handler) = onopenchange
        {
            handler.call(next);
        }
    };

    let mut schedule = move |ms: u32, next: bool| {
        let token = generation() + 1;
        generation.set(token);
        spawn(async move {
            sleep(ms).await;
            if generation() == token {
                change(next);
            }
        });
    };

    let mut show = move || {
        if disabled || empty || blocked() || shown() {
            return;
        }
        schedule(open_delay, true);
    };

    let mut hide = move || {
        if shown() {
            schedule(close_delay, false);
        }
    };

    let mut stay = move || {
        generation.set(generation() + 1);
        corridor_on.set(None);
        document::eval(&corridor_js(uid, false));
    };

    let setup = format!(
        r#"on(window, "keydown", (e) => {{
  if (e.key !== "Escape" || e.defaultPrevented) return;
  e.preventDefault();
  e.stopPropagation();
  const back = !!document.getElementById({card_id:?})?.contains(document.activeElement);
  send("escape");
  if (back) document.getElementById({anchor_id:?})?.firstElementChild?.focus({{ preventScroll: true }});
}}, true);
on(document, "pointerdown", (e) => {{
  const panel = document.getElementById({card_id:?});
  const anchor = document.getElementById({anchor_id:?});
  if (!panel?.contains(e.target) && !anchor?.contains(e.target)) send("close");
}}, true);
on(document, "pointermove", (e) => {{
  if (e.pointerType !== "touch" && window.__affectCorridor?.[{card_id:?}]) send("pointer:" + e.clientX + "," + e.clientY);
}});
on(window, "resize", () => send("move"));
on(window, "scroll", () => send("move"), true);"#
    );

    use_effect(use_reactive((&open_now,), {
        let card_id = card_id.clone();
        let anchor = anchor.clone();
        move |(open_now,)| {
            if !open_now {
                placed.set(None);
                document::eval(&release_js(&card_id));
                return;
            }
            let anchor = anchor.clone();
            listen(listen_js(&card_id, &setup), move |message| {
                if let Some(rest) = message.strip_prefix("pointer:") {
                    let mut parts = rest.split(',').filter_map(|v| v.parse::<f64>().ok());
                    if let (Some(x), Some(y)) = (parts.next(), parts.next())
                        && corridor_on().is_some_and(|poly| !inside_hull(&poly, x, y))
                    {
                        corridor_on.set(None);
                        document::eval(&corridor_js(uid, false));
                        hide();
                    }
                    return;
                }
                match message.as_str() {
                    "move" => refresh(card, placed, anchor.clone(), side, align, offset),
                    "escape" => {
                        blocked.set(true);
                        change(false);
                    }
                    "close" => change(false),
                    _ => {}
                }
            });
        }
    }));

    let shown_side = placed().map_or(side, |p| p.side);
    let style = placed_style(placed(), width.as_deref());
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        span {
            id: "{anchor_id}",
            style: "display: contents",
            onpointerenter: move |event| {
                if event.data().pointer_type() == "touch" {
                    return;
                }
                if shown() {
                    stay();
                } else {
                    show();
                }
            },
            onpointerleave: move |event| {
                blocked.set(false);
                if event.data().pointer_type() == "touch" {
                    return;
                }
                if !shown() {
                    generation.set(generation() + 1);
                    return;
                }
                let exit = event.data().client_coordinates();
                let node = card();
                spawn(async move {
                    let rect = match node {
                        Some(node) => node.get_client_rect().await.ok(),
                        None => None,
                    };
                    match rect {
                        Some(rect) => {
                            let r = Rect { left: rect.origin.x, top: rect.origin.y, width: rect.size.width, height: rect.size.height };
                            let poly = corridor(Point { x: exit.x, y: exit.y }, r);
                            generation.set(generation() + 1);
                            corridor_on.set(Some(poly));
                            document::eval(&corridor_js(uid, true));
                        }
                        None => hide(),
                    }
                });
            },
            onpointerdown: move |_| {
                pointer_focus.set(true);
                blocked.set(true);
                change(false);
            },
            onfocusin: move |_| {
                if !pointer_focus() {
                    show();
                }
                pointer_focus.set(false);
            },
            onfocusout: move |_| {
                blocked.set(false);
                hide();
            },
            {children}
            if open_now {
                div {
                    id: "{card_id}",
                    class: class_list("ui-hvc", &class),
                    popover: "manual",
                    "data-side": shown_side.as_str(),
                    style: "{style}",
                    onmounted: move |event: MountedEvent| {
                        card.set(Some(event.clone()));
                        refresh(card, placed, anchor.clone(), side, align, offset);
                        document::eval(&format!("document.getElementById({card_id:?})?.showPopover?.();"));
                    },
                    onpointerenter: move |event| {
                        if event.data().pointer_type() != "touch" {
                            stay();
                        }
                    },
                    onpointerleave: move |_| {
                        corridor_on.set(None);
                        document::eval(&corridor_js(uid, false));
                        hide();
                    },
                    onfocusin: move |_| stay(),
                    onfocusout: move |_| hide(),
                    {content}
                }
            }
        }
    }
}
