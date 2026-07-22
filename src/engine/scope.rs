use url::Url;

use crate::authz::Authorization;

/// Normalize URL for crawl de-duplication: drop fragment, optional sort query later.
pub fn normalize_url(url: &Url) -> Url {
    let mut u = url.clone();
    u.set_fragment(None);
    u
}

pub fn resolve_link(base: &Url, href: &str) -> Option<Url> {
    let href = href.trim();
    if href.is_empty()
        || href.starts_with('#')
        || href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("javascript:")
        || href.starts_with("data:")
    {
        return None;
    }
    base.join(href).ok().map(|u| normalize_url(&u))
}

pub fn in_scope(authz: &Authorization, url: &Url) -> bool {
    authz.url_in_scope(url)
}

pub fn same_host(a: &Url, b: &Url) -> bool {
    a.host_str() == b.host_str()
}
