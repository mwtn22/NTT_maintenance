# 全国版 通信工事・停電情報統合システム

NTT東日本／西日本の工事・故障情報と、各電力会社の停電・瞬時電圧低下情報を、
地域・内容で横断検索できる形に正規化して配信するシステムです。

本リポジトリは計画書の**第1イテレーション（基盤 + 縦スライス）**を実装しています。
取得 → 正規化 → 保存 → 配信のパイプラインを、代表的な3ソースでエンドツーエンドに実証します。

## 実装済みスコープ

- **Cargo ワークスペース**（取得・正規化・配信を疎結合に分離）
- **PostgreSQL + PostGIS スキーマ**（`migrations/0001_init.sql`、sqlx マイグレーション）
- **コネクタ 3本**：NTT東日本 / NTT西日本 / 関西電力送配電
  - フィクスチャ駆動（`FETCH_MODE=fixture`、既定）。実HTTP取得は `FETCH_MODE=live` の裏
  - 計画書の `publication_scope` / `customer_scope` / `visibility_status` を正しく設定
  - 関西電力は**一般公開の瞬時電圧低下（voltage_sag）**を含む
- **取り込みパイプライン**（`ingest`）：fetch → raw保存(sha256) → parse → 正規化 → upsert（冪等）
- **axum API**（`api`）：一覧/詳細/集計/GeoJSON/ソース/鮮度/OpenAPI
- **鮮度の二系統表示**：公式更新時刻（`source_updated_at`）と取得時刻（`fetched_at`）を分離

## クレート構成

| クレート | 役割 |
|---|---|
| `domain` | 共通型（Event, EventArea, enum群, エラー型） |
| `storage` | sqlx リポジトリ層（PostgreSQL/PostGIS） |
| `connectors` | `Source` trait + 各ソース実装 + scraper パーサ |
| `ingest` | スケジューラ + 取り込みCLI |
| `api` | axum HTTPサーバ + utoipa OpenAPI |

## セットアップと実行

```bash
# 1. DB起動（PostGIS）
docker compose up -d

# 2. 環境変数
cp .env.example .env
export DATABASE_URL=postgres://outage:outage@localhost:5432/outage

# 3. マイグレーション + フィクスチャ取込
cargo run -p ingest -- run-once            # 全ソース
cargo run -p ingest -- run-once --source kansai-td   # 単一ソース
cargo run -p ingest -- scheduler           # poll_interval_sec で巡回

# 4. API起動
cargo run -p api                           # http://localhost:8080
```

## 主なエンドポイント

| メソッド | パス | 説明 |
|---|---|---|
| GET | `/api/v1/events` | 一覧検索（family, event_kind, status, pref_code, municipality_code, q, started_from/to, page, per_page） |
| GET | `/api/v1/events/{event_id}` | 詳細（areas 同梱） |
| GET | `/api/v1/events/summary` | 件数・都道府県別集計 |
| GET | `/api/v1/map/events.geojson` | 地図描画用 FeatureCollection |
| GET | `/api/v1/sources` | ソース一覧 |
| GET | `/api/v1/areas/{municipality_code}/events` | 特定自治体のイベント |
| GET | `/api/v1/freshness` | ソース別鮮度（last_success_at, lag_sec） |
| GET | `/api/v1/health` | ヘルスチェック |
| GET | `/api/v1/openapi.json` | OpenAPI ドキュメント |

```bash
curl "http://localhost:8080/api/v1/events?family=power&event_kind=voltage_sag"
curl "http://localhost:8080/api/v1/map/events.geojson"
```

## テスト

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all        # パーサのゴールデンテスト（ネットワーク/DB不要）
```

## 後続イテレーション（未実装）

- 残り9電力ソース（同じ `Source` trait で横展開）
- フロントエンド（一覧 + 地図 + フィルタの三面同期）
- 行政区域マスタ（国土数値情報）と PostGIS ジオメトリ投入、GeoJSON のジオメトリ化
- ヘッドレス sidecar、Redis/CDN キャッシュ、認証/WAF/監査ログ
- live取得を有効化する前の robots.txt / 利用条件レビュー・個別許諾

## 法的留意

各公式ソースは利用条件・著作権を定めています。本システムは原文の全面転載ではなく、
**正規化メタデータの表示 + 原典リンク + 取得根拠（raw_documents）保持**を基本方針とします。
`FETCH_MODE=live` を有効化する前に、対象ドメインの利用条件と robots.txt を確認してください。
