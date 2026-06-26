use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// Okinawa Electric Power — outage information.
/// Typhoon-heavy operational profile; update frequency may degrade during
/// major weather events. Voltage sag excluded from public page.
pub struct OkinawaEpco;

impl OkinawaEpco {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OkinawaEpco {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for OkinawaEpco {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "okinawa-epco".into(),
            org_name: "沖縄電力株式会社".into(),
            family: Family::Power,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 300,
            canonical_url: "https://www.okiden.co.jp/teiden/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://www.okiden.co.jp/teiden/"
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
