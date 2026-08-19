//! Exact decimal text for IR amounts. Grammar matches evidence `DecimalText`.
//! Lexical identity for authored values; numeric compare / multiply never use binary floats.

use std::cmp::Ordering;
use std::fmt;

use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CanonicalDecimalError {
    #[error("invalid decimal: {0}")]
    Invalid(String),
}

/// Validated decimal text. Authored identity is lexical (`1.0` ≠ `1.00`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalDecimal(String);

impl CanonicalDecimal {
    pub fn parse(text: impl Into<String>) -> Result<Self, CanonicalDecimalError> {
        let text = text.into();
        if !is_canonical_decimal(&text) {
            return Err(CanonicalDecimalError::Invalid(text));
        }
        Ok(Self(text))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn cmp_numeric(&self, other: &Self) -> Ordering {
        cmp_decimal_text(&self.0, &other.0)
    }

    /// Exact decimal product, then canonicalize (no trailing fractional zeros).
    pub fn times(&self, other: &Self) -> Self {
        let a = math_parts(&self.0);
        let b = math_parts(&other.0);
        let digits = mul_digit_strings(&a.digits, &b.digits);
        let scale = a.scale + b.scale;
        let neg = a.neg != b.neg && digits != "0";
        Self(format_canonical(neg, &digits, scale))
    }
}

impl fmt::Display for CanonicalDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for CanonicalDecimal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CanonicalDecimal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
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

struct MathParts {
    neg: bool,
    digits: String,
    scale: usize,
}

fn math_parts(text: &str) -> MathParts {
    let (neg, rest) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let (int, frac) = match rest.split_once('.') {
        Some((int, frac)) => (int, frac),
        None => (rest, ""),
    };
    let mut digits = String::new();
    digits.push_str(int.trim_start_matches('0'));
    digits.push_str(frac);
    let digits = if digits.trim_start_matches('0').is_empty() {
        "0".to_string()
    } else {
        digits.trim_start_matches('0').to_string()
    };
    MathParts {
        neg: neg && digits != "0",
        digits,
        scale: frac.len(),
    }
}

fn mul_digit_strings(left: &str, right: &str) -> String {
    if left == "0" || right == "0" {
        return "0".into();
    }
    let a: Vec<u8> = left.bytes().map(|b| b - b'0').collect();
    let b: Vec<u8> = right.bytes().map(|b| b - b'0').collect();
    let mut acc = vec![0u32; a.len() + b.len()];
    for (i, da) in a.iter().rev().enumerate() {
        for (j, db) in b.iter().rev().enumerate() {
            acc[i + j] += u32::from(*da) * u32::from(*db);
        }
    }
    let mut carry = 0u32;
    for slot in &mut acc {
        let total = *slot + carry;
        *slot = total % 10;
        carry = total / 10;
    }
    while carry > 0 {
        acc.push(carry % 10);
        carry /= 10;
    }
    while acc.last() == Some(&0) {
        acc.pop();
    }
    if acc.is_empty() {
        return "0".into();
    }
    acc.into_iter()
        .rev()
        .map(|d| char::from(b'0' + d as u8))
        .collect()
}

fn format_canonical(neg: bool, digits: &str, scale: usize) -> String {
    let digits = digits.trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    if digits == "0" {
        return "0".into();
    }
    let body = if scale == 0 {
        digits.to_string()
    } else if scale >= digits.len() {
        let pad = scale - digits.len();
        let mut frac = "0".repeat(pad);
        frac.push_str(digits);
        let frac = frac.trim_end_matches('0');
        if frac.is_empty() {
            "0".to_string()
        } else {
            format!("0.{frac}")
        }
    } else {
        let split = digits.len() - scale;
        let (int, frac) = digits.split_at(split);
        let frac = frac.trim_end_matches('0');
        if frac.is_empty() {
            int.to_string()
        } else {
            format!("{int}.{frac}")
        }
    };
    if neg && body != "0" {
        format!("-{body}")
    } else {
        body
    }
}
