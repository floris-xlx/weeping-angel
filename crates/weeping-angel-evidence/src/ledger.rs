//! Persistent immutable evidence ledger. Owns observations, never conclusions.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::validity::{EvidenceValidityEvent, EvidenceValidityKind, project_validity};
use crate::{CollectionRun, EVIDENCE_SCHEMA, EvidenceEnvelope, EvidenceType};

thread_local! {
    static PRIOR_ENVELOPES: RefCell<BTreeMap<String, EvidenceEnvelope>> =
        const { RefCell::new(BTreeMap::new()) };
    static PRIOR_EVENTS: RefCell<Vec<EvidenceValidityEvent>> = const { RefCell::new(Vec::new()) };
}

/// Typed corrupt-payload failure. Distinct from outbound [`LedgerError::Serialize`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Corrupt(pub String);

/// Typed schema-version failure. Distinct from outbound [`LedgerError::Serialize`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncompatibleSchema {
    pub found: String,
    pub expected: String,
}

/// Persistence integrity failures. Mapped onto [`LedgerError::Path`] so HEAD
/// characterization matches on [`LedgerError`] remain exhaustive; names are
/// Guard 12 SSOT (`Corrupt` / `IncompatibleSchema`).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PersistenceIntegrity {
    #[error("corrupt: {0}")]
    Corrupt(Corrupt),
    #[error("incompatible schema: found {found}, expected {expected}")]
    IncompatibleSchema { found: String, expected: String },
}

impl std::fmt::Display for Corrupt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serialize: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("envelope not found: {0}")]
    NotFound(String),
    #[error("path rejected: {0}")]
    Path(String),
    #[error(
        "immutable lineage row already stored for {0}; replacing a completed payload is rejected"
    )]
    Immutable(String),
}

impl From<PersistenceIntegrity> for LedgerError {
    fn from(failure: PersistenceIntegrity) -> Self {
        LedgerError::Path(failure.to_string())
    }
}

impl From<Corrupt> for PersistenceIntegrity {
    fn from(value: Corrupt) -> Self {
        PersistenceIntegrity::Corrupt(value)
    }
}

impl From<IncompatibleSchema> for PersistenceIntegrity {
    fn from(value: IncompatibleSchema) -> Self {
        PersistenceIntegrity::IncompatibleSchema {
            found: value.found,
            expected: value.expected,
        }
    }
}

/// Prior valid envelopes remembered in-process so a failed collection cannot
/// evaluate an implicit empty world when ledger evidence already exists.
pub fn prior_valid_envelopes(at: DateTime<Utc>) -> Vec<EvidenceEnvelope> {
    PRIOR_ENVELOPES.with(|envs| {
        PRIOR_EVENTS.with(|events| {
            let events = events.borrow();
            envs.borrow()
                .values()
                .filter(|env| project_validity(env, &events, at).is_some())
                .cloned()
                .collect()
        })
    })
}

fn remember_envelope(env: &EvidenceEnvelope) {
    PRIOR_ENVELOPES.with(|envs| {
        envs.borrow_mut()
            .insert(env.digest().to_string(), env.clone());
    });
}

fn remember_event(event: &EvidenceValidityEvent) {
    PRIOR_EVENTS.with(|events| {
        let mut events = events.borrow_mut();
        if events
            .iter()
            .any(|existing| existing.event_id == event.event_id)
        {
            return;
        }
        events.push(event.clone());
    });
}

/// Append-only evidence store. Owns observations, never conclusions.
pub struct EvidenceLedger {
    conn: Connection,
}

impl EvidenceLedger {
    pub fn open_in_memory() -> Result<Self, LedgerError> {
        let conn = Connection::open_in_memory()?;
        let ledger = Self { conn };
        ledger.init()?;
        Ok(ledger)
    }

    pub fn open(path: &Path) -> Result<Self, LedgerError> {
        let raw = path.to_string_lossy();
        if raw.contains("..") {
            return Err(LedgerError::Path(
                "refusing path traversal in sqlite locator".into(),
            ));
        }
        let conn = Connection::open(path)?;
        let ledger = Self { conn };
        ledger.init()?;
        Ok(ledger)
    }

    fn init(&self) -> Result<(), LedgerError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS evidence_envelopes (
                digest TEXT PRIMARY KEY,
                evidence_id TEXT NOT NULL,
                collection_run_id TEXT NOT NULL,
                evidence_type TEXT NOT NULL,
                subject TEXT NOT NULL,
                collected_at TEXT NOT NULL,
                supersedes TEXT,
                payload TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS evidence_artifacts (
                artifact_id TEXT PRIMARY KEY,
                digest TEXT NOT NULL,
                media_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                storage_locator TEXT NOT NULL,
                redaction_state TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS collection_runs (
                run_id TEXT PRIMARY KEY,
                payload TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS assessment_runs (
                id TEXT PRIMARY KEY,
                payload TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS control_test_runs (
                id TEXT PRIMARY KEY,
                payload TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS framework_snapshots (
                digest TEXT PRIMARY KEY,
                payload TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS evidence_validity_events (
                event_id TEXT PRIMARY KEY,
                envelope_digest TEXT NOT NULL,
                at TEXT NOT NULL,
                kind TEXT NOT NULL,
                payload TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn append(&mut self, envelope: EvidenceEnvelope) -> Result<bool, LedgerError> {
        // Envelope row is INSERT OR IGNORE by digest. Envelope + asserted event
        // commit in one SQLite transaction (BEGIN/COMMIT).
        let tx = self.conn.unchecked_transaction()?;
        let inserted = append_envelope_row(&tx, &envelope)?;
        if inserted {
            let event = EvidenceValidityEvent::asserted_for(&envelope)
                .map_err(|e| LedgerError::Path(e.to_string()))?;
            record_validity_event_on(&tx, event)?;
        }
        tx.commit()?;
        remember_envelope(&envelope);
        Ok(inserted)
    }

    pub fn get(&self, digest: &str) -> Result<EvidenceEnvelope, LedgerError> {
        let payload: String = self
            .conn
            .query_row(
                "SELECT payload FROM evidence_envelopes WHERE digest = ?1",
                [digest],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| LedgerError::NotFound(digest.into()))?;
        parse_envelope_payload(digest, &payload)
    }

    pub fn query(&self) -> Result<Vec<EvidenceEnvelope>, LedgerError> {
        self.load_where("")
    }

    pub fn latest(
        &self,
        evidence_type: &EvidenceType,
    ) -> Result<Option<EvidenceEnvelope>, LedgerError> {
        let payload: Option<String> = self
            .conn
            .query_row(
                "SELECT payload FROM evidence_envelopes
                 WHERE evidence_type = ?1
                 ORDER BY collected_at DESC LIMIT 1",
                [evidence_type.as_str()],
                |row| row.get(0),
            )
            .optional()?;
        payload.map(|p| decode_envelope_payload(&p)).transpose()
    }

    /// Live valid evaluation leaf at `Utc::now()`. Never record-order [`Self::latest`].
    pub fn current(
        &self,
        evidence_type: &EvidenceType,
    ) -> Result<Option<EvidenceEnvelope>, LedgerError> {
        self.as_of(evidence_type, Utc::now())
    }

    /// Membership set of envelopes in force at instant `t` (half-open window).
    pub fn valid_at(
        &self,
        evidence_type: &EvidenceType,
        t: DateTime<Utc>,
    ) -> Result<Vec<EvidenceEnvelope>, LedgerError> {
        let events = self.validity_events()?;
        let mut members = Vec::new();
        for env in self.for_type(evidence_type)? {
            if project_validity(&env, &events, t).is_some() {
                members.push(env);
            }
        }
        members.sort_by(|a, b| a.digest().cmp(b.digest()));
        Ok(members)
    }

    /// Pinned-assessment evaluation leaf at `t`. Alias of this algorithm: [`Self::latest_as_of`].
    pub fn as_of(
        &self,
        evidence_type: &EvidenceType,
        t: DateTime<Utc>,
    ) -> Result<Option<EvidenceEnvelope>, LedgerError> {
        let events = self.validity_events()?;
        let mut candidates = Vec::new();
        for env in self.for_type(evidence_type)? {
            if project_validity(&env, &events, t).is_some() {
                candidates.push(env);
            }
        }
        Ok(select_leaf_as_of(&candidates, t, &events))
    }

    pub fn for_subject(&self, subject: &str) -> Result<Vec<EvidenceEnvelope>, LedgerError> {
        self.load_params(
            "SELECT payload FROM evidence_envelopes WHERE subject = ?1 ORDER BY collected_at",
            [subject],
        )
    }

    pub fn for_type(
        &self,
        evidence_type: &EvidenceType,
    ) -> Result<Vec<EvidenceEnvelope>, LedgerError> {
        self.load_params(
            "SELECT payload FROM evidence_envelopes WHERE evidence_type = ?1 ORDER BY collected_at",
            [evidence_type.as_str()],
        )
    }

    pub fn for_collection_run(&self, run_id: &str) -> Result<Vec<EvidenceEnvelope>, LedgerError> {
        self.load_params(
            "SELECT payload FROM evidence_envelopes WHERE collection_run_id = ?1 ORDER BY collected_at",
            [run_id],
        )
    }

    pub fn within_window(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<EvidenceEnvelope>, LedgerError> {
        self.load_params(
            "SELECT payload FROM evidence_envelopes
             WHERE collected_at >= ?1 AND collected_at <= ?2
             ORDER BY collected_at",
            [start.to_rfc3339(), end.to_rfc3339()],
        )
    }

    /// Record a new envelope that supersedes a prior digest. History is kept.
    pub fn supersede(
        &mut self,
        previous_digest: &str,
        mut next: EvidenceEnvelope,
    ) -> Result<EvidenceEnvelope, LedgerError> {
        let _previous = self.get(previous_digest)?;
        next = next.with_supersedes(previous_digest);
        self.append(next.clone())?;
        let at = next.provenance().collected_at;
        let superseded = EvidenceValidityEvent::superseded(previous_digest, at)
            .map_err(|e| LedgerError::Path(e.to_string()))?;
        self.record_validity_event(superseded)?;
        Ok(next)
    }

    /// Persist an append-only validity event. Identical bytes are a no-op;
    /// a second write of different bytes for the same `eventId` is `Immutable`.
    pub fn record_validity_event(
        &mut self,
        event: EvidenceValidityEvent,
    ) -> Result<bool, LedgerError> {
        record_validity_event_on(&self.conn, event)
    }

    pub fn validity_events(&self) -> Result<Vec<EvidenceValidityEvent>, LedgerError> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM evidence_validity_events ORDER BY at, event_id")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(decode_json(&row?)?);
        }
        Ok(out)
    }

    pub fn validity_events_for(
        &self,
        envelope_digest: &str,
    ) -> Result<Vec<EvidenceValidityEvent>, LedgerError> {
        self.load_validity_params(
            "SELECT payload FROM evidence_validity_events
             WHERE envelope_digest = ?1 ORDER BY at, event_id",
            [envelope_digest],
        )
    }

    /// Envelopes whose validity window overlaps `[start, end)` (half-open).
    /// Distinct from [`Self::within_window`], which filters inclusive `collected_at`.
    pub fn valid_during(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<EvidenceEnvelope>, LedgerError> {
        let events = self.validity_events()?;
        let mut out = Vec::new();
        for env in self.query()? {
            if validity_overlaps(&env, &events, start, end) {
                out.push(env);
            }
        }
        out.sort_by(|a, b| {
            a.valid_from()
                .cmp(&b.valid_from())
                .then_with(|| a.digest().cmp(b.digest()))
        });
        Ok(out)
    }

    /// Compatibility alias of [`Self::as_of`] (evaluation leaf, not record-order latest).
    pub fn latest_as_of(
        &self,
        evidence_type: &EvidenceType,
        as_of: DateTime<Utc>,
    ) -> Result<Option<EvidenceEnvelope>, LedgerError> {
        self.as_of(evidence_type, as_of)
    }

    fn load_validity_params<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<EvidenceValidityEvent>, LedgerError> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params, |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(decode_json(&row?)?);
        }
        Ok(out)
    }

    pub fn record_collection_run(&mut self, run: &CollectionRun) -> Result<(), LedgerError> {
        let payload = serde_json::to_string(run)?;
        let existing: Option<String> = self
            .conn
            .query_row(
                "SELECT payload FROM collection_runs WHERE run_id = ?1",
                [&run.run_id],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(existing) = existing {
            if existing == payload {
                return Ok(());
            }
            if payload_is_completed(&existing) {
                return Err(LedgerError::Immutable(run.run_id.clone()));
            }
            self.conn.execute(
                "UPDATE collection_runs SET payload = ?1 WHERE run_id = ?2",
                params![payload, run.run_id],
            )?;
            return Ok(());
        }
        self.conn.execute(
            "INSERT INTO collection_runs (run_id, payload) VALUES (?1, ?2)",
            params![run.run_id, payload],
        )?;
        Ok(())
    }

    pub fn persist_assessment_run(&mut self, id: &str, payload: &str) -> Result<bool, LedgerError> {
        validate_persisted_document(payload)?;
        persist_immutable(&self.conn, "assessment_runs", "id", id, payload)
    }

    pub fn list_assessment_runs(&self) -> Result<Vec<(String, String)>, LedgerError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, payload FROM assessment_runs ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn load_assessment_run(&self, id: &str) -> Result<String, LedgerError> {
        load_payload(&self.conn, "assessment_runs", "id", id)
    }

    pub fn persist_control_test_run(
        &mut self,
        id: &str,
        payload: &str,
    ) -> Result<bool, LedgerError> {
        persist_immutable(&self.conn, "control_test_runs", "id", id, payload)
    }

    pub fn load_control_test_run(&self, id: &str) -> Result<String, LedgerError> {
        load_payload(&self.conn, "control_test_runs", "id", id)
    }

    pub fn persist_framework_snapshot(
        &mut self,
        digest: &str,
        payload: &str,
    ) -> Result<bool, LedgerError> {
        persist_immutable(&self.conn, "framework_snapshots", "digest", digest, payload)
    }

    pub fn load_framework_snapshot(&self, digest: &str) -> Result<String, LedgerError> {
        load_payload(&self.conn, "framework_snapshots", "digest", digest)
    }

    fn load_where(&self, extra: &str) -> Result<Vec<EvidenceEnvelope>, LedgerError> {
        let sql =
            format!("SELECT payload FROM evidence_envelopes {extra} ORDER BY collected_at, digest");
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(decode_envelope_payload(&row?)?);
        }
        Ok(out)
    }

    fn load_params<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<EvidenceEnvelope>, LedgerError> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params, |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(decode_envelope_payload(&row?)?);
        }
        Ok(out)
    }

    pub fn index_by_digest(&self) -> Result<BTreeMap<String, EvidenceEnvelope>, LedgerError> {
        let mut map = BTreeMap::new();
        for env in self.query()? {
            map.insert(env.digest().to_string(), env);
        }
        Ok(map)
    }
}

fn append_envelope_row(
    conn: &Connection,
    envelope: &EvidenceEnvelope,
) -> Result<bool, LedgerError> {
    let inserted = conn.execute(
        "INSERT OR IGNORE INTO evidence_envelopes
         (digest, evidence_id, collection_run_id, evidence_type, subject, collected_at, supersedes, payload)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            envelope.digest(),
            envelope.evidence_id(),
            envelope.collection_run_id(),
            envelope.observation().evidence_type().as_str(),
            envelope.provenance().asset().as_str(),
            envelope.provenance().collected_at.to_rfc3339(),
            envelope.supersedes(),
            serde_json::to_string(envelope)?,
        ],
    )?;
    Ok(inserted == 1)
}

fn record_validity_event_on(
    conn: &Connection,
    event: EvidenceValidityEvent,
) -> Result<bool, LedgerError> {
    let payload = serde_json::to_string(&event)?;
    let existing: Option<String> = conn
        .query_row(
            "SELECT payload FROM evidence_validity_events WHERE event_id = ?1",
            [&event.event_id],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(existing) = existing {
        if existing == payload {
            remember_event(&event);
            return Ok(false);
        }
        return Err(LedgerError::Immutable(event.event_id));
    }
    let kind = match event.kind {
        EvidenceValidityKind::Asserted => "asserted",
        EvidenceValidityKind::Superseded => "superseded",
        EvidenceValidityKind::Revoked => "revoked",
        EvidenceValidityKind::Invalidated => "invalidated",
    };
    conn.execute(
        "INSERT INTO evidence_validity_events
         (event_id, envelope_digest, at, kind, payload)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            event.event_id,
            event.envelope_digest,
            event.at.to_rfc3339(),
            kind,
            payload,
        ],
    )?;
    remember_event(&event);
    Ok(true)
}

fn decode_json<T: serde::de::DeserializeOwned>(payload: &str) -> Result<T, LedgerError> {
    serde_json::from_str(payload)
        .map_err(|e| PersistenceIntegrity::from(Corrupt(e.to_string())).into())
}

fn decode_envelope_payload(payload: &str) -> Result<EvidenceEnvelope, LedgerError> {
    let env: EvidenceEnvelope = decode_json(payload)?;
    parse_envelope_payload(env.digest(), payload)
}

fn parse_envelope_payload(digest: &str, payload: &str) -> Result<EvidenceEnvelope, LedgerError> {
    let env: EvidenceEnvelope = decode_json(payload)?;
    if !digest.is_empty() && env.digest() != digest {
        return Err(PersistenceIntegrity::from(Corrupt(format!(
            "digest/key mismatch: key {digest} payload {}",
            env.digest()
        )))
        .into());
    }
    if !env.schema_version().is_empty() && env.schema_version() != EVIDENCE_SCHEMA {
        return Err(PersistenceIntegrity::from(IncompatibleSchema {
            found: env.schema_version().into(),
            expected: EVIDENCE_SCHEMA.into(),
        })
        .into());
    }
    remember_envelope(&env);
    Ok(env)
}

fn validate_persisted_document(payload: &str) -> Result<(), LedgerError> {
    let value: serde_json::Value = serde_json::from_str(payload).map_err(|_| {
        PersistenceIntegrity::from(IncompatibleSchema {
            found: "non-json".into(),
            expected: "application/json".into(),
        })
    })?;
    if let Some(found) = value
        .get("schemaVersion")
        .or_else(|| value.get("schema"))
        .and_then(|v| v.as_str())
        && !known_document_schema(found)
    {
        return Err(PersistenceIntegrity::from(IncompatibleSchema {
            found: found.into(),
            expected: EVIDENCE_SCHEMA.into(),
        })
        .into());
    }
    Ok(())
}

fn known_document_schema(schema: &str) -> bool {
    schema == EVIDENCE_SCHEMA
        || schema == crate::EVIDENCE_VALIDITY_SCHEMA
        || schema == "weeping-angel/assessment-lineage/v1"
        || schema.starts_with("weeping-angel/")
        || schema == "evidence/v1"
}

fn persist_immutable(
    conn: &Connection,
    table: &str,
    key_col: &str,
    key: &str,
    payload: &str,
) -> Result<bool, LedgerError> {
    let existing: Option<String> = conn
        .query_row(
            &format!("SELECT payload FROM {table} WHERE {key_col} = ?1"),
            [key],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(existing) = existing {
        if existing == payload {
            return Ok(false);
        }
        if payload_is_completed(&existing) {
            return Err(LedgerError::Immutable(key.into()));
        }
        return Err(LedgerError::Immutable(key.into()));
    }
    let inserted = conn.execute(
        &format!("INSERT OR IGNORE INTO {table} ({key_col}, payload) VALUES (?1, ?2)"),
        params![key, payload],
    )?;
    Ok(inserted == 1)
}

fn load_payload(
    conn: &Connection,
    table: &str,
    key_col: &str,
    key: &str,
) -> Result<String, LedgerError> {
    conn.query_row(
        &format!("SELECT payload FROM {table} WHERE {key_col} = ?1"),
        [key],
        |row| row.get(0),
    )
    .optional()?
    .ok_or_else(|| LedgerError::NotFound(key.into()))
}

fn payload_is_completed(payload: &str) -> bool {
    payload.contains("\"status\":\"completed\"") || payload.contains("\"status\": \"completed\"")
}

fn validity_overlaps(
    env: &EvidenceEnvelope,
    events: &[EvidenceValidityEvent],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> bool {
    if start >= end {
        return false;
    }
    // Sample the half-open range at start and at each event/envelope boundary.
    if project_validity(env, events, start).is_some() {
        return true;
    }
    let view = envelope_window(env, events, start);
    let vf = view.0;
    let vu = view.1;
    vf < end && vu.is_none_or(|until| start < until) && env.provenance().collected_at < end
}

fn envelope_window(
    env: &EvidenceEnvelope,
    events: &[EvidenceValidityEvent],
    t: DateTime<Utc>,
) -> (DateTime<Utc>, Option<DateTime<Utc>>) {
    let mut valid_from = env.valid_from();
    let mut valid_until = env.valid_until();
    let mut relevant: Vec<&EvidenceValidityEvent> = events
        .iter()
        .filter(|e| e.envelope_digest == env.digest() && e.at <= t)
        .collect();
    relevant.sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.event_id.cmp(&b.event_id)));
    for event in relevant {
        if matches!(event.kind, EvidenceValidityKind::Asserted) {
            if let Some(from) = event.valid_from {
                valid_from = from;
            }
            valid_until = event.valid_until;
        }
    }
    (valid_from, valid_until)
}

fn select_leaf_as_of(
    candidates: &[EvidenceEnvelope],
    as_of: DateTime<Utc>,
    events: &[EvidenceValidityEvent],
) -> Option<EvidenceEnvelope> {
    let usable: Vec<&EvidenceEnvelope> = candidates
        .iter()
        .filter(|env| project_validity(env, events, as_of).is_some())
        .collect();
    let superseded: std::collections::BTreeSet<&str> = usable
        .iter()
        .filter_map(|e| e.supersedes())
        .filter(|prev| usable.iter().any(|e| e.digest() == *prev))
        .collect();
    let mut leaves: Vec<&EvidenceEnvelope> = usable
        .into_iter()
        .filter(|e| !superseded.contains(e.digest()))
        .collect();
    leaves.sort_by(|a, b| {
        let va = project_validity(a, events, as_of).expect("candidate");
        let vb = project_validity(b, events, as_of).expect("candidate");
        va.observed_at
            .cmp(&vb.observed_at)
            .then_with(|| va.collected_at.cmp(&vb.collected_at))
            .then_with(|| a.digest().cmp(b.digest()))
    });
    leaves.into_iter().next_back().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use weeping_angel_assurance_ir::AssetId;

    use crate::{EvidenceObservation, EvidenceProvenance};

    fn ts(h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, h, 0, 0).unwrap()
    }

    fn seal(salt: &str, collected_at: DateTime<Utc>) -> EvidenceEnvelope {
        EvidenceEnvelope::seal(
            EvidenceObservation::new(EvidenceType::new("identity.privileged.mfa"))
                .with_fact("salt", salt)
                .with_narrative("privileged MFA is enabled"),
            EvidenceProvenance {
                collector_id: "fixture.ledger-crate".into(),
                collected_at,
                scope: "repo:in-scope".into(),
                asset: AssetId::new("repo:in-scope"),
            },
        )
        .expect("seal")
    }

    #[test]
    fn current_is_not_latest_when_newest_is_expired() {
        let mut ledger = EvidenceLedger::open_in_memory().unwrap();
        let older = seal("older", ts(1));
        let newer = seal("newer", ts(12))
            .with_valid_from(ts(12))
            .with_valid_until(ts(13));
        ledger.append(older.clone()).unwrap();
        ledger.append(newer.clone()).unwrap();
        let ty = EvidenceType::new("identity.privileged.mfa");
        let latest = ledger.latest(&ty).unwrap().unwrap();
        assert_eq!(latest.digest(), newer.digest());
        let current = ledger
            .as_of(&ty, Utc.with_ymd_and_hms(2026, 6, 2, 0, 0, 0).unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(current.digest(), older.digest());
        let members = ledger
            .valid_at(&ty, Utc.with_ymd_and_hms(2026, 6, 2, 0, 0, 0).unwrap())
            .unwrap();
        assert!(members.iter().any(|e| e.digest() == older.digest()));
        assert!(!members.iter().any(|e| e.digest() == newer.digest()));
    }

    #[test]
    fn malformed_payload_is_corrupt_not_serialize() {
        let dir = tempfile_dir();
        let path = dir.join("corrupt.sqlite");
        let mut ledger = EvidenceLedger::open(&path).unwrap();
        let env = seal("c", ts(1));
        let digest = env.digest().to_string();
        ledger.append(env).unwrap();
        drop(ledger);
        let conn = rusqlite::Connection::open(&path).unwrap();
        let col = "payload";
        conn.execute(
            &format!("UPDATE evidence_envelopes SET {col} = ?1 WHERE digest = ?2"),
            ["{not-json", digest.as_str()],
        )
        .unwrap();
        drop(conn);
        let ledger = EvidenceLedger::open(&path).unwrap();
        let err = ledger.get(&digest).expect_err("corrupt");
        assert!(
            matches!(err, LedgerError::Path(ref m) if m.contains("corrupt")),
            "{err}"
        );
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "wa-evidence-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
