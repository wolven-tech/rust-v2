//! JSON Pointer (RFC 6901) over the data model, with A2UI's write rules.
//!
//! `"/"` and `""` both address the whole model: A2UI uses `"/"` for "replace
//! everything", where RFC 6901 would read it as the key `""`. Writes create
//! missing intermediate objects. Removing an array element leaves `null` in
//! its place, because v0.9.1 keeps the array's length.

use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum PointerError {
    #[error("path {0:?} must start with \"/\"")]
    NotAbsolute(String),
    #[error("path {0:?} has an escape other than ~0 or ~1")]
    BadEscape(String),
    #[error("path {path:?} indexes an array with {token:?}")]
    BadIndex { path: String, token: String },
}

/// The unescaped tokens of an absolute pointer; empty for the whole model.
///
/// # Errors
///
/// [`PointerError`] for a relative path or a bad `~` escape.
pub fn tokens(path: &str) -> Result<Vec<String>, PointerError> {
    if path.is_empty() || path == "/" {
        return Ok(Vec::new());
    }
    let Some(rest) = path.strip_prefix('/') else {
        return Err(PointerError::NotAbsolute(path.to_owned()));
    };
    rest.split('/')
        .map(|raw| unescape(raw).ok_or_else(|| PointerError::BadEscape(path.to_owned())))
        .collect()
}

fn unescape(raw: &str) -> Option<String> {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '~' {
            match chars.next() {
                Some('0') => out.push('~'),
                Some('1') => out.push('/'),
                _ => return None,
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}

/// The value at `path`, or `None` when any step is missing.
#[must_use]
pub fn get<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = root;
    for token in tokens(path).ok()? {
        current = match current {
            Value::Object(map) => map.get(&token)?,
            Value::Array(items) => items.get(token.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

/// Upsert `value` at `path`.
///
/// A scalar standing where a container is needed is replaced by an object, so
/// a write always lands. An array step takes an index up to its length, where
/// it appends.
///
/// # Errors
///
/// [`PointerError`] for a malformed path or a non-numeric array step.
pub fn set(root: &mut Value, path: &str, value: Value) -> Result<(), PointerError> {
    let tokens = tokens(path)?;
    let Some((last, parents)) = tokens.split_last() else {
        *root = value;
        return Ok(());
    };
    let mut current = root;
    for token in parents {
        current = step_mut(current, token, path)?;
    }
    match current {
        Value::Array(items) => {
            let index = array_index(items.len(), last, path)?;
            if index == items.len() {
                items.push(value);
            } else {
                items[index] = value;
            }
        }
        other => {
            if !other.is_object() {
                *other = Value::Object(Map::new());
            }
            if let Value::Object(map) = other {
                map.insert(last.clone(), value);
            }
        }
    }
    Ok(())
}

/// Remove the key at `path`. An array element becomes `null`.
///
/// # Errors
///
/// [`PointerError`] for a malformed path.
pub fn remove(root: &mut Value, path: &str) -> Result<(), PointerError> {
    let tokens = tokens(path)?;
    let Some((last, parents)) = tokens.split_last() else {
        *root = Value::Object(Map::new());
        return Ok(());
    };
    let mut current = root;
    for token in parents {
        current = match current {
            Value::Object(map) => match map.get_mut(token) {
                Some(next) => next,
                None => return Ok(()),
            },
            Value::Array(items) => match token.parse::<usize>().ok().and_then(|i| items.get_mut(i))
            {
                Some(next) => next,
                None => return Ok(()),
            },
            _ => return Ok(()),
        };
    }
    match current {
        Value::Object(map) => {
            map.remove(last);
        }
        Value::Array(items) => {
            if let Some(slot) = last.parse::<usize>().ok().and_then(|i| items.get_mut(i)) {
                *slot = Value::Null;
            }
        }
        _ => {}
    }
    Ok(())
}

fn step_mut<'a>(
    current: &'a mut Value,
    token: &str,
    path: &str,
) -> Result<&'a mut Value, PointerError> {
    if let Value::Array(items) = current {
        let index = array_index(items.len(), token, path)?;
        if index == items.len() {
            items.push(Value::Object(Map::new()));
        }
        return Ok(&mut items[index]);
    }
    if !current.is_object() {
        *current = Value::Object(Map::new());
    }
    let Value::Object(map) = current else {
        unreachable!("replaced by an object above")
    };
    Ok(map
        .entry(token.to_owned())
        .or_insert_with(|| Value::Object(Map::new())))
}

fn array_index(len: usize, token: &str, path: &str) -> Result<usize, PointerError> {
    match token.parse::<usize>() {
        Ok(index) if index <= len => Ok(index),
        _ => Err(PointerError::BadIndex {
            path: path.to_owned(),
            token: token.to_owned(),
        }),
    }
}
