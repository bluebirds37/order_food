use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Local;
use rand::Rng;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::auth::{resolve_tenant, tenant_selector, CurrentUser};
use crate::error::{ApiJson, AppError};
use crate::models::{
    CreateOrderItemRequest, CreateOrderRequest, MenuItemLookupRow, OrderItemView, OrderRow,
    OrderStatusRequest, OrderView,
};
use crate::response::ApiResponse;
use crate::routes::required_text;
use crate::state::AppState;

const ORDER_SELECT: &str = r#"
SELECT id, order_no, customer_name, customer_phone, desk_code, status, total_amount, remark, created_time
FROM food_order
"#;

pub async fn create_order(
    auth: CurrentUser,
    State(state): State<AppState>,
    ApiJson(request): ApiJson<CreateOrderRequest>,
) -> Result<Json<ApiResponse<OrderView>>, AppError> {
    if !auth.is_customer() {
        return Err(AppError::forbidden("请使用顾客账号下单"));
    }
    let tenant_id = auth
        .tenant_id
        .ok_or_else(|| AppError::unauthorized("登录状态缺少租户信息"))?;

    let customer_name = request
        .customer_name
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| auth.display_name.clone());
    if let Some(phone) = request.customer_phone.as_deref().map(str::trim) {
        if !phone.is_empty() && phone != auth.identity {
            return Err(AppError::business("手机号与登录账号不一致"));
        }
    }
    let customer_phone = auth.identity.clone();
    let desk_code = required_text(request.desk_code.clone(), "桌号不能为空")?;
    let item_requests = validate_items(request.items.as_deref())?;

    let open =
        sqlx::query_scalar::<_, bool>("SELECT open FROM tenant_business_status WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_optional(&state.pool)
            .await?
            .unwrap_or(false);
    if !open {
        return Err(AppError::business("门店暂未营业，暂时无法下单"));
    }

    let mut resolved_items = Vec::with_capacity(item_requests.len());
    for item_request in item_requests {
        let menu_item = sqlx::query_as::<_, MenuItemLookupRow>(
            "SELECT id, tenant_id, name, price, available FROM menu_item WHERE id = $1 AND tenant_id = $2",
        )
        .bind(item_request.menu_item_id)
        .bind(tenant_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::business("存在无效菜品"))?;
        if !menu_item.available {
            return Err(AppError::business(format!("{} 当前已停售", menu_item.name)));
        }
        if menu_item.tenant_id != tenant_id {
            return Err(AppError::forbidden("菜品不属于当前门店"));
        }
        resolved_items.push((menu_item, item_request.quantity, item_request.flavor, item_request.remark));
    }

    let now = Local::now().naive_local();
    let order_no = generate_order_no();
    let total_amount = resolved_items
        .iter()
        .fold(Decimal::ZERO, |sum, (item, quantity, _, _)| {
            sum + item.price * Decimal::from(*quantity)
        });

    let mut tx = state.pool.begin().await?;
    let order_id = sqlx::query_scalar::<_, i64>(
        r#"
INSERT INTO food_order
    (tenant_id, customer_id, order_no, customer_name, customer_phone, desk_code, status, total_amount, remark, created_time, updated_time)
VALUES ($1, $2, $3, $4, $5, $6, 'PAID', $7, $8, $9, $9)
RETURNING id
"#,
    )
    .bind(tenant_id)
    .bind(auth.account_id)
    .bind(&order_no)
    .bind(&customer_name)
    .bind(&customer_phone)
    .bind(&desk_code)
    .bind(total_amount)
    .bind(request.remark.clone())
    .bind(now)
    .fetch_one(&mut *tx)
    .await?;

    let mut items = Vec::with_capacity(resolved_items.len());
    for (menu_item, quantity, flavor, remark) in resolved_items {
        sqlx::query(
            r#"
INSERT INTO food_order_item (order_id, menu_item_id, item_name, quantity, price, flavor, remark)
VALUES ($1, $2, $3, $4, $5, $6, $7)
"#,
        )
        .bind(order_id)
        .bind(menu_item.id)
        .bind(&menu_item.name)
        .bind(quantity)
        .bind(menu_item.price)
        .bind(flavor.clone().filter(|value| !value.trim().is_empty()))
        .bind(remark.clone().filter(|value| !value.trim().is_empty()))
        .execute(&mut *tx)
        .await?;

        items.push(OrderItemView {
            menu_item_id: menu_item.id,
            item_name: menu_item.name,
            quantity,
            price: menu_item.price,
            flavor: flavor.filter(|value| !value.trim().is_empty()),
            remark: remark.filter(|value| !value.trim().is_empty()),
        });
    }
    tx.commit().await?;

    Ok(ApiResponse::success_with_message(
        "下单成功",
        OrderView {
            id: order_id,
            order_no,
            customer_name,
            customer_phone,
            desk_code,
            status: "PAID".to_string(),
            total_amount,
            remark: request.remark,
            created_time: now,
            items,
        },
    ))
}

pub async fn append_items(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    ApiJson(request): ApiJson<CreateOrderRequest>,
) -> Result<Json<ApiResponse<OrderView>>, AppError> {
    if !auth.is_customer() {
        return Err(AppError::forbidden("请使用顾客账号加菜"));
    }
    let tenant_id = auth
        .tenant_id
        .ok_or_else(|| AppError::unauthorized("登录状态缺少租户信息"))?;
    let order_id = crate::routes::parse_id(&order_id)?;
    let item_requests = validate_items(request.items.as_deref())?;

    let order = sqlx::query_as::<_, OrderRow>(&format!(
        "{ORDER_SELECT} WHERE id = $1 AND tenant_id = $2 AND customer_id = $3"
    ))
    .bind(order_id)
    .bind(tenant_id)
    .bind(auth.account_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::business("订单不存在"))?;
    if matches!(order.status.as_str(), "COMPLETED" | "CANCELLED") {
        return Err(AppError::business("本轮消费已结束，无法追加菜品"));
    }

    let mut resolved_items = Vec::with_capacity(item_requests.len());
    for item_request in item_requests {
        let menu_item = sqlx::query_as::<_, MenuItemLookupRow>(
            "SELECT id, tenant_id, name, price, available FROM menu_item WHERE id = $1 AND tenant_id = $2",
        )
        .bind(item_request.menu_item_id)
        .bind(tenant_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::business("存在无效菜品"))?;
        if !menu_item.available {
            return Err(AppError::business(format!("{} 当前已停售", menu_item.name)));
        }
        let flavor = item_request
            .flavor
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let remark = item_request
            .remark
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        resolved_items.push((menu_item, item_request.quantity, flavor, remark));
    }

    let mut tx = state.pool.begin().await?;
    let mut added_amount = Decimal::ZERO;
    for (menu_item, quantity, flavor, remark) in &resolved_items {
        let existing = sqlx::query_scalar::<_, Option<i32>>(
            r#"
SELECT quantity FROM food_order_item
WHERE order_id = $1 AND menu_item_id = $2
  AND COALESCE(flavor, '') = $3 AND COALESCE(remark, '') = $4
"#,
        )
        .bind(order_id)
        .bind(menu_item.id)
        .bind(flavor.clone().unwrap_or_default())
        .bind(remark.clone().unwrap_or_default())
        .fetch_optional(&mut *tx)
        .await?
        .flatten();
        if let Some(current) = existing {
            sqlx::query(
                r#"
UPDATE food_order_item SET quantity = $1
WHERE order_id = $2 AND menu_item_id = $3
  AND COALESCE(flavor, '') = $4 AND COALESCE(remark, '') = $5
"#,
            )
            .bind(current + quantity)
            .bind(order_id)
            .bind(menu_item.id)
            .bind(flavor.clone().unwrap_or_default())
            .bind(remark.clone().unwrap_or_default())
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(
                r#"
INSERT INTO food_order_item (order_id, menu_item_id, item_name, quantity, price, flavor, remark)
VALUES ($1, $2, $3, $4, $5, $6, $7)
"#,
            )
            .bind(order_id)
            .bind(menu_item.id)
            .bind(&menu_item.name)
            .bind(quantity)
            .bind(menu_item.price)
            .bind(flavor.clone())
            .bind(remark.clone())
            .execute(&mut *tx)
            .await?;
        }
        added_amount += menu_item.price * Decimal::from(*quantity);
    }

    let new_total = order.total_amount + added_amount;
    sqlx::query(
        "UPDATE food_order SET total_amount = $1, updated_time = $2 WHERE id = $3 AND tenant_id = $4",
    )
    .bind(new_total)
    .bind(Local::now().naive_local())
    .bind(order_id)
    .bind(tenant_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let updated_row = OrderRow {
        total_amount: new_total,
        ..order
    };
    let order_view = assemble_order(&state.pool, updated_row).await?;
    Ok(ApiResponse::success_with_message("加菜成功", order_view))
}

pub async fn detail(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<OrderView>>, AppError> {
    let order_id = crate::routes::parse_id(&order_id)?;
    let order = if auth.is_customer() {
        let tenant_id = auth
            .tenant_id
            .ok_or_else(|| AppError::unauthorized("登录状态缺少租户信息"))?;
        let row = fetch_order_row(&state.pool, tenant_id, order_id, Some(auth.account_id))
            .await?
            .ok_or_else(|| AppError::business("订单不存在"))?;
        assemble_order(&state.pool, row).await?
    } else {
        auth.require_admin()?;
        let (tenant_id_param, tenant_code) = tenant_selector(&params);
        let tenant_id =
            resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
        let row = fetch_order_row(&state.pool, tenant_id, order_id, None)
            .await?
            .ok_or_else(|| AppError::business("订单不存在"))?;
        assemble_order(&state.pool, row).await?
    };
    Ok(ApiResponse::success(order))
}

pub async fn query_by_order_no(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<OrderView>>, AppError> {
    let order_no = params.get("orderNo").ok_or_else(AppError::internal)?;
    let row = if auth.is_customer() {
        let tenant_id = auth
            .tenant_id
            .ok_or_else(|| AppError::unauthorized("登录状态缺少租户信息"))?;
        sqlx::query_as::<_, OrderRow>(&format!(
            "{ORDER_SELECT} WHERE tenant_id = $1 AND customer_id = $2 AND order_no = $3"
        ))
        .bind(tenant_id)
        .bind(auth.account_id)
        .bind(order_no)
        .fetch_optional(&state.pool)
        .await?
    } else {
        auth.require_admin()?;
        let (tenant_id_param, tenant_code) = tenant_selector(&params);
        let tenant_id =
            resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
        sqlx::query_as::<_, OrderRow>(&format!(
            "{ORDER_SELECT} WHERE tenant_id = $1 AND order_no = $2"
        ))
        .bind(tenant_id)
        .bind(order_no)
        .fetch_optional(&state.pool)
        .await?
    }
    .ok_or_else(|| AppError::business("订单不存在"))?;

    let order = assemble_order(&state.pool, row).await?;
    Ok(ApiResponse::success(order))
}

pub async fn history(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<OrderView>>>, AppError> {
    let (tenant_id, phone) = if auth.is_customer() {
        let tenant_id = auth
            .tenant_id
            .ok_or_else(|| AppError::unauthorized("登录状态缺少租户信息"))?;
        (tenant_id, auth.identity.clone())
    } else {
        auth.require_admin()?;
        let (tenant_id_param, tenant_code) = tenant_selector(&params);
        let tenant_id =
            resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
        let phone = params
            .get("phone")
            .cloned()
            .ok_or_else(|| AppError::business("请提供手机号"))?;
        (tenant_id, phone)
    };

    let rows = sqlx::query_as::<_, OrderRow>(&format!(
        "{ORDER_SELECT} WHERE tenant_id = $1 AND customer_phone = $2 ORDER BY created_time DESC"
    ))
    .bind(tenant_id)
    .bind(phone)
    .fetch_all(&state.pool)
    .await?;
    let orders = load_order_views(&state.pool, rows).await?;
    Ok(ApiResponse::success(orders))
}

pub async fn list_orders(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<OrderView>>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
    let rows = sqlx::query_as::<_, OrderRow>(&format!(
        "{ORDER_SELECT} WHERE tenant_id = $1 ORDER BY created_time DESC LIMIT 20"
    ))
    .bind(tenant_id)
    .fetch_all(&state.pool)
    .await?;
    let orders = load_order_views(&state.pool, rows).await?;
    Ok(ApiResponse::success(orders))
}

pub async fn update_status(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    ApiJson(request): ApiJson<OrderStatusRequest>,
) -> Result<Json<ApiResponse<OrderView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
    let order_id = crate::routes::parse_id(&order_id)?;
    let status = parse_status(request.status)?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM food_order WHERE id = $1 AND tenant_id = $2)",
    )
    .bind(order_id)
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await?;
    if !exists {
        return Err(AppError::business("订单不存在或无权操作"));
    }

    sqlx::query(
        "UPDATE food_order SET status = $1, updated_time = $2 WHERE id = $3 AND tenant_id = $4",
    )
    .bind(&status)
    .bind(Local::now().naive_local())
    .bind(order_id)
    .bind(tenant_id)
    .execute(&state.pool)
    .await?;

    let order = load_order(&state.pool, tenant_id, order_id).await?;
    Ok(ApiResponse::success_with_message("状态更新成功", order))
}

fn validate_items(
    items: Option<&[CreateOrderItemRequest]>,
) -> Result<Vec<ValidatedItem>, AppError> {
    let items = match items {
        Some(items) if !items.is_empty() => items,
        _ => return Err(AppError::business("请至少选择一道菜品")),
    };

    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let menu_item_id = item
            .menu_item_id
            .ok_or_else(|| AppError::business("菜品不能为空"))?;
        let quantity = item
            .quantity
            .ok_or_else(|| AppError::business("数量不能为空"))?;
        if quantity < 1 {
            return Err(AppError::business("数量至少为1"));
        }
        result.push(ValidatedItem {
            menu_item_id,
            quantity,
            flavor: item.flavor.clone(),
            remark: item.remark.clone(),
        });
    }
    Ok(result)
}

fn parse_status(status: Option<String>) -> Result<String, AppError> {
    match status {
        None => Err(AppError::business("订单状态不能为空")),
        Some(status)
            if matches!(
                status.as_str(),
                "PENDING_PAYMENT" | "PAID" | "PREPARING" | "COMPLETED" | "CANCELLED"
            ) =>
        {
            Ok(status)
        }
        Some(_) => Err(AppError::bad_request()),
    }
}

async fn fetch_order_row(
    pool: &PgPool,
    tenant_id: i64,
    order_id: i64,
    customer_id: Option<i64>,
) -> Result<Option<OrderRow>, AppError> {
    let row = match customer_id {
        Some(customer_id) => {
            sqlx::query_as::<_, OrderRow>(&format!(
                "{ORDER_SELECT} WHERE id = $1 AND tenant_id = $2 AND customer_id = $3"
            ))
            .bind(order_id)
            .bind(tenant_id)
            .bind(customer_id)
            .fetch_optional(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, OrderRow>(&format!(
                "{ORDER_SELECT} WHERE id = $1 AND tenant_id = $2"
            ))
            .bind(order_id)
            .bind(tenant_id)
            .fetch_optional(pool)
            .await?
        }
    };
    Ok(row)
}

async fn load_order(pool: &PgPool, tenant_id: i64, order_id: i64) -> Result<OrderView, AppError> {
    let row = fetch_order_row(pool, tenant_id, order_id, None)
        .await?
        .ok_or_else(|| AppError::business("订单不存在"))?;
    assemble_order(pool, row).await
}

pub async fn load_order_views(
    pool: &PgPool,
    rows: Vec<OrderRow>,
) -> Result<Vec<OrderView>, AppError> {
    let mut orders = Vec::with_capacity(rows.len());
    for row in rows {
        orders.push(assemble_order(pool, row).await?);
    }
    Ok(orders)
}

async fn assemble_order(pool: &PgPool, row: OrderRow) -> Result<OrderView, AppError> {
    let items = sqlx::query_as::<_, OrderItemView>(
        r#"
SELECT menu_item_id, item_name, quantity, price, flavor, remark
FROM food_order_item
WHERE order_id = $1
ORDER BY id ASC
"#,
    )
    .bind(row.id)
    .fetch_all(pool)
    .await?;

    Ok(OrderView {
        id: row.id,
        order_no: row.order_no,
        customer_name: row.customer_name,
        customer_phone: row.customer_phone,
        desk_code: row.desk_code,
        status: row.status,
        total_amount: row.total_amount,
        remark: row.remark,
        created_time: row.created_time,
        items,
    })
}

fn generate_order_no() -> String {
    let now = Local::now();
    let suffix = rand::thread_rng().gen_range(100..1000);
    format!("OD{}{:03}", now.format("%Y%m%d%H%M%S"), suffix)
}

struct ValidatedItem {
    menu_item_id: i64,
    quantity: i32,
    flavor: Option<String>,
    remark: Option<String>,
}
