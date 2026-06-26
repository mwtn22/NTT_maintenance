use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// Kansai Transmission & Distribution — outage and the publicly-listed
/// voltage-sag history (power, public, general). One of the few operators
/// that publishes a general-public voltage-sag list.
pub struct KansaiTd;

impl KansaiTd {
    pub fn new() -> Self {
        Self
    }
}

impl Default for KansaiTd {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for KansaiTd {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "kansai-td".into(),
            org_name: "関西電力送配電株式会社".into(),
            family: Family::Power,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 300,
            canonical_url: "https://www.kansai-td.co.jp/teiden/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://www.kansai-td.co.jp/teiden/"
    }

    fn parse(&self, raw: &RawDocument) -> Result<Vec<NormalizedEvent>, ConnectorError> {
        parse_events(
            &raw.body,
            &ParseDefaults {
                customer_scope: CustomerScope::General,
                visibility_status: VisibilityStatus::Public,
                default_kind: EventKind::Outage,
            },
        )
    }
}
