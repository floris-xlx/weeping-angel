use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct AuthSurfaceCheck;

#[async_trait]
impl Check for AuthSurfaceCheck {
    fn id(&self) -> &'static str {
        "auth-surface"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        for resp in ctx.responses.values() {
            let path = resp.final_url.path().to_ascii_lowercase();
            let status = resp.status.as_u16();

            if resp.is_html() {
                let document = Html::parse_document(&resp.body);
                if let Ok(sel) = Selector::parse("form") {
                    for form in document.select(&sel) {
                        let html_form = form.html();
                        let lower = html_form.to_ascii_lowercase();
                        let has_password = lower.contains("type=\"password\"")
                            || lower.contains("type='password'")
                            || lower.contains("name=\"password\"");
                        if has_password {
                            findings.push(
                                Finding::builder(self.id(), "login-form")
                                    .title("Login form detected")
                                    .severity(Severity::Info)
                                    .url(resp.final_url.as_str())
                                    .description(
                                        "A form with a password field was found (authentication surface).",
                                    )
                                    .evidence(Evidence::new(
                                        "form",
                                        html_form.chars().take(200).collect::<String>(),
                                    ))
                                    .build(),
                            );

                            if resp.final_url.scheme() != "https" {
                                findings.push(
                                    Finding::builder(self.id(), "login-over-http")
                                        .title("Login form served over HTTP")
                                        .severity(Severity::High)
                                        .url(resp.final_url.as_str())
                                        .description(
                                            "Password form is not served over TLS.",
                                        )
                                        .remediation("Serve authentication only over HTTPS.")
                                        .cwe("CWE-319")
                                        .build(),
                                );
                            }
                        }
                    }
                }
            }

            // Unauthenticated admin-ish 200
            let admin_like = path.contains("admin")
                || path.contains("dashboard")
                || path.contains("manage")
                || path.contains("console");
            if admin_like && status == 200 {
                let body_l = resp.body.to_ascii_lowercase();
                let looks_login = body_l.contains("password") || body_l.contains("sign in");
                if !looks_login {
                    findings.push(
                        Finding::builder(self.id(), "admin-200")
                            .title("Admin-like path returned 200 without obvious login")
                            .severity(Severity::Medium)
                            .url(resp.final_url.as_str())
                            .description(
                                "Path name suggests administration UI and returned HTTP 200. Verify authorization.",
                            )
                            .remediation(
                                "Enforce authentication and authorization on administrative routes.",
                            )
                            .cwe("CWE-306")
                            .build(),
                    );
                } else {
                    findings.push(
                        Finding::builder(self.id(), "admin-login")
                            .title("Administrative login surface")
                            .severity(Severity::Info)
                            .url(resp.final_url.as_str())
                            .description("Admin-like path appears to present a login challenge.")
                            .build(),
                    );
                }
            }

            if path.contains("oauth") || path.contains("callback") || path.contains("sso") {
                findings.push(
                    Finding::builder(self.id(), "oauth-surface")
                        .title("OAuth/SSO-related path")
                        .severity(Severity::Info)
                        .url(resp.final_url.as_str())
                        .description(
                            "OAuth/callback-style path discovered; review redirect URI validation.",
                        )
                        .build(),
                );
            }

            if (path.contains("reset") && path.contains("password"))
                || path.contains("forgot-password")
                || path.contains("forgot_password")
            {
                findings.push(
                    Finding::builder(self.id(), "password-reset")
                        .title("Password reset surface")
                        .severity(Severity::Info)
                        .url(resp.final_url.as_str())
                        .description("Password reset flow endpoint discovered.")
                        .build(),
                );
            }

            for cookie in resp.set_cookies() {
                let name = cookie.split('=').next().unwrap_or("");
                let lname = name.to_ascii_lowercase();
                if lname.contains("session")
                    || lname.contains("sid")
                    || lname == "jwt"
                    || lname.contains("auth")
                {
                    findings.push(
                        Finding::builder(self.id(), "session-cookie")
                            .title(format!("Session-like cookie: {name}"))
                            .severity(Severity::Info)
                            .url(resp.final_url.as_str())
                            .description("Application sets a session-like cookie.")
                            .evidence(Evidence::new("set-cookie", cookie.chars().take(120).collect::<String>()))
                            .build(),
                    );
                }
            }
        }

        Ok(findings)
    }
}
