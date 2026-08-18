//! Execution for `weeping-angel assurance catalog`. Parser lives in `cli.rs`.

use anyhow::{Context, Result, bail};
use weeping_angel_canonical_catalog::CanonicalCatalog;

use crate::cli::{AssuranceCatalogArgs, AssuranceCatalogCommand};

const NOT_CERTIFICATION: &str = "This is a readiness assessment and is not certification.";

pub fn run(args: AssuranceCatalogArgs) -> Result<i32> {
    println!("{NOT_CERTIFICATION}");
    match args.command {
        AssuranceCatalogCommand::Validate { path } => {
            CanonicalCatalog::load(&path)
                .with_context(|| format!("validate catalog at {}", path.display()))?;
            println!("ok: {}", path.display());
            Ok(0)
        }
        AssuranceCatalogCommand::Stats { path } => {
            let catalog = CanonicalCatalog::load(&path)
                .with_context(|| format!("load catalog at {}", path.display()))?;
            let stats = catalog.stats().context("catalog digest")?;
            println!("schema: {}", stats.schema);
            println!("catalog: {}", stats.catalog_id);
            println!("version: {}", stats.catalog_version);
            println!("controls: {}", stats.control_count);
            println!("evidence: {}", stats.evidence_count);
            println!("tests: {}", stats.test_count);
            println!("digest: {}", stats.digest);
            Ok(0)
        }
        AssuranceCatalogCommand::Inspect { control_id, path } => {
            let catalog = CanonicalCatalog::load(&path)
                .with_context(|| format!("load catalog at {}", path.display()))?;
            let control = match catalog.control(&control_id) {
                Ok(control) => control,
                Err(err) => {
                    eprintln!("{err}");
                    bail!("{err}");
                }
            };
            println!("control: {}", control.id);
            println!("title: {}", control.title);
            if !control.objective.is_empty() {
                println!("objective: {}", control.objective);
            }
            println!("evidence:");
            for evidence_id in &control.evidence {
                println!("  {evidence_id}");
                if let Some(evidence) = catalog.evidence().get(evidence_id) {
                    println!("    title: {}", evidence.title);
                    println!("    evidence_type: {}", evidence.evidence_type);
                }
            }
            println!("tests:");
            for test_id in &control.tests {
                println!("  {test_id}");
                if let Some(test) = catalog.tests().get(test_id) {
                    println!("    kind: {}", test.kind);
                    println!("    control: {}", test.control);
                    for required in &test.required_evidence {
                        println!("    required_evidence: {required}");
                    }
                }
            }
            Ok(0)
        }
    }
}
