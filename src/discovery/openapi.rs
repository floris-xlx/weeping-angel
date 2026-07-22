use regex::Regex;
use url::Url;

pub fn candidate_urls(seed: &Url) -> Vec<Url> {
    const PATHS: &[&str] = &[
        "/openapi.json",
        "/openapi.yaml",
        "/swagger.json",
        "/swagger/v1/swagger.json",
        "/v2/api-docs",
        "/v3/api-docs",
        "/api-docs",
        "/api/openapi.json",
        "/api/swagger.json",
    ];
    let mut out = Vec::new();
    for p in PATHS {
        let mut u = seed.clone();
        u.set_path(p);
        u.set_query(None);
        out.push(u);
    }
    out
}

pub fn extract_paths(seed: &Url, body: &str) -> Vec<Url> {
    // paths in OpenAPI often appear as "/foo/bar": {
    let re = Regex::new(r#""(/[A-Za-z0-9._\-{}/]+)"\s*:\s*\{"#).unwrap();
    let mut out = Vec::new();
    for cap in re.captures_iter(body) {
        let path = &cap[1];
        if path.contains('{') {
            // skip templated for GET discovery or strip
            let simplified = Regex::new(r"\{[^}]+\}")
                .unwrap()
                .replace_all(path, "1");
            if let Ok(u) = seed.join(&simplified) {
                out.push(u);
            }
        } else if let Ok(u) = seed.join(path) {
            out.push(u);
        }
    }
    out
}
