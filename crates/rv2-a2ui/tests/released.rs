//! v1 only grows (A2UI4). `catalog/released/v1.json` is the catalog as first
//! published and is never regenerated; the live catalog must accept everything
//! it described. Blessing `rv2-v1.json` cannot hide a removal from this test.

use rv2_a2ui::catalog_document;
use serde_json::Value;

const RELEASED: &str = include_str!("../catalog/released/v1.json");

/// Why `new` refuses something `old` allowed, with the JSON path to it.
fn narrowing(path: &str, old: &Value, new: &Value) -> Vec<String> {
    let here = |key: &str| format!("{path}/{key}");
    match (old, new) {
        (Value::Object(old), Value::Object(new)) => {
            let mut found = Vec::new();
            for (key, was) in old {
                if matches!(key.as_str(), "description" | "title") {
                    continue;
                }
                let Some(now) = new.get(key) else {
                    found.push(format!("{} was removed", here(key)));
                    continue;
                };
                match key.as_str() {
                    "enum" => {
                        for value in was.as_array().into_iter().flatten() {
                            if !now.as_array().is_some_and(|a| a.contains(value)) {
                                found.push(format!("{} lost {value}", here(key)));
                            }
                        }
                    }
                    "required" => {
                        for name in now.as_array().into_iter().flatten() {
                            if !was.as_array().is_some_and(|a| a.contains(name)) {
                                found.push(format!("{} newly requires {name}", here(key)));
                            }
                        }
                    }
                    "minimum" if now.as_f64() > was.as_f64() => {
                        found.push(format!("{} rose from {was} to {now}", here(key)));
                    }
                    "maximum" if now.as_f64() < was.as_f64() => {
                        found.push(format!("{} fell from {was} to {now}", here(key)));
                    }
                    "minimum" | "maximum" => {}
                    _ => found.extend(narrowing(&here(key), was, now)),
                }
            }
            found
        }
        (Value::Array(old), Value::Array(new)) => {
            let mut found: Vec<String> = old
                .iter()
                .zip(new)
                .enumerate()
                .flat_map(|(i, (was, now))| narrowing(&here(&i.to_string()), was, now))
                .collect();
            if new.len() < old.len() {
                found.push(format!("{path} lost {} entries", old.len() - new.len()));
            }
            found
        }
        _ if old == new => Vec::new(),
        _ => vec![format!("{path} changed from {old} to {new}")],
    }
}

#[test]
fn the_live_catalog_accepts_everything_v1_released() {
    let released: Value = serde_json::from_str(RELEASED).unwrap();
    let found = narrowing("", &released, &catalog_document());
    assert!(
        found.is_empty(),
        "v1 only grows (A2UI4); add instead, or publish v2:\n  {}",
        found.join("\n  ")
    );
}

#[test]
fn the_check_catches_a_removed_operand_a_new_requirement_and_a_lost_component() {
    let released: Value = serde_json::from_str(RELEASED).unwrap();
    let mut narrowed = released.clone();
    let callout = &mut narrowed["components"]["Callout"];
    callout["properties"]["verdict"]["enum"] = serde_json::json!(["note", "clear"]);
    callout["required"]
        .as_array_mut()
        .unwrap()
        .push("title".into());
    narrowed["components"]
        .as_object_mut()
        .unwrap()
        .remove("Stamp");

    let found = narrowing("", &released, &narrowed);
    let has = |needle: &str| found.iter().any(|f| f.contains(needle));
    assert!(
        has("/components/Callout/properties/verdict/enum lost \"caution\""),
        "{found:?}"
    );
    assert!(
        has("/components/Callout/required newly requires \"title\""),
        "{found:?}"
    );
    assert!(has("/components/Stamp was removed"), "{found:?}");
}

#[test]
fn a_new_prop_or_a_reworded_description_is_not_a_narrowing() {
    let released: Value = serde_json::from_str(RELEASED).unwrap();
    let mut grown = released.clone();
    grown["components"]["Text"]["properties"]["align"] = serde_json::json!({"enum": ["start"]});
    grown["components"]["Text"]["description"] = "Reworded.".into();
    assert!(narrowing("", &released, &grown).is_empty());
}
