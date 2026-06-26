//! Code-first OpenAPI document (utoipa).

use crate::dto::{EventDto, EventListResponse, Impact, Provider};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "全国版 通信工事・停電情報統合API",
        version = "0.1.0",
        description = "NTT東西の工事/故障情報と各電力会社の停電/瞬時電圧低下情報を横断表示するAPI。"
    ),
    paths(),
    components(schemas(EventDto, EventListResponse, Provider, Impact))
)]
pub struct ApiDoc;
