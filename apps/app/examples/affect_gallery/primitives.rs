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
use rv2_affect::kbd::{Kbd, KbdPlatform};
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
                Ex { label: "Default, with footer",
                    Card {
                        element: CardElement::Section,
                        footer: rsx! { CardFooter { span { "Saved 2 min ago" } } },
                        CardHeader { title: rsx! { "Q3 plan" }, description: rsx! { "Owned by Ada" } }
                        CardContent { p { "Renewal terms for Northwind, due Friday." } }
                    }
                }
                Ex { label: "Interactive, toggle selected",
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
                Ex { label: "Card, indeterminate",
                    Checkbox {
                        variant: CheckboxVariant::Card,
                        label: rsx! { "Select all" },
                        description: rsx! { "Every row in this table" },
                        indeterminate: agree() && !all(),
                        checked: all(),
                        onchange: move |e: FormEvent| all.set(e.checked()),
                    }
                }
                Ex { label: "Invalid",
                    Checkbox {
                        label: rsx! { "Required consent" },
                        error: rsx! { "Required" },
                        invalid: true,
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
                p { class: "gl-out", "Bold pressed: {icon_pressed}. Delete clicks: {icon_clicks}" }
            }
            Item { name: "Kbd",
                Ex { label: "Mac",
                    Kbd { keys: ["mod", "K"], platform: KbdPlatform::Mac }
                }
                Ex { label: "Windows and Linux",
                    Kbd { keys: ["mod", "K"], platform: KbdPlatform::Other }
                }
                Ex { label: "Sequence",
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
                    Loading { label: "Loading runs",
                        div { class: "gl-box",
                            Skeleton { line: true, width: "40%" }
                            Skeleton { line: true, width: "80%" }
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
                    Spinner { size: 14u32 }
                    Spinner { size: 20u32 }
                    Spinner { size: 28u32 }
                }
                Ex { label: "With label",
                    Spinner { label: "Saving" }
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
