-- Nationwide telecom/power outage aggregator — initial schema.
-- Requires the postgis and pgcrypto (gen_random_uuid) extensions.

CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ── source_registry ──────────────────────────────────────────────────────────
CREATE TABLE source_registry (
    source_id           UUID PRIMARY KEY,
    org_code            TEXT NOT NULL UNIQUE,
    org_name            TEXT NOT NULL,
    family              TEXT NOT NULL CHECK (family IN ('telecom', 'power')),
    source_type         TEXT NOT NULL CHECK (source_type IN ('html', 'json', 'feed', 'app-web')),
    publication_scope   TEXT NOT NULL CHECK (publication_scope IN ('public', 'restricted', 'special_high_voltage_only')),
    retrieval_mode      TEXT NOT NULL CHECK (retrieval_mode IN ('http', 'headless', 'manual-backfill')),
    poll_interval_sec   INTEGER NOT NULL DEFAULT 600,
    terms_review_status TEXT NOT NULL DEFAULT 'pending',
    enabled             BOOLEAN NOT NULL DEFAULT true,
    canonical_url       TEXT NOT NULL
);

-- ── raw_documents ────────────────────────────────────────────────────────────
CREATE TABLE raw_documents (
    raw_document_id  UUID PRIMARY KEY,
    source_id        UUID NOT NULL REFERENCES source_registry(source_id),
    fetched_at       TIMESTAMPTZ NOT NULL,
    http_status      INTEGER NOT NULL,
    content_type     TEXT NOT NULL,
    body_sha256      TEXT NOT NULL,
    body_storage_uri TEXT
);
CREATE INDEX idx_raw_documents_source ON raw_documents(source_id, fetched_at DESC);

-- ── events ───────────────────────────────────────────────────────────────────
CREATE TABLE events (
    event_id             UUID PRIMARY KEY,
    source_id            UUID NOT NULL REFERENCES source_registry(source_id),
    raw_document_id      UUID REFERENCES raw_documents(raw_document_id),
    external_event_key   TEXT NOT NULL,
    event_kind           TEXT NOT NULL CHECK (event_kind IN ('maintenance', 'construction', 'fault', 'outage', 'voltage_sag')),
    status               TEXT NOT NULL CHECK (status IN ('planned', 'active', 'resolved', 'historical', 'unknown')),
    service_category     TEXT,
    title                TEXT NOT NULL,
    summary              TEXT,
    cause_text           TEXT,
    customer_scope       TEXT NOT NULL CHECK (customer_scope IN ('general', 'high_voltage', 'special_high_voltage')),
    source_updated_at    TIMESTAMPTZ,
    started_at           TIMESTAMPTZ,
    ended_at             TIMESTAMPTZ,
    recovery_estimate_at TIMESTAMPTZ,
    impact_count         INTEGER,
    impact_unit          TEXT,
    visibility_status    TEXT NOT NULL CHECK (visibility_status IN ('public', 'restricted', 'excluded_from_public_outage_page')),
    original_url         TEXT NOT NULL,
    search_vector        TSVECTOR GENERATED ALWAYS AS (
        to_tsvector('simple', coalesce(title, '') || ' ' || coalesce(summary, '') || ' ' || coalesce(cause_text, ''))
    ) STORED,
    UNIQUE (source_id, external_event_key)
);
CREATE INDEX idx_events_kind ON events(event_kind);
CREATE INDEX idx_events_status ON events(status);
CREATE INDEX idx_events_started ON events(started_at DESC);
CREATE INDEX idx_events_search ON events USING GIN(search_vector);

-- ── event_areas ──────────────────────────────────────────────────────────────
CREATE TABLE event_areas (
    event_area_id     UUID PRIMARY KEY,
    event_id          UUID NOT NULL REFERENCES events(event_id) ON DELETE CASCADE,
    pref_code         CHAR(2),
    municipality_code CHAR(5),
    pref_name         TEXT,
    municipality_name TEXT,
    locality_name     TEXT,
    area_level        TEXT NOT NULL CHECK (area_level IN ('prefecture', 'municipality', 'locality')),
    geom              GEOMETRY(MultiPolygon, 4326)
);
CREATE INDEX idx_event_areas_event ON event_areas(event_id);
CREATE INDEX idx_event_areas_pref ON event_areas(pref_code);
CREATE INDEX idx_event_areas_muni ON event_areas(municipality_code);
CREATE INDEX idx_event_areas_geom ON event_areas USING GIST(geom);

-- ── event_tags ───────────────────────────────────────────────────────────────
CREATE TABLE event_tags (
    event_id UUID NOT NULL REFERENCES events(event_id) ON DELETE CASCADE,
    tag      TEXT NOT NULL,
    PRIMARY KEY (event_id, tag)
);

-- ── areas (administrative master; minimally seeded for now) ───────────────────
CREATE TABLE areas (
    municipality_code CHAR(5) PRIMARY KEY,
    pref_code         CHAR(2) NOT NULL,
    pref_name         TEXT NOT NULL,
    municipality_name TEXT NOT NULL,
    geom              GEOMETRY(MultiPolygon, 4326)
);
CREATE INDEX idx_areas_pref ON areas(pref_code);
