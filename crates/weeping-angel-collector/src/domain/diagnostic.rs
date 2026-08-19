/// Internal diagnostic kinds. Public `CollectionBatch.errors` stays `Vec<String>`.

#[derive(Debug, Clone)]
pub struct CollectionDiagnostic {
    pub message: String,
}

impl CollectionDiagnostic {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
