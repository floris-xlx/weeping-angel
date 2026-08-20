//! Target suite for Operational ISMS v1 Prompt 07 — risk identification.
//!
//! Encodes desired behavior in `docs/specs/risk-identification.md` §4 / §6
//! (RI-001–RI-010). GREEN on this HEAD. Do not `#[ignore]` these tests.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance::bridge::{self, EngineHitView};
use weeping_angel_assurance::risk_identification::{
    IdentificationContext, IdentificationError, IdentificationPolicy, dismiss_candidate,
    identify_risk_candidates, observation_identity_digest, promote_candidate, should_resurface,
};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind,
    CandidateConfidence, CandidateStatus, DismissalRecord, Identity, IdentityId, IdentityKind,
    PrincipalRef, ProcessingActivity, ProcessingActivityId, PromotionRecord, Risk, RiskCandidate,
    RiskCandidateId, RiskId, RiskStatus, ScoreSuggestion, ValidateIr,
};
use weeping_angel_collector::{CollectorScope, EvidenceCollector, FixtureCollector};
use weeping_angel_control_test::Effectiveness;
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
    looks_like_compliance_claim,
};

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            walk_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn crate_src(name: &str) -> PathBuf {
    let path = manifest_dir().join("crates").join(name).join("src");
    assert!(
        path.is_dir(),
        "expected crate sources at {}",
        path.display()
    );
    path
}

fn ir_fixture(name: &str) -> PathBuf {
    manifest_dir()
        .join("tests/fixtures/assurance-ir/v1")
        .join(name)
}

fn as_of() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap()
}

struct Hit {
    rule_id: &'static str,
    path: &'static str,
    category: &'static str,
    title: &'static str,
}

impl EngineHitView for Hit {
    fn rule_id(&self) -> &str {
        self.rule_id
    }
    fn path(&self) -> &str {
        self.path
    }
    fn category(&self) -> &str {
        self.category
    }
    fn title(&self) -> &str {
        self.title
    }
}

fn vuln_hit(rule_id: &'static str, path: &'static str, title: &'static str) -> Hit {
    Hit {
        rule_id,
        path,
        category: "vulnerability",
        title,
    }
}

fn provenance_at(asset: &str, collected_at: DateTime<Utc>) -> EvidenceProvenance {
    EvidenceProvenance {
        collector_id: "fixture.risk-identification.target".into(),
        collected_at,
        scope: "target".into(),
        asset: AssetId::new(asset),
    }
}

fn fresh_provenance(asset: &str) -> EvidenceProvenance {
    provenance_at(asset, as_of())
}

fn seal_obs(obs: EvidenceObservation, asset: &str) -> EvidenceEnvelope {
    EvidenceEnvelope::seal(obs, fresh_provenance(asset)).expect("target fixture must seal")
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.risk-identification.target"))
}

fn reviewer() -> Identity {
    Identity::new(IdentityId::new("identity:risk-owner"), IdentityKind::User)
}

fn payments_service() -> Asset {
    Asset::new(
        AssetId::new("asset:payments-api"),
        AssetKind::Service,
        "payments-api",
    )
}

fn repo_asset() -> Asset {
    Asset::new(
        AssetId::new("asset:repo:source"),
        AssetKind::Repository,
        "source-of-record",
    )
}

fn card_payments_ropa() -> ProcessingActivity {
    let mut activity = ProcessingActivity::new(
        ProcessingActivityId::new("ropa:card-payments"),
        "Card payments",
    );
    activity.systems.push(AssetId::new("asset:payments-api"));
    activity
}

fn inventory_without_dual_scenario() -> AssessmentDefinition {
    let mut definition = empty_assessment();
    definition.assets.push(repo_asset());
    definition.identities.push(reviewer());
    definition
}

fn inventory_dual_scenario() -> AssessmentDefinition {
    let mut definition = empty_assessment();
    definition.assets.push(payments_service());
    definition.identities.push(reviewer());
    definition.processing_activities.push(card_payments_ropa());
    definition
}

fn owner_principal() -> PrincipalRef {
    PrincipalRef::Identity(IdentityId::new("identity:risk-owner"))
}

fn default_policy() -> IdentificationPolicy {
    IdentificationPolicy {
        max_evidence_age_seconds: None,
    }
}

fn stale_policy() -> IdentificationPolicy {
    IdentificationPolicy {
        max_evidence_age_seconds: Some(3_600),
    }
}

fn ctx<'a>(
    definition: &'a AssessmentDefinition,
    observations: &'a [EvidenceObservation],
    envelopes: &'a [EvidenceEnvelope],
    prior: &'a [RiskCandidate],
    dismissals: &'a [DismissalRecord],
    promotions: &'a [PromotionRecord],
    policy: IdentificationPolicy,
) -> IdentificationContext<'a> {
    IdentificationContext {
        definition,
        observations,
        envelopes,
        prior_candidates: prior,
        dismissals,
        promotions,
        policy,
        as_of: as_of(),
    }
}

fn identify_now(
    definition: &AssessmentDefinition,
    observations: &[EvidenceObservation],
    envelopes: &[EvidenceEnvelope],
) -> Vec<RiskCandidate> {
    identify_risk_candidates(&ctx(
        definition,
        observations,
        envelopes,
        &[],
        &[],
        &[],
        default_policy(),
    ))
}

fn proposed(candidates: &[RiskCandidate]) -> Vec<&RiskCandidate> {
    candidates
        .iter()
        .filter(|c| {
            c.status == CandidateStatus::Proposed || c.status == CandidateStatus::Resurfaced
        })
        .collect()
}

fn err_text(err: &IdentificationError) -> String {
    err.to_string()
}

fn promote_ok(
    definition: &mut AssessmentDefinition,
    candidate: &RiskCandidate,
    rationale: &str,
) -> (RiskCandidate, Risk, PromotionRecord) {
    promote_candidate(
        definition,
        candidate,
        owner_principal(),
        as_of(),
        rationale,
        None,
    )
    .unwrap_or_else(|e| panic!("promote must succeed: {e}"))
}

/// RI-001 — N security_finding observations with the same subject + scenarioKey
/// collapse to one Proposed survivor plus ClusteredDuplicate ids.
#[test]
fn p07_n_findings_collapse_to_one_candidate() {
    let definition = inventory_without_dual_scenario();
    let first = bridge::from_engine_hit(&vuln_hit(
        "cve-2024-example",
        "asset:repo:source",
        "Known vulnerability on the source of record",
    ));
    let second = bridge::from_engine_hit(&vuln_hit(
        "cve-2024-other",
        "asset:repo:source",
        "Second known vulnerability on the source of record",
    ));
    assert_eq!(
        first.fact("canonical_type"),
        Some("security.vulnerability.present")
    );
    assert_eq!(
        second.fact("canonical_type"),
        Some("security.vulnerability.present")
    );

    let envelopes = [
        seal_obs(first.clone(), "asset:repo:source"),
        seal_obs(second.clone(), "asset:repo:source"),
    ];
    let observations = [first, second];
    let risks_before = definition.risks.len();

    let candidates = identify_now(&definition, &observations, &envelopes);
    assert_eq!(
        definition.risks.len(),
        risks_before,
        "identify_risk_candidates must not insert AssessmentDefinition.risks"
    );

    let survivors: Vec<_> = candidates
        .iter()
        .filter(|c| c.status == CandidateStatus::Proposed)
        .collect();
    assert_eq!(
        survivors.len(),
        1,
        "same subject + scenarioKey must collapse to one Proposed survivor, got {candidates:?}"
    );
    let survivor = survivors[0];
    assert!(!survivor.correlation_key.as_str().is_empty());
    assert!(survivor.correlation_key.as_str().starts_with("ck:sha256:"));
    assert_eq!(survivor.duplicate_candidate_ids.len(), 1);
    assert_eq!(survivor.supporting_observations.len(), 2);
    assert_eq!(survivor.confidence, CandidateConfidence::High);
    assert!(survivor.resulting_risk_id.is_none());
    assert_ne!(survivor.status, CandidateStatus::Promoted);

    let duplicates: Vec<_> = candidates
        .iter()
        .filter(|c| c.status == CandidateStatus::ClusteredDuplicate)
        .collect();
    assert_eq!(duplicates.len(), 1);
    assert_eq!(survivor.duplicate_candidate_ids[0], duplicates[0].id);
    assert_eq!(
        survivor.correlation_key.as_str(),
        duplicates[0].correlation_key.as_str()
    );
}

/// RI-002 — one finding with two deterministic scenario keys yields two candidates.
#[test]
fn p07_one_finding_contributes_to_two_distinct_candidates() {
    let mut definition = inventory_dual_scenario();
    let obs = bridge::from_engine_hit(&vuln_hit(
        "cve-2024-example",
        "asset:payments-api",
        "Known vulnerability on payments API",
    ));
    assert_eq!(
        obs.fact("canonical_type"),
        Some("security.vulnerability.present")
    );
    let envelopes = [seal_obs(obs.clone(), "asset:payments-api")];
    let observations = [obs];

    let candidates = identify_now(&definition, &observations, &envelopes);
    let survivors = proposed(&candidates);
    assert_eq!(
        survivors.len(),
        2,
        "vuln on production service that is also a processing-activity system must emit two candidates, got {candidates:?}"
    );

    let keys: BTreeSet<_> = survivors
        .iter()
        .map(|c| c.correlation_key.as_str().to_string())
        .collect();
    assert_eq!(
        keys.len(),
        2,
        "the two candidates must not share a correlation key"
    );

    let scenario_keys: BTreeSet<_> = survivors
        .iter()
        .map(|c| c.scenario_proposal.scenario_key.clone())
        .collect();
    assert!(
        scenario_keys.contains("confidentiality exposure via known vulnerability"),
        "missing confidentiality scenario, got {scenario_keys:?}"
    );
    assert!(
        scenario_keys.contains("integrity or availability failure via known vulnerability"),
        "missing integrity/availability scenario, got {scenario_keys:?}"
    );

    let first_identities: BTreeSet<_> = survivors[0]
        .supporting_observations
        .iter()
        .map(observation_identity_digest)
        .collect();
    let second_identities: BTreeSet<_> = survivors[1]
        .supporting_observations
        .iter()
        .map(observation_identity_digest)
        .collect();
    assert_eq!(
        first_identities, second_identities,
        "both candidates must share the same ObservationIdentity"
    );
    assert_eq!(first_identities.len(), 1);

    let (left, left_risk, _) = promote_ok(
        &mut definition,
        survivors[0],
        "Promote confidentiality exposure after review.",
    );
    let (right, right_risk, _) = promote_ok(
        &mut definition,
        survivors[1],
        "Promote integrity or availability failure after review.",
    );
    assert_ne!(left_risk.id.as_str(), right_risk.id.as_str());
    assert!(left_risk.id.as_str().starts_with("risk:"));
    assert!(right_risk.id.as_str().starts_with("risk:"));
    assert_ne!(left.id.as_str(), left_risk.id.as_str());
    assert_ne!(right.id.as_str(), right_risk.id.as_str());
    assert_eq!(definition.risks.len(), 2);
    let risk_ids: BTreeSet<_> = definition
        .risks
        .iter()
        .map(|r| r.id.as_str().to_string())
        .collect();
    assert_eq!(risk_ids.len(), 2);
}

/// RI-003 — promote_candidate is explicit; identify never inserts.
#[test]
fn p07_candidate_promotion_is_explicit() {
    let mut definition = inventory_without_dual_scenario();
    let obs = bridge::from_engine_hit(&vuln_hit(
        "cve-2024-example",
        "asset:repo:source",
        "Known vulnerability on the source of record",
    ));
    let envelopes = [seal_obs(obs.clone(), "asset:repo:source")];
    let observations = [obs];
    let candidates = identify_now(&definition, &observations, &envelopes);
    assert!(
        definition.risks.is_empty(),
        "identify_risk_candidates never inserts into definition.risks"
    );
    let survivor = proposed(&candidates)
        .into_iter()
        .next()
        .expect("one proposed candidate");
    let candidate_id = survivor.id.clone();
    let correlation_key = survivor.correlation_key.as_str().to_string();

    promote_candidate(
        &mut definition,
        survivor,
        owner_principal(),
        as_of(),
        "",
        None,
    )
    .expect_err("empty rationale must fail closed");

    let dangling = promote_candidate(
        &mut definition,
        survivor,
        PrincipalRef::Identity(IdentityId::new("identity:unknown-reviewer")),
        as_of(),
        "Looks like a real risk after review.",
        None,
    )
    .expect_err("unknown identity principal must fail closed");
    assert!(
        err_text(&dangling).contains("dangling"),
        "dangling principal error, got {}",
        err_text(&dangling)
    );

    let (promoted, risk, record) = promote_ok(
        &mut definition,
        survivor,
        "Looks like a real risk after review.",
    );
    assert_eq!(promoted.id, candidate_id);
    assert_eq!(promoted.correlation_key.as_str(), correlation_key);
    assert_eq!(promoted.status, CandidateStatus::Promoted);
    assert_eq!(
        promoted.resulting_risk_id.as_ref().map(|id| id.as_str()),
        Some(risk.id.as_str())
    );
    assert_eq!(record.candidate_id, candidate_id);
    assert_eq!(record.correlation_key.as_str(), correlation_key);
    assert_eq!(record.risk_id.as_str(), risk.id.as_str());
    assert_eq!(record.principal, owner_principal());
    assert_eq!(record.at, as_of());
    assert!(!record.rationale.is_empty());
    assert_eq!(risk.title, survivor.scenario_proposal.title);
    assert_eq!(risk.status, RiskStatus::Open);
    assert_eq!(definition.risks.len(), 1);
    assert_eq!(definition.risks[0].id.as_str(), risk.id.as_str());
    definition
        .validate()
        .expect("promoted RiskId must remain IR-valid");

    let again = promote_candidate(
        &mut definition,
        &promoted,
        owner_principal(),
        as_of(),
        "Trying to promote twice.",
        None,
    )
    .expect_err("already-promoted candidate must fail closed");
    assert!(
        err_text(&again).contains("not promotable"),
        "re-promote class, got {}",
        err_text(&again)
    );
    assert_eq!(definition.risks.len(), 1);
}

/// RI-004 — dismissed candidates stay dismissed; new identity resurfaces; clock-only does not.
#[test]
fn p07_dismissed_candidates_do_not_auto_promote_and_follow_resurfacing_rules() {
    let mut definition = inventory_without_dual_scenario();
    let first = bridge::from_engine_hit(&vuln_hit(
        "cve-2024-example",
        "asset:repo:source",
        "Known vulnerability on the source of record",
    ));
    let envelopes = [seal_obs(first.clone(), "asset:repo:source")];
    let observations = [first.clone()];
    let candidates = identify_now(&definition, &observations, &envelopes);
    let survivor = proposed(&candidates)
        .into_iter()
        .next()
        .expect("proposed candidate")
        .clone();

    let (dismissed, dismissal) = dismiss_candidate(
        &survivor,
        owner_principal(),
        as_of(),
        "Not an organizational risk at this time.",
    )
    .unwrap_or_else(|e| panic!("dismiss must succeed: {e}"));
    assert_eq!(dismissed.status, CandidateStatus::Dismissed);
    assert!(dismissed.resulting_risk_id.is_none());
    assert_eq!(definition.risks.len(), 0);
    assert_eq!(dismissal.candidate_id, survivor.id);
    assert!(!dismissal.observation_identities.is_empty());

    let same_again = identify_risk_candidates(&ctx(
        &definition,
        &observations,
        &envelopes,
        std::slice::from_ref(&dismissed),
        std::slice::from_ref(&dismissal),
        &[],
        default_policy(),
    ));
    assert!(
        same_again
            .iter()
            .all(|c| c.status == CandidateStatus::Dismissed),
        "identical ObservationIdentity sets stay Dismissed, got {same_again:?}"
    );
    assert!(
        same_again
            .iter()
            .all(|c| c.status != CandidateStatus::Promoted),
        "dismissal must never auto-promote"
    );
    assert!(definition.risks.is_empty());
    assert!(!should_resurface(&same_again[0], &dismissal));

    let clock_only = EvidenceEnvelope::seal(
        first.clone(),
        provenance_at("asset:repo:source", as_of() + Duration::hours(2)),
    )
    .expect("re-collection of the same facts must still seal");
    let clock_refresh = identify_risk_candidates(&ctx(
        &definition,
        &observations,
        std::slice::from_ref(&clock_only),
        std::slice::from_ref(&dismissed),
        std::slice::from_ref(&dismissal),
        &[],
        default_policy(),
    ));
    assert!(
        clock_refresh
            .iter()
            .all(|c| c.status == CandidateStatus::Dismissed),
        "collected_at-only refresh must not resurface, got {clock_refresh:?}"
    );
    assert!(!should_resurface(&clock_refresh[0], &dismissal));

    let extra = bridge::from_engine_hit(&vuln_hit(
        "cve-2025-new",
        "asset:repo:source",
        "Newly observed vulnerability on the source of record",
    ));
    let extra_env = seal_obs(extra.clone(), "asset:repo:source");
    let with_new = [first, extra];
    let with_new_env = [envelopes[0].clone(), extra_env];
    let resurfaced = identify_risk_candidates(&ctx(
        &definition,
        &with_new,
        &with_new_env,
        std::slice::from_ref(&dismissed),
        std::slice::from_ref(&dismissal),
        &[],
        default_policy(),
    ));
    let live = proposed(&resurfaced);
    assert_eq!(
        live.len(),
        1,
        "new ObservationIdentity on the same key Resurfaces"
    );
    assert_eq!(live[0].status, CandidateStatus::Resurfaced);
    assert_eq!(live[0].id, dismissed.id);
    assert!(should_resurface(live[0], &dismissal));
    assert!(definition.risks.is_empty());

    let (promoted, risk, _) = promote_ok(
        &mut definition,
        live[0],
        "New evidence justifies promotion after prior dismissal.",
    );
    assert_eq!(promoted.status, CandidateStatus::Promoted);
    assert_eq!(definition.risks.len(), 1);
    assert_eq!(definition.risks[0].id.as_str(), risk.id.as_str());
}

/// RI-005 — stale supporting evidence may be listed; promote fails closed.
#[test]
fn p07_stale_evidence_cannot_promote() {
    let mut definition = inventory_without_dual_scenario();
    let obs = bridge::from_engine_hit(&vuln_hit(
        "cve-2024-example",
        "asset:repo:source",
        "Known vulnerability on the source of record",
    ));
    let stale_at = as_of() - Duration::hours(5);
    let stale_env =
        EvidenceEnvelope::seal(obs.clone(), provenance_at("asset:repo:source", stale_at))
            .expect("stale envelope still seals");
    let observations = [obs.clone()];
    let stale_envs = [stale_env];

    let listed = identify_risk_candidates(&ctx(
        &definition,
        &observations,
        &stale_envs,
        &[],
        &[],
        &[],
        stale_policy(),
    ));
    assert_eq!(listed.len(), 1, "stale evidence is still listed");
    assert!(
        listed[0].stale || listed[0].status == CandidateStatus::Stale,
        "identify must mark the candidate stale, got {:?}",
        listed[0].status
    );

    let err = promote_candidate(
        &mut definition,
        &listed[0],
        owner_principal(),
        as_of(),
        "Trying to promote stale evidence.",
        None,
    )
    .expect_err("stale evidence cannot promote");
    assert!(
        err_text(&err).contains("stale evidence"),
        "stale promote needle, got {}",
        err_text(&err)
    );
    assert!(definition.risks.is_empty());

    let fresh_env = seal_obs(obs, "asset:repo:source");
    let fresh = identify_risk_candidates(&ctx(
        &definition,
        &observations,
        std::slice::from_ref(&fresh_env),
        &[],
        &[],
        &[],
        stale_policy(),
    ));
    assert_eq!(fresh.len(), 1);
    assert!(
        !fresh[0].stale && fresh[0].status != CandidateStatus::Stale,
        "same ObservationIdentity with a fresh envelope must clear staleness"
    );
    let (_, _, _) = promote_ok(
        &mut definition,
        &fresh[0],
        "Fresh envelope of the same identity may promote.",
    );
    assert_eq!(definition.risks.len(), 1);
}

/// RI-006 — empty observations / inventory-only yields zero candidates.
#[test]
fn p07_no_finding_yields_no_candidate() {
    let definition = inventory_dual_scenario();
    let candidates = identify_now(&definition, &[], &[]);
    assert!(
        candidates.is_empty(),
        "inventory-only definition must not invent candidates, got {candidates:?}"
    );
    assert!(definition.risks.is_empty());
}

/// RI-007 — claim-deny rejects risk-accepted / ISO-control-failed in seal, collector, identify.
#[test]
fn p07_scanners_cannot_declare_risk_accepted_or_iso_control_failed() {
    for phrase in [
        "risk accepted",
        "risk is accepted",
        "ISO control failed",
        "iso control failed",
        "iso 27001 control failed",
        "iso27001 control failed",
    ] {
        assert!(
            looks_like_compliance_claim(phrase),
            "claim-deny must reject `{phrase}`"
        );
    }
    assert!(looks_like_compliance_claim("iso 27001 compliant"));
    assert!(
        !looks_like_compliance_claim("user accepted the TOS"),
        "bare 'accepted' is not a needle"
    );

    let risk_accepted = EvidenceObservation::new(EvidenceType::new("security_finding"))
        .with_fact("rule_id", "cve-2024-example")
        .with_fact("path", "asset:repo:source")
        .with_narrative("risk accepted");
    EvidenceEnvelope::seal(risk_accepted.clone(), fresh_provenance("asset:repo:source"))
        .expect_err("seal must reject `risk accepted`");

    let iso_failed = EvidenceObservation::new(EvidenceType::new("security_finding"))
        .with_fact("rule_id", "cve-2024-example")
        .with_fact("path", "asset:repo:source")
        .with_narrative("ISO control failed");
    EvidenceEnvelope::seal(iso_failed.clone(), fresh_provenance("asset:repo:source"))
        .expect_err("seal must reject `ISO control failed`");

    let asset = AssetId::new("asset:repo:source");
    let collector = FixtureCollector::new("fixture.risk-identification.target", "1.0.0")
        .with_evidence_types([EvidenceType::new("security_finding")])
        .with_planned(asset.clone(), risk_accepted.clone());
    let scope = CollectorScope::new().allow_asset(asset);
    collector
        .collect(&scope)
        .expect_err("collector collect-path must reject `risk accepted`");

    let definition = inventory_without_dual_scenario();
    let dropped = identify_now(&definition, &[risk_accepted, iso_failed], &[]);
    assert!(
        dropped.is_empty(),
        "identify must drop claim-deny narratives, got {dropped:?}"
    );

    let clean = bridge::from_engine_hit(&vuln_hit(
        "cve-2024-example",
        "asset:repo:source",
        "Known vulnerability on the source of record",
    ));
    let envelopes = [seal_obs(clean.clone(), "asset:repo:source")];
    let candidates = identify_now(&definition, &[clean], &envelopes);
    let mut promote_def = definition.clone();
    if let Some(candidate) = proposed(&candidates).into_iter().next() {
        let (_, risk, _) = promote_ok(
            &mut promote_def,
            candidate,
            "Reviewer promotes without accepting via scanner.",
        );
        assert_ne!(risk.status, RiskStatus::Accepted);
    }

    let _ = Effectiveness::Ineffective;
    let ident_dir = crate_src("weeping-angel-assurance").join("risk_identification");
    let mut ident_files = Vec::new();
    walk_rs_files(&ident_dir, &mut ident_files);
    let ident = ident_files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let ir_candidate = read_repo_file("crates/weeping-angel-assurance-ir/src/risk_candidate.rs")
        + &read_repo_file("crates/weeping-angel-assurance-ir/src/risk_promotion.rs");
    let bridge_src = read_repo_file("crates/weeping-angel-assurance/src/bridge.rs");
    assert!(
        !bridge_src.contains("Effectiveness") && !bridge_src.contains("RiskStatus"),
        "scanner bridge must not author Effectiveness or RiskStatus"
    );
    assert!(
        !ident.contains("Effectiveness::Ineffective")
            && !ident.contains("RiskStatus::Accepted")
            && !ir_candidate.contains("RiskStatus::Accepted = Candidate"),
        "identification must not author Effectiveness or RiskStatus::Accepted"
    );
}

/// RI-008 — RiskCandidate is not Risk; no auto From; candidate JSON does not decode as Risk.
#[test]
fn p07_risk_candidate_is_not_risk() {
    let mut definition = inventory_without_dual_scenario();
    let obs = bridge::from_engine_hit(&vuln_hit(
        "cve-2024-example",
        "asset:repo:source",
        "Known vulnerability on the source of record",
    ));
    let envelopes = [seal_obs(obs.clone(), "asset:repo:source")];
    let candidates = identify_now(&definition, &[obs], &envelopes);
    let survivor = proposed(&candidates)
        .into_iter()
        .next()
        .expect("proposed candidate");

    assert_eq!(survivor.schema_version, ASSURANCE_IR_SCHEMA);
    let json = serde_json::to_value(survivor).expect("candidate serializes");
    let decoded: Result<Risk, _> = serde_json::from_value(json.clone());
    assert!(
        decoded.is_err(),
        "a RiskCandidate document must not decode as Risk, got {decoded:?}"
    );
    assert!(json.get("correlationKey").is_some());
    assert!(json.get("scenarioProposal").is_some());

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("impl From<RiskCandidate> for Risk")
            && !ir.contains("type Risk = RiskCandidate"),
        "no auto From or type alias may collapse candidate into Risk"
    );

    let (promoted, risk, _) = promote_ok(
        &mut definition,
        survivor,
        "Promotion is the only insert path.",
    );
    assert_eq!(definition.risks.len(), 1);
    assert_eq!(definition.risks[0].id.as_str(), risk.id.as_str());
    assert_eq!(promoted.id.as_str(), survivor.id.as_str());
    let _distinct: fn(&RiskCandidate) -> RiskCandidateId = |c| c.id.clone();
    let _risk_id: fn(&Risk) -> RiskId = |r| r.id.clone();
    assert_ne!(promoted.id.as_str(), risk.id.as_str());
}

/// RI-009 — score suggestion is optional; no second matrix / RiskRating::High.
#[test]
fn p07_score_suggestion_is_optional_and_validated() {
    let mut definition = inventory_without_dual_scenario();
    let obs = bridge::from_engine_hit(&vuln_hit(
        "cve-2024-example",
        "asset:repo:source",
        "Known vulnerability on the source of record",
    ));
    let envelopes = [seal_obs(obs.clone(), "asset:repo:source")];
    let candidates = identify_now(&definition, &[obs], &envelopes);
    let survivor = proposed(&candidates)
        .into_iter()
        .next()
        .expect("proposed candidate");
    assert!(
        survivor.score_suggestion.is_none(),
        "identification may omit ScoreSuggestion entirely"
    );

    let (_, _, _record) = promote_candidate(
        &mut definition,
        survivor,
        owner_principal(),
        as_of(),
        "Promote without a score suggestion.",
        None::<ScoreSuggestion>,
    )
    .expect("omit is valid");

    let ident = crate_sources_joined("weeping-angel-assurance");
    let risk_id_dir = crate_src("weeping-angel-assurance").join("risk_identification");
    let ident_engine = if risk_id_dir.is_dir() {
        let mut files = Vec::new();
        walk_rs_files(&risk_id_dir, &mut files);
        files
            .iter()
            .map(|p| fs::read_to_string(p).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        ident.clone()
    };
    assert!(
        !ident_engine.contains("RiskRating::High")
            && !ident_engine.contains("enum RiskRating")
            && !ident_engine.contains("5 × 5")
            && !ident_engine.contains("5x5"),
        "identification must not hardcode a second scoring matrix"
    );

    let collector = crate_sources_joined("weeping-angel-collector");
    for needle in [
        "RiskCandidate",
        "ScoreSuggestion",
        "promote_candidate",
        "RiskRating",
        "DerivedRating",
    ] {
        assert!(
            !collector.contains(needle),
            "collectors must not construct `{needle}`"
        );
    }
}

/// RI-010 — dual-suite registration, golden risk.json, neighbor surfaces stay intact.
#[test]
fn p07_dual_suite_registered_and_risk_json_still_decodes() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        !toml.contains("sdd_risk_identification_baseline")
            && harness_src().contains("risk_identification.target.rs")
            && !toml.contains("tests/contracts/risk_identification.baseline.rs")
            && harness_src().contains("risk_identification.target.rs"),
        "dual-suite must be wired as a harness module"
    );

    let raw = fs::read_to_string(ir_fixture("risk.json")).unwrap();
    let risk: Risk = serde_json::from_str(&raw).unwrap();
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    assert_eq!(risk.title, "Source tampering");
    assert_eq!(risk.status, RiskStatus::Open);
    let value: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(value["id"], "risk:source-tamper");
    assert_eq!(value["status"], "open");

    let spec = read_repo_file("docs/specs/risk-identification.md");
    for id in [
        "RI-001", "RI-002", "RI-003", "RI-004", "RI-005", "RI-006", "RI-007", "RI-008", "RI-009",
        "RI-010",
    ] {
        assert!(spec.contains(id), "spec must list {id}");
    }
}
