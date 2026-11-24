//! The check every agent's tests run on its output (004, "Adding an agent").
//!
//! It takes the path a browser takes: the messages are framed as JSONL,
//! parsed back and applied through [`Surfaces`]. An agent whose output the
//! catalog refuses fails here instead of in a page that renders nothing.

use crate::surface::Surfaces;
use crate::wire::{ServerBody, ServerMessage, parse_jsonl, to_jsonl};

/// Apply `messages` to `surfaces` as the client would.
///
/// # Errors
///
/// A sentence naming the failure: a message that does not survive JSONL
/// framing, one the surfaces refuse, or a surface the stream leaves with no
/// `root` to render.
pub fn apply_as_client(surfaces: &mut Surfaces, messages: &[ServerMessage]) -> Result<(), String> {
    let framed = parse_jsonl(&to_jsonl(messages)).map_err(|e| format!("JSONL framing: {e}"))?;
    if framed != messages {
        return Err("a message changed when framed as JSONL and parsed back".into());
    }
    surfaces
        .apply_all(&framed)
        .map_err(|(surface, e)| format!("surface {surface:?}: {e}"))?;
    for message in &framed {
        if matches!(message.body, ServerBody::DeleteSurface(_)) {
            continue;
        }
        let id = message.surface_id();
        if surfaces.get(id).is_some_and(|s| !s.has_root()) {
            return Err(format!(
                "surface {id:?} has no \"root\" component to render"
            ));
        }
    }
    Ok(())
}
