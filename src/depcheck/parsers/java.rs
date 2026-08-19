use std::collections::BTreeMap;
use std::sync::OnceLock;

use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;

use super::map_to_vec;
use crate::depcheck::types::{Ecosystem, PackageRef};

pub fn parse_pom_xml(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut in_dependency = false;
    let mut group_id = String::new();
    let mut artifact_id = String::new();
    let mut version = String::new();
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let local = local_name(&e.name().as_ref());
                if local == "dependency" {
                    in_dependency = true;
                    group_id.clear();
                    artifact_id.clear();
                    version.clear();
                } else if in_dependency {
                    current_tag = local;
                }
            }
            Ok(Event::Text(t)) => {
                if in_dependency && !current_tag.is_empty() {
                    let text = t.decode().map(|c| c.into_owned()).unwrap_or_default();
                    match current_tag.as_str() {
                        "groupId" => group_id = text,
                        "artifactId" => artifact_id = text,
                        "version" => version = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let local = local_name(&e.name().as_ref());
                if local == "dependency" {
                    if !group_id.is_empty() && !artifact_id.is_empty() {
                        let key = format!("{group_id}:{artifact_id}");
                        let ver = if version.is_empty() {
                            "*".into()
                        } else {
                            version.clone()
                        };
                        packages.insert(key, ver);
                    }
                    in_dependency = false;
                    current_tag.clear();
                } else if in_dependency && local == current_tag {
                    current_tag.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok((map_to_vec(packages), Ecosystem::Maven))
}

fn local_name(name: &[u8]) -> String {
    let s = String::from_utf8_lossy(name);
    s.rsplit('}').next().unwrap_or(&s).to_string()
}

pub fn parse_build_gradle(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    static PAT: OnceLock<Regex> = OnceLock::new();
    let pat = PAT.get_or_init(|| {
        Regex::new(
            r#"(?:implementation|compile|api|runtimeOnly|testImplementation|compileOnly|testCompileOnly|annotationProcessor)\s*[\(]?\s*['"]([^'"]+):([^'"]+):([^'"]*?)['"]"#,
        )
        .expect("gradle dep") // panic-ok: regex literal
    });
    for c in pat.captures_iter(content) {
        packages.insert(format!("{}:{}", &c[1], &c[2]), c[3].to_string());
    }
    Ok((map_to_vec(packages), Ecosystem::Maven))
}
