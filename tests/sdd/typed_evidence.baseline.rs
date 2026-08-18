//! Baseline suite for typed evidence + canonical serialization (Prompt 02).
//!
//! Encodes CURRENT string-bag / `parse_fact` behavior on characterization SHA
//! `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` as specified in
//! `docs/sdd/typed-evidence.md` §3 / §6.1. Must stay GREEN until the target
//! suite is GREEN and this file is superseded. Does not implement typed storage.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use serde::Serialize;
use serde_json::Value;
use weeping_angel_assurance_ir::{AssetId, ControlId, ControlTestId, canonical_digest};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, EvidenceValue, TestExpr, ValueExpr, evaluate,
};
use weeping_angel_evidence::{
    EVIDENCE_SCHEMA, EvidenceEnvelope, EvidenceError, EvidenceLedger, EvidenceObservation,
    EvidenceProvenance, EvidenceType,
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

fn fresh_provenance() -> EvidenceProvenance {
    EvidenceProvenance {
        collector_id: "fixture.typed-evidence-baseline".into(),
        collected_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        scope: "repo:in-scope".into(),
        asset: AssetId::new("repo:in-scope"),
    }
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 30, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn sealed_with_facts(pairs: &[(&str, &str)]) -> EvidenceEnvelope {
    let mut obs = EvidenceObservation::new(EvidenceType::new("source.branch.protection"));
    for (k, v) in pairs {
        obs = obs.with_fact(*k, *v);
    }
    EvidenceEnvelope::seal(obs, fresh_provenance()).unwrap()
}

fn compiled(expr: TestExpr) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new("test.typed-evidence.baseline"))
        .control_id(ControlId::new("canonical.source-control"))
        .kind(ControlTestKind::Automated)
        .expr(expr)
        .build()
}

fn field_selector(field: &str) -> EvidenceSelector {
    EvidenceSelector {
        evidence_type: EvidenceType::new("source.branch.protection"),
        subject_selector: Default::default(),
        field: Some(field.into()),
        freshness: None,
    }
}

fn evaluate_field(
    env: EvidenceEnvelope,
    expr: TestExpr,
) -> weeping_angel_control_test::ControlTestResult {
    let mut set = EvidenceSet::new();
    set.insert(env);
    evaluate(&compiled(expr), &set, &fresh_context())
}

/// Mirrors private `DigestBody` in `weeping-angel-evidence` (observation + provenance only).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DigestBody<'a> {
    observation: &'a EvidenceObservation,
    provenance: &'a EvidenceProvenance,
}

#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn dual_suite_baseline_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_typed_evidence_baseline")
            && toml.contains("tests/sdd/typed_evidence.baseline.rs"),
        "baseline suite must be listed in root Cargo.toml"
    );
}

#[cfg(any())]
#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn observation_facts_are_string_bags() {
    let obs = EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
        .with_fact("enabled", "true")
        .with_fact("required_reviewers", "2")
        .with_fact("privileged_roles", "owner,admin");

    assert_eq!(obs.fact("enabled"), Some("true"));
    assert_eq!(obs.fact("required_reviewers"), Some("2"));
    assert_eq!(obs.fact("privileged_roles"), Some("owner,admin"));
    assert_eq!(obs.fact("missing"), None);

    let facts: &BTreeMap<String, String> = obs.facts();
    assert_eq!(facts.get("enabled").map(String::as_str), Some("true"));
    assert_eq!(facts.len(), 3);

    let src = crate_sources_joined("weeping-angel-evidence");
    assert!(
        src.contains("facts: BTreeMap<String, String>"),
        "current model stores facts as BTreeMap<String, String>"
    );
    assert!(
        src.contains(
            "pub fn with_fact(mut self, key: impl Into<String>, value: impl Into<String>)"
        ),
        "with_fact is the string insert API"
    );
    assert!(
        src.contains("pub fn fact(&self, key: &str) -> Option<&str>"),
        "fact returns the raw string"
    );
}

#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn evidence_crate_has_no_stored_typed_value_model() {
    let src = crate_sources_joined("weeping-angel-evidence");
    for needle in [
        "enum EvidenceValue",
        "fn with_value",
        "fn fact_value",
        "$evidenceValue",
        "DurationSeconds",
        "StringList",
        "evidence-value/v1",
    ] {
        assert!(
            !src.contains(needle),
            "evidence crate must not yet own typed value surface `{needle}`"
        );
    }
    assert!(
        !manifest_dir()
            .join("catalog")
            .join("canonical")
            .join("v1")
            .is_dir(),
        "Prompt 01 catalog tree is absent on this SHA; baseline must not require it"
    );
}

#[cfg(any())]
#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn evidence_value_and_parse_fact_live_in_control_test() {
    let src = crate_sources_joined("weeping-angel-control-test");
    assert!(
        src.contains("pub enum EvidenceValue"),
        "EvidenceValue currently lives in weeping-angel-control-test"
    );
    assert!(
        src.contains("pub fn parse_fact(raw: &str)"),
        "parse_fact is the current coerce path"
    );
    assert!(
        src.contains("trimmed.parse::<f64>()"),
        "current parse_fact probes f64 for decimal-looking strings"
    );

    assert_eq!(
        EvidenceValue::parse_fact("true"),
        EvidenceValue::Boolean(true)
    );
    assert_eq!(
        EvidenceValue::parse_fact("FALSE"),
        EvidenceValue::Boolean(false)
    );
    assert_eq!(
        EvidenceValue::parse_fact(" true "),
        EvidenceValue::Boolean(true)
    );
    assert_eq!(EvidenceValue::parse_fact("01"), EvidenceValue::Integer(1));
    assert_eq!(
        EvidenceValue::parse_fact("1.0"),
        EvidenceValue::Decimal("1.0".into())
    );
    assert_eq!(
        EvidenceValue::parse_fact("  not-a-scalar  "),
        EvidenceValue::String("  not-a-scalar  ".into())
    );
}

#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn seal_digest_is_canonical_digest_of_observation_and_provenance() {
    let obs = EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
        .with_fact("enabled", "true")
        .with_narrative("branch protection observed");
    let provenance = fresh_provenance();
    let expected = canonical_digest(&DigestBody {
        observation: &obs,
        provenance: &provenance,
    })
    .unwrap();

    let env = EvidenceEnvelope::seal(obs, provenance).unwrap();
    assert_eq!(env.digest(), expected);
    assert_eq!(env.content_digest(), expected);
    assert_eq!(env.evidence_id(), format!("ev:sha256:{expected}"));
    assert_eq!(
        env.observation().evidence_type().as_str(),
        "source.branch.protection"
    );
    let json = serde_json::to_value(&env).unwrap();
    assert_eq!(json["schemaVersion"], EVIDENCE_SCHEMA);
}

#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn string_fact_insertion_order_does_not_change_digest() {
    let first = sealed_with_facts(&[("zulu", "1"), ("alpha", "2"), ("mike", "3")]);
    let second = sealed_with_facts(&[("alpha", "2"), ("mike", "3"), ("zulu", "1")]);
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.content_digest(), second.content_digest());

    let keys: Vec<&str> = first
        .observation()
        .facts()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, vec!["alpha", "mike", "zulu"]);
}

#[cfg(any())]
#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn compare_eq_reads_string_facts_then_parse_fact() {
    let env = sealed_with_facts(&[("enabled", "true"), ("count", "01")]);

    let bool_ok = evaluate_field(
        env.clone(),
        TestExpr::Eq(
            ValueExpr::Field(field_selector("enabled")),
            EvidenceValue::Boolean(true),
        ),
    );
    assert_eq!(bool_ok.effectiveness, Effectiveness::Effective);
    assert_eq!(bool_ok.rationale, "field enabled compared");

    let string_expected = evaluate_field(
        env.clone(),
        TestExpr::Eq(
            ValueExpr::Field(field_selector("enabled")),
            EvidenceValue::String("true".into()),
        ),
    );
    assert_eq!(
        string_expected.effectiveness,
        Effectiveness::Ineffective,
        "stored \"true\" is coerced to Boolean, so Eq String(\"true\") currently fails"
    );

    let leading_zero = evaluate_field(
        env,
        TestExpr::Eq(
            ValueExpr::Field(field_selector("count")),
            EvidenceValue::Integer(1),
        ),
    );
    assert_eq!(
        leading_zero.effectiveness,
        Effectiveness::Effective,
        "parse_fact turns \"01\" into Integer(1)"
    );

    let src = crate_sources_joined("weeping-angel-control-test");
    assert!(
        src.contains("let raw = env.observation().fact(field).unwrap_or(\"\");")
            && src.contains("let have = EvidenceValue::parse_fact(raw);"),
        "compare_eq must still read observation().fact() then parse_fact"
    );
}

#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn compare_numeric_uses_parse_fact_and_type_mismatch_on_non_integers() {
    let count = sealed_with_facts(&[("count", "2")]);
    let gte = evaluate_field(
        count,
        TestExpr::Gte(
            ValueExpr::Field(field_selector("count")),
            EvidenceValue::Integer(2),
        ),
    );
    assert_eq!(gte.effectiveness, Effectiveness::Effective);

    let decimal = sealed_with_facts(&[("count", "1.0")]);
    let mismatch = evaluate_field(
        decimal,
        TestExpr::Gte(
            ValueExpr::Field(field_selector("count")),
            EvidenceValue::Integer(1),
        ),
    );
    assert_eq!(mismatch.effectiveness, Effectiveness::Ineffective);
    assert!(
        mismatch.rationale.contains("type mismatch"),
        "numeric path currently surfaces type mismatch via as_integer; got {}",
        mismatch.rationale
    );

    let src = crate_sources_joined("weeping-angel-control-test");
    assert!(
        src.contains("let Some(raw) = env.observation().fact(field)")
            && src.contains("let have = EvidenceValue::parse_fact(raw);"),
        "compare_numeric must still read observation().fact() then parse_fact"
    );
}

#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn contains_and_in_are_unsupported_not_tested() {
    let env = sealed_with_facts(&[("privileged_roles", "owner,admin")]);
    let contains = evaluate_field(
        env.clone(),
        TestExpr::Contains(
            ValueExpr::Field(field_selector("privileged_roles")),
            EvidenceValue::String("owner".into()),
        ),
    );
    assert_eq!(contains.effectiveness, Effectiveness::NotTested);
    assert!(contains.rationale.contains("unsupported expression arm"));

    let membership = evaluate_field(
        env,
        TestExpr::In(
            ValueExpr::Field(field_selector("privileged_roles")),
            vec![EvidenceValue::String("owner".into())],
        ),
    );
    assert_eq!(membership.effectiveness, Effectiveness::NotTested);
}

#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn credential_reject_is_on_fact_keys_not_values() {
    let token = EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
        .with_fact("token", "ghp_example");
    match EvidenceEnvelope::seal(token, fresh_provenance()) {
        Err(EvidenceError::CredentialInPayload { key }) => assert_eq!(key, "token"),
        other => panic!("expected CredentialInPayload for key token, got {other:?}"),
    }

    let hyphenated = EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
        .with_fact("api-key", "secret");
    assert!(
        EvidenceEnvelope::seal(hyphenated, fresh_provenance()).is_err(),
        "hyphenated credential keys are folded and rejected"
    );

    let value_only = EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
        .with_fact("note", "token=ghp_example");
    assert!(
        EvidenceEnvelope::seal(value_only, fresh_provenance()).is_ok(),
        "credential-shaped values on ordinary keys are currently allowed"
    );
}

#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn ledger_round_trips_string_facts() {
    let env = sealed_with_facts(&[("enabled", "true"), ("required_reviewers", "2")]);
    let digest = env.digest().to_string();
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    assert!(ledger.append(env).unwrap());
    let loaded = ledger.get(&digest).unwrap();
    assert_eq!(loaded.digest(), digest);
    assert_eq!(loaded.observation().fact("enabled"), Some("true"));
    assert_eq!(loaded.observation().fact("required_reviewers"), Some("2"));

    let payload = serde_json::to_value(loaded.observation()).unwrap();
    assert_eq!(payload["facts"]["enabled"], Value::String("true".into()));
    assert_eq!(
        payload["facts"]["required_reviewers"],
        Value::String("2".into())
    );
}

#[test]
#[ignore = "superseded by sdd_typed_evidence_target"]
fn sealed_envelope_has_no_framework_or_provider_fields() {
    let env = sealed_with_facts(&[("enabled", "true")]);
    let json = serde_json::to_value(&env).unwrap();
    let object = json.as_object().expect("envelope object");
    for forbidden in [
        "frameworks",
        "iso27001",
        "gdpr",
        "soc2",
        "controlTestResult",
        "control_test_result",
    ] {
        assert!(
            !object.contains_key(forbidden),
            "envelope must not carry framework/provider field `{forbidden}`"
        );
    }
    assert!(env.observation().fact("iso27001").is_none());
    assert!(env.observation().fact("frameworks").is_none());
}
