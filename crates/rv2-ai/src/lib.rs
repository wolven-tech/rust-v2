//! Contracts for grounded, reviewable generative behaviour.
//!
//! **Layer 1, WASM-safe.** This crate owns no model, network client, storage,
//! clock, or executor. Product crates supply a runtime and keep their
//! deterministic path available. Keeping policy and validation here gives
//! browser, native, and server adapters one contract without forcing every bet
//! to ship every runtime.
//!
//! Generated output is always a draft. Callers must present an explicit
//! accept, edit, or reject action before persistence, export, payment,
//! submission, or another irreversible action.

#![forbid(unsafe_code)]

use core::future::Future;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Hard guard against a caller turning an accidental model loop into an
/// unbounded browser allocation. Products may choose any smaller budget.
pub const HARD_MAX_OUTPUT_CHARS: usize = 16_000;

/// Where inference may receive request facts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataBoundary {
    /// No request data leaves the device. Foundation default.
    #[default]
    LocalOnly,
    /// Product obtained explicit consent for one remote inference request.
    RemoteWithExplicitConsent,
}

/// Whether one fact may cross a device boundary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactSensitivity {
    /// Already published, sourceable product or public-domain fact.
    Public,
    /// Supplied by customer for this feature. Remote use still needs consent.
    #[default]
    CustomerProvided,
    /// Must remain on device even when other request facts may go remote.
    LocalOnly,
}

/// One source fact a model may use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GroundedFact {
    id: String,
    label: String,
    value: String,
    #[serde(default)]
    sensitivity: FactSensitivity,
}

impl GroundedFact {
    /// Construct a fact after trimming and validating every visible field.
    ///
    /// # Errors
    ///
    /// Returns [`AiContractError`] when id, label, or value is empty.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        value: impl Into<String>,
        sensitivity: FactSensitivity,
    ) -> Result<Self, AiContractError> {
        let id = required("fact.id", id.into())?;
        let label = required("fact.label", label.into())?;
        let value = required("fact.value", value.into())?;
        Ok(Self {
            id,
            label,
            value,
            sensitivity,
        })
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    #[must_use]
    pub const fn sensitivity(&self) -> FactSensitivity {
        self.sensitivity
    }
}

/// Validated prompt inputs. Construction defaults to local-only inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GenerationRequest {
    instruction: String,
    facts: Vec<GroundedFact>,
    max_output_chars: usize,
    #[serde(default)]
    data_boundary: DataBoundary,
}

impl GenerationRequest {
    /// Build a local-only request.
    ///
    /// # Errors
    ///
    /// Returns [`AiContractError`] for empty instructions, missing or duplicate
    /// facts, or an invalid output budget.
    pub fn local(
        instruction: impl Into<String>,
        facts: Vec<GroundedFact>,
        max_output_chars: usize,
    ) -> Result<Self, AiContractError> {
        Self::build(
            instruction.into(),
            facts,
            max_output_chars,
            DataBoundary::LocalOnly,
        )
    }

    /// Build a remote request only after product UI obtained explicit consent.
    /// Local-only facts make this fail closed.
    ///
    /// # Errors
    ///
    /// Returns [`AiContractError`] for the same invalid inputs as [`Self::local`]
    /// or when any fact is classified [`FactSensitivity::LocalOnly`].
    pub fn remote_with_explicit_consent(
        instruction: impl Into<String>,
        facts: Vec<GroundedFact>,
        max_output_chars: usize,
    ) -> Result<Self, AiContractError> {
        Self::build(
            instruction.into(),
            facts,
            max_output_chars,
            DataBoundary::RemoteWithExplicitConsent,
        )
    }

    fn build(
        instruction: String,
        facts: Vec<GroundedFact>,
        max_output_chars: usize,
        data_boundary: DataBoundary,
    ) -> Result<Self, AiContractError> {
        let instruction = required("instruction", instruction)?;
        if facts.is_empty() {
            return Err(AiContractError::MissingFacts);
        }
        if max_output_chars == 0 || max_output_chars > HARD_MAX_OUTPUT_CHARS {
            return Err(AiContractError::InvalidOutputBudget {
                requested: max_output_chars,
                hard_max: HARD_MAX_OUTPUT_CHARS,
            });
        }

        let mut ids = BTreeSet::new();
        for fact in &facts {
            if !ids.insert(fact.id.as_str()) {
                return Err(AiContractError::DuplicateFactId(fact.id.clone()));
            }
            if data_boundary == DataBoundary::RemoteWithExplicitConsent
                && fact.sensitivity == FactSensitivity::LocalOnly
            {
                return Err(AiContractError::LocalOnlyFactInRemoteRequest(
                    fact.id.clone(),
                ));
            }
        }

        Ok(Self {
            instruction,
            facts,
            max_output_chars,
            data_boundary,
        })
    }

    #[must_use]
    pub fn instruction(&self) -> &str {
        &self.instruction
    }

    #[must_use]
    pub fn facts(&self) -> &[GroundedFact] {
        &self.facts
    }

    #[must_use]
    pub const fn max_output_chars(&self) -> usize {
        self.max_output_chars
    }

    #[must_use]
    pub const fn data_boundary(&self) -> DataBoundary {
        self.data_boundary
    }
}

/// Untrusted output returned by a product-owned model runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationCandidate {
    pub text: String,
    /// Stable ids from [`GenerationRequest::facts`], not model-authored URLs.
    pub cited_fact_ids: Vec<String>,
}

/// Draft safe to show for explicit human review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewableDraft {
    text: String,
    cited_fact_ids: Vec<String>,
}

impl ReviewableDraft {
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn cited_fact_ids(&self) -> &[String] {
        &self.cited_fact_ids
    }
}

/// Validate model output against the exact request that grounded it.
///
/// # Errors
///
/// Returns [`AiContractError`] when output is empty, exceeds its declared
/// budget, has no citations, repeats a citation, or cites an unknown fact.
pub fn validate_candidate(
    request: &GenerationRequest,
    candidate: GenerationCandidate,
) -> Result<ReviewableDraft, AiContractError> {
    let text = required("candidate.text", candidate.text)?;
    let actual = text.chars().count();
    if actual > request.max_output_chars {
        return Err(AiContractError::DraftTooLong {
            actual,
            max: request.max_output_chars,
        });
    }
    if candidate.cited_fact_ids.is_empty() {
        return Err(AiContractError::MissingCitations);
    }

    let known: BTreeSet<_> = request.facts.iter().map(|fact| fact.id.as_str()).collect();
    let mut seen = BTreeSet::new();
    for id in &candidate.cited_fact_ids {
        if !known.contains(id.as_str()) {
            return Err(AiContractError::UnknownCitation(id.clone()));
        }
        if !seen.insert(id.as_str()) {
            return Err(AiContractError::DuplicateCitation(id.clone()));
        }
    }

    Ok(ReviewableDraft {
        text,
        cited_fact_ids: candidate.cited_fact_ids,
    })
}

/// Provider-neutral async seam. Product crates implement this for browser
/// workers, native runtimes, or explicitly authorised remote adapters.
pub trait DraftGenerator {
    type Error;

    fn generate(
        &self,
        request: GenerationRequest,
    ) -> impl Future<Output = Result<GenerationCandidate, Self::Error>>;
}

/// Explicit human disposition. Generated drafts have no implicit accept state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "decision", content = "text")]
pub enum DraftDecision {
    Accepted,
    Edited(EditedDraft),
    Rejected,
}

/// Non-empty human-edited text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EditedDraft(String);

impl EditedDraft {
    /// # Errors
    ///
    /// Returns [`AiContractError`] when edited text is empty.
    pub fn new(text: impl Into<String>) -> Result<Self, AiContractError> {
        required("edited_text", text.into()).map(Self)
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AiContractError {
    #[error("{0} must not be empty")]
    Empty(&'static str),
    #[error("at least one grounded fact is required")]
    MissingFacts,
    #[error("fact id must be unique: {0}")]
    DuplicateFactId(String),
    #[error("output budget {requested} must be between 1 and {hard_max} characters")]
    InvalidOutputBudget { requested: usize, hard_max: usize },
    #[error("local-only fact cannot cross remote boundary: {0}")]
    LocalOnlyFactInRemoteRequest(String),
    #[error("generated draft has {actual} characters; maximum is {max}")]
    DraftTooLong { actual: usize, max: usize },
    #[error("generated draft must cite at least one grounded fact")]
    MissingCitations,
    #[error("generated draft cites unknown fact: {0}")]
    UnknownCitation(String),
    #[error("generated draft repeats fact citation: {0}")]
    DuplicateCitation(String),
}

fn required(field: &'static str, value: String) -> Result<String, AiContractError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AiContractError::Empty(field));
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(id: &str) -> GroundedFact {
        GroundedFact::new(
            id,
            "Customer input",
            "Known value",
            FactSensitivity::CustomerProvided,
        )
        .unwrap()
    }

    #[test]
    fn requests_are_local_only_by_default() {
        let request = GenerationRequest::local("Draft a note", vec![fact("input")], 500).unwrap();

        assert_eq!(request.data_boundary(), DataBoundary::LocalOnly);
    }

    #[test]
    fn request_rejects_duplicate_grounding_ids() {
        let error = GenerationRequest::local("Draft a note", vec![fact("same"), fact("same")], 500)
            .unwrap_err();

        assert_eq!(error, AiContractError::DuplicateFactId("same".into()));
    }

    #[test]
    fn remote_request_rejects_local_only_fact() {
        let secret = GroundedFact::new(
            "private",
            "Private note",
            "Never upload",
            FactSensitivity::LocalOnly,
        )
        .unwrap();

        let error =
            GenerationRequest::remote_with_explicit_consent("Draft a note", vec![secret], 500)
                .unwrap_err();

        assert_eq!(
            error,
            AiContractError::LocalOnlyFactInRemoteRequest("private".into())
        );
    }

    #[test]
    fn candidate_must_stay_grounded_and_bounded() {
        let request = GenerationRequest::local("Draft a note", vec![fact("input")], 12).unwrap();

        assert_eq!(
            validate_candidate(
                &request,
                GenerationCandidate {
                    text: "Grounded".into(),
                    cited_fact_ids: vec!["missing".into()],
                },
            )
            .unwrap_err(),
            AiContractError::UnknownCitation("missing".into())
        );
        assert_eq!(
            validate_candidate(
                &request,
                GenerationCandidate {
                    text: "Far too long for budget".into(),
                    cited_fact_ids: vec!["input".into()],
                },
            )
            .unwrap_err(),
            AiContractError::DraftTooLong {
                actual: 23,
                max: 12
            }
        );
    }

    #[test]
    fn valid_candidate_becomes_reviewable_not_accepted() {
        let request = GenerationRequest::local("Draft a note", vec![fact("input")], 500).unwrap();
        let draft = validate_candidate(
            &request,
            GenerationCandidate {
                text: " Check input. ".into(),
                cited_fact_ids: vec!["input".into()],
            },
        )
        .unwrap();

        assert_eq!(draft.text(), "Check input.");
        assert_eq!(draft.cited_fact_ids(), &["input"]);
        assert!(EditedDraft::new("  ").is_err());
    }

    #[test]
    fn wire_boundary_names_are_stable() {
        let json = serde_json::to_string(&DataBoundary::LocalOnly).unwrap();
        assert_eq!(json, r#""local_only""#);
    }
}
