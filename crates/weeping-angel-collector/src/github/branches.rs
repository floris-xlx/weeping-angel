pub const MODULE: &str = "branches";

/// Protection/ruleset path uses this name, never a hardcoded branch.
pub fn protection_path(owner: &str, name: &str, default_branch: &str) -> String {
    format!("/repos/{owner}/{name}/branches/{default_branch}/protection")
}
