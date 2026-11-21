//! Breadcrumb: the trail from the top of a hierarchy to the current page. A long trail folds its middle into a …
//! button whose menu lists the hidden crumbs, past `max_items` and again when the row is too narrow for them.
//!
//! Port losses: `icon` takes a rendered `Element`, not a Hugeicons icon object. `ref` is not forwarded. The menu is
//! placed with a `document::eval` measurement after it opens, so it can paint once at the wrong spot before it moves.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/breadcrumb.css");

/// Reports the row's width when it changes. The fold count starts over from `max_items` on each report.
const WIDTH: &str = r#"
const list = document.getElementById("__LIST__");
if (list && typeof ResizeObserver !== "undefined") {
  let width = list.clientWidth;
  new ResizeObserver(() => {
    if (Math.abs(list.clientWidth - width) < 1) return;
    width = list.clientWidth;
    dioxus.send(width);
  }).observe(list);
  await new Promise(() => {});
}
"#;

/// True while the crumbs at their full names (capped at 200px each) do not fit the row.
const NEED: &str = r#"
const list = document.getElementById("__LIST__");
if (!list) return false;
let sum = 0;
for (const li of Array.from(list.children)) {
  const label = li.querySelector("[data-label]");
  const box = li.getBoundingClientRect().width;
  sum += label ? box - label.getBoundingClientRect().width + Math.min(label.scrollWidth, 200) : box;
}
return sum > list.clientWidth + 1;
"#;

/// Sets the full name as a tooltip, only while the label is cut short.
const FULL_TITLE: &str = r#"
const el = document.getElementById("__ID__");
const text = el && el.querySelector("[data-label]");
if (el && text) {
  if (text.scrollWidth > text.clientWidth + 1) el.title = __LABEL__;
  else el.removeAttribute("title");
}
"#;

/// Places the open menu under its button (or above it when there is no room), focuses the first row, and reports
/// one outside press so the component can close it. The listeners stop on their own once the menu is gone.
const OPEN_MENU: &str = r#"
const el = document.getElementById("__PANEL__");
const b = document.getElementById("__BUTTON__");
if (el && b) {
  if (!el.matches(":popover-open")) el.showPopover();
  const place = () => {
    const r = b.getBoundingClientRect();
    const pad = 8;
    const gap = 6;
    const vw = document.documentElement.clientWidth || window.innerWidth;
    const vh = window.innerHeight;
    el.style.maxHeight = "";
    const w = el.offsetWidth;
    const h = el.offsetHeight;
    const below = vh - r.bottom - gap - pad;
    const above = r.top - gap - pad;
    const down = below >= h || below >= above;
    const fit = Math.max(120, Math.min(h, down ? below : above));
    const rtl = getComputedStyle(el).direction === "rtl";
    let left = rtl ? r.right - w : r.left - 4;
    left = Math.min(Math.max(pad, left), vw - pad - w);
    el.style.top = Math.round(down ? r.bottom + gap : r.top - gap - fit) + "px";
    el.style.left = Math.round(left) + "px";
    if (h > fit) el.style.maxHeight = fit + "px";
    el.dataset.side = down ? "bottom" : "top";
  };
  place();
  el.querySelector('[role="menuitem"]')?.focus({ preventScroll: true });
  const reposition = () => {
    if (!el.isConnected) {
      window.removeEventListener("resize", reposition);
      window.removeEventListener("scroll", reposition, true);
      return;
    }
    place();
  };
  window.addEventListener("resize", reposition);
  window.addEventListener("scroll", reposition, true);
  const dismiss = (event) => {
    if (!el.isConnected) {
      document.removeEventListener("pointerdown", dismiss, true);
      return;
    }
    if (!el.contains(event.target) && !b.contains(event.target)) {
      document.removeEventListener("pointerdown", dismiss, true);
      dioxus.send(true);
    }
  };
  document.addEventListener("pointerdown", dismiss, true);
}
"#;

/// One crumb: a link when it has an `href`, a button when it has an `onclick`, and plain text when it has neither.
#[derive(Clone, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    /// Makes the crumb a link.
    pub href: Option<String>,
    /// Makes the crumb a button when there is no href, or runs before a link navigates.
    pub onclick: Option<EventHandler<MouseEvent>>,
    /// A 14px glyph before the label.
    pub icon: Option<Element>,
}

/// What sits between the crumbs.
#[derive(Clone, PartialEq, Default)]
pub enum Separator {
    Slash,
    #[default]
    Chevron,
    Custom(Element),
}

/// The crumbs that stay folded into the … menu: everything past `max_items` (the first and the last always stay),
/// plus `extra` more that the width asked for. Never more than `total - 2`.
pub fn folded_count(total: usize, max_items: usize, extra: usize) -> usize {
    let by_count = total.saturating_sub(max_items.max(2));
    by_count.saturating_add(extra).min(total.saturating_sub(2))
}

/// The label of the … button, which names how many crumbs it hides.
pub fn more_label(hidden: usize) -> String {
    if hidden == 1 {
        "Show 1 more crumb".to_string()
    } else {
        format!("Show {hidden} more crumbs")
    }
}

#[derive(Clone, PartialEq)]
enum Part {
    Sep(String, Separator),
    More(Vec<BreadcrumbItem>),
    Crumb(usize, BreadcrumbItem, bool),
}

/// A trail of crumbs, the last one being the current page. Long names shorten with … and show in full on hover.
#[component]
pub fn Breadcrumb(
    items: Vec<BreadcrumbItem>,
    #[props(default)] separator: Separator,
    #[props(default = 4)] max_items: usize,
    #[props(default = "Breadcrumb".to_string(), into)] aria_label: String,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = nav)] attributes: Vec<Attribute>,
) -> Element {
    let id = use_hook(|| format!("ui-bcr-{}", next_id()));
    let list_id = format!("{id}-list");
    let n = items.len();
    let sig = format!(
        "{}|{max_items}",
        items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>()
            .join("\u{0}")
    );

    // The width-driven fold count, valid only for the crumbs it was measured for.
    let mut folds = use_signal(|| (String::new(), 0_usize));
    let extra = match folds() {
        (ref s, e) if *s == sig => e,
        _ => 0,
    };
    let hidden = folded_count(n, max_items, extra);
    let max_hidden = n.saturating_sub(2);

    let fit_id = list_id.clone();
    let fit_sig = sig.clone();
    use_effect(use_reactive((&hidden, &fit_sig), move |(hidden, sig)| {
        if hidden >= max_hidden {
            return;
        }
        let script = NEED.replace("__LIST__", &fit_id);
        spawn(async move {
            let overflowing = document::eval(&script)
                .await
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let current = match folds.peek().clone() {
                (ref s, e) if *s == sig => e,
                _ => 0,
            };
            if overflowing && folded_count(n, max_items, current) < max_hidden {
                folds.set((sig, current + 1));
            }
        });
    }));

    let reset_id = list_id.clone();
    use_effect(move || {
        let mut channel = document::eval(&WIDTH.replace("__LIST__", &reset_id));
        spawn(async move {
            while channel.recv::<f64>().await.is_ok() {
                folds.write().1 = 0;
            }
        });
    });

    if n == 0 {
        return rsx! {};
    }

    let mut parts = Vec::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 && i <= hidden {
            if i == 1 {
                parts.push(Part::Sep("s-more".to_string(), separator.clone()));
                parts.push(Part::More(items[1..=hidden].to_vec()));
            }
            continue;
        }
        if i > 0 {
            parts.push(Part::Sep(format!("s-{i}"), separator.clone()));
        }
        parts.push(Part::Crumb(i, item.clone(), i == n - 1));
    }

    let tight = hidden >= max_hidden && n > 2;
    let class = class_list("ui-bcr", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        nav {
            id: "{id}",
            class,
            aria_label: "{aria_label}",
            "data-tight": tight.then_some(""),
            ..attributes,
            ol { id: "{list_id}", class: "ui-bcr__list",
                for part in parts {
                    {part_view(part, &id)}
                }
            }
        }
    }
}

fn part_view(part: Part, id: &str) -> Element {
    match part {
        Part::Sep(key, sep) => rsx! {
            li { key: "{key}", class: "ui-bcr__sep", role: "presentation", "aria-hidden": "true",
                {separator_view(sep)}
            }
        },
        Part::More(folded) => {
            let key = format!("more-{}", folded.len());
            rsx! {
                li { key: "{key}", class: "ui-bcr__item",
                    More { items: folded }
                }
            }
        }
        Part::Crumb(i, item, current) => {
            let key = format!("{i}-{}", item.label);
            let dom = format!("{id}-c{i}");
            rsx! {
                li { key: "{key}", class: "ui-bcr__item", "data-current": current.then_some(""),
                    {crumb(dom, item, current)}
                }
            }
        }
    }
}

fn separator_view(sep: Separator) -> Element {
    match sep {
        Separator::Slash => rsx! { "/" },
        Separator::Chevron => rsx! {
            svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "focusable": "false",
                path { d: "M9 6s6 4.42 6 6-6 6-6 6" }
            }
        },
        Separator::Custom(node) => rsx! { {node} },
    }
}

fn crumb(dom: String, item: BreadcrumbItem, current: bool) -> Element {
    let script = FULL_TITLE
        .replace("__ID__", &dom)
        .replace("__LABEL__", &format!("{:?}", item.label));
    let on_enter = move |_| {
        document::eval(&script);
    };
    let inner = crumb_inner(item.icon.clone(), item.label.clone());
    let handler = item.onclick;

    if current {
        return rsx! {
            span { id: "{dom}", class: "ui-bcr__current", aria_current: "page", onpointerenter: on_enter,
                {inner}
            }
        };
    }
    if let Some(href) = item.href {
        return rsx! {
            a { id: "{dom}", class: "ui-bcr__link", href: "{href}", onpointerenter: on_enter,
                onclick: move |event: MouseEvent| {
                    if let Some(handler) = handler {
                        handler.call(event);
                    }
                },
                {inner}
            }
        };
    }
    if handler.is_some() {
        return rsx! {
            button { id: "{dom}", "type": "button", class: "ui-bcr__link", onpointerenter: on_enter,
                onclick: move |event: MouseEvent| {
                    if let Some(handler) = handler {
                        handler.call(event);
                    }
                },
                {inner}
            }
        };
    }
    rsx! {
        span { id: "{dom}", class: "ui-bcr__link", "data-static": "", onpointerenter: on_enter,
            {inner}
        }
    }
}

fn crumb_inner(icon: Option<Element>, label: String) -> Element {
    rsx! {
        if let Some(icon) = icon {
            span { class: "ui-bcr__icon", "aria-hidden": "true", {icon} }
        }
        span { class: "ui-bcr__label", "data-label": "", dir: "auto", "{label}" }
    }
}

/// The … button and its menu of the folded crumbs.
#[component]
fn More(items: Vec<BreadcrumbItem>) -> Element {
    let uid = use_hook(|| format!("ui-bcr-more-{}", next_id()));
    let menu_id = format!("{uid}-menu");
    let mut open = use_signal(|| false);
    let active = use_signal(|| None::<usize>);

    let open_menu = (menu_id.clone(), uid.clone());
    use_effect(use_reactive((&open(),), move |(is_open,)| {
        if !is_open {
            return;
        }
        let script = OPEN_MENU
            .replace("__PANEL__", &open_menu.0)
            .replace("__BUTTON__", &open_menu.1);
        let mut channel = document::eval(&script);
        spawn(async move {
            if channel.recv::<bool>().await.is_ok() {
                open.set(false);
            }
        });
    }));

    let len = items.len();
    let label = more_label(len);
    let titles = items
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>()
        .join(" / ");
    let key_menu = menu_id.clone();
    let key_btn = uid.clone();

    rsx! {
        button {
            id: "{uid}",
            "type": "button",
            class: "ui-bcr__more",
            aria_label: "{label}",
            "aria-haspopup": "menu",
            aria_expanded: if open() { "true" } else { "false" },
            aria_controls: open().then(|| menu_id.clone()),
            title: "{titles}",
            onclick: move |_| open.set(!open()),
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::ArrowDown && !open() {
                    event.prevent_default();
                    open.set(true);
                }
            },
            {more_icon()}
        }
        if open() {
            div {
                id: "{menu_id}",
                class: "ui-bcr__menu",
                role: "menu",
                aria_label: "More crumbs",
                "popover": "manual",
                onkeydown: move |event: KeyboardEvent| {
                    let current = active().map(|i| i as isize).unwrap_or(-1);
                    let to = match event.key() {
                        Key::ArrowDown => wrap(current + 1, len),
                        Key::ArrowUp => wrap(current - 1, len),
                        Key::Home => 0,
                        Key::End => len.saturating_sub(1),
                        Key::Escape => {
                            event.prevent_default();
                            event.stop_propagation();
                            close_menu(open, &key_btn, true);
                            return;
                        }
                        Key::Tab => {
                            open.set(false);
                            return;
                        }
                        _ => return,
                    };
                    event.prevent_default();
                    event.stop_propagation();
                    document::eval(&format!(
                        "document.getElementById({key_menu:?})?.querySelectorAll('[role=\"menuitem\"]')[{to}]?.focus({{ preventScroll: true }});"
                    ));
                },
                for (i, item) in items.into_iter().enumerate() {
                    {menu_row(i, item, open, uid.clone(), active)}
                }
            }
        }
    }
}

fn menu_row(
    i: usize,
    item: BreadcrumbItem,
    open: Signal<bool>,
    btn_id: String,
    mut active: Signal<Option<usize>>,
) -> Element {
    let style = format!("--depth: {};", i.min(3));
    let refocus = item.href.is_none();
    let handler = item.onclick;
    let label = item.label.clone();
    let body = rsx! {
        span { class: "ui-bcr__menu-icon", "aria-hidden": "true",
            if let Some(icon) = item.icon.clone() {
                {icon}
            } else {
                {nest_icon()}
            }
        }
        span { class: "ui-bcr__menu-label", dir: "auto", "{label}" }
    };
    let pick = move |event: MouseEvent| {
        if let Some(handler) = handler {
            handler.call(event);
        }
        close_menu(open, &btn_id, refocus);
    };
    let focus = move |_| active.set(Some(i));

    match item.href {
        Some(href) => rsx! {
            a { key: "{i}", role: "menuitem", "tabindex": "-1", class: "ui-bcr__menu-item", style: "{style}",
                href: "{href}", title: "{item.label}", onfocus: focus, onclick: pick,
                {body}
            }
        },
        None => rsx! {
            button { key: "{i}", "type": "button", role: "menuitem", "tabindex": "-1", class: "ui-bcr__menu-item",
                style: "{style}", title: "{item.label}", onfocus: focus,
                aria_disabled: handler.is_none().then_some("true"),
                onclick: move |event: MouseEvent| {
                    if handler.is_some() {
                        pick(event);
                    }
                },
                {body}
            }
        },
    }
}

fn close_menu(mut open: Signal<bool>, btn_id: &str, refocus: bool) {
    open.set(false);
    if refocus {
        document::eval(&format!(
            "document.getElementById({btn_id:?})?.focus({{ preventScroll: true }});"
        ));
    }
}

/// Index moved by `step`, wrapping at both ends.
fn wrap(index: isize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        index.rem_euclid(len as isize) as usize
    }
}

/* Icons: Hugeicons Free (MIT, © Hugeicons, hugeicons.com) */
fn more_icon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "focusable": "false", "aria-hidden": "true",
            path { d: "M6 12.5V12m12 .5V12m-6 .5V12m-5 .5a1 1 0 1 0-2 0 1 1 0 0 0 2 0Zm12 0a1 1 0 1 0-2 0 1 1 0 0 0 2 0Zm-6 0a1 1 0 1 0-2 0 1 1 0 0 0 2 0Z" }
        }
    }
}

fn nest_icon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "focusable": "false",
            path { d: "M19 15h-7c-3.77 0-5.66 0-6.83-1.17C4 12.66 4 10.77 4 7V4" }
            path { d: "M15 20s5-3.68 5-5-5-5-5-5" }
        }
    }
}
