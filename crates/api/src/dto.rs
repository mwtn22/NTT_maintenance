//! API response shapes (mirrors the JSON examples in the plan).

use chrono::{DateTime, Utc};
use domain::EventArea;
use serde::Serialize;
use storage::EventRecord;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct Provider {
    pub org_code: String,
    pub org_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Impact {
    pub value: Option<i32>,
    pub unit: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventDto {
    pub event_id: Uuid,
    pub family: String,
    pub provider: Provider,
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
    #[schema(value_type = Vec<Object>)]
    pub areas: Vec<EventArea>,
    pub impact: Impact,
    pub original_url: String,
    pub visibility_status: String,
}

impl From<EventRecord> for EventDto {
    fn from(e: EventRecord) -> Self {
        EventDto {
            event_id: e.event_id,
            family: e.family,
            provider: Provider {
                org_code: e.org_code,
                org_name: e.org_name,
            },
            event_kind: e.event_kind,
            status: e.status,
            publication_scope: e.publication_scope,
            customer_scope: e.customer_scope,
            service_category: e.service_category,
            title: e.title,
            summary: e.summary,
            cause_text: e.cause_text,
            source_updated_at: e.source_updated_at,
            started_at: e.started_at,
            ended_at: e.ended_at,
            recovery_estimate_at: e.recovery_estimate_at,
            areas: e.areas,
            impact: Impact {
                value: e.impact_count,
                unit: e.impact_unit,
            },
            original_url: e.original_url,
            visibility_status: e.visibility_status,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventListResponse {
    pub items: Vec<EventDto>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}
