use std::collections::{BTreeMap, HashSet};
use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;

use super::map_to_vec;
use crate::depcheck::types::{Ecosystem, PackageRef};

pub fn parse_go_mod(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    let mut replaced = HashSet::new();
    let mut in_require = false;
    let mut in_replace = false;

    static REQ_BLOCK: OnceLock<Regex> = OnceLock::new();
    let req_block = REQ_BLOCK.get_or_init(|| Regex::new(r"^require\s*\(").expect("require block"));
    static REP_BLOCK: OnceLock<Regex> = OnceLock::new();
    let rep_block = REP_BLOCK.get_or_init(|| Regex::new(r"^replace\s*\(").expect("replace block"));
    static REQ_LINE: OnceLock<Regex> = OnceLock::new();
    let req_line = REQ_LINE.get_or_init(|| Regex::new(r"^require\s+\S").expect("require line"));
    static REP_LINE: OnceLock<Regex> = OnceLock::new();
    let rep_line = REP_LINE.get_or_init(|| Regex::new(r"^replace\s+\S").expect("replace line"));
    static DOMAIN: OnceLock<Regex> = OnceLock::new();
    let domain = DOMAIN.get_or_init(|| Regex::new(r"^[a-zA-Z0-9]+\.[a-zA-Z]").expect("domain"));

    for line in content.lines() {
        let s = line.trim();
        if req_block.is_match(s) {
            in_require = true;
            in_replace = false;
            continue;
        }
        if rep_block.is_match(s) {
            in_replace = true;
            in_require = false;
            continue;
        }
        if s == ")" {
            in_require = false;
            in_replace = false;
            continue;
        }

        if in_replace || rep_line.is_match(s) {
            let rep = if in_replace {
                s.to_string()
            } else {
                s.replacen("replace ", "", 1)
            };
            let rep = rep.split("//").next().unwrap_or("").trim();
            if let Some((left, right)) = rep.split_once("=>") {
                let right_parts: Vec<&str> = right.trim().split_whitespace().collect();
                if let Some(first) = right_parts.first() {
                    if !domain.is_match(first) {
                        if let Some(mod_name) = left.trim().split_whitespace().next() {
                            replaced.insert(mod_name.to_string());
                        }
                    }
                }
            }
            continue;
        }

        let target = if in_require {
            Some(s.to_string())
        } else if req_line.is_match(s) {
            Some(s.replacen("require ", "", 1))
        } else {
            None
        };

        if let Some(target) = target {
            let target = target.split("//").next().unwrap_or("").trim();
            let parts: Vec<&str> = target.split_whitespace().collect();
            if parts.len() >= 2 && !parts[0].starts_with("//") {
                packages.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }

    for mod_name in replaced {
        packages.remove(&mod_name);
    }

    Ok((map_to_vec(packages), Ecosystem::Go))
}
