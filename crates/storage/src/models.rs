use chrono::{DateTime, Utc};
use domain::EventArea;
use serde::Serialize;
use uuid::Uuid;

/// Read model for an event row plus its provider and areas.
#[derive(Debug, Clone, Serialize)]
pub struct EventRecord {
    pub event_id: Uuid,
    pub family: String,
    pub org_code: String,
    pub org_name: String,
    pub event_kind: String,
    pub status: String,
    pub publication_scope: String,
    pub customer_scope: String,
    pub service_category: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub cause_text: Option<String>,
    pub source_updated_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub recovery_estimate_at: Option<DateTime<Utc>>,
    pub impact_count: Option<i32>,
    pub impact_unit: Option<String>,
    pub visibility_status: String,
    pub original_url: String,
    pub areas: Vec<EventArea>,
}

/// Filters accepted by the events list/summary/geojson queries.
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub family: Option<String>,
    pub event_kind: Option<String>,
    pub status: Option<String>,
    pub pref_code: Option<String>,
    pub municipality_code: Option<String>,
    pub q: Option<String>,
    pub started_from: Option<DateTime<Utc>>,
    pub started_to: Option<DateTime<Utc>>,
    pub limit: i64,
    pub offset: i64,
}

impl EventFilter {
    pub fn with_paging(limit: i64, offset: i64) -> Self {
        Self {
            limit,
            offset,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PrefCount {
    pub pref_code: Option<String>,
    pub pref_name: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryRecord {
    pub total: i64,
    pub by_pref: Vec<PrefCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceRecord {
    pub source_id: Uuid,
    pub org_code: String,
    pub org_name: String,
    pub family: String,
    pub source_type: String,
    pub publication_scope: String,
    pub retrieval_mode: String,
    pub poll_interval_sec: i32,
    pub enabled: bool,
    pub canonical_url: String,
}

/// Per-source freshness: distinguishes official update time from our fetch time.
#[derive(Debug, Clone, Serialize)]
pub struct FreshnessRecord {
    pub source_id: Uuid,
    pub org_code: String,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_source_updated_at: Option<DateTime<Utc>>,
    pub lag_sec: Option<i64>,
}
