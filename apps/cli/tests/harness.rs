//! One integration-test harness for package `weeping-angel` (DEBT-ENV P0).
//! Former explicit contract suites, autodiscovered `tests/*.rs`, and `e2e_*`
//! run as modules. `demo` is gated inside so contracts do not require it.

#[path = "../../../tests/contracts/applicability_engine.target.rs"]
mod applicability_engine_target;
#[path = "../../../tests/contracts/assessment_lineage.target.rs"]
mod assessment_lineage_target;
#[path = "../../../tests/contracts/assurance_runtime.target.rs"]
mod assurance_runtime_target;
#[path = "../../../tests/contracts/canonical_assurance_catalog.target.rs"]
mod canonical_assurance_catalog_target;
#[path = "../../../tests/contracts/collector_hexagonal.target.rs"]
mod collector_hexagonal_target;
#[path = "../../../tests/contracts/compliance_ir.target.rs"]
mod compliance_ir_target;
#[path = "../../../tests/contracts/continuity_resilience.target.rs"]
mod continuity_resilience_target;
#[path = "../../../tests/contracts/continuous_assurance_scheduler.target.rs"]
mod continuous_assurance_scheduler_target;
#[path = "../../../tests/contracts/control_implementation_registry.target.rs"]
mod control_implementation_registry_target;
#[path = "../../../tests/contracts/controlled_documents.target.rs"]
mod controlled_documents_target;
#[path = "../../../tests/contracts/documentation_layout.rs"]
mod documentation_layout;
#[path = "../../../tests/contracts/evidence_validity_temporal_assurance.target.rs"]
mod evidence_validity_temporal_assurance_target;
#[path = "../../../tests/contracts/github_collector.target.rs"]
mod github_collector_target;
#[path = "../../../tests/contracts/governance_catalog.target.rs"]
mod governance_catalog_target;
#[path = "../../../tests/contracts/iam_catalog.target.rs"]
mod iam_catalog_target;
#[path = "../../../tests/contracts/incident_governance.target.rs"]
mod incident_governance_target;
#[path = "../../../tests/contracts/infrastructure_catalog.target.rs"]
mod infrastructure_catalog_target;
#[path = "../../../tests/contracts/interested_parties_obligations.target.rs"]
mod interested_parties_obligations_target;
#[path = "../../../tests/contracts/internal_audit.target.rs"]
mod internal_audit_target;
#[path = "../../../tests/contracts/isms_context.target.rs"]
mod isms_context_target;
#[path = "../../../tests/contracts/isms_events_drift.target.rs"]
mod isms_events_drift_target;
#[path = "../../../tests/contracts/iso27001_assurance.target.rs"]
mod iso27001_assurance_target;
#[path = "../../../tests/contracts/iso27001_remap.target.rs"]
mod iso27001_remap_target;
#[path = "../../../tests/contracts/nonconformity_capa.target.rs"]
mod nonconformity_capa_target;
#[path = "../../../tests/contracts/operational_soa.target.rs"]
mod operational_soa_target;
#[path = "../../../tests/contracts/personnel_security.target.rs"]
mod personnel_security_target;
#[path = "../../../tests/contracts/population_runtime.target.rs"]
mod population_runtime_target;
#[path = "../../../tests/contracts/remediation_engine.target.rs"]
mod remediation_engine_target;
#[path = "../../../tests/contracts/repository_hygiene.target.rs"]
mod repository_hygiene_target;
#[path = "../../../tests/contracts/repository_integrity.target.rs"]
mod repository_integrity_target;
#[path = "../../../tests/contracts/residual_risk.target.rs"]
mod residual_risk_target;
#[path = "../../../tests/contracts/risk_identification.target.rs"]
mod risk_identification_target;
#[path = "../../../tests/contracts/risk_methodology.target.rs"]
mod risk_methodology_target;
#[path = "../../../tests/contracts/risk_register.target.rs"]
mod risk_register_target;
#[path = "../../../tests/contracts/risk_treatment.target.rs"]
mod risk_treatment_target;
#[path = "../../../tests/contracts/scope_engine.target.rs"]
mod scope_engine_target;
#[path = "../../../tests/contracts/sdlc_catalog.target.rs"]
mod sdlc_catalog_target;
#[path = "../../../tests/contracts/security_objectives.target.rs"]
mod security_objectives_target;
#[path = "../../../tests/contracts/supplier_risk.target.rs"]
mod supplier_risk_target;
#[path = "../../../tests/contracts/temporal_assurance.target.rs"]
mod temporal_assurance_target;
#[path = "../../../tests/contracts/temporal_lineage_evidence_soa.target.rs"]
mod temporal_lineage_evidence_soa_target;
#[path = "../../../tests/contracts/typed_evidence.target.rs"]
mod typed_evidence_target;
#[path = "../../../tests/contracts/vulnerability_catalog.target.rs"]
mod vulnerability_catalog_target;

#[path = "../../../tests/authz_cli.rs"]
mod authz_cli;
#[path = "../../../tests/authz_scope.rs"]
mod authz_scope;
#[path = "../../../tests/cli_parse.rs"]
mod cli_parse;
#[path = "../../../tests/code_engines.rs"]
mod code_engines;
#[path = "../../../tests/contract_spine.rs"]
mod contract_spine;
#[path = "../../../tests/depcheck_engine.rs"]
mod depcheck_engine;
#[path = "../../../tests/depcheck_parsers.rs"]
mod depcheck_parsers;
#[path = "../../../tests/discovery_unit.rs"]
mod discovery_unit;
#[path = "../../../tests/docs_export_cli.rs"]
mod docs_export_cli;
#[path = "../../../tests/exit_and_config.rs"]
mod exit_and_config;
#[path = "../../../tests/parse_helpers.rs"]
mod parse_helpers;
#[path = "../../../tests/report_formats.rs"]
mod report_formats;
#[path = "../../../tests/style_and_log.rs"]
mod style_and_log;
#[path = "../../../tests/target_matrix.rs"]
mod target_matrix;

#[cfg(feature = "demo")]
#[path = "../../../tests/e2e_demo.rs"]
mod e2e_demo;
#[cfg(feature = "demo")]
#[path = "../../../tests/e2e_recon.rs"]
mod e2e_recon;
