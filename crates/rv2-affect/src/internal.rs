//! Helpers shared by the ports.

use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

/// A process-unique number for DOM ids. The React originals use `useId`;
/// call this inside `use_hook` so a component keeps its id across renders.
pub(crate) fn next_id() -> usize {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// A JS expression that is true when the viewer has not asked for reduced
/// motion. For `document::eval` snippets that start animations by hand; CSS
/// animations already obey the stylesheet's own media query.
pub(crate) fn prefers_motion_script() -> &'static str {
    "!window.matchMedia('(prefers-reduced-motion: reduce)').matches"
}

/// Writes `text` to the clipboard. Fire-and-forget: a refusal (no focus, no
/// permission) is silent, as in the originals.
pub(crate) fn copy_to_clipboard(text: &str) {
    dioxus::document::eval(&format!(
        "navigator.clipboard && navigator.clipboard.writeText({text:?}).catch(() => {{}});"
    ));
}

/// Resolves after `ms` milliseconds. `rv2-affect` has no timer crate (no
/// tokio, no gloo), so this borrows the browser's `setTimeout` through eval;
/// with no renderer attached (SSR, tests) it resolves at once.
pub(crate) async fn sleep(ms: u32) {
    let _ = dioxus::document::eval(&format!(
        "await new Promise((resolve) => setTimeout(resolve, {ms})); return true;"
    ))
    .await;
}

/// Joins the component's base class with the caller's layout class.
pub(crate) fn class_list(base: &str, extra: &str) -> String {
    if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    }
}
