//! Avatar: a person's picture with initials as the fallback on a soft colour
//! picked from the name; an agent as its own pixel mark that twinkles while it
//! works; a status dot; and `AvatarGroup`, which overlaps them and ends in +N.
//!
//! Port losses: `AvatarGroup` takes its members as `AvatarItem` data, not as
//! children, because children cannot be trimmed to `max` or re-sized. Initials
//! are split on code points with a simple emoji and joiner rule, not full
//! grapheme segmentation.

use dioxus::prelude::*;

use crate::badge::group_thousands;
use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/avatar.css");
const WAVE: [u32; 9] = [0, 120, 240, 120, 240, 360, 240, 360, 480];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    /// 20px.
    S20,
    /// 24px.
    S24,
    /// 32px.
    #[default]
    S32,
    /// 40px.
    S40,
}

impl AvatarSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::S20 => "20",
            Self::S24 => "24",
            Self::S32 => "32",
            Self::S40 => "40",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarShape {
    Circle,
    Square,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarStatus {
    Online,
    Away,
    Busy,
}

impl AvatarStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Away => "away",
            Self::Busy => "busy",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarKind {
    /// A picture or initials.
    #[default]
    Person,
    /// The agent's pixel mark, never initials.
    Agent,
}

impl AvatarKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Agent => "agent",
        }
    }
}

/// One member of an `AvatarGroup`. Empty strings mean "not set".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AvatarItem {
    pub name: String,
    pub src: String,
    pub shape: Option<AvatarShape>,
    pub status: Option<AvatarStatus>,
    pub kind: AvatarKind,
    pub seed: String,
    pub color: String,
    pub working: bool,
}

/// A person's picture or initials, or an agent's pixel mark.
///
/// `name` gives the initials, the accessible name and the colour. `src` shows a
/// picture once it loads; until then, and if it fails, the initials stay.
#[component]
pub fn Avatar(
    #[props(default, into)] name: String,
    #[props(default, into)] src: String,
    #[props(default)] size: AvatarSize,
    #[props(default)] shape: Option<AvatarShape>,
    #[props(default)] status: Option<AvatarStatus>,
    #[props(default)] kind: AvatarKind,
    #[props(default, into)] seed: String,
    #[props(default, into)] color: String,
    #[props(default)] working: bool,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = span)] attributes: Vec<Attribute>,
) -> Element {
    let mut failed = use_signal(String::new);
    let mut loaded = use_signal(String::new);
    let img_id = use_hook(|| format!("ui-avatar-img-{}", next_id()));

    let agent = kind == AvatarKind::Agent;
    let form = form_of(shape, kind);
    let key = if seed.is_empty() {
        name.trim()
    } else {
        seed.trim()
    };
    let show_image = !src.is_empty() && failed.read().as_str() != src;
    let ready = show_image && loaded.read().as_str() == src;
    let who = name.trim();
    let label = (!who.is_empty()).then(|| match status {
        Some(state) => format!("{who}, {}", state.as_str()),
        None => who.to_string(),
    });
    let hue = (!agent && !key.is_empty()).then(|| hue_of(key).to_string());
    let style = (agent && !color.is_empty()).then(|| format!("--ui-avt-mark: {color};"));
    let initials = initials_of(&name);
    let cells = mark_cells(if key.is_empty() { "agent" } else { key });

    let src_load = src.clone();
    let src_error = src.clone();
    let src_mounted = src.clone();
    let mounted_id = img_id.clone();
    let class = class_list("ui-avt", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        span {
            class,
            "data-size": size.as_str(),
            "data-shape": form,
            "data-kind": kind.as_str(),
            "data-hue": hue,
            "data-ready": ready.then_some(""),
            "data-working": (agent && working).then_some(""),
            role: label.as_ref().map(|_| "img"),
            aria_label: label.clone(),
            aria_hidden: label.is_none().then_some("true"),
            style,
            ..attributes,
            span { class: "ui-avt__face",
                if agent {
                    span { class: "ui-avt__mark", aria_hidden: "true",
                        for (idx, (on, delay)) in cells.into_iter().zip(WAVE).enumerate() {
                            i { key: "{idx}", "data-on": on.then_some(""), style: "animation-delay: {delay}ms;" }
                        }
                    }
                } else {
                    span { class: "ui-avt__initials", dir: "auto", aria_hidden: "true",
                        if initials.is_empty() {
                            svg {
                                class: "ui-avt__anon",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.8",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M20 21C19.713 17.269 16.729 14.315 12.995 14.066L12 14C11.645 14.01 11.313 14.023 11.001 14.038C7.3 14.219 4.284 17.306 4 21" }
                                circle { cx: "12", cy: "7", r: "4" }
                            }
                        } else {
                            "{initials}"
                        }
                    }
                }
                if show_image {
                    img {
                        key: "{src}",
                        id: "{img_id}",
                        class: "ui-avt__img",
                        src: "{src}",
                        alt: "",
                        loading: "lazy",
                        decoding: "async",
                        draggable: "false",
                        onmounted: move |_| {
                            // A cached picture can finish before the load listener is attached.
                            let script = format!(
                                "const el = document.getElementById({mounted_id:?}); return !!el && el.complete && el.naturalWidth > 0;"
                            );
                            let picture = src_mounted.clone();
                            spawn(async move {
                                let cached = document::eval(&script).await.ok().and_then(|value| value.as_bool());
                                if cached == Some(true) {
                                    loaded.set(picture);
                                }
                            });
                        },
                        onload: move |_| loaded.set(src_load.clone()),
                        onerror: move |_| failed.set(src_error.clone()),
                    }
                }
            }
            if let Some(state) = status {
                span { class: "ui-avt__status", "data-status": state.as_str(), aria_hidden: "true" }
            }
        }
    }
}

/// A row of avatars, each cut out of the one before by a ring, ending in +N.
/// Only the first `max` members render; `total` counts members not passed in.
#[component]
pub fn AvatarGroup(
    #[props(default = 4)] max: usize,
    #[props(default)] size: AvatarSize,
    #[props(default)] total: Option<usize>,
    #[props(default)] avatars: Vec<AvatarItem>,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let limit = max.max(1);
    let all = avatars.len().max(total.unwrap_or(0));
    let shown: Vec<AvatarItem> = avatars.into_iter().take(limit).collect();
    let more = all - shown.len();
    let last_form = match shown.last() {
        Some(item) => form_of(item.shape, item.kind),
        None => "circle",
    };
    let more_text = if more > 99 {
        "99+".to_string()
    } else {
        format!("+{more}")
    };
    let more_label = format!("{} more", group_thousands(more as i64));
    let class = class_list("ui-avt__group", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            role: "group",
            class,
            "data-size": size.as_str(),
            ..attributes,
            for (idx, item) in shown.into_iter().enumerate() {
                Avatar {
                    key: "{idx}",
                    name: item.name,
                    src: item.src,
                    size,
                    shape: item.shape,
                    status: item.status,
                    kind: item.kind,
                    seed: item.seed,
                    color: item.color,
                    working: item.working,
                }
            }
            if more > 0 {
                span { class: "ui-avt__more", "data-shape": last_form, role: "img", aria_label: "{more_label}", "{more_text}" }
            }
        }
    }
}

fn form_of(shape: Option<AvatarShape>, kind: AvatarKind) -> &'static str {
    match shape {
        Some(AvatarShape::Circle) => "circle",
        Some(AvatarShape::Square) => "square",
        None if kind == AvatarKind::Agent => "square",
        None => "circle",
    }
}

/// One of eight hues, the same for the same name or id.
pub fn hue_of(key: &str) -> u8 {
    let mut hash: u32 = 7;
    for ch in key.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(ch as u32);
    }
    (hash % 8) as u8
}

/// The 3×3 mark of an agent, mirrored left to right, drawn from its id. Never
/// fewer than five squares on.
pub fn mark_cells(id: &str) -> [bool; 9] {
    let mut hash: u32 = 0;
    let mut unit = [0u16; 2];
    for ch in id.chars() {
        let first = ch.encode_utf16(&mut unit)[0] as u32;
        hash = hash.wrapping_mul(31).wrapping_add(first);
    }
    for step in 0..64u32 {
        let bits = (hash >> (step % 26)) & 63;
        let mut cells = [false; 9];
        for row in 0..3 {
            let left = (bits >> (row * 2)) & 1 == 1;
            let right = (bits >> (row * 2 + 1)) & 1 == 1;
            cells[row * 3] = left;
            cells[row * 3 + 1] = right;
            cells[row * 3 + 2] = left;
        }
        if cells.iter().filter(|on| **on).count() >= 5 {
            return cells;
        }
        // The multiply runs in doubles, as in the original: the product passes 2^53,
        // and the rounding it takes is part of which mark an id gets.
        hash = ((hash as f64) * 1_103_515_245.0 + 12_345.0).rem_euclid(4_294_967_296.0) as u32;
    }
    [true, false, true, false, true, false, true, false, true]
}

/// Initials from the first and last word: "Mara Lindqvist" → ML, "Cher" → C,
/// "🦊 Fox Team" → 🦊, "Christopher Montgomery III" → CM.
pub fn initials_of(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let Some(first_word) = words.first() else {
        return String::new();
    };
    let lead = clusters(first_word).into_iter().next().unwrap_or_default();
    if is_emoji_cluster(&lead) && !is_letter_cluster(&lead) {
        return lead;
    }
    let last = words.len() - 1;
    let firsts: Vec<String> = words
        .iter()
        .enumerate()
        .filter(|(idx, word)| !(*idx > 0 && *idx == last && is_suffix(word)))
        .filter_map(|(_, word)| {
            clusters(word)
                .into_iter()
                .find(|cluster| is_letter_cluster(cluster))
        })
        .collect();
    let Some(head) = firsts.first() else {
        return lead;
    };
    let pair = if firsts.len() > 1 {
        format!("{head}{}", firsts[firsts.len() - 1])
    } else {
        head.clone()
    };
    pair.to_uppercase()
}

fn is_suffix(word: &str) -> bool {
    matches!(
        word.trim_end_matches('.').to_lowercase().as_str(),
        "jr" | "sr" | "ii" | "iii" | "iv" | "vi" | "phd" | "md" | "esq"
    )
}

fn is_letter_cluster(cluster: &str) -> bool {
    cluster.chars().any(char::is_alphanumeric)
}

fn is_emoji_cluster(cluster: &str) -> bool {
    cluster.chars().any(|ch| {
        matches!(
            ch as u32,
            0x2300..=0x23FF | 0x2600..=0x27BF | 0x2B00..=0x2BFF | 0x1F000..=0x1FAFF
        )
    })
}

/// A cluster is one character plus whatever attaches to it: combining marks,
/// variation selectors, skin tones, and the character after a zero-width joiner.
fn clusters(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut join_next = false;
    for ch in text.chars() {
        let attaches = join_next
            || matches!(
                ch as u32,
                0x0300..=0x036F | 0xFE0E | 0xFE0F | 0x200D | 0x1F3FB..=0x1F3FF
            );
        match out.last_mut() {
            Some(last) if attaches => last.push(ch),
            _ => out.push(ch.to_string()),
        }
        join_next = ch == '\u{200D}';
    }
    out
}
