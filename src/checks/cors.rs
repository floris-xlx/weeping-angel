use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct CorsCheck;

#[async_trait]
impl Check for CorsCheck {
    fn id(&self) -> &'static str {
        "cors"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        let evil = "https://evil-weeping-angel.example";

        // Test a few representative URLs
        let mut urls = vec![ctx.seed.clone()];
        for asset in ctx.assets.iter().take(10) {
            urls.push(asset.url.clone());
        }

        let mut tested = std::collections::HashSet::new();
        for url in urls {
            if !tested.insert(url.as_str().to_string()) {
                continue;
            }
            let mut headers = HashMap::new();
            headers.insert("Origin".into(), evil.into());

            let resp = match ctx
                .client
                .request(reqwest::Method::GET, &url, None, Some(headers))
                .await
            {
                Ok(r) => r,
                Err(_) => continue,
            };

            let acao = resp.header("access-control-allow-origin");
            let acac = resp.header("access-control-allow-credentials");

            if let Some(origin) = acao {
                if origin == "*" {
                    let sev = if acac.map(|v| v.eq_ignore_ascii_case("true")).unwrap_or(false) {
                        Severity::High
                    } else {
                        Severity::Low
                    };
                    findings.push(
                        Finding::builder(self.id(), "cors-wildcard")
                            .title("CORS allows any origin (*)")
                            .severity(sev)
                            .url(url.as_str())
                            .description(
                                "Access-Control-Allow-Origin: * reflects a permissive cross-origin policy.",
                            )
                            .remediation("Reflect only trusted origins; avoid * with credentials.")
                            .cwe("CWE-942")
                            .evidence(Evidence::new(
                                "access-control-allow-origin",
                                origin,
                            ))
                            .build(),
                    );
                } else if origin == evil {
                    findings.push(
                        Finding::builder(self.id(), "cors-reflect-origin")
                            .title("CORS reflects arbitrary Origin")
                            .severity(Severity::High)
                            .url(url.as_str())
                            .description(
                                "Server reflected a foreign Origin, which can enable cross-site data reads if credentials are allowed.",
                            )
                            .remediation("Allowlist trusted origins only; never reflect unvalidated Origin.")
                            .cwe("CWE-942")
                            .evidence(Evidence::new("access-control-allow-origin", origin))
                            .build(),
                    );
                    if acac.map(|v| v.eq_ignore_ascii_case("true")).unwrap_or(false) {
                        findings.push(
                            Finding::builder(self.id(), "cors-credentials-reflected")
                                .title("CORS reflects Origin with credentials")
                                .severity(Severity::Critical)
                                .url(url.as_str())
                                .description(
                                    "Arbitrary origin reflection with Access-Control-Allow-Credentials: true is highly dangerous.",
                                )
                                .remediation("Never combine credentialed CORS with open origin reflection.")
                                .cwe("CWE-942")
                                .build(),
                        );
                    }
                }
            }
        }

        Ok(findings)
    }
}
