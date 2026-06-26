use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// Tohoku Electric Power Network — outage information (public, general).
/// Voltage sag excluded from public page. ~5 min updates, up to 20 min to appear.
/// Monday morning scheduled maintenance window.
pub struct TohokuEpco;

impl TohokuEpco {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TohokuEpco {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for TohokuEpco {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "tohoku-epco".into(),
            org_name: "東北電力ネットワーク株式会社".into(),
            family: Family::Power,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 300,
            canonical_url: "https://nw.tohoku-epco.co.jp/teiden/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://nw.tohoku-epco.co.jp/teiden/"
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
