//! Source connectors: fetch raw documents and parse them into
//! [`NormalizedEvent`]s. Fetching is fixture-driven by default; live HTTP is
//! gated behind [`FetchMode::Live`] so tests and sandboxes never hit the
//! network.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{ConnectorError, NormalizedEvent, SourceMeta};
use sha2::{Digest, Sha256};

pub mod chubu_pg;
pub mod chugoku_epco;
pub mod hokkaido_epco;
pub mod hokuriku_epco;
pub mod kansai_td;
pub mod kyushu_epco;
pub mod ntt_east;
pub mod ntt_west;
pub mod okinawa_epco;
pub mod parse_util;
pub mod shikoku_epco;
pub mod tepco_pg;
pub mod tohoku_epco;

/// A fetched raw document, persisted before parsing for traceability.
#[derive(Debug, Clone)]
pub struct RawDocument {
    pub fetched_at: DateTime<Utc>,
    pub http_status: i32,
    pub content_type: String,
    pub body: String,
    pub body_sha256: String,
}

impl RawDocument {
    pub fn new(http_status: i32, content_type: impl Into<String>, body: String) -> Self {
        let body_sha256 = hex(Sha256::digest(body.as_bytes()).as_slice());
        Self {
            fetched_at: Utc::now(),
            http_status,
            content_type: content_type.into(),
            body,
            body_sha256,
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Where a connector gets its bytes from.
#[derive(Debug, Clone)]
pub enum FetchMode {
    /// Read a saved HTML fixture from disk. Deterministic, no network.
    Fixture { path: std::path::PathBuf },
    /// Perform a real HTTP GET. Disabled by default.
    Live,
}

/// Per-fetch context passed to a [`Source`].
#[derive(Debug, Clone)]
pub struct FetchCtx {
    pub mode: FetchMode,
}

/// A single provider/source.
#[async_trait]
pub trait Source: Send + Sync {
    fn meta(&self) -> SourceMeta;

    /// The canonical URL fetched when running in [`FetchMode::Live`].
    fn fetch_url(&self) -> &str;

    /// Retrieve the raw document (fixture or live).
    async fn fetch(&self, ctx: &FetchCtx) -> Result<RawDocument, ConnectorError> {
        match &ctx.mode {
            FetchMode::Fixture { path } => {
                let body = std::fs::read_to_string(path)
                    .map_err(|e| ConnectorError::Fixture(format!("{}: {e}", path.display())))?;
                Ok(RawDocument::new(200, "text/html", body))
            }
            FetchMode::Live => {
                let url = self.fetch_url().to_string();
                let resp = reqwest::get(&url)
                    .await
                    .map_err(|e| ConnectorError::Network {
                        url: url.clone(),
                        source: Box::new(e),
                    })?;
                let status = resp.status().as_u16() as i32;
                let content_type = resp
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("text/html")
                    .to_string();
                let body = resp.text().await.map_err(|e| ConnectorError::Network {
                    url,
                    source: Box::new(e),
                })?;
                Ok(RawDocument::new(status, content_type, body))
            }
        }
    }

    /// Parse a raw document into normalized events.
    fn parse(&self, raw: &RawDocument) -> Result<Vec<NormalizedEvent>, ConnectorError>;
}

/// Build the registry of all connectors (telecom + all 10 power operators).
pub fn all_sources() -> Vec<Box<dyn Source>> {
    vec![
        // Telecom
        Box::new(ntt_east::NttEast::new()),
        Box::new(ntt_west::NttWest::new()),
        // Power — north to south
        Box::new(hokkaido_epco::HokkaidoEpco::new()),
        Box::new(tohoku_epco::TohokuEpco::new()),
        Box::new(tepco_pg::TepcoPg::new()),
        Box::new(chubu_pg::ChubuPg::new()),
        Box::new(hokuriku_epco::HokurikuEpco::new()),
        Box::new(kansai_td::KansaiTd::new()),
        Box::new(chugoku_epco::ChugokuEpco::new()),
        Box::new(shikoku_epco::ShikokuEpco::new()),
        Box::new(kyushu_epco::KyushuEpco::new()),
        Box::new(okinawa_epco::OkinawaEpco::new()),
    ]
}

/// Look up a single connector by `org_code`.
pub fn source_by_org_code(org_code: &str) -> Option<Box<dyn Source>> {
    all_sources()
        .into_iter()
        .find(|s| s.meta().org_code == org_code)
}
