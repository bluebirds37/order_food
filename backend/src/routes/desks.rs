use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::Json;

use crate::auth::{find_tenant_by_code, resolve_tenant, tenant_selector, CurrentUser};
use crate::error::{ApiJson, AppError};
use crate::models::{DeskRequest, DeskView};
use crate::response::ApiResponse;
use crate::routes::required_text;
use crate::state::AppState;

/// 公开接口：顾客端桌号下拉列表
pub async fn list_desks(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<DeskView>>>, AppError> {
    let tenant_code = params
        .get("tenantCode")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::business("缺少门店标识，请扫码进入"))?;
    let tenant = find_tenant_by_code(&state.pool, &tenant_code).await?;
    if !tenant.enabled {
        return Err(AppError::business("该门店已暂停服务"));
    }
    let desks = sqlx::query_as::<_, DeskView>(
        "SELECT id, desk_code FROM desk WHERE tenant_id = $1 ORDER BY desk_code ASC",
    )
    .bind(tenant.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(ApiResponse::success(desks))
}

pub async fn create_desk(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    ApiJson(request): ApiJson<DeskRequest>,
) -> Result<Json<ApiResponse<DeskView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id = resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;

    let desk_code = normalize_desk_code(request.desk_code.clone())?;

    let desk = sqlx::query_as::<_, DeskView>(
        r#"
INSERT INTO desk (tenant_id, desk_code)
VALUES ($1, $2)
RETURNING id, desk_code
"#,
    )
    .bind(tenant_id)
    .bind(&desk_code)
    .fetch_one(&state.pool)
    .await
    .map_err(|error| {
        if is_unique_violation(&error) {
            AppError::business("该桌号已存在")
        } else {
            AppError::from(error)
        }
    })?;

    Ok(ApiResponse::success_with_message("新增成功", desk))
}

pub async fn update_desk(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    ApiJson(request): ApiJson<DeskRequest>,
) -> Result<Json<ApiResponse<DeskView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id = resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
    let desk_id = crate::routes::parse_id(&id)?;

    let desk_code = normalize_desk_code(request.desk_code.clone())?;

    let desk = sqlx::query_as::<_, DeskView>(
        r#"
UPDATE desk SET desk_code = $1
WHERE id = $2 AND tenant_id = $3
RETURNING id, desk_code
"#,
    )
    .bind(&desk_code)
    .bind(desk_id)
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        if is_unique_violation(&error) {
            AppError::business("该桌号已存在")
        } else {
            AppError::from(error)
        }
    })?
    .ok_or_else(|| AppError::business("桌号不存在或无权操作"))?;

    Ok(ApiResponse::success_with_message("更新成功", desk))
}

pub async fn delete_desk(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id = resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
    let desk_id = crate::routes::parse_id(&id)?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM desk WHERE id = $1 AND tenant_id = $2)",
    )
    .bind(desk_id)
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await?;
    if !exists {
        return Err(AppError::business("桌号不存在或无权操作"));
    }

    sqlx::query("DELETE FROM desk WHERE id = $1 AND tenant_id = $2")
        .bind(desk_id)
        .bind(tenant_id)
        .execute(&state.pool)
        .await?;
    Ok(ApiResponse::<()>::success_empty("删除成功"))
}

fn normalize_desk_code(value: Option<String>) -> Result<String, AppError> {
    let code = required_text(value, "桌号不能为空")?
        .trim()
        .to_uppercase();
    if code.len() > 16 {
        return Err(AppError::business("桌号长度不能超过 16 个字符"));
    }
    Ok(code)
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|db| db.code().as_deref() == Some("23505"))
}
