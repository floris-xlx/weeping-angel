//! Structural IR validation. Compile receives already-valid documents.

use std::collections::BTreeSet;

use thiserror::Error;

use crate::{
    extension::{extension_key_is_well_formed, extensions_override_canonical},
    AssessmentDefinition, ASSURANCE_IR_SCHEMA,
};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IrValidationError {
    #[error("{0}")]
    Message(String),
}

pub trait ValidateIr {
    fn validate(&self) -> Result<(), IrValidationError>;
}

pub fn validate_assessment_ir(assessment: &AssessmentDefinition) -> Result<(), IrValidationError> {
    assessment.validate()
}

impl ValidateIr for AssessmentDefinition {
    fn validate(&self) -> Result<(), IrValidationError> {
        if self.schema_version != ASSURANCE_IR_SCHEMA {
            return Err(IrValidationError::Message(format!(
                "schema version mismatch: expected {ASSURANCE_IR_SCHEMA}, got {}",
                self.schema_version
            )));
        }

        let mut requirement_ids = BTreeSet::new();
        for req in &self.requirements {
            if !requirement_ids.insert(req.id().as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate requirement id {}",
                    req.id()
                )));
            }
            validate_extensions(req.framework().id().as_str())?;
        }

        let mut control_ids = BTreeSet::new();
        for control in &self.controls {
            if !control_ids.insert(control.id().as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate control id {}",
                    control.id()
                )));
            }
            for key in control.extensions().keys() {
                if !extension_key_is_well_formed(key) || extensions_override_canonical(key) {
                    return Err(IrValidationError::Message(format!(
                        "invalid extension namespace {key}"
                    )));
                }
            }
        }

        let mut evidence_ids = BTreeSet::new();
        for ev in &self.evidence_requirements {
            if !evidence_ids.insert(ev.id().as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate evidence requirement id {}",
                    ev.id()
                )));
            }
        }

        let risk_ids: BTreeSet<_> = self.risks.iter().map(|r| r.id.as_str().to_string()).collect();
        let exception_ids: BTreeSet<_> = self
            .exceptions
            .iter()
            .map(|e| e.id.as_str().to_string())
            .collect();

        for mapping in &self.mappings {
            if mapping.from_requirement().as_str() == mapping.to_control().as_str() {
                return Err(IrValidationError::Message(
                    "self mapping is not allowed".into(),
                ));
            }
            if !requirement_ids.contains(mapping.from_requirement().as_str())
                || !control_ids.contains(mapping.to_control().as_str())
            {
                return Err(IrValidationError::Message(format!(
                    "dangling mapping {} → {}",
                    mapping.from_requirement(),
                    mapping.to_control()
                )));
            }
            if let Some(req) = self
                .requirements
                .iter()
                .find(|r| r.id() == mapping.from_requirement())
                && !mapping.valid_for().contains(req.framework_version())
            {
                return Err(IrValidationError::Message(format!(
                    "mapping version constraint does not include {}",
                    req.framework_version()
                )));
            }
        }

        for test in &self.tests {
            if !control_ids.contains(test.control_id.as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling test {} control {}",
                    test.id, test.control_id
                )));
            }
        }

        for impln in &self.implementations {
            if !control_ids.contains(impln.control_id().as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling implementation control {}",
                    impln.control_id()
                )));
            }
            for risk in impln.risk_ids() {
                if !risk_ids.contains(risk.as_str()) {
                    return Err(IrValidationError::Message(format!(
                        "dangling risk reference {}",
                        risk
                    )));
                }
            }
            for exception in impln.exception_ids() {
                if !exception_ids.contains(exception.as_str()) {
                    return Err(IrValidationError::Message(format!(
                        "dangling exception reference {}",
                        exception
                    )));
                }
            }
        }

        Ok(())
    }
}

fn validate_extensions(_unused: &str) -> Result<(), IrValidationError> {
    Ok(())
}
