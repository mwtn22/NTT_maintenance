-- Migration 0003: approximate centroid geometry for areas master,
-- plus per-source health tracking columns for circuit-breaker support.

-- ── 1. Add centroid column to areas ──────────────────────────────────────────
-- Real MultiPolygon geometry is loaded later from KSJ N03 shapefiles.
-- Until then, centroid (Point) gives GeoJSON enough to place markers accurately.
ALTER TABLE areas ADD COLUMN IF NOT EXISTS centroid GEOMETRY(Point, 4326);

-- Seed centroids for the municipalities already in the minimal fixture seed.
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(141.430, 43.060), 4326)
    WHERE municipality_code = '01101';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(142.365, 43.770), 4326)
    WHERE municipality_code = '01204';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(140.869, 38.268), 4326)
    WHERE municipality_code = '04101';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(139.754, 35.694), 4326)
    WHERE municipality_code = '13101';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(139.703, 35.693), 4326)
    WHERE municipality_code = '13104';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(139.638, 35.444), 4326)
    WHERE municipality_code = '14100';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(137.211, 36.695), 4326)
    WHERE municipality_code = '16201';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(136.907, 35.171), 4326)
    WHERE municipality_code = '23106';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(135.520, 34.686), 4326)
    WHERE municipality_code = '27100';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(135.195, 34.690), 4326)
    WHERE municipality_code = '28100';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(132.449, 34.378), 4326)
    WHERE municipality_code = '34105';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(132.765, 33.839), 4326)
    WHERE municipality_code = '38201';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(130.420, 33.590), 4326)
    WHERE municipality_code = '40130';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(130.421, 33.589), 4326)
    WHERE municipality_code = '40135';
UPDATE areas SET centroid = ST_SetSRID(ST_MakePoint(127.681, 26.212), 4326)
    WHERE municipality_code = '47201';

CREATE INDEX IF NOT EXISTS idx_areas_centroid ON areas USING GIST(centroid);

-- ── 2. Per-source health columns for circuit-breaker ─────────────────────────
-- consecutive_failures resets to 0 on every successful ingest cycle.
-- Scheduler doubles the poll interval (up to 5 doublings) while > 0.
ALTER TABLE source_registry
    ADD COLUMN IF NOT EXISTS consecutive_failures INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_failure_at      TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_failure_text     TEXT;
