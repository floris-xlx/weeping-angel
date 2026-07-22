use anyhow::Result;
use async_trait::async_trait;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct ExposuresCheck;

#[async_trait]
impl Check for ExposuresCheck {
    fn id(&self) -> &'static str {
        "exposures"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        for (url_str, resp) in &ctx.responses {
            let path = resp.final_url.path().to_ascii_lowercase();
            let status = resp.status.as_u16();

            if status == 200 {
                if path.contains(".env")
                    && (resp.body.contains('=')
                        || resp.body.contains("API")
                        || resp.body.contains("SECRET"))
                {
                    findings.push(
                        Finding::builder(self.id(), "exposed-env")
                            .title("Possible exposed .env file")
                            .severity(Severity::Critical)
                            .url(url_str)
                            .description("A path resembling .env returned 200 with key=value-like content.")
                            .remediation("Block public access to environment files immediately; rotate secrets.")
                            .cwe("CWE-538")
                            .evidence(Evidence::new("body", resp.body.chars().take(120).collect::<String>()))
                            .build(),
                    );
                }

                if (path.contains(".git/head") || path.contains("/.git/"))
                    && (resp.body.contains("ref:") || resp.body.contains("refs/heads"))
                {
                    findings.push(
                        Finding::builder(self.id(), "exposed-git")
                            .title("Exposed .git metadata")
                            .severity(Severity::High)
                            .url(url_str)
                            .description("Git metadata appears publicly accessible.")
                            .remediation("Deny web access to .git directories.")
                            .cwe("CWE-538")
                            .evidence(Evidence::new("body", resp.body.chars().take(80).collect::<String>()))
                            .build(),
                    );
                }

                if path.contains("phpinfo") && resp.body.to_ascii_lowercase().contains("php version")
                {
                    findings.push(
                        Finding::builder(self.id(), "phpinfo")
                            .title("phpinfo() page exposed")
                            .severity(Severity::High)
                            .url(url_str)
                            .description("phpinfo output can leak paths, modules, and configuration.")
                            .remediation("Remove phpinfo scripts from production.")
                            .cwe("CWE-200")
                            .build(),
                    );
                }

                if path.contains("actuator") && (resp.is_json() || resp.body.contains("status")) {
                    findings.push(
                        Finding::builder(self.id(), "spring-actuator")
                            .title("Possible Spring Actuator endpoint")
                            .severity(Severity::High)
                            .url(url_str)
                            .description("Actuator endpoints may expose env, beans, or heap dumps.")
                            .remediation("Disable public actuator endpoints or require strong auth.")
                            .cwe("CWE-200")
                            .build(),
                    );
                }

                // Directory listing
                let body_l = resp.body.to_ascii_lowercase();
                if body_l.contains("index of /")
                    || body_l.contains("<title>directory listing")
                    || (body_l.contains("parent directory") && body_l.contains("<a href="))
                {
                    findings.push(
                        Finding::builder(self.id(), "directory-listing")
                            .title("Directory listing enabled")
                            .severity(Severity::Medium)
                            .url(url_str)
                            .description("Server appears to list directory contents.")
                            .remediation("Disable autoindex/directory listing.")
                            .cwe("CWE-548")
                            .build(),
                    );
                }

                // Stack traces
                if body_l.contains("traceback (most recent call last)")
                    || body_l.contains("stacktrace")
                    || body_l.contains("exception in thread")
                    || (body_l.contains("at com.") && body_l.contains("exception"))
                {
                    findings.push(
                        Finding::builder(self.id(), "stack-trace")
                            .title("Possible verbose error / stack trace")
                            .severity(Severity::Medium)
                            .url(url_str)
                            .description("Response may contain framework stack traces useful to attackers.")
                            .remediation("Return generic errors in production; log details server-side only.")
                            .cwe("CWE-209")
                            .build(),
                    );
                }

                if path.contains("swagger") || path.contains("openapi") || path.contains("api-docs")
                {
                    if resp.body.contains("swagger")
                        || resp.body.contains("openapi")
                        || resp.body.contains("paths")
                    {
                        findings.push(
                            Finding::builder(self.id(), "api-docs-exposed")
                                .title("API documentation endpoint accessible")
                                .severity(Severity::Low)
                                .url(url_str)
                                .description(
                                    "OpenAPI/Swagger docs expand attack surface mapping for unauthenticated users.",
                                )
                                .remediation(
                                    "Restrict API docs to internal networks or authenticated operators.",
                                )
                                .build(),
                        );
                    }
                }
            }

            // Interesting auth walls on sensitive paths
            if matches!(status, 401 | 403)
                && (path.contains("admin") || path.contains("actuator") || path.contains(".env"))
            {
                findings.push(
                    Finding::builder(self.id(), "protected-sensitive-path")
                        .title(format!("Sensitive path returned {status}"))
                        .severity(Severity::Info)
                        .url(url_str)
                        .description(
                            "Path exists but is protected. Confirm authorization is robust.",
                        )
                        .build(),
                );
            }
        }

        Ok(findings)
    }
}
