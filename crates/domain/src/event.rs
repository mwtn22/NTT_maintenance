use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Kind of event, normalized across telecom and power sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Maintenance,
    Construction,
    Fault,
    Outage,
    VoltageSag,
}

impl EventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventKind::Maintenance => "maintenance",
            EventKind::Construction => "construction",
            EventKind::Fault => "fault",
            EventKind::Outage => "outage",
            EventKind::VoltageSag => "voltage_sag",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Planned,
    Active,
    Resolved,
    Historical,
    Unknown,
}

impl EventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventStatus::Planned => "planned",
            EventStatus::Active => "active",
            EventStatus::Resolved => "resolved",
            EventStatus::Historical => "historical",
            EventStatus::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CustomerScope {
    General,
    HighVoltage,
    SpecialHighVoltage,
}

impl CustomerScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            CustomerScope::General => "general",
            CustomerScope::HighVoltage => "high_voltage",
            CustomerScope::SpecialHighVoltage => "special_high_voltage",
        }
    }
}

/// Whether the event is publicly visible, restricted, or excluded from the
/// public outage page (but documented elsewhere).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityStatus {
    Public,
    Restricted,
    ExcludedFromPublicOutagePage,
}

impl VisibilityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            VisibilityStatus::Public => "public",
            VisibilityStatus::Restricted => "restricted",
            VisibilityStatus::ExcludedFromPublicOutagePage => "excluded_from_public_outage_page",
        }
    }
}

/// An affected administrative area, keyed by JIS codes where resolvable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EventArea {
    /// Prefecture code, JIS X 0401 (2 digits), e.g. "27".
    pub pref_code: Option<String>,
    /// Municipality code, prefecture(2) + municipality(3) = 5 digits, e.g. "27100".
    pub municipality_code: Option<String>,
    pub pref_name: Option<String>,
    pub municipality_name: Option<String>,
    pub locality_name: Option<String>,
    /// "prefecture" | "municipality" | "locality".
    pub area_level: String,
}

/// A connector's parsed, normalized event prior to persistence.
///
/// `external_event_key` must be stable for the same logical event so that
/// re-fetches upsert rather than duplicate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct NormalizedEvent {
    pub external_event_key: String,
    pub event_kind: EventKind,
    pub status: EventStatus,
    pub service_category: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub cause_text: Option<String>,
    pub customer_scope: CustomerScope,
    pub source_updated_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub recovery_estimate_at: Option<DateTime<Utc>>,
    pub impact_count: Option<i32>,
    pub impact_unit: Option<String>,
    pub visibility_status: VisibilityStatus,
    pub original_url: String,
    pub areas: Vec<EventArea>,
}
