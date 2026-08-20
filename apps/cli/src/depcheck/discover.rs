//! Recursive discovery of dependency manifests.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::detect::detect_file_type;
use super::types::FileKind;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "__pycache__",
    "vendor",
    ".venv",
    "venv",
    "target",
    "dist",
    "build",
    ".idea",
    ".vs",
];

/// Find dependency files under `root` (sorted).
pub fn find_dep_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let walker = WalkDir::new(root).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            return !SKIP_DIRS.iter().any(|s| *s == name);
        }
        true
    });

    for entry in walker.flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let kind = detect_file_type(path, None);
        if kind != FileKind::Unknown {
            found.push(path.to_path_buf());
        }
    }
    found.sort();
    found
}
