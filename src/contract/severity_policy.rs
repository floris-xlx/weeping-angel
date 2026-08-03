//! Mechanical impact × likelihood severity matrix (Codex attack-path policy).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Impact {
    High,
    Medium,
    Low,
    Ignore,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Likelihood {
    High,
    Medium,
    Low,
    Ignore,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityLevel {
    Critical,
    High,
    Medium,
    Low,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    Reportable,
    Ignore,
}

impl SeverityLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Ignore => "ignore",
        }
    }

    pub fn priority(self) -> Option<&'static str> {
        match self {
            Self::Critical => Some("P0"),
            Self::High => Some("P1"),
            Self::Medium => Some("P2"),
            Self::Low => Some("P3"),
            Self::Ignore => None,
        }
    }
}

/// Apply the Codex severity calibration matrix.
///
/// `critical` requires both high impact and high likelihood **and** the
/// caller must already have established that critical criteria (immediate
/// demand) are satisfied; this function returns `Critical` only for that
/// impact/likelihood cell when `critical_ok` is true, otherwise `High`.
pub fn apply_severity_matrix(
    impact: Impact,
    likelihood: Likelihood,
    critical_ok: bool,
) -> (SeverityLevel, PolicyDecision) {
    if matches!(impact, Impact::Ignore) || matches!(likelihood, Likelihood::Ignore) {
        return (SeverityLevel::Ignore, PolicyDecision::Ignore);
    }

    let severity = match (impact, likelihood) {
        (Impact::High, Likelihood::High) if critical_ok => SeverityLevel::Critical,
        (Impact::High, Likelihood::High) => SeverityLevel::High,
        (Impact::High, Likelihood::Medium) => SeverityLevel::Medium,
        (Impact::High, Likelihood::Low) => SeverityLevel::Low,
        (Impact::High, Likelihood::Unknown) => SeverityLevel::Medium,

        (Impact::Medium, Likelihood::High) => SeverityLevel::Medium,
        (Impact::Medium, Likelihood::Medium) => SeverityLevel::Low,
        (Impact::Medium, Likelihood::Low) => SeverityLevel::Low,
        (Impact::Medium, Likelihood::Unknown) => SeverityLevel::Low,

        (Impact::Low, _) => SeverityLevel::Low,

        (Impact::Unknown, Likelihood::High) => SeverityLevel::Medium,
        (Impact::Unknown, Likelihood::Medium) => SeverityLevel::Low,
        (Impact::Unknown, Likelihood::Low) => SeverityLevel::Low,
        (Impact::Unknown, Likelihood::Unknown) => SeverityLevel::Low,

        _ => SeverityLevel::Low,
    };

    if severity == SeverityLevel::Ignore {
        (severity, PolicyDecision::Ignore)
    } else {
        (severity, PolicyDecision::Reportable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_critical_requires_flag() {
        let (s, d) = apply_severity_matrix(Impact::High, Likelihood::High, true);
        assert_eq!(s, SeverityLevel::Critical);
        assert_eq!(d, PolicyDecision::Reportable);

        let (s, d) = apply_severity_matrix(Impact::High, Likelihood::High, false);
        assert_eq!(s, SeverityLevel::High);
        assert_eq!(d, PolicyDecision::Reportable);
    }

    #[test]
    fn ignore_short_circuits() {
        let (s, d) = apply_severity_matrix(Impact::High, Likelihood::Ignore, true);
        assert_eq!(s, SeverityLevel::Ignore);
        assert_eq!(d, PolicyDecision::Ignore);
    }
}
