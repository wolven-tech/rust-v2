//! Whether a role lets its holder build a function and teach, or hands them a
//! squad inside someone else's.
//!
//! This replaces matching on job titles, which was the register's first attempt
//! and gets the interesting cases backwards. The role that prompted the rewrite
//! is Lloyds Banking Group's "Lead Engineer (Advanced AI Engineering)": a
//! hands-on role standing up a new unit for agentic AI adoption, bridging
//! applied research and production. On title alone it scores below every
//! Director of Engineering in the register. On mandate it outranks all of them.
//!
//! So: signals first, title band second, and a Lead Engineer carrying four
//! signals ranks above a Director carrying none.
//!
//! ## Strong and weak phrasing
//!
//! Every senior job description says "mentor junior engineers" somewhere. A
//! classifier that counted it would score every posting identically and tell
//! the reader nothing. Each signal therefore has two phrase lists: a strong
//! list that counts, and a weak list that is recorded and does not. A role
//! matching only weak phrasing scores [`MandateStrength::Boilerplate`], which
//! is a different answer from "no signals" and a useful one — it says the words
//! are there and the substance is not.

use crate::text::words;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum MandateSignal {
    /// Stands up a new team, unit, function, practice or capability.
    NewUnit,
    /// Carries hiring authority, or an explicit brief to grow the team.
    HireAndGrow,
    /// Carries a teaching, enablement or coaching remit.
    Teaching,
    /// Bridges applied research and production.
    ResearchToProduction,
    /// Zero-to-one, first engineer in, from scratch.
    Greenfield,
    /// Owns a department, practice or budget rather than a squad.
    OwnsDepartment,
}

impl MandateSignal {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::NewUnit => "Stands up a new unit",
            Self::HireAndGrow => "Hires and grows the team",
            Self::Teaching => "Teaching or enablement remit",
            Self::ResearchToProduction => "Bridges research and production",
            Self::Greenfield => "Greenfield, zero to one",
            Self::OwnsDepartment => "Owns a department",
        }
    }
}

/// Seniority read from the title. Secondary to the signals above.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum LevelBand {
    CLevel,
    VpGm,
    Director,
    HeadLead,
    #[default]
    Other,
}

impl LevelBand {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::CLevel => "C-level or advisory",
            Self::VpGm => "VP or GM",
            Self::Director => "Director",
            Self::HeadLead => "Head, lead or principal",
            Self::Other => "Below the target bands",
        }
    }

    /// The three bands the Litmus answer named: Director, VP / GM,
    /// C-level / Advisory.
    #[must_use]
    pub fn is_litmus_target(self) -> bool {
        matches!(self, Self::CLevel | Self::VpGm | Self::Director)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum MandateStrength {
    /// Three or more signals, each on strong phrasing.
    Strong,
    /// One or two signals.
    Partial,
    /// Only the phrasing every senior posting carries.
    Boilerplate,
    #[default]
    None,
}

impl MandateStrength {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Strong => "Builds and teaches",
            Self::Partial => "Some ownership",
            Self::Boilerplate => "Standard senior wording only",
            Self::None => "No department-building remit",
        }
    }
}

/// What a role's description says about the mandate, with the quotes that said
/// it.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Mandate {
    pub signals: Vec<MandateSignal>,
    /// One quote per signal, in the same order, so a reader can overrule the
    /// score without re-reading the posting.
    pub quotes: Vec<String>,
    /// Phrases that matched only the weak list, recorded and not counted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boilerplate: Vec<String>,
    pub level: LevelBand,
    pub strength: MandateStrength,
}

impl Mandate {
    /// Sort key: more signals first, then the stronger title band.
    ///
    /// Returned as a tuple so callers sort ascending and get the ranking the
    /// reader asked for without inverting anything at the call site.
    #[must_use]
    pub fn rank(&self) -> (usize, LevelBand) {
        (usize::MAX - self.signals.len(), self.level)
    }
}

/// Phrases that count, per signal.
const STRONG: &[(MandateSignal, &[&str])] = &[
    (
        MandateSignal::NewUnit,
        &[
            "new team",
            "new unit",
            "new function",
            "new practice",
            "new capability",
            "newly formed",
            "newly created team",
            "stand up a",
            "standing up a",
            "establish the practice",
            "establish a new",
            "set up the function",
            "build the function",
            "build out the function",
        ],
    ),
    (
        MandateSignal::HireAndGrow,
        &[
            "build the team",
            "build out the team",
            "grow the team",
            "grow the function",
            "hire and grow",
            "own the hiring",
            "hiring plan",
            "first hire",
            "first engineering hire",
            "recruit and develop",
            "scale the team",
            "future hires",
            "shaping the team",
            "shape the team",
        ],
    ),
    (
        MandateSignal::Teaching,
        &[
            "engineering enablement",
            "developer education",
            "developer relations",
            "technical academy",
            "upskill the team",
            "upskilling",
            "coach and develop",
            "training programme",
            "training program",
            "teach",
            "teaching",
            "raise the bar across",
            "uplift the engineering",
            "build a culture of learning",
            "evangelise",
            "evangelize",
        ],
    ),
    (
        MandateSignal::ResearchToProduction,
        &[
            "applied research",
            "research to production",
            "research into production",
            "from prototype to production",
            "production mvps",
            "production mvp",
            "bridge research",
            "bridges research",
            "proofs of concept",
            "proof of concept",
            "pocs to production",
            "poc to production",
            "productionise",
            "productionize",
        ],
    ),
    (
        MandateSignal::Greenfield,
        &[
            "greenfield",
            "zero to one",
            "0 to 1",
            "from scratch",
            "from the ground up",
            "first engineer",
            "founding engineer",
            "blank sheet",
        ],
    ),
    (
        MandateSignal::OwnsDepartment,
        &[
            "own the department",
            "lead the department",
            "head of the function",
            "own the engineering function",
            "own the practice",
            "own the budget",
            "p l responsibility",
            "own the roadmap and the team",
            "accountable for the function",
            "strategic direction",
            "define the technology strategy",
            "define the technical direction",
        ],
    ),
];

/// Phrasing every senior posting carries. Recorded, never counted.
const WEAK: &[&str] = &[
    "mentor junior engineers",
    "mentor junior",
    "mentoring",
    "mentor others",
    "mentor and support",
    "coach others",
    "share knowledge",
    "knowledge sharing",
    "lead by example",
    "champion best practices",
    "champion the principles",
    "collaborate with stakeholders",
    "pair programming",
    "code reviews",
];

const C_LEVEL: &[&str] = &[
    "chief technology officer",
    "chief technical officer",
    "chief ai officer",
    "chief data officer",
    "chief information officer",
    "cto",
    "cio",
    "cdo",
    "non executive",
    "board advisor",
    "technical advisor",
    "advisory",
];

const VP_GM: &[&str] = &[
    "vp",
    "vice president",
    "svp",
    "general manager",
    "gm",
    "managing director",
];

const DIRECTOR: &[&str] = &["director", "engineering director"];

const HEAD_LEAD: &[&str] = &[
    "head of",
    "lead engineer",
    "engineering lead",
    "tech lead",
    "technical lead",
    "team lead",
    "practice lead",
    "principal",
    "distinguished",
    "staff",
    "fellow",
    "engineering manager",
];

fn quote_around(folded: &str, phrase: &str) -> String {
    let needle = format!(" {phrase} ");
    let Some(at) = folded.find(&needle) else {
        return phrase.to_string();
    };
    let lo = folded[..at]
        .char_indices()
        .rev()
        .nth(40)
        .map_or(0, |(i, _)| i);
    let hi = (at + needle.len() + 40).min(folded.len());
    let hi = folded[..hi]
        .char_indices()
        .last()
        .map_or(folded.len(), |(i, c)| i + c.len_utf8());
    folded[lo..hi].trim().to_string()
}

/// Read the title for a seniority band.
///
/// Checked most senior first: "VP of Engineering, Director of Platform" is one
/// person's title in some companies, and the higher band is the right answer.
#[must_use]
pub fn level_band(title: &str) -> LevelBand {
    let w = words(title);
    let hit = |list: &[&str]| list.iter().any(|n| w.contains(&format!(" {n} ")));
    if hit(C_LEVEL) {
        LevelBand::CLevel
    } else if hit(VP_GM) {
        LevelBand::VpGm
    } else if hit(DIRECTOR) {
        LevelBand::Director
    } else if hit(HEAD_LEAD) {
        LevelBand::HeadLead
    } else {
        LevelBand::Other
    }
}

/// Score a role's mandate from its title and description.
#[must_use]
pub fn classify(title: &str, description: &str) -> Mandate {
    let folded = words(&format!("{title} {description}"));
    let contains = |phrase: &str| folded.contains(&format!(" {phrase} "));

    let mut signals = Vec::new();
    let mut quotes = Vec::new();
    for (signal, phrases) in STRONG {
        if let Some(phrase) = phrases.iter().copied().find(|p| contains(p)) {
            signals.push(*signal);
            quotes.push(quote_around(&folded, phrase));
        }
    }

    let boilerplate: Vec<String> = WEAK
        .iter()
        .copied()
        .filter(|p| contains(p))
        .map(str::to_string)
        .collect();

    let strength = match signals.len() {
        0 if boilerplate.is_empty() => MandateStrength::None,
        0 => MandateStrength::Boilerplate,
        1..=2 => MandateStrength::Partial,
        _ => MandateStrength::Strong,
    };

    Mandate {
        signals,
        quotes,
        boilerplate,
        level: level_band(title),
        strength,
    }
}

#[cfg(test)]
mod tests {
    use super::{LevelBand, MandateSignal, MandateStrength, classify, level_band};

    #[test]
    fn a_lead_engineer_in_a_new_ai_unit_outranks_a_director_with_no_mandate() {
        // The shape of the Lloyds role the reader named as the target.
        let lead = classify(
            "Lead Engineer (Advanced AI Engineering)",
            "You will join a newly formed unit standing up a new capability for agentic AI \
             adoption. You will bridge applied research and production MVPs, build the team \
             around you, and upskill the wider engineering community.",
        );
        let director = classify(
            "Engineering Director (ID&F) - Java, .NET, AWS",
            "Lead a group of squads delivering our identity products. Mentor junior engineers \
             and champion best practices.",
        );

        assert_eq!(lead.strength, MandateStrength::Strong);
        assert_eq!(director.strength, MandateStrength::Boilerplate);
        assert!(lead.rank() < director.rank());
    }

    #[test]
    fn mentoring_junior_engineers_alone_is_boilerplate_not_a_teaching_mandate() {
        let m = classify("Senior Engineer", "You will mentor junior engineers.");
        assert!(!m.signals.contains(&MandateSignal::Teaching));
        assert_eq!(m.strength, MandateStrength::Boilerplate);
        assert!(!m.boilerplate.is_empty());
    }

    #[test]
    fn a_posting_with_neither_strong_nor_weak_phrasing_scores_none() {
        let m = classify(
            "GO Senior Software Engineer",
            "Work on highly available services, exposed mainly by APIs. Proficient in Golang.",
        );
        assert_eq!(m.strength, MandateStrength::None);
        assert!(m.signals.is_empty());
    }

    #[test]
    fn every_counted_signal_carries_the_quote_that_produced_it() {
        let m = classify(
            "Head of Platform",
            "Build the team, own the budget, and take this greenfield platform from scratch.",
        );
        assert_eq!(m.signals.len(), m.quotes.len());
        assert!(m.quotes.iter().all(|q| !q.is_empty()));
    }

    #[test]
    fn a_title_naming_two_bands_is_read_at_the_more_senior_one() {
        assert_eq!(
            level_band("VP of Engineering, Director of Platform"),
            LevelBand::VpGm
        );
    }

    #[test]
    fn the_three_litmus_bands_are_director_and_above() {
        assert!(level_band("Engineering Director").is_litmus_target());
        assert!(level_band("VP Engineering").is_litmus_target());
        assert!(level_band("Fractional CTO").is_litmus_target());
        assert!(!level_band("Lead Engineer").is_litmus_target());
        assert!(!level_band("Senior Software Engineer").is_litmus_target());
    }

    #[test]
    fn a_lead_title_lands_in_the_head_lead_band_rather_than_other() {
        assert_eq!(
            level_band("Lead Engineer (Advanced AI Engineering)"),
            LevelBand::HeadLead
        );
        assert_eq!(level_band("Staff Software Engineer"), LevelBand::HeadLead);
        assert_eq!(level_band("GO Senior Software Engineer"), LevelBand::Other);
    }
}
