use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// Kyushu Electric Power Transmission & Distribution — outage information.
/// Covers outages of ~5+ minutes. No general-public voltage-sag history;
/// 6kV/22kV contracted customers have a separate email notification service.
pub struct KyushuEpco;

impl KyushuEpco {
    pub fn new() -> Self {
        Self
    }
}

impl Default for KyushuEpco {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for KyushuEpco {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "kyushu-epco".into(),
            org_name: "九州電力送配電株式会社".into(),
            family: Family::Power,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 300,
            canonical_url: "https://www.kyuden.co.jp/td/teiden/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://www.kyuden.co.jp/td/teiden/"
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
