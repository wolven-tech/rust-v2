//! Time picker: a text field for a time of day, with a list of times every few
//! minutes in the top layer. Typing ("9", "930", "9:30 pm", "21.30") filters
//! the list; arrow keys and the pointer move the highlight. The value is
//! "HH:mm" (24 h), like a native time input.
//!
//! Port losses: the locale is not read. Times show as en-US (`9:30 AM` or
//! `09:30`), and only the English day-period words are typed-in. `Auto` hour
//! cycle means 12 h. The list's placement, the highlight glide, scroll
//! centring and the outside-press check run in the browser through
//! `document::eval`, and the current time comes back from the browser. `ref`
//! is not forwarded to the input. A controlled "no time" is `Some("")`.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};
use crate::segmented_control::js_string;

const CSS: Asset = asset!("/assets/time-picker.css");

const DAY: u32 = 24 * 60;
const AM_WORDS: &[&str] = &["a", "am"];
const PM_WORDS: &[&str] = &["p", "pm"];
const ZEROS: [u32; 18] = [
    0x660, 0x6f0, 0x966, 0x9e6, 0xa66, 0xae6, 0xb66, 0xbe6, 0xc66, 0xce6, 0xd66, 0xe50, 0xed0,
    0xf20, 0x1040, 0x17e0, 0x1810, 0xff10,
];

/// Attaches the browser side once per field. Sends `["now", "", minutes]` when
/// the field asks for the current time, and `["dismiss", "", 0]` on a press
/// outside the field and list.
const INIT: &str = r##"
const box = document.getElementById(__BOX__);
if (box && !box.__tpk) {
  const list = () => document.getElementById(__LIST__);
  const send = (kind, number) => {
    try {
      dioxus.send([kind, '', number]);
    } catch (_) {}
  };
  const place = (el, field) => {
    const r = field.getBoundingClientRect();
    const gap = 4;
    const edge = 8;
    const vw = document.documentElement.clientWidth || window.innerWidth;
    const vh = window.innerHeight;
    const width = Math.min(Math.max(r.width, 120), vw - 2 * edge);
    el.style.width = `${width}px`;
    const want = Math.min(el.scrollHeight, 264);
    const below = vh - r.bottom - gap - edge;
    const above = r.top - gap - edge;
    const up = below < Math.min(want, 160) && above > below;
    const room = Math.max(120, Math.min(264, up ? above : below));
    el.style.maxHeight = `${room}px`;
    const h = Math.min(want, room);
    el.style.left = `${Math.round(Math.min(Math.max(edge, r.left), vw - edge - width))}px`;
    el.style.top = `${Math.round(up ? r.top - gap - h : r.bottom + gap)}px`;
    el.toggleAttribute('data-up', up);
  };
  const ctl = (box.__tpk = {
    isOpen: false,
    update: null,
    now() {
      const d = new Date();
      send('now', d.getHours() * 60 + d.getMinutes());
    },
    open() {
      const el = list();
      if (!el) return;
      ctl.isOpen = true;
      if ('showPopover' in el && !el.matches(':popover-open')) el.showPopover();
      ctl.update = (e) => {
        if (e?.type === 'scroll' && e.target instanceof Node && el.contains(e.target)) return;
        place(el, box.querySelector('.ui-tpk__box') ?? box);
      };
      ctl.update();
      window.addEventListener('resize', ctl.update);
      window.addEventListener('scroll', ctl.update, true);
    },
    close() {
      ctl.isOpen = false;
      if (ctl.update) {
        window.removeEventListener('resize', ctl.update);
        window.removeEventListener('scroll', ctl.update, true);
        ctl.update = null;
      }
      const el = list();
      if (el && 'hidePopover' in el && el.matches(':popover-open')) el.hidePopover();
    },
    glow(index, center) {
      const el = list();
      const row = el?.querySelector(`[data-index="${index}"]`);
      if (!el || !row) return;
      const glow = el.querySelector('.ui-tpk__glow');
      if (glow) {
        glow.style.transform = `translateY(${row.offsetTop}px)`;
        glow.style.height = `${row.offsetHeight}px`;
      }
      if (center) el.scrollTop = row.offsetTop - (el.clientHeight - row.offsetHeight) / 2;
      else row.scrollIntoView({ block: 'nearest' });
    },
  });
  document.addEventListener(
    'pointerdown',
    (e) => {
      if (!ctl.isOpen) return;
      if (box.contains(e.target) || list()?.contains(e.target)) return;
      send('dismiss', 0);
    },
    true,
  );
}
"##;

const NOW: &str = "document.getElementById(__BOX__)?.__tpk?.now();";
const OPEN: &str = "document.getElementById(__BOX__)?.__tpk?.open();";
const CLOSE: &str = "document.getElementById(__BOX__)?.__tpk?.close();";
const GLOW: &str = "document.getElementById(__BOX__)?.__tpk?.glow(__INDEX__, __CENTER__);";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeSize {
    /// 32px high.
    Sm,
    /// 36px high.
    #[default]
    Md,
    /// 40px high.
    Lg,
}

impl TimeSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeHourCycle {
    /// 12 hours; the locale is not read.
    #[default]
    Auto,
    /// 12 hours: 9:30 AM.
    Twelve,
    /// 24 hours: 09:30.
    TwentyFour,
}

/// "HH:mm" (seconds allowed) → minutes after midnight, or `None`.
pub fn from_value(text: &str) -> Option<u32> {
    let text = text.trim();
    let mut parts = text.split(':');
    let hour = parts.next()?;
    let minute = parts.next()?;
    let seconds = parts.next();
    if parts.next().is_some() {
        return None;
    }
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !(1..=2).contains(&hour.len()) || !digits(hour) || minute.len() != 2 || !digits(minute) {
        return None;
    }
    if let Some(seconds) = seconds {
        let (sec, frac) = match seconds.split_once('.') {
            Some((sec, frac)) => (sec, Some(frac)),
            None => (seconds, None),
        };
        if sec.len() != 2 || !digits(sec) || frac.is_some_and(|f| !digits(f)) {
            return None;
        }
    }
    let h: u32 = hour.parse().ok()?;
    let m: u32 = minute.parse().ok()?;
    (h < 24 && m < 60).then_some(h * 60 + m)
}

/// Minutes after midnight → "HH:mm".
pub fn to_value(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// A time as shown in the field and the list: "09:30", or "9:30 AM" when `twelve`.
pub fn format_time(minutes: u32, twelve: bool) -> String {
    let (h, m) = (minutes / 60, minutes % 60);
    if !twelve {
        return format!("{h:02}:{m:02}");
    }
    let suffix = if h < 12 { "AM" } else { "PM" };
    let h12 = match h % 12 {
        0 => 12,
        x => x,
    };
    format!("{h12}:{m:02} {suffix}")
}

/// The min and max of the list in minutes. A missing or unreadable min is
/// midnight; a max before the min, or missing, is the end of the day.
pub fn bounds(min: Option<&str>, max: Option<&str>) -> (u32, u32) {
    let min_t = min.and_then(from_value).unwrap_or(0);
    let max_t = match max.and_then(from_value) {
        Some(m) if m >= min_t => m,
        _ => DAY - 1,
    };
    (min_t, max_t)
}

/// The step between list rows, kept between one minute and a day.
pub fn clamp_step(step: u32) -> u32 {
    step.clamp(1, DAY)
}

/// The list rows from `min` to `max` inclusive, every `step` minutes.
pub fn build_times(min: u32, max: u32, step: u32) -> Vec<u32> {
    (min..=max).step_by(clamp_step(step) as usize).collect()
}

/// The row a time falls on: the first at or after it, else the last. `None`
/// for an empty list.
pub fn nearest_index(times: &[u32], t: u32) -> Option<usize> {
    if times.is_empty() {
        return None;
    }
    Some(
        times
            .iter()
            .position(|&x| x >= t)
            .unwrap_or(times.len() - 1),
    )
}

/// Digits from the scripts people type times in, as ASCII. Other characters
/// are returned unchanged.
pub fn latin_digits(text: &str) -> String {
    text.chars()
        .map(|c| {
            let cp = c as u32;
            ZEROS
                .iter()
                .find(|&&zero| cp >= zero && cp <= zero + 9)
                .and_then(|&zero| char::from_digit(cp - zero, 10))
                .unwrap_or(c)
        })
        .collect()
}

/// Lowercased, with dots and whitespace removed: how a day-period word is compared.
pub fn norm(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| !matches!(c, '.' | '\u{a0}' | '\u{202f}') && !c.is_whitespace())
        .collect()
}

/// What people type as a time, in minutes after midnight. Accepts "9", "930",
/// "9:30", "09.30", "9h30", "9:30 pm", "9p", "21:30", "2130". A day-period
/// word goes before or after the digits; without one the hour is 24 h.
pub fn parse_time(text: &str, am: &[&str], pm: &[&str]) -> Option<u32> {
    let typed = latin_digits(text).trim().to_lowercase();
    if typed.is_empty() {
        return None;
    }
    let chars: Vec<char> = typed.chars().collect();
    let (pre, mut hour, minute, post) = split_time(&chars)?;
    let before = norm(&pre);
    let after = norm(&post);
    if !before.is_empty() && !after.is_empty() {
        return None;
    }
    let word = if before.is_empty() { after } else { before };
    let period = if word.is_empty() {
        None
    } else if am.contains(&word.as_str()) {
        Some(false)
    } else if pm.contains(&word.as_str()) {
        Some(true)
    } else {
        return None;
    };
    if minute > 59 {
        return None;
    }
    match period {
        Some(is_pm) => {
            if !(1..=12).contains(&hour) {
                return None;
            }
            hour = hour % 12 + if is_pm { 12 } else { 0 };
        }
        None if hour == 24 && minute == 0 => hour = 0,
        None if hour > 23 => return None,
        None => {}
    }
    Some(hour * 60 + minute)
}

/// Splits typed text into the text before the digits, the hour, the minute and
/// the text after. One or two digits take an optional `:`, `.` or `h` and two
/// minute digits; three or four digits are read as HHmm.
fn split_time(s: &[char]) -> Option<(String, u32, u32, String)> {
    let start = s.iter().position(char::is_ascii_digit)?;
    let end = start + s[start..].iter().take_while(|c| c.is_ascii_digit()).count();
    let number = |range: std::ops::Range<usize>| {
        s[range]
            .iter()
            .fold(0, |n, c| n * 10 + c.to_digit(10).unwrap_or(0))
    };
    let rest_has_no_digits = |from: usize| s[from..].iter().all(|c| !c.is_ascii_digit());
    let skip_ws = |mut i: usize| {
        while i < s.len() && s[i].is_whitespace() {
            i += 1;
        }
        i
    };
    let pre: String = s[..start].iter().collect();
    let tail = |from: usize| -> String { s[from..].iter().collect() };
    match end - start {
        1 | 2 => {
            let mut j = skip_ws(end);
            if j < s.len() && matches!(s[j], ':' | '.' | 'h') {
                j = skip_ws(j + 1);
                if j + 2 <= s.len()
                    && s[j].is_ascii_digit()
                    && s[j + 1].is_ascii_digit()
                    && rest_has_no_digits(j + 2)
                {
                    return Some((pre, number(start..end), number(j..j + 2), tail(j + 2)));
                }
            }
            rest_has_no_digits(end).then(|| (pre, number(start..end), 0, tail(end)))
        }
        3 | 4 if rest_has_no_digits(end) => {
            Some((pre, number(start..end - 2), number(end - 2..end), tail(end)))
        }
        _ => None,
    }
}

#[derive(Clone)]
struct TpSettings {
    times: Vec<u32>,
    value: Option<u32>,
    controlled: bool,
    disabled: bool,
    min: u32,
    max: u32,
    twelve: bool,
    on_change: Option<EventHandler<Option<String>>>,
}

fn with_ids(script: &str, box_id: &str, list_id: &str) -> String {
    script
        .replace("__BOX__", &js_string(box_id))
        .replace("__LIST__", &js_string(list_id))
}

/// Asks the browser for the current time; the answer opens the list.
fn request_now(box_id: &str) {
    document::eval(&NOW.replace("__BOX__", &js_string(box_id)));
}

/// A time-of-day field with a list of times. `value` is "HH:mm" (24 h) and
/// makes the field controlled; `Some("")` is a controlled empty field. `on_change`
/// receives "HH:mm", or `None` when the field is cleared.
#[component]
pub fn TimePicker(
    #[props(default)] value: Option<String>,
    #[props(default)] default_value: Option<String>,
    #[props(default)] on_change: Option<EventHandler<Option<String>>>,
    #[props(default = 15)] step: u32,
    #[props(default, into)] min: Option<String>,
    #[props(default, into)] max: Option<String>,
    #[props(default)] hour_cycle: TimeHourCycle,
    #[props(default, into)] placeholder: Option<String>,
    #[props(default)] label: Option<Element>,
    #[props(default)] hint: Option<Element>,
    #[props(default)] error: Option<Element>,
    #[props(default)] optional: bool,
    #[props(default)] size: TimeSize,
    #[props(default)] disabled: bool,
    #[props(default, into)] name: Option<String>,
    #[props(default)] required: bool,
    #[props(default, into)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default, into)] aria_label: Option<String>,
) -> Element {
    let uid = use_hook(|| format!("ui-tpk-{}", next_id()));
    let box_id = format!("{uid}-box");
    let list_id = format!("{uid}-list");
    let label_id = format!("{uid}-label");
    let message_id = format!("{uid}-message");
    let input_id = id.clone().unwrap_or_else(|| format!("{uid}-input"));
    let box_key = use_signal(|| box_id.clone());

    let twelve = hour_cycle != TimeHourCycle::TwentyFour;
    let step = clamp_step(step);
    let (min_t, max_t) = bounds(min.as_deref(), max.as_deref());
    let times = build_times(min_t, max_t, step);
    let controlled = value.is_some();

    let mut own = use_signal(|| default_value.as_deref().and_then(from_value));
    let value_now = if controlled {
        value.as_deref().and_then(from_value)
    } else {
        own()
    };
    let shown = value_now
        .map(|t| format_time(t, twelve))
        .unwrap_or_default();
    let placeholder = placeholder.unwrap_or_else(|| format_time(9 * 60, twelve));

    let mut text = use_signal(|| shown.clone());
    let mut editing = use_signal(|| false);
    let mut open = use_signal(|| false);
    let mut active = use_signal(|| None::<usize>);
    let mut navigated = use_signal(|| false);
    let mut center = use_signal(|| false);

    let field_text = if editing() { text() } else { shown.clone() };
    let invalid = error.is_some();
    let message = if error.is_some() {
        error.clone()
    } else {
        hint.clone()
    };
    let message_key = if invalid { "error" } else { "hint" };
    let has_message = message.is_some();

    let mut settings = use_signal(|| TpSettings {
        times: times.clone(),
        value: value_now,
        controlled,
        disabled,
        min: min_t,
        max: max_t,
        twelve,
        on_change,
    });
    use_effect(use_reactive(
        (
            &times,
            &value_now,
            &controlled,
            &disabled,
            &(min_t, max_t),
            &twelve,
        ),
        move |(times, value, controlled, disabled, (min, max), twelve)| {
            settings.set(TpSettings {
                times,
                value,
                controlled,
                disabled,
                min,
                max,
                twelve,
                on_change,
            });
        },
    ));

    let mut commit = move |next: Option<u32>| {
        let s = settings.peek().clone();
        if !s.controlled {
            own.set(next);
        }
        if next != s.value
            && let Some(handler) = s.on_change
        {
            handler.call(next.map(to_value));
        }
        editing.set(false);
    };
    let mut hide = move || {
        open.set(false);
    };
    let mut pick = move |index: usize| {
        let Some(&t) = settings.peek().times.get(index) else {
            return;
        };
        commit(Some(t));
        hide();
    };
    let mut settle = move || {
        if !*editing.peek() {
            return;
        }
        let s = settings.peek().clone();
        let typed = text.peek().clone();
        if typed.trim().is_empty() {
            commit(None);
            return;
        }
        match parse_time(&typed, AM_WORDS, PM_WORDS) {
            Some(t) if t >= s.min && t <= s.max => commit(Some(t)),
            _ => editing.set(false),
        }
    };
    let mut type_text = move |next: String| {
        text.set(next.clone());
        editing.set(true);
        navigated.set(false);
        let s = settings.peek().clone();
        if s.times.is_empty() {
            return;
        }
        let typed = if next.trim().is_empty() {
            None
        } else {
            parse_time(&next, AM_WORDS, PM_WORDS)
        };
        let query = norm(&latin_digits(&next));
        let hit = match typed {
            Some(t) => nearest_index(&s.times, t),
            None if !query.is_empty() => s
                .times
                .iter()
                .position(|&x| norm(&latin_digits(&format_time(x, s.twelve))).starts_with(&query)),
            None => None,
        };
        if !*open.peek() {
            center.set(true);
            open.set(true);
        }
        if let Some(hit) = hit {
            active.set(Some(hit));
        }
    };
    let mut open_now = move |minutes: u32| {
        let s = settings.peek().clone();
        if s.disabled || s.times.is_empty() {
            return;
        }
        let exact = s.value.and_then(|v| s.times.iter().position(|&x| x == v));
        let at = exact.or_else(|| nearest_index(&s.times, s.value.unwrap_or(minutes)));
        navigated.set(false);
        center.set(true);
        active.set(at);
        open.set(true);
    };
    let mut move_by = move |by: i64| {
        if !*open.peek() {
            request_now(box_key.peek().as_str());
            return;
        }
        navigated.set(true);
        let len = settings.peek().times.len() as i64;
        if len == 0 {
            return;
        }
        let from = active.peek().map_or(0, |a| a as i64);
        active.set(Some((from + by).clamp(0, len - 1) as usize));
    };

    let channel_box = box_id.clone();
    let channel_list = list_id.clone();
    use_effect(move || {
        let mut events = document::eval(&with_ids(INIT, &channel_box, &channel_list));
        spawn(async move {
            while let Ok((kind, _, number)) = events.recv::<(String, String, i64)>().await {
                match kind.as_str() {
                    "now" => open_now((number.max(0) as u32).min(DAY - 1)),
                    "dismiss" => open.set(false),
                    _ => {}
                }
            }
        });
    });

    let open_box = box_id.clone();
    let open_list = list_id.clone();
    use_effect(use_reactive((&open(),), move |(is_open,)| {
        let script = if is_open { OPEN } else { CLOSE };
        document::eval(&with_ids(script, &open_box, &open_list));
    }));

    let glow_box = box_id.clone();
    use_effect(use_reactive(
        (&active(), &open()),
        move |(index, is_open)| {
            let centered = *center.peek();
            if !is_open {
                return;
            }
            let Some(index) = index else {
                return;
            };
            if centered {
                center.set(false);
            }
            let script = GLOW
                .replace("__BOX__", &js_string(&glow_box))
                .replace("__INDEX__", &index.to_string())
                .replace("__CENTER__", if centered { "true" } else { "false" });
            document::eval(&script);
        },
    ));

    let focus_script = format!(
        "document.getElementById({})?.focus();",
        js_string(&input_id)
    );
    let toggle_focus = focus_script.clone();
    let key_box = box_key;
    let list_label = aria_label.clone().unwrap_or_else(|| "Times".to_string());
    let step_page = ((60.0 / step as f64).round() as i64).max(1);
    let label_for = input_id.clone();
    let value_text = value_now.map(to_value).unwrap_or_default();
    let show_list = open();
    let class = class_list("ui-tpk", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-size": size.as_str(),
            "data-open": show_list.then_some("true"),
            "data-invalid": invalid.then_some("true"),
            "data-disabled": disabled.then_some("true"),
            if let Some(label) = label.clone() {
                div { class: "ui-tpk__head",
                    label { id: "{label_id}", class: "ui-tpk__label", "for": "{label_for}", {label} }
                    if optional {
                        span { class: "ui-tpk__optional", "Optional" }
                    }
                }
            }
            div {
                id: "{box_id}",
                class: "ui-tpk__box",
                onclick: move |_| {
                    document::eval(&focus_script);
                },
                span { class: "ui-tpk__icon", aria_hidden: "true", ClockIcon {} }
                input {
                    id: "{input_id}",
                    class: "ui-tpk__input",
                    r#type: "text",
                    role: "combobox",
                    "inputmode": "text",
                    "autocomplete": "off",
                    "autocorrect": "off",
                    "spellcheck": "false",
                    "aria-autocomplete": "list",
                    "aria-expanded": show_list.to_string(),
                    "aria-controls": "{list_id}",
                    "aria-activedescendant": if show_list { active().map(|i| format!("{uid}-option-{i}")) } else { None },
                    aria_label,
                    "aria-describedby": has_message.then(|| message_id.clone()),
                    "aria-invalid": invalid.then_some("true"),
                    "aria-required": required.then_some("true"),
                    placeholder: "{placeholder}",
                    disabled,
                    value: "{field_text}",
                    oninput: move |event: FormEvent| type_text(event.value()),
                    onkeydown: move |event: KeyboardEvent| {
                        if disabled {
                            return;
                        }
                        let page = step_page;
                        match event.key() {
                            Key::ArrowDown => {
                                event.prevent_default();
                                if event.modifiers().alt() && !*open.peek() {
                                    request_now(key_box.peek().as_str());
                                } else {
                                    move_by(1);
                                }
                            }
                            Key::ArrowUp => {
                                event.prevent_default();
                                move_by(-1);
                            }
                            Key::PageDown => {
                                if !*open.peek() {
                                    return;
                                }
                                event.prevent_default();
                                move_by(page);
                            }
                            Key::PageUp => {
                                if !*open.peek() {
                                    return;
                                }
                                event.prevent_default();
                                move_by(-page);
                            }
                            Key::Enter => {
                                if *open.peek() && *navigated.peek() {
                                    if let Some(index) = *active.peek() {
                                        event.prevent_default();
                                        pick(index);
                                    }
                                } else if *editing.peek() || *open.peek() {
                                    event.prevent_default();
                                    settle();
                                    hide();
                                }
                            }
                            Key::Escape => {
                                if *open.peek() {
                                    event.prevent_default();
                                    event.stop_propagation();
                                    hide();
                                } else if *editing.peek() {
                                    event.prevent_default();
                                    editing.set(false);
                                }
                            }
                            Key::Tab => hide(),
                            _ => {}
                        }
                    },
                    onpointerdown: move |_| {
                        if !*open.peek() {
                            request_now(key_box.peek().as_str());
                        }
                    },
                    onblur: move |_| {
                        settle();
                        hide();
                    },
                }
                button {
                    r#type: "button",
                    class: "ui-tpk__toggle",
                    tabindex: "-1",
                    aria_label: if show_list { "Hide times" } else { "Show times" },
                    disabled,
                    onpointerdown: move |event: PointerEvent| event.prevent_default(),
                    onclick: move |event: MouseEvent| {
                        event.stop_propagation();
                        document::eval(&toggle_focus);
                        if show_list {
                            hide();
                        } else {
                            request_now(key_box.peek().as_str());
                        }
                    },
                    ChevronIcon {}
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name: "{name}", value: "{value_text}" }
            }
            if show_list {
                div {
                    id: "{list_id}",
                    class: "ui-tpk__list",
                    role: "listbox",
                    "aria-labelledby": label.is_some().then(|| label_id.clone()),
                    "aria-label": label.is_none().then(|| list_label.clone()),
                    popover: "manual",
                    span { class: "ui-tpk__glow", "aria-hidden": "true" }
                    for (i, t) in times.iter().copied().enumerate() {
                        {
                            let is_active = active() == Some(i);
                            let is_value = value_now == Some(t);
                            rsx! {
                                div {
                                    key: "{t}",
                                    id: "{uid}-option-{i}",
                                    class: "ui-tpk__option",
                                    role: "option",
                                    "data-index": "{i}",
                                    "aria-selected": is_value.to_string(),
                                    "data-active": is_active.then_some("true"),
                                    onpointermove: move |_| {
                                        navigated.set(true);
                                        if !is_active {
                                            active.set(Some(i));
                                        }
                                    },
                                    onpointerdown: move |event: PointerEvent| event.prevent_default(),
                                    onclick: move |_| pick(i),
                                    span { class: "ui-tpk__time", {format_time(t, twelve)} }
                                    span { class: "ui-tpk__check", aria_hidden: "true",
                                        if is_value {
                                            CheckIcon {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(message) = message {
                p {
                    key: "{message_key}",
                    id: "{message_id}",
                    class: "ui-tpk__message",
                    "aria-live": invalid.then_some("polite"),
                    {message}
                }
            }
        }
    }
}

#[component]
fn ClockIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "M12 8V12L14 14" }
        }
    }
}

#[component]
fn ChevronIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "14",
            height: "14",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M18 9C18 9 13.58 15 12 15C10.42 15 6 9 6 9" }
        }
    }
}

#[component]
fn CheckIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "14",
            height: "14",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { class: "ui-tpk__tick", d: "M5 14L8.5 17.5L19 6.5", "pathLength": "1" }
        }
    }
}
