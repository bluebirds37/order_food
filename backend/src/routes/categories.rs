use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::Json;

use crate::auth::{resolve_tenant, tenant_selector, CurrentUser};
use crate::error::{ApiJson, AppError};
use crate::models::{CategoryRequest, MenuCategoryView};
use crate::response::ApiResponse;
use crate::routes::required_text;
use crate::state::AppState;

pub async fn create_category(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    ApiJson(request): ApiJson<CategoryRequest>,
) -> Result<Json<ApiResponse<MenuCategoryView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;

    let name = required_text(request.name.clone(), "分类名称不能为空")?;
    let sort_order = match request.sort_order {
        Some(value) => value,
        None => next_sort_order(&state, tenant_id).await?,
    };

    let category = sqlx::query_as::<_, MenuCategoryView>(
        r#"
INSERT INTO menu_category (tenant_id, name, sort_order)
VALUES ($1, $2, $3)
RETURNING id, name, sort_order
"#,
    )
    .bind(tenant_id)
    .bind(name)
    .bind(sort_order)
    .fetch_one(&state.pool)
    .await?;

    Ok(ApiResponse::success_with_message("新增成功", category))
}

pub async fn update_category(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    ApiJson(request): ApiJson<CategoryRequest>,
) -> Result<Json<ApiResponse<MenuCategoryView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
    let category_id = crate::routes::parse_id(&id)?;

    let name = required_text(request.name.clone(), "分类名称不能为空")?;
    let sort_order = request.sort_order.unwrap_or(0);

    let category = sqlx::query_as::<_, MenuCategoryView>(
        r#"
UPDATE menu_category
SET name = $1, sort_order = $2
WHERE id = $3 AND tenant_id = $4
RETURNING id, name, sort_order
"#,
    )
    .bind(name)
    .bind(sort_order)
    .bind(category_id)
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::business("分类不存在或无权操作"))?;

    Ok(ApiResponse::success_with_message("更新成功", category))
}

pub async fn delete_category(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
    let category_id = crate::routes::parse_id(&id)?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM menu_category WHERE id = $1 AND tenant_id = $2)",
    )
    .bind(category_id)
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await?;
    if !exists {
        return Err(AppError::business("分类不存在或无权操作"));
    }

    let item_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM menu_item WHERE category_id = $1 AND tenant_id = $2",
    )
    .bind(category_id)
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await?;
    if item_count > 0 {
        return Err(AppError::business("该分类下仍有菜品，请先删除或移动菜品"));
    }

    sqlx::query("DELETE FROM menu_category WHERE id = $1 AND tenant_id = $2")
        .bind(category_id)
        .bind(tenant_id)
        .execute(&state.pool)
        .await?;
    Ok(ApiResponse::<()>::success_empty("删除成功"))
}

async fn next_sort_order(state: &AppState, tenant_id: i64) -> Result<i32, AppError> {
    let max = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT MAX(sort_order) FROM menu_category WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await?;
    Ok(max.unwrap_or(0) + 1)
}
