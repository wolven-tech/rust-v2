//! The second half of `/kit`: dialogs, toasts and the record-keeping
//! instructions — stamps, gate lists, redlines, choice cards, comparisons.
//!
//! A coverage fixture like the rest of the page: each instruction is mounted
//! with real props and live state, so a removed prop stops this compiling.

use dioxus::prelude::*;
use rv2_ui::{
    ArmedDeleteButton, Badge, BadgeFill, Button, Card, Choice, ChoiceGroup, CompareRow,
    CompareTable, ConfirmDialog, Container, Dialog, DialogKind, Eyebrow, Gate, GateList, Grid,
    Heading, Pad, Press, PressButton, ProcessRail, ProcessStep, RailLayout, Redline, RedlineDiff,
    Section, Size, Stamp, Surface, Text, TextField, ToastProvider, Toaster, Tone, Variant, Verdict,
    use_toast,
};

#[component]
pub fn KitRecords() -> Element {
    rsx! {
        ToastProvider {
            Section { id: "decisions",
                Container {
                    Eyebrow { "Default theme" }
                    Heading { "Asking and telling" }
                    Text { class: "mt-2", tone: Tone::Muted,
                        "A native modal dialog, a drawer, a destructive confirm, a two-click delete and toasts that wait while you read them."
                    }
                    Overlays {}
                }
            }
            Section { id: "records",
                Container {
                    Heading { "Records and choices" }
                    Records {}
                }
            }
            Toaster {}
        }
    }
}

#[component]
fn Overlays() -> Element {
    let toast = use_toast();
    let mut dialog = use_signal(|| false);
    let mut drawer = use_signal(|| false);
    let mut confirm = use_signal(|| false);
    let mut armed = use_signal(|| false);
    let mut rows = use_signal(|| 3_u32);
    let mut note = use_signal(String::new);

    rsx! {
        div { class: "mt-6 flex flex-wrap gap-3",
            Button { onclick: move |_| dialog.set(true), "Open dialog" }
            Button { variant: Variant::Outline, onclick: move |_| drawer.set(true), "Open drawer" }
            Button { variant: Variant::Danger, onclick: move |_| confirm.set(true), "Delete project…" }
            ArmedDeleteButton {
                label: format!("Remove row ({})", rows()),
                confirm_label: "Click again to remove",
                armed: armed(),
                disabled: rows() == 0,
                on_armed_change: move |next| armed.set(next),
                on_confirm: move |_| {
                    rows.with_mut(|n| *n = n.saturating_sub(1));
                    toast.raise(Verdict::Clear, "Row removed.");
                },
            }
        }
        div { class: "mt-4 flex flex-wrap gap-2",
            Button { variant: Variant::Ghost, size: Size::Small, onclick: move |_| { toast.raise(Verdict::Note, "Draft saved locally."); }, "Note toast" }
            Button { variant: Variant::Ghost, size: Size::Small, onclick: move |_| { toast.raise_titled(Verdict::Caution, "Evidence missing", "3 of 14 sessions have no meter photo."); }, "Caution toast" }
            Button { variant: Variant::Ghost, size: Size::Small, onclick: move |_| { toast.raise_titled(Verdict::Stop, "Upload failed", "The file is over 10 MB."); }, "Stop toast" }
        }

        Dialog {
            id: "kit-dialog",
            open: dialog(),
            title: "Rename this scope",
            description: "The client sees the new name on the next receipt.",
            on_open_change: move |next| dialog.set(next),
            TextField { label: "Name", value: note(), oninput: move |e: FormEvent| note.set(e.value()) }
            div { class: "mt-4 flex justify-end",
                Button { onclick: move |_| dialog.set(false), "Save" }
            }
        }
        Dialog {
            id: "kit-drawer",
            open: drawer(),
            title: "Session detail",
            kind: DialogKind::Drawer,
            on_open_change: move |next| drawer.set(next),
            GateList {
                label: "Session evidence",
                gates: vec![
                    Gate::new("Meter photo", Verdict::Clear),
                    Gate::new("Receipt", Verdict::Caution).detail("Requested on 18 Nov"),
                ],
            }
        }
        ConfirmDialog {
            id: "kit-confirm",
            open: confirm(),
            title: "Delete this project?",
            description: "Its 14 sessions and every receipt go with it. This cannot be undone.",
            confirm_label: "Delete project",
            on_open_change: move |next| confirm.set(next),
            on_confirm: move |_| {
                confirm.set(false);
                toast.raise(Verdict::Stop, "Project deleted.");
            },
        }
    }
}

#[component]
fn Records() -> Element {
    let mut plan = use_signal(|| "split".to_string());
    let plan_cards = vec![
        Choice::new("split", "Split the bill").hint("Each household pays its own sessions."),
        Choice::new("share", "Share evenly").hint("One total, divided by households."),
        Choice::new("host", "Host pays").hint("Settle it later, outside the app."),
    ];

    rsx! {
        Grid { columns: 2, class: "mt-6 items-start gap-8",
            div { class: "space-y-6",
                div { class: "flex flex-wrap items-center gap-3",
                    Stamp { verdict: Verdict::Caution, "Open" }
                    Stamp { verdict: Verdict::Clear, "Ready" }
                    Stamp { verdict: Verdict::Stop, "Void" }
                    Stamp { "Draft" }
                }
                div { class: "flex items-center gap-2",
                    Heading { level: 3, "Queued for review" }
                    Badge { fill: BadgeFill::Tint, "12" }
                }
                GateList {
                    label: "Handoff readiness",
                    gates: vec![
                        Gate::new("Vector master supplied", Verdict::Clear),
                        Gate::new("Colour values confirmed", Verdict::Clear).detail("Pantone 2945 C"),
                        Gate::new("Usage rights signed", Verdict::Caution).detail("Waiting on the client"),
                        Gate::new("Favicon exported", Verdict::Stop).detail("Source is under 512px"),
                        Gate::new("Print proof", Verdict::Note),
                    ],
                }
                RedlineDiff {
                    lines: vec![
                        Redline::new("Hours", "10", "12"),
                        Redline::new("Scope", "Two logo concepts", "Three logo concepts and one revision round"),
                    ],
                }
            }
            div { class: "space-y-6",
                ChoiceGroup {
                    legend: "How the cost is split",
                    name: "kit-plan",
                    choices: plan_cards,
                    value: plan(),
                    onchange: move |next| plan.set(next),
                }
                Card { surface: Surface::Slab, pad: Pad::Roomy,
                    Heading { level: 3, "Printed slab" }
                    Text { class: "mt-2", tone: Tone::Muted, "Surface::Slab, beside the offset press it was drawn with." }
                    PressButton { press: Press::Offset, class: "mt-4", "Send the card" }
                }
                ProcessRail {
                    label: "Handoff timeline",
                    layout: RailLayout::Column,
                    active_step: Some(2),
                    steps: vec![
                        ProcessStep::new("Brief", "Signed 12 Nov"),
                        ProcessStep::new("Assets", "Two open"),
                        ProcessStep::new("Handoff", "Card sent to client"),
                    ],
                }
            }
        }
        CompareTable {
            class: "mt-10",
            caption: "Plans compared",
            columns: vec!["Feature".to_string(), "Free".to_string(), "Studio".to_string(), "Agency".to_string()],
            highlight: Some(1),
            rows: vec![
                CompareRow::new("Projects", vec!["1".to_string(), "Unlimited".to_string(), "Unlimited".to_string()]),
                CompareRow::new("Receipts", vec!["Watermarked".to_string(), "Branded".to_string(), "Branded".to_string()]),
                CompareRow::new("Seats", vec!["1".to_string(), "3".to_string(), "10".to_string()]),
                CompareRow::new("Price", vec!["£0".to_string(), "£12/mo".to_string(), "£39/mo".to_string()]),
            ],
        }
    }
}
