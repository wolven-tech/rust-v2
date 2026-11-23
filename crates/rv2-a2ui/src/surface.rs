//! Surface state, and the checks that make an agent's output safe to render.
//!
//! Every message is applied to a copy and committed only if the result passes
//! (A2UI5), so a rejected message leaves the surface exactly as it was.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};
use thiserror::Error;

use crate::catalog::{CATALOG_ID, CatalogError, Node};
use crate::pointer::{self, PointerError};
use crate::wire::{ErrorMessage, ServerBody, ServerMessage, WireError};

/// Deepest nesting from `root`. A page that needs more is a page nobody reads.
pub const MAX_DEPTH: usize = 16;

/// Most components one surface may hold.
pub const MAX_COMPONENTS: usize = 256;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum SurfaceError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error("surface {0:?} already exists; delete it first")]
    Exists(String),
    #[error("surface {0:?} does not exist")]
    Missing(String),
    #[error("catalog {0:?} is not supported; this client renders {CATALOG_ID}")]
    Catalog(String),
    #[error("component {index}: {error}")]
    Component { index: usize, error: CatalogError },
    #[error("component id {0:?} appears twice in one update")]
    DuplicateId(String),
    #[error("component {0:?} is placed by more than one parent")]
    SharedChild(String),
    #[error("component {0:?} contains itself")]
    Cycle(String),
    #[error("component {0:?} is nested deeper than {MAX_DEPTH}")]
    TooDeep(String),
    #[error("surface would hold {0} components; the limit is {MAX_COMPONENTS}")]
    TooMany(usize),
    #[error("data model path: {0}")]
    Path(#[from] PointerError),
}

impl SurfaceError {
    /// The v0.9.1 `error` message a client sends back for this failure.
    #[must_use]
    pub fn to_message(&self, surface_id: &str) -> ErrorMessage {
        let path = match self {
            SurfaceError::Component { index, error } => match &error.prop {
                Some(prop) => format!("/updateComponents/components/{index}/{prop}"),
                None => format!("/updateComponents/components/{index}"),
            },
            SurfaceError::Path(_) => "/updateDataModel/path".to_owned(),
            SurfaceError::Catalog(_) => "/createSurface/catalogId".to_owned(),
            _ => "/".to_owned(),
        };
        ErrorMessage {
            code: "VALIDATION_FAILED".to_owned(),
            surface_id: surface_id.to_owned(),
            path,
            message: self.to_string(),
        }
    }
}

/// One surface: its checked components and its data model.
#[derive(Debug, Clone, PartialEq)]
pub struct Surface {
    id: String,
    nodes: BTreeMap<String, Node>,
    data: Value,
}

impl Surface {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            nodes: BTreeMap::new(),
            data: Value::Object(Map::new()),
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Whether `root` has arrived. Until it has, v0.9.1 renders nothing.
    #[must_use]
    pub fn has_root(&self) -> bool {
        self.nodes.contains_key("root")
    }

    #[must_use]
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    #[must_use]
    pub fn data(&self) -> &Value {
        &self.data
    }

    /// Write what a user typed. Bindings were checked at parse, so this fails
    /// only for an array step the data model does not have.
    ///
    /// # Errors
    ///
    /// [`PointerError`] when the path cannot be written.
    pub fn set_value(&mut self, path: &str, value: Value) -> Result<(), PointerError> {
        pointer::set(&mut self.data, path, value)
    }

    fn update_components(
        &mut self,
        components: &[crate::wire::ComponentDef],
    ) -> Result<(), SurfaceError> {
        let mut seen = BTreeSet::new();
        for (index, def) in components.iter().enumerate() {
            if !seen.insert(def.id.as_str()) {
                return Err(SurfaceError::DuplicateId(def.id.clone()));
            }
            let node =
                Node::parse(def).map_err(|error| SurfaceError::Component { index, error })?;
            self.nodes.insert(def.id.clone(), node);
        }
        if self.nodes.len() > MAX_COMPONENTS {
            return Err(SurfaceError::TooMany(self.nodes.len()));
        }
        self.check_tree()
    }

    /// Every component reachable from `root` has one parent, no cycle, and
    /// sits within [`MAX_DEPTH`]. A child id with no component yet is allowed:
    /// v0.9.1 streams components in any order and renders the gap as pending.
    fn check_tree(&self) -> Result<(), SurfaceError> {
        let mut parent_of: BTreeMap<&str, &str> = BTreeMap::new();
        for (id, node) in &self.nodes {
            for child in node.children() {
                if child == "root" || child == id {
                    return Err(SurfaceError::Cycle(child.clone()));
                }
                if parent_of.insert(child, id).is_some() {
                    return Err(SurfaceError::SharedChild(child.clone()));
                }
            }
        }
        if !self.has_root() {
            return Ok(());
        }
        // With one parent each and `root` never a child, no cycle is reachable
        // from root; `on_path` is the backstop if that reasoning ever breaks.
        // An unreachable cycle is never rendered.
        let mut stack = vec![("root", 0usize)];
        let mut on_path = BTreeSet::new();
        while let Some((id, depth)) = stack.pop() {
            if depth > MAX_DEPTH {
                return Err(SurfaceError::TooDeep(id.to_owned()));
            }
            if !on_path.insert(id) {
                return Err(SurfaceError::Cycle(id.to_owned()));
            }
            if let Some(node) = self.nodes.get(id) {
                stack.extend(node.children().iter().map(|c| (c.as_str(), depth + 1)));
            }
        }
        Ok(())
    }
}

/// Every surface a client holds, keyed by id.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Surfaces {
    surfaces: BTreeMap<String, Surface>,
}

impl Surfaces {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Surface> {
        self.surfaces.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Surface> {
        self.surfaces.get_mut(id)
    }

    /// Apply one message, or none of it.
    ///
    /// # Errors
    ///
    /// [`SurfaceError`]; the surfaces are unchanged when this returns one.
    pub fn apply(&mut self, message: &ServerMessage) -> Result<(), SurfaceError> {
        crate::wire::check_version(&message.version)?;
        match &message.body {
            ServerBody::CreateSurface(create) => {
                if create.catalog_id != CATALOG_ID {
                    return Err(SurfaceError::Catalog(create.catalog_id.clone()));
                }
                if self.surfaces.contains_key(&create.surface_id) {
                    return Err(SurfaceError::Exists(create.surface_id.clone()));
                }
                self.surfaces
                    .insert(create.surface_id.clone(), Surface::new(&create.surface_id));
            }
            ServerBody::UpdateComponents(update) => {
                let current = self.existing(&update.surface_id)?;
                let mut next = current.clone();
                next.update_components(&update.components)?;
                self.surfaces.insert(update.surface_id.clone(), next);
            }
            ServerBody::UpdateDataModel(update) => {
                let current = self.existing(&update.surface_id)?;
                let path = update.path.as_deref().unwrap_or("/");
                let mut data = current.data.clone();
                match &update.value {
                    Some(value) => pointer::set(&mut data, path, value.clone())?,
                    None => pointer::remove(&mut data, path)?,
                }
                if let Some(surface) = self.surfaces.get_mut(&update.surface_id) {
                    surface.data = data;
                }
            }
            ServerBody::DeleteSurface(delete) => {
                self.surfaces.remove(&delete.surface_id);
            }
        }
        Ok(())
    }

    /// Apply messages in order, stopping at the first that fails.
    ///
    /// # Errors
    ///
    /// The failing message's surface id and error. Earlier messages stay
    /// applied: each is a complete instruction on its own.
    pub fn apply_all(&mut self, messages: &[ServerMessage]) -> Result<(), (String, SurfaceError)> {
        for message in messages {
            self.apply(message)
                .map_err(|e| (message.surface_id().to_owned(), e))?;
        }
        Ok(())
    }

    fn existing(&self, id: &str) -> Result<&Surface, SurfaceError> {
        self.surfaces
            .get(id)
            .ok_or_else(|| SurfaceError::Missing(id.to_owned()))
    }
}
