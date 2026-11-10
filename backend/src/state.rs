use sqlx::PgPool;

use crate::storage::ImageStore;

/// 全局共享状态。
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub images: ImageStore,
    pub token_ttl_hours: i64,
}
