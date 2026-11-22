//! Color picker: a swatch button that opens a card with a saturation and brightness area, hue and opacity
//! sliders, a hex field and preset swatches. `ColorPickerPanel` is the same card, inline.
//!
//! Port losses: when `open` is set to false from outside while the focus is in the card, the focus is not
//! returned to the button. `ref` forwarding is not available.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/color-picker.css");

const FALLBACK: &str = "#0190fe";
const FALLBACK_RGBA: Rgba = Rgba {
    r: 1.0,
    g: 144.0,
    b: 254.0,
    a: 1.0,
};
/// Gap between the button and the card, and the margin kept from the window edge.
const GAP: f64 = 6.0;
const PAD: f64 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba {
    /// 0 to 255.
    pub r: f64,
    /// 0 to 255.
    pub g: f64,
    /// 0 to 255.
    pub b: f64,
    /// 0 to 1.
    pub a: f64,
}

/// Hue in degrees; saturation, value and alpha from 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsva {
    pub h: f64,
    pub s: f64,
    pub v: f64,
    pub a: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edges {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extent {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardSide {
    #[default]
    Bottom,
    Top,
}

impl CardSide {
    fn as_str(self) -> &'static str {
        match self {
            Self::Bottom => "bottom",
            Self::Top => "top",
        }
    }

    fn other(self) -> Self {
        match self {
            Self::Bottom => Self::Top,
            Self::Top => Self::Bottom,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardAlign {
    #[default]
    Start,
    End,
}

impl CardAlign {
    fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardPlace {
    pub left: f64,
    pub top: f64,
    pub max_height: Option<f64>,
    pub side: CardSide,
    pub align: CardAlign,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorPickerSize {
    /// 24px high.
    Sm,
    /// 28px high.
    #[default]
    Md,
}

impl ColorPickerSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
        }
    }
}

/// A preset swatch. Its label is read out and shown as its tooltip; without one, the value in upper case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorPreset {
    pub value: String,
    pub label: Option<String>,
}

impl ColorPreset {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
        }
    }

    pub fn named(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: Some(label.into()),
        }
    }

    fn text(&self) -> String {
        self.label
            .clone()
            .unwrap_or_else(|| self.value.to_uppercase())
    }
}

fn hex2(n: f64) -> String {
    format!("{:02x}", n.clamp(0.0, 255.0).round() as u8)
}

fn is_hex(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// `#rrggbb`, or `#rrggbbaa` with `with_alpha` and alpha below 1.
pub fn format_hex(c: Rgba, with_alpha: bool) -> String {
    let mut out = format!("#{}{}{}", hex2(c.r), hex2(c.g), hex2(c.b));
    if with_alpha && c.a < 1.0 {
        out.push_str(&hex2(c.a * 255.0));
    }
    out
}

/// A CSS color for a swatch: `rgb(r g b / a)`.
pub fn paint(c: Rgba) -> String {
    format!(
        "rgb({} {} {} / {})",
        c.r.round(),
        c.g.round(),
        c.b.round(),
        c.a
    )
}

fn is_number(part: &str) -> bool {
    let body = part.strip_prefix('-').unwrap_or(part);
    let body = body.strip_suffix('%').unwrap_or(body);
    body.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        && body.bytes().any(|b| b.is_ascii_digit())
        && body.matches('.').count() <= 1
}

fn channel(part: &str) -> f64 {
    let value = match part.strip_suffix('%') {
        Some(pct) => pct.parse::<f64>().unwrap_or(0.0) * 2.55,
        None => part.parse::<f64>().unwrap_or(0.0),
    };
    value.round().clamp(0.0, 255.0)
}

/// `#0190fe`, `0190FE`, `#09f`, `#0190fecc`, `rgb(1, 144, 254)`, `rgb(1 144 254 / 80%)` or `rgba(1,144,254,.8)`.
/// `None` for anything else.
pub fn parse_color(input: &str) -> Option<Rgba> {
    let s = input.trim().to_ascii_lowercase();
    let hex = s.strip_prefix('#').unwrap_or(&s);
    if matches!(hex.len(), 3 | 4 | 6 | 8) && is_hex(hex) {
        let full: String = if hex.len() <= 4 {
            hex.chars().flat_map(|ch| [ch, ch]).collect()
        } else {
            hex.to_string()
        };
        let n = |i: usize| u8::from_str_radix(&full[i..i + 2], 16).ok().map(f64::from);
        return Some(Rgba {
            r: n(0)?,
            g: n(2)?,
            b: n(4)?,
            a: if full.len() == 8 { n(6)? / 255.0 } else { 1.0 },
        });
    }
    let body = s
        .strip_prefix("rgba(")
        .or_else(|| s.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    if body.contains(')') {
        return None;
    }
    let parts: Vec<&str> = body
        .trim()
        .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .collect();
    if !(parts.len() == 3 || parts.len() == 4) || !parts.iter().all(|p| is_number(p)) {
        return None;
    }
    let a = match parts.get(3) {
        None => 1.0,
        Some(p) => match p.strip_suffix('%') {
            Some(pct) => pct.parse::<f64>().unwrap_or(0.0) / 100.0,
            None => p.parse::<f64>().unwrap_or(0.0),
        }
        .clamp(0.0, 1.0),
    };
    Some(Rgba {
        r: channel(parts[0]),
        g: channel(parts[1]),
        b: channel(parts[2]),
        a,
    })
}

/// True when the text is a whole color that can apply while it is typed: six or eight hex digits, or a closed `rgb(…)`.
/// `#rgb` is not complete: it waits for Enter.
pub fn is_complete(input: &str) -> bool {
    let t = input.trim();
    let hex = t.strip_prefix('#').unwrap_or(t);
    let lower = t.to_ascii_lowercase();
    (matches!(hex.len(), 6 | 8) && is_hex(hex))
        || ((lower.starts_with("rgb(") || lower.starts_with("rgba(")) && t.ends_with(')'))
}

pub fn to_rgba(c: Hsva) -> Rgba {
    let f = |n: f64| {
        let k = (n + c.h / 60.0) % 6.0;
        ((c.v - c.v * c.s * 0.0_f64.max(k.min(4.0 - k).min(1.0))) * 255.0).round()
    };
    Rgba {
        r: f(5.0),
        g: f(3.0),
        b: f(1.0),
        a: c.a,
    }
}

/// Grays have no hue and black has no saturation: those keep `keep_h` and `keep_s`, so the thumbs stay put.
/// Without `alpha` the result is opaque.
pub fn to_hsva(c: Rgba, keep_h: f64, alpha: bool, keep_s: f64) -> Hsva {
    let (r, g, b) = (c.r / 255.0, c.g / 255.0, c.b / 255.0);
    let max = r.max(g).max(b);
    let d = max - r.min(g).min(b);
    let mut h = keep_h;
    if d > 0.0 {
        let raw = if max == r {
            ((g - b) / d) % 6.0
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        h = (raw * 60.0 + 360.0) % 360.0;
    }
    Hsva {
        h,
        s: if max == 0.0 { keep_s } else { d / max },
        v: max,
        a: if alpha { c.a } else { 1.0 },
    }
}

/// The hue as a word, for screen readers.
pub fn hue_name(h: f64) -> &'static str {
    match h {
        h if h < 15.0 => "red",
        h if h < 45.0 => "orange",
        h if h < 70.0 => "yellow",
        h if h < 165.0 => "green",
        h if h < 195.0 => "cyan",
        h if h < 255.0 => "blue",
        h if h < 290.0 => "purple",
        h if h < 335.0 => "pink",
        _ => "red",
    }
}

/// The saturation and brightness area, from arrow, Page, Home and End keys. Shift makes an arrow step 10.
/// `s` and `v` are 0 to 100 and rounded first. The result is 0 to 1 for both, or `None` for another key.
pub fn area_key(key: &str, shift: bool, s: f64, v: f64) -> Option<(f64, f64)> {
    let step = if shift { 10.0 } else { 1.0 };
    let (dx, dy) = match key {
        "ArrowRight" => (step, 0.0),
        "ArrowLeft" => (-step, 0.0),
        "ArrowUp" => (0.0, step),
        "ArrowDown" => (0.0, -step),
        "PageUp" => (0.0, 10.0),
        "PageDown" => (0.0, -10.0),
        "Home" => (-100.0, 0.0),
        "End" => (100.0, 0.0),
        _ => return None,
    };
    let (s, v) = (s.round(), v.round());
    Some((
        (s + dx).clamp(0.0, 100.0) / 100.0,
        (v + dy).clamp(0.0, 100.0) / 100.0,
    ))
}

/// A slider's keys: arrows step 1 (Shift 10), Page Up and Down step 10, Home is 0 and End is `max`.
/// The new value, clamped to `[0, max]`, or `None` for another key.
pub fn slider_key(key: &str, shift: bool, now: f64, max: f64) -> Option<f64> {
    let step = if shift { 10.0 } else { 1.0 };
    let next = match key {
        "ArrowRight" | "ArrowUp" => now + step,
        "ArrowLeft" | "ArrowDown" => now - step,
        "PageUp" => now + 10.0,
        "PageDown" => now - 10.0,
        "Home" => 0.0,
        "End" => max,
        _ => return None,
    };
    Some(next.clamp(0.0, max))
}

/// Under the button or over it, whichever has room. The card lines up with the start (or end) edge of the button
/// and flips edge when it would stick out. It shrinks to the room when it is taller than that.
pub fn place_card(
    trigger: Edges,
    card: Extent,
    view: Extent,
    side: CardSide,
    align: CardAlign,
) -> CardPlace {
    let room = |s: CardSide| match s {
        CardSide::Bottom => view.height - trigger.bottom - GAP - PAD,
        CardSide::Top => trigger.top - GAP - PAD,
    };
    let mut s = side;
    if room(s) < card.height && room(s.other()) > room(s) {
        s = s.other();
    }
    let fit = card.height.min(room(s).max(160.0));
    let top = match s {
        CardSide::Bottom => trigger.bottom + GAP,
        CardSide::Top => trigger.top - GAP - fit,
    };
    let mut a = align;
    let mut left = match a {
        CardAlign::Start => trigger.left,
        CardAlign::End => trigger.right - card.width,
    };
    if a == CardAlign::Start
        && left + card.width > view.width - PAD
        && trigger.right - card.width >= PAD
    {
        a = CardAlign::End;
        left = trigger.right - card.width;
    } else if a == CardAlign::End && left < PAD && trigger.left + card.width <= view.width - PAD {
        a = CardAlign::Start;
        left = trigger.left;
    }
    left = left.max(PAD).min(view.width - PAD - card.width);
    CardPlace {
        left,
        top,
        max_height: (card.height > fit).then_some(fit),
        side: s,
        align: a,
    }
}

/// A swatch button with its card. Controlled by `value` (and `open`), or started by `default_value` (and `default_open`).
#[component]
pub fn ColorPicker(
    #[props(default, into)] value: Option<String>,
    #[props(default, into)] default_value: Option<String>,
    #[props(default)] on_value_change: Option<EventHandler<String>>,
    /// Shows the opacity slider. Below 100% the value gets two more digits.
    #[props(default)]
    alpha: bool,
    #[props(default)] presets: Vec<ColorPreset>,
    #[props(default)] open: Option<bool>,
    #[props(default)] default_open: bool,
    #[props(default)] on_open_change: Option<EventHandler<bool>>,
    /// What the color is for. Names the button and the card for screen readers.
    #[props(default = "Color".to_string(), into)]
    label: String,
    #[props(default = true)] show_value: bool,
    #[props(default)] size: ColorPickerSize,
    #[props(default)] side: CardSide,
    #[props(default)] align: CardAlign,
    #[props(default, into)] name: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] onfocus: Option<EventHandler<FocusEvent>>,
    #[props(default)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(extends = GlobalAttributes, extends = button)] attributes: Vec<Attribute>,
) -> Element {
    let uid = use_hook(|| format!("ui-clpc-{}", next_id()));
    let card_id = format!("{uid}-card");
    let button_id = format!("{uid}-button");
    let (current, set_value) = use_color_value(value, default_value, on_value_change);

    let own_open = use_signal(|| default_open);
    let is_open = open.unwrap_or(own_open());
    let set_open = use_callback(move |next: bool| {
        if open.is_none() {
            let mut own = own_open;
            own.set(next);
        }
        if let Some(cb) = on_open_change {
            cb.call(next);
        }
    });

    let placed = use_signal(|| None::<CardPlace>);
    let mut was_open = use_signal(|| is_open);
    {
        let ids = PopIds {
            card: card_id.clone(),
            button: button_id.clone(),
        };
        use_effect(use_reactive((&is_open,), move |(now_open,)| {
            if !now_open {
                let mut placed = placed;
                placed.set(None);
                document::eval(&release_script(&ids.card));
                return;
            }
            listen(ids.clone(), side, align, placed, set_open);
            place_card_now(ids.clone(), side, align, placed);
        }));
    }
    use_effect(use_reactive((&is_open,), {
        let card_id = card_id.clone();
        move |(now_open,)| {
            if now_open && !*was_open.peek() {
                document::eval(&format!(
                    "document.getElementById({card_id:?})?.querySelector('[role=slider]')?.focus({{ preventScroll: true }});"
                ));
            }
            was_open.set(now_open);
        }
    }));

    let rgba = parse_color(&current);
    let shown = rgba.map_or_else(|| current.clone(), |c| format_hex(c, false).to_uppercase());
    let opacity = rgba
        .filter(|c| alpha && c.a < 1.0)
        .map(|c| format!("{}%", (c.a * 100.0).round()));
    let name_value = rgba.map_or_else(|| current.clone(), |c| format_hex(c, alpha));
    let swatch_style = format!(
        "--ui-clpc-c: {};",
        rgba.map_or_else(|| current.clone(), paint)
    );
    let placement = placed();
    let shown_side = placement.map_or(side, |p| p.side);
    let shown_align = placement.map_or(align, |p| p.align);
    let pop_style = match placement {
        Some(p) => {
            let max = p
                .max_height
                .map(|m| format!("max-height:{}px;", m.round()))
                .unwrap_or_default();
            format!(
                "top:{}px;left:{}px;visibility:visible;{max}",
                p.top.round(),
                p.left.round()
            )
        }
        None => "visibility:hidden;".to_string(),
    };
    let class = class_list("ui-clpc", &class);
    let size_attr = size.as_str();
    let side_attr = shown_side.as_str();
    let align_attr = shown_align.as_str();

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        span { class: "{class}", "data-size": "{size_attr}",
            button {
                id: "{button_id}",
                r#type: "button",
                class: "ui-clpc__trigger",
                "data-bare": (!show_value).then_some("true"),
                disabled,
                aria_haspopup: "dialog",
                aria_expanded: "{is_open}",
                aria_controls: is_open.then(|| card_id.clone()),
                onclick: move |event: MouseEvent| {
                    set_open.call(!is_open);
                    if let Some(handler) = &onclick {
                        handler.call(event);
                    }
                },
                onfocus: move |event| {
                    if let Some(handler) = &onfocus {
                        handler.call(event);
                    }
                },
                onblur: move |event| {
                    if let Some(handler) = &onblur {
                        handler.call(event);
                    }
                },
                onkeydown: move |event| {
                    if let Some(handler) = &onkeydown {
                        handler.call(event);
                    }
                },
                ..attributes,
                span { class: "ui-clpc__swatch", style: "{swatch_style}", aria_hidden: "true" }
                span { class: if show_value { "ui-clpc__value" } else { "ui-clpc__sr" },
                    span { class: "ui-clpc__sr", "{label}: " }
                    "{shown}"
                    if let Some(pct) = opacity {
                        span { class: "ui-clpc__alpha", " {pct}" }
                    }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name: "{name}", value: "{name_value}" }
            }
            if is_open {
                div {
                    id: "{card_id}",
                    class: "ui-clpc__pop",
                    role: "dialog",
                    aria_label: "{label}",
                    "popover": "manual",
                    "data-side": "{side_attr}",
                    "data-align": "{align_attr}",
                    style: "{pop_style}",
                    Panel { value: current.clone(), on_change: set_value, alpha, presets: presets.clone() }
                }
            }
        }
    }
}

/// The same card without the button, for a form or a settings page.
#[component]
pub fn ColorPickerPanel(
    #[props(default, into)] value: Option<String>,
    #[props(default, into)] default_value: Option<String>,
    #[props(default)] on_value_change: Option<EventHandler<String>>,
    #[props(default)] alpha: bool,
    #[props(default)] presets: Vec<ColorPreset>,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let (current, set_value) = use_color_value(value, default_value, on_value_change);
    let class = class_list("ui-clpc ui-clpc--inline", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "{class}", ..attributes,
            Panel { value: current, on_change: set_value, alpha, presets }
        }
    }
}

/// The value of a color control: the prop when controlled, the own state otherwise.
fn use_color_value(
    value: Option<String>,
    default_value: Option<String>,
    on_value_change: Option<EventHandler<String>>,
) -> (String, Callback<String>) {
    let own = use_signal(|| default_value.unwrap_or_else(|| FALLBACK.to_string()));
    let controlled = value.is_some();
    let current = value.unwrap_or(own());
    let set_value = use_callback(move |next: String| {
        if !controlled {
            let mut own = own;
            own.set(next.clone());
        }
        if let Some(cb) = on_value_change {
            cb.call(next);
        }
    });
    (current, set_value)
}

#[component]
fn Panel(
    value: String,
    on_change: Callback<String>,
    #[props(default)] alpha: bool,
    #[props(default)] presets: Vec<ColorPreset>,
) -> Element {
    let uid = use_hook(|| format!("ui-clpc-p-{}", next_id()));
    let area_id = format!("{uid}-area");
    let hue_id = format!("{uid}-hue");
    let opacity_id = format!("{uid}-opacity");
    let presets_id = format!("{uid}-presets");

    let mut c = use_signal(|| {
        to_hsva(
            parse_color(&value).unwrap_or(FALLBACK_RGBA),
            0.0,
            alpha,
            0.0,
        )
    });
    let mut shown = use_signal(|| value.clone());
    let mut draft = use_signal(|| None::<String>);
    let mut bad = use_signal(|| false);

    use_effect(use_reactive((&value, &alpha), move |(value, alpha)| {
        shown.set(value.clone());
        let Some(p) = parse_color(&value) else { return };
        let current = *c.peek();
        if format_hex(p, alpha) != format_hex(to_rgba(current), alpha) {
            c.set(to_hsva(p, current.h, alpha, current.s));
        }
    }));

    let cur = c();
    let rgb = to_rgba(cur);
    let hex = format_hex(rgb, alpha);
    let s_pct = (cur.s * 100.0).round();
    let v_pct = (cur.v * 100.0).round();
    let h_deg = cur.h.round();
    let a_pct = (cur.a * 100.0).round();
    let panel_style = format!(
        "--ui-clpc-c: {}; --ui-clpc-solid: {}; --ui-clpc-hue: hsl({} 100% 50%);",
        paint(rgb),
        paint(Rgba { a: 1.0, ..rgb }),
        h_deg,
    );
    let thumb_style = format!("left: {}%; top: {}%;", cur.s * 100.0, (1.0 - cur.v) * 100.0);
    let hue_knob = format!(
        "--p: {}; background: hsl({} 100% 50%);",
        cur.h / 360.0,
        h_deg
    );
    let alpha_knob = format!("--p: {};", cur.a);
    let hue_text = format!("{}°, {}", h_deg, hue_name(h_deg));
    let alpha_text = format!("{}%", a_pct);

    let list: Vec<(String, String)> = presets
        .iter()
        .map(|p| (p.value.clone(), p.text()))
        .collect();
    let picked = list
        .iter()
        .position(|(v, _)| parse_color(v).is_some_and(|q| format_hex(q, alpha) == hex));
    let focus_slot = picked.unwrap_or(0);
    let field_value = draft().unwrap_or_else(|| hex[1..].to_uppercase());

    rsx! {
        div { class: "ui-clpc__panel", style: "{panel_style}",
            div {
                id: "{area_id}",
                class: "ui-clpc__area",
                role: "slider",
                tabindex: "0",
                dir: "ltr",
                aria_label: "Saturation and brightness",
                aria_valuemin: "0",
                aria_valuemax: "100",
                aria_valuenow: "{s_pct}",
                aria_valuetext: "Saturation {s_pct}%, brightness {v_pct}%",
                onpointerdown: move |e: PointerEvent| drag(area_id.clone(), Track::Area, e, c, shown, on_change, alpha),
                onkeydown: move |e: KeyboardEvent| {
                    let key = e.key();
                    let Some(name) = key_name(&key) else { return };
                    let now = *c.peek();
                    if let Some((s, v)) = area_key(name, e.modifiers().shift(), s_pct, v_pct) {
                        e.prevent_default();
                        apply(c, shown, on_change, alpha, Hsva { s, v, ..now });
                    }
                },
                span { class: "ui-clpc__thumb", style: "{thumb_style}" }
            }

            div {
                id: "{hue_id}",
                class: "ui-clpc__slider ui-clpc__slider--hue",
                role: "slider",
                tabindex: "0",
                dir: "ltr",
                aria_label: "Hue",
                aria_orientation: "horizontal",
                aria_valuemin: "0",
                aria_valuemax: "360",
                aria_valuenow: "{h_deg}",
                aria_valuetext: "{hue_text}",
                onpointerdown: move |e: PointerEvent| drag(hue_id.clone(), Track::Hue, e, c, shown, on_change, alpha),
                onkeydown: move |e: KeyboardEvent| {
                    let key = e.key();
                    let Some(name) = key_name(&key) else { return };
                    if let Some(h) = slider_key(name, e.modifiers().shift(), h_deg, 360.0) {
                        e.prevent_default();
                        let now = *c.peek();
                        apply(c, shown, on_change, alpha, Hsva { h, ..now });
                    }
                },
                span { class: "ui-clpc__knob", style: "{hue_knob}" }
            }

            if alpha {
                div {
                    id: "{opacity_id}",
                    class: "ui-clpc__slider ui-clpc__slider--alpha",
                    role: "slider",
                    tabindex: "0",
                    dir: "ltr",
                    aria_label: "Opacity",
                    aria_orientation: "horizontal",
                    aria_valuemin: "0",
                    aria_valuemax: "100",
                    aria_valuenow: "{a_pct}",
                    aria_valuetext: "{alpha_text}",
                    onpointerdown: move |e: PointerEvent| drag(opacity_id.clone(), Track::Opacity, e, c, shown, on_change, alpha),
                    onkeydown: move |e: KeyboardEvent| {
                        let key = e.key();
                        let Some(name) = key_name(&key) else { return };
                        if let Some(a) = slider_key(name, e.modifiers().shift(), a_pct, 100.0) {
                            e.prevent_default();
                            let now = *c.peek();
                            apply(c, shown, on_change, alpha, Hsva { a: a / 100.0, ..now });
                        }
                    },
                    span { class: "ui-clpc__knob", style: "{alpha_knob}" }
                }
            }

            div { class: "ui-clpc__fields",
                span { class: "ui-clpc__preview", aria_hidden: "true" }
                label { class: "ui-clpc__hex", "data-invalid": bad().then_some("true"),
                    span { class: "ui-clpc__hash", aria_hidden: "true", "#" }
                    input {
                        class: "ui-clpc__input",
                        value: "{field_value}",
                        aria_label: "Hex color",
                        aria_invalid: bad().then_some("true"),
                        "spellcheck": "false",
                        "autocomplete": "off",
                        "autocapitalize": "off",
                        oninput: move |e: FormEvent| {
                            let text = e.value();
                            draft.set(Some(text.clone()));
                            bad.set(false);
                            if is_complete(&text) {
                                commit_text(c, shown, on_change, alpha, &text);
                            }
                        },
                        onblur: move |_| {
                            if let Some(text) = draft() {
                                commit_text(c, shown, on_change, alpha, &text);
                            }
                            draft.set(None);
                            bad.set(false);
                        },
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() != Key::Enter {
                                return;
                            }
                            let Some(text) = draft() else { return };
                            e.prevent_default();
                            if commit_text(c, shown, on_change, alpha, &text) {
                                draft.set(None);
                            } else {
                                bad.set(true);
                            }
                        },
                    }
                }
                if alpha {
                    span { class: "ui-clpc__pct", "{alpha_text}" }
                }
            }

            if !list.is_empty() {
                div { id: "{presets_id}", class: "ui-clpc__presets", role: "radiogroup", aria_label: "Presets",
                    for (i, (preset_value, text)) in list.into_iter().enumerate() {
                        button {
                            key: "{i}",
                            r#type: "button",
                            role: "radio",
                            class: "ui-clpc__preset",
                            aria_checked: "{picked == Some(i)}",
                            aria_label: "{text}",
                            title: "{text}",
                            tabindex: if i == focus_slot { "0" } else { "-1" },
                            style: "--ui-clpc-p: {preset_value};",
                            onclick: {
                                let preset_value = preset_value.clone();
                                move |_| {
                                    commit_text(c, shown, on_change, alpha, &preset_value);
                                }
                            },
                            onkeydown: {
                                let presets_id = presets_id.clone();
                                move |e: KeyboardEvent| {
                                    let key = e.key();
                                    let Some(name) = key_name(&key) else { return };
                                    if !matches!(name, "ArrowRight" | "ArrowDown" | "ArrowLeft" | "ArrowUp" | "Home" | "End") {
                                        return;
                                    }
                                    e.prevent_default();
                                    document::eval(&preset_step_script(&presets_id, i, name));
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Track {
    Area,
    Hue,
    Opacity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PopIds {
    card: String,
    button: String,
}

/// Sets the color from a track's new position. Reports to `on_change` only when the hex changed.
fn apply(
    mut c: Signal<Hsva>,
    mut shown: Signal<String>,
    on_change: Callback<String>,
    alpha: bool,
    next: Hsva,
) {
    c.set(next);
    let out = format_hex(to_rgba(next), alpha);
    if out != *shown.peek() {
        shown.set(out.clone());
        on_change.call(out);
    }
}

/// Applies a typed or chosen color. Returns false when `text` is not a color.
fn commit_text(
    c: Signal<Hsva>,
    shown: Signal<String>,
    on_change: Callback<String>,
    alpha: bool,
    text: &str,
) -> bool {
    let Some(p) = parse_color(text) else {
        return false;
    };
    let cur = *c.peek();
    let p = if alpha { p } else { Rgba { a: 1.0, ..p } };
    apply(c, shown, on_change, alpha, to_hsva(p, cur.h, alpha, cur.s));
    true
}

/// Press and drag on a track. The pointer is captured, so the drag keeps working outside the card; the track
/// reports its normalised position, and the drag ends with a press-up.
fn drag(
    id: String,
    track: Track,
    e: PointerEvent,
    c: Signal<Hsva>,
    shown: Signal<String>,
    on_change: Callback<String>,
    alpha: bool,
) {
    if e.trigger_button() != Some(dioxus::html::input_data::MouseButton::Primary) {
        return;
    }
    e.prevent_default();
    let point = e.client_coordinates();
    let script = drag_script(&id, track, point.x, point.y, e.pointer_id());
    spawn(async move {
        let mut events = document::eval(&script);
        while let Ok(message) = events.recv::<String>().await {
            if message == "end" {
                break;
            }
            let Some((x, y)) = message
                .split_once(' ')
                .and_then(|(x, y)| Some((x.parse::<f64>().ok()?, y.parse::<f64>().ok()?)))
            else {
                continue;
            };
            let cur = *c.peek();
            let next = match track {
                Track::Area => Hsva {
                    s: x,
                    v: 1.0 - y,
                    ..cur
                },
                Track::Hue => Hsva {
                    h: x * 360.0,
                    ..cur
                },
                Track::Opacity => Hsva {
                    a: (x * 100.0).round() / 100.0,
                    ..cur
                },
            };
            apply(c, shown, on_change, alpha, next);
        }
    });
}

fn drag_script(id: &str, track: Track, client_x: f64, client_y: f64, pointer: i32) -> String {
    let area = matches!(track, Track::Area);
    format!(
        r#"const el = document.getElementById({id:?});
const send = (message) => {{ try {{ dioxus.send(message); }} catch (_) {{}} }};
if (!el) {{ send("end"); return; }}
const pos = (x, y) => {{
  const r = el.getBoundingClientRect();
  const inset = {area} ? 0 : r.height / 2;
  const px = Math.min(1, Math.max(0, (x - r.left - inset) / Math.max(1, r.width - inset * 2)));
  const py = Math.min(1, Math.max(0, (y - r.top) / Math.max(1, r.height)));
  return px + " " + py;
}};
el.focus({{ preventScroll: true }});
try {{ el.setPointerCapture({pointer}); }} catch (_) {{}}
send(pos({client_x}, {client_y}));
const move = (e) => {{ if (e.pointerId === {pointer}) send(pos(e.clientX, e.clientY)); }};
const up = (e) => {{
  if (e.pointerId !== {pointer}) return;
  el.removeEventListener("pointermove", move);
  el.removeEventListener("pointerup", up);
  el.removeEventListener("pointercancel", up);
  try {{ el.releasePointerCapture({pointer}); }} catch (_) {{}}
  send("end");
}};
el.addEventListener("pointermove", move);
el.addEventListener("pointerup", up);
el.addEventListener("pointercancel", up);"#
    )
}

fn key_name(key: &Key) -> Option<&str> {
    Some(match key {
        Key::Character(c) => c.as_str(),
        Key::ArrowLeft => "ArrowLeft",
        Key::ArrowRight => "ArrowRight",
        Key::ArrowUp => "ArrowUp",
        Key::ArrowDown => "ArrowDown",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",
        Key::Home => "Home",
        Key::End => "End",
        Key::Escape => "Escape",
        _ => return None,
    })
}

/// Moves to the neighbouring preset (wrapping; Home and End go to the ends) and clicks it, so the color follows.
fn preset_step_script(id: &str, index: usize, key: &str) -> String {
    format!(
        r#"const cont = document.getElementById({id:?});
if (!cont) return;
const n = cont.children.length;
const back = getComputedStyle(cont).direction === "rtl";
const key = {key:?};
const d = {{ ArrowRight: back ? -1 : 1, ArrowDown: 1, ArrowLeft: back ? 1 : -1, ArrowUp: -1 }}[key];
const to = key === "Home" ? 0 : key === "End" ? n - 1 : d != null ? ({index} + d + n) % n : null;
if (to == null) return;
const next = cont.children[to];
next?.focus();
next?.click();"#
    )
}

fn release_script(card: &str) -> String {
    format!("window.__affectOwners?.[{card:?}]?.remove();")
}

/// Window, outside-press and Escape listeners for the open card. Installing again replaces the previous set.
/// Messages: `move` (re-place), `outside` or `close` (close).
fn listen_script(ids: &PopIds) -> String {
    format!(
        r#"const owners = (window.__affectOwners ||= {{}});
owners[{card:?}]?.remove();
const handlers = [];
const on = (target, type, fn, options) => {{ target.addEventListener(type, fn, options); handlers.push(() => target.removeEventListener(type, fn, options)); }};
const send = (message) => {{ try {{ dioxus.send(message); }} catch (_) {{}} }};
owners[{card:?}] = {{ remove: () => handlers.forEach((undo) => undo()) }};
const card = () => document.getElementById({card:?});
const button = () => document.getElementById({button:?});
const inside = (node) => !!node && (!!card()?.contains(node) || !!button()?.contains(node));
on(document, "pointerdown", (e) => {{ if (!inside(e.target)) send("outside"); }}, true);
on(window, "keydown", (e) => {{
  if (e.key !== "Escape" || e.defaultPrevented) return;
  e.preventDefault();
  e.stopPropagation();
  if (inside(document.activeElement)) button()?.focus({{ preventScroll: true }});
  send("close");
}}, true);
on(document, "focusout", (e) => {{ if (inside(e.target) && e.relatedTarget && !inside(e.relatedTarget)) send("close"); }});
on(window, "resize", () => send("move"));
on(window, "scroll", () => send("move"), true);
if (window.ResizeObserver && card()) {{
  const ro = new ResizeObserver(() => send("move"));
  ro.observe(card());
  handlers.push(() => ro.disconnect());
}}"#,
        card = ids.card,
        button = ids.button,
    )
}

/// Starts the listeners for the open card. Runs until the listeners are released by `release_script`.
fn listen(
    ids: PopIds,
    side: CardSide,
    align: CardAlign,
    placed: Signal<Option<CardPlace>>,
    close: Callback<bool>,
) {
    spawn(async move {
        let mut events = document::eval(&listen_script(&ids));
        while let Ok(message) = events.recv::<String>().await {
            match message.as_str() {
                "move" => place_card_now(ids.clone(), side, align, placed),
                "outside" | "close" => close.call(false),
                _ => {}
            }
        }
    });
}

/// Measures the button and the card, then sets the card's place. Clears the card's own max height first, so the
/// measured height is the card's natural one, as in the original.
fn place_card_now(
    ids: PopIds,
    side: CardSide,
    align: CardAlign,
    mut placed: Signal<Option<CardPlace>>,
) {
    let script = format!(
        r#"const card = document.getElementById({card:?});
const b = document.getElementById({button:?});
if (!card || !b) return null;
if ("showPopover" in card && !card.matches(":popover-open")) card.showPopover();
card.style.maxHeight = "";
const r = b.getBoundingClientRect();
return [r.left, r.top, r.right, r.bottom, document.documentElement.clientWidth || window.innerWidth, window.innerHeight, card.offsetWidth, card.offsetHeight];"#,
        card = ids.card,
        button = ids.button,
    );
    spawn(async move {
        let Ok(v) = document::eval(&script).await else {
            return;
        };
        let Some(nums) = (0..8)
            .map(|i| v.get(i)?.as_f64())
            .collect::<Option<Vec<f64>>>()
        else {
            return;
        };
        let trigger = Edges {
            left: nums[0],
            top: nums[1],
            right: nums[2],
            bottom: nums[3],
        };
        let view = Extent {
            width: nums[4],
            height: nums[5],
        };
        let card = Extent {
            width: nums[6],
            height: nums[7],
        };
        placed.set(Some(place_card(trigger, card, view, side, align)));
    });
}
