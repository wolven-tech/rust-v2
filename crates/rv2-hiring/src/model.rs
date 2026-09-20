//! The register's records.
//!
//! Every field added after the first version carries `#[serde(default)]`, so
//! the 101 company records written before the classifiers existed still
//! deserialize. That is not politeness to old data: the file is hand-edited
//! between machine runs, and a schema that refused a record missing a field
//! would make every hand edit a migration.
//!
//! [`Company::rest`] is a flattened catch-all for the descriptive fields the
//! page renders and the classifiers never read — sector, locations, stack
//! notes, confidence tier. Keeping them out of the struct means a new
//! descriptive field needs no code change at all.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::engagement::{Engagement, EngagementVerdict};
use crate::mandate::{LevelBand, Mandate};
use crate::work_pattern::{WorkPattern, WorkPatternVerdict};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Register {
    pub generated_on: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links_checked_on: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openings_checked_on: Option<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    pub companies: Vec<Company>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    pub name: String,
    #[serde(default)]
    pub careers_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub evidence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ats: Option<AtsRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openings: Option<Openings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<LinkCheck>,

    /// How far the company's market reaches. Derived, and [`Company::derived_from`]
    /// says from what.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_reach: Option<MarketReach>,
    /// The organisation's scale band. Derived from headcount and listing, not
    /// from revenue — the register holds no revenue figure, and the Litmus
    /// question asks in euros. The UI must say so.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_scale: Option<OrgScale>,
    /// Whether this company engages UK people business-to-business at all,
    /// independent of any one posting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engages_b2b: Option<bool>,
    /// What each derived field above was derived from, keyed by field name, so
    /// the page can show its basis beside the value.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub derived_from: Map<String, Value>,

    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    Greenhouse,
    GreenhouseEu,
    Lever,
    LeverEu,
    Ashby,
    Smartrecruiters,
    /// Recorded but not read. A company was added to the register from a
    /// bookmarks export carrying a Workable board, and the enum had no variant
    /// for it, so `hiring-register build` failed to parse its own data file
    /// with `unknown variant \`workable\``. Recording the board a company
    /// actually uses and reading its feed are separate capabilities, and the
    /// first must not depend on the second.
    Workable,
}

impl Provider {
    /// Every provider the register records is an employer applicant-tracking
    /// system, which is evidence about engagement, not just about where the
    /// posting lives. See [`crate::engagement::PostingSource`].
    #[must_use]
    pub fn is_employer_ats(self) -> bool {
        true
    }

    /// Whether the register can read this provider's postings, as opposed to
    /// merely knowing the company uses it.
    #[must_use]
    pub fn has_reader(self) -> bool {
        !matches!(self, Self::Workable)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AtsRef {
    pub provider: Provider,
    pub slug: String,
    pub source: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum MarketReach {
    Local,
    Regional,
    International,
    MultiRegion,
}

/// The bands the Litmus question offers, which it asks in euros of revenue.
///
/// The wire names are written out rather than left to `kebab-case`, which
/// renders `From50mTo500m` as `from50m-to500m` — a spelling nobody hand-editing
/// the register would guess, in a file that is hand-edited between machine runs.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrgScale {
    #[serde(rename = "under-50m")]
    Under50m,
    #[serde(rename = "50m-500m")]
    From50mTo500m,
    #[serde(rename = "500m-plus")]
    Over500m,
    #[serde(rename = "global-corporate")]
    GlobalCorporate,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Openings {
    pub total: usize,
    pub uk: usize,
    pub uk_engineering: usize,
    pub uk_rust: usize,
    #[serde(default)]
    pub uk_rust_in_title: usize,
    pub rust: usize,
    pub checked_on: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<Role>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_read_failed: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub title: String,
    pub location: String,
    pub url: String,
    pub rust: bool,
    #[serde(default)]
    pub rust_in_title: bool,

    #[serde(default)]
    pub work_pattern: WorkPattern,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub work_pattern_note: String,
    #[serde(default)]
    pub engagement: Engagement,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub engagement_note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mandate: Option<Mandate>,
}

impl Role {
    /// Attach the three classifications to a role, keeping the words that
    /// produced each.
    pub fn classify(&mut self, description: &str, source: crate::engagement::PostingSource) {
        let WorkPatternVerdict { pattern, basis } =
            crate::work_pattern::classify(&self.location, description);
        self.work_pattern = pattern;
        self.work_pattern_note = basis;

        let EngagementVerdict { engagement, basis } =
            crate::engagement::classify(&self.title, description, source);
        self.engagement = engagement;
        self.engagement_note = basis;

        self.mandate = Some(crate::mandate::classify(&self.title, description));
    }

    /// The reader's two essential filters, both of which must hold.
    #[must_use]
    pub fn clears_essential_filters(&self) -> bool {
        self.work_pattern.at_most_quarter_office() && self.engagement.is_b2b()
    }

    #[must_use]
    pub fn level(&self) -> LevelBand {
        self.mandate
            .as_ref()
            .map_or_else(|| crate::mandate::level_band(&self.title), |m| m.level)
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkCheck {
    pub status: LinkStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_code: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,
    pub checked_on: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LinkStatus {
    Live,
    Blocked,
    Broken,
    Unreachable,
    Missing,
}

#[cfg(test)]
mod tests {
    use super::{Company, Role};
    use crate::engagement::{Engagement, PostingSource};
    use crate::work_pattern::WorkPattern;

    #[test]
    fn a_company_record_written_before_the_classifiers_existed_still_deserializes() {
        let old = r#"{
            "name": "Sage Group",
            "careersUrl": "https://www.sage.com/en-gb/company/careers/",
            "confidence": "sand",
            "sector": "Enterprise software",
            "ticker": "SGE"
        }"#;
        let c: Company = serde_json::from_str(old).expect("old records must keep deserializing");
        assert_eq!(c.name, "Sage Group");
        assert!(c.market_reach.is_none());
        assert_eq!(c.rest["sector"], "Enterprise software");
    }

    #[test]
    fn a_board_the_register_cannot_read_is_still_a_board_it_can_record() {
        let c: Company = serde_json::from_str(
            r#"{"name":"X","ats":{"provider":"workable","slug":"x","source":"bookmarks"}}"#,
        )
        .expect("an unreadable provider must not fail the parse");
        let ats = c.ats.expect("the board was recorded");
        assert_eq!(ats.provider, crate::model::Provider::Workable);
        assert!(ats.provider.is_employer_ats());
        assert!(!ats.provider.has_reader());
    }

    #[test]
    fn a_role_record_without_the_new_fields_deserializes_to_the_unknown_defaults() {
        let old = r#"{
            "title": "Lead Engineer",
            "location": "London",
            "url": "https://example.com/1",
            "rust": false
        }"#;
        let r: Role = serde_json::from_str(old).expect("old roles must keep deserializing");
        assert_eq!(r.work_pattern, WorkPattern::Unknown);
        assert_eq!(r.engagement, Engagement::Unknown);
        assert!(!r.clears_essential_filters());
    }

    #[test]
    fn classifying_a_role_fills_all_three_judgements_with_their_quotes() {
        let mut r = Role {
            title: "Lead Engineer (Advanced AI Engineering)".to_string(),
            location: "United Kingdom, Remote".to_string(),
            url: "https://example.com/1".to_string(),
            rust: false,
            rust_in_title: false,
            work_pattern: WorkPattern::Unknown,
            work_pattern_note: String::new(),
            engagement: Engagement::Unknown,
            engagement_note: String::new(),
            mandate: None,
        };
        r.classify(
            "A newly formed unit. You will bridge applied research and production MVPs and \
             build the team. 6 month contract, outside IR35.",
            PostingSource::ContractBoard,
        );

        assert_eq!(r.work_pattern, WorkPattern::RemoteUk);
        assert_eq!(r.engagement, Engagement::OutsideIr35);
        assert!(r.clears_essential_filters());
        assert!(!r.work_pattern_note.is_empty());
        assert!(!r.engagement_note.is_empty());
        assert!(r.mandate.is_some_and(|m| m.signals.len() >= 3));
    }
}
