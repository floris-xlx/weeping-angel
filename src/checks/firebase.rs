//! Detect Firebase / Cloud Firestore client usage and common weak surfaces.
//!
//! Looks for SDK scripts, `firebaseConfig` / `initializeApp` blobs, Firestore REST
//! and Realtime Database URLs, Identity Toolkit auth endpoints, and unauthenticated
//! document/database probes when those URLs appear in-scope.

use anyhow::Result;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct FirebaseCheck;

static FIREBASE_HOST_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)(firestore\.googleapis\.com|firebaseio\.com|firebasestorage\.googleapis\.com|identitytoolkit\.googleapis\.com|securetoken\.googleapis\.com|firebaseapp\.com|web\.app|firebase\.google\.com|www\.gstatic\.com/firebasejs)",
    )
    .unwrap()
});

static API_KEY_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bAIza[0-9A-Za-z\-_]{35}\b").unwrap());

static PROJECT_ID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:projectId|project_id)\s*[:=]\s*["']([a-z0-9][a-z0-9-]{2,62})["']"#)
        .unwrap()
});

static AUTH_DOMAIN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:authDomain|auth_domain)\s*[:=]\s*["']([a-z0-9.-]+\.firebaseapp\.com)["']"#)
        .unwrap()
});

static FIRESTORE_DB_URL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)https?://(?:firestore\.googleapis\.com/v1/projects/([a-z0-9-]+)/databases/[^/\s"']+|([a-z0-9-]+)\.firebaseio\.com)"#,
    )
    .unwrap()
});

static FIREBASE_CONFIG_BLOB_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?is)(?:firebaseConfig|firebase\.initializeApp\s*\()\s*[=:{]\s*\{[^}]{20,800}\}",
    )
    .unwrap()
});

#[async_trait]
impl Check for FirebaseCheck {
    fn id(&self) -> &'static str {
        "firebase"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut project_ids: Vec<String> = Vec::new();
        let mut api_keys: Vec<String> = Vec::new();
        let mut firestore_urls: Vec<(String, String)> = Vec::new(); // (url, source_page)

        for resp in ctx.responses.values() {
            let body = &resp.body;
            let page = resp.final_url.as_str();

            if FIREBASE_HOST_RE.is_match(body)
                || body.contains("firebase/app")
                || body.contains("firebase/firestore")
                || body.contains("firebase/auth")
                || body.contains("getFirestore")
                || body.contains("initializeFirestore")
                || body.contains("collection(") && body.contains("firebase")
            {
                if seen.insert(format!("sdk:{page}")) {
                    findings.push(
                        Finding::builder(self.id(), "firebase-client-sdk")
                            .title("Firebase / Firestore client usage detected")
                            .severity(Severity::Info)
                            .url(page)
                            .description(
                                "Page or script references Firebase hosts, modular SDK paths, or Firestore APIs.",
                            )
                            .evidence(Evidence::new(
                                "body",
                                snippet_around(body, &["firebase", "firestore", "Firestore"]),
                            ))
                            .build(),
                    );
                }
            }

            for cap in API_KEY_RE.captures_iter(body) {
                let key = cap[0].to_string();
                if api_keys.iter().any(|k| k == &key) {
                    continue;
                }
                api_keys.push(key.clone());
                // Google API keys are often Firebase web keys; flag in firebase module context
                if body.to_ascii_lowercase().contains("firebase")
                    || body.contains("firestore")
                    || body.contains("authDomain")
                    || body.contains("projectId")
                {
                    findings.push(
                        Finding::builder(self.id(), "firebase-api-key")
                            .title("Firebase/Google API key exposed in client assets")
                            .severity(Severity::Low)
                            .url(page)
                            .description(
                                "A Google/Firebase web API key (AIza…) is embedded client-side. Keys are expected for Firebase web apps but must be locked down with App Check, API restrictions, and security rules.",
                            )
                            .remediation(
                                "Restrict the key by HTTP referrer/app, enable App Check, and never rely on the key alone for authorization. Enforce Firestore/RTDB security rules.",
                            )
                            .cwe("CWE-200")
                            .evidence(Evidence::new("apiKey", &key))
                            .build(),
                    );
                }
            }

            for cap in PROJECT_ID_RE.captures_iter(body) {
                let pid = cap[1].to_string();
                if project_ids.iter().any(|p| p == &pid) {
                    continue;
                }
                project_ids.push(pid.clone());
                findings.push(
                    Finding::builder(self.id(), "firebase-project-id")
                        .title(format!("Firebase projectId: {pid}"))
                        .severity(Severity::Info)
                        .url(page)
                        .description(
                            "Client config exposes a Firebase project identifier, which maps to Auth, Firestore, and Storage surfaces.",
                        )
                        .evidence(Evidence::new("projectId", &pid))
                        .build(),
                );
            }

            for cap in AUTH_DOMAIN_RE.captures_iter(body) {
                let domain = cap[1].to_string();
                if seen.insert(format!("authDomain:{domain}")) {
                    findings.push(
                        Finding::builder(self.id(), "firebase-auth-domain")
                            .title(format!("Firebase authDomain: {domain}"))
                            .severity(Severity::Info)
                            .url(page)
                            .description(
                                "authDomain indicates Firebase Authentication is configured for this app.",
                            )
                            .evidence(Evidence::new("authDomain", &domain))
                            .build(),
                    );
                }
            }

            if let Some(m) = FIREBASE_CONFIG_BLOB_RE.find(body) {
                if seen.insert(format!("config:{}", m.as_str().chars().take(40).collect::<String>())) {
                    findings.push(
                        Finding::builder(self.id(), "firebase-config-blob")
                            .title("Firebase config object embedded in client")
                            .severity(Severity::Low)
                            .url(page)
                            .description(
                                "A firebaseConfig / initializeApp object was found. Review which services (Auth, Firestore, Storage) are enabled and whether rules are production-hardened.",
                            )
                            .remediation(
                                "Confirm Firestore/RTDB rules deny by default; enable App Check; restrict API keys; disable unused providers.",
                            )
                            .cwe("CWE-200")
                            .evidence(Evidence::new(
                                "config",
                                m.as_str().chars().take(280).collect::<String>(),
                            ))
                            .build(),
                    );
                }
            }

            // Realtime DB open-rules smell: large JSON dump paths
            if body.contains(".firebaseio.com") && body.contains(".json") {
                if seen.insert(format!("rtdb-json:{page}")) {
                    findings.push(
                        Finding::builder(self.id(), "firebase-rtdb-url")
                            .title("Firebase Realtime Database URL pattern in client")
                            .severity(Severity::Medium)
                            .url(page)
                            .description(
                                "Realtime Database REST URLs (often ending in .json) appear in assets. Misconfigured rules historically exposed entire databases to unauthenticated reads/writes.",
                            )
                            .remediation(
                                "Audit RTDB rules; prefer deny-by-default; migrate sensitive data to Firestore with strong rules.",
                            )
                            .cwe("CWE-306")
                            .evidence(Evidence::new(
                                "body",
                                snippet_around(body, &["firebaseio.com", ".json"]),
                            ))
                            .build(),
                    );
                }
            }

            for cap in FIRESTORE_DB_URL_RE.captures_iter(body) {
                let full = cap.get(0).map(|m| m.as_str().to_string()).unwrap_or_default();
                if full.is_empty() {
                    continue;
                }
                if seen.insert(format!("fsurl:{full}")) {
                    firestore_urls.push((full.clone(), page.to_string()));
                    findings.push(
                        Finding::builder(self.id(), "firestore-endpoint-url")
                            .title("Firestore / Firebase database endpoint URL found")
                            .severity(Severity::Medium)
                            .url(page)
                            .description(
                                "A Firestore REST or Realtime Database endpoint URL is present in client content.",
                            )
                            .remediation(
                                "Ensure security rules and IAM block unauthenticated access; never expose admin SDKs client-side.",
                            )
                            .cwe("CWE-200")
                            .evidence(Evidence::new("url", &full))
                            .build(),
                    );
                }
            }

            // Identity Toolkit / email-password Auth REST references
            if body.contains("identitytoolkit.googleapis.com")
                || body.contains("signInWithPassword")
                || body.contains("accounts:signUp")
                || (body.contains("firebase")
                    && (body.contains("createUserWithEmailAndPassword")
                        || body.contains("signInWithEmailAndPassword")))
            {
                if seen.insert(format!("identity:{page}")) {
                    findings.push(
                        Finding::builder(self.id(), "firebase-auth-rest")
                            .title("Firebase Auth / Identity Toolkit surface referenced")
                            .severity(Severity::Info)
                            .url(page)
                            .description(
                                "Client references Firebase Auth (Identity Toolkit). Signup/login is unauthenticated by design; rate-limit and abuse protection matter.",
                            )
                            .remediation(
                                "Enable App Check, email enumeration protection, and backend rate limits on Auth; review authorized domains.",
                            )
                            .build(),
                    );
                }
            }
        }

        // Probe discovered Firebase-ish absolute URLs if they fall in allowlist scope
        // (usually they won't — firestore.googleapis.com is third-party).
        // Also synthesize likely project endpoints for informational findings only when
        // project id known — do not hammer Google APIs.
        for pid in project_ids.iter().take(3) {
            if seen.insert(format!("project-summary:{pid}")) {
                findings.push(
                    Finding::builder(self.id(), "firestore-project-surface")
                        .title(format!("Firestore surface map for project `{pid}`"))
                        .severity(Severity::Info)
                        .url(ctx.seed.as_str())
                        .description(format!(
                            "Derived surfaces: \
                             https://firestore.googleapis.com/v1/projects/{pid}/databases/(default)/documents · \
                             https://{pid}.firebaseio.com/.json · \
                             https://identitytoolkit.googleapis.com/v1/accounts:signUp · \
                             https://{pid}.firebaseapp.com"
                        ))
                        .remediation(
                            "Validate security rules with Firebase Emulator Suite and production rule unit tests; confirm App Check enforcement.",
                        )
                        .build(),
                );
            }

            // In-scope only: if seed host itself is *.firebaseapp.com / web.app
            let host = ctx.seed.host_str().unwrap_or("");
            if host.contains("firebaseapp.com") || host.ends_with(".web.app") {
                let probe = format!("https://{pid}.firebaseio.com/.json");
                if let Ok(u) = Url::parse(&probe) {
                    // only if authz allows — use client which enforces scope
                    if let Ok(resp) = ctx.client.get(&u).await {
                        let st = resp.status.as_u16();
                        if (200..300).contains(&st) && !resp.body.trim().is_empty() {
                            findings.push(
                                Finding::builder(self.id(), "firebase-rtdb-open-read")
                                    .title("Firebase Realtime Database appears readable without auth")
                                    .severity(Severity::Critical)
                                    .url(u.as_str())
                                    .description(
                                        "GET on the RTDB root (.json) returned data without authentication — classic open rules misconfiguration.",
                                    )
                                    .remediation(
                                        "Immediately lock RTDB rules to deny unauthenticated access; rotate any exposed secrets.",
                                    )
                                    .cwe("CWE-306")
                                    .evidence(Evidence::new(
                                        "body",
                                        resp.body.chars().take(200).collect::<String>(),
                                    ))
                                    .build(),
                            );
                        } else if st == 401 || st == 403 {
                            findings.push(
                                Finding::builder(self.id(), "firebase-rtdb-denied")
                                    .title("Firebase Realtime Database root denied (rules present)")
                                    .severity(Severity::Info)
                                    .url(u.as_str())
                                    .description(format!(
                                        "Unauthenticated GET returned HTTP {st}."
                                    ))
                                    .build(),
                            );
                        }
                    }
                }
            }
        }

        // Probe in-scope paths that look like Firestore proxies
        for asset in &ctx.assets {
            let path = asset.url.path().to_ascii_lowercase();
            if path.contains("firestore")
                || path.contains("/firebase")
                || path.ends_with("/__/auth/handler")
            {
                if seen.insert(format!("path:{}", asset.url)) {
                    let sev = if (200..300).contains(&asset.status) {
                        Severity::Medium
                    } else {
                        Severity::Info
                    };
                    findings.push(
                        Finding::builder(self.id(), "firestore-path")
                            .title("Firestore/Firebase-related path on target")
                            .severity(sev)
                            .url(asset.url.as_str())
                            .description(format!(
                                "Path suggests Firebase/Firestore proxy or auth handler (HTTP {}).",
                                asset.status
                            ))
                            .build(),
                    );
                }
            }
        }

        // Weakness summary if using Firestore but no rate-limit findings elsewhere —
        // keep as informational checklist.
        let using = findings.iter().any(|f| {
            matches!(
                f.id.as_str(),
                "firebase-client-sdk"
                    | "firebase-config-blob"
                    | "firebase-project-id"
                    | "firestore-endpoint-url"
            )
        });
        if using {
            findings.push(
                Finding::builder(self.id(), "firestore-weakness-checklist")
                    .title("Firestore/Firebase weakness review checklist")
                    .severity(Severity::Info)
                    .url(ctx.seed.as_str())
                    .description(
                        "When Firestore is in use, review: (1) security rules deny-by-default, \
                         (2) App Check enforced, (3) API key HTTP referrer restrictions, \
                         (4) Auth providers rate-limited / enumeration protected, \
                         (5) no admin SDK credentials in client bundles, \
                         (6) Storage rules separate from Firestore, \
                         (7) emulators not exposed publicly.",
                    )
                    .remediation(
                        "Run rules unit tests in CI; enable Firebase App Check; audit IAM for service accounts.",
                    )
                    .build(),
            );
        }

        let _ = firestore_urls;
        let _ = api_keys;
        Ok(findings)
    }
}

fn snippet_around(body: &str, needles: &[&str]) -> String {
    let lower = body.to_ascii_lowercase();
    for n in needles {
        if let Some(idx) = lower.find(&n.to_ascii_lowercase()) {
            let start = idx.saturating_sub(40);
            let end = (idx + n.len() + 80).min(body.len());
            return body[start..end].chars().take(200).collect();
        }
    }
    body.chars().take(120).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_api_key_pattern() {
        // AIza + exactly 35 url-safe chars
        let key = format!("AIza{}", "x".repeat(35));
        assert!(API_KEY_RE.is_match(&key), "key={key}");
        assert!(API_KEY_RE.is_match(&format!(r#"apiKey: "{key}""#)));
    }

    #[test]
    fn detects_project_id() {
        let s = r#"projectId: "my-cool-app-123""#;
        let cap = PROJECT_ID_RE.captures(s).unwrap();
        assert_eq!(&cap[1], "my-cool-app-123");
    }

    #[test]
    fn detects_firestore_host() {
        assert!(FIREBASE_HOST_RE.is_match("https://firestore.googleapis.com/v1/projects/x"));
    }
}
