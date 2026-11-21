use dioxus::prelude::*;
use rv2_affect::avatar::{Avatar, AvatarGroup, AvatarItem, AvatarKind, AvatarSize, AvatarStatus};
use rv2_affect::badge::{Badge, BadgeTone};
use rv2_affect::button::{Button, ButtonSize, ButtonVariant};
use rv2_affect::card::{Card, CardContent, CardElement, CardFooter, CardHeader};
use rv2_affect::checkbox::{Checkbox, CheckboxVariant};
use rv2_affect::empty_state::EmptyState;
use rv2_affect::form_field::{FormField, FormFieldControl};
use rv2_affect::icon_button::{IconButton, IconButtonVariant};
use rv2_affect::kbd::{Kbd, KbdPlatform};
use rv2_affect::separator::{Separator, SeparatorOrientation};
use rv2_affect::skeleton::{Loading, Skeleton};
use rv2_affect::spinner::Spinner;
use rv2_affect::switch::{Switch, SwitchPlacement, SwitchSize};

use crate::{Item, Section, glyph};

const PLUS: &str = "M8 3v10M3 8h10";
const TRASH: &str = "M3 5h10M6 5V3h4v2M5 5l1 9h4l1-9";
const CHECK: &str = "M3 8.5l3 3 7-7";

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

    rsx! {
        Section { id: "primitives", title: "Primitives",
            Item { name: "Avatar",
                div { class: "gl-row",
                    Avatar { name: "Mara Lindqvist", status: AvatarStatus::Busy }
                    Avatar { name: "Ben Ortiz", status: AvatarStatus::Online, size: AvatarSize::S40 }
                    Avatar { name: "Cy Lee", status: AvatarStatus::Away, size: AvatarSize::S24 }
                    Avatar { name: "Scout", kind: AvatarKind::Agent, working: true, color: "#0060e2" }
                    Avatar { name: "Scout idle", kind: AvatarKind::Agent, color: "#0060e2" }
                }
            }
            Item { name: "AvatarGroup",
                AvatarGroup { avatars: members(), max: 3 }
            }
            Item { name: "Badge",
                div { class: "gl-row",
                    Badge { tone: BadgeTone::Neutral, "Draft" }
                    Badge { tone: BadgeTone::Info, live: true, "Running" }
                    Badge { tone: BadgeTone::Success, "Shipped" }
                    Badge { tone: BadgeTone::Warning, "Stale" }
                    Badge { tone: BadgeTone::Danger, "Failed" }
                    Badge { tone: BadgeTone::Purple, "Beta" }
                    Badge { count: 120 }
                    Badge { count: 7 }
                }
            }
            Item { name: "Button",
                div { class: "gl-row",
                    Button { variant: ButtonVariant::Primary, label: "Save", loading: loading(), onclick: move |_| clicks += 1 }
                    Button { variant: ButtonVariant::Secondary, label: "Secondary", size: ButtonSize::Sm }
                    Button { variant: ButtonVariant::Ghost, label: "Ghost", size: ButtonSize::Lg }
                    Button { variant: ButtonVariant::Destructive, label: "Delete", icon: glyph(TRASH) }
                    Button { label: "Add", icon: glyph(PLUS), aria_label: "Add item" }
                    Button { variant: ButtonVariant::Secondary, label: "Toggle loading", onclick: move |_| loading.set(!loading()) }
                }
                p { class: "gl-out", "Save clicks: {clicks}. Loading: {loading}" }
            }
            Item { name: "Card",
                Card {
                    element: CardElement::Section,
                    interactive: true,
                    selected: selected(),
                    footer: rsx! { CardFooter { span { "Saved 2 min ago" } } },
                    CardHeader { title: rsx! { "Q3 plan" }, description: rsx! { "Owned by Ada" } }
                    CardContent {
                        Button { variant: ButtonVariant::Secondary, label: "Toggle selected", onclick: move |_| selected.set(!selected()) }
                    }
                }
                p { class: "gl-out", "Selected: {selected}" }
            }
            Item { name: "Checkbox",
                div { class: "gl-stack",
                    Checkbox {
                        label: rsx! { "Accept terms" },
                        checked: agree(),
                        onchange: move |e: FormEvent| agree.set(e.checked()),
                    }
                    Checkbox {
                        variant: CheckboxVariant::Card,
                        label: rsx! { "Select all" },
                        description: rsx! { "Every row in this table" },
                        indeterminate: agree() && !all(),
                        checked: all(),
                        onchange: move |e: FormEvent| all.set(e.checked()),
                    }
                    Checkbox {
                        label: rsx! { "Required consent" },
                        error: rsx! { "Required" },
                        invalid: true,
                    }
                }
                p { class: "gl-out", "Accepted: {agree}. Select all: {all}" }
            }
            Item { name: "EmptyState",
                div { class: "gl-box",
                    EmptyState {
                        title: rsx! { "No schedules yet" },
                        text: rsx! { "Add one to start" },
                        action: rsx! { Button { variant: ButtonVariant::Primary, label: "Create", size: ButtonSize::Sm } },
                    }
                }
            }
            Item { name: "FormField",
                div { class: "gl-stack",
                    FormField {
                        label: rsx! { "Name" },
                        required: true,
                        hint: rsx! { "As on your passport" },
                        counter: rsx! { "4/40" },
                        html_for: "gl-name",
                        render: |control: FormFieldControl| {
                            let id = control.id.clone();
                            rsx! { input { id: "{id}", value: "Ada", aria_describedby: control.described_by } }
                        },
                    }
                    FormField {
                        label: rsx! { "Email" },
                        hint: rsx! { "Work address" },
                        error: rsx! { "Enter an email" },
                        optional: true,
                        html_for: "gl-email",
                        render: |control: FormFieldControl| {
                            let id = control.id.clone();
                            rsx! { input { id: "{id}", aria_invalid: control.invalid.then_some("true") } }
                        },
                    }
                }
            }
            Item { name: "IconButton",
                div { class: "gl-row",
                    IconButton { icon: glyph(PLUS), label: "Close" }
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
                p { class: "gl-out", "Bold pressed: {icon_pressed}. Delete clicks: {icon_clicks}" }
            }
            Item { name: "Kbd",
                div { class: "gl-row",
                    Kbd { keys: ["mod", "K"], platform: KbdPlatform::Mac }
                    Kbd { keys: ["mod", "K"], platform: KbdPlatform::Other }
                    Kbd { keys: ["shift", "escape"] }
                }
            }
            Item { name: "Separator",
                div { class: "gl-stack",
                    Separator {}
                    Separator { label: rsx! { "or" } }
                    div { class: "gl-row", style: "height: 40px",
                        span { "Left" }
                        Separator { orientation: SeparatorOrientation::Vertical, decorative: false }
                        span { "Right" }
                    }
                }
            }
            Item { name: "Skeleton and Loading",
                Loading { label: "Loading runs",
                    div { class: "gl-stack",
                        Skeleton { line: true, width: "40%" }
                        Skeleton { line: true, width: "80%" }
                        Skeleton { width: "120px", height: "48px" }
                    }
                }
            }
            Item { name: "Spinner",
                div { class: "gl-row",
                    Spinner { label: "Saving" }
                    Spinner { size: 20u32 }
                }
            }
            Item { name: "Switch",
                div { class: "gl-stack",
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
                    Switch {
                        label: rsx! { "Locked" },
                        disabled: true,
                        checked: true,
                    }
                }
                p { class: "gl-out", "Notify: {notify}" }
            }
            Item { name: "Toolbar button sizes",
                div { class: "gl-row",
                    Button { label: "Small", size: ButtonSize::Sm }
                    Button { label: "Medium", size: ButtonSize::Md }
                    Button { label: "Large", size: ButtonSize::Lg }
                }
            }
        }
    }
}
