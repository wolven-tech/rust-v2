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

use crate::engagement::{Engagement, EngagementVerdict, PostingSource};
use crate::mandate::{LevelBand, Mandate};
use crate::work_pattern::{WorkPattern, WorkPatternVerdict};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
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

impl Register {
    /// Classify every role in the register, once, at load.
    #[must_use]
    pub fn classified(mut self) -> Self {
        for company in &mut self.companies {
            company.classify_roles();
        }
        self
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
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
    /// Which kind of board this company's roles were read from, when it is not
    /// the employer applicant-tracking system named in `ats`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posting_source: Option<PostingSource>,
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

impl Company {
    /// Which kind of board this company's postings came from.
    ///
    /// An explicit `postingSource` wins. Otherwise a recorded ATS means an
    /// employer board, and no ATS at all means nothing is known — which the
    /// engagement classifier treats as undecided rather than as permanent.
    #[must_use]
    pub fn posting_source(&self) -> PostingSource {
        self.posting_source.unwrap_or(if self.ats.is_some() {
            PostingSource::PermanentAtsBoard
        } else {
            PostingSource::Unknown
        })
    }

    /// Classify any role that does not already carry a stored verdict.
    ///
    /// A stored verdict wins, and that asymmetry is the whole point. The tool
    /// that reads a job board has the posting's full description and writes its
    /// verdicts into the register; the page has only what the register holds.
    /// Reclassifying at load would silently overwrite a judgement made with the
    /// description against one made from a title alone — and a title never says
    /// how many days are in the office or whether a role builds a team.
    ///
    /// So this fills gaps for records written before the classifiers existed,
    /// and never argues with the tool that could see more than it can.
    pub fn classify_roles(&mut self) {
        let source = self.posting_source();
        if let Some(openings) = self.openings.as_mut() {
            for role in &mut openings.roles {
                if role.mandate.is_none() {
                    role.classify("", source);
                }
            }
        }
    }

    /// Every index this company sits in, defaulting to the target list.
    #[must_use]
    pub fn indices(&self) -> Vec<String> {
        let listed: Vec<String> = self
            .rest
            .get("indices")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        if listed.is_empty() {
            vec!["targets".to_string()]
        } else {
            listed
        }
    }

    #[must_use]
    pub fn ticker(&self) -> String {
        self.rest
            .get("ticker")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_uppercase()
    }

    #[must_use]
    pub fn uk_locations(&self) -> &str {
        self.rest
            .get("ukLocations")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    #[must_use]
    pub fn tech_stack_note(&self) -> &str {
        self.rest
            .get("techStackNote")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// Listed on a public market, which is where share awards can be sold.
    #[must_use]
    pub fn public_equity(&self) -> bool {
        let x = self.exchange();
        !x.is_empty() && x != "Private" && x != "Unknown"
    }

    #[must_use]
    pub fn known_rust(&self) -> bool {
        self.rest
            .get("rust")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    #[must_use]
    pub fn rust_depth(&self) -> &str {
        self.rest
            .get("rustDepth")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// The job board's display name, or a statement that there is no feed.
    #[must_use]
    pub fn feed(&self) -> &'static str {
        match self.ats.as_ref().map(|a| a.provider) {
            Some(Provider::Greenhouse | Provider::GreenhouseEu) => "Greenhouse",
            Some(Provider::Lever | Provider::LeverEu) => "Lever",
            Some(Provider::Ashby) => "Ashby",
            Some(Provider::Smartrecruiters) => "SmartRecruiters",
            Some(Provider::Workable) => "Workable",
            None => "No public job feed",
        }
    }

    #[must_use]
    pub fn link_status(&self) -> LinkStatus {
        self.link.as_ref().map_or(
            if self.careers_url.is_empty() {
                LinkStatus::Missing
            } else {
                LinkStatus::Unchecked
            },
            |l| l.status,
        )
    }

    /// Whether the company is hiring UK engineers, preferring a job-board count
    /// over the research note.
    #[must_use]
    pub fn hiring_now(&self) -> bool {
        self.openings
            .as_ref()
            .map_or_else(|| self.hires_engineers_in_uk(), |o| o.uk_engineering > 0)
    }

    /// How far this company's descriptive details were confirmed.
    #[must_use]
    pub fn confidence(&self) -> Confidence {
        self.rest
            .get("confidence")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// How central Rust is here, where the descriptions were read in full.
    #[must_use]
    pub fn depth(&self) -> Option<RustDepth> {
        self.rest
            .get("rustDepth")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// The sentence explaining the depth judgement, with the date it was read.
    #[must_use]
    pub fn depth_note(&self) -> &str {
        self.rest
            .get("rustDepthNote")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    #[must_use]
    pub fn depth_checked_on(&self) -> &str {
        self.rest
            .get("rustDepthCheckedOn")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// Whether the company sits on a London index, as opposed to being a target
    /// carried for other reasons.
    #[must_use]
    pub fn london_listed(&self) -> bool {
        self.indices().iter().any(|g| g != "targets")
    }

    /// The short badge a reader scans for: a ticker where there is one, then a
    /// recorded mark, then initials.
    #[must_use]
    pub fn mark(&self) -> String {
        let ticker = self.ticker();
        if !ticker.is_empty() {
            return ticker;
        }
        if let Some(mark) = self.rest.get("mark").and_then(|v| v.as_str())
            && !mark.is_empty()
        {
            return mark.to_string();
        }
        let words: Vec<&str> = self
            .name
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|w| !w.is_empty())
            .collect();
        if words.len() > 1 {
            words
                .iter()
                .filter_map(|w| w.chars().next())
                .take(4)
                .collect::<String>()
                .to_uppercase()
        } else {
            self.name.chars().take(4).collect::<String>().to_uppercase()
        }
    }

    #[must_use]
    pub fn rust_signal(&self) -> bool {
        self.known_rust() || self.openings.as_ref().is_some_and(|o| o.uk_rust > 0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AtsRef {
    pub provider: Provider,
    pub slug: String,
    pub source: String,
}

impl AtsRef {
    /// Where a person can see every role on this board.
    ///
    /// Lives here rather than in the tool that reads the feeds because the page
    /// links to it, and the page cannot depend on a server-side crate.
    #[must_use]
    pub fn board_url(&self) -> String {
        let slug = &self.slug;
        match self.provider {
            Provider::Greenhouse => format!("https://job-boards.greenhouse.io/{slug}"),
            Provider::GreenhouseEu => format!("https://job-boards.eu.greenhouse.io/{slug}"),
            Provider::Lever => format!("https://jobs.lever.co/{slug}"),
            Provider::LeverEu => format!("https://jobs.eu.lever.co/{slug}"),
            Provider::Ashby => format!("https://jobs.ashbyhq.com/{slug}"),
            Provider::Smartrecruiters => format!("https://careers.smartrecruiters.com/{slug}"),
            Provider::Workable => format!("https://apply.workable.com/{slug}/"),
        }
    }
}

/// How far a company's descriptive details have actually been confirmed.
///
/// Applies to locations, stack notes and headcount — never to index membership
/// or job-board counts, which are read rather than judged. A reader deciding
/// whether to trust "Newcastle, London, Manchester" needs to know which of the
/// three this is.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Confirmed directly.
    Rock,
    /// Partly inferred.
    Sand,
    /// Mostly unknown.
    #[default]
    Water,
}

impl Confidence {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Rock => "Confirmed company details",
            Self::Sand => "Partly confirmed company details",
            Self::Water => "Unconfirmed company details",
        }
    }

    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::Rock => 1,
            Self::Sand => 2,
            Self::Water => 3,
        }
    }
}

/// How central Rust is to a company's work, where someone has read the job
/// descriptions in full.
///
/// Narrower than the Rust *signal*, which only means the word appeared
/// somewhere. A company whose standard job-description boilerplate lists its
/// whole stack mentions Rust in every posting and may not write any.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RustDepth {
    First,
    Mixed,
    Adjacent,
    Secondary,
}

impl RustDepth {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::First => "Rust-first",
            Self::Mixed => "Rust and one other",
            Self::Adjacent => "Rust one option of several",
            Self::Secondary => "Rust secondary to another language",
        }
    }

    /// The colour this depth carries wherever it is shown, so the scale reads
    /// as a scale rather than as four unrelated labels.
    #[must_use]
    pub fn colour(self) -> &'static str {
        match self {
            Self::First => "#7a2e12",
            Self::Mixed => "#8a5a12",
            Self::Adjacent => "#4a5568",
            Self::Secondary => "#5a5a6a",
        }
    }

    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::First => 1,
            Self::Mixed => 2,
            Self::Adjacent => 3,
            Self::Secondary => 4,
        }
    }
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

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
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
    /// When this role was applied to, if it was.
    ///
    /// A role that has been applied to is a different state from one merely
    /// listed, and the register is the only place that survives the session
    /// where the decision was made.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub applied_on: String,
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

#[derive(Serialize, Deserialize, Clone, PartialEq)]
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
    /// A careers URL is recorded and nothing has probed it yet.
    Unchecked,
}

impl LinkStatus {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Live => "Careers link live",
            Self::Blocked => "Site blocks automated checks",
            Self::Unreachable => "Careers site did not respond",
            Self::Broken => "Careers link broken",
            Self::Missing => "No careers link found",
            Self::Unchecked => "Link not checked yet",
        }
    }

    /// Sort order, healthiest first.
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::Live => 1,
            Self::Blocked => 2,
            Self::Unreachable => 3,
            Self::Broken => 4,
            Self::Missing => 5,
            Self::Unchecked => 6,
        }
    }
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
            applied_on: String::new(),
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
