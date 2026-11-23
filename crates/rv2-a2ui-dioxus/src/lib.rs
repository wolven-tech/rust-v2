//! Renders an rv2 A2UI surface with `rv2-ui` instructions.
//!
//! **Layer 2, WASM-safe.** [`A2uiSurface`] reads a surface out of a
//! `Signal<Surfaces>` the app owns, so the app applies server messages and
//! this crate only draws them. Inputs write to the surface's data model as the
//! user types; a button hands an [`ActionRequest`] to `on_action`, and the app
//! stamps and sends it (`rv2-client`), because nothing here has a clock.
//!
//! Each catalog operand maps to a kit operand by an exhaustive `match` below,
//! which is where a catalog addition fails to compile until it has a look
//! (A2UI3 in `docs/architecture/004-a2ui-surface.md`).

#![forbid(unsafe_code)]
#![allow(non_snake_case)] // Dioxus components are PascalCase by convention.

use dioxus::prelude::*;
use rv2_a2ui::catalog::{self, Node, Operand};
use rv2_a2ui::{ActionRequest, Dyn, Surfaces};
use serde_json::Value;

/// One surface, rendered from `root`. Until `root` arrives it shows a
/// skeleton, which is v0.9.1's "nothing to render yet".
#[component]
pub fn A2uiSurface(
    surfaces: Signal<Surfaces>,
    surface_id: String,
    on_action: EventHandler<ActionRequest>,
) -> Element {
    let ready = surfaces
        .read()
        .get(&surface_id)
        .is_some_and(rv2_a2ui::Surface::has_root);
    if !ready {
        return rsx! { rv2_ui::Skeleton { lines: 3 } };
    }
    rsx! {
        div { "data-a2ui-surface": "{surface_id}",
            NodeView { surfaces, surface_id: surface_id.clone(), id: "root", on_action }
        }
    }
}

#[component]
fn NodeView(
    surfaces: Signal<Surfaces>,
    surface_id: String,
    id: String,
    on_action: EventHandler<ActionRequest>,
) -> Element {
    let (node, data) = {
        let guard = surfaces.read();
        let Some(surface) = guard.get(&surface_id) else {
            return rsx! {};
        };
        let Some(node) = surface.node(&id) else {
            return rsx! { rv2_ui::Skeleton { lines: 1 } };
        };
        (node.clone(), surface.data().clone())
    };
    let text = |value: &Dyn<String>| value.resolve(&data);
    let maybe = |value: &Option<Dyn<String>>| value.as_ref().map(|v| v.resolve(&data));
    let kids = |ids: &[String]| {
        rsx! {
            for child in ids.iter().cloned() {
                NodeView { key: "{child}", surfaces, surface_id: surface_id.clone(), id: child, on_action }
            }
        }
    };

    match node {
        Node::Column { children } => rsx! {
            div { class: "flex flex-col gap-4", {kids(&children)} }
        },
        Node::Grid { children, columns } => rsx! {
            rv2_ui::Grid { columns, {kids(&children)} }
        },
        Node::Card {
            children,
            pad,
            surface,
        } => rsx! {
            rv2_ui::Card { pad: map_pad(pad), surface: map_surface(surface), class: "flex flex-col gap-4",
                {kids(&children)}
            }
        },
        Node::Heading {
            text: heading,
            level,
            size,
        } => rsx! {
            rv2_ui::Heading { level, size: map_heading(size), "{text(&heading)}" }
        },
        Node::Text { text: body, tone } => rsx! {
            rv2_ui::Text { tone: map_tone(tone), "{text(&body)}" }
        },
        Node::Callout {
            text: body,
            title,
            verdict,
        } => rsx! {
            rv2_ui::Callout { verdict: map_verdict(verdict), title: maybe(&title), "{text(&body)}" }
        },
        Node::Stamp {
            text: word,
            verdict,
        } => rsx! {
            rv2_ui::Stamp { verdict: map_verdict(verdict), "{text(&word)}" }
        },
        Node::StatStrip { stats, density } => {
            let stats = stats
                .iter()
                .map(|stat| rv2_ui::Stat {
                    label: text(&stat.label),
                    value: text(&stat.value),
                    note: maybe(&stat.note),
                    lean: map_lean(stat.lean),
                })
                .collect();
            rsx! { rv2_ui::StatStrip { stats, density: map_density(density) } }
        }
        Node::ResultPanel {
            label,
            value,
            context,
            lean,
        } => rsx! {
            rv2_ui::ResultPanel {
                label: text(&label),
                value: text(&value),
                context: maybe(&context),
                lean: map_lean(lean),
            }
        },
        Node::Meter {
            label,
            value,
            max,
            readout,
            warn_at,
            critical_at,
        } => rsx! {
            rv2_ui::Meter {
                label: text(&label),
                value: value.resolve(&data),
                max: max.resolve(&data),
                readout: text(&readout),
                warn_at,
                critical_at,
            }
        },
        Node::TextField {
            label,
            value,
            placeholder,
            help,
            input_type,
        } => {
            let current = Dyn::<String>::Bound(value.clone()).resolve(&data);
            let path = value.path().to_owned();
            let target = surface_id.clone();
            rsx! {
                rv2_ui::TextField {
                    id: field_id(&surface_id, &id),
                    label: text(&label),
                    value: current,
                    placeholder: maybe(&placeholder).unwrap_or_default(),
                    help: maybe(&help),
                    r#type: input_type.wire(),
                    oninput: move |event: FormEvent| write(surfaces, &target, &path, event.value()),
                }
            }
        }
        Node::UnitField {
            label,
            value,
            prefix,
            suffix,
            help,
        } => {
            let current = Dyn::<String>::Bound(value.clone()).resolve(&data);
            let path = value.path().to_owned();
            let target = surface_id.clone();
            rsx! {
                rv2_ui::UnitField {
                    id: field_id(&surface_id, &id),
                    label: text(&label),
                    value: current,
                    prefix: maybe(&prefix),
                    suffix: maybe(&suffix),
                    help: maybe(&help),
                    oninput: move |event: FormEvent| write(surfaces, &target, &path, event.value()),
                }
            }
        }
        Node::Button {
            label,
            variant,
            action,
        } => {
            let disabled = action.is_none();
            let source = id.clone();
            rsx! {
                rv2_ui::Button {
                    variant: map_variant(variant),
                    disabled,
                    onclick: move |_| {
                        let Some(action) = &action else { return };
                        // Built before the call so no read guard is alive when
                        // the app's handler writes the same signal.
                        let request = surfaces
                            .read()
                            .get(&surface_id)
                            .map(|s| action.request(&surface_id, &source, s.data()));
                        if let Some(request) = request {
                            on_action.call(request);
                        }
                    },
                    "{text(&label)}"
                }
            }
        }
    }
}

/// A write fails only for an array index the model does not have; the field
/// then keeps its previous value, which is visible rather than silent.
fn write(mut surfaces: Signal<Surfaces>, surface_id: &str, path: &str, typed: String) {
    if let Some(surface) = surfaces.write().get_mut(surface_id) {
        let _ = surface.set_value(path, Value::String(typed));
    }
}

/// An HTML id for a field's label to point at. Component ids are any string;
/// an id with a space would break the `for` association.
fn field_id(surface_id: &str, id: &str) -> String {
    format!("a2ui-{surface_id}-{id}")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

fn map_pad(pad: catalog::Pad) -> rv2_ui::Pad {
    match pad {
        catalog::Pad::Default => rv2_ui::Pad::Default,
        catalog::Pad::None => rv2_ui::Pad::None,
        catalog::Pad::Roomy => rv2_ui::Pad::Roomy,
    }
}

fn map_surface(surface: catalog::Surface) -> rv2_ui::Surface {
    match surface {
        catalog::Surface::Flat => rv2_ui::Surface::Flat,
        catalog::Surface::Slab => rv2_ui::Surface::Slab,
    }
}

fn map_heading(size: catalog::HeadingSize) -> rv2_ui::HeadingSize {
    match size {
        catalog::HeadingSize::Section => rv2_ui::HeadingSize::Section,
        catalog::HeadingSize::Display => rv2_ui::HeadingSize::Display,
        catalog::HeadingSize::Title => rv2_ui::HeadingSize::Title,
        catalog::HeadingSize::Card => rv2_ui::HeadingSize::Card,
    }
}

fn map_tone(tone: catalog::Tone) -> rv2_ui::Tone {
    match tone {
        catalog::Tone::Default => rv2_ui::Tone::Default,
        catalog::Tone::Muted => rv2_ui::Tone::Muted,
        catalog::Tone::Small => rv2_ui::Tone::Small,
        catalog::Tone::Lead => rv2_ui::Tone::Lead,
    }
}

fn map_verdict(verdict: catalog::Verdict) -> rv2_ui::Verdict {
    match verdict {
        catalog::Verdict::Note => rv2_ui::Verdict::Note,
        catalog::Verdict::Clear => rv2_ui::Verdict::Clear,
        catalog::Verdict::Caution => rv2_ui::Verdict::Caution,
        catalog::Verdict::Stop => rv2_ui::Verdict::Stop,
    }
}

fn map_density(density: catalog::Density) -> rv2_ui::Density {
    match density {
        catalog::Density::Compact => rv2_ui::Density::Compact,
        catalog::Density::Hero => rv2_ui::Density::Hero,
    }
}

fn map_lean(lean: catalog::Lean) -> rv2_ui::Lean {
    match lean {
        catalog::Lean::Neutral => rv2_ui::Lean::Neutral,
        catalog::Lean::Good => rv2_ui::Lean::Good,
        catalog::Lean::Close => rv2_ui::Lean::Close,
        catalog::Lean::Bad => rv2_ui::Lean::Bad,
    }
}

fn map_variant(variant: catalog::Variant) -> rv2_ui::Variant {
    match variant {
        catalog::Variant::Primary => rv2_ui::Variant::Primary,
        catalog::Variant::Secondary => rv2_ui::Variant::Secondary,
        catalog::Variant::Outline => rv2_ui::Variant::Outline,
        catalog::Variant::Danger => rv2_ui::Variant::Danger,
        catalog::Variant::Ghost => rv2_ui::Variant::Ghost,
    }
}
