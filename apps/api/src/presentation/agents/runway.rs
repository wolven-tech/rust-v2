//! Runway: months of cash at the current net burn.
//!
//! Deterministic on purpose. It is the end-to-end proof of the A2UI path, and
//! the seam a model-backed agent would replace: same surface, same catalog,
//! same action contract.

use rv2_a2ui::{ActionMessage, CATALOG_ID, ComponentDef, ServerMessage};
use rv2_domain::ValidationError;
use serde_json::{Value, json};

pub const SURFACE_ID: &str = "runway";
pub const CALCULATE: &str = "calculate";

/// Below this many months a raise has little room to happen.
const STOP_BELOW: f64 = 6.0;
/// From this many months there is time to plan.
const CLEAR_FROM: f64 = 12.0;

/// The opening stream: the surface, an empty form, and an empty result.
#[must_use]
pub fn open() -> Vec<ServerMessage> {
    let calculate = json!({
        "event": {
            "name": CALCULATE,
            "context": {
                "cash": { "path": "/inputs/cash" },
                "burn": { "path": "/inputs/burn" }
            }
        }
    });
    vec![
        ServerMessage::create_surface(SURFACE_ID, CATALOG_ID),
        ServerMessage::update_data_model(
            SURFACE_ID,
            "/",
            json!({ "inputs": { "cash": "", "burn": "" }, "result": {} }),
        ),
        ServerMessage::update_components(
            SURFACE_ID,
            vec![
                ComponentDef::new("root", "Column").children(["intro", "form", "result"]),
                ComponentDef::new("intro", "Text")
                    .prop(
                        "text",
                        "Cash divided by net burn. Nothing here forecasts revenue growth or a raise.",
                    )
                    .prop("tone", "muted"),
                ComponentDef::new("form", "Card").children(["cash", "burn", "calculate"]),
                ComponentDef::new("cash", "UnitField")
                    .prop("label", "Cash in the bank, £")
                    .prop("prefix", "£")
                    .bind("value", "/inputs/cash"),
                ComponentDef::new("burn", "UnitField")
                    .prop("label", "Net burn per month, £")
                    .prop("prefix", "£")
                    .prop("help", "Spend minus revenue, per month.")
                    .bind("value", "/inputs/burn"),
                ComponentDef::new("calculate", "Button")
                    .prop("label", "Calculate runway")
                    .prop("action", calculate),
                ComponentDef::new("result", "Column").children(Vec::<String>::new()),
            ],
        ),
    ]
}

/// Answer one action on the runway surface.
///
/// Input the user can fix is answered on the surface, as a `stop` callout.
/// An action this surface never offers is a client defect and is refused.
///
/// # Errors
///
/// [`ValidationError`] for another surface's action or an unknown name.
pub fn act(action: &ActionMessage) -> Result<Vec<ServerMessage>, ValidationError> {
    if action.surface_id != SURFACE_ID {
        return Err(ValidationError::new(
            "surfaceId",
            format!(
                "the runway agent does not own surface {:?}",
                action.surface_id
            ),
        ));
    }
    if action.name != CALCULATE {
        return Err(ValidationError::new(
            "name",
            format!("the runway surface has no action {:?}", action.name),
        ));
    }

    let cash = amount(action.context.get("cash"));
    let burn = amount(action.context.get("burn"));
    Ok(match (cash, burn) {
        (Err(reason), _) => cannot("Cash in the bank", reason),
        (_, Err(reason)) => cannot("Net burn per month", reason),
        (Ok(cash), Ok(burn)) => answer(cash, burn),
    })
}

/// A money amount as typed: `120000`, `120,000`, `£120,000.50`.
fn amount(value: Option<&Value>) -> Result<f64, &'static str> {
    let parsed = match value {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) => {
            let cleaned: String = s
                .chars()
                .filter(|c| !matches!(c, ',' | '£' | ' '))
                .collect();
            if cleaned.is_empty() {
                return Err("is empty");
            }
            cleaned.parse::<f64>().ok()
        }
        _ => return Err("is empty"),
    };
    match parsed {
        Some(n) if n.is_finite() && n >= 0.0 => Ok(n),
        Some(n) if n < 0.0 => Err("cannot be negative"),
        _ => Err("is not a number"),
    }
}

fn cannot(field: &str, reason: &str) -> Vec<ServerMessage> {
    vec![
        ServerMessage::update_data_model(SURFACE_ID, "/result", json!({})),
        ServerMessage::update_components(
            SURFACE_ID,
            vec![
                ComponentDef::new("result", "Column").children(["verdict"]),
                ComponentDef::new("verdict", "Callout")
                    .prop("verdict", "stop")
                    .prop("title", "Cannot calculate")
                    .prop("text", format!("{field} {reason}.")),
            ],
        ),
    ]
}

fn answer(cash: f64, burn: f64) -> Vec<ServerMessage> {
    let (runway, lean, verdict, title, text) = if burn == 0.0 {
        (
            "No limit".to_owned(),
            "good",
            "clear",
            "No net burn",
            "Revenue covers spend, so cash does not run out at this rate.",
        )
    } else {
        let months = cash / burn;
        let (lean, verdict, title, text) = if months < STOP_BELOW {
            (
                "bad",
                "stop",
                "Under six months",
                "A raise often takes months to close, so this leaves little room for one. Cut burn or start now.",
            )
        } else if months < CLEAR_FROM {
            (
                "close",
                "caution",
                "Six to twelve months",
                "Enough to raise from, not enough to wait. Plan the raise or the cuts now.",
            )
        } else {
            (
                "good",
                "clear",
                "A year or more",
                "Time to plan. Recalculate when burn changes.",
            )
        };
        (format_months(months), lean, verdict, title, text)
    };

    vec![
        ServerMessage::update_data_model(
            SURFACE_ID,
            "/result",
            json!({ "runway": runway, "cash": money(cash), "burn": money(burn) }),
        ),
        ServerMessage::update_components(
            SURFACE_ID,
            vec![
                ComponentDef::new("result", "Column").children(["figure", "figures", "verdict"]),
                ComponentDef::new("figure", "ResultPanel")
                    .prop("label", "Runway")
                    .prop("context", "at current net burn")
                    .prop("lean", lean)
                    .bind("value", "/result/runway"),
                ComponentDef::new("figures", "StatStrip").prop(
                    "stats",
                    json!([
                        { "label": "Cash", "value": { "path": "/result/cash" } },
                        { "label": "Net burn / month", "value": { "path": "/result/burn" } }
                    ]),
                ),
                ComponentDef::new("verdict", "Callout")
                    .prop("verdict", verdict)
                    .prop("title", title)
                    .prop("text", text),
            ],
        ),
    ]
}

fn format_months(months: f64) -> String {
    if months >= 1200.0 {
        "Over 100 years".to_owned()
    } else {
        format!("{months:.1} months")
    }
}

/// Whole pounds with thousands separators: `£120,000`.
fn money(amount: f64) -> String {
    // `amount` is finite and non-negative (see `amount`), so the cast saturates
    // only above u64::MAX pounds.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let whole = amount.round() as u64;
    let digits = whole.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    out.push('£');
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}
