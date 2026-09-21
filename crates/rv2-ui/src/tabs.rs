//! A tab list, controlled by its caller.
//!
//! The visual design — the two variants, the underline drawn as an `::after`
//! rather than a border, and the orientation groups — is adapted from the
//! Rust/UI registry (MIT, <https://github.com/rust-ui/ui>). Three things are
//! deliberately different here.
//!
//! **It is controlled.** The upstream component owns the active value in its
//! own signal. A page whose view lives in the URL would then have two sources
//! of truth, and the one the reader can see — the address bar — would be the
//! one that loses. [`Tabs`] takes `value` and hands back `onselect`.
//!
//! **There is no `TabsContent`.** Upstream renders every panel and hides the
//! inactive ones with `hidden`. For a page whose panels each read the whole
//! register that is two thirds of the work thrown away on every load, so the
//! caller keeps its own dispatch and renders one.
//!
//! **It carries the ARIA the pattern requires.** Upstream ships `data-state`
//! and no roles, so a screen reader is told nothing: not that this is a tab
//! list, not which tab is current. `role`, `aria-selected`, a roving tabindex
//! and the arrow keys are added here, per the WAI-ARIA tabs pattern.

use dioxus::prelude::*;

/// How a tab list draws the current tab.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum TabsVariant {
    /// A raised pill on a filled track.
    #[default]
    Pill,
    /// A rule under the current tab, on no track at all.
    Line,
}

/// One selectable tab.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Tab {
    pub value: String,
    pub label: String,
}

impl Tab {
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// A tab list whose current value belongs to the caller.
///
/// `label` names the group for a screen reader. Without it the list announces
/// as an unlabelled set of tabs, which says what the control is but not what it
/// switches.
///
/// `controls` is the `id` of the element the tabs switch, and it is required
/// rather than optional: a tab list referencing no panel announces as a set of
/// buttons that change nothing. The caller marks that element `role="tabpanel"`
/// and points its `aria-labelledby` at `tab-{value}`.
#[component]
pub fn Tabs(
    tabs: Vec<Tab>,
    value: String,
    label: String,
    controls: String,
    onselect: EventHandler<String>,
    #[props(default)] variant: TabsVariant,
    #[props(default)] class: String,
) -> Element {
    let line = variant == TabsVariant::Line;
    let track = if line {
        "gap-1 bg-transparent p-0"
    } else {
        "bg-muted rounded-lg p-[3px]"
    };

    // The roving tabindex needs the index of the current tab, and a value no
    // tab carries — a URL naming a view that no longer exists — must still
    // leave one stop reachable.
    let current = tabs.iter().position(|t| t.value == value).unwrap_or(0);

    rsx! {
        div {
            class: "inline-flex w-fit items-center justify-center text-sm text-muted-foreground {track} {class}",
            role: "tablist",
            aria_label: "{label}",
            for (index , tab) in tabs.iter().enumerate() {
                TabsTrigger {
                    key: "{tab.value}",
                    tab: tab.clone(),
                    selected: index == current,
                    focusable: index == current,
                    line,
                    controls: controls.clone(),
                    onselect,
                    onmove: {
                        let tabs = tabs.clone();
                        move |delta: TabStep| {
                            let last = tabs.len() - 1;
                            let next = match delta {
                                TabStep::Previous => current.saturating_sub(1),
                                TabStep::Next => (current + 1).min(last),
                                TabStep::First => 0,
                                TabStep::Last => last,
                            };
                            if next != current {
                                onselect.call(tabs[next].value.clone());
                            }
                        }
                    },
                }
            }
        }
    }
}

/// Which way an arrow key moves the selection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TabStep {
    Previous,
    Next,
    First,
    Last,
}

#[component]
fn TabsTrigger(
    tab: Tab,
    selected: bool,
    focusable: bool,
    line: bool,
    controls: String,
    onselect: EventHandler<String>,
    onmove: EventHandler<TabStep>,
) -> Element {
    // The current tab is the filled control, not a lighter shade of the track.
    // Upstream fills it with `background`, which on a light theme lifts the tab
    // out of a grey track — and on a dark one is darker than the track, so the
    // current tab reads as sunken and the least emphatic thing in the row.
    // `primary` is the pair that inverts with the theme instead of assuming
    // one.
    let tone = match (selected, line) {
        (true, false) => "bg-primary text-primary-foreground border-transparent",
        (true, true) => "text-foreground border-transparent",
        (false, _) => "text-foreground/60 hover:text-foreground border-transparent",
    };
    // The rule is an `::after` rather than a bottom border so that turning it
    // on and off cannot change the trigger's height and shift the row.
    let rule = if line {
        if selected {
            "after:absolute after:inset-x-0 after:-bottom-px after:h-0.5 after:bg-foreground"
        } else {
            "after:absolute after:inset-x-0 after:-bottom-px after:h-0.5 after:bg-transparent"
        }
    } else {
        ""
    };

    let value = tab.value.clone();

    rsx! {
        button {
            r#type: "button",
            class: "relative inline-flex flex-1 cursor-pointer items-center justify-center gap-1.5 rounded-md border px-3 py-1.5 font-medium whitespace-nowrap select-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none {tone} {rule}",
            role: "tab",
            id: "tab-{tab.value}",
            aria_selected: "{selected}",
            aria_controls: "{controls}",
            tabindex: if focusable { "0" } else { "-1" },
            onclick: move |_| onselect.call(value.clone()),
            onkeydown: move |event| {
                let step = match event.key() {
                    Key::ArrowLeft | Key::ArrowUp => Some(TabStep::Previous),
                    Key::ArrowRight | Key::ArrowDown => Some(TabStep::Next),
                    Key::Home => Some(TabStep::First),
                    Key::End => Some(TabStep::Last),
                    _ => None,
                };
                if let Some(step) = step {
                    event.prevent_default();
                    onmove.call(step);
                }
            },
            "{tab.label}"
        }
    }
}
