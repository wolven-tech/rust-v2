//! The filtered view, as a spreadsheet.
//!
//! Export lives here rather than in the page because it has to agree with what
//! the page shows. Two implementations of "the current view" drift, and the one
//! that drifts silently is the one nobody looks at until a column is wrong in a
//! file they already sent somewhere.
//!
//! Every row is a company, and the role columns collapse that company's
//! surviving roles into one cell. A row per role would be a different document
//! — useful, and not this one.

use crate::model::{Company, Register};
use crate::view::{Filters, apply, group_label};

/// The header row, which is also the column order.
const COLUMNS: &[&str] = &[
    "Company",
    "Ticker",
    "Index",
    "Sector",
    "UK locations",
    "People",
    "Company details confirmed",
    "UK engineering roles open",
    "UK roles open",
    "UK postings mentioning Rust",
    "Rust in the title",
    "Known Rust use",
    "How Rust-first",
    "Rust evidence",
    "Listed equity",
    "Roles shown",
    "Roles remote at 25% or less",
    "Roles B2B outside IR35",
    "Roles that build a department",
    "Role titles",
    "Job board",
    "Careers URL",
    "Careers link status",
    "Link checked on",
];

/// Escape one field.
///
/// Everything is quoted rather than only the fields that need it. A quoted
/// field is always valid, and deciding per-field is how a company name with a
/// comma in it splits a row in someone's spreadsheet.
fn cell(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn row(company: &Company, filters: &Filters) -> String {
    let roles = filters.surviving_roles(company);
    let openings = company.openings.as_ref();
    let count = |n: Option<usize>| n.map_or(String::new(), |n| n.to_string());

    let fields = [
        company.name.clone(),
        company.ticker(),
        company
            .indices()
            .iter()
            .map(|g| group_label(g).to_string())
            .collect::<Vec<_>>()
            .join("; "),
        company.sector().to_string(),
        company.uk_locations().to_string(),
        company.employee_scale().to_string(),
        company.confidence().label().to_string(),
        count(openings.map(|o| o.uk_engineering)),
        count(openings.map(|o| o.uk)),
        count(openings.map(|o| o.uk_rust)),
        count(openings.map(|o| o.uk_rust_in_title)),
        if company.known_rust() { "yes" } else { "no" }.to_string(),
        company
            .depth()
            .map(|d| d.label().to_string())
            .unwrap_or_default(),
        company.depth_note().to_string(),
        if company.public_equity() {
            format!("{} {}", company.exchange(), company.ticker())
                .trim()
                .to_string()
        } else {
            "Private".to_string()
        },
        roles.len().to_string(),
        roles
            .iter()
            .filter(|r| r.work_pattern.at_most_quarter_office())
            .count()
            .to_string(),
        roles
            .iter()
            .filter(|r| r.engagement.is_b2b())
            .count()
            .to_string(),
        roles
            .iter()
            .filter(|r| {
                r.mandate
                    .as_ref()
                    .is_some_and(|m| matches!(m.strength, crate::mandate::MandateStrength::Strong))
            })
            .count()
            .to_string(),
        roles
            .iter()
            .map(|r| r.title.clone())
            .collect::<Vec<_>>()
            .join("; "),
        company.feed().to_string(),
        company.careers_url.clone(),
        company.link_status().label().to_string(),
        company
            .link
            .as_ref()
            .map(|l| l.checked_on.clone())
            .unwrap_or_default(),
    ];

    fields.iter().map(|f| cell(f)).collect::<Vec<_>>().join(",")
}

/// Render the companies that survive `filters`, in the order the page shows
/// them.
#[must_use]
pub fn export(register: &Register, filters: &Filters) -> String {
    let mut out = String::new();
    out.push_str(
        &COLUMNS
            .iter()
            .map(|c| cell(c))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');
    for company in apply(register, filters) {
        out.push_str(&row(company, filters));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{COLUMNS, cell, export};
    use crate::model::Register;
    use crate::view::Filters;

    fn register() -> Register {
        serde_json::from_str::<Register>(include_str!("../data/companies.json"))
            .expect("the register must parse")
            .classified()
    }

    #[test]
    fn a_comma_in_a_company_name_cannot_split_a_row() {
        assert_eq!(cell("Marks, Spencer"), "\"Marks, Spencer\"");
    }

    #[test]
    fn a_quote_in_a_field_is_doubled_rather_than_ending_it() {
        assert_eq!(cell("the \"best\" firm"), "\"the \"\"best\"\" firm\"");
    }

    #[test]
    fn the_export_has_one_row_per_surviving_company_plus_a_header() {
        let r = register();
        let f = Filters::default();
        let lines = export(&r, &f).lines().count();
        assert_eq!(lines, r.companies.len() + 1);
    }

    #[test]
    fn every_row_carries_exactly_as_many_fields_as_the_header() {
        let r = register();
        let csv = export(&r, &Filters::default());
        for (n, line) in csv.lines().enumerate() {
            let fields = line.split("\",\"").count();
            assert_eq!(
                fields,
                COLUMNS.len(),
                "line {n} has {fields} fields, header has {}",
                COLUMNS.len()
            );
        }
    }

    #[test]
    fn the_export_follows_the_filters_rather_than_dumping_the_register() {
        let r = register();
        let filtered = Filters {
            remote_only: true,
            b2b_only: true,
            ..Filters::default()
        };
        let csv = export(&r, &filtered);
        assert_eq!(csv.lines().count(), 2, "header plus the one survivor");
        assert!(csv.contains("Outside IR35"));
        assert!(!csv.contains("Thunes"));
    }
}
