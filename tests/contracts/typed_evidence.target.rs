//! Target suite for typed evidence + canonical serialization (typed evidence).
//!
//! Encodes DESIRED behavior in `docs/specs/typed-evidence.md` §4 / §6.2 and
//! `docs/adr/0003-typed-evidence-canonical-serialization.md`. Must stay RED on
//! the current string-bag / `parse_fact` HEAD. Do not weaken these assertions
//! to match today's model, and do not implement the feature in this suite.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use serde::Serialize;
use serde_json::{Value, json};
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

fn require_needles(label: &str, src: &str, needles: &[&str]) {
    let missing: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| !src.contains(n))
        .collect();
    assert!(
        missing.is_empty(),
        "{label}: missing required surface {missing:?}"
    );
}

fn forbid_needles(label: &str, src: &str, needles: &[&str]) {
    let present: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| src.contains(*n))
        .collect();
    assert!(
        present.is_empty(),
        "{label}: forbidden leftover surface {present:?}"
    );
}

fn fresh_provenance() -> EvidenceProvenance {
    EvidenceProvenance {
        collector_id: "fixture.typed-evidence-target".into(),
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

/// Mirrors private `DigestBody` (observation + provenance only).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DigestBody<'a> {
    observation: &'a EvidenceObservation,
    provenance: &'a EvidenceProvenance,
}

fn observation_from_facts(facts: Value) -> EvidenceObservation {
    serde_json::from_value(json!({
        "evidenceType": "source.branch.protection",
        "facts": facts.clone(),
        "narrative": ""
    }))
    .unwrap_or_else(|e| {
        panic!("hybrid evidence-value/v1 observation facts must deserialize: {e}; facts={facts}")
    })
}

fn sealed_from_facts(facts: Value) -> EvidenceEnvelope {
    EvidenceEnvelope::seal(observation_from_facts(facts), fresh_provenance())
        .unwrap_or_else(|e| panic!("seal typed observation: {e}"))
}

fn sealed_with_fact_strings(pairs: &[(&str, &str)]) -> EvidenceEnvelope {
    let mut obs = EvidenceObservation::new(EvidenceType::new("source.branch.protection"));
    for (k, v) in pairs {
        obs = obs.with_fact(*k, *v);
    }
    EvidenceEnvelope::seal(obs, fresh_provenance()).unwrap()
}

/// Decode a stored/literal `EvidenceValue` via the hybrid `evidence-value/v1` codec.
fn ev(value: Value) -> EvidenceValue {
    serde_json::from_value(value.clone()).unwrap_or_else(|e| {
        panic!("hybrid evidence-value/v1 literal must deserialize: {e}; value={value}")
    })
}

fn compiled(expr: TestExpr) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new("test.typed-evidence.target"))
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

fn assert_type_mismatch(result: &weeping_angel_control_test::ControlTestResult) {
    assert_eq!(
        result.effectiveness,
        Effectiveness::Ineffective,
        "incompatible types must fail closed as Ineffective; got {:?} ({})",
        result.effectiveness,
        result.rationale
    );
    assert!(
        result.rationale.contains("type mismatch"),
        "deterministic type-mismatch rationale required, got {}",
        result.rationale
    );
}

#[test]
fn dual_suite_target_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_typed_evidence_target")
            && toml.contains("tests/contracts/typed_evidence.target.rs"),
        "target suite must be listed in root Cargo.toml"
    );
}

#[test]
fn one_evidence_value_lives_in_evidence_crate() {
    let evidence = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "weeping-angel-evidence typed value model",
        &evidence,
        &[
            "pub enum EvidenceValue",
            "Bool(bool)",
            "Integer(i64)",
            "DurationSeconds(u64)",
            "StringList(Vec<String>)",
            "Object(BTreeMap<String, EvidenceValue>)",
            "fn with_value",
            "fn fact_value",
            "$evidenceValue",
            "evidence-value/v1",
        ],
    );
    assert!(
        evidence.contains("Timestamp("),
        "Timestamp variant must be a stored native, not a string bag"
    );
    assert!(
        evidence.contains("Decimal("),
        "Decimal must be a text-backed variant, not f64"
    );
    forbid_needles(
        "evidence crate digest-critical float probe",
        &evidence,
        &["parse::<f64>()", "parse::<f32>()"],
    );

    let expr = fs::read_to_string(crate_src("weeping-angel-control-test").join("expr.rs")).unwrap();
    assert!(
        !expr.contains("pub enum EvidenceValue"),
        "control-test must re-export evidence EvidenceValue; it must not define a second enum"
    );

    let control = crate_sources_joined("weeping-angel-control-test");
    assert!(
        control.contains("weeping_angel_evidence::EvidenceValue")
            || control.contains("weeping_angel_evidence::{") && control.contains("EvidenceValue"),
        "control-test must consume EvidenceValue from weeping-angel-evidence"
    );
}

#[test]
fn observation_facts_are_typed_map_with_string_compat() {
    let src = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "typed observation fact map",
        &src,
        &[
            "facts: BTreeMap<String, EvidenceValue>",
            "pub fn with_fact(mut self, key: impl Into<String>, value: impl Into<String>)",
            "pub fn fact(&self, key: &str) -> Option<&str>",
        ],
    );
    assert!(
        src.contains("EvidenceValue::String") || src.contains("Self::String"),
        "with_fact must store EvidenceValue::String without coercing true/01/1.0"
    );

    let obs = EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
        .with_fact("enabled", "true")
        .with_fact("required_reviewers", "2")
        .with_fact("count", "01");
    assert_eq!(obs.fact("enabled"), Some("true"));
    assert_eq!(obs.fact("required_reviewers"), Some("2"));
    assert_eq!(obs.fact("count"), Some("01"));

    let payload = serde_json::to_value(&obs).unwrap();
    assert_eq!(payload["facts"]["enabled"], json!("true"));
    assert_eq!(payload["facts"]["count"], json!("01"));
    assert!(
        payload["facts"]["enabled"].is_string(),
        "with_fact must not silently encode \"true\" as a JSON bool"
    );
}

#[test]
fn hybrid_codec_round_trips_all_variants() {
    let facts = json!({
        "note": "hello",
        "branch_protected": true,
        "required_reviewers": 2,
        "privileged_roles": ["owner", "admin"],
        "empty_list": [],
        "empty_object": {},
        "nested": { "zulu": 1, "alpha": "keep" },
        "retention_ratio": { "$evidenceValue": "decimal", "value": "1.0" },
        "observed_at": { "$evidenceValue": "timestamp", "value": "2026-08-18T12:00:00.000Z" },
        "max_age": { "$evidenceValue": "durationSeconds", "value": 3600 },
        "min_i": i64::MIN,
        "max_i": i64::MAX
    });
    let obs = observation_from_facts(facts);
    assert_eq!(
        obs.fact("note"),
        Some("hello"),
        "String facts remain string-only via fact()"
    );
    assert_eq!(
        obs.fact("branch_protected"),
        None,
        "fact() must not stringify Bool(true)"
    );
    assert_eq!(
        obs.fact("required_reviewers"),
        None,
        "fact() must not stringify Integer"
    );

    let encoded = serde_json::to_value(&obs).unwrap();
    let facts = &encoded["facts"];
    assert_eq!(facts["note"], json!("hello"));
    assert_eq!(facts["branch_protected"], json!(true));
    assert_eq!(facts["required_reviewers"], json!(2));
    assert_eq!(facts["privileged_roles"], json!(["owner", "admin"]));
    assert_eq!(facts["empty_list"], json!([]));
    assert_eq!(facts["empty_object"], json!({}));
    assert_eq!(facts["nested"]["alpha"], json!("keep"));
    assert_eq!(facts["nested"]["zulu"], json!(1));
    assert_eq!(
        facts["retention_ratio"],
        json!({ "$evidenceValue": "decimal", "value": "1.0" })
    );
    assert_eq!(
        facts["observed_at"],
        json!({ "$evidenceValue": "timestamp", "value": "2026-08-18T12:00:00.000Z" })
    );
    assert_eq!(
        facts["max_age"],
        json!({ "$evidenceValue": "durationSeconds", "value": 3600 })
    );
    assert_eq!(facts["min_i"], json!(i64::MIN));
    assert_eq!(facts["max_i"], json!(i64::MAX));

    let again: EvidenceObservation = serde_json::from_value(encoded).unwrap();
    assert_eq!(
        serde_json::to_value(&again).unwrap()["facts"],
        serde_json::to_value(&obs).unwrap()["facts"]
    );
}

#[test]
fn no_silent_coerce_of_ambiguous_strings() {
    let obs = observation_from_facts(json!({
        "leading_zero": "01",
        "decimal_text": "1.0",
        "bool_text": "true",
        "false_text": "true"
    }));
    assert_eq!(obs.fact("leading_zero"), Some("01"));
    assert_eq!(obs.fact("decimal_text"), Some("1.0"));
    assert_eq!(obs.fact("bool_text"), Some("true"));

    let payload = serde_json::to_value(&obs).unwrap();
    assert_eq!(payload["facts"]["leading_zero"], json!("01"));
    assert_eq!(payload["facts"]["decimal_text"], json!("1.0"));
    assert_eq!(payload["facts"]["bool_text"], json!("true"));

    let native = sealed_from_facts(json!({
        "leading_zero": 1,
        "decimal_text": { "$evidenceValue": "decimal", "value": "1.0" },
        "bool_text": true
    }));
    let native_json = serde_json::to_value(native.observation()).unwrap();
    assert_eq!(native_json["facts"]["leading_zero"], json!(1));
    assert_eq!(native_json["facts"]["bool_text"], json!(true));

    let string_env = sealed_with_fact_strings(&[
        ("leading_zero", "01"),
        ("decimal_text", "1.0"),
        ("bool_text", "true"),
    ]);
    assert_ne!(
        string_env.digest(),
        native.digest(),
        "typed natives must not share identity with their string lookalikes"
    );
}

#[test]
fn deterministic_digest_under_insertion_order_and_nesting() {
    let first = sealed_from_facts(json!({
        "zulu": { "bravo": true, "alpha": 2 },
        "mike": ["owner", "admin"],
        "alpha": false
    }));
    let second = sealed_from_facts(json!({
        "alpha": false,
        "mike": ["owner", "admin"],
        "zulu": { "alpha": 2, "bravo": true }
    }));
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.content_digest(), second.content_digest());
    assert_eq!(first.evidence_id(), format!("ev:sha256:{}", first.digest()));

    let expected = canonical_digest(&DigestBody {
        observation: first.observation(),
        provenance: first.provenance(),
    })
    .unwrap();
    assert_eq!(first.digest(), expected);

    let swapped_list = sealed_from_facts(json!({
        "alpha": false,
        "mike": ["admin", "owner"],
        "zulu": { "alpha": 2, "bravo": true }
    }));
    assert_ne!(
        first.digest(),
        swapped_list.digest(),
        "StringList order is part of identity"
    );

    let decimal_a = sealed_from_facts(json!({
        "ratio": { "$evidenceValue": "decimal", "value": "1.0" }
    }));
    let decimal_b = sealed_from_facts(json!({
        "ratio": { "$evidenceValue": "decimal", "value": "1.00" }
    }));
    assert_ne!(
        decimal_a.digest(),
        decimal_b.digest(),
        "decimal identity is lexical; 1.0 ≠ 1.00"
    );
}

#[test]
fn timestamp_normalizes_to_utc_millis_z() {
    let offset = sealed_from_facts(json!({
        "observed_at": { "$evidenceValue": "timestamp", "value": "2026-08-18T13:00:00+01:00" }
    }));
    let utc = sealed_from_facts(json!({
        "observed_at": { "$evidenceValue": "timestamp", "value": "2026-08-18T12:00:00.000Z" }
    }));
    assert_eq!(offset.digest(), utc.digest());
    let encoded = serde_json::to_value(offset.observation()).unwrap();
    assert_eq!(
        encoded["facts"]["observed_at"],
        json!({ "$evidenceValue": "timestamp", "value": "2026-08-18T12:00:00.000Z" })
    );
}

#[test]
fn historical_string_fixtures_remain_digest_compatible() {
    let obs = EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
        .with_fact("enabled", "true")
        .with_fact("required_reviewers", "2");
    let provenance = fresh_provenance();
    let expected = canonical_digest(&DigestBody {
        observation: &obs,
        provenance: &provenance,
    })
    .unwrap();
    let env = EvidenceEnvelope::seal(obs, provenance).unwrap();
    assert_eq!(env.digest(), expected);
    let envelope_json = serde_json::to_value(&env).unwrap();
    assert_eq!(envelope_json["schemaVersion"], EVIDENCE_SCHEMA);

    let payload = serde_json::to_value(env.observation()).unwrap();
    assert_eq!(payload["facts"]["enabled"], json!("true"));
    assert_eq!(payload["facts"]["required_reviewers"], json!("2"));

    let loaded = observation_from_facts(json!({
        "enabled": "true",
        "required_reviewers": "2"
    }));
    assert_eq!(loaded.fact("enabled"), Some("true"));
    assert_eq!(loaded.fact("required_reviewers"), Some("2"));
}

#[test]
fn evaluator_consumes_stored_types_not_parse_fact() {
    let control = crate_sources_joined("weeping-angel-control-test");
    forbid_needles(
        "evaluate/compare path must not reparse strings",
        &control,
        &[
            "let have = EvidenceValue::parse_fact(raw);",
            "pub fn parse_fact(raw: &str)",
            "trimmed.parse::<f64>()",
        ],
    );
    require_needles(
        "evaluator reads stored EvidenceValue",
        &control,
        &["fact_value"],
    );

    let typed = sealed_from_facts(json!({
        "branch_protected": true,
        "required_reviewers": 2,
        "label": "protected"
    }));

    let bool_ok = evaluate_field(
        typed.clone(),
        TestExpr::Eq(
            ValueExpr::Field(field_selector("branch_protected")),
            ev(json!(true)),
        ),
    );
    assert_eq!(bool_ok.effectiveness, Effectiveness::Effective);

    let int_ok = evaluate_field(
        typed.clone(),
        TestExpr::Eq(
            ValueExpr::Field(field_selector("required_reviewers")),
            ev(json!(2)),
        ),
    );
    assert_eq!(int_ok.effectiveness, Effectiveness::Effective);

    let gte = evaluate_field(
        typed.clone(),
        TestExpr::Gte(
            ValueExpr::Field(field_selector("required_reviewers")),
            ev(json!(2)),
        ),
    );
    assert_eq!(gte.effectiveness, Effectiveness::Effective);

    let string_ok = evaluate_field(
        typed,
        TestExpr::Eq(
            ValueExpr::Field(field_selector("label")),
            ev(json!("protected")),
        ),
    );
    assert_eq!(string_ok.effectiveness, Effectiveness::Effective);
}

#[test]
fn incompatible_types_fail_closed_without_coercing_lookalikes() {
    let strings =
        sealed_with_fact_strings(&[("enabled", "true"), ("count", "01"), ("ratio", "1.0")]);

    let bool_vs_string = evaluate_field(
        strings.clone(),
        TestExpr::Eq(ValueExpr::Field(field_selector("enabled")), ev(json!(true))),
    );
    assert_type_mismatch(&bool_vs_string);

    let string_eq = evaluate_field(
        strings.clone(),
        TestExpr::Eq(
            ValueExpr::Field(field_selector("enabled")),
            ev(json!("true")),
        ),
    );
    assert_eq!(
        string_eq.effectiveness,
        Effectiveness::Effective,
        "stored String(\"true\") must equal String(\"true\") without parse_fact"
    );

    let leading_zero = evaluate_field(
        strings.clone(),
        TestExpr::Eq(ValueExpr::Field(field_selector("count")), ev(json!(1))),
    );
    assert_type_mismatch(&leading_zero);

    let decimal_lookalike = evaluate_field(
        strings,
        TestExpr::Eq(
            ValueExpr::Field(field_selector("ratio")),
            ev(json!({ "$evidenceValue": "decimal", "value": "1.0" })),
        ),
    );
    assert_type_mismatch(&decimal_lookalike);
}

#[test]
fn string_list_contains_and_in_use_stored_types() {
    let env = sealed_from_facts(json!({
        "privileged_roles": ["owner", "admin"],
        "role": "admin"
    }));

    let contains = evaluate_field(
        env.clone(),
        TestExpr::Contains(
            ValueExpr::Field(field_selector("privileged_roles")),
            ev(json!("owner")),
        ),
    );
    assert_eq!(contains.effectiveness, Effectiveness::Effective);

    let missing = evaluate_field(
        env.clone(),
        TestExpr::Contains(
            ValueExpr::Field(field_selector("privileged_roles")),
            ev(json!("guest")),
        ),
    );
    assert_eq!(missing.effectiveness, Effectiveness::Ineffective);

    let membership = evaluate_field(
        env,
        TestExpr::In(
            ValueExpr::Field(field_selector("role")),
            vec![ev(json!("owner")), ev(json!("admin"))],
        ),
    );
    assert_eq!(membership.effectiveness, Effectiveness::Effective);
}

#[test]
fn credential_rejection_walks_typed_and_nested_object_keys() {
    let top = EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
        .with_fact("token", "ghp_example");
    match EvidenceEnvelope::seal(top, fresh_provenance()) {
        Err(EvidenceError::CredentialInPayload { key }) => assert_eq!(key, "token"),
        other => panic!("expected CredentialInPayload for key token, got {other:?}"),
    }

    let nested = observation_from_facts(json!({
        "profile": { "display": "ok", "api-key": "secret" }
    }));
    match EvidenceEnvelope::seal(nested, fresh_provenance()) {
        Err(EvidenceError::CredentialInPayload { key }) => {
            assert!(
                key.contains("api-key") || key.contains("api_key"),
                "nested credential key must be reported, got {key}"
            );
        }
        other => panic!("expected nested CredentialInPayload, got {other:?}"),
    }

    let reserved = observation_from_facts(json!({
        "meta": { "ok": 1 }
    }));
    let src = crate_sources_joined("weeping-angel-evidence");
    require_needles("reserved tagged-wrapper key", &src, &["$evidenceValue"]);
    let _ = reserved;
}

#[test]
fn ledger_round_trips_typed_values() {
    let env = sealed_from_facts(json!({
        "branch_protected": true,
        "required_reviewers": 2,
        "retention_days": 365,
        "privileged_roles": ["owner", "admin"],
        "ratio": { "$evidenceValue": "decimal", "value": "1.0" },
        "observed_at": { "$evidenceValue": "timestamp", "value": "2026-08-18T12:00:00.000Z" },
        "window": { "$evidenceValue": "durationSeconds", "value": 60 },
        "nested": { "alpha": false }
    }));
    let digest = env.digest().to_string();
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    assert!(ledger.append(env).unwrap());
    let loaded = ledger.get(&digest).unwrap();
    assert_eq!(loaded.digest(), digest);

    let facts = serde_json::to_value(loaded.observation()).unwrap()["facts"].clone();
    assert_eq!(facts["branch_protected"], json!(true));
    assert_eq!(facts["required_reviewers"], json!(2));
    assert_eq!(facts["retention_days"], json!(365));
    assert_eq!(facts["privileged_roles"], json!(["owner", "admin"]));
    assert_eq!(
        facts["ratio"],
        json!({ "$evidenceValue": "decimal", "value": "1.0" })
    );
    assert_eq!(
        facts["observed_at"],
        json!({ "$evidenceValue": "timestamp", "value": "2026-08-18T12:00:00.000Z" })
    );
    assert_eq!(
        facts["window"],
        json!({ "$evidenceValue": "durationSeconds", "value": 60 })
    );
    assert_eq!(facts["nested"]["alpha"], json!(false));
}

#[test]
fn sealed_envelope_still_has_no_framework_or_provider_fields() {
    let env = sealed_from_facts(json!({
        "branch_protected": true,
        "required_reviewers": 2
    }));
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
    assert_eq!(json["schemaVersion"], EVIDENCE_SCHEMA);
    assert!(env.observation().fact("iso27001").is_none());
    assert!(env.observation().fact("frameworks").is_none());
}

#[test]
fn handoff_examples_construct_seal_digest_evaluate_and_ledger() {
    let src = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "handoff constructors",
        &src,
        &[
            "fn with_value",
            "EvidenceValue::Bool",
            "EvidenceValue::Integer",
            "EvidenceValue::StringList",
        ],
    );

    let env = sealed_from_facts(json!({
        "branch_protected": true,
        "required_reviewers": 2,
        "retention_days": 365,
        "privileged_roles": ["owner", "admin"]
    }));
    assert_eq!(
        env.digest(),
        canonical_digest(&DigestBody {
            observation: env.observation(),
            provenance: env.provenance(),
        })
        .unwrap()
    );

    let protected = evaluate_field(
        env.clone(),
        TestExpr::Eq(
            ValueExpr::Field(field_selector("branch_protected")),
            ev(json!(true)),
        ),
    );
    assert_eq!(protected.effectiveness, Effectiveness::Effective);

    let reviewers = evaluate_field(
        env.clone(),
        TestExpr::Gte(
            ValueExpr::Field(field_selector("required_reviewers")),
            ev(json!(2)),
        ),
    );
    assert_eq!(reviewers.effectiveness, Effectiveness::Effective);

    let retention = evaluate_field(
        env.clone(),
        TestExpr::Eq(
            ValueExpr::Field(field_selector("retention_days")),
            ev(json!(365)),
        ),
    );
    assert_eq!(retention.effectiveness, Effectiveness::Effective);

    let roles = evaluate_field(
        env.clone(),
        TestExpr::Contains(
            ValueExpr::Field(field_selector("privileged_roles")),
            ev(json!("admin")),
        ),
    );
    assert_eq!(roles.effectiveness, Effectiveness::Effective);

    let digest = env.digest().to_string();
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    assert!(ledger.append(env).unwrap());
    let loaded = ledger.get(&digest).unwrap();
    let facts = serde_json::to_value(loaded.observation()).unwrap()["facts"].clone();
    assert_eq!(facts["branch_protected"], json!(true));
    assert_eq!(facts["required_reviewers"], json!(2));
    assert_eq!(facts["retention_days"], json!(365));
    assert_eq!(facts["privileged_roles"], json!(["owner", "admin"]));
}
