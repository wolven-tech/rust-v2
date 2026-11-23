//! A2UI v0.9.1 envelopes.
//!
//! Every message carries a top-level `version` and exactly one body key. The
//! spec's own pages disagree on the string (`"v0.9"` on the messages page,
//! `"v0.9.1"` in the specification), so reading accepts any `v0.9` revision
//! and writing always says [`VERSION`].

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

/// The version this crate writes.
pub const VERSION: &str = "v0.9.1";

/// The prefix every accepted version starts with. v1.0 renames the binding
/// keys, so a `v1.0` message parsed as v0.9 would bind nothing.
const ACCEPTED_PREFIX: &str = "v0.9";

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum WireError {
    #[error("line {line}: {message}")]
    Malformed { line: usize, message: String },
    #[error("unsupported A2UI version {0:?}; this client reads v0.9")]
    Version(String),
}

/// Agent to client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerMessage {
    pub version: String,
    #[serde(flatten)]
    pub body: ServerBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServerBody {
    CreateSurface(CreateSurface),
    UpdateComponents(UpdateComponents),
    UpdateDataModel(UpdateDataModel),
    DeleteSurface(DeleteSurface),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSurface {
    pub surface_id: String,
    pub catalog_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<Value>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub send_data_model: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponents {
    pub surface_id: String,
    pub components: Vec<ComponentDef>,
}

/// Upsert at `path`, or remove the key there when `value` is absent.
///
/// `value: null` and no `value` are different instructions in v0.9.1 (set to
/// null, remove), so the field keeps a present `null` as `Some(Value::Null)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDataModel {
    pub surface_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub value: Option<Value>,
}

fn present<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Value>, D::Error> {
    Value::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSurface {
    pub surface_id: String,
}

/// One entry of `updateComponents`: an id, a catalog name, and flat props.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentDef {
    pub id: String,
    pub component: String,
    #[serde(flatten)]
    pub props: Map<String, Value>,
}

impl ComponentDef {
    pub fn new(id: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            component: component.into(),
            props: Map::new(),
        }
    }

    #[must_use]
    pub fn prop(mut self, name: &str, value: impl Into<Value>) -> Self {
        self.props.insert(name.to_owned(), value.into());
        self
    }

    #[must_use]
    pub fn children<I, S>(self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let ids: Vec<Value> = ids.into_iter().map(|id| Value::String(id.into())).collect();
        self.prop("children", ids)
    }

    /// `{"path": path}`, the binding form of any value prop.
    #[must_use]
    pub fn bind(self, name: &str, path: &str) -> Self {
        self.prop(name, serde_json::json!({ "path": path }))
    }
}

impl ServerMessage {
    fn new(body: ServerBody) -> Self {
        Self {
            version: VERSION.to_owned(),
            body,
        }
    }

    pub fn create_surface(surface_id: impl Into<String>, catalog_id: impl Into<String>) -> Self {
        Self::new(ServerBody::CreateSurface(CreateSurface {
            surface_id: surface_id.into(),
            catalog_id: catalog_id.into(),
            theme: None,
            send_data_model: false,
        }))
    }

    pub fn update_components(surface_id: impl Into<String>, components: Vec<ComponentDef>) -> Self {
        Self::new(ServerBody::UpdateComponents(UpdateComponents {
            surface_id: surface_id.into(),
            components,
        }))
    }

    pub fn update_data_model(
        surface_id: impl Into<String>,
        path: impl Into<String>,
        value: Value,
    ) -> Self {
        Self::new(ServerBody::UpdateDataModel(UpdateDataModel {
            surface_id: surface_id.into(),
            path: Some(path.into()),
            value: Some(value),
        }))
    }

    pub fn delete_surface(surface_id: impl Into<String>) -> Self {
        Self::new(ServerBody::DeleteSurface(DeleteSurface {
            surface_id: surface_id.into(),
        }))
    }

    #[must_use]
    pub fn surface_id(&self) -> &str {
        match &self.body {
            ServerBody::CreateSurface(m) => &m.surface_id,
            ServerBody::UpdateComponents(m) => &m.surface_id,
            ServerBody::UpdateDataModel(m) => &m.surface_id,
            ServerBody::DeleteSurface(m) => &m.surface_id,
        }
    }
}

/// Client to agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientMessage {
    pub version: String,
    #[serde(flatten)]
    pub body: ClientBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClientBody {
    Action(ActionMessage),
    Error(ErrorMessage),
}

impl ClientMessage {
    #[must_use]
    pub fn action(action: ActionMessage) -> Self {
        Self {
            version: VERSION.to_owned(),
            body: ClientBody::Action(action),
        }
    }

    #[must_use]
    pub fn error(error: ErrorMessage) -> Self {
        Self {
            version: VERSION.to_owned(),
            body: ClientBody::Error(error),
        }
    }

    /// # Errors
    ///
    /// [`WireError::Version`] when the message is not v0.9.
    pub fn check_version(&self) -> Result<(), WireError> {
        check_version(&self.version)
    }
}

/// A user action, with its `context` already resolved from the data model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionMessage {
    pub name: String,
    pub surface_id: String,
    pub source_component_id: String,
    /// ISO 8601, stamped by the client.
    pub timestamp: String,
    #[serde(default)]
    pub context: Map<String, Value>,
}

/// A client's report that a message failed validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessage {
    /// `VALIDATION_FAILED` is the one code v0.9.1 defines.
    pub code: String,
    pub surface_id: String,
    /// JSON Pointer into the failing message.
    pub path: String,
    pub message: String,
}

pub(crate) fn check_version(version: &str) -> Result<(), WireError> {
    if version == ACCEPTED_PREFIX || version.starts_with("v0.9.") {
        Ok(())
    } else {
        Err(WireError::Version(version.to_owned()))
    }
}

/// Parse a JSONL stream: one message per line, blank lines skipped.
///
/// # Errors
///
/// [`WireError::Malformed`] naming the 1-based line, or [`WireError::Version`].
pub fn parse_jsonl(text: &str) -> Result<Vec<ServerMessage>, WireError> {
    let mut messages = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let message: ServerMessage =
            serde_json::from_str(line).map_err(|e| WireError::Malformed {
                line: index + 1,
                message: e.to_string(),
            })?;
        check_version(&message.version)?;
        messages.push(message);
    }
    Ok(messages)
}

/// One message per line, each line ending in `\n`.
#[must_use]
pub fn to_jsonl(messages: &[ServerMessage]) -> String {
    let mut out = String::new();
    for message in messages {
        // A derived `Serialize` over strings, maps and `Value` cannot fail.
        out.push_str(&serde_json::to_string(message).unwrap_or_default());
        out.push('\n');
    }
    out
}
