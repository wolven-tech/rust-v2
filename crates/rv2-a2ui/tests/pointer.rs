//! JSON Pointer reads and A2UI's write rules on the data model.

use rv2_a2ui::pointer::{self, PointerError};
use serde_json::json;

#[test]
fn reads_follow_objects_arrays_and_escapes() {
    let data = json!({"a/b": {"c~d": [10, 20]}});
    assert_eq!(pointer::get(&data, "/a~1b/c~0d/1"), Some(&json!(20)));
    assert_eq!(pointer::get(&data, "/a~1b/missing"), None);
    assert_eq!(pointer::get(&data, "/"), Some(&data));
}

#[test]
fn a_write_creates_the_objects_on_its_way() {
    let mut data = json!({});
    pointer::set(&mut data, "/reservation/guest/email", json!("a@b.c")).unwrap();
    assert_eq!(data, json!({"reservation": {"guest": {"email": "a@b.c"}}}));
}

#[test]
fn a_write_to_the_root_replaces_the_model() {
    let mut data = json!({"old": true});
    pointer::set(&mut data, "/", json!({"new": true})).unwrap();
    assert_eq!(data, json!({"new": true}));
}

#[test]
fn an_array_takes_an_index_up_to_its_length_and_nothing_else() {
    let mut data = json!({"items": [1]});
    pointer::set(&mut data, "/items/1", json!(2)).unwrap();
    assert_eq!(data, json!({"items": [1, 2]}));
    assert!(matches!(
        pointer::set(&mut data, "/items/5", json!(0)),
        Err(PointerError::BadIndex { .. })
    ));
    assert!(matches!(
        pointer::set(&mut data, "/items/x", json!(0)),
        Err(PointerError::BadIndex { .. })
    ));
}

#[test]
fn removing_an_array_element_keeps_the_length() {
    let mut data = json!({"items": [1, 2, 3], "keep": 1, "drop": 2});
    pointer::remove(&mut data, "/items/1").unwrap();
    pointer::remove(&mut data, "/drop").unwrap();
    assert_eq!(data, json!({"items": [1, null, 3], "keep": 1}));
}

#[test]
fn a_relative_path_or_a_bad_escape_is_refused() {
    assert!(matches!(
        pointer::tokens("name"),
        Err(PointerError::NotAbsolute(_))
    ));
    assert!(matches!(
        pointer::tokens("/a~2"),
        Err(PointerError::BadEscape(_))
    ));
}
