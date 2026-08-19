//! Versioned, organization-configurable risk methodology and pure scoring.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::decimal::CanonicalDecimalError;
use crate::id::validate_stable_id;
use crate::{ASSURANCE_IR_SCHEMA, CanonicalDecimal, RiskMethodologyId};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RiskMethodologyError {
    #[error("malformed matrix: {0}")]
    MalformedMatrix(&'static str),
    #[error("duplicate ordinal {0}")]
    DuplicateOrdinal(u32),
    #[error("unreachable rating: {0}")]
    UnreachableRating(String),
    #[error("invalid boundaries: overlap")]
    Overlap,
    #[error("invalid boundaries: gap")]
    Gap,
    #[error("invalid boundaries: inverted domain")]
    InvertedDomain,
    #[error("invalid boundaries: {0}")]
    InvalidBoundaries(&'static str),
    #[error("appetite exceeds tolerance")]
    AppetiteExceedsTolerance,
    #[error("out of domain: {0}")]
    OutOfDomain(String),
    #[error("mode mismatch: {0}")]
    ModeMismatch(String),
    #[error("locked methodology cannot change scoring semantics")]
    Locked,
    #[error("identity: {0}")]
    Identity(String),
    #[error("{0}")]
    Decimal(#[from] CanonicalDecimalError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScoringMode {
    Qualitative,
    SemiQuantitative,
    Quantitative,
    CustomBounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Combination {
    Matrix,
    Product,
    Sum,
    ExpectedLoss,
    Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaleLevel {
    pub id: String,
    pub label: String,
    pub ordinal: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikelihoodScale {
    pub id: String,
    pub levels: Vec<ScaleLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactScale {
    pub id: String,
    pub levels: Vec<ScaleLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingLevel {
    pub id: String,
    pub label: String,
    pub ordinal: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingScale {
    pub levels: Vec<RatingLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixCell {
    pub likelihood_ordinal: u32,
    pub impact_ordinal: u32,
    pub rating_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskMatrix {
    pub cells: Vec<MatrixCell>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumericDomain {
    pub min: CanonicalDecimal,
    pub max: CanonicalDecimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingBand {
    pub rating_id: String,
    pub min_inclusive: CanonicalDecimal,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_exclusive: Option<CanonicalDecimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskAppetite {
    pub max_rating_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskTolerance {
    pub max_rating_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceThreshold {
    pub max_rating_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskMethodology {
    pub schema_version: String,
    pub id: RiskMethodologyId,
    pub logical_id: String,
    pub revision: u32,
    pub title: String,
    pub scoring_mode: ScoringMode,
    pub likelihood_scale: LikelihoodScale,
    pub impact_scale: ImpactScale,
    pub rating_scale: RatingScale,
    pub combination: Combination,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matrix: Option<RiskMatrix>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<NumericDomain>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bands: Option<Vec<RatingBand>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    pub appetite: RiskAppetite,
    pub tolerance: RiskTolerance,
    pub acceptance_threshold: AcceptanceThreshold,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<RiskMethodologyId>,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RiskScoreInput {
    Qualitative {
        likelihood_id: String,
        impact_id: String,
    },
    SemiQuantitative {
        likelihood: u32,
        impact: u32,
    },
    Quantitative {
        probability: CanonicalDecimal,
        loss: CanonicalDecimal,
    },
    CustomBounded {
        value: CanonicalDecimal,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RiskScore {
    Qualitative {
        likelihood_ordinal: u32,
        impact_ordinal: u32,
    },
    SemiQuantitative {
        value: u32,
    },
    Quantitative {
        expected_loss: CanonicalDecimal,
    },
    CustomBounded {
        value: CanonicalDecimal,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedRating {
    pub methodology_id: RiskMethodologyId,
    pub revision: u32,
    pub rating_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoredRisk {
    pub input: RiskScoreInput,
    pub score: RiskScore,
    pub rating: DerivedRating,
}

impl RiskMethodology {
    pub fn try_new(mut draft: Self) -> Result<Self, RiskMethodologyError> {
        draft.normalize();
        validate_risk_methodology(&draft)?;
        Ok(draft)
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn supersede(&self, new_id: RiskMethodologyId) -> Result<Self, RiskMethodologyError> {
        if new_id.as_str() == self.id.as_str() {
            return Err(RiskMethodologyError::Identity(
                "self-supersession is not allowed".into(),
            ));
        }
        let mut child = self.clone();
        child.id = new_id;
        child.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| RiskMethodologyError::Identity("revision overflow".into()))?;
        child.supersedes = Some(self.id.clone());
        child.locked = false;
        child.normalize();
        validate_risk_methodology(&child)?;
        Ok(child)
    }

    fn normalize(&mut self) {
        self.likelihood_scale
            .levels
            .sort_by_key(|level| level.ordinal);
        self.impact_scale.levels.sort_by_key(|level| level.ordinal);
        self.rating_scale.levels.sort_by_key(|level| level.ordinal);
        if let Some(matrix) = &mut self.matrix {
            matrix
                .cells
                .sort_by_key(|cell| (cell.likelihood_ordinal, cell.impact_ordinal));
        }
        if let Some(bands) = &mut self.bands {
            bands.sort_by(|a, b| a.min_inclusive.cmp_numeric(&b.min_inclusive));
        }
    }
}

pub fn validate_risk_methodology(
    methodology: &RiskMethodology,
) -> Result<(), RiskMethodologyError> {
    if methodology.schema_version != ASSURANCE_IR_SCHEMA {
        return Err(RiskMethodologyError::Identity(format!(
            "schemaVersion must be {ASSURANCE_IR_SCHEMA}"
        )));
    }
    if methodology.logical_id.trim().is_empty() {
        return Err(RiskMethodologyError::Identity("logicalId is empty".into()));
    }
    validate_stable_id(&methodology.logical_id)
        .map_err(|e| RiskMethodologyError::Identity(format!("logicalId: {e}")))?;
    if methodology.revision < 1 {
        return Err(RiskMethodologyError::Identity(
            "revision must be at least 1".into(),
        ));
    }
    if methodology.title.trim().is_empty() {
        return Err(RiskMethodologyError::Identity("title is empty".into()));
    }
    if methodology
        .supersedes
        .as_ref()
        .is_some_and(|parent| parent.as_str() == methodology.id.as_str())
    {
        return Err(RiskMethodologyError::Identity(
            "self-supersession is not allowed".into(),
        ));
    }

    validate_scale("likelihood", &methodology.likelihood_scale.levels)?;
    validate_scale("impact", &methodology.impact_scale.levels)?;
    validate_rating_scale(&methodology.rating_scale)?;
    validate_mode_combination(methodology)?;
    validate_policy_thresholds(methodology)?;
    Ok(())
}

pub fn score_risk(
    methodology: &RiskMethodology,
    input: &RiskScoreInput,
) -> Result<ScoredRisk, RiskMethodologyError> {
    validate_risk_methodology(methodology)?;
    let rating_id = match (methodology.scoring_mode, input) {
        (
            ScoringMode::Qualitative,
            RiskScoreInput::Qualitative {
                likelihood_id,
                impact_id,
            },
        ) => {
            let likelihood_ordinal =
                scale_ordinal(&methodology.likelihood_scale.levels, likelihood_id)?;
            let impact_ordinal = scale_ordinal(&methodology.impact_scale.levels, impact_id)?;
            let rating_id = matrix_rating(methodology, likelihood_ordinal, impact_ordinal)?;
            return Ok(ScoredRisk {
                input: input.clone(),
                score: RiskScore::Qualitative {
                    likelihood_ordinal,
                    impact_ordinal,
                },
                rating: derived(methodology, rating_id),
            });
        }
        (
            ScoringMode::SemiQuantitative,
            RiskScoreInput::SemiQuantitative { likelihood, impact },
        ) => score_semi_quantitative(methodology, *likelihood, *impact)?,
        (ScoringMode::Quantitative, RiskScoreInput::Quantitative { probability, loss }) => {
            let expected_loss = expected_loss(methodology, probability, loss)?;
            let rating_id = band_rating(methodology, &expected_loss)?;
            return Ok(ScoredRisk {
                input: input.clone(),
                score: RiskScore::Quantitative { expected_loss },
                rating: derived(methodology, rating_id),
            });
        }
        (ScoringMode::CustomBounded, RiskScoreInput::CustomBounded { value }) => {
            let rating_id = band_rating(methodology, value)?;
            return Ok(ScoredRisk {
                input: input.clone(),
                score: RiskScore::CustomBounded {
                    value: value.clone(),
                },
                rating: derived(methodology, rating_id),
            });
        }
        (mode, _) => {
            return Err(RiskMethodologyError::ModeMismatch(format!(
                "input does not match scoring mode {mode}"
            )));
        }
    };
    Ok(rating_id)
}

fn score_semi_quantitative(
    methodology: &RiskMethodology,
    likelihood: u32,
    impact: u32,
) -> Result<ScoredRisk, RiskMethodologyError> {
    let n_l = methodology.likelihood_scale.levels.len() as u32;
    let n_i = methodology.impact_scale.levels.len() as u32;
    if likelihood < 1 || likelihood > n_l {
        return Err(RiskMethodologyError::OutOfDomain(format!(
            "likelihood {likelihood} is outside 1..={n_l}"
        )));
    }
    if impact < 1 || impact > n_i {
        return Err(RiskMethodologyError::OutOfDomain(format!(
            "impact {impact} is outside 1..={n_i}"
        )));
    }
    let (score_value, rating_id) = match methodology.combination {
        Combination::Matrix => {
            let rating_id = matrix_rating(methodology, likelihood, impact)?;
            (likelihood.saturating_mul(impact), rating_id)
        }
        Combination::Product => {
            let value = likelihood.saturating_mul(impact);
            let amount = u32_decimal(value)?;
            (value, band_rating(methodology, &amount)?)
        }
        Combination::Sum => {
            let value = likelihood.saturating_add(impact);
            let amount = u32_decimal(value)?;
            (value, band_rating(methodology, &amount)?)
        }
        other => {
            return Err(RiskMethodologyError::ModeMismatch(format!(
                "semi-quantitative combination {other:?} is not allowed"
            )));
        }
    };
    Ok(ScoredRisk {
        input: RiskScoreInput::SemiQuantitative { likelihood, impact },
        score: RiskScore::SemiQuantitative { value: score_value },
        rating: derived(methodology, rating_id),
    })
}

fn expected_loss(
    methodology: &RiskMethodology,
    probability: &CanonicalDecimal,
    loss: &CanonicalDecimal,
) -> Result<CanonicalDecimal, RiskMethodologyError> {
    let zero = CanonicalDecimal::parse("0")?;
    let one = CanonicalDecimal::parse("1")?;
    if probability.cmp_numeric(&zero) == std::cmp::Ordering::Less
        || probability.cmp_numeric(&one) == std::cmp::Ordering::Greater
    {
        return Err(RiskMethodologyError::OutOfDomain(format!(
            "probability {probability} is outside [0, 1]"
        )));
    }
    if loss.cmp_numeric(&zero) == std::cmp::Ordering::Less {
        return Err(RiskMethodologyError::OutOfDomain(format!(
            "loss {loss} is negative"
        )));
    }
    let expected = probability.times(loss);
    let domain = methodology
        .domain
        .as_ref()
        .ok_or(RiskMethodologyError::InvalidBoundaries(
            "quantitative methodology requires a domain",
        ))?;
    if !in_domain(domain, &expected) {
        return Err(RiskMethodologyError::OutOfDomain(format!(
            "expected loss {expected} is outside declared domain"
        )));
    }
    Ok(expected)
}

fn band_rating(
    methodology: &RiskMethodology,
    value: &CanonicalDecimal,
) -> Result<String, RiskMethodologyError> {
    let domain = methodology
        .domain
        .as_ref()
        .ok_or(RiskMethodologyError::InvalidBoundaries(
            "bands require a numeric domain",
        ))?;
    if !in_domain(domain, value) {
        return Err(RiskMethodologyError::OutOfDomain(format!(
            "{value} is outside declared domain"
        )));
    }
    let bands = methodology
        .bands
        .as_deref()
        .ok_or(RiskMethodologyError::InvalidBoundaries(
            "combination requires rating bands",
        ))?;
    for band in bands {
        if value.cmp_numeric(&band.min_inclusive) == std::cmp::Ordering::Less {
            continue;
        }
        match &band.max_exclusive {
            Some(upper) => {
                if value.cmp_numeric(upper) == std::cmp::Ordering::Less {
                    return Ok(band.rating_id.clone());
                }
            }
            None => return Ok(band.rating_id.clone()),
        }
    }
    Err(RiskMethodologyError::OutOfDomain(format!(
        "{value} is outside declared domain"
    )))
}

fn matrix_rating(
    methodology: &RiskMethodology,
    likelihood_ordinal: u32,
    impact_ordinal: u32,
) -> Result<String, RiskMethodologyError> {
    let matrix = methodology
        .matrix
        .as_ref()
        .ok_or(RiskMethodologyError::MalformedMatrix("missing cell"))?;
    matrix
        .cells
        .iter()
        .find(|cell| {
            cell.likelihood_ordinal == likelihood_ordinal && cell.impact_ordinal == impact_ordinal
        })
        .map(|cell| cell.rating_id.clone())
        .ok_or(RiskMethodologyError::MalformedMatrix("missing cell"))
}

fn derived(methodology: &RiskMethodology, rating_id: String) -> DerivedRating {
    DerivedRating {
        methodology_id: methodology.id.clone(),
        revision: methodology.revision,
        rating_id,
    }
}

fn scale_ordinal(levels: &[ScaleLevel], id: &str) -> Result<u32, RiskMethodologyError> {
    levels
        .iter()
        .find(|level| level.id == id)
        .map(|level| level.ordinal)
        .ok_or_else(|| RiskMethodologyError::OutOfDomain(format!("unknown level id {id}")))
}

fn u32_decimal(value: u32) -> Result<CanonicalDecimal, RiskMethodologyError> {
    Ok(CanonicalDecimal::parse(value.to_string())?)
}

fn in_domain(domain: &NumericDomain, value: &CanonicalDecimal) -> bool {
    value.cmp_numeric(&domain.min) != std::cmp::Ordering::Less
        && value.cmp_numeric(&domain.max) != std::cmp::Ordering::Greater
}

fn validate_mode_combination(methodology: &RiskMethodology) -> Result<(), RiskMethodologyError> {
    match methodology.scoring_mode {
        ScoringMode::Qualitative => {
            if methodology.combination != Combination::Matrix {
                return Err(RiskMethodologyError::ModeMismatch(
                    "qualitative requires combination matrix".into(),
                ));
            }
            validate_matrix(methodology)?;
            reject_numeric_sections(methodology)?;
        }
        ScoringMode::SemiQuantitative => match methodology.combination {
            Combination::Matrix => {
                validate_matrix(methodology)?;
                reject_numeric_sections(methodology)?;
            }
            Combination::Product | Combination::Sum => {
                if methodology.matrix.is_some() {
                    return Err(RiskMethodologyError::MalformedMatrix("extra cell"));
                }
                validate_bands(methodology)?;
            }
            other => {
                return Err(RiskMethodologyError::ModeMismatch(format!(
                    "semi-quantitative does not allow combination {other:?}"
                )));
            }
        },
        ScoringMode::Quantitative => {
            if methodology.combination != Combination::ExpectedLoss {
                return Err(RiskMethodologyError::ModeMismatch(
                    "quantitative requires combination expectedLoss".into(),
                ));
            }
            if methodology.matrix.is_some() {
                return Err(RiskMethodologyError::MalformedMatrix("extra cell"));
            }
            validate_bands(methodology)?;
        }
        ScoringMode::CustomBounded => {
            if methodology.combination != Combination::Identity {
                return Err(RiskMethodologyError::ModeMismatch(
                    "customBounded requires combination identity".into(),
                ));
            }
            if methodology.matrix.is_some() {
                return Err(RiskMethodologyError::MalformedMatrix("extra cell"));
            }
            validate_bands(methodology)?;
        }
    }
    Ok(())
}

fn reject_numeric_sections(methodology: &RiskMethodology) -> Result<(), RiskMethodologyError> {
    if methodology.domain.is_some() || methodology.bands.is_some() {
        return Err(RiskMethodologyError::InvalidBoundaries(
            "matrix combination must not declare numeric bands",
        ));
    }
    Ok(())
}

fn validate_matrix(methodology: &RiskMethodology) -> Result<(), RiskMethodologyError> {
    let matrix = methodology
        .matrix
        .as_ref()
        .ok_or(RiskMethodologyError::MalformedMatrix("missing cell"))?;
    let n_l = methodology.likelihood_scale.levels.len() as u32;
    let n_i = methodology.impact_scale.levels.len() as u32;
    let expected = n_l.saturating_mul(n_i) as usize;
    if matrix.cells.len() < expected {
        return Err(RiskMethodologyError::MalformedMatrix("missing cell"));
    }
    if matrix.cells.len() > expected {
        return Err(RiskMethodologyError::MalformedMatrix("extra cell"));
    }

    let mut seen = BTreeSet::new();
    let mut reached = BTreeSet::new();
    for cell in &matrix.cells {
        if cell.likelihood_ordinal < 1
            || cell.likelihood_ordinal > n_l
            || cell.impact_ordinal < 1
            || cell.impact_ordinal > n_i
        {
            return Err(RiskMethodologyError::MalformedMatrix("unknown ordinal"));
        }
        if !seen.insert((cell.likelihood_ordinal, cell.impact_ordinal)) {
            return Err(RiskMethodologyError::MalformedMatrix("extra cell"));
        }
        if !rating_exists(&methodology.rating_scale, &cell.rating_id) {
            return Err(RiskMethodologyError::MalformedMatrix("unknown ordinal"));
        }
        reached.insert(cell.rating_id.as_str());
    }
    for pair in (1..=n_l).flat_map(|l| (1..=n_i).map(move |i| (l, i))) {
        if !seen.contains(&pair) {
            return Err(RiskMethodologyError::MalformedMatrix("missing cell"));
        }
    }
    assert_all_ratings_reachable(&methodology.rating_scale, &reached)?;
    Ok(())
}

fn validate_bands(methodology: &RiskMethodology) -> Result<(), RiskMethodologyError> {
    let domain = methodology
        .domain
        .as_ref()
        .ok_or(RiskMethodologyError::InvalidBoundaries(
            "numeric combination requires a domain",
        ))?;
    if domain.min.cmp_numeric(&domain.max) != std::cmp::Ordering::Less {
        return Err(RiskMethodologyError::InvertedDomain);
    }
    let bands = methodology
        .bands
        .as_ref()
        .ok_or(RiskMethodologyError::InvalidBoundaries(
            "numeric combination requires bands",
        ))?;
    if bands.is_empty() {
        return Err(RiskMethodologyError::InvalidBoundaries("bands are empty"));
    }

    let mut ordered = bands.clone();
    ordered.sort_by(|a, b| a.min_inclusive.cmp_numeric(&b.min_inclusive));
    if ordered[0].min_inclusive.cmp_numeric(&domain.min) != std::cmp::Ordering::Equal {
        return Err(RiskMethodologyError::InvalidBoundaries(
            "first band must start at domain min",
        ));
    }

    let mut reached = BTreeSet::new();
    for (index, band) in ordered.iter().enumerate() {
        if !rating_exists(&methodology.rating_scale, &band.rating_id) {
            return Err(RiskMethodologyError::UnreachableRating(
                band.rating_id.clone(),
            ));
        }
        reached.insert(band.rating_id.as_str());
        let last = index + 1 == ordered.len();
        if last {
            if band.max_exclusive.is_some() {
                return Err(RiskMethodologyError::InvalidBoundaries(
                    "last band must omit maxExclusive so domain max is inclusive",
                ));
            }
            if band.min_inclusive.cmp_numeric(&domain.max) == std::cmp::Ordering::Greater {
                return Err(RiskMethodologyError::Gap);
            }
        } else {
            let Some(upper) = band.max_exclusive.as_ref() else {
                return Err(RiskMethodologyError::InvalidBoundaries(
                    "non-last band must declare maxExclusive",
                ));
            };
            if band.min_inclusive.cmp_numeric(upper) != std::cmp::Ordering::Less {
                return Err(RiskMethodologyError::InvertedDomain);
            }
            let next = &ordered[index + 1];
            match next.min_inclusive.cmp_numeric(upper) {
                std::cmp::Ordering::Less => return Err(RiskMethodologyError::Overlap),
                std::cmp::Ordering::Greater => return Err(RiskMethodologyError::Gap),
                std::cmp::Ordering::Equal => {}
            }
        }
    }
    assert_all_ratings_reachable(&methodology.rating_scale, &reached)?;
    Ok(())
}

fn validate_policy_thresholds(methodology: &RiskMethodology) -> Result<(), RiskMethodologyError> {
    let appetite = rating_ordinal(
        &methodology.rating_scale,
        &methodology.appetite.max_rating_id,
    )?;
    let tolerance = rating_ordinal(
        &methodology.rating_scale,
        &methodology.tolerance.max_rating_id,
    )?;
    let acceptance = rating_ordinal(
        &methodology.rating_scale,
        &methodology.acceptance_threshold.max_rating_id,
    )?;
    if appetite > tolerance {
        return Err(RiskMethodologyError::AppetiteExceedsTolerance);
    }
    if acceptance > tolerance {
        return Err(RiskMethodologyError::InvalidBoundaries(
            "acceptance exceeds tolerance",
        ));
    }
    Ok(())
}

fn validate_scale(kind: &str, levels: &[ScaleLevel]) -> Result<(), RiskMethodologyError> {
    if levels.is_empty() {
        return Err(RiskMethodologyError::Identity(format!(
            "{kind} scale must have at least one level"
        )));
    }
    let mut ids = BTreeSet::new();
    let mut ordinals = BTreeMap::new();
    for level in levels {
        if level.label.trim().is_empty() {
            return Err(RiskMethodologyError::Identity(format!(
                "{kind} scale label is empty"
            )));
        }
        if level.id.trim().is_empty() {
            return Err(RiskMethodologyError::Identity(format!(
                "{kind} scale id is empty"
            )));
        }
        if !ids.insert(level.id.as_str()) {
            return Err(RiskMethodologyError::Identity(format!(
                "{kind} scale duplicate id {}",
                level.id
            )));
        }
        if ordinals.insert(level.ordinal, ()).is_some() {
            return Err(RiskMethodologyError::DuplicateOrdinal(level.ordinal));
        }
        if level.ordinal < 1 {
            return Err(RiskMethodologyError::DuplicateOrdinal(level.ordinal));
        }
    }
    let n = levels.len() as u32;
    for expected in 1..=n {
        if !ordinals.contains_key(&expected) {
            return Err(RiskMethodologyError::Identity(format!(
                "{kind} scale ordinals must be contiguous 1..={n}"
            )));
        }
    }
    Ok(())
}

fn validate_rating_scale(scale: &RatingScale) -> Result<(), RiskMethodologyError> {
    if scale.levels.is_empty() {
        return Err(RiskMethodologyError::Identity(
            "rating scale must have at least one level".into(),
        ));
    }
    let mut ids = BTreeSet::new();
    let mut ordinals = BTreeMap::new();
    for level in &scale.levels {
        if level.label.trim().is_empty() || level.id.trim().is_empty() {
            return Err(RiskMethodologyError::Identity(
                "rating scale label is empty".into(),
            ));
        }
        if !ids.insert(level.id.as_str()) {
            return Err(RiskMethodologyError::Identity(format!(
                "rating scale duplicate id {}",
                level.id
            )));
        }
        if ordinals.insert(level.ordinal, ()).is_some() {
            return Err(RiskMethodologyError::DuplicateOrdinal(level.ordinal));
        }
        if level.ordinal < 1 {
            return Err(RiskMethodologyError::DuplicateOrdinal(level.ordinal));
        }
    }
    let n = scale.levels.len() as u32;
    for expected in 1..=n {
        if !ordinals.contains_key(&expected) {
            return Err(RiskMethodologyError::Identity(format!(
                "rating scale ordinals must be contiguous 1..={n}"
            )));
        }
    }
    Ok(())
}

fn rating_exists(scale: &RatingScale, id: &str) -> bool {
    scale.levels.iter().any(|level| level.id == id)
}

fn rating_ordinal(scale: &RatingScale, id: &str) -> Result<u32, RiskMethodologyError> {
    scale
        .levels
        .iter()
        .find(|level| level.id == id)
        .map(|level| level.ordinal)
        .ok_or_else(|| RiskMethodologyError::UnreachableRating(id.to_string()))
}

fn assert_all_ratings_reachable(
    scale: &RatingScale,
    reached: &BTreeSet<&str>,
) -> Result<(), RiskMethodologyError> {
    for level in &scale.levels {
        if !reached.contains(level.id.as_str()) {
            return Err(RiskMethodologyError::UnreachableRating(level.id.clone()));
        }
    }
    Ok(())
}

impl fmt::Display for ScoringMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Qualitative => f.write_str("qualitative"),
            Self::SemiQuantitative => f.write_str("semiQuantitative"),
            Self::Quantitative => f.write_str("quantitative"),
            Self::CustomBounded => f.write_str("customBounded"),
        }
    }
}
