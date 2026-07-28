use anyhow::Result;
use async_trait::async_trait;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct TechCheck;

#[async_trait]
impl Check for TechCheck {
    fn id(&self) -> &'static str {
        "tech"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for resp in ctx.responses.values() {
            for header in ["server", "x-powered-by", "x-aspnet-version", "x-generator"] {
                if let Some(val) = resp.header(header) {
                    let key = format!("{header}:{val}");
                    if seen.insert(key) {
                        findings.push(
                            Finding::builder(self.id(), "tech-header")
                                .title(format!("Technology header: {header}"))
                                .severity(Severity::Info)
                                .url(resp.final_url.as_str())
                                .description(format!(
                                    "Response advertises `{header}: {val}`, which aids fingerprinting."
                                ))
                                .remediation("Omit or genericize server technology headers where practical.")
                                .evidence(Evidence::new(header, val))
                                .build(),
                        );
                    }
                }
            }

            let body = &resp.body;
            let signatures = [
                ("wp-content", "WordPress"),
                ("__NEXT_DATA__", "Next.js"),
                ("ng-version", "Angular"),
                ("csrfmiddlewaretoken", "Django"),
                ("rails-env", "Ruby on Rails"),
                ("laravel_session", "Laravel"),
                ("firebase/app", "Firebase"),
                ("firebase/firestore", "Cloud Firestore"),
                ("getFirestore", "Cloud Firestore"),
                ("firestore.googleapis.com", "Cloud Firestore"),
                ("firebaseio.com", "Firebase Realtime Database"),
                ("identitytoolkit.googleapis.com", "Firebase Auth"),
                ("initializeApp", "Firebase (possible)"),
            ];
            for (sig, name) in signatures {
                if body.contains(sig) && seen.insert(format!("body:{name}")) {
                    findings.push(
                        Finding::builder(self.id(), "tech-body")
                            .title(format!("Possible technology: {name}"))
                            .severity(Severity::Info)
                            .url(resp.final_url.as_str())
                            .description(format!("Body signature `{sig}` suggests {name}."))
                            .evidence(Evidence::new("body", sig))
                            .build(),
                    );
                }
            }
        }

        Ok(findings)
    }
}
