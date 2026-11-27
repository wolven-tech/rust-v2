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

// The browser helpers below call the DOM through web-sys rather than
// `document::eval`. An eval'd string needs `script-src 'unsafe-eval'`, so under a
// strict content security policy (`'self' 'wasm-unsafe-eval'`, which a desktop
// shell such as Tauri wants) every kit component that slept, read the clock or
// measured itself died with an `EvalError`. Off wasm (SSR, tests) there is no
// DOM: each helper does what the eval did there, which was to fail silently.

/// Fire-and-forget clipboard write; a refusal is silent.
pub(crate) fn copy_to_clipboard(text: &str) {
    #[cfg(target_arch = "wasm32")]
    web::copy_to_clipboard(text);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = text;
}

/// Resolves after `ms` through the browser's `setTimeout`; with no renderer
/// attached (SSR, tests) it resolves at once.
pub(crate) async fn sleep(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    web::sleep(ms).await;
    #[cfg(not(target_arch = "wasm32"))]
    let _ = ms;
}

/// The current time in epoch milliseconds for time-based components.
///
/// `fixed` (a host-supplied clock) wins; otherwise the browser clock is read at
/// mount and every `tick_ms` after. Never anchor elapsed times to mount time: a
/// run that started before the component mounted must show its true age. With
/// no renderer (SSR, tests) and no `fixed`, this is 0.
pub(crate) fn use_now_ms(fixed: Option<u64>, tick_ms: u32) -> u64 {
    // Read at mount, not after the first tick: a clock that starts at 0 shows
    // every elapsed time as "0:00" until the first tick lands.
    let mut now = use_signal(|| fixed.unwrap_or_else(browser_now_ms));
    use_hook(move || {
        if fixed.is_none() && cfg!(target_arch = "wasm32") {
            spawn(async move {
                loop {
                    sleep(tick_ms).await;
                    now.set(browser_now_ms());
                }
            });
        }
    });
    fixed.unwrap_or_else(|| now())
}

/// Epoch milliseconds from the browser clock; 0 with no browser.
fn browser_now_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    return js_sys::Date::now() as u64;
    #[cfg(not(target_arch = "wasm32"))]
    0
}

/// Rendered width in CSS px of the element with `id`, kept current by a
/// `ResizeObserver` in the browser. `None` until the first measurement, which is
/// what SSR and tests see.
pub(crate) fn use_element_width(id: String) -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    return web::use_element_width(id);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        None
    }
}

/// Moves focus to the element with `id` after the next two animation frames.
///
/// Returning focus while the closing panel is still mounted (or before the
/// target has re-rendered) lands it on `<body>`. Use this for every "close and
/// return focus to the trigger" path.
pub(crate) fn focus_after_frame(id: &str) {
    #[cfg(target_arch = "wasm32")]
    web::focus_after_frame(id);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = id;
}

/// Focuses the element with `id` now, if it exists.
pub(crate) fn focus_id(id: &str) {
    #[cfg(target_arch = "wasm32")]
    web::focus_id(id);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = id;
}

/// Opens the `<details>` after the one whose `<summary>` holds focus, among the
/// summaries `selector` matches under `#root`, and moves focus to it; wraps
/// to the first.
pub(crate) fn focus_next_summary(root: &str, selector: &str) {
    #[cfg(target_arch = "wasm32")]
    web::focus_next_summary(root, selector);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (root, selector);
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::RefCell;
    use std::rc::Rc;

    use dioxus::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    fn document() -> Option<web_sys::Document> {
        web_sys::window()?.document()
    }

    fn html_element(id: &str) -> Option<web_sys::HtmlElement> {
        document()?.get_element_by_id(id)?.dyn_into().ok()
    }

    pub(super) fn copy_to_clipboard(text: &str) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let promise = window.navigator().clipboard().write_text(text);
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        });
    }

    pub(super) async fn sleep(ms: u32) {
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let scheduled = web_sys::window().and_then(|window| {
                window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &resolve,
                        i32::try_from(ms).unwrap_or(i32::MAX),
                    )
                    .ok()
            });
            if scheduled.is_none() {
                let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
            }
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }

    async fn next_frame() {
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let scheduled =
                web_sys::window().and_then(|window| window.request_animation_frame(&resolve).ok());
            if scheduled.is_none() {
                let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
            }
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }

    /// The observer and the closure it calls, kept alive together and dropped
    /// (disconnected first) when the component unmounts.
    struct Watch {
        observer: web_sys::ResizeObserver,
        _callback: Closure<dyn FnMut(js_sys::Array)>,
    }

    impl Drop for Watch {
        fn drop(&mut self) {
            self.observer.disconnect();
        }
    }

    pub(super) fn use_element_width(id: String) -> Option<f64> {
        let mut width = use_signal(|| None::<f64>);
        let watch = use_hook(|| Rc::new(RefCell::new(None::<Watch>)));
        let slot = watch.clone();
        use_hook(move || {
            spawn(async move {
                // The element is rendered after this hook runs: wait for it,
                // for at most 600 frames.
                let mut element = None;
                for _ in 0..600 {
                    element = document().and_then(|d| d.get_element_by_id(&id));
                    if element.is_some() {
                        break;
                    }
                    next_frame().await;
                }
                let Some(element) = element else {
                    return;
                };
                // The observer reports from outside the Dioxus runtime, so it
                // hands widths over a channel to this task, which sets the
                // signal from inside it.
                let (tx, mut rx) = futures_channel::mpsc::unbounded::<f64>();
                let measured = element.clone();
                let callback = Closure::<dyn FnMut(js_sys::Array)>::new(move |_entries| {
                    let _ = tx.unbounded_send(measured.get_bounding_client_rect().width());
                });
                let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref())
                else {
                    return;
                };
                observer.observe(&element);
                width.set(Some(element.get_bounding_client_rect().width()));
                *slot.borrow_mut() = Some(Watch {
                    observer,
                    _callback: callback,
                });
                while let Ok(value) = rx.recv().await {
                    width.set(Some(value));
                }
            });
        });
        use_drop(move || {
            watch.borrow_mut().take();
        });
        width()
    }

    pub(super) fn focus_after_frame(id: &str) {
        let id = id.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            next_frame().await;
            next_frame().await;
            if let Some(element) = html_element(&id) {
                let options = web_sys::FocusOptions::new();
                options.set_prevent_scroll(true);
                let _ = element.focus_with_options(&options);
            }
        });
    }

    pub(super) fn focus_id(id: &str) {
        if let Some(element) = html_element(id) {
            let _ = element.focus();
        }
    }

    pub(super) fn focus_next_summary(root: &str, selector: &str) {
        let Some(document) = document() else {
            return;
        };
        let Ok(list) = document.query_selector_all(&format!("#{root} {selector}")) else {
            return;
        };
        let summaries: Vec<web_sys::HtmlElement> = (0..list.length())
            .filter_map(|i| list.item(i)?.dyn_into().ok())
            .collect();
        if summaries.is_empty() {
            return;
        }
        let active = document.active_element();
        let at = summaries
            .iter()
            .position(|summary| active.as_ref() == Some(summary.unchecked_ref()));
        let next = &summaries[at.map_or(0, |at| (at + 1) % summaries.len())];
        if let Some(details) = next
            .parent_element()
            .and_then(|parent| parent.dyn_into::<web_sys::HtmlDetailsElement>().ok())
        {
            details.set_open(true);
        }
        let _ = next.focus();
    }

    /// A scroll container kept at its end while the reader stays within `slack`
    /// px of it. Dropping it disconnects the observer and removes the listener.
    struct Follow {
        el: web_sys::Element,
        observer: web_sys::ResizeObserver,
        on_scroll: Closure<dyn FnMut()>,
        _on_resize: Closure<dyn FnMut(js_sys::Array)>,
    }

    impl Follow {
        /// `None` when `el` already carries `marker`: a re-run effect must not
        /// add a second listener.
        fn start(
            el: web_sys::Element,
            observed: &[web_sys::Element],
            marker: &str,
            slack: i32,
        ) -> Option<Self> {
            if el.has_attribute(marker) {
                return None;
            }
            let _ = el.set_attribute(marker, "1");
            let at_end = Rc::new(std::cell::Cell::new(true));
            let on_scroll = {
                let (el, at_end) = (el.clone(), at_end.clone());
                Closure::<dyn FnMut()>::new(move || {
                    at_end.set(el.scroll_top() + el.client_height() >= el.scroll_height() - slack);
                })
            };
            let _ =
                el.add_event_listener_with_callback("scroll", on_scroll.as_ref().unchecked_ref());
            let on_resize = {
                let el = el.clone();
                Closure::<dyn FnMut(js_sys::Array)>::new(move |_entries| {
                    if at_end.get() {
                        el.set_scroll_top(el.scroll_height());
                    }
                })
            };
            let observer = web_sys::ResizeObserver::new(on_resize.as_ref().unchecked_ref()).ok()?;
            for target in observed {
                observer.observe(target);
            }
            Some(Self {
                el,
                observer,
                on_scroll,
                _on_resize: on_resize,
            })
        }
    }

    impl Drop for Follow {
        fn drop(&mut self) {
            self.observer.disconnect();
            let _ = self.el.remove_event_listener_with_callback(
                "scroll",
                self.on_scroll.as_ref().unchecked_ref(),
            );
        }
    }

    /// Lives as long as the page, as the eval'd script it replaces did.
    pub(super) fn follow_thread_end(thread_id: &str) {
        let Some(el) = document().and_then(|d| d.get_element_by_id(thread_id)) else {
            return;
        };
        let Some(stream) = el.first_element_child() else {
            return;
        };
        if let Some(follow) = Follow::start(el, &[stream], "data-follow", 24) {
            std::mem::forget(follow);
        }
    }

    pub(super) fn use_pin_to_end(id: String) {
        let pin = use_hook(|| Rc::new(RefCell::new(None::<Follow>)));
        let slot = pin.clone();
        use_effect(move || {
            let Some(el) = document().and_then(|d| d.get_element_by_id(&id)) else {
                return;
            };
            let mut observed = vec![el.clone()];
            observed.extend(el.first_element_child());
            if let Some(follow) = Follow::start(el.clone(), &observed, "data-pinned", 4) {
                el.set_scroll_top(el.scroll_height());
                *slot.borrow_mut() = Some(follow);
            }
        });
        use_drop(move || {
            pin.borrow_mut().take();
        });
    }

    pub(super) fn offset_width(selector: &str) -> Option<u32> {
        let width = document()
            .and_then(|d| d.query_selector(selector).ok().flatten())
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
            .map_or(0, |el| el.offset_width());
        Some(u32::try_from(width).unwrap_or(0))
    }

    /// The observer and its closure, disconnected when the awaiting task ends.
    struct Resizes {
        observer: web_sys::ResizeObserver,
        _callback: Closure<dyn FnMut(js_sys::Array)>,
    }

    impl Drop for Resizes {
        fn drop(&mut self) {
            self.observer.disconnect();
        }
    }

    pub(super) async fn each_resize(id: &str, mut on_resize: impl FnMut()) {
        const KEY: &str = "__kitFitObserver";
        let Some(el) = document().and_then(|d| d.get_element_by_id(id)) else {
            return;
        };
        // A remount observes again: the observer it left behind is disconnected first.
        if let Ok(previous) = js_sys::Reflect::get(&el, &KEY.into())
            && let Ok(previous) = previous.dyn_into::<web_sys::ResizeObserver>()
        {
            previous.disconnect();
        }
        let (tx, mut rx) = futures_channel::mpsc::unbounded::<()>();
        let callback = Closure::<dyn FnMut(js_sys::Array)>::new(move |_entries| {
            let _ = tx.unbounded_send(());
        });
        let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) else {
            return;
        };
        let _ = js_sys::Reflect::set(&el, &KEY.into(), &observer);
        observer.observe(&el);
        let _watch = Resizes {
            observer,
            _callback: callback,
        };
        while rx.recv().await.is_ok() {
            on_resize();
        }
    }
}

/// Keeps the scroll container `id` at its end while the reader is within 4 px
/// of it, following size changes of the container and its first child. Pins
/// once at mount; stops when the component unmounts.
pub(crate) fn use_pin_to_end(id: String) {
    #[cfg(target_arch = "wasm32")]
    web::use_pin_to_end(id);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = id;
}

/// `offsetWidth` of the first element `selector` matches, 0 when none does.
/// `None` with no browser (SSR, tests).
pub(crate) fn offset_width(selector: &str) -> Option<u32> {
    #[cfg(target_arch = "wasm32")]
    return web::offset_width(selector);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = selector;
        None
    }
}

/// Calls `on_resize` from the awaiting task each time the element `id` changes
/// size, until that task is dropped. Returns at once with no browser, or when
/// no element has `id`.
pub(crate) async fn each_resize(id: &str, on_resize: impl FnMut()) {
    #[cfg(target_arch = "wasm32")]
    web::each_resize(id, on_resize).await;
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (id, on_resize);
}

/// Keeps the scroll container `thread_id` pinned to its end while the user has not scrolled up. Its first child is
/// the stream whose size changes as charts draw, so the pin follows size changes rather than running once at mount.
pub(crate) fn follow_thread_end(thread_id: &str) {
    #[cfg(target_arch = "wasm32")]
    web::follow_thread_end(thread_id);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = thread_id;
}

/// Joins a component's base class with the caller's layout class.
pub(crate) fn class_list(base: &str, extra: &str) -> String {
    if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    }
}
