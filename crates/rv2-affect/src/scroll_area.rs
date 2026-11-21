//! Scroll area: a box that scrolls the browser's own way, with a slim scrollbar drawn over its content. The bar shows
//! while you hover or scroll, its thumb can be dragged, and a press on the track pages towards it.
//!
//! Port losses: the scrollbar is driven by script (`document::eval`) on the rendered elements: the thumb geometry,
//! the drag and the fades are not re-rendered by Dioxus. `onScroll` and `ref` are not forwarded; listen on the
//! viewport element (`.ui-sa__viewport`) from `document::eval` if needed.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/scroll-area.css");

/// Binds the scrollbar of the box with id `__ROOT__` once: the thumb follows the viewport, the thumb and track take
/// presses, and the bar shows for a moment after each scroll. Each element is bound once, so a re-run only binds bars
/// that appeared since.
const BIND: &str = r#"
const root = document.getElementById("__ROOT__");
const view = root && root.querySelector(":scope > .ui-sa__viewport");
const content = view && view.firstElementChild;
if (root && view && content) {
  if (!view.dataset.uiSaBound) {
    view.dataset.uiSaBound = "1";
    let idle = 0;
    const bar = (axis) => root.querySelector(`.ui-sa__bar[data-axis="${axis}"]`);
    const update = () => {
      const cs = getComputedStyle(root);
      const rtl = cs.direction === "rtl";
      const cap = parseFloat(cs.getPropertyValue("--ui-sa-fade")) || 28;
      const has = {
        y: !!bar("y") && view.scrollHeight - view.clientHeight > 1,
        x: !!bar("x") && view.scrollWidth - view.clientWidth > 1,
      };
      for (const axis of ["y", "x"]) {
        root.toggleAttribute(`data-scroll-${axis}`, has[axis]);
        const track = bar(axis);
        const thumb = track && track.firstElementChild;
        if (has[axis] && thumb) {
          const size = axis === "y" ? view.clientHeight : view.clientWidth;
          const full = axis === "y" ? view.scrollHeight : view.scrollWidth;
          const pos = Math.abs(axis === "y" ? view.scrollTop : view.scrollLeft);
          const length = axis === "y" ? track.clientHeight : track.clientWidth;
          const len = Math.max(Math.min(24, length), Math.round((length * size) / full));
          const at = full - size > 0 ? ((length - len) * Math.min(pos, full - size)) / (full - size) : 0;
          if (axis === "y") {
            thumb.style.height = `${len}px`;
            thumb.style.transform = `translate3d(0, ${at}px, 0)`;
          } else {
            thumb.style.width = `${len}px`;
            thumb.style.transform = `translate3d(${rtl ? -at : at}px, 0, 0)`;
          }
          const end = full - size - pos;
          root.style.setProperty(`--ui-sa-${axis}-start`, `${Math.min(pos, cap)}px`);
          root.style.setProperty(`--ui-sa-${axis}-end`, `${Math.min(Math.max(end, 0), cap)}px`);
        } else {
          root.style.setProperty(`--ui-sa-${axis}-start`, "0px");
          root.style.setProperty(`--ui-sa-${axis}-end`, "0px");
        }
      }
      if (has.x || has.y) view.tabIndex = 0;
      else view.removeAttribute("tabindex");
    };
    view.addEventListener("scroll", () => {
      update();
      root.setAttribute("data-scrolling", "");
      clearTimeout(idle);
      idle = setTimeout(() => root.removeAttribute("data-scrolling"), 800);
    }, { passive: true });
    if (typeof ResizeObserver !== "undefined") {
      const ro = new ResizeObserver(() => update());
      ro.observe(view);
      ro.observe(content);
    }
    update();
  }
  for (const axis of ["y", "x"]) {
    const track = root.querySelector(`.ui-sa__bar[data-axis="${axis}"]`);
    if (!track || track.dataset.uiSaBound) continue;
    track.dataset.uiSaBound = "1";
    const thumb = track.firstElementChild;
    let drag = null;
    const drop = () => {
      drag = null;
      root.removeAttribute("data-dragging");
    };
    thumb.addEventListener("pointerdown", (e) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      const rect = track.getBoundingClientRect();
      const scale = axis === "y" ? rect.height / (track.offsetHeight || 1) : rect.width / (track.offsetWidth || 1);
      const length = axis === "y" ? track.clientHeight : track.clientWidth;
      const len = axis === "y" ? thumb.offsetHeight : thumb.offsetWidth;
      const room = axis === "y" ? view.scrollHeight - view.clientHeight : view.scrollWidth - view.clientWidth;
      drag = {
        axis,
        from: axis === "y" ? e.clientY : e.clientX,
        start: axis === "y" ? view.scrollTop : view.scrollLeft,
        per: room / Math.max(1, length - len) / (scale || 1),
      };
      thumb.setPointerCapture(e.pointerId);
      root.setAttribute("data-dragging", "");
    });
    thumb.addEventListener("pointermove", (e) => {
      if (!drag) return;
      const by = ((drag.axis === "y" ? e.clientY : e.clientX) - drag.from) * drag.per;
      if (drag.axis === "y") view.scrollTop = drag.start + by;
      else view.scrollLeft = drag.start + by;
    });
    thumb.addEventListener("pointerup", drop);
    thumb.addEventListener("pointercancel", drop);
    thumb.addEventListener("lostpointercapture", drop);
    track.addEventListener("pointerdown", (e) => {
      if (e.button !== 0 || e.target === thumb) return;
      e.preventDefault();
      const tr = thumb.getBoundingClientRect();
      const rtl = getComputedStyle(view).direction === "rtl";
      const before = axis === "y" ? e.clientY < tr.top : rtl ? e.clientX > tr.right : e.clientX < tr.left;
      const step = (axis === "y" ? view.clientHeight : view.clientWidth) * 0.9 * (before ? -1 : 1);
      const smooth = !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
      view.scrollBy({
        [axis === "y" ? "top" : "left"]: step * (axis === "x" && rtl ? -1 : 1),
        behavior: smooth ? "smooth" : "auto",
      });
    });
  }
}
"#;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollOrientation {
    #[default]
    Vertical,
    Horizontal,
    Both,
}

impl ScrollOrientation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
            Self::Both => "both",
        }
    }
}

/// When the scrollbar shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    /// While the pointer is over the box, or while it scrolls.
    #[default]
    Hover,
    /// Only while it scrolls.
    Scroll,
    /// Whenever there is more to scroll.
    Always,
}

impl ScrollbarVisibility {
    fn as_str(self) -> &'static str {
        match self {
            Self::Hover => "hover",
            Self::Scroll => "scroll",
            Self::Always => "always",
        }
    }
}

/// A box that scrolls with a slim bar of its own over the content. Give it a height with `class`/`style`, or a
/// `max_height` to grow until it scrolls.
#[component]
pub fn ScrollArea(
    #[props(default)] orientation: ScrollOrientation,
    #[props(default)] visibility: ScrollbarVisibility,
    #[props(default)] fade: bool,
    #[props(default, into)] max_height: Option<String>,
    #[props(default, into)] aria_label: Option<String>,
    #[props(default, into)] aria_labelledby: Option<String>,
    #[props(default)] class: String,
    #[props(default)] style: String,
    children: Element,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let uid = use_hook(|| format!("ui-sa-{}", next_id()));
    let script = BIND.replace("__ROOT__", &uid);
    use_effect(use_reactive((&orientation,), move |_| {
        document::eval(&script);
    }));

    let wants_y = orientation != ScrollOrientation::Horizontal;
    let wants_x = orientation != ScrollOrientation::Vertical;
    let has_name = aria_label.is_some() || aria_labelledby.is_some();
    let style = match &max_height {
        Some(height) => format!("max-height: {height}; {style}"),
        None => style,
    };
    let class = class_list("ui-sa", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{uid}",
            class,
            "data-orientation": orientation.as_str(),
            "data-type": visibility.as_str(),
            "data-fade": fade.then_some(""),
            style: "{style}",
            ..attributes,
            div {
                class: "ui-sa__viewport",
                role: has_name.then_some("region"),
                aria_label,
                aria_labelledby,
                div { class: "ui-sa__content", {children} }
            }
            if wants_y {
                {bar("y")}
            }
            if wants_x {
                {bar("x")}
            }
        }
    }
}

fn bar(axis: &'static str) -> Element {
    rsx! {
        div { class: "ui-sa__bar", "data-axis": axis, "aria-hidden": "true",
            div { class: "ui-sa__thumb" }
        }
    }
}
