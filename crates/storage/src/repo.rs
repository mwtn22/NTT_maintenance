use crate::models::*;
use crate::StorageError;
use chrono::{DateTime, Utc};
use domain::{EventArea, NormalizedEvent, SourceMeta};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{PgPool, QueryBuilder, Row};
use uuid::Uuid;

/// Thin repository wrapper around a connection pool.
#[derive(Clone)]
pub struct Repo {
    pool: PgPool,
}

const EVENT_SELECT: &str = r#"
SELECT e.event_id, sr.family, sr.org_code, sr.org_name, e.event_kind, e.status,
       sr.publication_scope, e.customer_scope, e.service_category, e.title, e.summary,
       e.cause_text, e.source_updated_at, e.started_at, e.ended_at, e.recovery_estimate_at,
       e.impact_count, e.impact_unit, e.visibility_status, e.original_url,
       COALESCE((SELECT json_agg(json_build_object(
            'pref_code', ea.pref_code, 'municipality_code', ea.municipality_code,
            'pref_name', ea.pref_name, 'municipality_name', ea.municipality_name,
            'locality_name', ea.locality_name, 'area_level', ea.area_level))
         FROM event_areas ea WHERE ea.event_id = e.event_id), '[]'::json) AS areas
FROM events e
JOIN source_registry sr ON sr.source_id = e.source_id
"#;

impl Repo {
    pub async fn connect(database_url: &str) -> Result<Self, StorageError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<(), StorageError> {
        sqlx::migrate!("../../migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Insert or update a source by `org_code`, returning its id.
    pub async fn upsert_source(&self, meta: &SourceMeta) -> Result<Uuid, StorageError> {
        let row = sqlx::query(
            r#"
INSERT INTO source_registry
    (source_id, org_code, org_name, family, source_type, publication_scope,
     retrieval_mode, poll_interval_sec, terms_review_status, enabled, canonical_url)
VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, 'reviewed', true, $8)
ON CONFLICT (org_code) DO UPDATE SET
    org_name = EXCLUDED.org_name,
    family = EXCLUDED.family,
    source_type = EXCLUDED.source_type,
    publication_scope = EXCLUDED.publication_scope,
    retrieval_mode = EXCLUDED.retrieval_mode,
    poll_interval_sec = EXCLUDED.poll_interval_sec,
    canonical_url = EXCLUDED.canonical_url
RETURNING source_id
"#,
        )
        .bind(&meta.org_code)
        .bind(&meta.org_name)
        .bind(meta.family.as_str())
        .bind(source_type_str(meta.source_type))
        .bind(publication_scope_str(meta.publication_scope))
        .bind(retrieval_mode_str(meta.retrieval_mode))
        .bind(meta.poll_interval_sec)
        .bind(&meta.canonical_url)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get("source_id"))
    }

    pub async fn insert_raw_document(
        &self,
        source_id: Uuid,
        fetched_at: DateTime<Utc>,
        http_status: i32,
        content_type: &str,
        body_sha256: &str,
    ) -> Result<Uuid, StorageError> {
        let row = sqlx::query(
            r#"
INSERT INTO raw_documents
    (raw_document_id, source_id, fetched_at, http_status, content_type, body_sha256)
VALUES (gen_random_uuid(), $1, $2, $3, $4, $5)
RETURNING raw_document_id
"#,
        )
        .bind(source_id)
        .bind(fetched_at)
        .bind(http_status)
        .bind(content_type)
        .bind(body_sha256)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get("raw_document_id"))
    }

    /// Upsert a normalized event (keyed by source_id + external_event_key) and
    /// replace its affected areas. Idempotent across re-fetches.
    pub async fn upsert_event(
        &self,
        source_id: Uuid,
        raw_document_id: Uuid,
        ev: &NormalizedEvent,
    ) -> Result<Uuid, StorageError> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
INSERT INTO events
    (event_id, source_id, raw_document_id, external_event_key, event_kind, status,
     service_category, title, summary, cause_text, customer_scope, source_updated_at,
     started_at, ended_at, recovery_estimate_at, impact_count, impact_unit,
     visibility_status, original_url)
VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
        $15, $16, $17, $18)
ON CONFLICT (source_id, external_event_key) DO UPDATE SET
    raw_document_id = EXCLUDED.raw_document_id,
    event_kind = EXCLUDED.event_kind,
    status = EXCLUDED.status,
    service_category = EXCLUDED.service_category,
    title = EXCLUDED.title,
    summary = EXCLUDED.summary,
    cause_text = EXCLUDED.cause_text,
    customer_scope = EXCLUDED.customer_scope,
    source_updated_at = EXCLUDED.source_updated_at,
    started_at = EXCLUDED.started_at,
    ended_at = EXCLUDED.ended_at,
    recovery_estimate_at = EXCLUDED.recovery_estimate_at,
    impact_count = EXCLUDED.impact_count,
    impact_unit = EXCLUDED.impact_unit,
    visibility_status = EXCLUDED.visibility_status,
    original_url = EXCLUDED.original_url
RETURNING event_id
"#,
        )
        .bind(source_id)
        .bind(raw_document_id)
        .bind(&ev.external_event_key)
        .bind(ev.event_kind.as_str())
        .bind(ev.status.as_str())
        .bind(&ev.service_category)
        .bind(&ev.title)
        .bind(&ev.summary)
        .bind(&ev.cause_text)
        .bind(ev.customer_scope.as_str())
        .bind(ev.source_updated_at)
        .bind(ev.started_at)
        .bind(ev.ended_at)
        .bind(ev.recovery_estimate_at)
        .bind(ev.impact_count)
        .bind(&ev.impact_unit)
        .bind(ev.visibility_status.as_str())
        .bind(&ev.original_url)
        .fetch_one(&mut *tx)
        .await?;
        let event_id: Uuid = row.get("event_id");

        sqlx::query("DELETE FROM event_areas WHERE event_id = $1")
            .bind(event_id)
            .execute(&mut *tx)
            .await?;

        for a in &ev.areas {
            sqlx::query(
                r#"
INSERT INTO event_areas
    (event_area_id, event_id, pref_code, municipality_code, pref_name,
     municipality_name, locality_name, area_level)
VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7)
"#,
            )
            .bind(event_id)
            .bind(&a.pref_code)
            .bind(&a.municipality_code)
            .bind(&a.pref_name)
            .bind(&a.municipality_name)
            .bind(&a.locality_name)
            .bind(&a.area_level)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(event_id)
    }

    pub async fn list_events(&self, f: &EventFilter) -> Result<Vec<EventRecord>, StorageError> {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(EVENT_SELECT);
        push_filters(&mut qb, f);
        qb.push(" ORDER BY COALESCE(e.started_at, e.source_updated_at) DESC NULLS LAST ");
        qb.push(" LIMIT ").push_bind(f.limit);
        qb.push(" OFFSET ").push_bind(f.offset);

        let rows = qb.build().fetch_all(&self.pool).await?;
        rows.iter().map(row_to_event_record).collect()
    }

    pub async fn count_events(&self, f: &EventFilter) -> Result<i64, StorageError> {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "SELECT COUNT(*) AS n FROM events e JOIN source_registry sr ON sr.source_id = e.source_id ",
        );
        push_filters(&mut qb, f);
        let row = qb.build().fetch_one(&self.pool).await?;
        Ok(row.get::<i64, _>("n"))
    }

    pub async fn get_event(&self, id: Uuid) -> Result<Option<EventRecord>, StorageError> {
        let sql = format!("{EVENT_SELECT} WHERE e.event_id = $1");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.as_ref().map(row_to_event_record).transpose()
    }

    pub async fn summary(&self, f: &EventFilter) -> Result<SummaryRecord, StorageError> {
        let total = self.count_events(f).await?;
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            r#"SELECT ea.pref_code, MAX(ea.pref_name) AS pref_name, COUNT(DISTINCT e.event_id) AS n
               FROM events e
               JOIN source_registry sr ON sr.source_id = e.source_id
               JOIN event_areas ea ON ea.event_id = e.event_id "#,
        );
        push_filters(&mut qb, f);
        qb.push(" GROUP BY ea.pref_code ORDER BY n DESC ");
        let rows = qb.build().fetch_all(&self.pool).await?;
        let by_pref = rows
            .iter()
            .map(|r| PrefCount {
                pref_code: r.try_get("pref_code").ok(),
                pref_name: r.try_get("pref_name").ok(),
                count: r.get::<i64, _>("n"),
            })
            .collect();
        Ok(SummaryRecord { total, by_pref })
    }

    pub async fn list_sources(
        &self,
        family: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<Vec<SourceRecord>, StorageError> {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            r#"SELECT source_id, org_code, org_name, family, source_type, publication_scope,
                      retrieval_mode, poll_interval_sec, enabled, canonical_url
               FROM source_registry WHERE 1=1 "#,
        );
        if let Some(fam) = family {
            qb.push(" AND family = ").push_bind(fam.to_string());
        }
        if let Some(en) = enabled {
            qb.push(" AND enabled = ").push_bind(en);
        }
        qb.push(" ORDER BY org_code ");
        let rows = qb.build().fetch_all(&self.pool).await?;
        Ok(rows
            .iter()
            .map(|r| SourceRecord {
                source_id: r.get("source_id"),
                org_code: r.get("org_code"),
                org_name: r.get("org_name"),
                family: r.get("family"),
                source_type: r.get("source_type"),
                publication_scope: r.get("publication_scope"),
                retrieval_mode: r.get("retrieval_mode"),
                poll_interval_sec: r.get("poll_interval_sec"),
                enabled: r.get("enabled"),
                canonical_url: r.get("canonical_url"),
            })
            .collect())
    }

    pub async fn freshness(&self) -> Result<Vec<FreshnessRecord>, StorageError> {
        let rows = sqlx::query(
            r#"
SELECT sr.source_id, sr.org_code,
       MAX(rd.fetched_at) AS last_success_at,
       MAX(e.source_updated_at) AS last_source_updated_at
FROM source_registry sr
LEFT JOIN raw_documents rd ON rd.source_id = sr.source_id
LEFT JOIN events e ON e.source_id = sr.source_id
GROUP BY sr.source_id, sr.org_code
ORDER BY sr.org_code
"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .iter()
            .map(|r| {
                let last_success_at: Option<DateTime<Utc>> =
                    r.try_get("last_success_at").ok().flatten();
                let lag_sec = last_success_at.map(|t| (Utc::now() - t).num_seconds());
                FreshnessRecord {
                    source_id: r.get("source_id"),
                    org_code: r.get("org_code"),
                    last_success_at,
                    last_source_updated_at: r.try_get("last_source_updated_at").ok().flatten(),
                    lag_sec,
                }
            })
            .collect())
    }

    /// Build a GeoJSON FeatureCollection. Geometry is omitted (null) until the
    /// `areas` master is populated with PostGIS shapes; properties carry the
    /// event metadata so the map layer is already wired end-to-end.
    pub async fn events_geojson(&self, f: &EventFilter) -> Result<serde_json::Value, StorageError> {
        let records = self.list_events(f).await?;
        let features: Vec<serde_json::Value> = records
            .iter()
            .map(|e| {
                serde_json::json!({
                    "type": "Feature",
                    "geometry": serde_json::Value::Null,
                    "properties": {
                        "event_id": e.event_id,
                        "family": e.family,
                        "event_kind": e.event_kind,
                        "status": e.status,
                        "title": e.title,
                        "visibility_status": e.visibility_status,
                        "pref_codes": e.areas.iter().filter_map(|a| a.pref_code.clone()).collect::<Vec<_>>(),
                    }
                })
            })
            .collect();
        Ok(serde_json::json!({
            "type": "FeatureCollection",
            "features": features,
        }))
    }
}

/// Append the shared WHERE filters used by list/count/summary.
fn push_filters(qb: &mut QueryBuilder<sqlx::Postgres>, f: &EventFilter) {
    qb.push(" WHERE 1=1 ");
    if let Some(fam) = &f.family {
        qb.push(" AND sr.family = ").push_bind(fam.clone());
    }
    if let Some(k) = &f.event_kind {
        qb.push(" AND e.event_kind = ").push_bind(k.clone());
    }
    if let Some(s) = &f.status {
        qb.push(" AND e.status = ").push_bind(s.clone());
    }
    if let Some(p) = &f.pref_code {
        qb.push(" AND EXISTS (SELECT 1 FROM event_areas ea WHERE ea.event_id = e.event_id AND ea.pref_code = ")
            .push_bind(p.clone())
            .push(") ");
    }
    if let Some(m) = &f.municipality_code {
        qb.push(" AND EXISTS (SELECT 1 FROM event_areas ea WHERE ea.event_id = e.event_id AND ea.municipality_code = ")
            .push_bind(m.clone())
            .push(") ");
    }
    if let Some(q) = &f.q {
        qb.push(" AND e.search_vector @@ plainto_tsquery('simple', ")
            .push_bind(q.clone())
            .push(") ");
    }
    if let Some(from) = f.started_from {
        qb.push(" AND e.started_at >= ").push_bind(from);
    }
    if let Some(to) = f.started_to {
        qb.push(" AND e.started_at <= ").push_bind(to);
    }
}

fn row_to_event_record(r: &PgRow) -> Result<EventRecord, StorageError> {
    let areas_json: serde_json::Value = r.get("areas");
    let areas: Vec<EventArea> = serde_json::from_value(areas_json)?;
    Ok(EventRecord {
        event_id: r.get("event_id"),
        family: r.get("family"),
        org_code: r.get("org_code"),
        org_name: r.get("org_name"),
        event_kind: r.get("event_kind"),
        status: r.get("status"),
        publication_scope: r.get("publication_scope"),
        customer_scope: r.get("customer_scope"),
        service_category: r.try_get("service_category").ok().flatten(),
        title: r.get("title"),
        summary: r.try_get("summary").ok().flatten(),
        cause_text: r.try_get("cause_text").ok().flatten(),
        source_updated_at: r.try_get("source_updated_at").ok().flatten(),
        started_at: r.try_get("started_at").ok().flatten(),
        ended_at: r.try_get("ended_at").ok().flatten(),
        recovery_estimate_at: r.try_get("recovery_estimate_at").ok().flatten(),
        impact_count: r.try_get("impact_count").ok().flatten(),
        impact_unit: r.try_get("impact_unit").ok().flatten(),
        visibility_status: r.get("visibility_status"),
        original_url: r.get("original_url"),
        areas,
    })
}

fn source_type_str(t: domain::SourceType) -> &'static str {
    use domain::SourceType::*;
    match t {
        Html => "html",
        Json => "json",
        Feed => "feed",
        AppWeb => "app-web",
    }
}

fn publication_scope_str(s: domain::PublicationScope) -> &'static str {
    use domain::PublicationScope::*;
    match s {
        Public => "public",
        Restricted => "restricted",
        SpecialHighVoltageOnly => "special_high_voltage_only",
    }
}

fn retrieval_mode_str(m: domain::RetrievalMode) -> &'static str {
    use domain::RetrievalMode::*;
    match m {
        Http => "http",
        Headless => "headless",
        ManualBackfill => "manual-backfill",
    }
}
