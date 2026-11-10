use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::Json;

use crate::auth::{resolve_public_tenant, resolve_tenant, tenant_selector, CurrentUser};
use crate::error::{ApiJson, AppError};
use crate::models::{BusinessStatusRequest, BusinessStatusRow, BusinessStatusView};
use crate::response::ApiResponse;
use crate::routes::required_text;
use crate::state::AppState;
use sqlx::PgPool;

pub async fn get_status(
    auth: Option<CurrentUser>,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<BusinessStatusView>>, AppError> {
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_public_tenant(&state, auth.as_ref(), tenant_id_param, tenant_code.as_deref(), true)
            .await?;
    Ok(ApiResponse::success(load_status(&state.pool, tenant_id).await?))
}

pub async fn update_status(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    ApiJson(request): ApiJson<BusinessStatusRequest>,
) -> Result<Json<ApiResponse<BusinessStatusView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;

    let store_name = required_text(request.store_name.clone(), "门店名称不能为空")?;
    let notice = required_text(request.notice.clone(), "营业公告不能为空")?;
    let current = load_status(&state.pool, tenant_id).await?;
    let intro = request
        .intro
        .clone()
        .unwrap_or(current.intro)
        .trim()
        .to_string();

    sqlx::query(
        r#"
INSERT INTO tenant_business_status (tenant_id, open, notice, store_name, intro)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (tenant_id) DO UPDATE
SET open = EXCLUDED.open,
    notice = EXCLUDED.notice,
    store_name = EXCLUDED.store_name,
    intro = EXCLUDED.intro
"#,
    )
    .bind(tenant_id)
    .bind(request.open)
    .bind(&notice)
    .bind(&store_name)
    .bind(&intro)
    .execute(&state.pool)
    .await?;

    let status = load_status(&state.pool, tenant_id).await?;
    Ok(ApiResponse::success_with_message("更新成功", status))
}

pub async fn load_status(pool: &PgPool, tenant_id: i64) -> Result<BusinessStatusView, AppError> {
    let row = sqlx::query_as::<_, BusinessStatusRow>(
        "SELECT open, notice, store_name, intro FROM tenant_business_status WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await?;
    Ok(BusinessStatusView::from_row(row))
}
