use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// Shikoku Electric Power Transmission & Distribution — outage information.
/// Voltage dips and short-duration outages excluded from public page.
/// Updates ~5 min; major disasters may cause longer delays.
pub struct ShikokuEpco;

impl ShikokuEpco {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShikokuEpco {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for ShikokuEpco {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "shikoku-epco".into(),
            org_name: "四国電力送配電株式会社".into(),
            family: Family::Power,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 300,
            canonical_url: "https://www.yonden.co.jp/nw/teiden/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://www.yonden.co.jp/nw/teiden/"
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
