use thiserror::Error;

/// Business-meaningful errors for the fetch/parse pipeline.
///
/// These mirror the error taxonomy in the plan so that failures can be
/// traced to a specific cause (network vs. DOM change vs. policy) rather
/// than being collapsed into an opaque string.
#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("network error fetching {url}: {source}")]
    Network {
        url: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("parse error: {0}")]
    Parse(String),

    /// The page loaded but an expected CSS selector / structure was missing.
    /// Distinct from `Parse` so DOM drift can be alerted on specifically.
    #[error("schema mismatch (DOM likely changed): {0}")]
    SchemaMismatch(String),

    #[error("retrieval denied by policy/terms: {0}")]
    PolicyDenied(String),

    #[error("rate limited by source: {0}")]
    RateLimited(String),

    #[error("fixture error: {0}")]
    Fixture(String),
}
