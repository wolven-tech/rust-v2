//! Helpers shared by the kit's components.

#![allow(dead_code)] // Each helper is used by some components, not all.

use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;

/// The token sheet every component links before its own stylesheet.
pub(crate) const KIT_CSS: Asset = asset!("/assets/kit.css");

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

/// A process-unique number for DOM ids; call inside `use_hook`.
pub(crate) fn next_id() -> usize {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// JS expression, true unless the viewer asked for reduced motion.
pub(crate) fn prefers_motion_script() -> &'static str {
    "!window.matchMedia('(prefers-reduced-motion: reduce)').matches"
}

/// Fire-and-forget clipboard write; a refusal is silent.
pub(crate) fn copy_to_clipboard(text: &str) {
    document::eval(&format!(
        "navigator.clipboard && navigator.clipboard.writeText({text:?}).catch(() => {{}});"
    ));
}

/// Resolves after `ms` through the browser's `setTimeout`; with no renderer
/// attached (SSR, tests) it resolves at once.
pub(crate) async fn sleep(ms: u32) {
    let _ = document::eval(&format!(
        "await new Promise((resolve) => setTimeout(resolve, {ms})); return true;"
    ))
    .await;
}

/// The current time in epoch milliseconds for time-based components.
///
/// `fixed` (a host-supplied clock) wins; otherwise the browser clock is read
/// every `tick_ms`. Never anchor elapsed times to mount time: a run that
/// started before the component mounted must show its true age. With no
/// renderer (SSR, tests) and no `fixed`, this is 0.
pub(crate) fn use_now_ms(fixed: Option<u64>, tick_ms: u32) -> u64 {
    let mut now = use_signal(|| fixed.unwrap_or(0));
    use_hook(move || {
        if fixed.is_none() {
            spawn(async move {
                loop {
                    let Ok(value) = document::eval("return Date.now();").await else {
                        break;
                    };
                    if let Some(ms) = value.as_f64() {
                        now.set(ms as u64);
                    }
                    sleep(tick_ms).await;
                }
            });
        }
    });
    fixed.unwrap_or_else(|| now())
}

/// Rendered width in CSS px of the element with `id`, kept current by a
/// `ResizeObserver` in the browser. `None` until the first measurement, which is
/// what SSR and tests see.
pub(crate) fn use_element_width(id: String) -> Option<f64> {
    let mut width = use_signal(|| None::<f64>);
    let observe_id = id.clone();
    use_hook(move || {
        spawn(async move {
            let script = format!(
                "const id = {observe_id:?};\
                 let el = null;\
                 for (let i = 0; i < 600 && !(el = document.getElementById(id)); i++) \
                     await new Promise((resolve) => requestAnimationFrame(resolve));\
                 if (!el) return;\
                 const observers = (window.__kitWidthObservers ||= new Map());\
                 observers.get(id)?.disconnect();\
                 const measure = () => dioxus.send(el.getBoundingClientRect().width);\
                 const observer = new ResizeObserver(measure);\
                 observer.observe(el);\
                 observers.set(id, observer);\
                 measure();\
                 await new Promise(() => {{}});"
            );
            let mut eval = document::eval(&script);
            while let Ok(value) = eval.recv::<f64>().await {
                width.set(Some(value));
            }
        })
    });
    use_drop(move || {
        document::eval(&format!(
            "window.__kitWidthObservers?.get({id:?})?.disconnect(); window.__kitWidthObservers?.delete({id:?});"
        ));
    });
    width()
}

/// Moves focus to the element with `id` after the next two animation frames.
///
/// Returning focus while the closing panel is still mounted (or before the
/// target has re-rendered) lands it on `<body>`. Use this for every "close and
/// return focus to the trigger" path.
pub(crate) fn focus_after_frame(id: &str) {
    document::eval(&format!(
        "requestAnimationFrame(() => requestAnimationFrame(() => \
         document.getElementById({id:?})?.focus({{ preventScroll: true }})));"
    ));
}

/// Keeps the scroll container `thread_id` pinned to its end while the user has not scrolled up. Its first child is
/// the stream whose size changes as charts draw, so the pin follows size changes rather than running once at mount.
pub(crate) fn follow_thread_end(thread_id: &str) {
    document::eval(&format!(
        "const el = document.getElementById({thread_id:?}); const stream = el && el.firstElementChild; \
         if (stream && !el.dataset.follow) {{ el.dataset.follow = '1'; let stick = true; \
         el.addEventListener('scroll', () => {{ stick = el.scrollTop + el.clientHeight >= el.scrollHeight - 24; }}); \
         new ResizeObserver(() => {{ if (stick) el.scrollTop = el.scrollHeight; }}).observe(stream); }}"
    ));
}

/// Joins a component's base class with the caller's layout class.
pub(crate) fn class_list(base: &str, extra: &str) -> String {
    if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    }
}
