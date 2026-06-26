use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// Hokuriku Electric Power Transmission & Distribution — outage information.
/// Outages under ~5 minutes and momentary voltage dips excluded from public page.
/// Special-high-voltage customers have a separate notification service.
pub struct HokurikuEpco;

impl HokurikuEpco {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HokurikuEpco {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for HokurikuEpco {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "hokuriku-epco".into(),
            org_name: "北陸電力送配電株式会社".into(),
            family: Family::Power,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 300,
            canonical_url: "https://www.rikuden.co.jp/nw/teiden/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://www.rikuden.co.jp/nw/teiden/"
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
