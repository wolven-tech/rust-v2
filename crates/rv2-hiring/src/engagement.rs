//! Whether a role is permanent employment or a B2B engagement the reader can
//! take through their own company.
//!
//! IR35 is the UK legislation that decides whether a contract is taxed as
//! employment. An **outside** determination leaves the engagement as genuine
//! business-to-business work; an **inside** one taxes it as employment and
//! removes the reason to take it as a contract at all.
//!
//! Three things are worth knowing before reading the rules below, because each
//! one is a seam this classifier has to keep open:
//!
//! - A **small company** end client, under the Companies Act thresholds, does
//!   not owe the determination. It stays with the contractor's own company.
//!   That is the richest outside-IR35 seam in the UK market, and it is why
//!   scale-ups appear here where banks do not.
//! - A client with **no UK entity** engaging a UK-based contractor is a
//!   different route again, and several all-remote companies use it.
//! - A title never states IR35 status. This reads descriptions.
//!
//! The classifier does not guess. Where a posting says nothing, the answer is
//! [`Engagement::Unknown`], and Unknown fails the filter — the reader asked for
//! B2B, and "it might be" is not that.

use crate::text::{first_match, words};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Engagement {
    /// The posting states an outside-IR35 determination.
    OutsideIr35,
    /// A day rate through the contractor's own company, or a statement of work.
    B2bDayRate,
    /// Fractional or interim leadership, contracted business-to-business.
    FractionalInterim,
    /// An inside determination, or an umbrella company.
    InsideIr35,
    /// Permanent employment, including fixed-term employment.
    Permanent,
    /// The posting does not say.
    #[default]
    Unknown,
}

impl Engagement {
    /// Whether this is an engagement the reader can take through their own
    /// company.
    #[must_use]
    pub fn is_b2b(self) -> bool {
        matches!(
            self,
            Self::OutsideIr35 | Self::B2bDayRate | Self::FractionalInterim
        )
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::OutsideIr35 => "Contract, outside IR35",
            Self::B2bDayRate => "Business-to-business day rate",
            Self::FractionalInterim => "Fractional or interim, contracted",
            Self::InsideIr35 => "Contract, inside IR35",
            Self::Permanent => "Permanent employment",
            Self::Unknown => "Engagement not stated",
        }
    }
}

/// Where the posting was read from, which is evidence in its own right.
///
/// Greenhouse, Lever, Ashby and `SmartRecruiters` are employer applicant-tracking
/// systems. A company advertising through one is advertising a job it intends to
/// fill with an employee, and 103 of the register's 103 roles arrive this way.
/// Treating that silence as "might be contract" would fill the reader's list
/// with roles that are not.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PostingSource {
    /// An employer ATS board: Greenhouse, Lever, Ashby, `SmartRecruiters`.
    PermanentAtsBoard,
    /// A board that carries outside-IR35 roles only.
    ///
    /// The determination is a property of the board's admission policy, not of
    /// the posting's words: reading 30 listings off the UK's specialist
    /// outside-IR35 board found the status stated in the page headline and in
    /// none of the descriptions. Classifying those from description text alone
    /// returns Unknown for all 30 and throws away the strongest engagement
    /// evidence the register has.
    OutsideIr35Board,
    /// A contract or freelance marketplace, where silence means nothing.
    ContractBoard,
    #[default]
    Unknown,
}

/// A classification with the words that produced it.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EngagementVerdict {
    pub engagement: Engagement,
    pub basis: String,
}

impl EngagementVerdict {
    fn new(engagement: Engagement, basis: impl Into<String>) -> Self {
        Self {
            engagement,
            basis: basis.into(),
        }
    }
}

const INSIDE: &[&str] = &[
    "inside ir35",
    "inside of ir35",
    "umbrella company",
    "via umbrella",
    "umbrella only",
    "deemed employment",
    "employed status",
];

const OUTSIDE: &[&str] = &[
    "outside ir35",
    "outside of ir35",
    "outside the scope of ir35",
    "ir35 exempt",
    "small company exemption",
];

const FRACTIONAL: &[&str] = &[
    "fractional",
    "interim",
    "part time cto",
    "cto as a service",
    "advisory engagement",
];

const B2B: &[&str] = &[
    "day rate",
    "daily rate",
    "per day rate",
    "limited company",
    "own limited company",
    "personal service company",
    "statement of work",
    "consultancy engagement",
    "independent contractor",
    "b2b contract",
    "freelance contract",
    "contract role",
    "contract position",
    "contractor role",
];

const PERMANENT: &[&str] = &[
    "permanent",
    "permanent role",
    "permanent position",
    "full time employee",
    "fte",
    "fixed term contract",
    "fixed term employment",
    "employment contract",
    "paye",
];

/// Classify a role's engagement basis from its title, description and source.
///
/// Order is inside-before-outside on purpose: a posting that says "this role is
/// inside IR35, not outside" contains both phrases, and the restrictive reading
/// is the safe one when the cost of a wrong pass is an application to work the
/// reader cannot take.
#[must_use]
pub fn classify(title: &str, description: &str, source: PostingSource) -> EngagementVerdict {
    let folded = words(&format!("{title} {description}"));

    if let Some(phrase) = first_match(&folded, INSIDE) {
        return EngagementVerdict::new(Engagement::InsideIr35, format!("says \"{phrase}\""));
    }

    if let Some(phrase) = first_match(&folded, OUTSIDE) {
        return EngagementVerdict::new(Engagement::OutsideIr35, format!("says \"{phrase}\""));
    }

    // Fixed-term is checked before the fractional and B2B word lists because it
    // is employment wearing a contract's vocabulary, and "fixed term contract"
    // contains the word every contract list is looking for.
    if let Some(phrase) = first_match(&folded, &["fixed term contract", "fixed term employment"]) {
        return EngagementVerdict::new(
            Engagement::Permanent,
            format!("says \"{phrase}\", which is employment for a fixed period"),
        );
    }

    if let Some(phrase) = first_match(&folded, FRACTIONAL) {
        return EngagementVerdict::new(Engagement::FractionalInterim, format!("says \"{phrase}\""));
    }

    if let Some(phrase) = first_match(&folded, B2B) {
        return EngagementVerdict::new(Engagement::B2bDayRate, format!("says \"{phrase}\""));
    }

    if let Some(phrase) = first_match(&folded, PERMANENT) {
        return EngagementVerdict::new(Engagement::Permanent, format!("says \"{phrase}\""));
    }

    match source {
        PostingSource::OutsideIr35Board => EngagementVerdict::new(
            Engagement::OutsideIr35,
            "silent on engagement, and listed on a board that admits outside-IR35 roles only"
                .to_string(),
        ),
        PostingSource::PermanentAtsBoard => EngagementVerdict::new(
            Engagement::Permanent,
            "silent on engagement, and advertised through an employer applicant-tracking board"
                .to_string(),
        ),
        PostingSource::ContractBoard | PostingSource::Unknown => EngagementVerdict::new(
            Engagement::Unknown,
            "the posting does not say how the engagement is contracted".to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{Engagement, PostingSource, classify};

    #[test]
    fn an_outside_determination_is_a_b2b_engagement() {
        let v = classify(
            "Lead Engineer",
            "6 month contract, outside IR35",
            PostingSource::ContractBoard,
        );
        assert_eq!(v.engagement, Engagement::OutsideIr35);
        assert!(v.engagement.is_b2b());
    }

    #[test]
    fn an_inside_determination_is_not_a_b2b_engagement() {
        let v = classify(
            "Lead Engineer",
            "Inside IR35, umbrella only",
            PostingSource::ContractBoard,
        );
        assert_eq!(v.engagement, Engagement::InsideIr35);
        assert!(!v.engagement.is_b2b());
    }

    #[test]
    fn a_posting_naming_both_determinations_takes_the_restrictive_one() {
        let v = classify(
            "Platform Lead",
            "This engagement is inside IR35 and cannot be offered outside IR35.",
            PostingSource::ContractBoard,
        );
        assert_eq!(v.engagement, Engagement::InsideIr35);
    }

    #[test]
    fn a_fixed_term_contract_is_employment_not_a_contract_engagement() {
        let v = classify(
            "Head of Engineering",
            "12 month fixed term contract with full benefits",
            PostingSource::ContractBoard,
        );
        assert_eq!(v.engagement, Engagement::Permanent);
        assert!(!v.engagement.is_b2b());
    }

    #[test]
    fn a_fractional_title_is_a_b2b_engagement() {
        let v = classify("Fractional CTO", "", PostingSource::Unknown);
        assert_eq!(v.engagement, Engagement::FractionalInterim);
        assert!(v.engagement.is_b2b());
    }

    #[test]
    fn a_day_rate_through_a_limited_company_is_a_b2b_engagement() {
        let v = classify(
            "Principal Engineer",
            "£750 day rate, paid to your own limited company",
            PostingSource::ContractBoard,
        );
        assert_eq!(v.engagement, Engagement::B2bDayRate);
    }

    #[test]
    fn a_silent_posting_on_an_employer_ats_board_is_read_as_permanent() {
        // Every one of the register's 103 roles arrives this way.
        let v = classify(
            "GO Senior Software Engineer",
            "Work on highly available services, exposed mainly by APIs",
            PostingSource::PermanentAtsBoard,
        );
        assert_eq!(v.engagement, Engagement::Permanent);
        assert!(v.basis.contains("applicant-tracking board"));
    }

    #[test]
    fn a_silent_posting_on_an_outside_ir35_board_takes_the_board_as_the_evidence() {
        let v = classify(
            "Head Of Artificial Intelligence",
            "Define the strategic direction for AI initiatives across the enterprise.",
            PostingSource::OutsideIr35Board,
        );
        assert_eq!(v.engagement, Engagement::OutsideIr35);
        assert!(v.engagement.is_b2b());
        assert!(v.basis.contains("outside-IR35 roles only"));
    }

    #[test]
    fn a_silent_posting_from_an_unknown_source_stays_unknown_and_fails_the_filter() {
        let v = classify("Engineer", "Build things", PostingSource::Unknown);
        assert_eq!(v.engagement, Engagement::Unknown);
        assert!(!v.engagement.is_b2b());
    }
}
