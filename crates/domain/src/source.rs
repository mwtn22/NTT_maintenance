use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Whether a source belongs to the telecom or power family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Telecom,
    Power,
}

impl Family {
    pub fn as_str(&self) -> &'static str {
        match self {
            Family::Telecom => "telecom",
            Family::Power => "power",
        }
    }
}

/// How the source is published / what raw format it exposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Html,
    Json,
    Feed,
    AppWeb,
}

/// Visibility of the data a source publishes. Critical for voltage-sag, where
/// publication granularity differs nationwide (public vs. special-high-voltage
/// only vs. excluded from the public outage page).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PublicationScope {
    Public,
    Restricted,
    SpecialHighVoltageOnly,
}

/// How we technically retrieve the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RetrievalMode {
    Http,
    Headless,
    ManualBackfill,
}

/// Static metadata describing a single source/provider.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SourceMeta {
    pub org_code: String,
    pub org_name: String,
    pub family: Family,
    pub source_type: SourceType,
    pub publication_scope: PublicationScope,
    pub retrieval_mode: RetrievalMode,
    pub poll_interval_sec: i32,
    pub canonical_url: String,
}
