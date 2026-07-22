use anyhow::Result;
use async_trait::async_trait;
use url::Url;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct SqliErrorProbe;

const PAYLOADS: &[&str] = &["'", "\"", "' OR '1'='1", "1' AND '1'='1"];

const ERROR_SIGS: &[&str] = &[
    "you have an error in your sql syntax",
    "warning: mysql",
    "unclosed quotation mark after the character string",
    "quoted string not properly terminated",
    "pg_query()",
    "sqlite3.operationalerror",
    "sqlstate[",
    "odbc sql server driver",
    "oracle error",
    "mysql_fetch",
    "syntax error at or near",
];

#[async_trait]
impl Check for SqliErrorProbe {
    fn id(&self) -> &'static str {
        "sqli"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Active
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        let mut tested = 0usize;
        const MAX: usize = 20;

        let mut targets: Vec<Url> = ctx
            .discovered_urls
            .iter()
            .filter_map(|u| Url::parse(u).ok())
            .filter(|u| u.query().is_some())
            .take(MAX)
            .collect();
        if targets.is_empty() {
            targets.push(ctx.seed.clone());
        }

        for base in targets {
            if tested >= MAX {
                break;
            }
            let pairs: Vec<(String, String)> = base
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            let params = if pairs.is_empty() {
                vec![("id".into(), "1".into())]
            } else {
                pairs
            };

            for (pk, _) in &params {
                for payload in PAYLOADS {
                    if tested >= MAX {
                        break;
                    }
                    tested += 1;
                    let mut url = base.clone();
                    url.query_pairs_mut().clear();
                    for (k, v) in &params {
                        if k == pk {
                            url.query_pairs_mut().append_pair(k, payload);
                        } else {
                            url.query_pairs_mut().append_pair(k, v);
                        }
                    }
                    if params.len() == 1 && params[0].0 == "id" && base.query().is_none() {
                        url.query_pairs_mut().clear();
                        url.query_pairs_mut().append_pair("id", payload);
                    }

                    let resp = match ctx.client.get(&url).await {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    let body_l = resp.body.to_ascii_lowercase();
                    if let Some(sig) = ERROR_SIGS.iter().find(|s| body_l.contains(*s)) {
                        findings.push(
                            Finding::builder(self.id(), "sqli-error-based")
                                .title("Possible SQL injection (error-based signature)")
                                .severity(Severity::High)
                                .url(url.as_str())
                                .description(format!(
                                    "Payload on parameter `{pk}` triggered a database error signature."
                                ))
                                .remediation(
                                    "Use parameterized queries; never concatenate untrusted input into SQL.",
                                )
                                .cwe("CWE-89")
                                .evidence(Evidence::new("body", *sig))
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
