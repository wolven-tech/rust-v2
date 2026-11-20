//! Tooltip: a small dark label for one control. Shows on hover after a wait and at once on keyboard focus.
//!
//! Port losses: `children` is wrapped in a `display: contents` span, which takes the pointer and focus
//! events. The label is `String`, not a node, so the quiet-label rule (the label repeating the trigger's
//! own name) is not applied. Opening a second tooltip closes the first through a thread-local.

use std::cell::Cell;

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};
use crate::popover::{
    Align, Placement, Side, listen, listen_js, placed_style, refresh, release_js,
};

const CSS: Asset = asset!("/assets/tooltip.css");

/// A neighbour that opens within this many ms of the last one closing shows at once.
pub const SKIP_MS: u32 = 300;
/// The pointer may cross the gap onto the label without it closing.
pub const GRACE_MS: u32 = 120;

thread_local! {
    /// The tooltip that is open, and its signal, so that opening another one closes it.
    static OPEN: Cell<Option<(usize, Signal<bool>)>> = const { Cell::new(None) };
    /// Set for `SKIP_MS` after a close, so the next label appears without a wait.
    static WARM: Cell<bool> = const { Cell::new(false) };
}

/// Whether a label has anything to show.
pub fn has_label(content: &str) -> bool {
    !content.is_empty()
}

#[component]
pub fn Tooltip(
    #[props(into)] content: String,
    #[props(default, into)] shortcut: Option<String>,
    #[props(default = Side::Top)] side: Side,
    #[props(default = Align::Center)] align: Align,
    #[props(default = 6.0)] offset: f64,
    #[props(default = 500)] delay: u32,
    #[props(default)] disabled: bool,
    /// Controls it from outside (with `onopenchange`); leave it out and it opens on its own.
    #[props(default)]
    open: Option<bool>,
    #[props(default)] onopenchange: Option<EventHandler<bool>>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let uid = use_hook(next_id);
    let tip_id = format!("ui-tip-{uid}");
    let anchor_id = format!("ui-tip-anchor-{uid}");
    let anchor = format!("document.getElementById({anchor_id:?})?.firstElementChild");
    let empty = !has_label(&content);
    let mut shown = use_signal(|| false);
    let mut instant = use_signal(|| false);
    let mut blocked = use_signal(|| false);
    let mut pointer_focus = use_signal(|| false);
    let mut generation = use_signal(|| 0u32);
    let mut card = use_signal(|| None::<MountedEvent>);
    let mut placed = use_signal(|| None::<Placement>);
    let open_now = !disabled && !empty && open.unwrap_or(shown());

    let mut change = move |next: bool| {
        generation.set(generation() + 1);
        if open.is_none() {
            shown.set(next);
        }
        if next != open_now
            && let Some(handler) = onopenchange
        {
            handler.call(next);
        }
        if next {
            if let Some((other, mut signal)) = OPEN.get()
                && other != uid
            {
                signal.set(false);
            }
            OPEN.set(Some((uid, shown)));
        } else if OPEN.get().is_some_and(|(id, _)| id == uid) {
            OPEN.set(None);
            WARM.set(true);
            spawn(async move {
                sleep(SKIP_MS).await;
                WARM.set(false);
            });
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

    let mut show = move |now: bool| {
        if disabled || empty || blocked() || shown() {
            return;
        }
        let warm = WARM.get() || OPEN.get().is_some_and(|(id, _)| id != uid);
        if now || warm {
            instant.set(warm);
            change(true);
        } else {
            instant.set(false);
            schedule(delay, true);
        }
    };

    let mut hide = move |later: bool| {
        if later {
            schedule(GRACE_MS, false);
        } else {
            generation.set(generation() + 1);
            change(false);
        }
    };

    let setup = r#"on(window, "keydown", (e) => {
  if (e.key !== "Escape" || e.defaultPrevented) return;
  e.preventDefault();
  e.stopPropagation();
  send("escape");
}, true);
on(window, "resize", () => send("move"));
on(window, "scroll", () => send("move"), true);"#;

    use_effect(use_reactive((&open_now,), {
        let tip_id = tip_id.clone();
        let anchor_id = anchor_id.clone();
        move |(open_now,)| {
            if !open_now {
                placed.set(None);
                document::eval(&release_js(&tip_id));
                document::eval(&format!(
                    "document.getElementById({anchor_id:?})?.firstElementChild?.removeAttribute('aria-describedby');"
                ));
                return;
            }
            document::eval(&format!(
                "document.getElementById({anchor_id:?})?.firstElementChild?.setAttribute('aria-describedby', {tip_id:?});"
            ));
            let anchor = format!("document.getElementById({anchor_id:?})?.firstElementChild");
            listen(listen_js(&tip_id, setup), move |message| {
                match message.as_str() {
                    "move" => refresh(card, placed, anchor.clone(), side, align, offset),
                    "escape" => {
                        blocked.set(true);
                        change(false);
                    }
                    _ => {}
                }
            });
        }
    }));

    let shown_side = placed().map_or(side, |p| p.side);
    let style = placed_style(placed(), None);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        span {
            id: "{anchor_id}",
            style: "display: contents",
            onpointerenter: move |event| {
                if event.data().pointer_type() != "touch" {
                    show(false);
                }
            },
            onpointerleave: move |event| {
                blocked.set(false);
                if event.data().pointer_type() != "touch" {
                    hide(true);
                }
            },
            onpointerdown: move |_| {
                pointer_focus.set(true);
                blocked.set(true);
                hide(false);
            },
            onclick: move |_| {
                blocked.set(true);
                hide(false);
            },
            onfocusin: move |_| {
                if !pointer_focus() {
                    show(true);
                }
                pointer_focus.set(false);
            },
            onfocusout: move |_| {
                blocked.set(false);
                hide(false);
            },
            {children}
            if open_now {
                span {
                    id: "{tip_id}",
                    role: "tooltip",
                    class: class_list("ui-tip", &class),
                    popover: "manual",
                    "data-side": shown_side.as_str(),
                    "data-instant": instant().then_some("true"),
                    style: "{style}",
                    onmounted: move |event: MountedEvent| {
                        card.set(Some(event.clone()));
                        refresh(card, placed, anchor.clone(), side, align, offset);
                        document::eval(&format!("document.getElementById({tip_id:?})?.showPopover?.();"));
                    },
                    onpointerenter: move |_| generation.set(generation() + 1),
                    onpointerleave: move |_| hide(true),
                    span { class: "ui-tip__text", dir: "auto", "{content}" }
                    if let Some(keys) = shortcut {
                        kbd { class: "ui-tip__keys", "{keys}" }
                    }
                }
            }
        }
    }
}
