use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{Duration, Local, NaiveDateTime};
use rust_decimal::Decimal;

use crate::auth::{find_tenant_by_id, hash_password, normalize_tenant_code, CurrentUser};
use crate::error::{ApiJson, AppError};
use crate::models::{
    AccountCreateRequest, AccountUpdateRequest, AdminAccountView, OrderRow, SuperOrderView,
    SuperOverviewView, SuperTenantView, TenantCreateRequest, TenantUpdateRequest,
};
use crate::response::ApiResponse;
use crate::routes::orders::load_order_views;
use crate::routes::required_text;
use crate::routes::tenants::generate_tenant_qr;
use crate::state::AppState;

// ========== 租户管理 ==========

#[derive(sqlx::FromRow)]
struct SuperTenantRow {
    id: i64,
    code: String,
    name: String,
    enabled: bool,
    store_name: String,
    open: bool,
    notice: String,
    intro: String,
    created_time: NaiveDateTime,
    order_count: i64,
    menu_item_count: i64,
    admin_count: i64,
    customer_count: i64,
}

impl From<SuperTenantRow> for SuperTenantView {
    fn from(row: SuperTenantRow) -> Self {
        Self {
            id: row.id,
            code: row.code,
            name: row.name,
            enabled: row.enabled,
            store_name: row.store_name,
            open: row.open,
            notice: row.notice,
            intro: row.intro,
            created_time: row.created_time,
            order_count: row.order_count,
            menu_item_count: row.menu_item_count,
            admin_count: row.admin_count,
            customer_count: row.customer_count,
        }
    }
}

const TENANT_SELECT: &str = r#"
SELECT t.id,
       t.code,
       t.name,
       t.enabled,
       COALESCE(bs.store_name, t.name) AS store_name,
       COALESCE(bs.open, FALSE) AS open,
       COALESCE(bs.notice, '暂无公告') AS notice,
       COALESCE(bs.intro, '') AS intro,
       t.created_time,
       (SELECT COUNT(*) FROM food_order o WHERE o.tenant_id = t.id) AS order_count,
       (SELECT COUNT(*) FROM menu_item mi WHERE mi.tenant_id = t.id) AS menu_item_count,
       (SELECT COUNT(*) FROM admin_account aa WHERE aa.tenant_id = t.id) AS admin_count,
       (SELECT COUNT(*) FROM customer_account ca WHERE ca.tenant_id = t.id) AS customer_count
FROM tenant t
LEFT JOIN tenant_business_status bs ON bs.tenant_id = t.id
"#;

pub async fn list_tenants(
    auth: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<SuperTenantView>>>, AppError> {
    auth.require_super_admin()?;
    let rows = sqlx::query_as::<_, SuperTenantRow>(&format!(
        "{TENANT_SELECT} ORDER BY t.id DESC"
    ))
    .fetch_all(&state.pool)
    .await?;
    Ok(ApiResponse::success(
        rows.into_iter().map(SuperTenantView::from).collect(),
    ))
}

pub async fn create_tenant(
    auth: CurrentUser,
    State(state): State<AppState>,
    ApiJson(request): ApiJson<TenantCreateRequest>,
) -> Result<Json<ApiResponse<SuperTenantView>>, AppError> {
    auth.require_super_admin()?;

    let code = normalize_tenant_code(&required_text(request.code.clone(), "租户编码不能为空")?)?;
    let name = required_text(request.name.clone(), "租户名称不能为空")?;
    let store_name = request
        .store_name
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| name.clone());
    let notice = request
        .notice
        .clone()
        .unwrap_or_else(|| "欢迎光临".to_string());
    let intro = request.intro.clone().unwrap_or_default();
    let open = request.open.unwrap_or(true);

    let duplicate =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM tenant WHERE code = $1)")
            .bind(&code)
            .fetch_one(&state.pool)
            .await?;
    if duplicate {
        return Err(AppError::business("租户编码已存在"));
    }

    let admin = prepare_tenant_admin(&request)?;

    let mut tx = state.pool.begin().await?;
    let tenant_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO tenant (code, name, enabled) VALUES ($1, $2, TRUE) RETURNING id",
    )
    .bind(&code)
    .bind(&name)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
INSERT INTO tenant_business_status (tenant_id, open, notice, store_name, intro)
VALUES ($1, $2, $3, $4, $5)
"#,
    )
    .bind(tenant_id)
    .bind(open)
    .bind(&notice)
    .bind(&store_name)
    .bind(&intro)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO menu_category (tenant_id, name, sort_order) VALUES ($1, '默认分类', 1)",
    )
    .bind(tenant_id)
    .execute(&mut *tx)
    .await?;

    if let Some((username, password_hash, display_name)) = admin {
        let username_taken =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM admin_account WHERE username = $1)")
                .bind(&username)
                .fetch_one(&mut *tx)
                .await?;
        if username_taken {
            return Err(AppError::business("管理员账号已存在"));
        }
        sqlx::query(
            r#"
INSERT INTO admin_account (username, password_hash, display_name, role, tenant_id, enabled)
VALUES ($1, $2, $3, 'TENANT_ADMIN', $4, TRUE)
"#,
        )
        .bind(&username)
        .bind(&password_hash)
        .bind(&display_name)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    let view = fetch_super_tenant(&state, tenant_id).await?;
    Ok(ApiResponse::success_with_message("租户创建成功", view))
}

pub async fn update_tenant(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ApiJson(request): ApiJson<TenantUpdateRequest>,
) -> Result<Json<ApiResponse<SuperTenantView>>, AppError> {
    auth.require_super_admin()?;
    let tenant_id = crate::routes::parse_id(&id)?;
    let tenant = find_tenant_by_id(&state.pool, tenant_id).await?;

    let name = match request.name.clone() {
        Some(value) if !value.trim().is_empty() => value.trim().to_string(),
        Some(_) => return Err(AppError::business("租户名称不能为空")),
        None => tenant.name,
    };
    let enabled = request.enabled.unwrap_or(tenant.enabled);

    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE tenant SET name = $1, enabled = $2 WHERE id = $3")
        .bind(&name)
        .bind(enabled)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;

    let status = sqlx::query_as::<_, (bool, String, String, String)>(
        "SELECT open, notice, store_name, intro FROM tenant_business_status WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_optional(&mut *tx)
    .await?;

    let (open, notice, store_name, intro) = status.unwrap_or((
        false,
        "暂无公告".to_string(),
        name.clone(),
        String::new(),
    ));
    let store_name = request
        .store_name
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or(store_name);
    let notice = request.notice.clone().unwrap_or(notice);
    let intro = request.intro.clone().unwrap_or(intro);

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
    .bind(open)
    .bind(&notice)
    .bind(&store_name)
    .bind(&intro)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let view = fetch_super_tenant(&state, tenant_id).await?;
    Ok(ApiResponse::success_with_message("租户已更新", view))
}

pub async fn tenant_qrcode(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<crate::models::ImageView>>, AppError> {
    auth.require_super_admin()?;
    let tenant_id = crate::routes::parse_id(&id)?;
    let tenant = find_tenant_by_id(&state.pool, tenant_id).await?;
    let image = generate_tenant_qr(&state, &tenant.code).await?;
    Ok(ApiResponse::success(image))
}

async fn fetch_super_tenant(state: &AppState, tenant_id: i64) -> Result<SuperTenantView, AppError> {
    let row = sqlx::query_as::<_, SuperTenantRow>(&format!(
        "{TENANT_SELECT} WHERE t.id = $1"
    ))
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::business("租户不存在"))?;
    Ok(SuperTenantView::from(row))
}

fn prepare_tenant_admin(
    request: &TenantCreateRequest,
) -> Result<Option<(String, String, String)>, AppError> {
    let username = match request.admin_username.as_deref().map(str::trim) {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => return Ok(None),
    };
    let password = required_text(request.admin_password.clone(), "管理员密码不能为空")?;
    if password.len() < 6 {
        return Err(AppError::business("管理员密码至少 6 位"));
    }
    let display_name = request
        .admin_display_name
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| username.clone());
    let hash = hash_password(&password)?;
    Ok(Some((username, hash, display_name)))
}

// ========== 全局概览 ==========

#[derive(sqlx::FromRow)]
struct SuperOrderRow {
    tenant_id: i64,
    tenant_code: String,
    tenant_name: String,
    id: i64,
    order_no: String,
    customer_name: String,
    customer_phone: String,
    desk_code: String,
    status: String,
    total_amount: Decimal,
    remark: Option<String>,
    created_time: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
struct GlobalStats {
    total_tenants: i64,
    enabled_tenants: i64,
    total_orders: i64,
    total_revenue: Decimal,
    total_customers: i64,
    total_menu_items: i64,
}

pub async fn overview(
    auth: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SuperOverviewView>>, AppError> {
    auth.require_super_admin()?;

    let stats = sqlx::query_as::<_, GlobalStats>(
        r#"
SELECT (SELECT COUNT(*) FROM tenant) AS total_tenants,
       (SELECT COUNT(*) FROM tenant WHERE enabled = TRUE) AS enabled_tenants,
       (SELECT COUNT(*) FROM food_order) AS total_orders,
       (SELECT COALESCE(SUM(total_amount), 0) FROM food_order) AS total_revenue,
       (SELECT COUNT(*) FROM customer_account) AS total_customers,
       (SELECT COUNT(*) FROM menu_item) AS total_menu_items
"#,
    )
    .fetch_one(&state.pool)
    .await?;

    let start = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("合法的一天开始时间");
    let end = start + Duration::days(1);
    let today_orders =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM food_order WHERE created_time >= $1 AND created_time < $2")
            .bind(start)
            .bind(end)
            .fetch_one(&state.pool)
            .await?;
    let today_revenue: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total_amount), 0) FROM food_order WHERE created_time >= $1 AND created_time < $2",
    )
    .bind(start)
    .bind(end)
    .fetch_one(&state.pool)
    .await?;

    let rows = sqlx::query_as::<_, SuperOrderRow>(
        r#"
SELECT o.tenant_id,
       t.code AS tenant_code,
       t.name AS tenant_name,
       o.id,
       o.order_no,
       o.customer_name,
       o.customer_phone,
       o.desk_code,
       o.status,
       o.total_amount,
       o.remark,
       o.created_time
FROM food_order o
JOIN tenant t ON t.id = o.tenant_id
ORDER BY o.created_time DESC
LIMIT 10
"#,
    )
    .fetch_all(&state.pool)
    .await?;

    let order_rows: Vec<OrderRow> = rows
        .iter()
        .map(|row| OrderRow {
            id: row.id,
            order_no: row.order_no.clone(),
            customer_name: row.customer_name.clone(),
            customer_phone: row.customer_phone.clone(),
            desk_code: row.desk_code.clone(),
            status: row.status.clone(),
            total_amount: row.total_amount,
            remark: row.remark.clone(),
            created_time: row.created_time,
        })
        .collect();
    let orders = load_order_views(&state.pool, order_rows).await?;
    let recent_orders = rows
        .into_iter()
        .zip(orders)
        .map(|(row, order)| SuperOrderView {
            tenant_id: row.tenant_id,
            tenant_code: row.tenant_code,
            tenant_name: row.tenant_name,
            order,
        })
        .collect();

    Ok(ApiResponse::success(SuperOverviewView {
        total_tenants: stats.total_tenants,
        enabled_tenants: stats.enabled_tenants,
        total_orders: stats.total_orders,
        today_orders,
        total_revenue: stats.total_revenue,
        today_revenue,
        total_customers: stats.total_customers,
        total_menu_items: stats.total_menu_items,
        recent_orders,
    }))
}

// ========== 账号管理 ==========

#[derive(sqlx::FromRow)]
struct AdminAccountRowView {
    id: i64,
    username: String,
    display_name: String,
    role: String,
    tenant_id: Option<i64>,
    tenant_code: Option<String>,
    tenant_name: Option<String>,
    enabled: bool,
    created_time: NaiveDateTime,
}

impl From<AdminAccountRowView> for AdminAccountView {
    fn from(row: AdminAccountRowView) -> Self {
        Self {
            id: row.id,
            username: row.username,
            display_name: row.display_name,
            role: row.role,
            tenant_id: row.tenant_id,
            tenant_code: row.tenant_code,
            tenant_name: row.tenant_name,
            enabled: row.enabled,
            created_time: row.created_time,
        }
    }
}

const ACCOUNT_SELECT: &str = r#"
SELECT aa.id,
       aa.username,
       aa.display_name,
       aa.role,
       aa.tenant_id,
       t.code AS tenant_code,
       t.name AS tenant_name,
       aa.enabled,
       aa.created_time
FROM admin_account aa
LEFT JOIN tenant t ON t.id = aa.tenant_id
"#;

pub async fn list_accounts(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<AdminAccountView>>>, AppError> {
    auth.require_super_admin()?;
    let rows = sqlx::query_as::<_, AdminAccountRowView>(&format!(
        "{ACCOUNT_SELECT} ORDER BY aa.id ASC"
    ))
    .fetch_all(&state.pool)
    .await?;

    let role_filter = params.get("role").map(|value| value.to_ascii_uppercase());
    let tenant_filter = params
        .get("tenantId")
        .and_then(|value| value.parse::<i64>().ok());

    let accounts = rows
        .into_iter()
        .filter(|row| {
            role_filter
                .as_deref()
                .map(|role| row.role == role)
                .unwrap_or(true)
        })
        .filter(|row| {
            tenant_filter
                .map(|tenant_id| row.tenant_id == Some(tenant_id))
                .unwrap_or(true)
        })
        .map(AdminAccountView::from)
        .collect();
    Ok(ApiResponse::success(accounts))
}

pub async fn create_account(
    auth: CurrentUser,
    State(state): State<AppState>,
    ApiJson(request): ApiJson<AccountCreateRequest>,
) -> Result<Json<ApiResponse<AdminAccountView>>, AppError> {
    auth.require_super_admin()?;

    let username = required_text(request.username.clone(), "账号不能为空")?;
    let password = required_text(request.password.clone(), "密码不能为空")?;
    if password.len() < 6 {
        return Err(AppError::business("密码至少 6 位"));
    }
    let display_name = request
        .display_name
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| username.clone());
    let role = request
        .role
        .clone()
        .unwrap_or_else(|| "TENANT_ADMIN".to_string())
        .to_ascii_uppercase();

    let tenant_id = match role.as_str() {
        "SUPER_ADMIN" => None,
        "TENANT_ADMIN" => {
            let tenant_id = request
                .tenant_id
                .ok_or_else(|| AppError::business("租户管理员必须指定租户"))?;
            find_tenant_by_id(&state.pool, tenant_id).await?;
            Some(tenant_id)
        }
        _ => return Err(AppError::business("角色不合法")),
    };

    let duplicate =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM admin_account WHERE username = $1)")
            .bind(&username)
            .fetch_one(&state.pool)
            .await?;
    if duplicate {
        return Err(AppError::business("账号已存在"));
    }

    let password_hash = hash_password(&password)?;
    let account_id = sqlx::query_scalar::<_, i64>(
        r#"
INSERT INTO admin_account (username, password_hash, display_name, role, tenant_id, enabled)
VALUES ($1, $2, $3, $4, $5, $6)
RETURNING id
"#,
    )
    .bind(&username)
    .bind(password_hash)
    .bind(&display_name)
    .bind(&role)
    .bind(tenant_id)
    .bind(request.enabled.unwrap_or(true))
    .fetch_one(&state.pool)
    .await?;

    let view = fetch_account(&state, account_id).await?;
    Ok(ApiResponse::success_with_message("账号创建成功", view))
}

pub async fn update_account(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ApiJson(request): ApiJson<AccountUpdateRequest>,
) -> Result<Json<ApiResponse<AdminAccountView>>, AppError> {
    auth.require_super_admin()?;
    let account_id = crate::routes::parse_id(&id)?;
    let account = sqlx::query_as::<_, (String, Option<i64>, bool, String)>(
        "SELECT role, tenant_id, enabled, display_name FROM admin_account WHERE id = $1",
    )
    .bind(account_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::business("账号不存在"))?;
    let (role, tenant_id, enabled, display_name) = account;

    let display_name = request
        .display_name
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or(display_name);
    let enabled = request.enabled.unwrap_or(enabled);

    if enabled == false && account_id == auth.account_id {
        return Err(AppError::business("不能停用当前登录账号"));
    }
    if enabled == false && role == "SUPER_ADMIN" && enabled_super_admin_count(&state).await? <= 1 {
        return Err(AppError::business("至少需要保留一个启用状态的超级管理员"));
    }

    let password_hash = match request.password.as_deref().map(str::trim) {
        Some(password) if !password.is_empty() => {
            if password.len() < 6 {
                return Err(AppError::business("密码至少 6 位"));
            }
            Some(hash_password(password)?)
        }
        _ => None,
    };

    sqlx::query(
        r#"
UPDATE admin_account
SET display_name = $1,
    enabled = $2,
    password_hash = COALESCE($3, password_hash)
WHERE id = $4
"#,
    )
    .bind(&display_name)
    .bind(enabled)
    .bind(password_hash)
    .bind(account_id)
    .execute(&state.pool)
    .await?;

    if !enabled {
        remove_account_sessions(&state, account_id, tenant_id).await?;
    }

    let view = fetch_account(&state, account_id).await?;
    Ok(ApiResponse::success_with_message("账号已更新", view))
}

pub async fn delete_account(
    auth: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    auth.require_super_admin()?;
    let account_id = crate::routes::parse_id(&id)?;
    if account_id == auth.account_id {
        return Err(AppError::business("不能删除当前登录账号"));
    }

    let account = sqlx::query_as::<_, (String, Option<i64>)>(
        "SELECT role, tenant_id FROM admin_account WHERE id = $1",
    )
    .bind(account_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::business("账号不存在"))?;
    let (role, tenant_id) = account;

    if role == "SUPER_ADMIN" && enabled_super_admin_count(&state).await? <= 1 {
        return Err(AppError::business("至少需要保留一个启用状态的超级管理员"));
    }

    remove_account_sessions(&state, account_id, tenant_id).await?;
    sqlx::query("DELETE FROM admin_account WHERE id = $1")
        .bind(account_id)
        .execute(&state.pool)
        .await?;
    Ok(ApiResponse::<()>::success_empty("账号已删除"))
}

async fn fetch_account(state: &AppState, account_id: i64) -> Result<AdminAccountView, AppError> {
    let row = sqlx::query_as::<_, AdminAccountRowView>(&format!(
        "{ACCOUNT_SELECT} WHERE aa.id = $1"
    ))
    .bind(account_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::business("账号不存在"))?;
    Ok(AdminAccountView::from(row))
}

async fn enabled_super_admin_count(state: &AppState) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM admin_account WHERE role = 'SUPER_ADMIN' AND enabled = TRUE",
    )
    .fetch_one(&state.pool)
    .await?)
}

async fn remove_account_sessions(
    state: &AppState,
    account_id: i64,
    tenant_id: Option<i64>,
) -> Result<(), AppError> {
    let account_type = if tenant_id.is_some() {
        "TENANT_ADMIN"
    } else {
        "SUPER_ADMIN"
    };
    sqlx::query("DELETE FROM auth_session WHERE account_type = $1 AND account_id = $2")
        .bind(account_type)
        .bind(account_id)
        .execute(&state.pool)
        .await?;
    Ok(())
}
