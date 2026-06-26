use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// Chugoku Electric Power Network — outage information (~5 min updates).
/// Voltage dip excluded from public outage page; available only via special-HV
/// email notification service for contracted customers.
pub struct ChugokuEpco;

impl ChugokuEpco {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ChugokuEpco {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for ChugokuEpco {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "chugoku-epco".into(),
            org_name: "中国電力ネットワーク株式会社".into(),
            family: Family::Power,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 300,
            canonical_url: "https://www.energia.co.jp/nw/teiden/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://www.energia.co.jp/nw/teiden/"
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
