//! Profile menu: the signed-in user, a list of account actions, and profile setup progress as the card's last row.
//!
//! Port losses: `ref` is not forwarded. A broken avatar image does not fall back to the default avatar (the browser
//! does not bubble image errors); pass an avatar that handles its own `onerror`.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};
use crate::pagination::group_digits;

const CSS: Asset = asset!("/assets/profile-menu.css");

/// Fills the setup bar once the card is at least 30% on screen. The attribute is set from script only, so Rust
/// re-renders leave it alone.
const FILL_ON_VIEW: &str = r#"
const el = document.getElementById("__ROOT__");
if (el && typeof IntersectionObserver !== "undefined") {
  const io = new IntersectionObserver(([entry]) => {
    if (entry.isIntersecting) {
      el.setAttribute("data-filled", "");
      io.disconnect();
    }
  }, { threshold: 0.3 });
  io.observe(el);
}
"#;

/// A row of the menu.
#[derive(Clone, PartialEq)]
pub struct ProfileMenuItem {
    pub id: String,
    pub label: String,
    pub icon: Option<Element>,
    /// A small pill on the right, e.g. "New".
    pub badge: Option<String>,
    pub onselect: Option<EventHandler<()>>,
}

/// Progress through profile setup, with the next open step shown under the bar.
#[derive(Clone, PartialEq)]
pub struct ProfileSetup {
    pub done: usize,
    pub total: usize,
    pub next: Option<String>,
    pub on_continue: Option<EventHandler<()>>,
}

/// Keys the menu answers to. Anything else is left to the browser.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavKey {
    Down,
    Up,
    Home,
    End,
}

/// The fraction of setup done, held between 0 and 1. No steps at all reads as 0.
pub fn setup_ratio(done: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        done.min(total) as f64 / total as f64
    }
}

/// The percentage shown to assistive tech, rounded half up.
pub fn setup_percent(done: usize, total: usize) -> u32 {
    (setup_ratio(done, total) * 100.0).round() as u32
}

/// The menu item that the arrow key moves to from `current`. Down and up wrap at the ends; with nothing focused, Down
/// takes the first item and Up the last.
pub fn next_index(current: Option<usize>, len: usize, key: NavKey) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match key {
        NavKey::Down => current.map_or(0, |i| (i + 1) % len),
        NavKey::Up => current.map_or(len - 1, |i| (i + len - 1) % len),
        NavKey::Home => 0,
        NavKey::End => len - 1,
    })
}

/// The account actions shown when `items` is not given.
pub fn default_items() -> Vec<ProfileMenuItem> {
    vec![
        ProfileMenuItem {
            id: "settings".to_string(),
            label: "Settings".to_string(),
            icon: Some(settings_icon()),
            badge: None,
            onselect: None,
        },
        ProfileMenuItem {
            id: "docs".to_string(),
            label: "Docs".to_string(),
            icon: Some(docs_icon()),
            badge: None,
            onselect: None,
        },
        ProfileMenuItem {
            id: "apps".to_string(),
            label: "Apps".to_string(),
            icon: Some(apps_icon()),
            badge: Some("New".to_string()),
            onselect: None,
        },
        ProfileMenuItem {
            id: "logout".to_string(),
            label: "Logout".to_string(),
            icon: Some(logout_icon()),
            badge: None,
            onselect: None,
        },
    ]
}

/// An account card with the signed-in user, a menu of actions and, when `setup` is given, profile setup progress.
#[component]
pub fn ProfileMenu(
    #[props(into)] name: String,
    #[props(into)] email: String,
    #[props(default)] avatar: Option<Element>,
    #[props(default)] items: Option<Vec<ProfileMenuItem>>,
    #[props(default)] setup: Option<ProfileSetup>,
    #[props(default)] class: String,
) -> Element {
    let uid = use_hook(|| format!("ui-pm-{}", next_id()));
    let list_id = format!("{uid}-items");
    let list = items.unwrap_or_else(default_items);
    let len = list.len();
    let shown_name = name.trim().to_string();
    let focused = use_signal(|| None::<usize>);

    let fill_script = FILL_ON_VIEW.replace("__ROOT__", &uid);
    use_effect(move || {
        document::eval(&fill_script);
    });

    let keys_list = list_id.clone();
    let on_keys = move |event: KeyboardEvent| {
        let key = match event.key() {
            Key::ArrowDown => NavKey::Down,
            Key::ArrowUp => NavKey::Up,
            Key::Home => NavKey::Home,
            Key::End => NavKey::End,
            _ => return,
        };
        event.prevent_default();
        if let Some(i) = next_index(focused(), len, key) {
            document::eval(&format!(
                "document.getElementById({keys_list:?})?.querySelectorAll(\"button\")[{i}]?.focus();"
            ));
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        section { id: "{uid}", class: class_list("ui-pmc", &class),
            div { class: "ui-pmc__card",
                header { class: "ui-pmc__user",
                    span { class: "ui-pmc__avatar", {avatar.unwrap_or_else(default_avatar)} }
                    span { class: "ui-pmc__who",
                        if !shown_name.is_empty() {
                            span { class: "ui-pmc__name", dir: "auto", title: "{shown_name}", "{shown_name}" }
                        }
                        span { class: "ui-pmc__email", dir: "auto", title: "{email}", "{email}" }
                    }
                }

                div { id: "{list_id}", class: "ui-pmc__items", role: "menu", aria_label: "Account", onkeydown: on_keys,
                    for (i, item) in list.into_iter().enumerate() {
                        {item_view(i, item, focused)}
                    }
                }
                if let Some(setup) = setup {
                    {setup_view(setup)}
                }
            }
        }
    }
}

fn item_view(i: usize, item: ProfileMenuItem, mut focused: Signal<Option<usize>>) -> Element {
    let label = item.label.clone();
    let onselect = item.onselect;
    rsx! {
        button {
            key: "{i}-{item.id}",
            "type": "button",
            role: "menuitem",
            class: "ui-pmc__item",
            onfocus: move |_| focused.set(Some(i)),
            onclick: move |_| {
                if let Some(handler) = onselect {
                    handler.call(());
                }
            },
            span { class: "ui-pmc__icon", {item.icon} }
            span { class: "ui-pmc__label", dir: "auto", title: "{label}", "{label}" }
            if let Some(badge) = item.badge {
                span { class: "ui-pmc__badge", dir: "auto", title: "{badge}", "{badge}" }
            }
        }
    }
}

/// Compact setup progress: a count, a bar, and the next step on one line.
fn setup_view(setup: ProfileSetup) -> Element {
    let done = setup.done.min(setup.total);
    let ratio = setup_ratio(setup.done, setup.total);
    let percent = setup_percent(setup.done, setup.total);
    let count = format!("{}/{}", group_digits(done), group_digits(setup.total));
    let open = setup.next.is_some() && done < setup.total;
    let next_label = setup.next.clone().unwrap_or_default();
    let on_continue = setup.on_continue;
    rsx! {
        div { class: "ui-pmc__setup",
            div { class: "ui-pmc__setup-head",
                span { class: "ui-pmc__setup-title", "Profile setup" }
                span { class: "ui-pmc__setup-count", "{count}" }
            }
            span {
                class: "ui-pmc__track",
                role: "progressbar",
                aria_label: "Profile setup",
                aria_valuemin: "0",
                aria_valuemax: "100",
                aria_valuenow: "{percent}",
                span { class: "ui-pmc__fill", style: "transform: scaleX({ratio});" }
            }
            if open {
                button {
                    "type": "button",
                    class: "ui-pmc__next",
                    onclick: move |_| {
                        if let Some(handler) = on_continue {
                            handler.call(());
                        }
                    },
                    span { class: "ui-pmc__next-label", dir: "auto", title: "{next_label}",
                        span { class: "ui-pmc__next-hint", "Next:" }
                        " {next_label}"
                    }
                    {arrow_icon()}
                }
            }
        }
    }
}

/// Default avatar: a person icon centered on a soft disc.
fn default_avatar() -> Element {
    rsx! {
        svg { view_box: "0 0 40 40", "aria-hidden": "true",
            circle { cx: "20", cy: "20", r: "20", fill: "currentColor", opacity: "0.1" }
            circle { cx: "20", cy: "16", r: "4.75", fill: "currentColor" }
            path { d: "M11.5 28.25c.9-4 4.3-6.75 8.5-6.75s7.6 2.75 8.5 6.75c.15.7-.4 1.25-1.1 1.25H12.6c-.7 0-1.25-.55-1.1-1.25Z", fill: "currentColor" }
        }
    }
}

fn arrow_icon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.1", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M5 12h14M13 6l6 6-6 6" }
        }
    }
}

fn outline_icon(children: Element) -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.1", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            {children}
        }
    }
}

pub fn settings_icon() -> Element {
    outline_icon(rsx! {
        path { d: "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" }
        circle { cx: "12", cy: "12", r: "3" }
    })
}

pub fn language_icon() -> Element {
    outline_icon(rsx! {
        path { d: "m5 8 6 6M4 14l6-6 2-3M2 5h12M7 2h1M22 22l-5-10-5 10M14 18h6" }
    })
}

pub fn docs_icon() -> Element {
    outline_icon(rsx! {
        path { d: "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" }
        path { d: "M14 2v4a2 2 0 0 0 2 2h4M16 13H8M16 17H8M10 9H8" }
    })
}

pub fn apps_icon() -> Element {
    outline_icon(rsx! {
        rect { x: "3", y: "3", width: "7", height: "7", rx: "1.5" }
        rect { x: "3", y: "14", width: "7", height: "7", rx: "1.5" }
        rect { x: "14", y: "14", width: "7", height: "7", rx: "1.5" }
        path { d: "M17.5 3v7M14 6.5h7" }
    })
}

pub fn logout_icon() -> Element {
    outline_icon(rsx! {
        path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9" }
    })
}
