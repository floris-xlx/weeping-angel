use anyhow::Result;
use async_trait::async_trait;
use url::Url;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct PathTraversalProbe;

const PAYLOADS: &[&str] = &[
    "../../../etc/passwd",
    "..%2f..%2f..%2fetc%2fpasswd",
    "....//....//....//etc/passwd",
];

const SIGS: &[&str] = &["root:x:0:0:", "[extensions]", "for 16-bit app support"];

#[async_trait]
impl Check for PathTraversalProbe {
    fn id(&self) -> &'static str {
        "path-traversal"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Active
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        let mut tested = 0usize;

        let mut urls: Vec<Url> = ctx
            .discovered_urls
            .iter()
            .filter_map(|u| Url::parse(u).ok())
            .filter(|u| {
                u.query()
                    .map(|q| {
                        q.to_ascii_lowercase().contains("file")
                            || q.to_ascii_lowercase().contains("path")
                            || q.to_ascii_lowercase().contains("page")
                            || q.to_ascii_lowercase().contains("template")
                            || q.to_ascii_lowercase().contains("doc")
                    })
                    .unwrap_or(false)
            })
            .take(15)
            .collect();

        if urls.is_empty() {
            // light probe on seed with file param only once
            let mut u = ctx.seed.clone();
            u.query_pairs_mut().append_pair("file", PAYLOADS[0]);
            urls.push(u);
        }

        for base in urls {
            if tested >= 12 {
                break;
            }
            let pairs: Vec<(String, String)> = base
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            let keys: Vec<String> = if pairs.is_empty() {
                vec!["file".into()]
            } else {
                pairs.iter().map(|(k, _)| k.clone()).collect()
            };

            for key in keys {
                for payload in PAYLOADS {
                    if tested >= 12 {
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
                        if k == &key {
                            url.query_pairs_mut().append_pair(k, payload);
                            replaced = true;
                        } else {
                            url.query_pairs_mut().append_pair(k, v);
                        }
                    }
                    if !replaced {
                        url.query_pairs_mut().append_pair(&key, payload);
                    }

                    let resp = match ctx.client.get(&url).await {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    if SIGS.iter().any(|s| resp.body.contains(s)) {
                        findings.push(
                            Finding::builder(self.id(), "path-traversal")
                                .title("Possible path traversal")
                                .severity(Severity::High)
                                .url(url.as_str())
                                .description(format!(
                                    "Parameter `{key}` may allow reading local files."
                                ))
                                .remediation(
                                    "Canonicalize paths; reject .. segments; use allowlists for file access.",
                                )
                                .cwe("CWE-22")
                                .evidence(Evidence::new("body", "local file signature matched"))
                                .build(),
                        );
                        break;
                    }
                }
            }
        }

        Ok(findings)
    }
}
