use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::Json;
use rust_decimal::Decimal;

use crate::auth::{resolve_public_tenant, resolve_tenant, tenant_selector, CurrentUser};
use crate::error::{ApiJson, AppError};
use crate::models::{
    BusinessStatusRow, BusinessStatusView, MenuCategoryView, MenuItemRequest, MenuItemView,
    MenuOverviewView, TenantInfoView,
};
use crate::response::ApiResponse;
use crate::routes::required_text;
use crate::state::AppState;

const ITEM_SELECT: &str = r#"
SELECT mi.id,
       mi.category_id,
       mc.name AS category_name,
       mi.name,
       mi.price,
       mi.description,
       mi.image_url,
       mi.detail_images,
       mi.options,
       mi.recommended,
       mi.available,
       mi.sort_order
FROM menu_item mi
JOIN menu_category mc ON mc.id = mi.category_id
"#;

pub async fn overview(
    auth: Option<CurrentUser>,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<MenuOverviewView>>, AppError> {
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_public_tenant(&state, auth.as_ref(), tenant_id_param, tenant_code.as_deref(), true)
            .await?;

    let tenant = crate::auth::find_tenant_by_id(&state.pool, tenant_id).await?;

    let business = sqlx::query_as::<_, BusinessStatusRow>(
        "SELECT open, notice, store_name, intro FROM tenant_business_status WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await?;

    let categories = sqlx::query_as::<_, MenuCategoryView>(
        "SELECT id, name, sort_order FROM menu_category WHERE tenant_id = $1 ORDER BY sort_order ASC, id ASC",
    )
    .bind(tenant_id)
    .fetch_all(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, MenuItemView>(
        "SELECT mi.id, mi.category_id, mc.name AS category_name, mi.name, mi.price, mi.description, mi.image_url, mi.detail_images, mi.options, mi.recommended, mi.available, mi.sort_order
         FROM menu_item mi JOIN menu_category mc ON mc.id = mi.category_id
         WHERE mi.tenant_id = $1
         ORDER BY mi.sort_order ASC, mi.id ASC",
    )
    .bind(tenant_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(ApiResponse::success(MenuOverviewView {
        tenant: TenantInfoView {
            id: tenant.id,
            code: tenant.code,
            name: tenant.name,
        },
        business_status: BusinessStatusView::from_row(business),
        categories,
        menu_items: items,
    }))
}

pub async fn list_items(
    auth: Option<CurrentUser>,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<MenuItemView>>>, AppError> {
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_public_tenant(&state, auth.as_ref(), tenant_id_param, tenant_code.as_deref(), true)
            .await?;
    let items = load_items(&state, tenant_id).await?;
    Ok(ApiResponse::success(items))
}

pub async fn create_item(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    ApiJson(request): ApiJson<MenuItemRequest>,
) -> Result<Json<ApiResponse<MenuItemView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;

    let (category_id, name, price, description, image_url, sort_order, detail_images, options) =
        validate(&request)?;
    ensure_category_exists(&state, tenant_id, category_id).await?;

    let item_id = sqlx::query_scalar::<_, i64>(
        r#"
INSERT INTO menu_item
    (tenant_id, category_id, name, price, description, image_url, detail_images, options, recommended, available, sort_order)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
RETURNING id
"#,
    )
    .bind(tenant_id)
    .bind(category_id)
    .bind(name)
    .bind(price)
    .bind(description)
    .bind(image_url)
    .bind(detail_images)
    .bind(options)
    .bind(request.recommended)
    .bind(request.available)
    .bind(sort_order)
    .fetch_one(&state.pool)
    .await?;

    let item = fetch_item(&state, tenant_id, item_id).await?;
    Ok(ApiResponse::success_with_message("新增成功", item))
}

pub async fn update_item(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    ApiJson(request): ApiJson<MenuItemRequest>,
) -> Result<Json<ApiResponse<MenuItemView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
    let item_id = crate::routes::parse_id(&id)?;

    let (category_id, name, price, description, image_url, sort_order, detail_images, options) =
        validate(&request)?;
    ensure_category_exists(&state, tenant_id, category_id).await?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM menu_item WHERE id = $1 AND tenant_id = $2)",
    )
    .bind(item_id)
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await?;
    if !exists {
        return Err(AppError::business("菜品不存在或无权操作"));
    }

    sqlx::query(
        r#"
UPDATE menu_item
SET category_id = $1,
    name = $2,
    price = $3,
    description = $4,
    image_url = $5,
    detail_images = $6,
    options = $7,
    recommended = $8,
    available = $9,
    sort_order = $10
WHERE id = $11 AND tenant_id = $12
"#,
    )
    .bind(category_id)
    .bind(name)
    .bind(price)
    .bind(description)
    .bind(image_url)
    .bind(detail_images)
    .bind(options)
    .bind(request.recommended)
    .bind(request.available)
    .bind(sort_order)
    .bind(item_id)
    .bind(tenant_id)
    .execute(&state.pool)
    .await?;

    let item = fetch_item(&state, tenant_id, item_id).await?;
    Ok(ApiResponse::success_with_message("更新成功", item))
}

pub async fn delete_item(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
    let item_id = crate::routes::parse_id(&id)?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM menu_item WHERE id = $1 AND tenant_id = $2)",
    )
    .bind(item_id)
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await?;
    if !exists {
        return Err(AppError::business("菜品不存在或无权操作"));
    }

    let in_use = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM food_order_item WHERE menu_item_id = $1)",
    )
    .bind(item_id)
    .fetch_one(&state.pool)
    .await?;
    if in_use {
        return Err(AppError::business("该菜品已有历史订单，无法删除，可改为停售"));
    }

    sqlx::query("DELETE FROM menu_item WHERE id = $1 AND tenant_id = $2")
        .bind(item_id)
        .bind(tenant_id)
        .execute(&state.pool)
        .await?;
    Ok(ApiResponse::<()>::success_empty("删除成功"))
}

async fn load_items(state: &AppState, tenant_id: i64) -> Result<Vec<MenuItemView>, AppError> {
    Ok(sqlx::query_as::<_, MenuItemView>(
        &format!("{ITEM_SELECT} WHERE mi.tenant_id = $1 ORDER BY mi.sort_order ASC, mi.id ASC"),
    )
    .bind(tenant_id)
    .fetch_all(&state.pool)
    .await?)
}

async fn fetch_item(state: &AppState, tenant_id: i64, id: i64) -> Result<MenuItemView, AppError> {
    sqlx::query_as::<_, MenuItemView>(&format!(
        "{ITEM_SELECT} WHERE mi.id = $1 AND mi.tenant_id = $2"
    ))
    .bind(id)
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::business("菜品不存在"))
}

async fn ensure_category_exists(
    state: &AppState,
    tenant_id: i64,
    category_id: i64,
) -> Result<(), AppError> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM menu_category WHERE id = $1 AND tenant_id = $2)",
    )
    .bind(category_id)
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await?;
    if exists {
        Ok(())
    } else {
        Err(AppError::business("菜品分类不存在或无权操作"))
    }
}

fn validate(
    request: &MenuItemRequest,
) -> Result<(i64, String, Decimal, String, String, i32, serde_json::Value, serde_json::Value), AppError> {
    let category_id = request
        .category_id
        .ok_or_else(|| AppError::business("分类不能为空"))?;
    let name = required_text(request.name.clone(), "菜品名称不能为空")?;
    let price = request
        .price
        .ok_or_else(|| AppError::business("价格不能为空"))?;
    if price <= Decimal::ZERO {
        return Err(AppError::business("价格必须大于0"));
    }
    let description = required_text(request.description.clone(), "菜品描述不能为空")?;
    let image_url = request.image_url.clone().unwrap_or_default().trim().to_string();
    let sort_order = request
        .sort_order
        .ok_or_else(|| AppError::business("排序不能为空"))?;
    let detail_images: Vec<String> = request
        .detail_images
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
        .collect();
    let detail_images = serde_json::to_value(detail_images).map_err(|_| AppError::internal())?;
    let options: Vec<String> = request
        .options
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();
    let options = serde_json::to_value(options).map_err(|_| AppError::internal())?;
    Ok((
        category_id,
        name,
        price,
        description,
        image_url,
        sort_order,
        detail_images,
        options,
    ))
}
