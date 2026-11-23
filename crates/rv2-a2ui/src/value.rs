//! Props that are a literal or a binding into the data model, and actions.

use serde_json::{Map, Value};

use crate::pointer;
use crate::wire::ActionMessage;

/// An absolute JSON Pointer into a surface's data model, checked at parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding(String);

impl Binding {
    /// # Errors
    ///
    /// The reason, in words, when `path` is relative or malformed (A2UI9).
    pub fn new(path: &str) -> Result<Self, String> {
        if !path.starts_with('/') {
            return Err(format!(
                "relative path {path:?} is not supported; bind with an absolute path starting \"/\""
            ));
        }
        pointer::tokens(path).map_err(|e| e.to_string())?;
        Ok(Self(path.to_owned()))
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.0
    }
}

/// A prop's value: written into the component, or read from the data model.
#[derive(Debug, Clone, PartialEq)]
pub enum Dyn<T> {
    Literal(T),
    Bound(Binding),
}

impl Dyn<String> {
    /// The text to show. A missing value renders empty, which is v0.9.1's rule
    /// for a path the stream has not filled yet.
    #[must_use]
    pub fn resolve(&self, data: &Value) -> String {
        match self {
            Dyn::Literal(text) => text.clone(),
            Dyn::Bound(binding) => pointer::get(data, binding.path())
                .map(display)
                .unwrap_or_default(),
        }
    }
}

impl Dyn<f64> {
    /// The number, or NaN when the bound value is missing or not numeric, so
    /// a renderer that already guards NaN (`Meter`) shows an empty state.
    #[must_use]
    pub fn resolve(&self, data: &Value) -> f64 {
        match self {
            Dyn::Literal(n) => *n,
            Dyn::Bound(binding) => match pointer::get(data, binding.path()) {
                Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
                Some(Value::String(s)) => s.trim().parse().unwrap_or(f64::NAN),
                _ => f64::NAN,
            },
        }
    }
}

/// v0.9.1's string conversion: numbers and booleans as written, null as empty,
/// objects and arrays as JSON.
fn display(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// A Button's `action.event`: a name and a context of literals or bindings.
#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub name: String,
    pub context: Vec<(String, ContextValue)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContextValue {
    Literal(Value),
    Bound(Binding),
}

impl Action {
    /// Resolve every binding against `data`. A missing binding sends `null`,
    /// so the agent sees the key and can say what is missing.
    #[must_use]
    pub fn request(
        &self,
        surface_id: &str,
        source_component_id: &str,
        data: &Value,
    ) -> ActionRequest {
        let context = self
            .context
            .iter()
            .map(|(key, value)| {
                let resolved = match value {
                    ContextValue::Literal(v) => v.clone(),
                    ContextValue::Bound(binding) => pointer::get(data, binding.path())
                        .cloned()
                        .unwrap_or(Value::Null),
                };
                (key.clone(), resolved)
            })
            .collect();
        ActionRequest {
            name: self.name.clone(),
            surface_id: surface_id.to_owned(),
            source_component_id: source_component_id.to_owned(),
            context,
        }
    }
}

/// An action ready to send, waiting only for its timestamp.
///
/// Split from [`ActionMessage`] because the WASM crates have no clock: the
/// renderer builds this, and the transport stamps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionRequest {
    pub name: String,
    pub surface_id: String,
    pub source_component_id: String,
    pub context: Map<String, Value>,
}

impl ActionRequest {
    #[must_use]
    pub fn stamp(self, timestamp: impl Into<String>) -> ActionMessage {
        ActionMessage {
            name: self.name,
            surface_id: self.surface_id,
            source_component_id: self.source_component_id,
            timestamp: timestamp.into(),
            context: self.context,
        }
    }
}
