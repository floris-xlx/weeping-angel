//! Persistent immutable evidence ledger. Owns observations, never conclusions.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

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
        Ok(next)
    }

    pub fn record_collection_run(&mut self, run: &CollectionRun) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO collection_runs (run_id, payload) VALUES (?1, ?2)",
            params![run.run_id, serde_json::to_string(run)?],
        )?;
        Ok(())
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
