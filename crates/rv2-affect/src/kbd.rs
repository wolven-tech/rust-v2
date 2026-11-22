//! Kbd: a key chip for shortcuts in menus, tooltips and help lists. One key
//! (`esc`) or a combo in one chip (`["mod", "K"]`). Screen readers hear the keys
//! spelled out; the glyphs are hidden from them.
//!
//! Port losses: the platform is detected once on mount through `document::eval`,
//! so the first client frame after hydration may draw the other platform's keys.
//! The server render always draws the non-Mac keys.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/kbd.css");
const IS_MAC: &str = "const nav = navigator; return /mac|iphone|ipad|ipod/i.test(nav.userAgentData?.platform || nav.platform || nav.userAgent || '');";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KbdSize {
    /// 16px high, 10.5px text: in menus and hints.
    #[default]
    Sm,
    /// 20px high, 12px text: in lists and help.
    Md,
}

impl KbdSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KbdPlatform {
    Mac,
    Other,
}

impl KbdPlatform {
    fn as_str(self) -> &'static str {
        match self {
            Self::Mac => "mac",
            Self::Other => "other",
        }
    }
}

/// The keys of one chip, in order. Converts from `"esc"` or `["mod", "K"]`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KbdKeys(pub Vec<String>);

impl From<&str> for KbdKeys {
    fn from(key: &str) -> Self {
        Self(vec![key.to_string()])
    }
}

impl<const N: usize> From<[&str; N]> for KbdKeys {
    fn from(keys: [&str; N]) -> Self {
        Self(keys.iter().map(|key| key.to_string()).collect())
    }
}

/// A key chip. With no `keys` the children show as one plain key.
#[component]
pub fn Kbd(
    #[props(default, into)] keys: KbdKeys,
    #[props(default)] size: KbdSize,
    #[props(default)] platform: Option<KbdPlatform>,
    #[props(default)] children: Option<Element>,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = kbd)] attributes: Vec<Attribute>,
) -> Element {
    let mut detected = use_signal(|| KbdPlatform::Other);
    use_effect(move || {
        spawn(async move {
            if let Ok(value) = document::eval(IS_MAC).await {
                detected.set(if value.as_bool() == Some(true) {
                    KbdPlatform::Mac
                } else {
                    KbdPlatform::Other
                });
            }
        });
    });

    let os = platform.unwrap_or(detected());
    let parts: Vec<KeyPart> = keys
        .0
        .iter()
        .filter(|key| !key.trim().is_empty())
        .map(|key| describe_key(key, os))
        .collect();
    if parts.is_empty() && children.is_none() {
        return rsx! {};
    }
    let spoken = parts
        .iter()
        .map(|part| part.spoken.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let has_keys = !parts.is_empty();
    let class = class_list("ui-kbdc", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        kbd {
            class,
            "data-size": size.as_str(),
            "data-os": os.as_str(),
            ..attributes,
            if !has_keys {
                {children}
            }
            if has_keys {
                span { class: "ui-kbdc__keys", aria_hidden: "true",
                    for (idx, part) in parts.iter().enumerate() {
                        span { key: "{idx}", class: "ui-kbdc__key", "data-glyph": part.glyph.then_some(""), {part.label.clone()} }
                    }
                }
                span { class: "ui-kbdc__sr", "{spoken}" }
            }
        }
    }
}

/// One drawn key: the glyph or word for the eye, and the name for screen readers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyPart {
    pub label: String,
    pub spoken: String,
    /// A symbol such as ⌘, drawn a little larger than letters.
    pub glyph: bool,
}

struct Named {
    id: &'static str,
    mac: &'static str,
    other: &'static str,
    spoken: (&'static str, &'static str),
    glyph: bool,
}

const NAMED: &[Named] = &[
    Named {
        id: "mod",
        mac: "⌘",
        other: "Ctrl",
        spoken: ("Command", "Control"),
        glyph: true,
    },
    Named {
        id: "cmd",
        mac: "⌘",
        other: "⌘",
        spoken: ("Command", "Command"),
        glyph: true,
    },
    Named {
        id: "ctrl",
        mac: "⌃",
        other: "Ctrl",
        spoken: ("Control", "Control"),
        glyph: true,
    },
    Named {
        id: "alt",
        mac: "⌥",
        other: "Alt",
        spoken: ("Option", "Alt"),
        glyph: true,
    },
    Named {
        id: "shift",
        mac: "⇧",
        other: "Shift",
        spoken: ("Shift", "Shift"),
        glyph: true,
    },
    Named {
        id: "enter",
        mac: "↵",
        other: "Enter",
        spoken: ("Return", "Enter"),
        glyph: true,
    },
    Named {
        id: "esc",
        mac: "Esc",
        other: "Esc",
        spoken: ("Escape", "Escape"),
        glyph: false,
    },
    Named {
        id: "tab",
        mac: "Tab",
        other: "Tab",
        spoken: ("Tab", "Tab"),
        glyph: false,
    },
    Named {
        id: "space",
        mac: "Space",
        other: "Space",
        spoken: ("Space", "Space"),
        glyph: false,
    },
    Named {
        id: "backspace",
        mac: "⌫",
        other: "Backspace",
        spoken: ("Delete", "Backspace"),
        glyph: true,
    },
    Named {
        id: "delete",
        mac: "⌦",
        other: "Del",
        spoken: ("Forward delete", "Delete"),
        glyph: true,
    },
    Named {
        id: "up",
        mac: "↑",
        other: "↑",
        spoken: ("Up arrow", "Up arrow"),
        glyph: true,
    },
    Named {
        id: "down",
        mac: "↓",
        other: "↓",
        spoken: ("Down arrow", "Down arrow"),
        glyph: true,
    },
    Named {
        id: "left",
        mac: "←",
        other: "←",
        spoken: ("Left arrow", "Left arrow"),
        glyph: true,
    },
    Named {
        id: "right",
        mac: "→",
        other: "→",
        spoken: ("Right arrow", "Right arrow"),
        glyph: true,
    },
    Named {
        id: "pageup",
        mac: "PgUp",
        other: "PgUp",
        spoken: ("Page up", "Page up"),
        glyph: false,
    },
    Named {
        id: "pagedown",
        mac: "PgDn",
        other: "PgDn",
        spoken: ("Page down", "Page down"),
        glyph: false,
    },
    Named {
        id: "home",
        mac: "Home",
        other: "Home",
        spoken: ("Home", "Home"),
        glyph: false,
    },
    Named {
        id: "end",
        mac: "End",
        other: "End",
        spoken: ("End", "End"),
        glyph: false,
    },
];

/// Other spellings people use for the same key.
const ALIASES: &[(&str, &str)] = &[
    ("command", "cmd"),
    ("meta", "cmd"),
    ("⌘", "cmd"),
    ("control", "ctrl"),
    ("option", "alt"),
    ("opt", "alt"),
    ("return", "enter"),
    ("escape", "esc"),
    ("del", "delete"),
    ("arrowup", "up"),
    ("arrowdown", "down"),
    ("arrowleft", "left"),
    ("arrowright", "right"),
    ("pgup", "pageup"),
    ("pgdn", "pagedown"),
];

/// Spoken names for punctuation a screen reader might skip.
const SPOKEN_PUNCTUATION: &[(&str, &str)] = &[
    (".", "Period"),
    (",", "Comma"),
    ("/", "Slash"),
    ("\\", "Backslash"),
    ("?", "Question mark"),
    ("[", "Left bracket"),
    ("]", "Right bracket"),
    ("-", "Minus"),
    ("+", "Plus"),
    ("=", "Equals"),
    (";", "Semicolon"),
    ("'", "Quote"),
    ("`", "Backtick"),
];

/// Resolves one key name: a named key, an alias, a single letter (capitalised)
/// or any other text (shown as it is).
pub fn describe_key(raw: &str, platform: KbdPlatform) -> KeyPart {
    let key = raw.trim();
    let lower = key.to_lowercase();
    let id = ALIASES
        .iter()
        .find(|(alias, _)| *alias == lower)
        .map(|(_, id)| *id)
        .unwrap_or(lower.as_str());
    if let Some(named) = NAMED.iter().find(|named| named.id == id) {
        let mac = platform == KbdPlatform::Mac;
        let label = if mac { named.mac } else { named.other };
        let spoken = if mac { named.spoken.0 } else { named.spoken.1 };
        return KeyPart {
            label: label.to_string(),
            spoken: spoken.to_string(),
            glyph: named.glyph && label.chars().count() == 1,
        };
    }
    let label = if key.chars().count() == 1 {
        key.to_uppercase()
    } else {
        key.to_string()
    };
    let spoken = SPOKEN_PUNCTUATION
        .iter()
        .find(|(mark, _)| *mark == key)
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| label.clone());
    KeyPart {
        label,
        spoken,
        glyph: false,
    }
}
