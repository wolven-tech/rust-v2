//! Out-of-flow surfaces: a modal dialog and its confirm form, toasts, and
//! popovers (a row menu and a help tip).
//!
//! Drawn from jbt-fit's `jbt-ui` dialog and toast. Two changes from there keep
//! this crate free of `web-sys` and timers:
//!
//! - The dialog is a native `<dialog>`. Opening it modally needs
//!   `showModal()`, which has no attribute form, so [`Dialog`] issues it through
//!   `document::eval` — part of the `lib` feature the kit already takes. The
//!   browser then supplies the backdrop, `inert` on everything behind and
//!   Escape. Scroll lock is `html:has(dialog:modal)` in `assets/overlay.css`.
//! - A toast's lifetime is a CSS animation, and `animationend` dismisses it.
//!   Hover and focus pause the animation, which is the pause-on-attention rule
//!   jbt-fit kept with a cancellable task per toast. Caution and Stop toasts
//!   have no lifetime and stay until dismissed.

use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::feedback::Verdict;
use crate::primitives::{Button, Variant};

/// Where a [`Dialog`] sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogKind {
    /// Centred panel, for a decision.
    #[default]
    Centered,
    /// Full-height panel from the inline end, for a form or a detail view
    /// beside the list it came from.
    Drawer,
}

impl DialogKind {
    fn name(self) -> &'static str {
        match self {
            DialogKind::Centered => "centered",
            DialogKind::Drawer => "drawer",
        }
    }
}

/// A JavaScript string literal for `value`. Rust's `Debug` escaping covers
/// quotes, backslashes and control characters; `<` is escaped too so an id
/// can never close a script element.
fn js_string(value: &str) -> String {
    format!("{value:?}").replace('<', "\\u003c")
}

/// A modal dialog over the page.
///
/// The caller owns `open` and hears every request to change it through
/// `on_open_change`: Escape, the close button and a click on the backdrop all
/// ask rather than close. Keep the component mounted while closed and its
/// children keep their drafts.
///
/// `alert` makes it an `alertdialog`, for a decision the reader must answer.
/// A focusable child with `autofocus` takes first focus; otherwise the browser
/// focuses the first focusable element.
#[component]
pub fn Dialog(
    id: String,
    open: bool,
    title: String,
    on_open_change: EventHandler<bool>,
    #[props(default)] description: Option<String>,
    #[props(default)] kind: DialogKind,
    /// `false` while a pending action must not be abandoned: Escape, the
    /// backdrop and the close button stop asking to close.
    #[props(default = true)]
    dismissible: bool,
    #[props(default = false)] alert: bool,
    children: Element,
) -> Element {
    // Effects run only in a browser; the drop below also runs on a server
    // render, where there is no document to evaluate against.
    let mounted = use_hook(|| Rc::new(Cell::new(false)));
    let target = js_string(&id);
    let in_effect = mounted.clone();
    use_effect(use_reactive!(|open| {
        in_effect.set(true);
        document::eval(&format!(
            "const d = document.getElementById({target}); \
             if (d && {open} && !d.open) d.showModal(); \
             else if (d && !{open} && d.open) d.close();"
        ));
    }));
    let dropped = js_string(&id);
    use_drop(move || {
        if mounted.get() {
            document::eval(&format!(
                "const d = document.getElementById({dropped}); if (d?.open) d.close();"
            ));
        }
    });

    let described = description.as_ref().map(|_| format!("{id}-description"));
    rsx! {
        dialog {
            id: "{id}",
            class: "rv2-dialog",
            "data-kind": kind.name(),
            role: if alert { "alertdialog" } else { "dialog" },
            aria_labelledby: "{id}-title",
            aria_describedby: described,
            oncancel: move |event| {
                event.prevent_default();
                if dismissible {
                    on_open_change.call(false);
                }
            },
            onclick: move |_| {
                if dismissible {
                    on_open_change.call(false);
                }
            },
            div { class: "rv2-dialog__panel", onclick: move |event| event.stop_propagation(),
                header { class: "flex items-start justify-between gap-4",
                    h2 { id: "{id}-title", class: "text-lg font-semibold text-ink", "{title}" }
                    Button {
                        variant: Variant::Ghost,
                        disabled: !dismissible,
                        onclick: move |_| on_open_change.call(false),
                        span { class: "sr-only", "Close dialog" }
                        span { aria_hidden: "true", "✕" }
                    }
                }
                if let Some(text) = description {
                    p { id: "{id}-description", class: "mt-1 text-sm text-ink-muted", "{text}" }
                }
                div { class: "mt-4", {children} }
            }
        }
    }
}

/// A destructive decision in an `alertdialog`.
///
/// First focus is the least destructive action. While `pending` it cannot be
/// dismissed and the confirm button is busy; on failure it stays open with
/// `error` shown, so the reader is never left guessing whether it happened.
#[component]
pub fn ConfirmDialog(
    id: String,
    open: bool,
    title: String,
    description: String,
    confirm_label: String,
    on_open_change: EventHandler<bool>,
    on_confirm: EventHandler<()>,
    #[props(default = "Keep it".to_string())] cancel_label: String,
    #[props(default = false)] pending: bool,
    #[props(default)] error: Option<String>,
    #[props(default)] children: Element,
) -> Element {
    rsx! {
        Dialog {
            id,
            open,
            title,
            description,
            on_open_change,
            alert: true,
            dismissible: !pending,
            if let Some(message) = error {
                crate::ErrorBanner { message }
            }
            {children}
            div { class: "mt-6 flex flex-wrap justify-end gap-2",
                Button {
                    variant: Variant::Outline,
                    autofocus: true,
                    disabled: pending,
                    onclick: move |_| on_open_change.call(false),
                    "{cancel_label}"
                }
                Button {
                    variant: Variant::Danger,
                    busy: pending,
                    onclick: move |_| on_confirm.call(()),
                    "{confirm_label}"
                }
            }
        }
    }
}

/// One notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub id: u64,
    pub verdict: Verdict,
    pub title: Option<String>,
    pub message: String,
}

impl Toast {
    /// How long it stays unattended, or `None` when it stays until dismissed.
    /// A caution or a failure is one the reader must act on, and a timeout
    /// that removes it fails WCAG 2.2.1 for anyone slower to reach it.
    #[must_use]
    pub fn life_ms(&self) -> Option<u64> {
        match self.verdict {
            Verdict::Clear => Some(3200),
            Verdict::Note => Some(4000),
            Verdict::Caution | Verdict::Stop => None,
        }
    }

    /// Caution and Stop interrupt (`alert`); the rest wait (`status`).
    #[must_use]
    pub fn assertive(&self) -> bool {
        matches!(self.verdict, Verdict::Caution | Verdict::Stop)
    }
}

/// The toasts on screen, oldest first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToastQueue {
    next_id: u64,
    toasts: Vec<Toast>,
}

impl ToastQueue {
    /// At most this many show at once; a new one drops the oldest.
    pub const MAX: usize = 4;

    pub fn push(&mut self, verdict: Verdict, title: Option<String>, message: String) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.toasts.push(Toast {
            id,
            verdict,
            title,
            message,
        });
        if self.toasts.len() > Self::MAX {
            self.toasts.remove(0);
        }
        id
    }

    pub fn dismiss(&mut self, id: u64) {
        self.toasts.retain(|toast| toast.id != id);
    }

    #[must_use]
    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }
}

/// Raises and dismisses toasts. Copy it into any handler.
#[derive(Clone, Copy, PartialEq)]
pub struct ToastHandle {
    queue: Signal<ToastQueue>,
}

impl ToastHandle {
    pub fn raise(self, verdict: Verdict, message: impl Into<String>) -> u64 {
        let mut queue = self.queue;
        queue.write().push(verdict, None, message.into())
    }

    pub fn raise_titled(
        self,
        verdict: Verdict,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> u64 {
        let mut queue = self.queue;
        queue
            .write()
            .push(verdict, Some(title.into()), message.into())
    }

    pub fn dismiss(self, id: u64) {
        let mut queue = self.queue;
        queue.write().dismiss(id);
    }
}

/// Mount once above everything that raises a toast.
#[component]
pub fn ToastProvider(children: Element) -> Element {
    let queue = use_signal(ToastQueue::default);
    use_context_provider(|| ToastHandle { queue });
    children
}

/// The handle from the nearest [`ToastProvider`]. Panics without one.
#[must_use]
pub fn use_toast() -> ToastHandle {
    use_context()
}

/// Where toasts render. Mount once, inside the [`ToastProvider`].
///
/// A named `region` landmark rather than a live region: each toast carries its
/// own `alert` or `status`, and a live container would announce it twice.
#[component]
pub fn Toaster() -> Element {
    let handle = use_toast();
    rsx! {
        div { class: "rv2-toaster", role: "region", aria_label: "Notifications",
            for toast in handle.queue.read().toasts().iter().cloned() {
                ToastView { key: "{toast.id}", toast, handle }
            }
        }
    }
}

#[component]
fn ToastView(toast: Toast, handle: ToastHandle) -> Element {
    let id = toast.id;
    let life = toast.life_ms();
    let edge = toast.verdict.edge();
    let role = if toast.assertive() { "alert" } else { "status" };
    rsx! {
        div {
            class: "rv2-toast border-l-4 {edge}",
            style: life.map(|ms| format!("--rv2-toast-life: {ms}ms")),
            "data-held": life.is_none().then_some("true"),
            role,
            aria_atomic: "true",
            onanimationend: move |event: AnimationEvent| {
                if event.animation_name() == "rv2-toast-life" {
                    handle.dismiss(id);
                }
            },
            div { class: "min-w-0 flex-1",
                if let Some(title) = toast.title {
                    p { class: "text-sm font-semibold {toast.verdict.ink()}", "{title}" }
                }
                p { class: "text-sm text-ink", "{toast.message}" }
            }
            Button { variant: Variant::Ghost, onclick: move |_| handle.dismiss(id),
                span { class: "sr-only", "Dismiss notification" }
                span { aria_hidden: "true", "✕" }
            }
        }
    }
}

/// Which edge of its trigger a [`Popover`] lines up with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverAlign {
    #[default]
    Start,
    End,
}

impl PopoverAlign {
    fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// What a [`Popover`]'s panel holds, which sets its role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverRole {
    /// Controls or links; no role.
    #[default]
    Content,
    /// An explanation the reader asked for: `role="note"`.
    Note,
}

/// A panel opened by a button, on the native `popover` attribute.
///
/// The browser supplies light dismiss, Escape, one-open-at-a-time and the top
/// layer, and exposes the trigger's expanded state, so none of it is scripted.
/// Where CSS anchor positioning is supported the panel sits under its trigger;
/// elsewhere the browser centres it, which is still usable.
///
/// `id` names the panel and its anchor, so it must be a valid CSS identifier.
#[component]
pub fn Popover(
    id: String,
    /// The trigger's accessible name. Required, because a trigger is often a
    /// glyph.
    label: String,
    /// What the trigger shows.
    trigger: Element,
    #[props(default)] align: PopoverAlign,
    #[props(default)] role: PopoverRole,
    #[props(default)] labelledby: Option<String>,
    children: Element,
) -> Element {
    let anchor = format!("--rv2-pop-{id}");
    let role = match role {
        PopoverRole::Content => None,
        PopoverRole::Note => Some("note"),
    };
    rsx! {
        button {
            r#type: "button",
            class: "rv2-popover-trigger",
            popovertarget: "{id}",
            aria_label: "{label}",
            style: "anchor-name: {anchor}",
            {trigger}
        }
        div {
            id: "{id}",
            class: "rv2-popover",
            popover: "auto",
            role,
            aria_labelledby: labelledby,
            "data-align": align.name(),
            style: "position-anchor: {anchor}",
            {children}
        }
    }
}

/// A row's secondary actions behind a "⋯" button.
///
/// Children are the actions, usually Ghost [`Button`]s. The panel is a plain
/// list of buttons and not `role="menu"`, which would promise arrow-key roving
/// that a script has to provide.
///
/// Choosing an action closes the menu. A native popover stays open on a click
/// inside it, so this one call goes through `document::eval`, as `Dialog`'s
/// `showModal()` does.
#[component]
pub fn RowMenu(
    id: String,
    #[props(default = "More actions".to_string())] label: String,
    children: Element,
) -> Element {
    let hide = format!("document.getElementById({})?.hidePopover()", js_string(&id));
    rsx! {
        Popover {
            id,
            label,
            align: PopoverAlign::End,
            trigger: rsx! { span { aria_hidden: "true", "⋯" } },
            div {
                class: "rv2-row-menu",
                onclick: move |_| {
                    document::eval(&hide);
                },
                {children}
            }
        }
    }
}

/// A "?" that opens a short explanation beside the thing it explains.
///
/// Help a reader needs in order to fill a field belongs in the field's own help
/// text; this is for the definition someone may want once.
#[component]
pub fn HelpTip(
    id: String,
    /// What the "?" explains, as its accessible name.
    label: String,
    title: String,
    children: Element,
) -> Element {
    let title_id = format!("{id}-tip-title");
    rsx! {
        Popover {
            id,
            label,
            role: PopoverRole::Note,
            labelledby: title_id.clone(),
            trigger: rsx! { span { aria_hidden: "true", "?" } },
            p { id: "{title_id}", class: "text-sm font-semibold text-ink", "{title}" }
            div { class: "mt-1 text-sm text-ink-muted", {children} }
        }
    }
}
