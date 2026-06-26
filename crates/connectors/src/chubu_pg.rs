use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// Chubu Electric Power Grid — outage information (public, general).
/// Momentary voltage dips and sub-second outages excluded from public page.
/// Special-high-voltage customers have a separate web/email notification service.
pub struct ChubuPg;

impl ChubuPg {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ChubuPg {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for ChubuPg {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "chubu-pg".into(),
            org_name: "中部電力パワーグリッド株式会社".into(),
            family: Family::Power,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 300,
            canonical_url: "https://www.chuden.co.jp/network/teiden/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://www.chuden.co.jp/network/teiden/"
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
