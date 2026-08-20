use std::collections::BTreeMap;

use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;

use super::map_to_vec;
use crate::depcheck::types::{Ecosystem, PackageRef};

pub fn parse_packages_config(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let local = local_name(e.name().as_ref());
                if local == "package" {
                    let mut id = None;
                    let mut version = "*".to_string();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(attr.value.as_ref()).into_owned();
                        match key.as_str() {
                            "id" => id = Some(val),
                            "version" => version = val,
                            _ => {}
                        }
                    }
                    if let Some(name) = id
                        && !name.is_empty()
                    {
                        packages.insert(name, version);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok((map_to_vec(packages), Ecosystem::Nuget))
}

pub fn parse_csproj(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let local = local_name(e.name().as_ref());
                if local == "PackageReference" {
                    let mut include = None;
                    let mut version = "*".to_string();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let key = key.rsplit('}').next().unwrap_or(&key);
                        let val = String::from_utf8_lossy(attr.value.as_ref()).into_owned();
                        match key {
                            "Include" => include = Some(val),
                            "Version" => version = val,
                            _ => {}
                        }
                    }
                    if let Some(name) = include
                        && !name.is_empty()
                    {
                        packages.insert(name, version);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok((map_to_vec(packages), Ecosystem::Nuget))
}

fn local_name(name: &[u8]) -> String {
    let s = String::from_utf8_lossy(name);
    s.rsplit('}').next().unwrap_or(&s).to_string()
}
