//! Ingestion pipeline: fetch (fixture or live) -> persist raw -> parse ->
//! normalize -> upsert events + areas.

use connectors::{FetchCtx, FetchMode, Source};
use std::path::PathBuf;
use storage::Repo;
use uuid::Uuid;

/// Resolve the default fixture path for a connector by org_code.
pub fn fixture_path(org_code: &str) -> Option<PathBuf> {
    let root = std::env::var("FIXTURES_DIR").unwrap_or_else(|_| "fixtures".to_string());
    let rel = match org_code {
        "ntt-east" => "ntt_east/sample_outage.html",
        "ntt-west" => "ntt_west/sample_outage.html",
        "hokkaido-epco" => "hokkaido_epco/sample_outage.html",
        "tohoku-epco" => "tohoku_epco/sample_outage.html",
        "tepco-pg" => "tepco_pg/sample_outage.html",
        "chubu-pg" => "chubu_pg/sample_outage.html",
        "hokuriku-epco" => "hokuriku_epco/sample_outage.html",
        "kansai-td" => "kansai_td/sample_voltage_sag.html",
        "chugoku-epco" => "chugoku_epco/sample_outage.html",
        "shikoku-epco" => "shikoku_epco/sample_outage.html",
        "kyushu-epco" => "kyushu_epco/sample_outage.html",
        "okinawa-epco" => "okinawa_epco/sample_outage.html",
        _ => return None,
    };
    Some(PathBuf::from(root).join(rel))
}

/// Decide the fetch mode from the `FETCH_MODE` env var (default: fixture).
pub fn fetch_ctx_for(org_code: &str) -> anyhow::Result<FetchCtx> {
    let mode = std::env::var("FETCH_MODE").unwrap_or_else(|_| "fixture".to_string());
    let mode = match mode.as_str() {
        "live" => FetchMode::Live,
        _ => {
            let path = fixture_path(org_code)
                .ok_or_else(|| anyhow::anyhow!("no fixture mapping for {org_code}"))?;
            FetchMode::Fixture { path }
        }
    };
    Ok(FetchCtx { mode })
}

/// Run one full ingest cycle for a single source.
/// Updates source health tracking (consecutive_failures) on success or failure.
pub async fn run_source(repo: &Repo, source: &dyn Source) -> anyhow::Result<usize> {
    let meta = source.meta();
    let source_id = repo.upsert_source(&meta).await?;
    let ctx = fetch_ctx_for(&meta.org_code)?;

    match run_source_inner(repo, source, source_id, &ctx).await {
        Ok(count) => {
            if let Err(e) = repo.record_ingest_success(source_id).await {
                tracing::warn!(error = %e, "failed to record ingest success");
            }
            Ok(count)
        }
        Err(e) => {
            let err_str = e.to_string();
            if let Err(re) = repo.record_ingest_failure(source_id, &err_str).await {
                tracing::warn!(error = %re, "failed to record ingest failure");
            }
            Err(e)
        }
    }
}

async fn run_source_inner(
    repo: &Repo,
    source: &dyn Source,
    source_id: Uuid,
    ctx: &FetchCtx,
) -> anyhow::Result<usize> {
    let meta = source.meta();
    let raw = source.fetch(ctx).await?;
    let raw_document_id = repo
        .insert_raw_document(
            source_id,
            raw.fetched_at,
            raw.http_status,
            &raw.content_type,
            &raw.body_sha256,
        )
        .await?;

    let events = source.parse(&raw)?;
    let count = events.len();
    for ev in &events {
        repo.upsert_event(source_id, raw_document_id, ev).await?;
    }
    tracing::info!(
        org_code = %meta.org_code,
        body_sha256 = %raw.body_sha256,
        events = count,
        "ingest cycle complete"
    );
    Ok(count)
}

/// Run an ingest cycle for every registered connector.
/// Failures are isolated: one source error does not abort the rest.
pub async fn run_all(repo: &Repo) -> anyhow::Result<usize> {
    let mut total = 0;
    for source in connectors::all_sources() {
        match run_source(repo, source.as_ref()).await {
            Ok(n) => total += n,
            Err(e) => tracing::error!(error = %e, "source ingest failed"),
        }
    }
    Ok(total)
}

/// Compute the backoff multiplier based on consecutive failures.
/// Returns the effective poll interval: base * 2^min(failures, 4).
/// Cap at 4 doublings (16× base) so we never wait more than ~80 minutes.
pub fn backoff_interval_secs(base_secs: u64, consecutive_failures: i32) -> u64 {
    let exp = consecutive_failures.clamp(0, 4) as u32;
    base_secs.saturating_mul(2u64.pow(exp))
}
