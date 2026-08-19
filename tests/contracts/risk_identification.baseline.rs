//! Baseline suite for Operational ISMS v1 Prompt 07 — risk identification.
//!
//! Encodes characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`
//! as specified in `docs/specs/risk-identification.md` §3 / §6. Target
//! `sdd_risk_identification_target` is the source of truth. This baseline
//! is skipped (`#[ignore = "superseded by target suite"]`). `p07_b06` is
//! skip-superseded because the register now fail-closes duplicate `RiskId`s.
//!
//! Does **not** implement identification, clustering, or promotion.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance::bridge::{self, EngineHitView, SemanticFindingView};
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssessmentId, AssetId, Control, ControlId, ControlImplementation,
    ControlImplementationId, Risk, RiskId, RiskStatus, ValidateIr,
};
use weeping_angel_collector::{CollectorScope, EvidenceCollector, FixtureCollector};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
    looks_like_compliance_claim,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

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

fn crate_sources_joined(name: &str) -> String {
    let mut files = Vec::new();
    walk_rs_files(&crate_src(name), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn product_crate_sources_joined() -> String {
    let crates_dir = manifest_dir().join("crates");
    let entries = fs::read_dir(&crates_dir).unwrap_or_else(|e| {
        panic!("read {}: {e}", crates_dir.display());
    });
    let mut chunks = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let src = entry.path().join("src");
        if !src.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        walk_rs_files(&src, &mut files);
        for path in files {
            chunks.push(fs::read_to_string(&path).unwrap());
        }
    }
    chunks.join("\n")
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn ir_fixture(name: &str) -> PathBuf {
    manifest_dir()
        .join("tests/fixtures/assurance-ir/v1")
        .join(name)
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.risk-identification.baseline"))
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

struct SemanticHit {
    rule_id: &'static str,
    finding_id: &'static str,
    title: &'static str,
}

impl SemanticFindingView for SemanticHit {
    fn rule_id(&self) -> &str {
        self.rule_id
    }
    fn finding_id(&self) -> &str {
        self.finding_id
    }
    fn title(&self) -> &str {
        self.title
    }
}

fn sample_hit() -> Hit {
    Hit {
        rule_id: "cve-2024-example",
        path: "asset:payments-api",
        category: "vulnerability",
        title: "Known vulnerability on payments API",
    }
}

fn fresh_provenance(asset: &str) -> EvidenceProvenance {
    EvidenceProvenance {
        collector_id: "fixture.risk-identification.baseline".into(),
        collected_at: Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap(),
        scope: "baseline".into(),
        asset: AssetId::new(asset),
    }
}

fn identification_needles() -> &'static [&'static str] {
    &[
        "RiskCandidate",
        "RiskCandidateId",
        "CorrelationKey",
        "identify_risk_candidates",
        "correlate_candidates",
        "PromotionRecord",
        "promote_candidate",
        "DismissalRecord",
        "dismiss_candidate",
        "CandidateStatus",
        "should_resurface",
        "ObservationIdentity",
        "ScoreSuggestion",
        "SuggestedRiskCategory",
        "IdentificationContext",
        "IdentificationPolicy",
    ]
}

/// P07-B01: product crate sources have no RiskCandidate / identification symbols.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b01_product_sources_have_no_risk_candidate_symbols() {
    let src = product_crate_sources_joined();
    for needle in identification_needles() {
        assert!(
            !src.contains(needle),
            "product crate sources must not yet expose `{needle}`"
        );
    }
}

/// P07-B02: IR lib.rs re-exports Risk / RiskStatus only; no candidate module.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b02_ir_lib_reexports_only_the_minimal_risk_record() {
    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        lib.contains("pub use risk::{Risk, RiskStatus};"),
        "IR lib.rs must re-export Risk / RiskStatus"
    );
    assert!(
        lib.contains("pub mod risk;"),
        "IR crate still has a risk module"
    );
    assert!(
        !lib.contains("pub mod risk_candidate") && !lib.contains("pub mod risk_promotion"),
        "IR lib.rs must not declare risk_candidate / risk_promotion on characterization HEAD"
    );
    for needle in [
        "RiskCandidate",
        "RiskCandidateId",
        "CorrelationKey",
        "PromotionRecord",
        "DismissalRecord",
        "CandidateStatus",
    ] {
        assert!(
            !lib.contains(needle),
            "IR lib.rs must not export `{needle}` on characterization HEAD"
        );
    }
}

/// P07-B03: Risk::new is four fields and Open; JSON has no candidate / promotion slots.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b03_risk_is_four_field_inventory_stub() {
    let risk = Risk::new(
        RiskId::new("risk:source-tamper"),
        "Source tampering",
        "Unauthorized change to the source of record.",
    );
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    assert_eq!(risk.title, "Source tampering");
    assert_eq!(
        risk.description,
        "Unauthorized change to the source of record."
    );
    assert_eq!(risk.status, RiskStatus::Open);

    let json = serde_json::to_value(&risk).unwrap();
    let obj = json.as_object().expect("Risk serializes as an object");
    assert_eq!(
        obj.keys().count(),
        4,
        "found-case Risk JSON is exactly four keys, got {obj:?}"
    );
    assert_eq!(json["status"], "open");
    for absent in [
        "correlationKey",
        "candidateId",
        "resultingRiskId",
        "findingRefs",
        "scenario",
        "source",
        "confidence",
        "stale",
        "scoreSuggestion",
        "suggestedRiskCategory",
    ] {
        assert!(
            obj.get(absent).is_none(),
            "found-case Risk JSON must not contain `{absent}`"
        );
    }

    let _ = RiskStatus::Accepted;
    let _ = RiskStatus::Mitigated;
    let _ = RiskStatus::Closed;
}

/// P07-B04: golden risk.json decodes as the four-field stub.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b04_golden_risk_json_decodes_four_keys() {
    let raw = fs::read_to_string(ir_fixture("risk.json")).unwrap();
    let risk: Risk = serde_json::from_str(&raw).unwrap();
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    assert_eq!(risk.title, "Source tampering");
    assert_eq!(risk.status, RiskStatus::Open);

    let value: Value = serde_json::from_str(&raw).unwrap();
    let obj = value.as_object().expect("risk.json is an object");
    assert_eq!(obj.keys().count(), 4);
    assert_eq!(obj["id"], "risk:source-tamper");
    assert_eq!(obj["status"], "open");
}

/// P07-B05: AssessmentDefinition.risks is author-supplied and empty by default.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b05_assessment_risks_are_author_supplied_and_empty_by_default() {
    let assessment = empty_assessment();
    assert!(
        assessment.risks.is_empty(),
        "AssessmentDefinition.risks is empty by default"
    );

    let raw = fs::read_to_string(ir_fixture("assessment.json")).unwrap();
    let value: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        value["risks"],
        serde_json::json!([]),
        "golden assessment.json must keep an empty risks inventory"
    );

    let decoded: AssessmentDefinition = serde_json::from_str(&raw).unwrap();
    assert!(decoded.risks.is_empty());
}

/// P07-B06: IR-019 only checks ControlImplementation.risk_ids; duplicate RiskIds collapse.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b06_ir_019_is_the_only_risk_integrity_check() {
    let mut dangling = empty_assessment();
    dangling.controls.push(Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    ));
    dangling.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(RiskId::new("risk:missing")),
    );
    let err = dangling
        .validate()
        .expect_err("IR-019: dangling risk must fail closed");
    let msg = err.to_string();
    assert!(
        msg.contains("dangling risk"),
        "IR-019 error must mention dangling risk, got {msg}"
    );

    let mut dupes = empty_assessment();
    let id = RiskId::new("risk:same");
    dupes
        .risks
        .push(Risk::new(id.clone(), "first", "first copy"));
    dupes
        .risks
        .push(Risk::new(id.clone(), "second", "second copy"));
    dupes.controls.push(Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    ));
    dupes.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(id),
    );
    dupes
        .validate()
        .expect("duplicate RiskIds silently collapse into the IR-019 id bag");
}

/// P07-B07: N findings do not collapse into a candidate or insert risks.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b07_multiple_findings_do_not_insert_or_cluster_risks() {
    let definition = empty_assessment();
    let first = bridge::from_engine_hit(&sample_hit());
    let second = bridge::from_engine_hit(&Hit {
        rule_id: "cve-2024-other",
        path: "asset:payments-api",
        category: "vulnerability",
        title: "Second known vulnerability on payments API",
    });
    assert_eq!(
        first.evidence_type(),
        &EvidenceType::new("security_finding")
    );
    assert_eq!(
        second.evidence_type(),
        &EvidenceType::new("security_finding")
    );
    assert_eq!(definition.risks.len(), 0);
    definition
        .validate()
        .expect("author-empty risks stay valid with unused observations");
    assert!(
        definition.risks.is_empty(),
        "observations never insert AssessmentDefinition.risks"
    );
}

/// P07-B08: one finding does not contribute two Risks; no identify API exists.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b08_one_finding_does_not_create_two_risks() {
    let definition = empty_assessment();
    let obs = bridge::from_engine_hit(&sample_hit());
    assert_eq!(
        obs.fact("canonical_type"),
        Some("security.vulnerability.present")
    );
    assert!(definition.risks.is_empty());
    let assurance = crate_sources_joined("weeping-angel-assurance");
    assert!(
        !assurance.contains("fn identify_risk_candidates")
            && !assurance.contains("mod risk_identification"),
        "assurance crate has no identification engine on characterization HEAD"
    );
}

/// P07-B09: no dismissal / resurfacing types or APIs.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b09_no_dismissal_or_resurface_api() {
    let src = product_crate_sources_joined();
    for needle in [
        "fn dismiss_candidate",
        "fn should_resurface",
        "struct DismissalRecord",
        "enum CandidateStatus",
        "Resurfaced",
    ] {
        assert!(
            !src.contains(needle),
            "product sources must not yet contain `{needle}`"
        );
    }
}

/// P07-B10: no promote_candidate; RiskCandidate is not Risk.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b10_no_promotion_path_from_candidate_to_risk() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("From<RiskCandidate>") && !ir.contains("fn promote_candidate"),
        "IR must not promote a candidate to Risk"
    );
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        risk_src.contains("//! Minimal risk record. Not a risk engine."),
        "risk.rs must keep the found-case module comment"
    );
    assert!(
        !manifest_dir()
            .join("crates/weeping-angel-assurance-ir/src/risk_candidate.rs")
            .exists(),
        "risk_candidate.rs must be absent on characterization HEAD"
    );
}

/// P07-B11: no stale-candidate / freshness gate on identification.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b11_no_stale_candidate_or_freshness_identification_gate() {
    let src = product_crate_sources_joined();
    for needle in [
        "maxEvidenceAgeSeconds",
        "IdentificationPolicy",
        "CandidateStatus",
        "fn promote_candidate",
    ] {
        assert!(
            !src.contains(needle),
            "product sources must not yet contain identification `{needle}`"
        );
    }
}

/// P07-B12: no-finding / inventory-only definition yields no candidates (no identify API).
#[test]
#[ignore = "superseded by target suite"]
fn p07_b12_no_finding_and_inventory_only_leave_risks_untouched() {
    let definition = empty_assessment();
    assert!(definition.risks.is_empty());
    definition.validate().unwrap();
    let assurance = crate_sources_joined("weeping-angel-assurance");
    assert!(
        !assurance.contains("identify_risk_candidates"),
        "empty evidence has no identify API on characterization HEAD"
    );
}

/// P07-B13: looks_like_compliance_claim is false for risk accepted / ISO control failed.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b13_claim_deny_does_not_cover_risk_or_control_verdicts() {
    assert!(
        !looks_like_compliance_claim("risk accepted"),
        "found case: `risk accepted` is not a claim-deny needle"
    );
    assert!(
        !looks_like_compliance_claim("risk is accepted"),
        "found case: `risk is accepted` is not a claim-deny needle"
    );
    assert!(
        !looks_like_compliance_claim("ISO control failed"),
        "found case: `ISO control failed` is not a claim-deny needle"
    );
    assert!(
        !looks_like_compliance_claim("iso control failed"),
        "found case: `iso control failed` is not a claim-deny needle"
    );
    assert!(
        !looks_like_compliance_claim("iso 27001 control failed"),
        "found case: `iso 27001 control failed` is not a claim-deny needle"
    );
    assert!(
        !looks_like_compliance_claim("iso27001 control failed"),
        "found case: `iso27001 control failed` is not a claim-deny needle"
    );

    assert!(looks_like_compliance_claim("iso 27001 compliant"));
    assert!(looks_like_compliance_claim("gdpr compliant"));
    assert!(looks_like_compliance_claim("soc 2 compliant"));
    assert!(looks_like_compliance_claim("control test result"));
    assert!(looks_like_compliance_claim("audit passed"));
}

/// P07-B14: seal and collector collect-path accept risk-accepted / ISO-control-failed narratives.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b14_seal_and_collector_accept_risk_accepted_and_iso_control_failed() {
    let risk_accepted = EvidenceObservation::new(EvidenceType::new("security_finding"))
        .with_narrative("risk accepted");
    EvidenceEnvelope::seal(risk_accepted, fresh_provenance("asset:payments-api"))
        .expect("found case: seal allows `risk accepted`");

    let iso_failed = EvidenceObservation::new(EvidenceType::new("security_finding"))
        .with_narrative("ISO control failed");
    EvidenceEnvelope::seal(iso_failed, fresh_provenance("asset:payments-api"))
        .expect("found case: seal allows `ISO control failed`");

    let asset = AssetId::new("asset:payments-api");
    let collector = FixtureCollector::new("fixture.risk-identification", "1.0.0")
        .with_evidence_types([EvidenceType::new("security_finding")])
        .with_planned(
            asset.clone(),
            EvidenceObservation::new(EvidenceType::new("security_finding"))
                .with_narrative("risk accepted"),
        );
    let scope = CollectorScope::new().allow_asset(asset);
    collector
        .collect(&scope)
        .expect("found case: collector collect-path allows `risk accepted`");
}

/// P07-B15: bridge is one-way security_finding; never constructs Risk or Effectiveness.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b15_bridge_emits_security_finding_and_does_not_construct_risk() {
    let obs = bridge::from_engine_hit(&sample_hit());
    assert_eq!(obs.evidence_type(), &EvidenceType::new("security_finding"));
    assert_eq!(obs.fact("rule_id"), Some("cve-2024-example"));
    assert_eq!(obs.fact("path"), Some("asset:payments-api"));
    assert_eq!(obs.fact("category"), Some("vulnerability"));
    assert!(obs.fact("iso27001").is_none());
    assert!(obs.fact("risk_status").is_none());

    let from_sem = bridge::from_semantic_finding(&SemanticHit {
        rule_id: "cve-2024-example",
        finding_id: "finding:payments-api",
        title: "Known vulnerability on payments API",
    });
    assert_eq!(
        from_sem.evidence_type(),
        &EvidenceType::new("security_finding")
    );
    assert_eq!(from_sem.fact("finding_id"), Some("finding:payments-api"));

    let bridge_src = read_repo_file("crates/weeping-angel-assurance/src/bridge.rs");
    assert!(
        !bridge_src.contains("Risk") && !bridge_src.contains("Effectiveness"),
        "scanner bridge must not construct Risk or Effectiveness"
    );
}

/// P07-B16: no Incident IR type and no candidate ledger.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b16_no_incident_type_or_candidate_ledger() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("pub struct Incident") && !ir.contains("struct Incident {"),
        "this slice must not invent an Incident IR document"
    );
    assert!(
        !ir.contains("pub struct RiskCandidate") && !ir.contains("struct PromotionRecord"),
        "no candidate ledger on characterization HEAD"
    );
}

/// P07-B17: RiskId exists; RiskCandidateId / PromotionId / DismissalId do not.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b17_typed_ids_have_risk_id_only() {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    assert!(
        src.contains("typed_id!(RiskId);"),
        "RiskId must remain a typed_id!"
    );
    for needle in ["RiskCandidateId", "PromotionId", "DismissalId"] {
        assert!(
            !src.contains(needle),
            "`{needle}` must not exist on characterization HEAD"
        );
    }
    let id = RiskId::new("risk:source-tamper");
    assert_eq!(id.as_str(), "risk:source-tamper");
}

/// P07-B18: collectors do not import identification or rating types.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b18_collectors_have_no_candidate_or_rating_types() {
    let src = crate_sources_joined("weeping-angel-collector");
    for needle in [
        "RiskCandidate",
        "ScoreSuggestion",
        "promote_candidate",
        "RiskRating",
        "DerivedRating",
    ] {
        assert!(
            !src.contains(needle),
            "collector sources must not contain `{needle}`"
        );
    }
}

/// P07-B19: collision fence — GitHub collector mapping sources stay identification-free.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b19_collision_fence_github_collector() {
    let github_src = crate_src("weeping-angel-collector").join("github");
    assert!(
        github_src.is_dir(),
        "GitHub collector remains a collision fence at {}",
        github_src.display()
    );
    let mut files = Vec::new();
    walk_rs_files(&github_src, &mut files);
    let src = files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for needle in [
        "RiskCandidate",
        "promote_candidate",
        "identify_risk_candidates",
        "RiskRating",
    ] {
        assert!(
            !src.contains(needle),
            "GitHub collector mapping must not contain `{needle}`"
        );
    }
}

/// P07-B20: dual-suite names are listed in root Cargo.toml.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b20_dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_risk_identification_baseline")
            && toml.contains("sdd_risk_identification_target")
            && toml.contains("tests/contracts/risk_identification.baseline.rs")
            && toml.contains("tests/contracts/risk_identification.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/risk_identification.baseline.rs")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/risk_identification.target.rs")
            .is_file()
    );
}

/// P07-B21: author-listed risks stay put when observations exist; scanners do not set Accepted.
#[test]
#[ignore = "superseded by target suite"]
fn p07_b21_author_listed_risks_are_unchanged_by_observations() {
    let mut definition = empty_assessment();
    definition.risks.push(Risk::new(
        RiskId::new("risk:source-tamper"),
        "Source tampering",
        "Unauthorized change to the source of record.",
    ));
    let before = serde_json::to_value(&definition.risks).unwrap();
    let _obs = bridge::from_engine_hit(&sample_hit());
    let after = serde_json::to_value(&definition.risks).unwrap();
    assert_eq!(before, after);
    assert_eq!(definition.risks[0].status, RiskStatus::Open);
    assert_ne!(definition.risks[0].status, RiskStatus::Accepted);
}
