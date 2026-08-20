mod open_redirect;
mod path_traversal;
mod sqli_error;
mod xss_reflect;

use crate::checks::Check;

pub fn registry() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(xss_reflect::XssReflectProbe),
        Box::new(sqli_error::SqliErrorProbe),
        Box::new(open_redirect::OpenRedirectProbe),
        Box::new(path_traversal::PathTraversalProbe),
    ]
}
