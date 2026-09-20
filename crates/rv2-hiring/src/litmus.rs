//! The Litmus Submission filter group.
//!
//! The 9-to-6.com Litmus questionnaire asks eight substantive questions about a
//! career move. All eight are filters on this page, which took a decision: four
//! of them read company or role data directly, and four needed a derivation
//! invented for them, because the register holds nothing that answers "how soon
//! are you looking to move".
//!
//! The rule applied to every derivation is that it must be able to change the
//! result. A control that cannot is worse than no control — it reads as a
//! filter, and it is decoration. Each derived axis below therefore names the
//! register fact it reduces to:
//!
//! | Question | Reduces to |
//! |---|---|
//! | Level of role | the role's title band |
//! | Markets | the company's reach, derived from listing and role locations |
//! | Countries | the role's location string |
//! | Organisation scale | the company's scale band, derived from headcount |
//! | Industries | the company's sector |
//! | Current situation | which title bands are worth showing |
//! | How soon | whether there is a route to apply today |
//! | Main challenge | which obstacle the list should be arranged to remove |
//!
//! The ninth question, "Investment Alignment", asks whether the reader would
//! pay the advisory firm's fee. It is about the firm, not the search, so it is
//! recorded on the profile and is not a filter.

use serde::{Deserialize, Serialize};

use crate::mandate::{LevelBand, MandateSignal};
use crate::model::{Company, MarketReach, OrgScale, Role};
use crate::text::words;

/// "Which of the following best describes your current situation?"
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Situation {
    AdvancingSameTrack,
    MovingHigher,
    TransitioningAcrossFunctions,
    ShiftingToAdvisory,
}

/// "How soon are you looking to make a move?"
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Timing {
    Exploring,
    ThreeToSixMonths,
    OneToThreeMonths,
    Immediately,
}

/// "What is the main challenge you're facing?"
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Challenge {
    LimitedInterviewTraction,
    ConsideredBelowMyLevel,
    ProfileNotReflectingScope,
    UnclearPositioning,
}

impl Challenge {
    /// Whether this answer arranges the page rather than shortening the list.
    ///
    /// "Unclear positioning in the market" is answered by showing the reader
    /// what each role's description actually says about scope, so it turns the
    /// mandate quotes on instead of removing rows. It is the one axis option
    /// that changes the rendering rather than the result, and saying so here
    /// stops a later reader filing it as a broken filter.
    #[must_use]
    pub fn shows_evidence_instead_of_filtering(self) -> bool {
        matches!(self, Self::UnclearPositioning)
    }
}

/// Titles that name a function other than general software engineering, which
/// is what "transitioning across functions" reduces to.
const CROSS_FUNCTION: &[&str] = &[
    "ai",
    "machine learning",
    "ml",
    "data",
    "platform",
    "product",
    "research",
    "developer relations",
    "enablement",
    "security",
    "infrastructure",
];

/// One reader's answers. Every field empty means the axis is off.
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LitmusFilter {
    pub levels: Vec<LevelBand>,
    pub markets: Vec<MarketReach>,
    /// Free text from "Which countries are you focused on?", matched against a
    /// role's location string.
    pub countries: Vec<String>,
    pub scales: Vec<OrgScale>,
    /// Free text from "Which industries are relevant to your background?",
    /// matched against a company's sector.
    pub industries: Vec<String>,
    pub situations: Vec<Situation>,
    pub timings: Vec<Timing>,
    pub challenges: Vec<Challenge>,
}

impl LitmusFilter {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
            && self.markets.is_empty()
            && self.countries.is_empty()
            && self.scales.is_empty()
            && self.industries.is_empty()
            && self.situations.is_empty()
            && self.timings.is_empty()
            && self.challenges.is_empty()
    }

    /// How many axes are switched on, for the "(n on)" count beside the rail.
    #[must_use]
    pub fn active_axes(&self) -> usize {
        usize::from(!self.levels.is_empty())
            + usize::from(!self.markets.is_empty())
            + usize::from(!self.countries.is_empty())
            + usize::from(!self.scales.is_empty())
            + usize::from(!self.industries.is_empty())
            + usize::from(!self.situations.is_empty())
            + usize::from(!self.timings.is_empty())
            + usize::from(!self.challenges.is_empty())
    }

    /// Whether the page should show every role's mandate quotes by default.
    #[must_use]
    pub fn shows_mandate_evidence(&self) -> bool {
        self.challenges
            .iter()
            .any(|c| c.shows_evidence_instead_of_filtering())
    }

    /// The title bands the "current situation" answers imply.
    fn situation_bands(&self) -> Vec<LevelBand> {
        let mut bands = Vec::new();
        for situation in &self.situations {
            match situation {
                Situation::AdvancingSameTrack => bands.push(LevelBand::HeadLead),
                Situation::MovingHigher => {
                    bands.push(LevelBand::Director);
                    bands.push(LevelBand::VpGm);
                }
                Situation::ShiftingToAdvisory => bands.push(LevelBand::CLevel),
                // Handled by title vocabulary rather than by band, because
                // moving across functions is orthogonal to moving up.
                Situation::TransitioningAcrossFunctions => {}
            }
        }
        bands
    }

    /// Whether a role survives every axis that is switched on.
    #[must_use]
    pub fn matches_role(&self, company: &Company, role: &Role) -> bool {
        if !self.levels.is_empty() && !self.levels.contains(&role.level()) {
            return false;
        }

        if !self.markets.is_empty()
            && !company
                .market_reach
                .is_some_and(|reach| self.markets.contains(&reach))
        {
            return false;
        }

        if !self.countries.is_empty() {
            let where_it_is = words(&role.location);
            if !self
                .countries
                .iter()
                .any(|c| where_it_is.contains(&words(c).trim_end().to_string()))
            {
                return false;
            }
        }

        if !self.scales.is_empty()
            && !company
                .org_scale
                .is_some_and(|scale| self.scales.contains(&scale))
        {
            return false;
        }

        if !self.industries.is_empty() {
            let sector = words(company.sector());
            if !self
                .industries
                .iter()
                .any(|i| sector.contains(&words(i).trim_end().to_string()))
            {
                return false;
            }
        }

        if !self.situations.is_empty() && !self.matches_situation(role) {
            return false;
        }

        if !self.timings.is_empty() && !self.matches_timing(company) {
            return false;
        }

        if !self.challenges.is_empty() && !self.matches_challenge(company, role) {
            return false;
        }

        true
    }

    fn matches_situation(&self, role: &Role) -> bool {
        let bands = self.situation_bands();
        if !bands.is_empty() && bands.contains(&role.level()) {
            return true;
        }
        if self
            .situations
            .contains(&Situation::TransitioningAcrossFunctions)
        {
            let title = words(&role.title);
            if CROSS_FUNCTION
                .iter()
                .any(|f| title.contains(&format!(" {f} ")))
            {
                return true;
            }
        }
        if self.situations.contains(&Situation::ShiftingToAdvisory)
            && matches!(
                role.engagement,
                crate::engagement::Engagement::FractionalInterim
            )
        {
            return true;
        }
        false
    }

    /// "How soon" reduces to whether there is a route to apply, which is the
    /// only thing the register knows that changes with urgency.
    fn matches_timing(&self, company: &Company) -> bool {
        let readable_feed = company.ats.is_some();
        let live_link = company
            .link
            .as_ref()
            .is_some_and(|l| matches!(l.status, crate::model::LinkStatus::Live));
        let has_openings = company.openings.is_some();

        self.timings.iter().any(|timing| match timing {
            Timing::Immediately => readable_feed && live_link,
            Timing::OneToThreeMonths => has_openings,
            Timing::ThreeToSixMonths => has_openings || company.hires_engineers_in_uk(),
            Timing::Exploring => true,
        })
    }

    fn matches_challenge(&self, company: &Company, role: &Role) -> bool {
        self.challenges.iter().any(|challenge| match challenge {
            Challenge::LimitedInterviewTraction => {
                company.ats.is_some()
                    && company
                        .link
                        .as_ref()
                        .is_some_and(|l| matches!(l.status, crate::model::LinkStatus::Live))
            }
            Challenge::ConsideredBelowMyLevel => role.level().is_litmus_target(),
            Challenge::ProfileNotReflectingScope => role.mandate.as_ref().is_some_and(|m| {
                m.signals.contains(&MandateSignal::OwnsDepartment)
                    || m.signals.contains(&MandateSignal::HireAndGrow)
            }),
            Challenge::UnclearPositioning => true,
        })
    }
}

impl Company {
    /// The descriptive fields live in the flattened catch-all, so reading one
    /// goes through a named accessor rather than a string key at every call
    /// site.
    #[must_use]
    pub fn sector(&self) -> &str {
        self.rest
            .get("sector")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    #[must_use]
    pub fn hires_engineers_in_uk(&self) -> bool {
        self.rest
            .get("hiresEngineersInUk")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    }

    #[must_use]
    pub fn employee_scale(&self) -> &str {
        self.rest
            .get("employeeScale")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    #[must_use]
    pub fn exchange(&self) -> &str {
        self.rest
            .get("exchange")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }
}

/// Derive the organisation scale band from headcount and listing.
///
/// The Litmus question asks in euros of revenue and the register holds neither
/// revenue nor currency, so this is an approximation and the page says so
/// beside the value. Headcount is the closest honest proxy the register has.
#[must_use]
pub fn derive_org_scale(company: &Company) -> (OrgScale, String) {
    let scale = company.employee_scale();
    let folded = words(scale);
    let listed = !company.exchange().is_empty()
        && company.exchange() != "Private"
        && company.exchange() != "Unknown";

    let band = if folded.contains(" 10k ") || folded.contains(" 50k ") {
        OrgScale::GlobalCorporate
    } else if folded.contains(" 1k ") || folded.contains(" 5k ") {
        OrgScale::Over500m
    } else if folded.contains(" 500 ") || folded.contains(" 250 ") {
        OrgScale::From50mTo500m
    } else if scale.is_empty() {
        if listed {
            OrgScale::Over500m
        } else {
            OrgScale::Unknown
        }
    } else {
        OrgScale::Under50m
    };

    let basis = if scale.is_empty() {
        if listed {
            format!("no headcount recorded; listed on {}", company.exchange())
        } else {
            "no headcount and no listing recorded".to_string()
        }
    } else {
        format!("headcount band \"{scale}\", not revenue")
    };

    (band, basis)
}

#[cfg(test)]
mod tests {
    use super::{Challenge, LitmusFilter, Situation, Timing};
    use crate::mandate::LevelBand;
    use crate::model::{Company, Role};

    fn company(json: &str) -> Company {
        serde_json::from_str(json).expect("test fixture must parse")
    }

    fn role(title: &str, location: &str) -> Role {
        serde_json::from_str(&format!(
            r#"{{"title":"{title}","location":"{location}","url":"https://example.com/1","rust":false}}"#
        ))
        .expect("test fixture must parse")
    }

    #[test]
    fn an_empty_filter_lets_every_role_through() {
        let f = LitmusFilter::default();
        assert!(f.is_empty());
        assert!(f.matches_role(&company(r#"{"name":"X"}"#), &role("Engineer", "London")));
    }

    #[test]
    fn the_level_axis_removes_roles_below_the_chosen_bands() {
        let f = LitmusFilter {
            levels: vec![LevelBand::Director, LevelBand::VpGm],
            ..LitmusFilter::default()
        };
        let c = company(r#"{"name":"X"}"#);
        assert!(f.matches_role(&c, &role("Engineering Director", "London")));
        assert!(!f.matches_role(&c, &role("Senior Software Engineer", "London")));
    }

    #[test]
    fn moving_to_a_higher_level_shows_director_and_vp_and_hides_lead() {
        let f = LitmusFilter {
            situations: vec![Situation::MovingHigher],
            ..LitmusFilter::default()
        };
        let c = company(r#"{"name":"X"}"#);
        assert!(f.matches_role(&c, &role("Engineering Director", "London")));
        assert!(!f.matches_role(&c, &role("Lead Engineer", "London")));
    }

    #[test]
    fn transitioning_across_functions_keeps_a_role_naming_another_function() {
        let f = LitmusFilter {
            situations: vec![Situation::TransitioningAcrossFunctions],
            ..LitmusFilter::default()
        };
        let c = company(r#"{"name":"X"}"#);
        assert!(f.matches_role(&c, &role("Lead Engineer Advanced AI Engineering", "London")));
        assert!(!f.matches_role(&c, &role("Backend Engineer", "London")));
    }

    #[test]
    fn immediately_keeps_only_companies_with_a_readable_feed_and_a_live_link() {
        let f = LitmusFilter {
            timings: vec![Timing::Immediately],
            ..LitmusFilter::default()
        };
        let reachable = company(
            r#"{"name":"X","ats":{"provider":"greenhouse","slug":"x","source":"research"},
                "link":{"status":"live","checkedOn":"2026-09-15"}}"#,
        );
        let blocked =
            company(r#"{"name":"Y","link":{"status":"blocked","checkedOn":"2026-09-15"}}"#);
        assert!(f.matches_role(&reachable, &role("Engineer", "London")));
        assert!(!f.matches_role(&blocked, &role("Engineer", "London")));
    }

    #[test]
    fn exploring_keeps_a_company_with_no_board_at_all() {
        let f = LitmusFilter {
            timings: vec![Timing::Exploring],
            ..LitmusFilter::default()
        };
        assert!(f.matches_role(&company(r#"{"name":"X"}"#), &role("Engineer", "London")));
    }

    #[test]
    fn being_considered_below_my_level_removes_everything_under_director() {
        let f = LitmusFilter {
            challenges: vec![Challenge::ConsideredBelowMyLevel],
            ..LitmusFilter::default()
        };
        let c = company(r#"{"name":"X"}"#);
        assert!(f.matches_role(&c, &role("VP Engineering", "London")));
        assert!(!f.matches_role(&c, &role("Lead Engineer", "London")));
    }

    #[test]
    fn unclear_positioning_shows_evidence_rather_than_shortening_the_list() {
        let f = LitmusFilter {
            challenges: vec![Challenge::UnclearPositioning],
            ..LitmusFilter::default()
        };
        assert!(f.shows_mandate_evidence());
        assert!(f.matches_role(&company(r#"{"name":"X"}"#), &role("Engineer", "London")));
    }

    #[test]
    fn the_countries_axis_matches_against_the_role_location() {
        let f = LitmusFilter {
            countries: vec!["United Kingdom".to_string()],
            ..LitmusFilter::default()
        };
        let c = company(r#"{"name":"X"}"#);
        assert!(f.matches_role(&c, &role("Engineer", "London, England, United Kingdom")));
        assert!(!f.matches_role(&c, &role("Engineer", "Barcelona, Spain")));
    }

    #[test]
    fn org_scale_is_derived_from_headcount_and_says_so() {
        let c = company(r#"{"name":"X","employeeScale":"10k+","exchange":"LSE"}"#);
        let (band, basis) = super::derive_org_scale(&c);
        assert_eq!(band, crate::model::OrgScale::GlobalCorporate);
        assert!(basis.contains("not revenue"));
    }

    #[test]
    fn eight_axes_switched_on_report_eight_active() {
        let f = LitmusFilter {
            levels: vec![LevelBand::Director],
            markets: vec![crate::model::MarketReach::International],
            countries: vec!["UK".to_string()],
            scales: vec![crate::model::OrgScale::Over500m],
            industries: vec!["Fintech".to_string()],
            situations: vec![Situation::MovingHigher],
            timings: vec![Timing::Immediately],
            challenges: vec![Challenge::ConsideredBelowMyLevel],
        };
        assert_eq!(f.active_axes(), 8);
    }
}
