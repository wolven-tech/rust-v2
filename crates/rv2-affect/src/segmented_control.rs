//! Segmented control: two to six options as separate chips (the default), as a
//! tray with a sliding pill, or as underline tabs. Arrow keys, Home and End move the selection; pressing the
//! track and dragging picks options as the pointer crosses them.
//!
//! Port losses: the pill, the measurements, the scroll fades and the pointer
//! tracking run in the browser through `document::eval`, so a pointer pick
//! reaches Rust one message later than in the original. The synchronous
//! "latest value" that stops repeated picks is refreshed after each render
//! rather than during it.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/segmented-control.css");

/// Installs the browser controller once per control and reports picks back
/// with `dioxus.send(value)`. The controller owns the pill, measurement,
/// scroll fades and pointer drag, which need synchronous DOM geometry.
const INIT: &str = r##"
const root = document.getElementById(__ID__);
if (root && !root.__uiSeg) {
  const scroller = root.querySelector(':scope > .ui-segc__scroller');
  const track = scroller.querySelector(':scope > .ui-segc__track');
  const pill = track.querySelector(':scope > .ui-segc__pill');
  const buttons = () => Array.from(track.querySelectorAll(':scope > .ui-segc__opt'));
  const activeButton = () => track.querySelector(':scope > .ui-segc__opt[data-active]');
  const optionAt = (clientX) => {
    let best = -1;
    let dist = Infinity;
    buttons().forEach((b, i) => {
      const r = b.getBoundingClientRect();
      const d = clientX < r.left ? r.left - clientX : clientX > r.right ? clientX - r.right : 0;
      if (d < dist) {
        dist = d;
        best = i;
      }
    });
    return best;
  };
  const pick = (i) => {
    const button = buttons()[i];
    const value = seg.values[i];
    if (!button || button.disabled || value === undefined) return;
    try {
      dioxus.send(value);
    } catch (_) {}
  };
  const endPress = () => {
    seg.press = null;
    root.removeAttribute('data-pressed');
  };
  const seg = (root.__uiSeg = {
    values: [],
    placedOnce: false,
    placed: null,
    gliding: false,
    glideTimer: 0,
    frame: 0,
    ro: null,
    press: null,
    measure() {
      const els = buttons();
      if (!els.length) return;
      const cs = getComputedStyle(els[0]);
      const pad = parseFloat(cs.paddingLeft) + parseFloat(cs.paddingRight);
      const widths = els.map((b) => b.firstElementChild.offsetWidth + pad);
      if (root.hasAttribute('data-full')) {
        track.style.setProperty('--ui-segc-min', `${Math.max(...widths)}px`);
        els.forEach((b) => (b.style.width = ''));
      } else els.forEach((b, i) => (b.style.width = `${widths[i]}px`));
    },
    place(animate) {
      const btn = activeButton();
      if (!btn || !btn.offsetWidth) {
        pill.style.opacity = '0';
        return;
      }
      let x = btn.offsetLeft;
      let w = btn.offsetWidth;
      if (root.dataset.variant === 'underline') {
        const content = btn.firstElementChild;
        x += content.offsetLeft;
        w = content.offsetWidth;
      }
      const instant = !pill.dataset.ready || (!animate && !seg.gliding);
      const prev = parseFloat(pill.style.left) || 0;
      if (!instant && x !== prev) pill.dataset.dir = x > prev ? 'right' : 'left';
      if (instant) pill.style.transition = 'none';
      pill.style.opacity = '1';
      pill.style.left = `${x}px`;
      pill.style.right = `${track.clientWidth - x - w}px`;
      if (instant) {
        pill.dataset.ready = '1';
        void pill.offsetWidth;
        pill.style.transition = '';
      }
    },
    fades() {
      const max = scroller.scrollWidth - scroller.clientWidth;
      root.toggleAttribute('data-overflow', max > 1);
      scroller.style.setProperty('--ui-segc-fade-l', scroller.scrollLeft > 1 ? '24px' : '0px');
      scroller.style.setProperty('--ui-segc-fade-r', scroller.scrollLeft < max - 1 ? '24px' : '0px');
    },
    reveal(smooth) {
      const btn = activeButton();
      if (!btn || scroller.scrollWidth <= scroller.clientWidth + 1) return;
      const left = track.offsetLeft + btn.offsetLeft - (scroller.clientWidth - btn.offsetWidth) / 2;
      scroller.scrollTo({ left: Math.max(0, left), behavior: smooth ? 'smooth' : 'auto' });
    },
    sync() {
      seg.measure();
      seg.place(false);
      seg.fades();
      if (!root.hasAttribute('data-ready')) requestAnimationFrame(() => root.setAttribute('data-ready', ''));
    },
    observe() {
      seg.ro?.disconnect();
      seg.ro = new ResizeObserver(() => {
        cancelAnimationFrame(seg.frame);
        seg.frame = requestAnimationFrame(() => seg.sync());
      });
      seg.ro.observe(scroller);
      seg.ro.observe(track);
      track.querySelectorAll('.ui-segc__opt, .ui-segc__content').forEach((el) => seg.ro.observe(el));
    },
    show(current) {
      if (seg.placedOnce && seg.placed === current) return;
      const first = !seg.placedOnce;
      seg.placedOnce = true;
      seg.placed = current;
      if (!first) {
        seg.gliding = true;
        clearTimeout(seg.glideTimer);
        seg.glideTimer = setTimeout(() => (seg.gliding = false), 240);
      }
      seg.place(!first);
      seg.reveal(!first);
      if (!first && track.contains(document.activeElement)) {
        const i = buttons().findIndex((b) => b.hasAttribute('data-active'));
        buttons()[i]?.focus({ preventScroll: true });
      }
    },
    focus(i) {
      buttons()[i]?.focus();
    },
  });

  root.addEventListener('error', (e) => {
    if (e.target instanceof Element && e.target.closest('.ui-segc__icon')) e.target.style.visibility = 'hidden';
  }, true);
  scroller.addEventListener('scroll', () => seg.fades());
  track.addEventListener('pointerdown', (e) => {
    if (root.hasAttribute('data-disabled') || e.button !== 0) return;
    const i = optionAt(e.clientX);
    if (i < 0 || buttons()[i].disabled) return;
    const touch = e.pointerType === 'touch';
    const drag = !touch || buttons()[i].hasAttribute('data-active');
    seg.press = { id: e.pointerId, touch, drag, x: e.clientX, moved: false };
    if (drag) root.setAttribute('data-pressed', '');
    if (!touch) {
      pick(i);
      try {
        track.setPointerCapture(e.pointerId);
      } catch (_) {}
    }
  });
  track.addEventListener('pointermove', (e) => {
    const p = seg.press;
    if (!p || p.id !== e.pointerId) return;
    if (!p.moved && Math.abs(e.clientX - p.x) < (p.touch ? 8 : 3)) return;
    p.moved = true;
    if (!p.drag) return;
    const i = optionAt(e.clientX);
    if (i >= 0) pick(i);
  });
  track.addEventListener('pointerup', (e) => {
    const p = seg.press;
    if (!p || p.id !== e.pointerId) return;
    if (p.touch && !p.moved) {
      const i = optionAt(e.clientX);
      if (i >= 0) pick(i);
    }
    endPress();
  });
  track.addEventListener('pointercancel', endPress);
  track.addEventListener('lostpointercapture', endPress);
}
"##;

/// Hands the option values to the controller and re-measures. Runs whenever the
/// option list, the variant, the size or the width mode changes.
const LAYOUT: &str = r##"
const seg = document.getElementById(__ID__)?.__uiSeg;
if (seg) {
  seg.values = __VALUES__;
  seg.sync();
  seg.observe();
}
"##;

/// Moves the pill to the current option; `null` when nothing is selected.
const SHOW: &str = "document.getElementById(__ID__)?.__uiSeg?.show(__CURRENT__);";

/// Moves focus to the option at `__INDEX__` (its position in the option list).
const FOCUS: &str = "document.getElementById(__ID__)?.__uiSeg?.focus(__INDEX__);";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SegmentSize {
    /// 26px high.
    Sm,
    /// 30px high.
    #[default]
    Md,
    /// 36px high.
    Lg,
}

impl SegmentSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SegmentVariant {
    /// Separate chips with a hairline; the active one is filled light gray.
    #[default]
    Buttons,
    /// A compact light-gray track with a white raised pill.
    Tray,
    /// Text tabs on a hairline with a sliding underline.
    Underline,
}

impl SegmentVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Buttons => "buttons",
            Self::Tray => "tray",
            Self::Underline => "underline",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SegmentKind {
    /// A radiogroup: arrow keys move in both directions.
    #[default]
    Radio,
    /// A tablist: left and right arrows only, and each tab controls a panel.
    Tabs,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SegmentCount {
    /// Formatted with thousands separators; NaN and infinities show nothing.
    Number(f64),
    /// Shown as given; an empty string shows nothing.
    Text(String),
}

#[derive(Clone, Default, PartialEq)]
pub struct SegmentOption {
    pub value: String,
    /// Shown as text. Without a label or icon the value is shown.
    pub label: Option<Element>,
    pub icon: Option<Element>,
    /// A small gray number after the label, e.g. unread items.
    pub count: Option<SegmentCount>,
    /// A small tinted tag after the label, e.g. "Save 20%".
    pub badge: Option<Element>,
    pub disabled: bool,
    /// Accessible name. Needed when the option shows only an icon.
    pub aria_label: Option<String>,
    /// With `SegmentKind::Tabs`: the id of the panel this tab shows.
    pub controls: Option<String>,
}

/// Drops options with a repeated value; the first one wins.
pub fn dedupe_options(options: Vec<SegmentOption>) -> Vec<SegmentOption> {
    let mut seen: Vec<String> = Vec::new();
    options
        .into_iter()
        .filter(|option| {
            if seen.contains(&option.value) {
                false
            } else {
                seen.push(option.value.clone());
                true
            }
        })
        .collect()
}

/// The value selected before anything is picked: `default_value` when given,
/// otherwise the first enabled option.
pub fn initial_value(options: &[SegmentOption], default_value: Option<&str>) -> Option<String> {
    default_value.map(str::to_owned).or_else(|| {
        options
            .iter()
            .find(|option| !option.disabled)
            .map(|option| option.value.clone())
    })
}

/// The value a key press moves to, or `None` when the key does nothing.
/// Only enabled options take part; an unknown `current` counts as before the
/// first one, so ArrowLeft from nowhere lands on the last one.
pub fn next_for_key(
    options: &[SegmentOption],
    current: Option<&str>,
    key: &str,
    tabs: bool,
) -> Option<String> {
    let enabled: Vec<&SegmentOption> = options.iter().filter(|option| !option.disabled).collect();
    if enabled.is_empty() {
        return None;
    }
    let len = enabled.len() as i64;
    let at = current
        .and_then(|value| enabled.iter().position(|option| option.value == value))
        .map_or(-1, |index| index as i64);
    let (prev, next): (&[&str], &[&str]) = if tabs {
        (&["ArrowLeft"], &["ArrowRight"])
    } else {
        (&["ArrowLeft", "ArrowUp"], &["ArrowRight", "ArrowDown"])
    };
    let index = if prev.contains(&key) {
        (at - 1).rem_euclid(len)
    } else if next.contains(&key) {
        (at + 1).rem_euclid(len)
    } else if key == "Home" {
        0
    } else if key == "End" {
        len - 1
    } else {
        return None;
    };
    Some(enabled[index as usize].value.clone())
}

/// The count as shown: `1284` becomes `"1,284"` (en-US grouping, at most three
/// decimals), and a non-finite number shows nothing.
pub fn count_text(count: &SegmentCount) -> Option<String> {
    let n = match count {
        SegmentCount::Text(text) => return Some(text.clone()),
        SegmentCount::Number(n) if !n.is_finite() => return None,
        SegmentCount::Number(n) => *n,
    };
    let fixed = format!("{:.3}", n.abs());
    let trimmed = if fixed.contains('.') {
        fixed.trim_end_matches('0').trim_end_matches('.')
    } else {
        fixed.as_str()
    };
    let (int, frac) = match trimmed.split_once('.') {
        Some((int, frac)) => (int, Some(frac)),
        None => (trimmed, None),
    };
    let mut grouped = String::with_capacity(int.len() + int.len() / 3);
    for (i, digit) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    let sign = if n < 0.0 { "-" } else { "" };
    Some(match frac {
        Some(frac) => format!("{sign}{grouped}.{frac}"),
        None => format!("{sign}{grouped}"),
    })
}

/// A double-quoted JavaScript string literal for `text`, for splicing into
/// `document::eval` source. Escapes quotes, backslashes and control characters.
pub fn js_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 || c == '\u{2028}' || c == '\u{2029}' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn with_id(script: &str, id: &str) -> String {
    script.replace("__ID__", &js_string(id))
}

/// A row of options, one of which is selected. Picks are reported through
/// `on_change`; without `value` the control keeps its own selection.
#[component]
pub fn SegmentedControl(
    options: Vec<SegmentOption>,
    #[props(default)] value: Option<String>,
    #[props(default)] default_value: Option<String>,
    #[props(default)] on_change: Option<EventHandler<String>>,
    #[props(default)] size: SegmentSize,
    #[props(default)] variant: SegmentVariant,
    #[props(default)] full_width: bool,
    #[props(default)] kind: SegmentKind,
    #[props(default)] disabled: bool,
    #[props(default, into)] aria_label: Option<String>,
    #[props(default, into)] aria_labelledby: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("ui-seg-{}", next_id()));
    let options = Rc::new(dedupe_options(options));
    let tabs = kind == SegmentKind::Tabs;
    let controlled = value.is_some();

    let mut inner = use_signal(|| initial_value(&options, default_value.as_deref()));
    let current: Option<String> = if controlled { value.clone() } else { inner() };
    // Written by the event handlers before the re-render, so a burst of picks
    // in one frame never reports the same value twice.
    let mut latest = use_signal(|| current.clone());

    let active_index = options
        .iter()
        .position(|option| Some(option.value.as_str()) == current.as_deref());
    let tabbable = active_index.or_else(|| options.iter().position(|option| !option.disabled));

    let mut select = move |next: String| {
        if latest.peek().as_deref() == Some(next.as_str()) {
            return;
        }
        latest.set(Some(next.clone()));
        if !controlled {
            inner.set(Some(next.clone()));
        }
        if let Some(handler) = on_change {
            handler.call(next);
        }
    };

    use_effect(use_reactive((&current,), move |(current,)| {
        latest.set(current)
    }));

    let channel_id = id.clone();
    use_effect(move || {
        let mut picks = document::eval(&with_id(INIT, &channel_id));
        spawn(async move {
            while let Ok(next) = picks.recv::<String>().await {
                select(next);
            }
        });
    });

    let values: Vec<String> = options.iter().map(|option| option.value.clone()).collect();
    let layout_id = id.clone();
    use_effect(use_reactive(
        (&values, &variant, &size, &full_width),
        move |(values, _, _, _)| {
            let list: Vec<String> = values.iter().map(|value| js_string(value)).collect();
            let script =
                with_id(LAYOUT, &layout_id).replace("__VALUES__", &format!("[{}]", list.join(",")));
            document::eval(&script);
        },
    ));

    let show_id = id.clone();
    use_effect(use_reactive((&current,), move |(current,)| {
        let arg = current.as_deref().map_or("null".to_string(), js_string);
        document::eval(&with_id(SHOW, &show_id).replace("__CURRENT__", &arg));
    }));

    let key_id = id.clone();
    let key_options = options.clone();
    let class = class_list("ui-segc", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            class,
            "data-size": size.as_str(),
            "data-variant": variant.as_str(),
            "data-full": full_width.then_some("true"),
            "data-disabled": disabled.then_some("true"),
            div { class: "ui-segc__scroller",
                div {
                    class: "ui-segc__track",
                    role: if tabs { "tablist" } else { "radiogroup" },
                    aria_label,
                    aria_labelledby,
                    aria_disabled: disabled.then_some("true"),
                    "aria-orientation": tabs.then_some("horizontal"),
                    onkeydown: move |event: KeyboardEvent| {
                        if disabled {
                            return;
                        }
                        let name = match event.key() {
                            Key::ArrowLeft => "ArrowLeft",
                            Key::ArrowRight => "ArrowRight",
                            Key::ArrowUp => "ArrowUp",
                            Key::ArrowDown => "ArrowDown",
                            Key::Home => "Home",
                            Key::End => "End",
                            _ => return,
                        };
                        let Some(target) = next_for_key(&key_options, latest.peek().as_deref(), name, tabs) else {
                            return;
                        };
                        event.prevent_default();
                        let index = key_options.iter().position(|option| option.value == target);
                        select(target);
                        if let Some(index) = index {
                            document::eval(&with_id(FOCUS, &key_id).replace("__INDEX__", &index.to_string()));
                        }
                    },
                    span { class: "ui-segc__pill", "aria-hidden": "true" }
                    for (index, option) in options.iter().enumerate() {
                        {
                            let value = option.value.clone();
                            let active = active_index == Some(index);
                            let label = match (&option.label, &option.icon) {
                                (Some(label), _) => Some(label.clone()),
                                (None, None) => Some(rsx! { "{value}" }),
                                (None, Some(_)) => None,
                            };
                            let count = option.count.as_ref().and_then(count_text).filter(|text| !text.is_empty());
                            rsx! {
                                button {
                                    key: "{value}",
                                    r#type: "button",
                                    class: "ui-segc__opt",
                                    role: if tabs { "tab" } else { "radio" },
                                    aria_checked: (!tabs).then(|| active.to_string()),
                                    aria_selected: tabs.then(|| active.to_string()),
                                    aria_controls: option.controls.clone().filter(|_| tabs),
                                    aria_label: option.aria_label.clone(),
                                    "data-active": active.then_some("true"),
                                    disabled: option.disabled || disabled,
                                    tabindex: if tabbable == Some(index) { "0" } else { "-1" },
                                    onclick: move |_| select(value.clone()),
                                    span { class: "ui-segc__content",
                                        if let Some(icon) = option.icon.clone() {
                                            span { class: "ui-segc__icon", {icon} }
                                        }
                                        if let Some(label) = label {
                                            span { class: "ui-segc__label", dir: "auto", {label} }
                                        }
                                        if let Some(text) = count {
                                            // Remounted when the count changes, which replays its entrance.
                                            for key in [text.clone()] {
                                                span { key: "{key}", class: "ui-segc__count", "{text}" }
                                            }
                                        }
                                        if let Some(badge) = option.badge.clone() {
                                            span { class: "ui-segc__badge", {badge} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
