//! Lightweight Nuclei-style HTTP path templates (YAML).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use url::Url;

use crate::finding::{Evidence, Finding, Severity};
use crate::http::HttpClient;

#[derive(Debug, Clone, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub remediation: Option<String>,
    #[serde(default)]
    pub cwe: Option<String>,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub matchers: Vec<Matcher>,
}

fn default_severity() -> String {
    "medium".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Matcher {
    #[serde(rename = "type", default = "default_matcher_kind")]
    pub kind: String,
    #[serde(default)]
    pub status: Vec<u16>,
    #[serde(default)]
    pub contains: Vec<String>,
    #[serde(default)]
    pub regex: Vec<String>,
}

fn default_matcher_kind() -> String {
    "body".into()
}

static WORD_SPLIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

impl Template {
    pub fn severity_enum(&self) -> Severity {
        Severity::from_str_loose(&self.severity).unwrap_or(Severity::Medium)
    }

    pub fn matches(&self, status: u16, body: &str) -> bool {
        if self.matchers.is_empty() {
            return status == 200;
        }
        self.matchers.iter().all(|m| m.matches(status, body))
    }
}

impl Matcher {
    fn matches(&self, status: u16, body: &str) -> bool {
        match self.kind.as_str() {
            "status" => {
                if self.status.is_empty() {
                    true
                } else {
                    self.status.contains(&status)
                }
            }
            "body" | _ => {
                let contains_ok = self.contains.is_empty()
                    || self.contains.iter().any(|c| body.contains(c.as_str()));
                let regex_ok = if self.regex.is_empty() {
                    true
                } else {
                    self.regex
                        .iter()
                        .any(|pat| Regex::new(pat).map(|re| re.is_match(body)).unwrap_or(false))
                };
                // if both empty under body, require 200 via status matcher elsewhere
                if self.contains.is_empty() && self.regex.is_empty() {
                    true
                } else {
                    contains_ok && regex_ok
                }
            }
        }
    }
}

pub fn default_templates_dir() -> PathBuf {
    PathBuf::from("templates")
}

pub fn load_templates(dir: &Path) -> Result<Vec<Template>> {
    if !dir.exists() {
        return Ok(embedded_templates());
    }
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x == "yaml" || x == "yml")
        })
    {
        let raw = std::fs::read_to_string(entry.path())
            .with_context(|| format!("read template {}", entry.path().display()))?;
        let t: Template = serde_yaml::from_str(&raw)
            .with_context(|| format!("parse template {}", entry.path().display()))?;
        out.push(t);
    }
    if out.is_empty() {
        return Ok(embedded_templates());
    }
    Ok(out)
}

/// Built-in fallbacks if templates/ is missing (e.g. installed binary).
pub fn embedded_templates() -> Vec<Template> {
    let yamls = [
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/exposed-env.yaml"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/git-exposed.yaml"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/swagger-exposed.yaml"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/spring-actuator.yaml"
        )),
    ];
    yamls
        .iter()
        .filter_map(|y| serde_yaml::from_str(y).ok())
        .collect()
}

pub async fn run_templates(
    client: &HttpClient,
    seed: &Url,
    templates: &[Template],
    max_paths: usize,
) -> Result<Vec<Finding>> {
    use futures::stream::{self, StreamExt};

    let origin = {
        let mut u = seed.clone();
        u.set_path("/");
        u.set_query(None);
        u.set_fragment(None);
        u
    };

    // Flatten (template_idx, path) jobs up to max_paths
    let mut jobs: Vec<(usize, String, Url)> = Vec::new();
    'outer: for (ti, t) in templates.iter().enumerate() {
        for path in &t.paths {
            if jobs.len() >= max_paths {
                break 'outer;
            }
            let mut url = origin.clone();
            let p = if path.starts_with('/') {
                path.clone()
            } else {
                format!("/{path}")
            };
            url.set_path(&p);
            jobs.push((ti, path.clone(), url));
        }
    }

    let concurrency = client.concurrency().max(1);
    let results = stream::iter(jobs.into_iter().map(|(ti, _path, url)| {
        let client = client.clone();
        async move {
            let resp = client.get(&url).await.ok();
            (ti, url, resp)
        }
    }))
    .buffer_unordered(concurrency)
    .collect::<Vec<_>>()
    .await;

    let mut findings = Vec::new();
    for (ti, url, resp) in results {
        let Some(resp) = resp else { continue };
        let t = &templates[ti];
        if t.matches(resp.status.as_u16(), &resp.body) {
            let mut b = Finding::builder("templates", &t.id)
                .title(&t.name)
                .severity(t.severity_enum())
                .url(url.as_str())
                .description(if t.description.is_empty() {
                    format!("Template `{}` matched.", t.id)
                } else {
                    t.description.clone()
                });
            if let Some(r) = &t.remediation {
                b = b.remediation(r);
            }
            if let Some(c) = &t.cwe {
                b = b.cwe(c);
            }
            b = b.evidence(Evidence::new(
                "body",
                resp.body.chars().take(160).collect::<String>(),
            ));
            findings.push(b.build());
        }
    }
    let _ = WORD_SPLIT;
    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_embedded() {
        let t = embedded_templates();
        assert!(t.len() >= 3);
        assert!(t.iter().any(|x| x.id.contains("env")));
    }

    #[test]
    fn matcher_body_regex() {
        let m = Matcher {
            kind: "body".into(),
            status: vec![],
            contains: vec![],
            regex: vec![r"(?i)ref:\s*refs/".into()],
        };
        assert!(m.matches(200, "ref: refs/heads/main\n"));
    }
}
