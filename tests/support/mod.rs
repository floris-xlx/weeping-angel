// Crate-private needle + filesystem helpers for contract dual-suite binaries.

#[allow(dead_code)]
fn require_needles(label: &str, src: &str, needles: &[&str]) {
    let missing: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| !src.contains(n))
        .collect();
    assert!(
        missing.is_empty(),
        "{label}: missing required surface {missing:?}"
    );
}

#[allow(dead_code)]
fn manifest_dir() -> std::path::PathBuf {
    repo_root()
}

#[allow(dead_code)]
fn repo_root() -> std::path::PathBuf {
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..4 {
        if dir.join("xtask").join("Cargo.toml").is_file()
            && dir.join("architecture").join("architecture.toml").is_file()
        {
            return dir;
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => break,
        }
    }
    panic!(
        "repo root not found from apps/cli CARGO_MANIFEST_DIR {}",
        env!("CARGO_MANIFEST_DIR")
    );
}

#[allow(dead_code)]
fn read_repo_file(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[allow(dead_code)]
fn harness_src() -> String {
    read_repo_file("apps/cli/tests/harness.rs")
}

#[allow(dead_code)]
fn assert_suite_in_harness(contract_file: &str) {
    let harness = harness_src();
    assert!(
        harness.contains(contract_file),
        "{contract_file} must remain a harness module"
    );
}

#[allow(dead_code)]
fn sdd_suite_wired(name: &str) -> bool {
    let harness = harness_src();
    if harness.contains(name) {
        return true;
    }
    let stem = name.strip_prefix("sdd_").unwrap_or(name);
    let stem = stem.strip_suffix("_target").unwrap_or(stem);
    harness.contains(&format!("{stem}.target.rs")) || harness.contains(&format!("{stem}.rs"))
}

#[allow(dead_code)]
fn crate_sources_joined(name: &str) -> String {
    fn walk_rs_tree(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let entries =
            std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                walk_rs_tree(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }

    let src = repo_root().join("crates").join(name).join("src");
    assert!(
        src.is_dir(),
        "expected crate sources at {}",
        src.display()
    );
    let mut files = Vec::new();
    walk_rs_tree(&src, &mut files);
    files
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(dead_code)]
fn text_has(haystack: &str, needle: &str) -> bool {
    haystack.find(needle).is_some()
}
