//! The A2UI catalog as the kit sees it: which instructions an agent may place,
//! and under which wire props (`docs/architecture/004-a2ui-surface.md`).
//!
//! The catalog is compiled in from `rv2-a2ui`, so this is the table the parser
//! checks rather than a copy of it.

use std::fmt::Write as _;

use rv2_a2ui::catalog::{COMPONENTS, ComponentSpec, PropKind};

pub use rv2_a2ui::CATALOG_ID;

use crate::index::{Index, Prop};

/// Catalog entries the renderer draws itself, with no kit instruction behind
/// them.
pub const LAYOUT_ONLY: &[&str] = &["Column"];

/// The catalog entry for an instruction, if an agent may place it.
#[must_use]
pub fn entry(name: &str) -> Option<&'static ComponentSpec> {
    COMPONENTS
        .iter()
        .find(|c| c.name == name && !LAYOUT_ONLY.contains(&c.name))
}

/// An entry's wire props in the index's own shape.
#[must_use]
pub fn props(spec: &ComponentSpec) -> Vec<Prop> {
    spec.props
        .iter()
        .map(|p| Prop {
            name: p.name.to_string(),
            ty: wire_type(p.kind),
            required: p.required,
            default: None,
            docs: p.doc.to_string(),
        })
        .collect()
}

/// What a prop accepts, as an agent author writes it.
#[must_use]
pub fn wire_type(kind: PropKind) -> String {
    match kind {
        PropKind::Text => "string | {path}".into(),
        PropKind::Number => "number | {path}".into(),
        PropKind::Int { min, max } => format!("integer {min}–{max}"),
        PropKind::Share => "number 0–1".into(),
        PropKind::Choice(values) => values.join(" / "),
        PropKind::Children => "[component id]".into(),
        PropKind::Bound => "{path}".into(),
        PropKind::Action => "{event: {name, context}}".into(),
        PropKind::Stats => "[{label, value, note?, lean?}]".into(),
    }
}

/// Catalog entries that name no instruction in `index`.
#[must_use]
pub fn problems(index: &Index) -> Vec<String> {
    COMPONENTS
        .iter()
        .filter(|c| !LAYOUT_ONLY.contains(&c.name))
        .filter(|c| !index.component(c.name).is_some_and(|k| k.in_isa))
        .map(|c| {
            format!(
                "{CATALOG_ID} places `{}`, which is not an rv2-ui instruction; \
                 the catalog only grows (A2UI4), so restore the instruction",
                c.name
            )
        })
        .collect()
}

/// Every catalog entry with the instruction it renders.
#[must_use]
pub fn table() -> String {
    let mut out = String::from("| Component | Renders | Props |\n|---|---|---|\n");
    for spec in COMPONENTS {
        let renders = if LAYOUT_ONLY.contains(&spec.name) {
            "a layout `div`".to_string()
        } else {
            format!("[`{}`](#{})", spec.name, spec.name.to_lowercase())
        };
        let names: Vec<String> = spec
            .props
            .iter()
            .map(|p| {
                if p.required {
                    format!("`{}`", p.name)
                } else {
                    format!("`{}`?", p.name)
                }
            })
            .collect();
        let _ = writeln!(
            out,
            "| `{}` | {renders} | {} |",
            spec.name,
            names.join(", ")
        );
    }
    out
}
