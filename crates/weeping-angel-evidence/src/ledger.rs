//! Persistent immutable evidence ledger. Owns observations, never conclusions.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::validity::{EvidenceValidityEvent, EvidenceValidityKind, project_validity};
use crate::{CollectionRun, EvidenceEnvelope, EvidenceType};

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
        let inserted = self.conn.execute(
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
                serde_json::to_string(&envelope)?,
            ],
        )?;
        if inserted == 1 {
            let event = EvidenceValidityEvent::asserted_for(&envelope)
                .map_err(|e| LedgerError::Path(e.to_string()))?;
            self.record_validity_event(event)?;
        }
        Ok(inserted == 1)
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
        Ok(serde_json::from_str(&payload)?)
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
        payload
            .map(|p| serde_json::from_str(&p).map_err(LedgerError::from))
            .transpose()
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
        let payload = serde_json::to_string(&event)?;
        let existing: Option<String> = self
            .conn
            .query_row(
                "SELECT payload FROM evidence_validity_events WHERE event_id = ?1",
                [&event.event_id],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(existing) = existing {
            if existing == payload {
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
        self.conn.execute(
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
        Ok(true)
    }

    pub fn validity_events(&self) -> Result<Vec<EvidenceValidityEvent>, LedgerError> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM evidence_validity_events ORDER BY at, event_id")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(serde_json::from_str(&row?)?);
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

    /// Latest usable envelope of `evidence_type` at `as_of` (supersession + validity).
    pub fn latest_as_of(
        &self,
        evidence_type: &EvidenceType,
        as_of: DateTime<Utc>,
    ) -> Result<Option<EvidenceEnvelope>, LedgerError> {
        let events = self.validity_events()?;
        let mut candidates = Vec::new();
        for env in self.for_type(evidence_type)? {
            if project_validity(&env, &events, as_of).is_some() {
                candidates.push(env);
            }
        }
        Ok(select_leaf_as_of(&candidates, as_of, &events))
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
            out.push(serde_json::from_str(&row?)?);
        }
        Ok(out)
    }

    pub fn record_collection_run(&mut self, run: &CollectionRun) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO collection_runs (run_id, payload) VALUES (?1, ?2)",
            params![run.run_id, serde_json::to_string(run)?],
        )?;
        Ok(())
    }

    pub fn persist_assessment_run(&mut self, id: &str, payload: &str) -> Result<bool, LedgerError> {
        persist_immutable(&self.conn, "assessment_runs", "id", id, payload)
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
            out.push(serde_json::from_str(&row?)?);
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
            out.push(serde_json::from_str(&row?)?);
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
