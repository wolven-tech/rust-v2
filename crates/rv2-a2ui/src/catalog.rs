//! The rv2 v1 catalog: what an agent may place on a surface.
//!
//! [`COMPONENTS`] is the one table. The parser checks every message against
//! it, and [`catalog_document`] publishes it as JSON Schema, so the document an
//! agent author reads cannot describe a prop the parser rejects (A2UI4).
//!
//! Operands are lowercase strings on the wire and enums here. They are not
//! `rv2-ui`'s enums: this catalog only grows, the kit's operands may be
//! renamed, and the renderer's exhaustive `match` is the join (A2UI3).

use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::value::{Action, Binding, ContextValue, Dyn};
use crate::wire::ComponentDef;

/// The `catalogId` a surface must name to be rendered by this crate.
pub const CATALOG_ID: &str = "rv2:a2ui/catalog/v1";

/// A wire operand: a closed set of lowercase strings.
pub trait Operand: Sized + Copy + Default {
    const WIRE: &'static [&'static str];
    fn from_wire(text: &str) -> Option<Self>;
    fn wire(self) -> &'static str;
}

/// The first variant is the default, as it is for the kit's operand.
macro_rules! operand {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $wire:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }

        impl Default for $name {
            fn default() -> Self {
                [$(Self::$variant),+][0]
            }
        }

        impl Operand for $name {
            const WIRE: &'static [&'static str] = &[$($wire),+];

            fn from_wire(text: &str) -> Option<Self> {
                match text {
                    $($wire => Some(Self::$variant),)+
                    _ => None,
                }
            }

            fn wire(self) -> &'static str {
                match self {
                    $(Self::$variant => $wire),+
                }
            }
        }
    };
}

operand!(Pad { Default = "default", None = "none", Roomy = "roomy" });
operand!(Surface { Flat = "flat", Slab = "slab" });
operand!(HeadingSize { Section = "section", Display = "display", Title = "title", Card = "card" });
operand!(Tone { Default = "default", Muted = "muted", Small = "small", Lead = "lead" });
operand!(Verdict { Note = "note", Clear = "clear", Caution = "caution", Stop = "stop" });
operand!(Density { Compact = "compact", Hero = "hero" });
operand!(Lean { Neutral = "neutral", Good = "good", Close = "close", Bad = "bad" });
operand!(InputType { Text = "text", Email = "email", Number = "number" });
operand!(Variant {
    Primary = "primary",
    Secondary = "secondary",
    Outline = "outline",
    Danger = "danger",
    Ghost = "ghost",
});

/// What a prop accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropKind {
    /// A string, or a binding.
    Text,
    /// A number, or a binding.
    Number,
    /// A literal integer in range.
    Int { min: i64, max: i64 },
    /// A literal number from 0 to 1: a share of a maximum.
    Share,
    /// One of a closed set of strings.
    Choice(&'static [&'static str]),
    /// An array of component ids.
    Children,
    /// A binding only: the data-model path an input writes to.
    Bound,
    /// `{"event": {"name", "context"}}`.
    Action,
    /// An array of `{label, value, note?, lean?}`.
    Stats,
}

#[derive(Debug, Clone, Copy)]
pub struct PropSpec {
    pub name: &'static str,
    pub kind: PropKind,
    pub required: bool,
    pub doc: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentSpec {
    pub name: &'static str,
    pub doc: &'static str,
    pub props: &'static [PropSpec],
}

const fn req(name: &'static str, kind: PropKind, doc: &'static str) -> PropSpec {
    PropSpec {
        name,
        kind,
        required: true,
        doc,
    }
}

const fn opt(name: &'static str, kind: PropKind, doc: &'static str) -> PropSpec {
    PropSpec {
        name,
        kind,
        required: false,
        doc,
    }
}

const CHILDREN: PropSpec = req(
    "children",
    PropKind::Children,
    "Ids of the components inside, in order.",
);

pub static COMPONENTS: &[ComponentSpec] = &[
    ComponentSpec {
        name: "Column",
        doc: "Stacks its children vertically. Layout only.",
        props: &[CHILDREN],
    },
    ComponentSpec {
        name: "Grid",
        doc: "Lays its children out in equal columns, one column on a narrow screen.",
        props: &[
            CHILDREN,
            opt(
                "columns",
                PropKind::Int { min: 1, max: 4 },
                "Columns at full width. Default 3.",
            ),
        ],
    },
    ComponentSpec {
        name: "Card",
        doc: "A bordered surface around its children.",
        props: &[
            CHILDREN,
            opt("pad", PropKind::Choice(Pad::WIRE), "Inner spacing."),
            opt(
                "surface",
                PropKind::Choice(Surface::WIRE),
                "flat: hairline border. slab: ink border and offset shadow.",
            ),
        ],
    },
    ComponentSpec {
        name: "Heading",
        doc: "A heading. Level sets the outline, size sets the look.",
        props: &[
            req("text", PropKind::Text, "The heading."),
            opt(
                "level",
                PropKind::Int { min: 1, max: 4 },
                "h1 to h4. Default 2.",
            ),
            opt("size", PropKind::Choice(HeadingSize::WIRE), "Visual size."),
        ],
    },
    ComponentSpec {
        name: "Text",
        doc: "A paragraph.",
        props: &[
            req("text", PropKind::Text, "The paragraph."),
            opt("tone", PropKind::Choice(Tone::WIRE), "Size and emphasis."),
        ],
    },
    ComponentSpec {
        name: "Callout",
        doc: "A verdict strip: the title says the verdict in words, the edge repeats it.",
        props: &[
            req("text", PropKind::Text, "The body."),
            opt("title", PropKind::Text, "The verdict in words."),
            opt(
                "verdict",
                PropKind::Choice(Verdict::WIRE),
                "What the callout is telling the reader. stop is announced as an alert.",
            ),
        ],
    },
    ComponentSpec {
        name: "Stamp",
        doc: "A short outline status tag in capitals.",
        props: &[
            req("text", PropKind::Text, "One or two words."),
            opt(
                "verdict",
                PropKind::Choice(Verdict::WIRE),
                "The colour that repeats the word.",
            ),
        ],
    },
    ComponentSpec {
        name: "StatStrip",
        doc: "A row of headline figures, each with its label.",
        props: &[
            req("stats", PropKind::Stats, "The figures, in order."),
            opt(
                "density",
                PropKind::Choice(Density::WIRE),
                "compact: a summary row. hero: the band the page is about.",
            ),
        ],
    },
    ComponentSpec {
        name: "ResultPanel",
        doc: "A calculator's answer: a label, one large figure announced when it changes.",
        props: &[
            req("label", PropKind::Text, "What the figure is."),
            req(
                "value",
                PropKind::Text,
                "The figure, as text, with its sign and unit.",
            ),
            opt("context", PropKind::Text, "The period or basis."),
            opt(
                "lean",
                PropKind::Choice(Lean::WIRE),
                "Whether the figure is good news.",
            ),
        ],
    },
    ComponentSpec {
        name: "Meter",
        doc: "How far a quantity is towards its limit; turns at the given shares.",
        props: &[
            req("label", PropKind::Text, "What is measured."),
            req("value", PropKind::Number, "The current amount."),
            req("max", PropKind::Number, "The limit."),
            req(
                "readout",
                PropKind::Text,
                "The amount in words, shown beside the fill.",
            ),
            opt(
                "warnAt",
                PropKind::Share,
                "Share of max from which the fill shows caution.",
            ),
            opt(
                "criticalAt",
                PropKind::Share,
                "Share of max from which the fill shows fault.",
            ),
        ],
    },
    ComponentSpec {
        name: "TextField",
        doc: "A labelled text input that writes to the data model as the user types.",
        props: &[
            req("label", PropKind::Text, "The label."),
            req(
                "value",
                PropKind::Bound,
                "Where the input reads and writes.",
            ),
            opt("placeholder", PropKind::Text, "Example input."),
            opt("help", PropKind::Text, "A line under the field."),
            opt(
                "inputType",
                PropKind::Choice(InputType::WIRE),
                "The input's type.",
            ),
        ],
    },
    ComponentSpec {
        name: "UnitField",
        doc: "A labelled number input with its unit inside the box.",
        props: &[
            req(
                "label",
                PropKind::Text,
                "The label; name the unit here for screen readers.",
            ),
            req(
                "value",
                PropKind::Bound,
                "Where the input reads and writes.",
            ),
            opt("prefix", PropKind::Text, "A unit before the number."),
            opt("suffix", PropKind::Text, "A unit after the number."),
            opt("help", PropKind::Text, "A line under the field."),
        ],
    },
    ComponentSpec {
        name: "Button",
        doc: "Sends an action to the agent. With no action it is disabled.",
        props: &[
            req("label", PropKind::Text, "The button's text."),
            opt("variant", PropKind::Choice(Variant::WIRE), "Emphasis."),
            opt("action", PropKind::Action, "The event sent when pressed."),
        ],
    },
];

/// The spec for a component name.
#[must_use]
pub fn spec(name: &str) -> Option<&'static ComponentSpec> {
    COMPONENTS.iter().find(|c| c.name == name)
}

/// One cell of a [`Node::StatStrip`].
#[derive(Debug, Clone, PartialEq)]
pub struct Stat {
    pub label: Dyn<String>,
    pub value: Dyn<String>,
    pub note: Option<Dyn<String>>,
    pub lean: Lean,
}

/// A parsed, checked component. Children are ids into the same surface.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Column {
        children: Vec<String>,
    },
    Grid {
        children: Vec<String>,
        columns: u8,
    },
    Card {
        children: Vec<String>,
        pad: Pad,
        surface: Surface,
    },
    Heading {
        text: Dyn<String>,
        level: u8,
        size: HeadingSize,
    },
    Text {
        text: Dyn<String>,
        tone: Tone,
    },
    Callout {
        text: Dyn<String>,
        title: Option<Dyn<String>>,
        verdict: Verdict,
    },
    Stamp {
        text: Dyn<String>,
        verdict: Verdict,
    },
    StatStrip {
        stats: Vec<Stat>,
        density: Density,
    },
    ResultPanel {
        label: Dyn<String>,
        value: Dyn<String>,
        context: Option<Dyn<String>>,
        lean: Lean,
    },
    Meter {
        label: Dyn<String>,
        value: Dyn<f64>,
        max: Dyn<f64>,
        readout: Dyn<String>,
        warn_at: Option<f64>,
        critical_at: Option<f64>,
    },
    TextField {
        label: Dyn<String>,
        value: Binding,
        placeholder: Option<Dyn<String>>,
        help: Option<Dyn<String>>,
        input_type: InputType,
    },
    UnitField {
        label: Dyn<String>,
        value: Binding,
        prefix: Option<Dyn<String>>,
        suffix: Option<Dyn<String>>,
        help: Option<Dyn<String>>,
    },
    Button {
        label: Dyn<String>,
        variant: Variant,
        action: Option<Action>,
    },
}

impl Node {
    /// The ids this node places inside itself.
    #[must_use]
    pub fn children(&self) -> &[String] {
        match self {
            Node::Column { children }
            | Node::Grid { children, .. }
            | Node::Card { children, .. } => children,
            _ => &[],
        }
    }

    /// Check `def` against the catalog and build the node.
    ///
    /// # Errors
    ///
    /// [`CatalogError`] naming the component, the prop and the reason.
    pub fn parse(def: &ComponentDef) -> Result<Node, CatalogError> {
        let Some(spec) = spec(&def.component) else {
            let names: Vec<_> = COMPONENTS.iter().map(|c| c.name).collect();
            return Err(CatalogError::new(
                def,
                None,
                format!(
                    "unknown component {:?}; {CATALOG_ID} has {}",
                    def.component,
                    names.join(", ")
                ),
            ));
        };
        for key in def.props.keys() {
            if !spec.props.iter().any(|p| p.name == key) {
                return Err(CatalogError::new(
                    def,
                    Some(key),
                    format!("{} has no prop {key:?}", spec.name),
                ));
            }
        }
        for prop in spec.props.iter().filter(|p| p.required) {
            if !def.props.contains_key(prop.name) {
                return Err(CatalogError::new(
                    def,
                    Some(prop.name),
                    "is required".into(),
                ));
            }
        }

        let r = Reader { def, spec };
        Ok(match spec.name {
            "Column" => Node::Column {
                children: r.children()?,
            },
            "Grid" => Node::Grid {
                children: r.children()?,
                columns: r.int("columns")?.map_or(3, narrow),
            },
            "Card" => Node::Card {
                children: r.children()?,
                pad: r.choice("pad")?,
                surface: r.choice("surface")?,
            },
            "Heading" => Node::Heading {
                text: r.text_req("text")?,
                level: r.int("level")?.map_or(2, narrow),
                size: r.choice("size")?,
            },
            "Text" => Node::Text {
                text: r.text_req("text")?,
                tone: r.choice("tone")?,
            },
            "Callout" => Node::Callout {
                text: r.text_req("text")?,
                title: r.text("title")?,
                verdict: r.choice("verdict")?,
            },
            "Stamp" => Node::Stamp {
                text: r.text_req("text")?,
                verdict: r.choice("verdict")?,
            },
            "StatStrip" => Node::StatStrip {
                stats: r.stats("stats")?,
                density: r.choice("density")?,
            },
            "ResultPanel" => Node::ResultPanel {
                label: r.text_req("label")?,
                value: r.text_req("value")?,
                context: r.text("context")?,
                lean: r.choice("lean")?,
            },
            "Meter" => Node::Meter {
                label: r.text_req("label")?,
                value: r.number_req("value")?,
                max: r.number_req("max")?,
                readout: r.text_req("readout")?,
                warn_at: r.share("warnAt")?,
                critical_at: r.share("criticalAt")?,
            },
            "TextField" => Node::TextField {
                label: r.text_req("label")?,
                value: r.bound("value")?,
                placeholder: r.text("placeholder")?,
                help: r.text("help")?,
                input_type: r.choice("inputType")?,
            },
            "UnitField" => Node::UnitField {
                label: r.text_req("label")?,
                value: r.bound("value")?,
                prefix: r.text("prefix")?,
                suffix: r.text("suffix")?,
                help: r.text("help")?,
            },
            "Button" => Node::Button {
                label: r.text_req("label")?,
                variant: r.choice("variant")?,
                action: r.action("action")?,
            },
            other => unreachable!("{other} is in COMPONENTS but has no parse arm"),
        })
    }
}

/// The ranges in [`COMPONENTS`] fit a `u8`.
fn narrow(n: i64) -> u8 {
    u8::try_from(n).unwrap_or(u8::MAX)
}

/// Why a component was rejected.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("component {component_id:?}{}: {reason}", prop.as_ref().map(|p| format!(", prop {p:?}")).unwrap_or_default())]
pub struct CatalogError {
    pub component_id: String,
    pub prop: Option<String>,
    pub reason: String,
}

impl CatalogError {
    fn new(def: &ComponentDef, prop: Option<&str>, reason: String) -> Self {
        Self {
            component_id: def.id.clone(),
            prop: prop.map(str::to_owned),
            reason,
        }
    }
}

struct Reader<'a> {
    def: &'a ComponentDef,
    spec: &'static ComponentSpec,
}

impl Reader<'_> {
    fn get(&self, name: &str) -> Option<&Value> {
        debug_assert!(
            self.spec.props.iter().any(|p| p.name == name),
            "{} reads {name}, which COMPONENTS does not declare",
            self.spec.name
        );
        self.def.props.get(name)
    }

    fn fail<T>(&self, prop: &str, reason: impl Into<String>) -> Result<T, CatalogError> {
        Err(CatalogError::new(self.def, Some(prop), reason.into()))
    }

    fn text(&self, name: &str) -> Result<Option<Dyn<String>>, CatalogError> {
        match self.get(name) {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(Dyn::Literal(s.clone()))),
            Some(Value::Object(map)) => binding(map)
                .map(|b| Some(Dyn::Bound(b)))
                .or_else(|reason| self.fail(name, reason)),
            Some(other) => self.fail(
                name,
                format!("expected a string or {{\"path\"}}, got {}", kind(other)),
            ),
        }
    }

    fn text_req(&self, name: &str) -> Result<Dyn<String>, CatalogError> {
        self.text(name)?
            .map_or_else(|| self.fail(name, "is required"), Ok)
    }

    fn number_req(&self, name: &str) -> Result<Dyn<f64>, CatalogError> {
        match self.get(name) {
            Some(Value::Number(n)) => match n.as_f64() {
                Some(n) if n.is_finite() => Ok(Dyn::Literal(n)),
                _ => self.fail(name, "must be a finite number"),
            },
            Some(Value::Object(map)) => binding(map)
                .map(Dyn::Bound)
                .or_else(|reason| self.fail(name, reason)),
            Some(other) => self.fail(
                name,
                format!("expected a number or {{\"path\"}}, got {}", kind(other)),
            ),
            None => self.fail(name, "is required"),
        }
    }

    fn int(&self, name: &str) -> Result<Option<i64>, CatalogError> {
        let Some(value) = self.get(name) else {
            return Ok(None);
        };
        let Some(PropKind::Int { min, max }) = self.kind_of(name) else {
            unreachable!("{name} is read as an integer");
        };
        match value.as_i64() {
            Some(n) if (min..=max).contains(&n) => Ok(Some(n)),
            _ => self.fail(name, format!("expected an integer from {min} to {max}")),
        }
    }

    fn share(&self, name: &str) -> Result<Option<f64>, CatalogError> {
        match self.get(name) {
            None => Ok(None),
            Some(value) => match value.as_f64() {
                Some(n) if (0.0..=1.0).contains(&n) => Ok(Some(n)),
                _ => self.fail(name, "expected a number from 0 to 1"),
            },
        }
    }

    fn choice<E: Operand>(&self, name: &str) -> Result<E, CatalogError> {
        match self.get(name) {
            None => Ok(E::default()),
            Some(Value::String(s)) => E::from_wire(s).map_or_else(
                || {
                    self.fail(
                        name,
                        format!("expected one of {}, got {s:?}", E::WIRE.join(", ")),
                    )
                },
                Ok,
            ),
            Some(other) => self.fail(
                name,
                format!(
                    "expected one of {}, got {}",
                    E::WIRE.join(", "),
                    kind(other)
                ),
            ),
        }
    }

    fn children(&self) -> Result<Vec<String>, CatalogError> {
        match self.get("children") {
            Some(Value::Array(items)) => items
                .iter()
                .map(|item| match item {
                    Value::String(id) if !id.is_empty() => Ok(id.clone()),
                    _ => self.fail("children", "every child must be a component id"),
                })
                .collect(),
            Some(Value::Object(_)) => self.fail(
                "children",
                "child-list templates ({\"path\", \"componentId\"}) are not supported; list the ids",
            ),
            _ => self.fail("children", "expected an array of component ids"),
        }
    }

    fn bound(&self, name: &str) -> Result<Binding, CatalogError> {
        match self.get(name) {
            Some(Value::Object(map)) => binding(map).or_else(|reason| self.fail(name, reason)),
            _ => self.fail(
                name,
                "an input's value must be {\"path\": \"/…\"}, so typing has somewhere to write",
            ),
        }
    }

    fn action(&self, name: &str) -> Result<Option<Action>, CatalogError> {
        let Some(value) = self.get(name) else {
            return Ok(None);
        };
        if value.get("functionCall").is_some() {
            return self.fail(
                name,
                "client-side function calls are not supported; send an event",
            );
        }
        let Some(event) = value.get("event").and_then(Value::as_object) else {
            return self.fail(name, "expected {\"event\": {\"name\": …}}");
        };
        if let Some(extra) = event.keys().find(|k| *k != "name" && *k != "context") {
            return self.fail(name, format!("event has no field {extra:?}"));
        }
        let Some(event_name) = event
            .get("name")
            .and_then(Value::as_str)
            .filter(|n| !n.is_empty())
        else {
            return self.fail(name, "event.name must be a non-empty string");
        };
        let mut context = Vec::new();
        match event.get("context") {
            None => {}
            Some(Value::Object(entries)) => {
                for (key, entry) in entries {
                    let resolved = match entry {
                        Value::Object(map)
                            if map.contains_key("path") || map.contains_key("call") =>
                        {
                            ContextValue::Bound(binding(map).or_else(|reason| {
                                self.fail(name, format!("context {key:?}: {reason}"))
                            })?)
                        }
                        literal => ContextValue::Literal(literal.clone()),
                    };
                    context.push((key.clone(), resolved));
                }
            }
            Some(_) => return self.fail(name, "event.context must be an object"),
        }
        Ok(Some(Action {
            name: event_name.to_owned(),
            context,
        }))
    }

    fn stats(&self, name: &str) -> Result<Vec<Stat>, CatalogError> {
        let Some(Value::Array(items)) = self.get(name) else {
            return self.fail(name, "expected an array of {label, value, note?, lean?}");
        };
        items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let Value::Object(map) = item else {
                    return self.fail(name, format!("item {index} must be an object"));
                };
                if let Some(extra) = map
                    .keys()
                    .find(|k| !matches!(k.as_str(), "label" | "value" | "note" | "lean"))
                {
                    return self.fail(name, format!("item {index} has no field {extra:?}"));
                }
                let field = |key: &str| -> Result<Option<Dyn<String>>, CatalogError> {
                    match map.get(key) {
                        None => Ok(None),
                        Some(Value::String(s)) => Ok(Some(Dyn::Literal(s.clone()))),
                        Some(Value::Object(b)) => {
                            binding(b).map(|b| Some(Dyn::Bound(b))).or_else(|reason| {
                                self.fail(name, format!("item {index}.{key}: {reason}"))
                            })
                        }
                        Some(_) => self.fail(
                            name,
                            format!("item {index}.{key} must be a string or {{\"path\"}}"),
                        ),
                    }
                };
                let required = |key: &str| {
                    field(key)?.map_or_else(
                        || self.fail(name, format!("item {index}.{key} is required")),
                        Ok,
                    )
                };
                let lean = match map.get("lean") {
                    None => Lean::default(),
                    Some(Value::String(s)) => match Lean::from_wire(s) {
                        Some(lean) => lean,
                        None => {
                            return self.fail(
                                name,
                                format!(
                                    "item {index}.lean must be one of {}",
                                    Lean::WIRE.join(", ")
                                ),
                            );
                        }
                    },
                    Some(_) => {
                        return self.fail(name, format!("item {index}.lean must be a string"));
                    }
                };
                Ok(Stat {
                    label: required("label")?,
                    value: required("value")?,
                    note: field("note")?,
                    lean,
                })
            })
            .collect()
    }

    fn kind_of(&self, name: &str) -> Option<PropKind> {
        self.spec
            .props
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.kind)
    }
}

/// `{"path": "/…"}` and nothing else.
fn binding(map: &Map<String, Value>) -> Result<Binding, String> {
    if map.contains_key("call") {
        return Err("function calls are not supported; bind with {\"path\": \"/…\"}".into());
    }
    match (map.len(), map.get("path")) {
        (1, Some(Value::String(path))) => Binding::new(path),
        _ => Err("expected {\"path\": \"/…\"}".into()),
    }
}

fn kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

/// The catalog as a JSON Schema document, `$id` and `catalogId` both set to
/// [`CATALOG_ID`] as v0.9.1 asks.
#[must_use]
pub fn catalog_document() -> Value {
    let components: Map<String, Value> = COMPONENTS
        .iter()
        .map(|c| (c.name.to_owned(), component_schema(c)))
        .collect();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": CATALOG_ID,
        "catalogId": CATALOG_ID,
        "title": "rv2 A2UI catalog, v1",
        "description": "Components an agent may place on an rv2 surface. Additive only: a v1 prop or component is never renamed or removed. Values bind with {\"path\": \"/…\"} only; function calls, relative paths and child-list templates are rejected.",
        "components": components,
        "functions": {},
        "$defs": {
            "ComponentId": { "type": "string", "minLength": 1 },
            "DataBinding": {
                "type": "object",
                "properties": { "path": { "type": "string", "pattern": "^/" } },
                "required": ["path"],
                "additionalProperties": false
            },
            "Action": {
                "type": "object",
                "properties": {
                    "event": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "minLength": 1 },
                            "context": {
                                "type": "object",
                                "description": "Literal values, or {\"path\"} bindings the client resolves before sending."
                            }
                        },
                        "required": ["name"],
                        "additionalProperties": false
                    }
                },
                "required": ["event"],
                "additionalProperties": false
            }
        }
    })
}

fn component_schema(spec: &ComponentSpec) -> Value {
    let mut properties = Map::new();
    properties.insert("id".into(), json!({ "$ref": "#/$defs/ComponentId" }));
    properties.insert("component".into(), json!({ "const": spec.name }));
    let mut required = vec![json!("id"), json!("component")];
    for prop in spec.props {
        let mut schema = prop_schema(prop.kind);
        if let Value::Object(map) = &mut schema {
            map.insert("description".into(), json!(prop.doc));
        }
        properties.insert(prop.name.into(), schema);
        if prop.required {
            required.push(json!(prop.name));
        }
    }
    json!({
        "type": "object",
        "description": spec.doc,
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}

fn prop_schema(kind: PropKind) -> Value {
    match kind {
        PropKind::Text => {
            json!({ "oneOf": [{ "type": "string" }, { "$ref": "#/$defs/DataBinding" }] })
        }
        PropKind::Number => {
            json!({ "oneOf": [{ "type": "number" }, { "$ref": "#/$defs/DataBinding" }] })
        }
        PropKind::Int { min, max } => json!({ "type": "integer", "minimum": min, "maximum": max }),
        PropKind::Share => json!({ "type": "number", "minimum": 0, "maximum": 1 }),
        PropKind::Choice(values) => json!({ "enum": values }),
        PropKind::Children => {
            json!({ "type": "array", "items": { "$ref": "#/$defs/ComponentId" } })
        }
        PropKind::Bound => json!({ "$ref": "#/$defs/DataBinding" }),
        PropKind::Action => json!({ "$ref": "#/$defs/Action" }),
        PropKind::Stats => json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "label": { "oneOf": [{ "type": "string" }, { "$ref": "#/$defs/DataBinding" }] },
                    "value": { "oneOf": [{ "type": "string" }, { "$ref": "#/$defs/DataBinding" }] },
                    "note": { "oneOf": [{ "type": "string" }, { "$ref": "#/$defs/DataBinding" }] },
                    "lean": { "enum": Lean::WIRE }
                },
                "required": ["label", "value"],
                "additionalProperties": false
            }
        }),
    }
}
