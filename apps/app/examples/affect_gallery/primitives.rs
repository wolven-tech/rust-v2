use dioxus::prelude::*;
use rv2_affect::avatar::{
    Avatar, AvatarGroup, AvatarItem, AvatarKind, AvatarShape, AvatarSize, AvatarStatus,
};
use rv2_affect::badge::{Badge, BadgeSize, BadgeTone};
use rv2_affect::button::{Button, ButtonSize, ButtonVariant};
use rv2_affect::card::{Card, CardContent, CardElement, CardFooter, CardHeader};
use rv2_affect::checkbox::{Checkbox, CheckboxVariant};
use rv2_affect::empty_state::EmptyState;
use rv2_affect::form_field::{FormField, FormFieldControl};
use rv2_affect::icon_button::{IconButton, IconButtonSize, IconButtonVariant};
use rv2_affect::kbd::{Kbd, KbdKeys, KbdPlatform};
use rv2_affect::segmented_control::{SegmentOption, SegmentSize, SegmentedControl};
use rv2_affect::separator::{Separator, SeparatorOrientation};
use rv2_affect::skeleton::{Loading, Skeleton};
use rv2_affect::spinner::Spinner;
use rv2_affect::switch::{Switch, SwitchPlacement, SwitchSize};

use crate::gallery::Ex;
use crate::{Item, Section, glyph};

const PLUS: &str = "M8 3v10M3 8h10";
const TRASH: &str = "M3 5h10M6 5V3h4v2M5 5l1 9h4l1-9";
const CHECK: &str = "M3 8.5l3 3 7-7";
const ARROW: &str = "M3 8h9M9 4l4 4-4 4";
const SEARCH: &str = "M7 12a5 5 0 1 0 0-10 5 5 0 0 0 0 10zM14 14l-3.5-3.5";
const SLIDERS: &str = "M2.5 4.5h11M2.5 11.5h11M5.5 2.5v4M10.5 9.5v4";
const PIN: &str = "M6 2.5h4l-.5 4 2 2v1H4.5v-1l2-2zM8 9.5v4";
const MORE: &str = "M3.5 8h.01M8 8h.01M12.5 8h.01";
const COPY: &str = "M5.5 5.5h7v8h-7zM3.5 10.5v-8h7";
const THUMB_UP: &str = "M4.5 14V7.5M4.5 7.5 7 2c1 0 1.5.6 1.3 1.8L7.8 6h4.6c.9 0 1.5.8 1.3 1.6l-1.2 4.8c-.2.7-.7 1.1-1.4 1.1H4.5";
const REGENERATE: &str = "M13 8a5 5 0 1 1-1.5-3.5M13 2.5v3h-3";
/// Keys as space-separated names, split into one chip each.
const SHORTCUTS: [(&str, &str); 6] = [
    ("Search runs and files", "mod K"),
    ("New task", "C"),
    ("Run the agent", "mod enter"),
    ("Stop the run", "mod ."),
    ("Approve and continue", "mod shift A"),
    ("Close the panel", "escape"),
];
const PORTRAIT: &str = "data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20viewBox='0%200%2040%2040'%3E%3Crect width='40' height='40' fill='%23e9d8cc'/%3E%3Ccircle cx='20' cy='16' r='7' fill='%237a4e3a'/%3E%3Cpath d='M6 40c2-9 8-13 14-13s12 4 14 13z' fill='%237a4e3a'/%3E%3C/svg%3E";

fn platform_of(mac: bool) -> KbdPlatform {
    if mac {
        KbdPlatform::Mac
    } else {
        KbdPlatform::Other
    }
}

fn members() -> Vec<AvatarItem> {
    ["Ana Ruiz", "Ben Ortiz", "Cy Lee", "Dee Park", "Eve Moss"]
        .into_iter()
        .map(|name| AvatarItem {
            name: name.to_string(),
            ..Default::default()
        })
        .collect()
}

#[component]
pub fn Primitives() -> Element {
    let mut clicks = use_signal(|| 0usize);
    let mut loading = use_signal(|| false);
    let mut selected = use_signal(|| false);
    let mut agree = use_signal(|| false);
    let mut all = use_signal(|| false);
    let mut notify = use_signal(|| true);
    let mut icon_pressed = use_signal(|| false);
    let mut icon_clicks = use_signal(|| 0usize);
    let mut filter_on = use_signal(|| true);
    let mut liked = use_signal(|| false);
    let mut mac_keys = use_signal(|| true);

    rsx! {
        Section { id: "primitives", title: "Primitives",
            Item { name: "Avatar",
                Ex { label: "Presence",
                    Avatar { name: "Mara Lindqvist", status: AvatarStatus::Busy }
                    Avatar { name: "Ben Ortiz", status: AvatarStatus::Online }
                    Avatar { name: "Cy Lee", status: AvatarStatus::Away }
                }
                Ex { label: "Sizes",
                    Avatar { name: "Dee Park", size: AvatarSize::S20 }
                    Avatar { name: "Dee Park", size: AvatarSize::S24 }
                    Avatar { name: "Dee Park", size: AvatarSize::S32 }
                    Avatar { name: "Dee Park", size: AvatarSize::S40 }
                }
                Ex { label: "Square and agent",
                    Avatar { name: "Eve Moss", shape: AvatarShape::Square }
                    Avatar { name: "Scout", kind: AvatarKind::Agent, working: true, color: "#0060e2" }
                    Avatar { name: "Scout idle", kind: AvatarKind::Agent, color: "#0060e2" }
                }
                Ex { label: "Member list",
                    div { class: "gl-stack", style: "max-width: none; gap: 12px;",
                        for (name, role, status) in [("Mara Lindqvist", "Design lead", AvatarStatus::Online), ("Jonas Weber", "Reviewing PR #482", AvatarStatus::Away), ("Priya Raman", "In a call until 15:30", AvatarStatus::Busy)] {
                            div { style: "display: flex; align-items: center; gap: 10px; font-size: 13px;",
                                Avatar { name: name, src: PORTRAIT, status: status }
                                div { style: "display: grid;",
                                    span { "{name}" }
                                    span { style: "font-size: 12px; opacity: 0.6;", "{role}" }
                                }
                            }
                        }
                        div { style: "display: flex; align-items: center; gap: 10px; font-size: 13px;",
                            Avatar { name: "Christopher Montgomery III" }
                            div { style: "display: grid;",
                                span { "Christopher Montgomery III" }
                                span { style: "font-size: 12px; opacity: 0.6;", "Joined yesterday" }
                            }
                        }
                        div { style: "display: flex; align-items: center; gap: 10px; font-size: 13px;",
                            Avatar { name: "Triage agent", kind: AvatarKind::Agent, working: true, color: "#0060e2" }
                            div { style: "display: grid;",
                                span { "Triage agent" }
                                span { style: "font-size: 12px; opacity: 0.6;", "Writing the summary for 14 tickets" }
                            }
                        }
                    }
                }
                Ex { label: "Group, with overflow count",
                    AvatarGroup { avatars: members(), max: 3, total: 6usize }
                }
            }
            Item { name: "AvatarGroup",
                Ex { label: "Team, capped at three",
                    AvatarGroup { avatars: members(), max: 3 }
                }
                Ex { label: "Small, with total",
                    AvatarGroup { avatars: members(), max: 3, size: AvatarSize::S24, total: 12usize }
                }
            }
            Item { name: "Badge",
                Ex { label: "Tones",
                    Badge { tone: BadgeTone::Neutral, "Draft" }
                    Badge { tone: BadgeTone::Info, live: true, "Running" }
                    Badge { tone: BadgeTone::Success, "Shipped" }
                    Badge { tone: BadgeTone::Warning, "Stale" }
                    Badge { tone: BadgeTone::Danger, "Failed" }
                    Badge { tone: BadgeTone::Purple, "Beta" }
                }
                Ex { label: "Sizes and dots",
                    Badge { tone: BadgeTone::Success, size: BadgeSize::Sm, dot: true, "Synced" }
                    Badge { tone: BadgeTone::Success, size: BadgeSize::Md, dot: true, "Synced" }
                    Badge { tone: BadgeTone::Warning, size: BadgeSize::Md, icon: glyph(CHECK), "Needs review" }
                }
                Ex { label: "Counts",
                    Badge { count: 7 }
                    Badge { count: 120 }
                }
            }
            Item { name: "Button",
                Ex { label: "Variants",
                    Button { variant: ButtonVariant::Primary, label: "Save", loading: loading(), onclick: move |_| clicks += 1 }
                    Button { variant: ButtonVariant::Secondary, label: "Secondary" }
                    Button { variant: ButtonVariant::Ghost, label: "Ghost" }
                    Button { variant: ButtonVariant::Destructive, label: "Delete", icon: glyph(TRASH) }
                }
                Ex { label: "Sizes",
                    Button { variant: ButtonVariant::Primary, label: "Small", size: ButtonSize::Sm }
                    Button { variant: ButtonVariant::Primary, label: "Medium", size: ButtonSize::Md }
                    Button { variant: ButtonVariant::Primary, label: "Large", size: ButtonSize::Lg }
                }
                Ex { label: "With icons",
                    Button { variant: ButtonVariant::Secondary, label: "Add seat", icon: glyph(PLUS) }
                    Button { variant: ButtonVariant::Secondary, label: "Continue", icon_right: glyph(ARROW) }
                    Button { label: "Add", icon: glyph(PLUS), aria_label: "Add item" }
                }
                Ex { label: "Disabled and loading",
                    Button { variant: ButtonVariant::Primary, label: "Send invoice", disabled: true }
                    Button { variant: ButtonVariant::Secondary, label: "Export", disabled: true }
                    Button { variant: ButtonVariant::Primary, label: "Save", loading: loading() }
                    Button { variant: ButtonVariant::Secondary, label: "Toggle loading", onclick: move |_| loading.set(!loading()) }
                }
                p { class: "gl-out", "Save clicks: {clicks}. Loading: {loading}" }
            }
            Item { name: "Card",
                Ex { label: "Agent card, with icon, metrics and footer action",
                    div { class: "gl-box",
                        Card {
                            element: CardElement::Section,
                            footer: rsx! {
                                CardFooter {
                                    span { "Running · started 2 min ago" }
                                    Button { variant: ButtonVariant::Secondary, label: "Open", size: ButtonSize::Sm }
                                }
                            },
                            CardHeader {
                                title: rsx! { "Support triage" },
                                description: rsx! { "Sorts new tickets and drafts the first reply" },
                                icon: rsx! { Avatar { name: "Support triage", kind: AvatarKind::Agent, color: "#0060e2" } },
                                action: rsx! { IconButton { icon: glyph(MORE), label: "More actions", variant: IconButtonVariant::Ghost } },
                            }
                            CardContent {
                                div { class: "gl-metrics",
                                    for (label, value) in [("Tickets", "142"), ("Resolved", "96%"), ("Spent today", "$4.20")] {
                                        div {
                                            span { class: "gl-metric-k", "{label}" }
                                            div { class: "gl-metric-v", "{value}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ex { label: "Replies, one selected",
                    div { class: "gl-box",
                        Card {
                            element: CardElement::Section,
                            interactive: true,
                            selected: selected(),
                            onclick: move |_| selected.set(true),
                            CardHeader { title: rsx! { "Draft only" }, description: rsx! { "You send every reply" } }
                        }
                    }
                    div { class: "gl-box",
                        Card {
                            element: CardElement::Section,
                            interactive: true,
                            selected: !selected(),
                            onclick: move |_| selected.set(false),
                            CardHeader { title: rsx! { "Send for me" }, description: rsx! { "Asks before refunds" } }
                        }
                    }
                }
                p { class: "gl-out", "Selected: {selected}" }
            }
            Item { name: "Checkbox",
                Ex { label: "Plain",
                    Checkbox {
                        label: rsx! { "Accept terms" },
                        checked: agree(),
                        onchange: move |e: FormEvent| agree.set(e.checked()),
                    }
                    Checkbox { label: rsx! { "Send weekly digest" }, checked: true }
                    Checkbox { label: rsx! { "Locked by admin" }, checked: true, disabled: true }
                }
                Ex { label: "Nested, parent indeterminate",
                    div { class: "gl-box",
                        Checkbox {
                            label: rsx! { "Email me about" },
                            indeterminate: true,
                            children: rsx! {
                                Checkbox {
                                    label: rsx! { "Mentions" },
                                    description: rsx! { "Someone mentions you in a chat or a file" },
                                    checked: true,
                                }
                                Checkbox {
                                    label: rsx! { "Approvals" },
                                    description: rsx! { "An agent waits for your OK before it acts" },
                                }
                                Checkbox { label: rsx! { "Weekly summary" } }
                            },
                        }
                    }
                }
                Ex { label: "Card, checked and indeterminate",
                    div { class: "gl-stack", style: "max-width: none;",
                        Checkbox {
                            variant: CheckboxVariant::Card,
                            label: rsx! { "Keep a copy of every run" },
                            description: rsx! { "Inputs and outputs stay for 30 days, so you can replay a run." },
                            checked: true,
                        }
                        Checkbox {
                            variant: CheckboxVariant::Card,
                            label: rsx! { "Select all" },
                            description: rsx! { "Every row in this table" },
                            indeterminate: agree() && !all(),
                            checked: all(),
                            onchange: move |e: FormEvent| all.set(e.checked()),
                        }
                    }
                }
                Ex { label: "Invalid",
                    Checkbox {
                        label: rsx! { "I agree to the terms of service" },
                        error: rsx! { "Agree to the terms to continue." },
                    }
                }
                p { class: "gl-out", "Accepted: {agree}. Select all: {all}" }
            }
            Item { name: "EmptyState",
                Ex { label: "With action",
                    div { class: "gl-box",
                        EmptyState {
                            title: rsx! { "No schedules yet" },
                            text: rsx! { "Add one to start" },
                            action: rsx! { Button { variant: ButtonVariant::Primary, label: "Create", size: ButtonSize::Sm } },
                        }
                    }
                }
                Ex { label: "Text only",
                    div { class: "gl-box",
                        EmptyState {
                            title: rsx! { "Nothing matches" },
                            text: rsx! { "Try a shorter search or clear the filters." },
                        }
                    }
                }
            }
            Item { name: "FormField",
                Ex { label: "Required, with hint and counter",
                    div { class: "gl-box",
                        FormField {
                            label: rsx! { "Name" },
                            required: true,
                            hint: rsx! { "As on your passport" },
                            counter: rsx! { "4/40" },
                            html_for: "gl-name",
                            render: |control: FormFieldControl| {
                                let id = control.id.clone();
                                rsx! { input { class: "gl-input", id: "{id}", value: "Ada", aria_describedby: control.described_by } }
                            },
                        }
                    }
                }
                Ex { label: "Optional",
                    div { class: "gl-box",
                        FormField {
                            label: rsx! { "Company" },
                            hint: rsx! { "Shown on invoices" },
                            optional: true,
                            html_for: "gl-company",
                            render: |control: FormFieldControl| {
                                let id = control.id.clone();
                                rsx! { input { class: "gl-input", id: "{id}", placeholder: "Northwind Traders" } }
                            },
                        }
                    }
                }
                Ex { label: "Error",
                    div { class: "gl-box",
                        FormField {
                            label: rsx! { "Email" },
                            hint: rsx! { "Work address" },
                            error: rsx! { "Enter an email" },
                            html_for: "gl-email",
                            render: |control: FormFieldControl| {
                                let id = control.id.clone();
                                rsx! { input { class: "gl-input", id: "{id}", aria_invalid: control.invalid.then_some("true") } }
                            },
                        }
                    }
                }
            }
            Item { name: "IconButton",
                Ex { label: "Variants",
                    IconButton { icon: glyph(PLUS), label: "Add" }
                    IconButton {
                        icon: glyph(CHECK),
                        label: "Bold",
                        variant: IconButtonVariant::Secondary,
                        pressed: icon_pressed(),
                        onclick: move |_| icon_pressed.set(!icon_pressed()),
                    }
                    IconButton {
                        icon: glyph(TRASH),
                        label: "Delete",
                        variant: IconButtonVariant::Primary,
                        onclick: move |_| icon_clicks += 1,
                    }
                }
                Ex { label: "Sizes",
                    IconButton { icon: glyph(PLUS), label: "Small add", size: IconButtonSize::Sm, variant: IconButtonVariant::Secondary }
                    IconButton { icon: glyph(PLUS), label: "Medium add", size: IconButtonSize::Md, variant: IconButtonVariant::Secondary }
                }
                Ex { label: "Toolbar, one pressed",
                    div { class: "gl-box",
                        div { class: "gl-toolbar", style: "margin-top: 0;",
                            span { style: "display: flex; gap: 8px; align-items: baseline; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase;",
                                "Runs"
                                span { style: "opacity: 0.55;", "24" }
                            }
                            div { class: "gl-toolbar-tools",
                                IconButton { icon: glyph(SEARCH), label: "Search runs" }
                                IconButton {
                                    icon: glyph(SLIDERS),
                                    label: "Filter runs",
                                    pressed: filter_on(),
                                    onclick: move |_| filter_on.set(!filter_on()),
                                }
                                IconButton { icon: glyph(PIN), label: "Pin runs" }
                                IconButton { icon: glyph(PLUS), label: "New run", variant: IconButtonVariant::Primary }
                            }
                        }
                    }
                }
                Ex { label: "In a message, one pressed",
                    div { class: "gl-box",
                        div { style: "display: grid; gap: 10px;",
                            p { style: "margin: 0; font-size: 13px;", "Summarised 14 support tickets. Three mention the export timeout, so I drafted a fix in billing-api." }
                            div { class: "gl-row", style: "gap: 4px;",
                                IconButton { icon: glyph(COPY), label: "Copy" }
                                IconButton {
                                    icon: glyph(THUMB_UP),
                                    label: "Helpful",
                                    pressed: liked(),
                                    onclick: move |_| liked.set(!liked()),
                                }
                                IconButton { icon: glyph(REGENERATE), label: "Regenerate" }
                                IconButton { icon: glyph(MORE), label: "More actions" }
                            }
                        }
                    }
                }
                p { class: "gl-out", "Bold pressed: {icon_pressed}. Delete clicks: {icon_clicks}. Filter: {filter_on}. Helpful: {liked}" }
            }
            Item { name: "Kbd",
                Ex { label: "Shortcut list, switchable platform",
                    div { class: "gl-box",
                        div { class: "gl-shortcuts",
                            div { class: "gl-shortcuts-head",
                                span { class: "gl-shortcuts-title", "Keyboard shortcuts" }
                                SegmentedControl {
                                    size: SegmentSize::Sm,
                                    aria_label: "Keycap platform",
                                    value: if mac_keys() { "mac".to_string() } else { "windows".to_string() },
                                    on_change: move |next: String| mac_keys.set(next == "mac"),
                                    options: vec![
                                        SegmentOption { value: "mac".into(), label: Some(rsx! { "macOS" }), ..Default::default() },
                                        SegmentOption { value: "windows".into(), label: Some(rsx! { "Windows" }), ..Default::default() },
                                    ],
                                }
                            }
                            for (label, keys) in SHORTCUTS {
                                div { class: "gl-shortcut-row",
                                    span { "{label}" }
                                    Kbd { keys: KbdKeys(keys.split(' ').map(str::to_string).collect()), platform: platform_of(mac_keys()) }
                                }
                            }
                            div { class: "gl-shortcuts-foot",
                                "Press "
                                Kbd { keys: "?", platform: platform_of(mac_keys()) }
                                " anytime to open this list"
                            }
                        }
                    }
                }
                Ex { label: "Keycaps by platform",
                    Kbd { keys: ["mod", "K"], platform: KbdPlatform::Mac }
                    Kbd { keys: ["mod", "K"], platform: KbdPlatform::Other }
                    Kbd { keys: ["shift", "escape"] }
                }
            }
            Item { name: "Separator",
                Ex { label: "Horizontal, plain and labelled",
                    div { class: "gl-box",
                        Separator {}
                        Separator { label: rsx! { "or" } }
                    }
                }
                Ex { label: "Vertical, in a row",
                    div { class: "gl-row", style: "height: 40px",
                        span { "Projects" }
                        Separator { orientation: SeparatorOrientation::Vertical, decorative: false }
                        span { "Members" }
                    }
                }
            }
            Item { name: "Skeleton and Loading",
                Ex { label: "Text lines",
                    Loading { label: "Loading runs", class: "gl-box",
                        Skeleton { line: true, width: "40%" }
                        Skeleton { line: true, width: "80%" }
                    }
                }
                Ex { label: "List rows",
                    Loading { label: "Loading recent runs", class: "gl-box",
                        for _ in 0..4 {
                            div { style: "display: flex; align-items: center; gap: 12px; padding: 12px 0; border-bottom: 1px solid #0000000f;",
                                Skeleton { width: "16px", height: "16px", radius: "4px" }
                                div { style: "display: grid; gap: 6px; flex: 1; min-width: 0;",
                                    Skeleton { line: true, width: "62%" }
                                    Skeleton { line: true, width: "38%" }
                                }
                                Skeleton { width: "24px", height: "8px", radius: "999px" }
                            }
                        }
                    }
                }
                Ex { label: "Card placeholder",
                    Loading { label: "Loading deal",
                        div { class: "gl-row",
                            Skeleton { width: "32px", height: "32px", radius: "999px" }
                            div { class: "gl-stack",
                                Skeleton { line: true, width: "120px" }
                                Skeleton { line: true, width: "180px" }
                            }
                        }
                    }
                }
                Ex { label: "Block",
                    Skeleton { width: "120px", height: "48px" }
                }
            }
            Item { name: "Spinner",
                Ex { label: "Sizes",
                    Spinner { size: 12u32 }
                    Spinner { size: 14u32 }
                    Spinner { size: 16u32 }
                    Spinner { size: 20u32 }
                }
                Ex { label: "With label",
                    div { class: "gl-row", style: "align-items: center; font-size: 13px;",
                        Spinner { label: "Saving" }
                        span { "Saving changes" }
                    }
                }
                Ex { label: "In a running list",
                    div { class: "gl-box",
                        for (name, done) in [("Type check", true), ("Lint", true), ("Unit tests", false), ("Eval suite", false)] {
                            div { style: "display: flex; justify-content: space-between; align-items: center; min-height: 34px; font-size: 13px; border-bottom: 1px solid #0000000f;",
                                span { "{name}" }
                                if done {
                                    span { class: "gl-tag", "done" }
                                } else {
                                    Spinner { size: 12u32, label: "{name} running" }
                                }
                            }
                        }
                    }
                }
            }
            Item { name: "Switch",
                Ex { label: "Placement, with description",
                    div { class: "gl-box",
                        Switch {
                            label: rsx! { "Notify" },
                            description: rsx! { "Email me when a run finishes" },
                            checked: notify(),
                            onchange: move |e: FormEvent| notify.set(e.checked()),
                        }
                        Switch {
                            label: rsx! { "Compact rows" },
                            placement: SwitchPlacement::End,
                            size: SwitchSize::Sm,
                        }
                    }
                }
                Ex { label: "States",
                    Switch { label: rsx! { "Off" } }
                    Switch { label: rsx! { "On" }, checked: true }
                    Switch { label: rsx! { "Locked" }, disabled: true, checked: true }
                }
                p { class: "gl-out", "Notify: {notify}" }
            }
        }
    }
}
