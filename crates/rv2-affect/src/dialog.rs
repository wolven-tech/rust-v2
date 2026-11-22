//! Dialog and Alert Dialog: a modal card on a dimmed backdrop, built on a native `<dialog>`.
//!
//! Modal layers call `showModal()`, so the browser traps focus and makes the page inert. Contained layers
//! call `show()` and trap Tab in JS. Escape and a press on the backdrop report to Rust through the listener
//! script; the focus returns to the element that had it before the open, and the page scroll is locked
//! while any modal layer is open (counted, so stacked layers unlock once).
//!
//! Port losses: the confirm action of an Alert Dialog is synchronous; the React version keeps the dialog
//! open with a busy button until a returned promise settles. The focus returns to the element focused
//! at open, not to the trigger that was clicked (Safari does not focus buttons on click).

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};
use crate::popover::{listen, listen_js, release_js};

const CSS: Asset = asset!("/assets/dialog.css");

/// How long the way out takes: the backdrop fades, the card shrinks.
const EXIT_MS: u32 = 200;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogSize {
    /// 400px.
    Sm,
    /// 480px.
    #[default]
    Md,
    /// 720px.
    Lg,
}

impl DialogSize {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

/// The Tab wrap for a contained layer: from the last control forward (or the first backward) goes round.
/// `None` lets the browser move the focus itself.
pub fn tab_wrap(count: usize, current: Option<usize>, shift: bool) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let last = count - 1;
    match current {
        Some(at) if shift && at == 0 => Some(last),
        Some(at) if !shift && at == last => Some(0),
        Some(at) if at < count => None,
        _ if shift => Some(last),
        _ => Some(0),
    }
}

fn layer_open_script(id: &str, modal: bool) -> String {
    format!(
        r#"const el = document.getElementById({id:?});
if (!el) return;
const stack = (window.__affectScroll ||= {{ n: 0, overflow: "", gutter: "" }});
const html = document.documentElement;
const back = document.activeElement;
const modal = {modal};
if (!el.open) {{ modal ? el.showModal() : el.show(); }}
let locked = false;
if (modal) {{
  if (stack.n++ === 0) {{
    const bar = window.innerWidth - html.clientWidth;
    stack.overflow = html.style.overflow;
    stack.gutter = html.style.scrollbarGutter;
    html.style.overflow = "hidden";
    if (bar > 0) html.style.scrollbarGutter = "stable";
  }}
  locked = true;
}}
const firstControl = (root) => root ? [...root.querySelectorAll("input, textarea, select, button, [href], [tabindex]")].find((c) => c.tabIndex >= 0 && !c.disabled && c.getClientRects().length > 0) : null;
const target = el.querySelector("[data-autofocus]") ?? firstControl(el.querySelector(".ui-dlgc__body")) ?? firstControl(el.querySelector(".ui-dlgc__foot")) ?? firstControl(el) ?? el.querySelector(".ui-dlgc__panel");
target?.focus({{ preventScroll: true }});
const cleanup = [];
cleanup.push(() => {{ if (locked && --stack.n === 0) {{ html.style.overflow = stack.overflow; html.style.scrollbarGutter = stack.gutter; }} }});
cleanup.push(() => {{ if (el.open) el.close(); }});
cleanup.push(() => {{ if (back?.isConnected && (el.contains(document.activeElement) || !document.activeElement || document.activeElement === document.body)) back.focus({{ preventScroll: true }}); }});
"#
    )
}

/// Listener setup for an open layer: Escape and the native cancel report "cancel", a press on the
/// backdrop reports "backdrop", and the browser closing the dialog reports "closed".
fn layer_listeners(id: &str, modal: bool) -> String {
    format!(
        r#"{open}
let down = false;
on(el, "cancel", (e) => {{ e.preventDefault(); send("cancel"); }});
on(el, "close", () => {{ if (!el.open) return; send("closed"); }});
on(el, "pointerdown", (e) => {{ down = e.target === el; }});
on(el, "click", (e) => {{ if (down && e.target === el) send("backdrop"); down = false; }});
on(el, "keydown", (e) => {{
  if (!modal && e.key === "Escape") {{ e.stopPropagation(); send("cancel"); }}
  if (e.key === "Tab") {{
    const list = [...el.querySelectorAll("input, textarea, select, button, [href], [tabindex]")].filter((c) => c.tabIndex >= 0 && !c.disabled);
    const at = list.indexOf(document.activeElement);
    if (!list.length) {{ e.preventDefault(); return; }}
    if (e.shiftKey && (at <= 0)) {{ e.preventDefault(); list[list.length - 1].focus(); }}
    else if (!e.shiftKey && (at === -1 || at === list.length - 1)) {{ e.preventDefault(); list[0].focus(); }}
  }}
}});
cleanup.forEach((undo) => handlers.push(undo));"#,
        open = layer_open_script(id, modal),
    )
}

#[component]
fn Layer(
    open: bool,
    ondismiss: EventHandler<()>,
    title: Element,
    #[props(default)] description: Option<Element>,
    #[props(default)] size: DialogSize,
    #[props(default, into)] width: Option<String>,
    #[props(default)] contained: bool,
    #[props(default = true)] close_on_backdrop: bool,
    #[props(default = true)] close_button: bool,
    #[props(default = "Close".to_string(), into)] close_label: String,
    #[props(default)] alert: bool,
    #[props(default)] busy: bool,
    #[props(default)] footer: Option<Element>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let uid = use_hook(next_id);
    let layer_id = format!("ui-dlgc-{uid}");
    let title_id = format!("{layer_id}-title");
    let desc_id = format!("{layer_id}-desc");
    let mut shown = use_signal(|| open);
    let mut leaving = use_signal(|| false);
    let mut blocked = use_signal(|| busy);
    use_effect(use_reactive((&busy,), move |(now_busy,)| {
        blocked.set(now_busy)
    }));

    use_effect(use_reactive((&open,), {
        let layer_id = layer_id.clone();
        move |(now_open,)| {
            if now_open {
                shown.set(true);
                leaving.set(false);
                return;
            }
            if !shown() {
                return;
            }
            leaving.set(true);
            let layer_id = layer_id.clone();
            spawn(async move {
                sleep(EXIT_MS).await;
                document::eval(&release_js(&layer_id));
                shown.set(false);
                leaving.set(false);
            });
        }
    }));

    if !shown() {
        return rsx! {};
    }
    let modal = !contained;
    let style = width
        .map(|w| format!("--ui-dlgc-w: {w};"))
        .unwrap_or_default();
    let class = class_list("ui-dlgc", &class);
    let describedby = description.is_some().then(|| desc_id.clone());
    let role = if alert { "alertdialog" } else { "dialog" };
    let listeners = layer_listeners(&layer_id, modal);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        dialog {
            id: "{layer_id}",
            class,
            role,
            aria_modal: contained.then_some("true"),
            aria_labelledby: title_id.clone(),
            aria_describedby: describedby,
            aria_busy: busy.then_some("true"),
            "data-contained": contained.then_some("true"),
            "data-leaving": leaving().then_some("true"),
            style: "{style}",
            onmounted: {
                let layer_id = layer_id.clone();
                move |_: MountedEvent| {
                    let layer_id = layer_id.clone();
                    let listeners = listeners.clone();
                    listen(listen_js(&layer_id, &listeners), move |message| match message.as_str() {
                        "cancel" | "closed" | "backdrop" => {
                            if message == "backdrop" && !close_on_backdrop {
                                return;
                            }
                            if !blocked() {
                                ondismiss.call(());
                            }
                        }
                        _ => {}
                    });
                }
            },
            div {
                class: "ui-dlgc__panel",
                "data-size": size.as_str(),
                "data-alert": alert.then_some("true"),
                tabindex: "-1",
                div { class: "ui-dlgc__head",
                    div { class: "ui-dlgc__heading",
                        h2 { class: "ui-dlgc__title", id: "{title_id}", {title} }
                        if let Some(description) = description {
                            p { class: "ui-dlgc__desc", id: "{desc_id}", {description} }
                        }
                    }
                    if close_button {
                        button {
                            r#type: "button",
                            class: "ui-dlgc__close",
                            aria_label: "{close_label}",
                            title: "{close_label}",
                            onclick: move |_| ondismiss.call(()),
                            svg { view_box: "0 0 24 24", fill: "none", "aria-hidden": "true", "focusable": "false",
                                path { d: "M18 6L6.00081 17.9992M17.9992 18L6 6.00085", stroke: "currentColor", stroke_width: "1.9", stroke_linecap: "round", stroke_linejoin: "round" }
                            }
                        }
                    }
                }
                div { class: "ui-dlgc__body", {children} }
                if let Some(footer) = footer {
                    div { class: "ui-dlgc__foot", {footer} }
                }
            }
        }
    }
}

#[component]
pub fn Dialog(
    /// Controls it from outside (with `onopenchange`); leave it out and `trigger` opens it.
    #[props(default)]
    open: Option<bool>,
    #[props(default)] default_open: bool,
    #[props(default)] onopenchange: Option<EventHandler<bool>>,
    /// A control that opens it; it is wrapped in a `display: contents` span.
    #[props(default)]
    trigger: Option<Element>,
    title: Element,
    #[props(default)] description: Option<Element>,
    #[props(default)] size: DialogSize,
    #[props(default, into)] width: Option<String>,
    #[props(default)] contained: bool,
    #[props(default = true)] close_on_backdrop: bool,
    #[props(default = true)] close_button: bool,
    #[props(default = "Close".to_string(), into)] close_label: String,
    #[props(default)] footer: Option<Element>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let mut own = use_signal(|| default_open);
    let is_open = open.unwrap_or(own());
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
    rsx! {
        if let Some(trigger) = trigger {
            span { style: "display: contents", aria_haspopup: "dialog", aria_expanded: "{is_open}", onclick: move |_| set_open(true), {trigger} }
        }
        Layer {
            open: is_open,
            ondismiss: move |_| set_open(false),
            title,
            description,
            size,
            width,
            contained,
            close_on_backdrop,
            close_button,
            close_label,
            footer,
            class,
            {children}
        }
    }
}

#[component]
pub fn AlertDialog(
    #[props(default)] open: Option<bool>,
    #[props(default)] default_open: bool,
    #[props(default)] onopenchange: Option<EventHandler<bool>>,
    #[props(default)] trigger: Option<Element>,
    title: Element,
    #[props(default)] description: Option<Element>,
    #[props(default = DialogSize::Sm)] size: DialogSize,
    #[props(default, into)] width: Option<String>,
    #[props(default)] contained: bool,
    #[props(default = "Confirm".to_string(), into)] confirm_label: String,
    #[props(default = "Cancel".to_string(), into)] cancel_label: String,
    #[props(default)] onconfirm: Option<EventHandler<()>>,
    #[props(default)] oncancel: Option<EventHandler<()>>,
    /// A red confirm button, for things that can't be undone.
    #[props(default)]
    danger: bool,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let mut own = use_signal(|| default_open);
    let is_open = open.unwrap_or(own());
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
    let mut cancel = move || {
        if let Some(handler) = oncancel {
            handler.call(());
        }
        set_open(false);
    };
    rsx! {
        if let Some(trigger) = trigger {
            span { style: "display: contents", aria_haspopup: "dialog", aria_expanded: "{is_open}", onclick: move |_| set_open(true), {trigger} }
        }
        Layer {
            open: is_open,
            ondismiss: move |_| cancel(),
            title,
            description,
            size,
            width,
            contained,
            close_on_backdrop: false,
            alert: true,
            class,
            footer: rsx! {
                button { r#type: "button", class: "ui-dlgc__btn", onclick: move |_| cancel(), span { class: "ui-dlgc__btn-label", "{cancel_label}" } }
                button {
                    r#type: "button",
                    class: "ui-dlgc__btn",
                    "data-tone": if danger { "danger" } else { "primary" },
                    onclick: move |_| {
                        if let Some(handler) = onconfirm {
                            handler.call(());
                        }
                        set_open(false);
                    },
                    span { class: "ui-dlgc__btn-label", "{confirm_label}" }
                }
            },
            {children}
        }
    }
}
