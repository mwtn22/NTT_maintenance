//! axum HTTP API for the outage aggregator.

mod dto;
mod openapi;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use dto::{EventDto, EventListResponse};
use serde::Deserialize;
use std::net::SocketAddr;
use storage::{EventFilter, Repo};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    repo: Repo,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
    let repo = Repo::connect(&database_url).await?;
    let state = AppState { repo };

    let app = build_router(state);

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;
    tracing::info!(%addr, "starting api");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/events", get(list_events))
        .route("/api/v1/events/summary", get(events_summary))
        .route("/api/v1/events/:event_id", get(get_event))
        .route("/api/v1/map/events.geojson", get(events_geojson))
        .route("/api/v1/sources", get(list_sources))
        .route("/api/v1/freshness", get(freshness))
        .route("/api/v1/areas/:municipality_code/events", get(area_events))
        .route("/api/v1/openapi.json", get(openapi_json))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// ── error type ───────────────────────────────────────────────────────────────
struct AppError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(e.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self.0, "request failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": self.0.to_string() })),
        )
            .into_response()
    }
}

// ── query params ─────────────────────────────────────────────────────────────
#[derive(Debug, Deserialize)]
struct EventQuery {
    family: Option<String>,
    event_kind: Option<String>,
    status: Option<String>,
    pref_code: Option<String>,
    municipality_code: Option<String>,
    q: Option<String>,
    started_from: Option<DateTime<Utc>>,
    started_to: Option<DateTime<Utc>>,
    page: Option<i64>,
    per_page: Option<i64>,
}

impl EventQuery {
    fn to_filter(&self) -> (EventFilter, i64, i64) {
        let page = self.page.unwrap_or(1).max(1);
        let per_page = self.per_page.unwrap_or(20).clamp(1, 200);
        let filter = EventFilter {
            family: self.family.clone(),
            event_kind: self.event_kind.clone(),
            status: self.status.clone(),
            pref_code: self.pref_code.clone(),
            municipality_code: self.municipality_code.clone(),
            q: self.q.clone(),
            started_from: self.started_from,
            started_to: self.started_to,
            limit: per_page,
            offset: (page - 1) * per_page,
        };
        (filter, page, per_page)
    }
}

// ── handlers ─────────────────────────────────────────────────────────────────
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_events(
    State(st): State<AppState>,
    Query(q): Query<EventQuery>,
) -> Result<Json<EventListResponse>, AppError> {
    let (filter, page, per_page) = q.to_filter();
    let items = st.repo.list_events(&filter).await?;
    let total = st.repo.count_events(&filter).await?;
    Ok(Json(EventListResponse {
        items: items.into_iter().map(EventDto::from).collect(),
        page,
        per_page,
        total,
    }))
}

async fn get_event(
    State(st): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<Response, AppError> {
    match st.repo.get_event(event_id).await? {
        Some(e) => Ok(Json(EventDto::from(e)).into_response()),
        None => Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "not found" })),
        )
            .into_response()),
    }
}

async fn events_summary(
    State(st): State<AppState>,
    Query(q): Query<EventQuery>,
) -> Result<Response, AppError> {
    let (filter, _, _) = q.to_filter();
    let s = st.repo.summary(&filter).await?;
    Ok(Json(s).into_response())
}

async fn events_geojson(
    State(st): State<AppState>,
    Query(q): Query<EventQuery>,
) -> Result<Response, AppError> {
    let (mut filter, _, _) = q.to_filter();
    filter.limit = 1000;
    filter.offset = 0;
    let fc = st.repo.events_geojson(&filter).await?;
    Ok(Json(fc).into_response())
}

#[derive(Debug, Deserialize)]
struct SourceQuery {
    family: Option<String>,
    enabled: Option<bool>,
}

async fn list_sources(
    State(st): State<AppState>,
    Query(q): Query<SourceQuery>,
) -> Result<Response, AppError> {
    let items = st.repo.list_sources(q.family.as_deref(), q.enabled).await?;
    Ok(Json(serde_json::json!({ "items": items })).into_response())
}

async fn freshness(State(st): State<AppState>) -> Result<Response, AppError> {
    let items = st.repo.freshness().await?;
    Ok(Json(serde_json::json!({ "items": items })).into_response())
}

async fn area_events(
    State(st): State<AppState>,
    Path(municipality_code): Path<String>,
    Query(q): Query<EventQuery>,
) -> Result<Json<EventListResponse>, AppError> {
    let (mut filter, page, per_page) = q.to_filter();
    filter.municipality_code = Some(municipality_code);
    let items = st.repo.list_events(&filter).await?;
    let total = st.repo.count_events(&filter).await?;
    Ok(Json(EventListResponse {
        items: items.into_iter().map(EventDto::from).collect(),
        page,
        per_page,
        total,
    }))
}

async fn openapi_json() -> impl IntoResponse {
    Json(openapi::ApiDoc::openapi())
}
