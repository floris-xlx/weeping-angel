//! Target suite for Operational ISMS v1 risk methodology — risk methodology IR.
//!
//! Encodes DESIRED behavior in `docs/specs/risk-methodology.md` §4 / §6.2
//! (P05-T01–T17). Must stay RED on CURRENT HEAD because scoring types and
//! `score_risk` do not exist. Do not weaken these assertions to match today's
//! four-field `Risk` record, and do not implement the feature in this suite.

use std::fs;
use std::path::{Path, PathBuf};

use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AcceptanceThreshold, CanonicalDecimal, Combination, DerivedRating,
    IdError, ImpactScale, LikelihoodScale, MatrixCell, NumericDomain, RatingBand, RatingLevel,
    RatingScale, Risk, RiskAppetite, RiskId, RiskMatrix, RiskMethodology, RiskMethodologyError,
    RiskMethodologyId, RiskScore, RiskScoreInput, RiskStatus, RiskTolerance, ScaleLevel,
    ScoredRisk, ScoringMode, StableId, canonical_digest, score_risk, validate_risk_methodology,
};
use weeping_angel_evidence::EvidenceValue;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn ir_fixture(name: &str) -> PathBuf {
    manifest_dir()
        .join("tests/fixtures/assurance-ir/v1")
        .join(name)
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
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

fn product_crate_sources_joined() -> String {
    let crates_dir = manifest_dir().join("crates");
    let mut chunks = Vec::new();
    for entry in fs::read_dir(&crates_dir).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let src = entry.path().join("src");
        if !src.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        walk_rs_files(&src, &mut files);
        for path in files {
            chunks.push(fs::read_to_string(&path).unwrap());
        }
    }
    chunks.join("\n")
}

fn dec(text: &str) -> CanonicalDecimal {
    CanonicalDecimal::parse(text).unwrap_or_else(|e| panic!("canonical decimal `{text}`: {e}"))
}

fn level(id: &str, label: &str, ordinal: u32) -> ScaleLevel {
    ScaleLevel {
        id: id.to_string(),
        label: label.to_string(),
        ordinal,
        description: None,
    }
}

fn rating(id: &str, label: &str, ordinal: u32) -> RatingLevel {
    RatingLevel {
        id: id.to_string(),
        label: label.to_string(),
        ordinal,
        description: None,
    }
}

fn lmh_levels() -> Vec<ScaleLevel> {
    vec![
        level("low", "Low", 1),
        level("medium", "Medium", 2),
        level("high", "High", 3),
    ]
}

fn lmh_ratings() -> RatingScale {
    RatingScale {
        levels: vec![
            rating("low", "Low", 1),
            rating("medium", "Medium", 2),
            rating("high", "High", 3),
        ],
    }
}

fn cell(likelihood_ordinal: u32, impact_ordinal: u32, rating_id: &str) -> MatrixCell {
    MatrixCell {
        likelihood_ordinal,
        impact_ordinal,
        rating_id: rating_id.to_string(),
    }
}

fn cells_3x3() -> Vec<MatrixCell> {
    vec![
        cell(1, 1, "low"),
        cell(1, 2, "low"),
        cell(1, 3, "medium"),
        cell(2, 1, "low"),
        cell(2, 2, "medium"),
        cell(2, 3, "high"),
        cell(3, 1, "medium"),
        cell(3, 2, "high"),
        cell(3, 3, "high"),
    ]
}

fn dummy_scale() -> Vec<ScaleLevel> {
    vec![level("n-a", "n/a", 1)]
}

fn adopt(draft: RiskMethodology) -> RiskMethodology {
    RiskMethodology::try_new(draft).expect("methodology must validate")
}

fn qualitative_3x3() -> RiskMethodology {
    qualitative_3x3_with_cells(cells_3x3())
}

fn qualitative_3x3_with_cells(cells: Vec<MatrixCell>) -> RiskMethodology {
    adopt(RiskMethodology {
        schema_version: ASSURANCE_IR_SCHEMA.to_string(),
        id: RiskMethodologyId::new("rm:acme-qual-3x3:1"),
        logical_id: "rm:acme-qual-3x3".into(),
        revision: 1,
        title: "ACME qualitative 3x3".into(),
        scoring_mode: ScoringMode::Qualitative,
        likelihood_scale: LikelihoodScale {
            id: "likelihood.lmh".into(),
            levels: lmh_levels(),
        },
        impact_scale: ImpactScale {
            id: "impact.lmh".into(),
            levels: lmh_levels(),
        },
        rating_scale: lmh_ratings(),
        combination: Combination::Matrix,
        matrix: Some(RiskMatrix { cells }),
        domain: None,
        bands: None,
        currency: None,
        appetite: RiskAppetite {
            max_rating_id: "medium".into(),
        },
        tolerance: RiskTolerance {
            max_rating_id: "high".into(),
        },
        acceptance_threshold: AcceptanceThreshold {
            max_rating_id: "medium".into(),
        },
        supersedes: None,
        locked: false,
    })
}

fn rating_for_5x5_product(likelihood: u32, impact: u32) -> &'static str {
    let product = likelihood * impact;
    if product >= 15 {
        "critical"
    } else if product >= 8 {
        "high"
    } else if product >= 4 {
        "medium"
    } else {
        "low"
    }
}

fn cells_5x5() -> Vec<MatrixCell> {
    let mut cells = Vec::with_capacity(25);
    for likelihood in 1..=5 {
        for impact in 1..=5 {
            cells.push(cell(
                likelihood,
                impact,
                rating_for_5x5_product(likelihood, impact),
            ));
        }
    }
    cells
}

fn semi_quantitative_5x5() -> RiskMethodology {
    let likelihood_labels = ["Rare", "Unlikely", "Possible", "Likely", "Almost certain"];
    let impact_labels = [
        "Insignificant",
        "Minor",
        "Moderate",
        "Major",
        "Catastrophic",
    ];
    adopt(RiskMethodology {
        schema_version: ASSURANCE_IR_SCHEMA.to_string(),
        id: RiskMethodologyId::new("rm:acme-semi-5x5:1"),
        logical_id: "rm:acme-semi-5x5".into(),
        revision: 1,
        title: "ACME semi-quantitative 5x5".into(),
        scoring_mode: ScoringMode::SemiQuantitative,
        likelihood_scale: LikelihoodScale {
            id: "likelihood.1-5".into(),
            levels: (1..=5)
                .map(|n| level(&n.to_string(), likelihood_labels[(n - 1) as usize], n))
                .collect(),
        },
        impact_scale: ImpactScale {
            id: "impact.1-5".into(),
            levels: (1..=5)
                .map(|n| level(&n.to_string(), impact_labels[(n - 1) as usize], n))
                .collect(),
        },
        rating_scale: RatingScale {
            levels: vec![
                rating("low", "Low", 1),
                rating("medium", "Medium", 2),
                rating("high", "High", 3),
                rating("critical", "Critical", 4),
            ],
        },
        combination: Combination::Matrix,
        matrix: Some(RiskMatrix { cells: cells_5x5() }),
        domain: None,
        bands: None,
        currency: None,
        appetite: RiskAppetite {
            max_rating_id: "medium".into(),
        },
        tolerance: RiskTolerance {
            max_rating_id: "critical".into(),
        },
        acceptance_threshold: AcceptanceThreshold {
            max_rating_id: "medium".into(),
        },
        supersedes: None,
        locked: false,
    })
}

fn expected_loss_bands() -> Vec<RatingBand> {
    vec![
        RatingBand {
            rating_id: "low".into(),
            min_inclusive: dec("0"),
            max_exclusive: Some(dec("1000")),
        },
        RatingBand {
            rating_id: "medium".into(),
            min_inclusive: dec("1000"),
            max_exclusive: Some(dec("10000")),
        },
        RatingBand {
            rating_id: "high".into(),
            min_inclusive: dec("10000"),
            max_exclusive: None,
        },
    ]
}

fn quantitative_expected_loss() -> RiskMethodology {
    adopt(RiskMethodology {
        schema_version: ASSURANCE_IR_SCHEMA.to_string(),
        id: RiskMethodologyId::new("rm:acme-expected-loss:1"),
        logical_id: "rm:acme-expected-loss".into(),
        revision: 1,
        title: "ACME expected-loss bands".into(),
        scoring_mode: ScoringMode::Quantitative,
        likelihood_scale: LikelihoodScale {
            id: "likelihood.unused".into(),
            levels: dummy_scale(),
        },
        impact_scale: ImpactScale {
            id: "impact.unused".into(),
            levels: dummy_scale(),
        },
        rating_scale: lmh_ratings(),
        combination: Combination::ExpectedLoss,
        matrix: None,
        domain: Some(NumericDomain {
            min: dec("0"),
            max: dec("1000000"),
        }),
        bands: Some(expected_loss_bands()),
        currency: Some("EUR".into()),
        appetite: RiskAppetite {
            max_rating_id: "medium".into(),
        },
        tolerance: RiskTolerance {
            max_rating_id: "high".into(),
        },
        acceptance_threshold: AcceptanceThreshold {
            max_rating_id: "medium".into(),
        },
        supersedes: None,
        locked: false,
    })
}

fn custom_bounded_1_3() -> RiskMethodology {
    adopt(RiskMethodology {
        schema_version: ASSURANCE_IR_SCHEMA.to_string(),
        id: RiskMethodologyId::new("rm:acme-bounded-1-3:1"),
        logical_id: "rm:acme-bounded-1-3".into(),
        revision: 1,
        title: "ACME custom bounded 1-3 product domain".into(),
        scoring_mode: ScoringMode::CustomBounded,
        likelihood_scale: LikelihoodScale {
            id: "likelihood.1-3".into(),
            levels: (1..=3)
                .map(|n| level(&n.to_string(), &n.to_string(), n))
                .collect(),
        },
        impact_scale: ImpactScale {
            id: "impact.1-3".into(),
            levels: (1..=3)
                .map(|n| level(&n.to_string(), &n.to_string(), n))
                .collect(),
        },
        rating_scale: lmh_ratings(),
        combination: Combination::Identity,
        matrix: None,
        domain: Some(NumericDomain {
            min: dec("1"),
            max: dec("9"),
        }),
        bands: Some(vec![
            RatingBand {
                rating_id: "low".into(),
                min_inclusive: dec("1"),
                max_exclusive: Some(dec("4")),
            },
            RatingBand {
                rating_id: "medium".into(),
                min_inclusive: dec("4"),
                max_exclusive: Some(dec("7")),
            },
            RatingBand {
                rating_id: "high".into(),
                min_inclusive: dec("7"),
                max_exclusive: None,
            },
        ]),
        currency: None,
        appetite: RiskAppetite {
            max_rating_id: "medium".into(),
        },
        tolerance: RiskTolerance {
            max_rating_id: "high".into(),
        },
        acceptance_threshold: AcceptanceThreshold {
            max_rating_id: "medium".into(),
        },
        supersedes: None,
        locked: false,
    })
}

fn custom_bounded_1_5() -> RiskMethodology {
    adopt(RiskMethodology {
        schema_version: ASSURANCE_IR_SCHEMA.to_string(),
        id: RiskMethodologyId::new("rm:acme-bounded-1-5:1"),
        logical_id: "rm:acme-bounded-1-5".into(),
        revision: 1,
        title: "ACME custom bounded 1-5".into(),
        scoring_mode: ScoringMode::CustomBounded,
        likelihood_scale: LikelihoodScale {
            id: "likelihood.1-5".into(),
            levels: (1..=5)
                .map(|n| level(&n.to_string(), &n.to_string(), n))
                .collect(),
        },
        impact_scale: ImpactScale {
            id: "impact.1-5".into(),
            levels: (1..=5)
                .map(|n| level(&n.to_string(), &n.to_string(), n))
                .collect(),
        },
        rating_scale: lmh_ratings(),
        combination: Combination::Identity,
        matrix: None,
        domain: Some(NumericDomain {
            min: dec("1"),
            max: dec("5"),
        }),
        bands: Some(vec![
            RatingBand {
                rating_id: "low".into(),
                min_inclusive: dec("1"),
                max_exclusive: Some(dec("3")),
            },
            RatingBand {
                rating_id: "medium".into(),
                min_inclusive: dec("3"),
                max_exclusive: Some(dec("4")),
            },
            RatingBand {
                rating_id: "high".into(),
                min_inclusive: dec("4"),
                max_exclusive: None,
            },
        ]),
        currency: None,
        appetite: RiskAppetite {
            max_rating_id: "medium".into(),
        },
        tolerance: RiskTolerance {
            max_rating_id: "high".into(),
        },
        acceptance_threshold: AcceptanceThreshold {
            max_rating_id: "medium".into(),
        },
        supersedes: None,
        locked: false,
    })
}

fn load_methodology_fixture(name: &str) -> RiskMethodology {
    let path = ir_fixture(name);
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("methodology fixture {} must exist: {e}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("decode methodology fixture {name}: {e}"))
}

fn err_text(err: &RiskMethodologyError) -> String {
    err.to_string().to_ascii_lowercase()
}

fn assert_err_contains(err: &RiskMethodologyError, needle: &str) {
    let msg = err_text(err);
    assert!(
        msg.contains(needle),
        "expected error to contain `{needle}`, got `{err}`"
    );
}

fn assert_derived_rating(scored: &ScoredRisk, methodology: &RiskMethodology, rating_id: &str) {
    assert_eq!(scored.rating.rating_id, rating_id);
    assert_eq!(scored.rating.revision, methodology.revision);
    assert_eq!(
        scored.rating.methodology_id.as_str(),
        methodology.id.as_str()
    );
}

/// P05: 3x3 qualitative fixture scores derived ratings
#[test]
fn p05_t01_3x3_qualitative_fixture_scores_derived_ratings() {
    let methodology = qualitative_3x3();
    validate_risk_methodology(&methodology).expect("3x3 methodology must validate");

    let medium_high = RiskScoreInput::Qualitative {
        likelihood_id: "medium".into(),
        impact_id: "high".into(),
    };
    let scored = score_risk(&methodology, &medium_high).expect("medium×high must score");
    assert_eq!(scored.input, medium_high);
    assert_derived_rating(&scored, &methodology, "high");
    match &scored.score {
        RiskScore::Qualitative {
            likelihood_ordinal,
            impact_ordinal,
        } => {
            assert_eq!(*likelihood_ordinal, 2);
            assert_eq!(*impact_ordinal, 3);
        }
        other => panic!("3x3 qualitative score must retain ordinals, got {other:?}"),
    }

    let low_high = RiskScoreInput::Qualitative {
        likelihood_id: "low".into(),
        impact_id: "high".into(),
    };
    let scored = score_risk(&methodology, &low_high).expect("low×high must score");
    assert_derived_rating(&scored, &methodology, "medium");

    let fixture = load_methodology_fixture("risk-methodology-3x3.json");
    validate_risk_methodology(&fixture).expect("golden 3x3 fixture must validate");
    let from_fixture = score_risk(&fixture, &medium_high).expect("fixture medium×high");
    assert_derived_rating(&from_fixture, &fixture, "high");

    let digest_a = canonical_digest(&methodology).expect("digest 3x3");
    let digest_b = canonical_digest(&qualitative_3x3()).expect("digest 3x3 again");
    assert_eq!(digest_a, digest_b, "3x3 digest must be stable");
}

/// P05: 5x5 semi-quantitative fixture is data not a compiler constant
#[test]
fn p05_t02_5x5_semi_quantitative_fixture_is_data_not_a_compiler_constant() {
    let methodology = semi_quantitative_5x5();
    let cells = methodology
        .matrix
        .as_ref()
        .expect("5x5 combination is matrix")
        .cells
        .len();
    assert_eq!(cells, 25, "5x5 fixture must materialize 25 data cells");

    let input = RiskScoreInput::SemiQuantitative {
        likelihood: 5,
        impact: 5,
    };
    let scored = score_risk(&methodology, &input).expect("5×5 must score");
    assert_derived_rating(&scored, &methodology, "critical");

    let fixture = load_methodology_fixture("risk-methodology-5x5.json");
    assert_eq!(
        fixture
            .matrix
            .as_ref()
            .expect("golden 5x5 is a matrix")
            .cells
            .len(),
        25
    );
    let from_fixture = score_risk(&fixture, &input).expect("fixture 5×5");
    assert_eq!(from_fixture.rating.rating_id, "critical");

    let scoring_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk_methodology.rs");
    assert!(
        !scoring_src.contains("LIKELIHOOD_MAX") && !scoring_src.contains("LIKELIHOOD_MAX = 5"),
        "scoring source must not hardcode LIKELIHOOD_MAX=5"
    );
    let product = product_crate_sources_joined();
    assert!(
        !product.contains("const LIKELIHOOD_MAX: u32 = 5"),
        "product scoring/control paths must not hardcode a 5-wide likelihood"
    );
}

/// P05: custom quantitative thresholds and expected loss
#[test]
fn p05_t03_custom_quantitative_thresholds_and_expected_loss() {
    let methodology = quantitative_expected_loss();

    let half_of_2000 = RiskScoreInput::Quantitative {
        probability: dec("0.5"),
        loss: dec("2000"),
    };
    let scored = score_risk(&methodology, &half_of_2000).expect("0.5×2000 must score");
    match &scored.score {
        RiskScore::Quantitative { expected_loss } => {
            assert_eq!(
                expected_loss.as_str(),
                "1000",
                "0.5×2000 must canonicalize to 1000"
            );
        }
        other => panic!("expected quantitative score, got {other:?}"),
    }
    assert_derived_rating(&scored, &methodology, "medium");

    let exact = RiskScoreInput::Quantitative {
        probability: dec("0.1"),
        loss: dec("0.2"),
    };
    let scored = score_risk(&methodology, &exact).expect("0.1×0.2 must score");
    match &scored.score {
        RiskScore::Quantitative { expected_loss } => {
            assert_eq!(
                expected_loss.as_str(),
                "0.02",
                "0.1×0.2 must be exact CanonicalDecimal, not f64"
            );
        }
        other => panic!("expected quantitative score, got {other:?}"),
    }

    let scoring_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk_methodology.rs");
    assert!(
        !scoring_src.contains("f64") && !scoring_src.contains("f32"),
        "methodology scoring must not use IEEE-754 floats"
    );

    let fixture = load_methodology_fixture("risk-methodology-expected-loss.json");
    let from_fixture = score_risk(&fixture, &half_of_2000).expect("fixture expected loss");
    assert_eq!(from_fixture.rating.rating_id, "medium");
}

/// P05: custom bounded 1-3 domain
#[test]
fn p05_t04_custom_bounded_1_3_domain() {
    let methodology = custom_bounded_1_3();

    let nine = RiskScoreInput::CustomBounded { value: dec("9") };
    let scored = score_risk(&methodology, &nine).expect("value 9 must score");
    assert_derived_rating(&scored, &methodology, "high");

    let three = RiskScoreInput::CustomBounded { value: dec("3") };
    let scored = score_risk(&methodology, &three).expect("value 3 must score");
    assert_derived_rating(&scored, &methodology, "low");

    let ten = RiskScoreInput::CustomBounded { value: dec("10") };
    let err = score_risk(&methodology, &ten).expect_err("value 10 is outside [1, 9]");
    assert_err_contains(&err, "out of domain");
}

/// P05: malformed matrix rejected
#[test]
fn p05_t05_malformed_matrix_rejected() {
    let mut missing = cells_3x3();
    missing.pop();
    let err = RiskMethodology::try_new(RiskMethodology {
        matrix: Some(RiskMatrix { cells: missing }),
        ..qualitative_3x3()
    })
    .expect_err("missing cell must fail closed");
    let missing_msg = err_text(&err);
    assert!(
        missing_msg.contains("malformed matrix") || missing_msg.contains("missing cell"),
        "missing cell class, got `{err}`"
    );
    assert!(
        validate_risk_methodology(&RiskMethodology {
            matrix: Some(RiskMatrix {
                cells: cells_3x3().into_iter().take(8).collect(),
            }),
            ..qualitative_3x3()
        })
        .is_err(),
        "validate_risk_methodology must reject a missing cell"
    );

    let mut extra = cells_3x3();
    extra.push(cell(1, 1, "high"));
    let err = RiskMethodology::try_new(RiskMethodology {
        matrix: Some(RiskMatrix { cells: extra }),
        ..qualitative_3x3()
    })
    .expect_err("extra cell must fail closed");
    let extra_msg = err_text(&err);
    assert!(
        extra_msg.contains("malformed matrix") || extra_msg.contains("extra cell"),
        "extra cell class, got `{err}`"
    );
}

/// P05: duplicate ordinals rejected
#[test]
fn p05_t06_duplicate_ordinals_rejected() {
    let mut levels = lmh_levels();
    levels[2].ordinal = 2;
    let err = RiskMethodology::try_new(RiskMethodology {
        likelihood_scale: LikelihoodScale {
            id: "likelihood.lmh".into(),
            levels,
        },
        ..qualitative_3x3()
    })
    .expect_err("duplicate ordinal 2 must fail closed");
    assert_err_contains(&err, "duplicate ordinal");
}

/// P05: unreachable ratings rejected
#[test]
fn p05_t07_unreachable_ratings_rejected() {
    let mut ratings = lmh_ratings();
    ratings.levels.push(rating("critical", "Critical", 4));
    let err = RiskMethodology::try_new(RiskMethodology {
        rating_scale: ratings,
        ..qualitative_3x3()
    })
    .expect_err("rating never present on a cell must fail closed");
    assert_err_contains(&err, "unreachable");
}

/// P05: invalid boundaries rejected
#[test]
fn p05_t08_invalid_boundaries_rejected() {
    let base = quantitative_expected_loss();

    let overlap = RiskMethodology::try_new(RiskMethodology {
        bands: Some(vec![
            RatingBand {
                rating_id: "low".into(),
                min_inclusive: dec("0"),
                max_exclusive: Some(dec("1000")),
            },
            RatingBand {
                rating_id: "medium".into(),
                min_inclusive: dec("500"),
                max_exclusive: Some(dec("10000")),
            },
            RatingBand {
                rating_id: "high".into(),
                min_inclusive: dec("10000"),
                max_exclusive: None,
            },
        ]),
        ..base.clone()
    })
    .expect_err("overlapping bands must fail closed");
    assert_err_contains(&overlap, "overlap");

    let gap = RiskMethodology::try_new(RiskMethodology {
        bands: Some(vec![
            RatingBand {
                rating_id: "low".into(),
                min_inclusive: dec("0"),
                max_exclusive: Some(dec("1000")),
            },
            RatingBand {
                rating_id: "medium".into(),
                min_inclusive: dec("2000"),
                max_exclusive: Some(dec("10000")),
            },
            RatingBand {
                rating_id: "high".into(),
                min_inclusive: dec("10000"),
                max_exclusive: None,
            },
        ]),
        ..base.clone()
    })
    .expect_err("gapped bands must fail closed");
    assert_err_contains(&gap, "gap");

    let inverted = RiskMethodology::try_new(RiskMethodology {
        domain: Some(NumericDomain {
            min: dec("100"),
            max: dec("1"),
        }),
        ..base.clone()
    })
    .expect_err("inverted domain must fail closed");
    let inverted_msg = err_text(&inverted);
    assert!(
        inverted_msg.contains("invalid boundar")
            || inverted_msg.contains("min")
            || inverted_msg.contains("domain"),
        "inverted domain class, got `{inverted}`"
    );

    let appetite = RiskMethodology::try_new(RiskMethodology {
        appetite: RiskAppetite {
            max_rating_id: "high".into(),
        },
        tolerance: RiskTolerance {
            max_rating_id: "low".into(),
        },
        ..base
    })
    .expect_err("appetite above tolerance must fail closed");
    let appetite_msg = err_text(&appetite);
    assert!(
        appetite_msg.contains("appetite") && appetite_msg.contains("tolerance"),
        "appetite > tolerance class, got `{appetite}`"
    );
}

/// P05: scores outside declared domain rejected
#[test]
fn p05_t09_scores_outside_declared_domain_rejected() {
    let semi = semi_quantitative_5x5();
    let err = score_risk(
        &semi,
        &RiskScoreInput::SemiQuantitative {
            likelihood: 6,
            impact: 1,
        },
    )
    .expect_err("likelihood 6 on a 1–5 scale must fail closed (no clamp)");
    assert_err_contains(&err, "out of domain");

    let quant = quantitative_expected_loss();
    let err = score_risk(
        &quant,
        &RiskScoreInput::Quantitative {
            probability: dec("1.1"),
            loss: dec("10"),
        },
    )
    .expect_err("probability 1.1 must fail closed");
    let msg = err_text(&err);
    assert!(
        msg.contains("out of domain") || msg.contains("probability"),
        "probability 1.1 class, got `{err}`"
    );

    let err = score_risk(
        &quant,
        &RiskScoreInput::Quantitative {
            probability: dec("1"),
            loss: dec("1000000.01"),
        },
    )
    .expect_err("expected loss above domain max must fail closed (no clamp)");
    assert_err_contains(&err, "out of domain");
}

/// P05: deterministic canonical serialization
#[test]
fn p05_t10_deterministic_canonical_serialization() {
    let mut reversed = cells_3x3();
    reversed.reverse();
    let a = qualitative_3x3_with_cells(cells_3x3());
    let b = qualitative_3x3_with_cells(reversed);
    assert_eq!(
        canonical_digest(&a).unwrap(),
        canonical_digest(&b).unwrap(),
        "canonical_digest must be insert-order independent"
    );

    let fixture = load_methodology_fixture("risk-methodology-3x3.json");
    let value = serde_json::to_value(&fixture).expect("serialize 3x3 fixture");
    let round_trip: RiskMethodology =
        serde_json::from_value(value.clone()).expect("deserialize 3x3 fixture JSON");
    assert_eq!(
        canonical_digest(&fixture).unwrap(),
        canonical_digest(&round_trip).unwrap(),
        "3x3 fixture JSON must round-trip through serde"
    );
    assert_eq!(value["scoringMode"], "qualitative");
    assert_eq!(value["schemaVersion"], ASSURANCE_IR_SCHEMA);
}

/// P05: methodology lock and supersession
#[test]
fn p05_t11_methodology_lock_and_supersession() {
    let mut parent = qualitative_3x3();
    assert!(!parent.is_locked());
    parent.lock();
    assert!(parent.is_locked());
    parent.lock();
    assert!(parent.is_locked(), "lock() is idempotent");

    let before = canonical_digest(&parent).unwrap();
    let child = parent
        .supersede(RiskMethodologyId::new("rm:acme-qual-3x3:2"))
        .expect("supersede of a locked parent must produce a new revision");
    let after = canonical_digest(&parent).unwrap();
    assert_eq!(before, after, "locked parent must stay byte-identical");
    assert!(parent.is_locked());
    assert_eq!(parent.revision, 1);
    assert_eq!(child.revision, 2);
    assert_eq!(child.logical_id, parent.logical_id);
    assert_eq!(
        child.supersedes.as_ref().map(|id| id.as_str()),
        Some(parent.id.as_str())
    );
    assert!(!child.is_locked());
    assert_ne!(child.id.as_str(), parent.id.as_str());
    assert_ne!(
        canonical_digest(&child).unwrap(),
        before,
        "superseding revision must have a different digest"
    );

    let input = RiskScoreInput::Qualitative {
        likelihood_id: "medium".into(),
        impact_id: "high".into(),
    };
    let parent_score = score_risk(&parent, &input).unwrap();
    let child_score = score_risk(&child, &input).unwrap();
    assert_eq!(parent_score.rating.revision, 1);
    assert_eq!(child_score.rating.revision, 2);
    assert_eq!(
        parent_score.rating.methodology_id.as_str(),
        parent.id.as_str()
    );
    assert_eq!(
        child_score.rating.methodology_id.as_str(),
        child.id.as_str()
    );
}

/// P05: boundary calculations
#[test]
fn p05_t12_boundary_calculations() {
    let methodology = quantitative_expected_loss();
    let cases = [
        ("0", "low"),
        ("999.99", "low"),
        ("1000", "medium"),
        ("10000", "high"),
        ("1000000", "high"),
    ];
    for (loss, rating_id) in cases {
        let input = RiskScoreInput::Quantitative {
            probability: dec("1"),
            loss: dec(loss),
        };
        let scored = score_risk(&methodology, &input)
            .unwrap_or_else(|e| panic!("boundary {loss} must belong to a band: {e}"));
        assert_eq!(
            scored.rating.rating_id, rating_id,
            "exclusive-upper except last inclusive: {loss}"
        );
    }

    let err = score_risk(
        &methodology,
        &RiskScoreInput::Quantitative {
            probability: dec("1"),
            loss: dec("1000000.01"),
        },
    )
    .expect_err("1000000.01 is outside the declared domain");
    assert_err_contains(&err, "out of domain");
}

/// P05: raw input separated from derived rating
#[test]
fn p05_t13_raw_input_separated_from_derived_rating() {
    let methodology = qualitative_3x3();
    let input = RiskScoreInput::Qualitative {
        likelihood_id: "medium".into(),
        impact_id: "high".into(),
    };
    let scored = score_risk(&methodology, &input).unwrap();
    assert_eq!(scored.input, input, "ScoredRisk must retain the raw input");
    let _rating: &DerivedRating = &scored.rating;
    assert_eq!(scored.rating.rating_id, "high");

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk_methodology.rs");
    assert!(
        src.contains("fn score_risk"),
        "score_risk must be the only scoring entry"
    );
    let score_fn = src
        .split("fn score_risk")
        .nth(1)
        .expect("score_risk signature");
    let signature = score_fn.split('{').next().unwrap();
    assert!(
        !signature.to_ascii_lowercase().contains("ratingid")
            && !signature.contains("DerivedRating")
            && !signature.contains("RiskRating"),
        "score_risk must take raw input only, got `{signature}`"
    );
}

/// P05: collectors cannot emit RiskRating as evidence
#[test]
fn p05_t14_collectors_cannot_emit_risk_rating_as_evidence() {
    let product = product_crate_sources_joined();
    assert!(
        !product.contains("enum RiskRating")
            && !product.contains("RiskRating::High")
            && !product.contains("enum RiskRating {"),
        "no global RiskRating::High unit variant"
    );

    let collector = crate_sources_joined("weeping-angel-collector");
    for needle in [
        "RiskMethodology",
        "score_risk",
        "DerivedRating",
        "RiskRating",
    ] {
        assert!(
            !collector.contains(needle),
            "collector crate must remain free of `{needle}`"
        );
    }

    let evidence = crate_sources_joined("weeping-angel-evidence");
    assert!(
        !evidence.contains("Rating(") && !evidence.contains("RiskRating"),
        "EvidenceValue must not gain a rating variant"
    );
    match EvidenceValue::string("high") {
        EvidenceValue::String(_)
        | EvidenceValue::Bool(_)
        | EvidenceValue::Integer(_)
        | EvidenceValue::Decimal(_)
        | EvidenceValue::Timestamp(_)
        | EvidenceValue::DurationSeconds(_)
        | EvidenceValue::StringList(_)
        | EvidenceValue::Object(_) => {}
    }
}

/// P05: Risk::new and risk.json remain compatible
#[test]
fn p05_t15_risk_new_and_risk_json_remain_compatible() {
    let risk = Risk::new(
        RiskId::new("risk:source-tamper"),
        "Source tampering",
        "Unauthorized change to the source of record.",
    );
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    assert_eq!(risk.status, RiskStatus::Open);

    let json = serde_json::to_value(&risk).unwrap();
    let obj = json.as_object().unwrap();
    assert_eq!(obj.keys().count(), 4);
    for absent in ["likelihood", "impact", "score", "rating", "methodology"] {
        assert!(obj.get(absent).is_none(), "Risk must not gain `{absent}`");
    }

    let raw = fs::read_to_string(ir_fixture("risk.json")).unwrap();
    let decoded: Risk = serde_json::from_str(&raw).unwrap();
    assert_eq!(decoded.id.as_str(), "risk:source-tamper");
}

/// P05: qualitative vs quantitative modes without control-logic change
#[test]
fn p05_t16_qualitative_vs_quantitative_modes_without_control_logic_change() {
    let qualitative = qualitative_3x3();
    let quantitative = quantitative_expected_loss();
    let bounded_1_3 = custom_bounded_1_3();
    let bounded_1_5 = custom_bounded_1_5();

    let q = score_risk(
        &qualitative,
        &RiskScoreInput::Qualitative {
            likelihood_id: "medium".into(),
            impact_id: "high".into(),
        },
    )
    .unwrap();
    assert_eq!(q.rating.rating_id, "high");

    let n = score_risk(
        &quantitative,
        &RiskScoreInput::Quantitative {
            probability: dec("0.5"),
            loss: dec("2000"),
        },
    )
    .unwrap();
    assert_eq!(n.rating.rating_id, "medium");

    let b3 = score_risk(
        &bounded_1_3,
        &RiskScoreInput::CustomBounded { value: dec("9") },
    )
    .unwrap();
    assert_eq!(b3.rating.rating_id, "high");

    let b5_low = score_risk(
        &bounded_1_5,
        &RiskScoreInput::CustomBounded { value: dec("1") },
    )
    .unwrap();
    assert_eq!(b5_low.rating.rating_id, "low");
    let b5_high = score_risk(
        &bounded_1_5,
        &RiskScoreInput::CustomBounded { value: dec("5") },
    )
    .unwrap();
    assert_eq!(b5_high.rating.rating_id, "high");

    let err = score_risk(
        &quantitative,
        &RiskScoreInput::Qualitative {
            likelihood_id: "medium".into(),
            impact_id: "high".into(),
        },
    )
    .expect_err("cross-mode input must fail closed");
    assert_err_contains(&err, "mode");

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk_methodology.rs");
    let score_count = src.matches("fn score_risk").count();
    assert_eq!(
        score_count, 1,
        "all modes must share the same score_risk function, found {score_count}"
    );
}

/// P05: catalog infrastructure can name RiskMethodologyId
#[test]
fn p05_t17_catalog_infrastructure_can_name_risk_methodology_id() {
    let id = RiskMethodologyId::new("rm:acme-default:1");
    assert_eq!(id.as_str(), "rm:acme-default:1");
    let _stable: &dyn StableId = &id;

    assert!(matches!(
        RiskMethodologyId::try_new(""),
        Err(IdError::Empty)
    ));
    assert!(matches!(
        RiskMethodologyId::try_new("has space"),
        Err(IdError::InvalidCharacter)
    ));
    assert!(
        RiskMethodologyId::try_new("550e8400-e29b-41d4-a716-446655440000").is_err(),
        "uuid-v4 identities must still fail"
    );

    let ids = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    assert!(
        ids.contains("typed_id!(RiskMethodologyId);"),
        "RiskMethodologyId must be a typed_id! sibling of RiskId"
    );

    // IsmsContext-absence found-case skip-superseded by ISMS context IR.
    let _ = ScoringMode::Qualitative;
    let _ = ScoringMode::SemiQuantitative;
    let _ = ScoringMode::Quantitative;
    let _ = ScoringMode::CustomBounded;
}
