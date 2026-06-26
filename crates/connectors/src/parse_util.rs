//! Small parsing helpers shared by connectors.

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use domain::{
    ConnectorError, CustomerScope, EventArea, EventKind, EventStatus, NormalizedEvent,
    VisibilityStatus,
};
use scraper::{ElementRef, Html, Selector};

/// Japan Standard Time offset (+09:00).
pub fn jst() -> FixedOffset {
    FixedOffset::east_opt(9 * 3600).expect("valid JST offset")
}

/// Parse a `YYYY-MM-DD HH:MM` (or with `/` separators) JST string into UTC.
pub fn parse_jst_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim().replace('/', "-");
    if s.is_empty() {
        return None;
    }
    for fmt in ["%Y-%m-%d %H:%M", "%Y-%m-%d %H:%M:%S"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, fmt) {
            let dt = jst().from_local_datetime(&naive).single()?;
            return Some(dt.with_timezone(&Utc));
        }
    }
    None
}

/// Collapse internal whitespace and trim.
pub fn clean_text(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Connector-supplied defaults applied when a fixture omits an attribute.
pub struct ParseDefaults {
    pub customer_scope: CustomerScope,
    pub visibility_status: VisibilityStatus,
    /// Fallback event kind when a row omits `data-kind`.
    pub default_kind: EventKind,
}

fn map_kind(s: &str, default: EventKind) -> EventKind {
    match s {
        "maintenance" => EventKind::Maintenance,
        "construction" => EventKind::Construction,
        "fault" => EventKind::Fault,
        "outage" => EventKind::Outage,
        "voltage_sag" => EventKind::VoltageSag,
        _ => default,
    }
}

fn map_status(s: &str) -> EventStatus {
    match s {
        "planned" => EventStatus::Planned,
        "active" => EventStatus::Active,
        "resolved" => EventStatus::Resolved,
        "historical" => EventStatus::Historical,
        _ => EventStatus::Unknown,
    }
}

fn map_customer(s: &str, default: CustomerScope) -> CustomerScope {
    match s {
        "general" => CustomerScope::General,
        "high_voltage" => CustomerScope::HighVoltage,
        "special_high_voltage" => CustomerScope::SpecialHighVoltage,
        _ => default,
    }
}

fn sel(s: &str) -> Selector {
    Selector::parse(s).expect("static selector")
}

fn text_of(el: ElementRef, css: &str) -> Option<String> {
    let selector = sel(css);
    el.select(&selector)
        .next()
        .map(|n| clean_text(&n.text().collect::<String>()))
        .filter(|s| !s.is_empty())
}

fn datetime_attr(el: ElementRef, css: &str) -> Option<DateTime<Utc>> {
    let selector = sel(css);
    el.select(&selector)
        .next()
        .and_then(|n| n.value().attr("datetime"))
        .and_then(parse_jst_datetime)
}

fn parse_areas(event: ElementRef) -> Vec<EventArea> {
    let area_sel = sel(".areas .area");
    event
        .select(&area_sel)
        .map(|a| {
            let v = a.value();
            EventArea {
                pref_code: v.attr("data-pref").map(str::to_string),
                municipality_code: v.attr("data-muni").map(str::to_string),
                pref_name: text_of(a, ".pref"),
                municipality_name: text_of(a, ".muni"),
                locality_name: text_of(a, ".locality"),
                area_level: v.attr("data-level").unwrap_or("prefecture").to_string(),
            }
        })
        .collect()
}

/// Parse the shared `.event` fixture structure into normalized events.
pub fn parse_events(
    raw_body: &str,
    defaults: &ParseDefaults,
) -> Result<Vec<NormalizedEvent>, ConnectorError> {
    let doc = Html::parse_document(raw_body);
    let event_sel = sel(".event");
    let events: Vec<ElementRef> = doc.select(&event_sel).collect();
    if events.is_empty() {
        return Err(ConnectorError::SchemaMismatch(
            "no `.event` nodes found".to_string(),
        ));
    }

    let mut out = Vec::with_capacity(events.len());
    for ev in events {
        let v = ev.value();
        let external_event_key = v
            .attr("data-key")
            .ok_or_else(|| ConnectorError::SchemaMismatch("event missing data-key".into()))?
            .to_string();
        let title = text_of(ev, ".title")
            .ok_or_else(|| ConnectorError::SchemaMismatch("event missing .title".into()))?;
        let original_url = ev
            .select(&sel("a.original"))
            .next()
            .and_then(|a| a.value().attr("href"))
            .ok_or_else(|| ConnectorError::SchemaMismatch("event missing a.original".into()))?
            .to_string();

        let impact = ev.select(&sel(".impact")).next();
        let impact_count = impact
            .map(|i| clean_text(&i.text().collect::<String>()))
            .and_then(|t| t.replace(',', "").parse::<i32>().ok());
        let impact_unit = impact
            .and_then(|i| i.value().attr("data-unit"))
            .map(str::to_string);

        out.push(NormalizedEvent {
            external_event_key,
            event_kind: map_kind(v.attr("data-kind").unwrap_or(""), defaults.default_kind),
            status: map_status(v.attr("data-status").unwrap_or("")),
            service_category: text_of(ev, ".service"),
            title,
            summary: text_of(ev, ".summary"),
            cause_text: text_of(ev, ".cause"),
            customer_scope: map_customer(
                v.attr("data-customer").unwrap_or(""),
                defaults.customer_scope,
            ),
            source_updated_at: datetime_attr(ev, "time.source-updated"),
            started_at: datetime_attr(ev, "time.started"),
            ended_at: datetime_attr(ev, "time.ended"),
            recovery_estimate_at: datetime_attr(ev, "time.recovery"),
            impact_count,
            impact_unit,
            visibility_status: defaults.visibility_status,
            original_url,
            areas: parse_areas(ev),
        });
    }
    Ok(out)
}
