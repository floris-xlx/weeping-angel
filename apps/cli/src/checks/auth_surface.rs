//! Map authentication surfaces and whether login/signup are publicly reachable
//! (unguarded) vs challenging unauthenticated callers (guarded).

use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};
use url::Url;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct AuthSurfaceCheck;

const LOGIN_HINTS: &[&str] = &[
    "login",
    "signin",
    "sign-in",
    "sign_in",
    "log-in",
    "log_in",
    "session/new",
    "auth/login",
    "accounts/login",
];

const SIGNUP_HINTS: &[&str] = &[
    "signup",
    "sign-up",
    "sign_up",
    "register",
    "registration",
    "join",
    "create-account",
    "create_account",
    "auth/register",
    "accounts/signup",
];

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
        let mut seen = std::collections::HashSet::new();

        for resp in ctx.responses.values() {
            let path = resp.final_url.path().to_ascii_lowercase();
            let status = resp.status.as_u16();
            let url = resp.final_url.as_str();

            if resp.is_html() {
                let document = Html::parse_document(&resp.body);
                if let Ok(sel) = Selector::parse("form") {
                    for form in document.select(&sel) {
                        let html_form = form.html();
                        let lower = html_form.to_ascii_lowercase();
                        let has_password = lower.contains("type=\"password\"")
                            || lower.contains("type='password'")
                            || lower.contains("name=\"password\"")
                            || lower.contains("name='password'");
                        if !has_password {
                            continue;
                        }

                        let action = form
                            .value()
                            .attr("action")
                            .unwrap_or("")
                            .to_ascii_lowercase();
                        let form_text = format!("{lower} {action} {path}");
                        let is_signup = SIGNUP_HINTS.iter().any(|h| form_text.contains(h))
                            || lower.contains("register")
                            || lower.contains("create account")
                            || lower.contains("sign up");
                        let is_login =
                            !is_signup || LOGIN_HINTS.iter().any(|h| form_text.contains(h));

                        if is_signup && seen.insert(format!("signup-form:{url}")) {
                            findings.push(
                                    Finding::builder(self.id(), "signup-form")
                                        .title("Signup / registration form detected")
                                        .severity(Severity::Info)
                                        .url(url)
                                        .description(
                                            "A registration-style form with a password field is publicly reachable (unauthenticated GET).",
                                        )
                                        .evidence(Evidence::new(
                                            "form",
                                            html_form.chars().take(220).collect::<String>(),
                                        ))
                                        .build(),
                                );
                            findings.push(
                                    Finding::builder(self.id(), "signup-unguarded")
                                        .title("Signup surface is unauthenticated (public form)")
                                        .severity(Severity::Info)
                                        .url(url)
                                        .description(
                                            "Signup is intentionally unauthenticated for most apps. Confirm rate limits, CAPTCHA/App Check, email verification, and abuse controls are present.",
                                        )
                                        .remediation(
                                            "Rate-limit registration, require verification, block disposable emails, enable bot protection.",
                                        )
                                        .build(),
                                );
                        }

                        if is_login || !is_signup {
                            if seen.insert(format!("login-form:{url}")) {
                                findings.push(
                                    Finding::builder(self.id(), "login-form")
                                        .title("Login form detected")
                                        .severity(Severity::Info)
                                        .url(url)
                                        .description(
                                            "A form with a password field was found (authentication surface). The form page itself is unauthenticated — expected for login UX.",
                                        )
                                        .evidence(Evidence::new(
                                            "form",
                                            html_form.chars().take(200).collect::<String>(),
                                        ))
                                        .build(),
                                );
                                findings.push(
                                    Finding::builder(self.id(), "login-unguarded")
                                        .title("Login surface is unauthenticated (public form)")
                                        .severity(Severity::Info)
                                        .url(url)
                                        .description(
                                            "Login page returned a password form without prior authentication (normal). Downstream session issuance must still enforce credentials; check credential-stuffing protections separately (rate-limits module).",
                                        )
                                        .build(),
                                );
                            }

                            if resp.final_url.scheme() != "https" {
                                findings.push(
                                    Finding::builder(self.id(), "login-over-http")
                                        .title("Login form served over HTTP")
                                        .severity(Severity::High)
                                        .url(url)
                                        .description("Password form is not served over TLS.")
                                        .remediation("Serve authentication only over HTTPS.")
                                        .cwe("CWE-319")
                                        .build(),
                                );
                            }
                        }
                    }
                }
            }

            // Path-based login/signup classification for non-HTML / JSON APIs
            let looks_login_path = LOGIN_HINTS.iter().any(|h| path.contains(h));
            let looks_signup_path = SIGNUP_HINTS.iter().any(|h| path.contains(h));

            if looks_login_path || looks_signup_path {
                let kind = if looks_signup_path { "signup" } else { "login" };
                let classification = classify_auth_response(status, &resp.body, resp.is_html());
                if seen.insert(format!("auth-path:{kind}:{url}")) {
                    match classification {
                        AuthClass::PublicOk => {
                            findings.push(
                                Finding::builder(self.id(), format!("{kind}-unguarded"))
                                    .title(format!(
                                        "{kind} endpoint reachable without authentication (HTTP {status})"
                                    ))
                                    .severity(Severity::Info)
                                    .url(url)
                                    .description(format!(
                                        "Path looks like {kind} and returned HTTP {status} to an unauthenticated client. \
                                         For HTML login/signup this is usually expected; for JSON APIs that return tokens/data without credentials it is a weakness."
                                    ))
                                    .evidence(Evidence::new(
                                        "body",
                                        resp.body.chars().take(160).collect::<String>(),
                                    ))
                                    .build(),
                            );
                            if resp.is_json() && body_looks_sensitive(&resp.body) {
                                findings.push(
                                    Finding::builder(
                                        self.id(),
                                        "auth-endpoint-unauthenticated",
                                    )
                                    .title(format!(
                                        "{kind} API returns data without credentials"
                                    ))
                                    .severity(Severity::High)
                                    .url(url)
                                    .description(
                                        "Auth-related JSON endpoint returned success with sensitive-looking content without a session.",
                                    )
                                    .remediation(
                                        "Require credentials; return 401 for anonymous callers.",
                                    )
                                    .cwe("CWE-306")
                                    .build(),
                                );
                            }
                        }
                        AuthClass::Challenge => {
                            findings.push(
                                Finding::builder(self.id(), format!("{kind}-guarded"))
                                    .title(format!(
                                        "{kind} endpoint challenges unauthenticated callers (HTTP {status})"
                                    ))
                                    .severity(Severity::Info)
                                    .url(url)
                                    .description(format!(
                                        "Unauthenticated request received HTTP {status} (auth challenge or deny). Endpoint is not openly returning success content.",
                                    ))
                                    .build(),
                            );
                            findings.push(
                                Finding::builder(self.id(), "auth-endpoint-protected")
                                    .title(format!("Auth-related path protected ({kind})"))
                                    .severity(Severity::Info)
                                    .url(url)
                                    .description(
                                        "Anonymous access is denied or redirected away from privileged content.",
                                    )
                                    .build(),
                            );
                        }
                        AuthClass::Error => {
                            findings.push(
                                Finding::builder(self.id(), format!("{kind}-error"))
                                    .title(format!(
                                        "{kind} path returned error HTTP {status}"
                                    ))
                                    .severity(Severity::Info)
                                    .url(url)
                                    .description(
                                        "Auth-related path returned a client/server error to anonymous GET.",
                                    )
                                    .build(),
                            );
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
                            .url(url)
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
                            .url(url)
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
                        .url(url)
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
                        .url(url)
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
                            .url(url)
                            .description("Application sets a session-like cookie.")
                            .evidence(Evidence::new(
                                "set-cookie",
                                cookie.chars().take(120).collect::<String>(),
                            ))
                            .build(),
                    );
                }
            }
        }

        // Probe discovered auth paths not yet in response cache (wordlist hits with status only)
        let probe_client = ctx.anon_client.as_ref().unwrap_or(&ctx.client);
        let mut extra: Vec<Url> = Vec::new();
        for u in &ctx.discovered_urls {
            let Ok(parsed) = Url::parse(u) else {
                continue;
            };
            let path = parsed.path().to_ascii_lowercase();
            let interesting = LOGIN_HINTS.iter().any(|h| path.contains(h))
                || SIGNUP_HINTS.iter().any(|h| path.contains(h));
            if interesting && !ctx.responses.contains_key(u) {
                extra.push(parsed);
            }
        }
        extra.truncate(15);
        for url in extra {
            if let Ok(resp) = probe_client.get(&url).await {
                let status = resp.status.as_u16();
                let path = url.path().to_ascii_lowercase();
                let kind = if SIGNUP_HINTS.iter().any(|h| path.contains(h)) {
                    "signup"
                } else {
                    "login"
                };
                let key = format!("probe:{kind}:{}", url.as_str());
                if !seen.insert(key) {
                    continue;
                }
                if (200..300).contains(&status) {
                    findings.push(
                        Finding::builder(self.id(), format!("{kind}-unguarded"))
                            .title(format!(
                                "{kind} path publicly reachable (HTTP {status})"
                            ))
                            .severity(Severity::Info)
                            .url(url.as_str())
                            .description(
                                "Unauthenticated GET succeeded on an auth-named path. Typical for login/signup pages; verify POST handlers enforce credentials and throttling.",
                            )
                            .build(),
                    );
                } else if status == 401 || status == 403 {
                    findings.push(
                        Finding::builder(self.id(), format!("{kind}-guarded"))
                            .title(format!(
                                "{kind} path denies anonymous access (HTTP {status})"
                            ))
                            .severity(Severity::Info)
                            .url(url.as_str())
                            .description(
                                "Auth-named path is not openly readable without credentials.",
                            )
                            .build(),
                    );
                }
            }
        }

        // Summary
        let login_public = findings
            .iter()
            .any(|f| f.id == "login-unguarded" || f.id == "login-form");
        let signup_public = findings
            .iter()
            .any(|f| f.id == "signup-unguarded" || f.id == "signup-form");
        let login_guarded = findings.iter().any(|f| f.id == "login-guarded");
        let signup_guarded = findings.iter().any(|f| f.id == "signup-guarded");

        findings.push(
            Finding::builder(self.id(), "auth-guard-summary")
                .title("Login / signup authentication guard summary")
                .severity(Severity::Info)
                .url(ctx.seed.as_str())
                .description(format!(
                    "login: {} · signup: {}. \
                     Public forms are normal; guarded means anonymous GET was challenged (401/403/redirect). \
                     Use auth-compare + rate-limits for deeper checks.",
                    if login_public && login_guarded {
                        "public form present; some paths guarded"
                    } else if login_public {
                        "unauthenticated (public form/endpoint)"
                    } else if login_guarded {
                        "guarded (anonymous challenged)"
                    } else {
                        "not clearly observed"
                    },
                    if signup_public && signup_guarded {
                        "public form present; some paths guarded"
                    } else if signup_public {
                        "unauthenticated (public form/endpoint)"
                    } else if signup_guarded {
                        "guarded (anonymous challenged)"
                    } else {
                        "not clearly observed"
                    }
                ))
                .build(),
        );

        Ok(findings)
    }
}

enum AuthClass {
    PublicOk,
    Challenge,
    Error,
}

fn classify_auth_response(status: u16, body: &str, is_html: bool) -> AuthClass {
    if status == 401 || status == 403 {
        return AuthClass::Challenge;
    }
    if (300..400).contains(&status) {
        // redirect often means "go login" or away from resource — treat as challenge-ish
        return AuthClass::Challenge;
    }
    if (200..300).contains(&status) {
        let lower = body.to_ascii_lowercase();
        if is_html
            && (lower.contains("password")
                || lower.contains("sign in")
                || lower.contains("log in")
                || lower.contains("create account"))
        {
            return AuthClass::PublicOk;
        }
        if (200..300).contains(&status) {
            return AuthClass::PublicOk;
        }
    }
    if status >= 400 {
        return AuthClass::Error;
    }
    AuthClass::PublicOk
}

fn body_looks_sensitive(body: &str) -> bool {
    let l = body.to_ascii_lowercase();
    l.contains("email")
        || l.contains("token")
        || l.contains("access_token")
        || l.contains("\"role\"")
        || l.contains("password")
        || l.contains("users")
}
