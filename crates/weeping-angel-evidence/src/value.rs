//! Typed evidence values and hybrid `evidence-value/v1` canonical JSON.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, Datelike, FixedOffset, Timelike, Utc};
use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Nested fact encoding used inside `evidence/v1` observation facts.
pub const EVIDENCE_VALUE_SCHEMA: &str = "evidence-value/v1";

/// Reserved object key that distinguishes tagged scalars from nested objects.
pub const EVIDENCE_VALUE_TAG: &str = "$evidenceValue";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EvidenceValueError {
    #[error("invalid decimal text: {0}")]
    InvalidDecimal(String),
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("invalid evidence-value/v1 encoding: {0}")]
    InvalidEncoding(String),
}

/// Validated decimal *text*. Identity is lexical (`1.0` ≠ `1.00`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DecimalText(String);

impl DecimalText {
    pub fn parse(text: impl Into<String>) -> Result<Self, EvidenceValueError> {
        let text = text.into();
        if !is_canonical_decimal(&text) {
            return Err(EvidenceValueError::InvalidDecimal(text));
        }
        Ok(Self(text))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DecimalText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One typed evidence value. No `f64` / `f32`. Absence is a missing key, not `Null`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EvidenceValue {
    String(String),
    Bool(bool),
    Integer(i64),
    Decimal(DecimalText),
    Timestamp(DateTime<Utc>),
    DurationSeconds(u64),
    StringList(Vec<String>),
    Object(BTreeMap<String, EvidenceValue>),
}

impl EvidenceValue {
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn from_bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn integer(value: i64) -> Self {
        Self::Integer(value)
    }

    pub fn decimal(text: impl Into<String>) -> Result<Self, EvidenceValueError> {
        Ok(Self::Decimal(DecimalText::parse(text)?))
    }

    pub fn timestamp(dt: DateTime<Utc>) -> Result<Self, EvidenceValueError> {
        if !dt.timestamp_subsec_nanos().is_multiple_of(1_000_000) {
            return Err(EvidenceValueError::InvalidTimestamp(
                "sub-millisecond remainder is not allowed".into(),
            ));
        }
        Ok(Self::Timestamp(dt))
    }

    pub fn timestamp_rfc3339(raw: &str) -> Result<Self, EvidenceValueError> {
        let parsed = DateTime::parse_from_rfc3339(raw)
            .map_err(|e| EvidenceValueError::InvalidTimestamp(format!("{raw}: {e}")))?;
        Self::from_fixed_offset(parsed)
    }

    fn from_fixed_offset(parsed: DateTime<FixedOffset>) -> Result<Self, EvidenceValueError> {
        let utc = parsed.with_timezone(&Utc);
        Self::timestamp(utc)
    }

    pub fn duration_seconds(seconds: u64) -> Self {
        Self::DurationSeconds(seconds)
    }

    pub fn string_list(items: Vec<String>) -> Self {
        Self::StringList(items)
    }

    pub fn object(map: BTreeMap<String, EvidenceValue>) -> Self {
        Self::Object(map)
    }

    /// String-only accessor. Does not stringify other variants.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "String",
            Self::Bool(_) => "Bool",
            Self::Integer(_) => "Integer",
            Self::Decimal(_) => "Decimal",
            Self::Timestamp(_) => "Timestamp",
            Self::DurationSeconds(_) => "DurationSeconds",
            Self::StringList(_) => "StringList",
            Self::Object(_) => "Object",
        }
    }

    pub fn typed_eq(&self, expected: &Self) -> Result<bool, String> {
        if std::mem::discriminant(self) != std::mem::discriminant(expected) {
            return Err(type_mismatch(expected.type_name(), self.type_name()));
        }
        Ok(self == expected)
    }

    /// Numeric / temporal order. Integer↔Decimal is exact decimal compare, never `f64`.
    pub fn cmp_numeric(&self, expected: &Self) -> Result<Ordering, String> {
        match (self, expected) {
            (Self::Integer(a), Self::Integer(b)) => Ok(a.cmp(b)),
            (Self::Decimal(a), Self::Decimal(b)) => Ok(cmp_decimal_text(a.as_str(), b.as_str())),
            (Self::Integer(a), Self::Decimal(b)) => {
                Ok(cmp_decimal_text(&a.to_string(), b.as_str()))
            }
            (Self::Decimal(a), Self::Integer(b)) => {
                Ok(cmp_decimal_text(a.as_str(), &b.to_string()))
            }
            (Self::Timestamp(a), Self::Timestamp(b)) => Ok(a.cmp(b)),
            (Self::DurationSeconds(a), Self::DurationSeconds(b)) => Ok(a.cmp(b)),
            (have, want) => Err(type_mismatch(want.type_name(), have.type_name())),
        }
    }

    pub fn list_contains(&self, expected: &Self) -> Result<bool, String> {
        let Self::StringList(items) = self else {
            return Err(type_mismatch("StringList", self.type_name()));
        };
        let Self::String(needle) = expected else {
            return Err(type_mismatch("String", expected.type_name()));
        };
        Ok(items.iter().any(|item| item == needle))
    }

    pub fn canonical_timestamp_string(dt: DateTime<Utc>) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second(),
            dt.timestamp_subsec_millis()
        )
    }
}

fn type_mismatch(expected: &str, got: &str) -> String {
    format!("type mismatch: expected {expected}, got {got}")
}

fn is_canonical_decimal(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    if bytes[0] == b'-' {
        i = 1;
        if i >= bytes.len() {
            return false;
        }
    }
    if bytes[i] == b'0' {
        i += 1;
    } else if bytes[i].is_ascii_digit() {
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    } else {
        return false;
    }
    if i < bytes.len() {
        if bytes[i] != b'.' {
            return false;
        }
        i += 1;
        let frac_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == frac_start {
            return false;
        }
    }
    i == bytes.len()
}

struct DecimalParts<'a> {
    neg: bool,
    int: &'a str,
    frac: &'a str,
}

fn decimal_parts(text: &str) -> DecimalParts<'_> {
    let (neg, rest) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let (int, frac) = match rest.split_once('.') {
        Some((int, frac)) => (int, frac),
        None => (rest, ""),
    };
    let int = int.trim_start_matches('0');
    DecimalParts {
        neg,
        int: if int.is_empty() { "0" } else { int },
        frac,
    }
}

fn decimal_is_zero(parts: &DecimalParts<'_>) -> bool {
    parts.int == "0" && parts.frac.bytes().all(|b| b == b'0')
}

fn cmp_decimal_text(left: &str, right: &str) -> Ordering {
    let a = decimal_parts(left);
    let b = decimal_parts(right);
    let a_zero = decimal_is_zero(&a);
    let b_zero = decimal_is_zero(&b);
    if a_zero && b_zero {
        return Ordering::Equal;
    }
    if a.neg != b.neg {
        return if a.neg {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    let mag = match a.int.len().cmp(&b.int.len()) {
        Ordering::Equal => match a.int.cmp(b.int) {
            Ordering::Equal => {
                let width = a.frac.len().max(b.frac.len());
                let mut fa = a.frac.to_string();
                let mut fb = b.frac.to_string();
                fa.extend(std::iter::repeat_n('0', width - a.frac.len()));
                fb.extend(std::iter::repeat_n('0', width - b.frac.len()));
                fa.cmp(&fb)
            }
            other => other,
        },
        other => other,
    };
    if a.neg { mag.reverse() } else { mag }
}

#[derive(Serialize)]
struct TaggedText<'a> {
    #[serde(rename = "$evidenceValue")]
    kind: &'static str,
    value: &'a str,
}

#[derive(Serialize)]
struct TaggedDuration {
    #[serde(rename = "$evidenceValue")]
    kind: &'static str,
    value: u64,
}

impl Serialize for EvidenceValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::String(s) => s.serialize(serializer),
            Self::Bool(b) => b.serialize(serializer),
            Self::Integer(n) => n.serialize(serializer),
            Self::StringList(items) => items.serialize(serializer),
            Self::Object(map) => map.serialize(serializer),
            Self::Decimal(text) => TaggedText {
                kind: "decimal",
                value: text.as_str(),
            }
            .serialize(serializer),
            Self::Timestamp(dt) => {
                let encoded = Self::canonical_timestamp_string(*dt);
                TaggedText {
                    kind: "timestamp",
                    value: &encoded,
                }
                .serialize(serializer)
            }
            Self::DurationSeconds(seconds) => TaggedDuration {
                kind: "durationSeconds",
                value: *seconds,
            }
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for EvidenceValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        from_json(value).map_err(de::Error::custom)
    }
}

fn from_json(value: serde_json::Value) -> Result<EvidenceValue, EvidenceValueError> {
    match value {
        serde_json::Value::String(s) => Ok(EvidenceValue::String(s)),
        serde_json::Value::Bool(b) => Ok(EvidenceValue::Bool(b)),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                Ok(EvidenceValue::Integer(n.as_i64().expect("is_i64")))
            } else {
                Err(EvidenceValueError::InvalidEncoding(format!(
                    "JSON number must be an i64 integer without fraction, got {n}"
                )))
            }
        }
        serde_json::Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    serde_json::Value::String(s) => out.push(s),
                    other => {
                        return Err(EvidenceValueError::InvalidEncoding(format!(
                            "StringList elements must be strings, got {other}"
                        )));
                    }
                }
            }
            Ok(EvidenceValue::StringList(out))
        }
        serde_json::Value::Object(map) => {
            if let Some(tag) = map.get(EVIDENCE_VALUE_TAG) {
                decode_tagged(tag, &map)
            } else {
                let mut out = BTreeMap::new();
                for (key, nested) in map {
                    out.insert(key, from_json(nested)?);
                }
                Ok(EvidenceValue::Object(out))
            }
        }
        serde_json::Value::Null => Err(EvidenceValueError::InvalidEncoding(
            "JSON null is not an evidence value".into(),
        )),
    }
}

fn decode_tagged(
    tag: &serde_json::Value,
    map: &serde_json::Map<String, serde_json::Value>,
) -> Result<EvidenceValue, EvidenceValueError> {
    if map.len() != 2 || !map.contains_key("value") {
        return Err(EvidenceValueError::InvalidEncoding(
            "tagged evidence-value/v1 object must have exactly $evidenceValue and value".into(),
        ));
    }
    let tag = tag.as_str().ok_or_else(|| {
        EvidenceValueError::InvalidEncoding("$evidenceValue tag must be a string".into())
    })?;
    let payload = map.get("value").expect("value key");
    match tag {
        "decimal" => {
            let text = payload.as_str().ok_or_else(|| {
                EvidenceValueError::InvalidEncoding("decimal value must be a string".into())
            })?;
            EvidenceValue::decimal(text)
        }
        "timestamp" => {
            let text = payload.as_str().ok_or_else(|| {
                EvidenceValueError::InvalidEncoding("timestamp value must be a string".into())
            })?;
            EvidenceValue::timestamp_rfc3339(text)
        }
        "durationSeconds" => {
            let seconds = payload.as_u64().ok_or_else(|| {
                EvidenceValueError::InvalidEncoding(
                    "durationSeconds value must be a u64 number".into(),
                )
            })?;
            Ok(EvidenceValue::DurationSeconds(seconds))
        }
        other => Err(EvidenceValueError::InvalidEncoding(format!(
            "unknown $evidenceValue tag: {other}"
        ))),
    }
}
