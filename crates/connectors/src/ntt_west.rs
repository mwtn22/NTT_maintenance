use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// NTT West — construction / fault information (telecom, public, general).
pub struct NttWest;

impl NttWest {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NttWest {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for NttWest {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "ntt-west".into(),
            org_name: "西日本電信電話株式会社".into(),
            family: Family::Telecom,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 600,
            canonical_url: "https://www.ntt-west.co.jp/important/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://www.ntt-west.co.jp/important/"
    }

    fn parse(&self, raw: &RawDocument) -> Result<Vec<NormalizedEvent>, ConnectorError> {
        parse_events(
            &raw.body,
            &ParseDefaults {
                customer_scope: CustomerScope::General,
                visibility_status: VisibilityStatus::Public,
                default_kind: EventKind::Construction,
            },
        )
    }
}
