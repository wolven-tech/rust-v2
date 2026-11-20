//! One-time code field: a row of boxes over one real input, so typing, paste,
//! SMS autofill and Backspace all work natively. The focus ring glides between
//! boxes, digits pop in, verifying shimmers, a wrong code shakes and clears,
//! and a right one folds the boxes into a "Verified" pill. A resend link counts
//! down below.
//!
//! Port losses: the caret, paste, the ring and the pill are measured in the
//! browser, which reports input and paste back through `document::eval`. Paste
//! text is read from the DOM rather than from Dioxus event data. The error
//! message is text only; the React original accepts any node. Unicode is folded
//! for digits and full-width Latin letters only, not full NFKC.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, prefers_motion_script, sleep};
use crate::segmented_control::js_string;

const CSS: Asset = asset!("/assets/otp-input.css");

const MAX_LENGTH: usize = 24;
const CHECK_PATH: &str = "M5.5 12.5l4 4L18.5 8";
const ALERT_PATH: &str = "M12 8v4.5M12 16h.01M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z";

/// Installs the browser side once per field. Raw input, paste and pointer
/// picks go back as `[kind, text, number]`; the ring, pill and caret are laid
/// out by `set`, which Rust calls after every change.
const INIT: &str = r##"
const root = document.getElementById(__ID__);
if (root && !root.__otp) {
  const input = document.getElementById(__INPUT__);
  const slots = document.getElementById(__SLOTS__);
  const ring = document.getElementById(__RING__);
  const pill = document.getElementById(__PILL__);
  const pillInner = document.getElementById(__PILL_INNER__);
  const boxes = () => Array.from(root.querySelectorAll('.ui-otp__box'));
  const send = (kind, text, number) => {
    try {
      dioxus.send([kind, text, number]);
    } catch (_) {}
  };
  const ctl = (root.__otp = {
    state: { code: '', idx: 0, length: 1, status: 'idle' },
    active() {
      const s = ctl.state;
      return Math.min(s.idx, s.code.length, s.length - 1);
    },
    ringTo(box) {
      ring.style.width = `${box.offsetWidth}px`;
      ring.style.height = `${box.offsetHeight}px`;
      ring.style.transform = `translate(${box.offsetLeft}px, ${box.offsetTop}px)`;
    },
    layout() {
      const box = boxes()[ctl.active()];
      if (box) {
        const first = !ring.dataset.ready;
        if (first) ring.style.transition = 'none';
        ctl.ringTo(box);
        if (first) {
          ring.dataset.ready = '1';
          void ring.offsetWidth;
          ring.style.transition = '';
        }
      }
      const w = ctl.state.status === 'success' ? pillInner.offsetWidth : slots.offsetWidth;
      pill.style.width = `${w}px`;
      let shift = 0;
      if (w > slots.offsetWidth) {
        const bounds = root.getBoundingClientRect();
        const own = slots.getBoundingClientRect();
        const left = own.left + own.width / 2 - w / 2;
        shift = Math.min(Math.max(left, bounds.left), bounds.right - w) - left;
      }
      pill.style.marginLeft = shift ? `${shift}px` : '';
    },
    resize() {
      const box = boxes()[ctl.active()];
      if (box) ctl.ringTo(box);
      if (ctl.state.status !== 'success') pill.style.width = `${slots.offsetWidth}px`;
    },
    selection() {
      if (document.activeElement !== input) return;
      const i = ctl.active();
      if (i < ctl.state.code.length) input.setSelectionRange(i, i + 1);
      else input.setSelectionRange(i, i);
    },
    set(next) {
      Object.assign(ctl.state, next);
      if (input.value !== ctl.state.code) input.value = ctl.state.code;
      ctl.layout();
      ctl.selection();
    },
    resync() {
      input.value = ctl.state.code;
      ctl.selection();
    },
  });

  new ResizeObserver(() => ctl.resize()).observe(slots);
  input.addEventListener('input', () => send('input', input.value, input.selectionStart ?? input.value.length));
  input.addEventListener('paste', (e) => {
    e.preventDefault();
    send('paste', e.clipboardData?.getData('text') ?? '', 0);
  });
  input.addEventListener('pointerup', (e) => {
    let i = boxes().findIndex((b) => e.clientX < b.getBoundingClientRect().right + 4);
    if (i < 0) i = ctl.state.length - 1;
    send('pick', '', i);
  });
  input.addEventListener('select', () => {
    const i = ctl.active();
    const want = i < ctl.state.code.length ? [i, i + 1] : [i, i];
    if (input.selectionStart !== want[0] || input.selectionEnd !== want[1]) ctl.selection();
  });
  input.addEventListener('focus', () => requestAnimationFrame(() => ctl.selection()));
  if (__AUTOFOCUS__) input.focus({ preventScroll: true });
}
"##;

/// Re-applies the stored code and caret, after input that changed nothing.
const RESYNC: &str = "document.getElementById(__ID__)?.__otp?.resync();";

/// The verifying shake, skipped for viewers who asked for reduced motion.
const SHAKE: &str = r##"
if (__MOTION__) {
  document.getElementById(__SLOTS__)?.animate(
    [{ transform: 'none' }, { transform: 'translateX(-6px)' }, { transform: 'translateX(5px)' }, { transform: 'translateX(-3px)' }, { transform: 'translateX(2px)' }, { transform: 'none' }],
    { duration: 420, easing: 'ease-out' },
  );
}
"##;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OtpStatus {
    #[default]
    Idle,
    Verifying,
    Error,
    Success,
}

impl OtpStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Verifying => "verifying",
            Self::Error => "error",
            Self::Success => "success",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OtpSeparator {
    #[default]
    None,
    /// A dash between the two halves.
    Middle,
    /// A dash after this many boxes.
    At(usize),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OtpAlign {
    #[default]
    Start,
    Center,
}

impl OtpAlign {
    fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
        }
    }
}

/// The box count the field draws: at least 1, at most 24.
pub fn clamp_length(length: usize) -> usize {
    length.clamp(1, MAX_LENGTH)
}

/// Folds the digit sets people type codes in and full-width Latin letters to
/// their ASCII forms. Everything else is returned unchanged.
pub fn fold_char(c: char) -> char {
    let cp = c as u32;
    let fold = |base: u32, zero: u32| char::from_u32(cp - base + zero).unwrap_or(c);
    match cp {
        0x660..=0x669 => fold(0x660, '0' as u32),
        0x6f0..=0x6f9 => fold(0x6f0, '0' as u32),
        0xff10..=0xff19 => fold(0xff10, '0' as u32),
        0xff21..=0xff3a => fold(0xff21, 'A' as u32),
        0xff41..=0xff5a => fold(0xff41, 'a' as u32),
        _ => c,
    }
}

/// Keeps what a code may contain: digits only when `numeric`, otherwise letters
/// and digits in capitals. Stops at `length` characters.
pub fn clean_code(text: &str, numeric: bool, length: usize) -> String {
    let mut out = String::new();
    let mut count = 0;
    for ch in text.chars().map(fold_char) {
        let kept = if numeric {
            ch.is_ascii_digit().then_some(ch)
        } else if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_uppercase())
        } else {
            None
        };
        if let Some(c) = kept.filter(|_| count < length) {
            out.push(c);
            count += 1;
        }
    }
    out
}

/// The box index after which a separator is drawn, if it falls inside the field.
pub fn separator_slot(separator: OtpSeparator, length: usize) -> Option<usize> {
    let slot = match separator {
        OtpSeparator::None => return None,
        OtpSeparator::Middle => length / 2,
        OtpSeparator::At(at) => at,
    };
    (slot > 0 && slot < length).then_some(slot)
}

/// The box the ring sits on: the typed length, or the last box when full.
pub fn active_index(index: usize, code_len: usize, length: usize) -> usize {
    index.min(code_len).min(length - 1)
}

/// Where the active box goes for an arrow or Home/End key, or `None` when the
/// key is not one of them. The result stays between the first box and the
/// last filled one (or the last box).
pub fn key_move(key: &str, active: usize, code_len: usize, length: usize) -> Option<usize> {
    let last = code_len.min(length - 1) as isize;
    let target = match key {
        "ArrowLeft" => active as isize - 1,
        "ArrowRight" => active as isize + 1,
        "Home" | "ArrowUp" => 0,
        "End" | "ArrowDown" => last,
        _ => return None,
    };
    Some(target.clamp(0, last) as usize)
}

/// The code and caret after typing. `None` when the input leaves the code as
/// it was. `caret` is the UTF-16 offset the browser reports.
pub fn input_result(
    code: &str,
    raw: &str,
    caret: usize,
    numeric: bool,
    length: usize,
) -> Option<(String, usize)> {
    let next = clean_code(raw, numeric, length);
    if next == code {
        return None;
    }
    let units: Vec<u16> = raw.encode_utf16().take(caret).collect();
    let before = String::from_utf16_lossy(&units);
    let at = clean_code(&before, numeric, length).chars().count();
    Some((next, at))
}

/// The code and caret after pasting over the boxes. A whole code replaces
/// everything; a part is written in from the active box on. `None` when the
/// paste holds no kept characters.
pub fn paste_result(
    code: &str,
    active: usize,
    pasted: &str,
    numeric: bool,
    length: usize,
) -> Option<(String, usize)> {
    let text = clean_code(pasted, numeric, length);
    let text_len = text.chars().count();
    if text_len == 0 {
        return None;
    }
    if text_len >= length {
        return Some((text.chars().take(length).collect(), length - 1));
    }
    let chars: Vec<char> = code.chars().collect();
    let head: String = chars.iter().take(active).collect();
    let tail: String = chars.iter().skip(active + text_len).collect();
    let next: String = format!("{head}{text}{tail}").chars().take(length).collect();
    Some((next, active + text_len))
}

/// Which boxes pop in when `after` replaces `before`, with the delay in ms for
/// each. A bulk change (a paste) staggers 45 ms per box from the first change.
pub fn pop_in_delays(before: &str, after: &str) -> Vec<(usize, u32)> {
    let before: Vec<char> = before.chars().collect();
    let after: Vec<char> = after.chars().collect();
    let changed: Vec<usize> = (0..after.len())
        .filter(|&i| before.get(i) != Some(&after[i]))
        .collect();
    let Some(&first) = changed.first() else {
        return Vec::new();
    };
    let bulk = changed.len() > 1;
    changed
        .into_iter()
        .map(|i| (i, if bulk { (i - first) as u32 * 45 } else { 0 }))
        .collect()
}

/// "0:24", "1:05:00", "3d 1h".
pub fn clock(seconds: u32) -> String {
    if seconds >= 86_400 {
        format!("{}d {}h", seconds / 86_400, (seconds % 86_400) / 3_600)
    } else if seconds >= 3_600 {
        format!(
            "{}:{:02}:{:02}",
            seconds / 3_600,
            (seconds % 3_600) / 60,
            seconds % 60
        )
    } else {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    }
}

/// Props the event handlers read. Written by an effect after each render, so a
/// handler sees the props of the last committed render.
#[derive(Clone)]
struct Settings {
    value: Option<String>,
    length: usize,
    numeric: bool,
    disabled: bool,
    busy: bool,
    on_change: Option<EventHandler<String>>,
    on_complete: Option<EventHandler<String>>,
}

fn code_of(settings: &Settings, inner: &str) -> String {
    clean_code(
        settings.value.as_deref().unwrap_or(inner),
        settings.numeric,
        settings.length,
    )
}

fn resync(id: &str) {
    document::eval(&RESYNC.replace("__ID__", &js_string(id)));
}

fn with_ids(script: &str, id: &str) -> String {
    script
        .replace("__ID__", &js_string(id))
        .replace("__INPUT__", &js_string(&format!("{id}-input")))
        .replace("__SLOTS__", &js_string(&format!("{id}-slots")))
        .replace("__RING__", &js_string(&format!("{id}-ring")))
        .replace("__PILL__", &js_string(&format!("{id}-pill")))
        .replace("__PILL_INNER__", &js_string(&format!("{id}-pill-inner")))
}

/// Verification code field. Typing, paste and autofill go through one hidden
/// input; `on_complete` fires once the last box is filled. `value` makes the
/// code controlled, `default_value` starts an uncontrolled one.
#[component]
pub fn OtpInput(
    #[props(default = 6)] length: usize,
    #[props(default)] separator: OtpSeparator,
    #[props(default = true)] numeric: bool,
    #[props(default)] value: Option<String>,
    #[props(default, into)] default_value: String,
    #[props(default)] on_change: Option<EventHandler<String>>,
    #[props(default)] on_complete: Option<EventHandler<String>>,
    #[props(default)] status: OtpStatus,
    #[props(default = "That code didn't work. Try again.".to_string(), into)] error_message: String,
    #[props(default = true)] clear_on_error: bool,
    #[props(default = "Verified".to_string(), into)] success_label: String,
    #[props(default)] on_resend: Option<EventHandler<()>>,
    #[props(default = 30)] resend_in: u32,
    #[props(default)] align: OtpAlign,
    #[props(default)] disabled: bool,
    #[props(default)] auto_focus: bool,
    #[props(default = "Verification code".to_string(), into)] label: String,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("ui-otp-{}", next_id()));
    let length = clamp_length(length);
    let sep = separator_slot(separator, length);
    let busy = matches!(status, OtpStatus::Verifying | OtpStatus::Success);

    let mut inner = use_signal(|| clean_code(&default_value, numeric, length));
    let mut idx = use_signal(|| 0usize);
    let mut focused = use_signal(|| false);
    let mut error_shown = use_signal(|| false);
    let mut clearing = use_signal(|| false);
    let mut locked = use_signal(|| false);
    let mut left = use_signal(|| resend_in);
    let mut announce = use_signal(String::new);
    let mut marks = use_signal(Vec::<(u64, u32)>::new);
    let mut mark_counter = use_signal(|| 0u64);
    let mut prev_code = use_signal(String::new);
    let mut prev_status = use_signal(|| status);
    let initial_value = value.clone();
    let mut settings = use_signal(move || Settings {
        value: initial_value,
        length,
        numeric,
        disabled,
        busy,
        on_change,
        on_complete,
    });

    let code = clean_code(value.as_deref().unwrap_or(&inner()), numeric, length);
    let code_len = code.chars().count();
    let active = active_index(idx(), code_len, length);
    let read_only = busy || locked() || disabled;
    let show_error = status == OtpStatus::Error && error_shown();
    let show_ring = !busy && !disabled && (focused() || (code_len > 0 && code_len < length));
    let caret_show = focused() && code.chars().nth(active).is_none() && !read_only;
    let msg_id = format!("{id}-msg");
    let noun = match (numeric, length == 1) {
        (true, true) => "digit",
        (true, false) => "digits",
        (false, true) => "character",
        (false, false) => "characters",
    };
    let aria = format!("{label}, {length} {noun}");

    use_effect(use_reactive(
        (&value, &length, &numeric, &disabled, &busy),
        move |(current_value, length, numeric, disabled, busy)| {
            settings.set(Settings {
                value: current_value,
                length,
                numeric,
                disabled,
                busy,
                on_change,
                on_complete,
            });
        },
    ));

    let mut commit = move |next: String, next_idx: usize, user: bool| {
        let s = settings.peek().clone();
        let current = code_of(&s, inner.peek().as_str());
        if s.value.is_none() {
            inner.set(next.clone());
        }
        if next != current
            && let Some(handler) = s.on_change
        {
            handler.call(next.clone());
        }
        let len = next.chars().count();
        idx.set(active_index(next_idx, len, s.length));
        if !user {
            return;
        }
        if *error_shown.peek() && !next.is_empty() {
            error_shown.set(false);
        }
        if len == s.length
            && next != current
            && let Some(handler) = s.on_complete
        {
            handler.call(next);
        }
    };

    let channel_id = id.clone();
    use_effect(move || {
        let mut events = document::eval(
            &with_ids(INIT, &channel_id)
                .replace("__AUTOFOCUS__", if auto_focus { "true" } else { "false" }),
        );
        let resync_id = channel_id.clone();
        spawn(async move {
            while let Ok((kind, text, number)) = events.recv::<(String, String, i64)>().await {
                let s = settings.peek().clone();
                let readonly = s.disabled || s.busy || *locked.peek();
                let current = code_of(&s, inner.peek().as_str());
                let current_len = current.chars().count();
                let at = active_index(*idx.peek(), current_len, s.length);
                match kind.as_str() {
                    "input" if readonly => resync(&resync_id),
                    "input" => match input_result(
                        &current,
                        &text,
                        number.max(0) as usize,
                        s.numeric,
                        s.length,
                    ) {
                        Some((next, caret)) => commit(next, caret, true),
                        None => resync(&resync_id),
                    },
                    "paste" if !readonly => {
                        if let Some((next, caret)) =
                            paste_result(&current, at, &text, s.numeric, s.length)
                        {
                            commit(next, caret, true);
                        }
                    }
                    "pick" => {
                        idx.set(active_index(number.max(0) as usize, current_len, s.length));
                        resync(&resync_id);
                    }
                    _ => {}
                }
            }
        });
    });

    let push = format!(
        "document.getElementById({id})?.__otp?.set({{ code: {code}, idx: {index}, length: {length}, status: {status} }});",
        id = js_string(&id),
        code = js_string(&code),
        index = idx(),
        status = js_string(status.as_str()),
    );
    use_effect(use_reactive((&push,), move |(push,)| {
        document::eval(&push);
    }));

    let install_id = id.clone();
    let status_success = success_label.clone();
    let status_error = error_message.clone();
    use_effect(use_reactive((&status,), move |(status,)| {
        if *prev_status.peek() == status {
            return;
        }
        prev_status.set(status);
        match status {
            OtpStatus::Verifying => announce.set("Verifying code".to_string()),
            OtpStatus::Success => announce.set(status_success.clone()),
            _ => {}
        }
        if status != OtpStatus::Error {
            error_shown.set(false);
            return;
        }
        error_shown.set(true);
        announce.set(status_error.clone());
        let shake = SHAKE
            .replace("__MOTION__", prefers_motion_script())
            .replace("__SLOTS__", &js_string(&format!("{install_id}-slots")));
        document::eval(&shake);
        if !clear_on_error {
            return;
        }
        locked.set(true);
        spawn(async move {
            sleep(700).await;
            clearing.set(true);
            sleep(280).await;
            clearing.set(false);
            locked.set(false);
            commit(String::new(), 0, false);
        });
    }));

    use_effect(use_reactive((&code,), move |(code,)| {
        let before = prev_code.peek().clone();
        if before == code {
            return;
        }
        let changed = pop_in_delays(&before, &code);
        let mut next_marks = marks.peek().clone();
        for (i, delay) in changed {
            if next_marks.len() <= i {
                next_marks.resize(i + 1, (0, 0));
            }
            let stamp = *mark_counter.peek() + 1;
            mark_counter.set(stamp);
            next_marks[i] = (stamp, delay);
        }
        marks.set(next_marks);
        prev_code.set(code);
    }));

    let left_now = left();
    use_effect(use_reactive((&left_now,), move |(remaining,)| {
        if remaining == 0 {
            if on_resend.is_some() && resend_in > 0 {
                announce.set("You can resend the code now".to_string());
            }
            return;
        }
        spawn(async move {
            sleep(1000).await;
            let next = left.peek().saturating_sub(1);
            left.set(next);
        });
    }));

    let cells_marks = marks();
    let mut cells: Vec<Element> = Vec::with_capacity(length + 1);
    for i in 0..length {
        if sep == Some(i) {
            cells.push(rsx! { span { key: "sep-{i}", class: "ui-otp__sep" } });
        }
        let ch = code.chars().nth(i);
        let (stamp, delay) = cells_marks.get(i).copied().unwrap_or((0, 0));
        cells.push(rsx! {
            span {
                key: "{i}",
                class: "ui-otp__box",
                "data-filled": ch.is_some().then_some("true"),
                style: "--i: {i};",
                if let Some(ch) = ch {
                    span { key: "{stamp}", class: "ui-otp__char", style: "animation-delay: {delay}ms;", "{ch}" }
                }
            }
        });
    }

    let class = class_list("ui-otp", &class);
    let input_id = format!("{id}-input");
    let slots_style = format!(
        "--n: {length}; --sep-w: {};",
        if sep.is_some() {
            "calc(var(--ui-otp-gap) + 10px)"
        } else {
            "0px"
        }
    );
    let ring_id = format!("{id}-ring");
    let pill_id = format!("{id}-pill");
    let pill_inner_id = format!("{id}-pill-inner");
    let slots_id = format!("{id}-slots");

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            class,
            "data-status": status.as_str(),
            "data-error": show_error.then_some("true"),
            "data-align": align.as_str(),
            "data-disabled": disabled.then_some("true"),
            div { class: "ui-otp__field",
                div {
                    id: "{slots_id}",
                    class: "ui-otp__slots",
                    dir: "ltr",
                    "data-clearing": clearing().then_some("true"),
                    style: "{slots_style}",
                    for cell in cells {
                        {cell}
                    }
                    span {
                        id: "{ring_id}",
                        class: "ui-otp__ring",
                        "data-show": show_ring.then_some("true"),
                        "aria-hidden": "true",
                        span { class: "ui-otp__caret", "data-show": caret_show.then_some("true") }
                    }
                    span { id: "{pill_id}", class: "ui-otp__pill", "aria-hidden": "true",
                        span { id: "{pill_inner_id}", class: "ui-otp__pill-inner",
                            svg { view_box: "0 0 24 24", class: "ui-otp__check", "aria-hidden": "true",
                                path { d: CHECK_PATH, "pathLength": "1" }
                            }
                            span { class: "ui-otp__pill-text", title: "{success_label}", "{success_label}" }
                        }
                    }
                    input {
                        id: "{input_id}",
                        class: "ui-otp__input",
                        r#type: "text",
                        value: "{code}",
                        "inputmode": if numeric { "numeric" } else { "text" },
                        "pattern": numeric.then_some("[0-9]*"),
                        "autocomplete": "one-time-code",
                        "autocapitalize": if numeric { "off" } else { "characters" },
                        "autocorrect": "off",
                        "spellcheck": "false",
                        "enterkeyhint": "done",
                        aria_label: "{aria}",
                        "aria-invalid": show_error.then_some("true"),
                        "aria-describedby": show_error.then_some(msg_id.clone()),
                        disabled,
                        "readonly": (busy || locked()).then_some("true"),
                        onkeydown: move |event: KeyboardEvent| {
                            let name = match event.key() {
                                Key::ArrowLeft => "ArrowLeft",
                                Key::ArrowRight => "ArrowRight",
                                Key::ArrowUp => "ArrowUp",
                                Key::ArrowDown => "ArrowDown",
                                Key::Home => "Home",
                                Key::End => "End",
                                _ => return,
                            };
                            if event.modifiers().shift() || event.modifiers().meta() {
                                return;
                            }
                            if let Some(next) = key_move(name, active, code_len, length) {
                                event.prevent_default();
                                idx.set(next);
                            }
                        },
                        onfocus: move |_| focused.set(true),
                        onblur: move |_| focused.set(false),
                    }
                }
            }
            div { class: "ui-otp__msg", "data-show": show_error.then_some("true"),
                div { class: "ui-otp__msg-inner",
                    p { id: "{msg_id}", class: "ui-otp__msg-text",
                        svg { view_box: "0 0 24 24", class: "ui-otp__alert", "aria-hidden": "true",
                            path { d: ALERT_PATH }
                        }
                        span { "{error_message}" }
                    }
                }
            }
            if let Some(on_resend) = on_resend {
                div { class: "ui-otp__resend", "data-hidden": (status == OtpStatus::Success).then_some("true"),
                    span { class: "ui-otp__resend-q", "Didn't get a code?" }
                    " "
                    if left_now > 0 {
                        span { class: "ui-otp__swap ui-otp__wait",
                            "Resend in\u{a0}"
                            span { class: "ui-otp__time", "{clock(left_now)}" }
                        }
                    } else {
                        button {
                            r#type: "button",
                            class: "ui-otp__swap ui-otp__resend-btn",
                            disabled: disabled || busy,
                            onclick: move |_| {
                                on_resend.call(());
                                left.set(resend_in);
                                announce.set("A new code is on its way".to_string());
                            },
                            "Resend code"
                        }
                    }
                }
            }
            span { class: "ui-otp__live", "aria-live": "polite", "{announce}" }
        }
    }
}
