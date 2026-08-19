use serde_json::Value;

use super::client::{GitHubClient, PageWalk};
use super::normalize::{archived_of, default_branch_of};

pub const MODULE: &str = "repositories";

pub struct ListedRepo {
    pub owner: String,
    pub name: String,
    pub json: Value,
    pub archived: bool,
    pub default_branch: Option<String>,
}

pub fn parse_listed_repo(item: &Value, org_fallback: &str) -> Option<ListedRepo> {
    let full = item.get("full_name").and_then(Value::as_str);
    let (owner, name) = if let Some(full) = full {
        full.split_once('/')?
    } else {
        let name = item.get("name").and_then(Value::as_str)?;
        (org_fallback, name)
    };
    let listed = ListedRepo {
        owner: owner.to_string(),
        name: name.to_string(),
        json: item.clone(),
        archived: archived_of(item),
        default_branch: default_branch_of(item).map(str::to_string),
    };
    let _ = (listed.archived, listed.default_branch.as_deref());
    Some(listed)
}

pub fn list_org_repos(client: &GitHubClient, org: &str) -> PageWalk {
    let _ = super::client::DEFAULT_PER_PAGE;
    client.get_pages(&format!("/orgs/{org}/repos"))
}
