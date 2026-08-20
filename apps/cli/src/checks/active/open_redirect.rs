use anyhow::Result;
use async_trait::async_trait;
use url::Url;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct OpenRedirectProbe;

const REDIRECT_PARAMS: &[&str] = &[
    "url",
    "next",
    "redirect",
    "redirect_uri",
    "return",
    "returnUrl",
    "return_to",
    "goto",
    "dest",
    "destination",
    "continue",
    "rurl",
    "target",
];

#[async_trait]
impl Check for OpenRedirectProbe {
    fn id(&self) -> &'static str {
        "open-redirect"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Active
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        let evil = "https://evil-weeping-angel.example/landing";
        let mut tested = 0usize;

        for u in ctx.discovered_urls.iter().take(40) {
            let Ok(base) = Url::parse(u) else {
                continue;
            };
            let pairs: Vec<(String, String)> = base
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            let mut param_names: Vec<String> = pairs
                .iter()
                .map(|(k, _)| k.clone())
                .filter(|k| REDIRECT_PARAMS.iter().any(|p| k.eq_ignore_ascii_case(p)))
                .collect();

            if param_names.is_empty() && base.path().to_ascii_lowercase().contains("login") {
                param_names.push("next".into());
            }

            for pk in param_names {
                if tested >= 15 {
                    break;
                }
                tested += 1;
                let mut url = base.clone();
                let existing: Vec<_> = url
                    .query_pairs()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                url.query_pairs_mut().clear();
                let mut replaced = false;
                for (k, v) in &existing {
                    if k == &pk {
                        url.query_pairs_mut().append_pair(k, evil);
                        replaced = true;
                    } else {
                        url.query_pairs_mut().append_pair(k, v);
                    }
                }
                if !replaced {
                    url.query_pairs_mut().append_pair(&pk, evil);
                }

                let resp = match ctx.client.get(&url).await {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                let location = resp.header("location").unwrap_or("");
                let final_host = resp.final_url.host_str().unwrap_or("");
                if location.contains("evil-weeping-angel.example")
                    || final_host.contains("evil-weeping-angel.example")
                    || resp.body.contains("evil-weeping-angel.example")
                {
                    findings.push(
                        Finding::builder(self.id(), "open-redirect")
                            .title("Possible open redirect")
                            .severity(Severity::Medium)
                            .url(url.as_str())
                            .description(format!(
                                "Parameter `{pk}` appears to influence redirect to an external host."
                            ))
                            .remediation(
                                "Allowlist redirect targets; reject absolute external URLs.",
                            )
                            .cwe("CWE-601")
                            .evidence(Evidence::new(
                                "location/final",
                                format!("location={location}; final={}", resp.final_url),
                            ))
                            .build(),
                    );
                }
            }
        }

        Ok(findings)
    }
}
