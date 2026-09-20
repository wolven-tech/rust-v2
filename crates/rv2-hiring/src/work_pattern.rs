//! How much of the week a role is spent in an office.
//!
//! The bar is **at most 25% in-office**. A five-day week makes that 1.25 days,
//! so one day a week passes and two days a week fails.
//!
//! Counting days is not enough on its own, because the cadence changes the
//! answer completely: "2 days a week" is 40% and fails, while "2 days a month"
//! is about 9% and passes comfortably. A classifier that read the number and
//! ignored the period would reject the second one, which is the pattern most
//! distributed companies actually run.

use crate::text::{first_match, has_any, words};
use serde::{Deserialize, Serialize};

/// Working days in the period a stated cadence is measured over.
const DAYS_PER_WEEK: f32 = 5.0;
const DAYS_PER_MONTH: f32 = 21.0;
const DAYS_PER_QUARTER: f32 = 63.0;

/// The ceiling the reader set: a quarter of working time, at most.
const OFFICE_CEILING: f32 = 0.25;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WorkPattern {
    /// No office tie at all.
    RemoteAnywhere,
    /// Remote, but the company wants the person in the UK.
    RemoteUk,
    /// Remote by default with an office day that stays inside the ceiling.
    RemoteFirstOneDay,
    /// Two or more office days a week, or a bare "hybrid" with no stated cap.
    HybridTwoPlus,
    /// Named an office and nothing else.
    Onsite,
    /// Nothing in the posting decides it.
    #[default]
    Unknown,
}

impl WorkPattern {
    /// Whether this pattern clears the 25% ceiling.
    ///
    /// [`WorkPattern::Unknown`] does **not** clear it. An absence is not
    /// evidence, and treating it as a pass is how a hybrid role reaches a
    /// remote-only list.
    #[must_use]
    pub fn at_most_quarter_office(self) -> bool {
        matches!(
            self,
            Self::RemoteAnywhere | Self::RemoteUk | Self::RemoteFirstOneDay
        )
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::RemoteAnywhere => "Remote, anywhere",
            Self::RemoteUk => "Remote, UK-based",
            Self::RemoteFirstOneDay => "Remote-first, up to one office day a week",
            Self::HybridTwoPlus => "Two or more office days a week",
            Self::Onsite => "In the office",
            Self::Unknown => "Working pattern not stated",
        }
    }
}

/// A classification with the words that produced it, so a reader can disagree.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkPatternVerdict {
    pub pattern: WorkPattern,
    /// The phrase that decided it, or a plain statement of what was absent.
    pub basis: String,
}

impl WorkPatternVerdict {
    fn new(pattern: WorkPattern, basis: impl Into<String>) -> Self {
        Self {
            pattern,
            basis: basis.into(),
        }
    }
}

/// The period an office-day count is measured over.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Cadence {
    Week,
    Month,
    Quarter,
}

impl Cadence {
    fn working_days(self) -> f32 {
        match self {
            Self::Week => DAYS_PER_WEEK,
            Self::Month => DAYS_PER_MONTH,
            Self::Quarter => DAYS_PER_QUARTER,
        }
    }
}

const REMOTE_ANYWHERE: &[&str] = &[
    "work from anywhere",
    "remote anywhere",
    "anywhere in the world",
    "fully distributed",
    "all remote",
    "100 remote",
];

const REMOTE_WORDS: &[&str] = &["remote", "fully remote", "remote first", "work from home"];

const UK_TIE: &[&str] = &["uk", "united kingdom", "gb", "england", "scotland", "wales"];

const ONSITE_WORDS: &[&str] = &[
    "in office",
    "on site",
    "onsite",
    "in person",
    "office based",
    "in the office",
];

const HYBRID_WORDS: &[&str] = &["hybrid"];

/// Numbers a posting spells out rather than writing as a digit.
const SPELLED: &[(&str, u8)] = &[
    ("zero", 0),
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
];

/// Words that put an office-day count in context, searched within a short
/// window either side of the count.
const OFFICE_CONTEXT: &[&str] = &[
    "office", "onsite", "site", "person", "hq", "hub", "desk", "studio",
];

fn token_number(token: &str) -> Option<u8> {
    if let Ok(n) = token.parse::<u8>() {
        return Some(n);
    }
    SPELLED
        .iter()
        .find(|(word, _)| *word == token)
        .map(|(_, n)| *n)
}

fn cadence_after(tokens: &[&str], from: usize) -> Cadence {
    // The cadence word follows the count closely: "3 days a week",
    // "2 days per month", "one day each quarter".
    let window = tokens.iter().skip(from).take(6);
    for token in window {
        match *token {
            "week" | "weekly" | "wk" => return Cadence::Week,
            "month" | "monthly" => return Cadence::Month,
            "quarter" | "quarterly" => return Cadence::Quarter,
            _ => {}
        }
    }
    // No period stated. A bare "3 days in the office" means a week in every
    // posting that has ever been written, and assuming the longest period
    // would turn a four-day office week into a pass.
    Cadence::Week
}

fn has_office_context(tokens: &[&str], at: usize) -> bool {
    let lo = at.saturating_sub(6);
    let hi = (at + 8).min(tokens.len());
    tokens[lo..hi].iter().any(|t| OFFICE_CONTEXT.contains(t))
}

/// The stated office commitment as a fraction of working time, if the text
/// states one.
fn office_fraction(folded: &str) -> Option<(f32, String)> {
    let tokens: Vec<&str> = folded.split_whitespace().collect();
    for (i, token) in tokens.iter().enumerate() {
        if *token != "day" && *token != "days" {
            continue;
        }
        let Some(count) = i.checked_sub(1).and_then(|p| token_number(tokens[p])) else {
            continue;
        };
        if !has_office_context(&tokens, i) {
            continue;
        }
        let cadence = cadence_after(&tokens, i + 1);
        let lo = i.saturating_sub(1);
        let hi = (i + 6).min(tokens.len());
        let quote = tokens[lo..hi].join(" ");
        return Some((f32::from(count) / cadence.working_days(), quote));
    }
    None
}

/// Classify a role's working pattern from its location string and description.
///
/// Order matters, and it is specific-to-general: a stated day count beats the
/// word "remote" appearing in a benefits list, and the word "hybrid" beats a
/// city name. The last rule is the one that carries the register's most common
/// case — a posting that names a city and says nothing else.
#[must_use]
pub fn classify(location: &str, description: &str) -> WorkPatternVerdict {
    let folded = words(&format!("{location} {description}"));

    if let Some((fraction, quote)) = office_fraction(&folded) {
        return if fraction > OFFICE_CEILING {
            WorkPatternVerdict::new(WorkPattern::HybridTwoPlus, format!("states \"{quote}\""))
        } else {
            WorkPatternVerdict::new(
                WorkPattern::RemoteFirstOneDay,
                format!("states \"{quote}\", inside a quarter of working time"),
            )
        };
    }

    if let Some(phrase) = first_match(&folded, REMOTE_ANYWHERE) {
        return WorkPatternVerdict::new(WorkPattern::RemoteAnywhere, format!("says \"{phrase}\""));
    }

    // "Remote-first" with no day count is the pattern this reader wants and is
    // worth separating from plain "remote", because it usually carries an
    // occasional office day that a later read should confirm.
    if folded.contains(" remote first ") {
        return WorkPatternVerdict::new(
            WorkPattern::RemoteFirstOneDay,
            "says \"remote first\" with no office day count".to_string(),
        );
    }

    if has_any(
        &folded,
        &["monthly onsite", "quarterly onsite", "occasional travel"],
    ) {
        return WorkPatternVerdict::new(
            WorkPattern::RemoteFirstOneDay,
            "onsite stated as monthly, quarterly or occasional".to_string(),
        );
    }

    if let Some(phrase) = first_match(&folded, HYBRID_WORDS) {
        // A bare "hybrid" is the single most common way a posting hides a
        // three-day office week. The absence of a number is the finding, so it
        // is recorded rather than resolved.
        return WorkPatternVerdict::new(
            WorkPattern::HybridTwoPlus,
            format!("says \"{phrase}\" with no stated office day count"),
        );
    }

    if let Some(phrase) = first_match(&folded, REMOTE_WORDS) {
        return if has_any(&folded, UK_TIE) {
            WorkPatternVerdict::new(
                WorkPattern::RemoteUk,
                format!("says \"{phrase}\" and ties the role to the UK"),
            )
        } else {
            WorkPatternVerdict::new(WorkPattern::RemoteAnywhere, format!("says \"{phrase}\""))
        };
    }

    if let Some(phrase) = first_match(&folded, ONSITE_WORDS) {
        return WorkPatternVerdict::new(WorkPattern::Onsite, format!("says \"{phrase}\""));
    }

    if location.trim().is_empty() {
        return WorkPatternVerdict::new(
            WorkPattern::Unknown,
            "no location and no working-pattern wording".to_string(),
        );
    }

    WorkPatternVerdict::new(
        WorkPattern::Onsite,
        format!("names \"{}\" with no remote wording", location.trim()),
    )
}

#[cfg(test)]
mod tests {
    use super::{WorkPattern, classify};

    #[test]
    fn a_location_of_in_office_is_read_as_onsite() {
        // Cloudflare posts this literal string as the location.
        let v = classify("In-Office", "");
        assert_eq!(v.pattern, WorkPattern::Onsite);
    }

    #[test]
    fn a_uk_remote_location_string_is_read_as_remote_uk() {
        // Grafana Labs and Datadog both post in this shape.
        assert_eq!(
            classify("UK | Remote | United Kingdom (Remote)", "").pattern,
            WorkPattern::RemoteUk
        );
        assert_eq!(
            classify("United Kingdom, Remote", "").pattern,
            WorkPattern::RemoteUk
        );
    }

    #[test]
    fn a_city_or_remote_location_is_read_as_remote_uk() {
        // Monzo posts this shape.
        assert_eq!(
            classify("Cardiff, London or Remote (UK)", "").pattern,
            WorkPattern::RemoteUk
        );
    }

    #[test]
    fn a_bare_city_with_no_remote_wording_is_read_as_onsite() {
        // Thunes posts this shape, and so does most of the register.
        let v = classify(
            "London, England, United Kingdom",
            "Work on highly available services",
        );
        assert_eq!(v.pattern, WorkPattern::Onsite);
        assert!(v.basis.contains("no remote wording"));
    }

    #[test]
    fn three_office_days_a_week_fails_the_ceiling() {
        let v = classify("London", "We work 3 days in the office each week");
        assert_eq!(v.pattern, WorkPattern::HybridTwoPlus);
        assert!(!v.pattern.at_most_quarter_office());
    }

    #[test]
    fn two_office_days_a_week_fails_the_ceiling() {
        assert_eq!(
            classify("London", "two days a week in the office").pattern,
            WorkPattern::HybridTwoPlus
        );
    }

    #[test]
    fn one_office_day_a_week_clears_the_ceiling() {
        let v = classify("London", "one day a week in the office");
        assert_eq!(v.pattern, WorkPattern::RemoteFirstOneDay);
        assert!(v.pattern.at_most_quarter_office());
    }

    #[test]
    fn two_office_days_a_month_clears_the_ceiling_though_two_a_week_does_not() {
        assert!(
            classify("London", "2 days per month in the office")
                .pattern
                .at_most_quarter_office()
        );
        assert!(
            !classify("London", "2 days per week in the office")
                .pattern
                .at_most_quarter_office()
        );
    }

    #[test]
    fn a_bare_hybrid_with_no_number_fails_and_says_the_number_was_missing() {
        let v = classify("London", "This is a hybrid role");
        assert_eq!(v.pattern, WorkPattern::HybridTwoPlus);
        assert!(v.basis.contains("no stated office day count"));
    }

    #[test]
    fn a_stated_day_count_beats_the_word_remote_appearing_elsewhere() {
        let v = classify(
            "London",
            "Remote working is supported. The team is in the office 3 days a week.",
        );
        assert_eq!(v.pattern, WorkPattern::HybridTwoPlus);
    }

    #[test]
    fn an_unknown_pattern_does_not_clear_the_ceiling() {
        assert!(!WorkPattern::Unknown.at_most_quarter_office());
    }

    #[test]
    fn work_from_anywhere_is_read_as_remote_anywhere() {
        assert_eq!(
            classify("", "You can work from anywhere").pattern,
            WorkPattern::RemoteAnywhere
        );
    }

    #[test]
    fn a_quarterly_onsite_clears_the_ceiling() {
        assert!(
            classify("", "Fully distributed with a quarterly onsite")
                .pattern
                .at_most_quarter_office()
        );
    }
}
