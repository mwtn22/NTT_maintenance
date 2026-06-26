use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// NTT East — construction / fault information (telecom, public, general).
pub struct NttEast;

impl NttEast {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NttEast {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for NttEast {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "ntt-east".into(),
            org_name: "東日本電信電話株式会社".into(),
            family: Family::Telecom,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 600,
            canonical_url: "https://www.ntt-east.co.jp/info-st/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://www.ntt-east.co.jp/info-st/"
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
