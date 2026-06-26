use crate::parse_util::{parse_events, ParseDefaults};
use crate::{RawDocument, Source};
use domain::{
    ConnectorError, CustomerScope, EventKind, Family, NormalizedEvent, PublicationScope,
    RetrievalMode, SourceMeta, SourceType, VisibilityStatus,
};

/// Tokyo Electric Power Grid — outage and voltage-sag history.
///
/// The public outage page covers general customers. The voltage-sag history
/// page targets special-high-voltage (66 kV+) contract areas only; those
/// events carry `customer_scope: special_high_voltage` and
/// `visibility_status: public` (it is a publicly listed page, but the scope
/// is limited — which is encoded in `customer_scope`, not `visibility_status`).
pub struct TepcoPg;

impl TepcoPg {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TepcoPg {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for TepcoPg {
    fn meta(&self) -> SourceMeta {
        SourceMeta {
            org_code: "tepco-pg".into(),
            org_name: "東京電力パワーグリッド株式会社".into(),
            family: Family::Power,
            source_type: SourceType::Html,
            publication_scope: PublicationScope::Public,
            retrieval_mode: RetrievalMode::Http,
            poll_interval_sec: 300,
            canonical_url: "https://teiden.tepco.co.jp/teiden/".into(),
        }
    }

    fn fetch_url(&self) -> &str {
        "https://teiden.tepco.co.jp/teiden/"
    }

    fn parse(&self, raw: &RawDocument) -> Result<Vec<NormalizedEvent>, ConnectorError> {
        // The fixture mixes general outages and special-high-voltage voltage sag.
        // `data-customer` on each event node carries the correct scope; the
        // parser reads it and the defaults below serve as fallback.
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
