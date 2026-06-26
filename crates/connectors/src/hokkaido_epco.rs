use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// Hokkaido Electric Power Network — outage information (public, general).
/// Voltage sag excluded from public page.
pub struct HokkaidoEpco;

impl HokkaidoEpco {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HokkaidoEpco {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for HokkaidoEpco {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "hokkaido-epco".into(),
            org_name: "北海道電力ネットワーク株式会社".into(),
            family: Family::Power,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 300,
            canonical_url: "https://www.hepco.co.jp/network/teiden/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://www.hepco.co.jp/network/teiden/"
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
