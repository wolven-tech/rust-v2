//! Number input: one number in a field, with − and + buttons (hold to repeat),
//! the arrow keys, Page Up/Down for large steps, and Home/End for the limits.
//! A unit sits after the value. The value is clamped to `min`..`max` and set
//! on the step grid when typing ends (blur or Enter).
//!
//! Port losses: `locale`, `formatOptions` and percent style are not ported.
//! Values are shown with `,` grouping and at most `max_fraction_digits`
//! decimals, and `unit` is text. Keyboard activation of − or + with Enter or
//! Space does nothing; pointer press steps, and the arrow keys step.
//! Controlled empty is `f64::NAN`. Hold-to-repeat keeps firing at the limit
//! until released (each press there is a no-op). The roll animation runs
//! through `document::eval`.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/number-input.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NumberInputSize {
    /// 32px high.
    Sm,
    /// 36px high.
    #[default]
    Md,
    /// 40px high.
    Lg,
}

impl NumberInputSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

/// What the typed text reads as.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reading {
    Empty,
    Invalid,
    Value(f64),
}

/// Reads typed text: `,` groups, `.` is the decimal, a leading or trailing
/// minus or parentheses make it negative, and letters are dropped.
pub fn read_value(text: &str) -> Reading {
    let mut t: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    t = t.replace(',', "");
    if t.is_empty() {
        return Reading::Empty;
    }
    let negative = t.starts_with(['-', '−'])
        || t.ends_with(['-', '−'])
        || (t.starts_with('(') && t.ends_with(')'));
    let digits: String = t
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if digits.is_empty() || digits == "." || digits.matches('.').count() > 1 {
        return Reading::Invalid;
    }
    match digits.parse::<f64>() {
        Ok(n) => Reading::Value(if negative { -n } else { n }),
        Err(_) => Reading::Invalid,
    }
}

/// `1234.5` with at most `max_fraction_digits` decimals and `,` grouping.
pub fn format_value(value: f64, max_fraction_digits: usize) -> String {
    let fixed = format!("{:.*}", max_fraction_digits, value);
    let (sign, rest) = match fixed.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", fixed.as_str()),
    };
    let (int_part, frac_part) = match rest.split_once('.') {
        Some((int, frac)) => (int, frac.trim_end_matches('0')),
        None => (rest, ""),
    };
    let mut grouped = String::new();
    for (i, ch) in int_part.chars().enumerate() {
        if i > 0 && (int_part.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let zero = grouped == "0" && frac_part.is_empty();
    let sign = if zero { "" } else { sign };
    if frac_part.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped}.{frac_part}")
    }
}

/// The decimals a step has, at least 10 places, so float noise rounds away.
pub fn round_to_step(value: f64, step: f64) -> f64 {
    let decimals = format!("{step}")
        .split_once('.')
        .map(|(_, frac)| frac.len())
        .unwrap_or(0);
    let places = decimals.clamp(10, 15);
    format!("{value:.places$}").parse().unwrap_or(value)
}

/// Where one step from `base` lands, on the grid that starts at `origin`.
pub fn next_on_grid(base: f64, origin: f64, step: f64, by: f64, dir: f64) -> f64 {
    if by != step {
        return origin + ((base + dir * by - origin) / step + 0.5).floor() * step;
    }
    let k = (base - origin) / by;
    let snapped = if dir > 0.0 {
        (k + 1e-9).floor() + 1.0
    } else {
        (k - 1e-9).ceil() - 1.0
    };
    origin + snapped * by
}

#[component]
pub fn NumberInput(
    #[props(default, into)] id: Option<String>,
    /// `Some(NAN)` is an empty controlled field.
    #[props(default)]
    value: Option<f64>,
    #[props(default)] default_value: Option<f64>,
    #[props(default)] on_value_change: Option<EventHandler<Option<f64>>>,
    #[props(default)] onchange: Option<EventHandler<FormEvent>>,
    #[props(default)] oninput: Option<EventHandler<FormEvent>>,
    #[props(default)] onfocus: Option<EventHandler<FocusEvent>>,
    #[props(default)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(default)] min: Option<f64>,
    #[props(default)] max: Option<f64>,
    #[props(default = 1.0)] step: f64,
    #[props(default)] large_step: Option<f64>,
    #[props(default = 3)] max_fraction_digits: usize,
    #[props(default)] unit: Option<Element>,
    #[props(default)] prefix: Option<Element>,
    #[props(default)] label: Option<Element>,
    #[props(default)] hint: Option<Element>,
    #[props(default)] error: Option<Element>,
    #[props(default)] optional: bool,
    #[props(default)] size: NumberInputSize,
    #[props(default, into)] name: String,
    #[props(default, into)] form: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] read_only: bool,
    #[props(default, into)] placeholder: Option<String>,
    #[props(default)] class: String,
    #[props(default)] input_class: String,
    #[props(extends = GlobalAttributes, extends = input)] attributes: Vec<Attribute>,
) -> Element {
    let auto_id = use_hook(|| format!("ui-nmi-{}", next_id()));
    let input_id = id.unwrap_or_else(|| format!("{auto_id}-input"));
    let message_id = format!("{auto_id}-message");

    let controlled = value.is_some();
    let mut own = use_signal(|| default_value);
    let mut draft = use_signal(|| None::<String>);
    let mut holding = use_signal(|| 0i8);

    let current = value.unwrap_or_else(|| own().unwrap_or(f64::NAN));
    let num = Some(current).filter(|n| n.is_finite());

    let step_size = if step > 0.0 && step.is_finite() {
        step
    } else {
        1.0
    };
    let big_step = large_step.filter(|s| *s > 0.0).unwrap_or(step_size * 10.0);
    let lo = min.unwrap_or(f64::NEG_INFINITY);
    let hi = max.unwrap_or(f64::INFINITY);
    let origin = if lo.is_finite() { lo } else { 0.0 };

    let shown = draft().unwrap_or_else(|| {
        num.map(|n| format_value(n, max_fraction_digits))
            .unwrap_or_default()
    });
    let invalid = error.is_some();
    let message: Option<Element> = error.clone().or(hint.clone());
    let show_error = error.is_some();
    let message_key = if show_error { "error" } else { "hint" };
    let at_min = num.is_some_and(|n| n <= lo);
    let at_max = num.is_some_and(|n| n >= hi);
    let class = class_list("ui-nmi", &class);
    let input_class = class_list("ui-nmi__input", &input_class);
    let decimals_shown = max_fraction_digits > 0 && step_size.fract() != 0.0;
    let input_mode = if lo < 0.0 {
        "text"
    } else if decimals_shown {
        "decimal"
    } else {
        "numeric"
    };
    let width_ch = shown
        .chars()
        .count()
        .max(placeholder.as_deref().unwrap_or("").chars().count())
        .max(2);
    let value_text = num.map(|n| format_value(n, max_fraction_digits));
    let roll_target = use_signal(|| input_id.clone());

    let mut set = move |next: Option<f64>| {
        if !controlled {
            own.set(next);
        }
        if next != num
            && let Some(handler) = &on_value_change
        {
            handler.call(next);
        }
    };

    let clamp = move |v: f64| v.max(lo).min(hi);

    let mut nudge = move |dir: f64, by: f64| {
        if disabled || read_only {
            return;
        }
        let typed = match draft() {
            Some(text) => match read_value(&text) {
                Reading::Value(v) => Some(v),
                _ => None,
            },
            None => num,
        };
        let next = match typed {
            None => clamp(if lo.is_finite() { lo } else { 0.0 }),
            Some(base) => clamp(round_to_step(
                next_on_grid(base, origin, step_size, by, dir),
                by,
            )),
        };
        let was_typing = draft().is_some();
        draft.set(None);
        if Some(next) == typed {
            if was_typing {
                set(Some(next));
            }
            return;
        }
        set(Some(next));
        document::eval(&roll_script(&roll_target.read(), dir));
    };

    let mut commit = move || {
        let Some(text) = draft() else { return };
        let reading = read_value(&text);
        draft.set(None);
        match reading {
            Reading::Empty => set(None),
            Reading::Invalid => {}
            Reading::Value(v) => set(Some(clamp(round_to_step(v, step_size)))),
        }
    };

    let mut press = move |dir: f64| {
        let sign = dir as i8;
        holding.set(sign);
        nudge(dir, step_size);
        spawn(async move {
            let mut delay = 400u32;
            sleep(delay).await;
            while holding() == sign {
                nudge(dir, step_size);
                delay = if delay >= 400 {
                    120
                } else {
                    ((f64::from(delay) * 0.9) as u32).max(40)
                };
                sleep(delay).await;
            }
        });
    };

    let stop = move |_| holding.set(0);
    let described = message.as_ref().map(|_| message_id.clone());

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-size": size.as_str(),
            "data-invalid": invalid.then_some("true"),
            "data-disabled": disabled.then_some("true"),
            "data-readonly": read_only.then_some("true"),
            if label.is_some() || optional {
                div { class: "ui-nmi__head",
                    if let Some(label) = label {
                        label { class: "ui-nmi__label", r#for: "{input_id}", {label} }
                    }
                    if optional {
                        span { class: "ui-nmi__optional", "Optional" }
                    }
                }
            }
            div { class: "ui-nmi__box",
                if let Some(prefix) = prefix {
                    span { class: "ui-nmi__affix", "data-side": "start", {prefix} }
                }
                input {
                    id: "{input_id}",
                    class: input_class,
                    r#type: "text",
                    "inputmode": input_mode,
                    role: "spinbutton",
                    autocomplete: "off",
                    autocorrect: "off",
                    spellcheck: "false",
                    value: "{shown}",
                    placeholder: placeholder.clone(),
                    disabled,
                    readonly: read_only,
                    form: form.clone(),
                    aria_valuenow: num.map(|n| n.to_string()),
                    aria_valuemin: lo.is_finite().then(|| lo.to_string()),
                    aria_valuemax: hi.is_finite().then(|| hi.to_string()),
                    aria_valuetext: value_text,
                    aria_invalid: invalid.then_some("true"),
                    aria_describedby: described,
                    style: "width: calc({width_ch}ch + 2px);",
                    oninput: move |event: FormEvent| {
                        let text: String = event.value().chars().filter(|c| c.is_ascii_digit() || ".,'’ -−%()".contains(*c)).collect();
                        draft.set(Some(text));
                        if let Some(handler) = &oninput {
                            handler.call(event);
                        }
                    },
                    onchange: move |event| {
                        if let Some(handler) = &onchange {
                            handler.call(event);
                        }
                    },
                    onfocus: move |event| {
                        if let Some(handler) = &onfocus {
                            handler.call(event);
                        }
                    },
                    onblur: move |event: FocusEvent| {
                        commit();
                        if let Some(handler) = &onblur {
                            handler.call(event);
                        }
                    },
                    onkeydown: move |event: KeyboardEvent| {
                        if !read_only {
                            let key = event.key();
                            match key {
                                Key::ArrowUp => { event.prevent_default(); nudge(1.0, step_size); }
                                Key::ArrowDown => { event.prevent_default(); nudge(-1.0, step_size); }
                                Key::PageUp => { event.prevent_default(); nudge(1.0, big_step); }
                                Key::PageDown => { event.prevent_default(); nudge(-1.0, big_step); }
                                Key::Home if lo.is_finite() => { event.prevent_default(); draft.set(None); set(Some(lo)); }
                                Key::End if hi.is_finite() => { event.prevent_default(); draft.set(None); set(Some(hi)); }
                                Key::Enter => commit(),
                                Key::Escape if draft().is_some() => { event.prevent_default(); draft.set(None); }
                                _ => {}
                            }
                        }
                        if let Some(handler) = &onkeydown {
                            handler.call(event);
                        }
                    },
                    ..attributes,
                }
                if let Some(unit) = unit {
                    span { class: "ui-nmi__affix", "data-side": "end", {unit} }
                }
                span { class: "ui-nmi__steps",
                    button {
                        r#type: "button",
                        class: "ui-nmi__step",
                        tabindex: "-1",
                        aria_label: "Decrease",
                        aria_controls: "{input_id}",
                        aria_disabled: ((at_min) || disabled || read_only).then_some("true"),
                        disabled,
                        onpointerdown: move |event: PointerEvent| {
                            event.prevent_default();
                            if !(at_min || disabled || read_only) {
                                press(-1.0);
                            }
                        },
                        onpointerup: stop,
                        onpointerleave: stop,
                        onpointercancel: stop,
                        oncontextmenu: move |event: MouseEvent| event.prevent_default(),
                        MinusIcon {}
                    }
                    button {
                        r#type: "button",
                        class: "ui-nmi__step",
                        tabindex: "-1",
                        aria_label: "Increase",
                        aria_controls: "{input_id}",
                        aria_disabled: ((at_max) || disabled || read_only).then_some("true"),
                        disabled,
                        onpointerdown: move |event: PointerEvent| {
                            event.prevent_default();
                            if !(at_max || disabled || read_only) {
                                press(1.0);
                            }
                        },
                        onpointerup: stop,
                        onpointerleave: stop,
                        onpointercancel: stop,
                        oncontextmenu: move |event: MouseEvent| event.prevent_default(),
                        PlusIcon {}
                    }
                }
            }
            if !name.is_empty() {
                input { r#type: "hidden", name: "{name}", form: form.clone(), value: num.map(|n| n.to_string()).unwrap_or_default(), disabled }
            }
            if let Some(message) = message {
                p {
                    class: "ui-nmi__message",
                    id: "{message_id}",
                    key: "{message_key}",
                    dir: "auto",
                    aria_live: show_error.then_some("polite"),
                    {message}
                }
            }
        }
    }
}

fn roll_script(input_id: &str, dir: f64) -> String {
    format!(
        "const el = document.getElementById({input_id:?}); if (!el || typeof el.animate !== 'function') return; if (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return; el.animate([{{ transform: 'translateY({}px)', opacity: 0.35, filter: 'blur(1.5px)' }}, {{ transform: 'none', opacity: 1, filter: 'none' }}], {{ duration: 220, easing: 'cubic-bezier(0.22, 1, 0.36, 1)' }});",
        dir * 5.0
    )
}

#[component]
fn MinusIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", "aria-hidden": "true",
            path { d: "M20 12H4" }
        }
    }
}

#[component]
fn PlusIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", "aria-hidden": "true",
            path { d: "M12 4v16M20 12H4" }
        }
    }
}
