//! Golden tests: parse each fixture and assert the normalized output.
//! These run with no network and no database.

use connectors::{FetchCtx, FetchMode, Source};
use domain::{EventKind, EventStatus, VisibilityStatus};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    // crate dir is crates/connectors; fixtures live at the workspace root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
}

async fn parse_fixture(source: &dyn Source, rel: &str) -> Vec<domain::NormalizedEvent> {
    let ctx = FetchCtx {
        mode: FetchMode::Fixture {
            path: fixtures_dir().join(rel),
        },
    };
    let raw = source.fetch(&ctx).await.expect("fetch fixture");
    source.parse(&raw).expect("parse fixture")
}

#[tokio::test]
async fn ntt_east_parses_two_events() {
    let s = connectors::ntt_east::NttEast::new();
    let events = parse_fixture(&s, "ntt_east/sample_outage.html").await;
    assert_eq!(events.len(), 2);

    let first = &events[0];
    assert_eq!(first.external_event_key, "ntt-east-20260626-0001");
    assert_eq!(first.event_kind, EventKind::Construction);
    assert_eq!(first.status, EventStatus::Planned);
    assert_eq!(first.title, "東京都千代田区 設備工事のお知らせ");
    assert_eq!(first.visibility_status, VisibilityStatus::Public);
    assert_eq!(first.areas.len(), 1);
    assert_eq!(first.areas[0].pref_code.as_deref(), Some("13"));
    assert_eq!(first.areas[0].municipality_code.as_deref(), Some("13101"));
    assert!(first.started_at.is_some());

    assert_eq!(events[1].event_kind, EventKind::Fault);
    assert_eq!(events[1].status, EventStatus::Active);
}

#[tokio::test]
async fn ntt_west_parses_two_events() {
    let s = connectors::ntt_west::NttWest::new();
    let events = parse_fixture(&s, "ntt_west/sample_outage.html").await;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].areas[0].pref_code.as_deref(), Some("27"));
    assert_eq!(events[1].event_kind, EventKind::Fault);
}

#[tokio::test]
async fn kansai_parses_outage_and_voltage_sag() {
    let s = connectors::kansai_td::KansaiTd::new();
    let events = parse_fixture(&s, "kansai_td/sample_voltage_sag.html").await;
    assert_eq!(events.len(), 2);

    let outage = &events[0];
    assert_eq!(outage.event_kind, EventKind::Outage);
    assert_eq!(outage.impact_count, Some(80));
    assert_eq!(outage.impact_unit.as_deref(), Some("戸"));
    assert!(outage.recovery_estimate_at.is_some());

    let sag = &events[1];
    assert_eq!(sag.event_kind, EventKind::VoltageSag);
    assert_eq!(sag.status, EventStatus::Historical);
    // Kansai publishes voltage sag to the general public.
    assert_eq!(sag.visibility_status, VisibilityStatus::Public);
}

#[tokio::test]
async fn hokkaido_parses_active_and_historical() {
    let s = connectors::hokkaido_epco::HokkaidoEpco::new();
    let events = parse_fixture(&s, "hokkaido_epco/sample_outage.html").await;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_kind, EventKind::Outage);
    assert_eq!(events[0].status, EventStatus::Active);
    assert_eq!(events[0].impact_count, Some(120));
    assert_eq!(events[1].status, EventStatus::Historical);
}

#[tokio::test]
async fn tepco_pg_separates_general_and_special_hv() {
    use domain::CustomerScope;
    let s = connectors::tepco_pg::TepcoPg::new();
    let events = parse_fixture(&s, "tepco_pg/sample_outage.html").await;
    assert_eq!(events.len(), 2);

    let outage = &events[0];
    assert_eq!(outage.event_kind, EventKind::Outage);
    assert_eq!(outage.customer_scope, CustomerScope::General);

    // The voltage-sag entry is scoped to special high voltage only.
    let sag = &events[1];
    assert_eq!(sag.event_kind, EventKind::VoltageSag);
    assert_eq!(sag.customer_scope, CustomerScope::SpecialHighVoltage);
    // But it is still on a public page (unlike restricted contracts).
    assert_eq!(sag.visibility_status, VisibilityStatus::Public);
}

#[tokio::test]
async fn all_power_connectors_parse_fixtures() {
    // Smoke-test that every connector can round-trip its fixture without error.
    let sources: Vec<(Box<dyn connectors::Source>, &str)> = vec![
        (
            Box::new(connectors::hokkaido_epco::HokkaidoEpco::new()),
            "hokkaido_epco/sample_outage.html",
        ),
        (
            Box::new(connectors::tohoku_epco::TohokuEpco::new()),
            "tohoku_epco/sample_outage.html",
        ),
        (
            Box::new(connectors::chubu_pg::ChubuPg::new()),
            "chubu_pg/sample_outage.html",
        ),
        (
            Box::new(connectors::hokuriku_epco::HokurikuEpco::new()),
            "hokuriku_epco/sample_outage.html",
        ),
        (
            Box::new(connectors::chugoku_epco::ChugokuEpco::new()),
            "chugoku_epco/sample_outage.html",
        ),
        (
            Box::new(connectors::shikoku_epco::ShikokuEpco::new()),
            "shikoku_epco/sample_outage.html",
        ),
        (
            Box::new(connectors::kyushu_epco::KyushuEpco::new()),
            "kyushu_epco/sample_outage.html",
        ),
        (
            Box::new(connectors::okinawa_epco::OkinawaEpco::new()),
            "okinawa_epco/sample_outage.html",
        ),
    ];
    for (source, fixture) in &sources {
        let events = parse_fixture(source.as_ref(), fixture).await;
        assert!(!events.is_empty(), "no events from {fixture}");
        for ev in &events {
            assert!(!ev.title.is_empty(), "empty title in {fixture}");
            assert!(!ev.original_url.is_empty(), "empty url in {fixture}");
        }
    }
}

#[tokio::test]
async fn missing_event_node_is_schema_mismatch() {
    use domain::ConnectorError;
    let s = connectors::ntt_east::NttEast::new();
    let raw =
        connectors::RawDocument::new(200, "text/html", "<html><body>empty</body></html>".into());
    let err = s.parse(&raw).unwrap_err();
    assert!(matches!(err, ConnectorError::SchemaMismatch(_)));
}
