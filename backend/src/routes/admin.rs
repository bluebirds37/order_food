use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::Json;
use chrono::{Duration, Local};
use rust_decimal::Decimal;

use crate::auth::{
    create_session, find_tenant_by_code, find_tenant_by_id, normalize_tenant_code,
    resolve_tenant, tenant_selector, verify_password, AccountType, CurrentUser,
};
use crate::error::{ApiJson, AppError};
use crate::models::{
    AdminAccountRow, AdminLoginRequest, AdminLoginView, DashboardView, OrderRow, TenantEntryView,
};
use crate::response::ApiResponse;
use crate::routes::business::load_status;
use crate::routes::orders::load_order_views;
use crate::routes::required_text;
use crate::routes::tenants::generate_tenant_qr;
use crate::state::AppState;

pub async fn login(
    State(state): State<AppState>,
    ApiJson(request): ApiJson<AdminLoginRequest>,
) -> Result<Json<ApiResponse<AdminLoginView>>, AppError> {
    let username = required_text(request.username.clone(), "用户名不能为空")?;
    let password = required_text(request.password.clone(), "密码不能为空")?;

    let account = sqlx::query_as::<_, AdminAccountRow>(
        r#"
SELECT id, username, password_hash, display_name, role, tenant_id, enabled
FROM admin_account
WHERE username = $1
"#,
    )
    .bind(&username)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::business("用户名或密码错误"))?;

    if !account.enabled {
        return Err(AppError::forbidden("账号已停用"));
    }
    if !verify_password(&account.password_hash, &password) {
        return Err(AppError::business("用户名或密码错误"));
    }

    let role = account.role.clone();
    let account_type = match role.as_str() {
        "SUPER_ADMIN" => AccountType::SuperAdmin,
        "TENANT_ADMIN" => AccountType::TenantAdmin,
        _ => return Err(AppError::internal()),
    };

    let requested_code = request
        .tenant_code
        .as_deref()
        .map(normalize_tenant_code)
        .transpose()?;

    let (tenant_id, tenant_code, tenant_name) = if account_type == AccountType::TenantAdmin {
        let tenant = find_tenant_by_id(
            &state.pool,
            account
                .tenant_id
                .ok_or_else(|| AppError::internal())?,
        )
        .await?;
        if !tenant.enabled {
            return Err(AppError::forbidden("租户已停用"));
        }
        if let Some(code) = requested_code.as_deref() {
            if code != tenant.code {
                return Err(AppError::business("该账号不属于当前门店"));
            }
        }
        (Some(tenant.id), Some(tenant.code), Some(tenant.name))
    } else {
        match requested_code.as_deref() {
            Some(code) => {
                let tenant = find_tenant_by_code(&state.pool, code).await?;
                if !tenant.enabled {
                    return Err(AppError::forbidden("租户已停用"));
                }
                (Some(tenant.id), Some(tenant.code), Some(tenant.name))
            }
            None => (None, None, None),
        }
    };

    let token = create_session(
        &state.pool,
        account_type,
        account.id,
        account.tenant_id,
        state.token_ttl_hours,
    )
    .await?;

    Ok(ApiResponse::success_with_message(
        "登录成功",
        AdminLoginView {
            admin_id: account.id,
            username: account.username,
            display_name: account.display_name,
            token,
            role,
            tenant_id,
            tenant_code,
            tenant_name,
        },
    ))
}

#[derive(sqlx::FromRow)]
struct TodayStats {
    count: i64,
    revenue: Decimal,
}

pub async fn dashboard(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<DashboardView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;

    let status = load_status(&state.pool, tenant_id).await?;

    let start = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("合法的一天开始时间");
    let end = start + Duration::days(1);

    let stats = sqlx::query_as::<_, TodayStats>(
        r#"
SELECT COUNT(*) AS count,
       COALESCE(SUM(total_amount), 0) AS revenue
FROM food_order
WHERE tenant_id = $1 AND created_time >= $2 AND created_time < $3
"#,
    )
    .bind(tenant_id)
    .bind(start)
    .bind(end)
    .fetch_one(&state.pool)
    .await?;

    let available_menu_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM menu_item WHERE tenant_id = $1 AND available = TRUE",
    )
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await?;

    let latest_rows = sqlx::query_as::<_, OrderRow>(
        r#"
SELECT id, order_no, customer_name, customer_phone, desk_code, status, total_amount, remark, created_time
FROM food_order
WHERE tenant_id = $1
ORDER BY created_time DESC
LIMIT 20
"#,
    )
    .bind(tenant_id)
    .fetch_all(&state.pool)
    .await?;
    let latest_orders = load_order_views(&state.pool, latest_rows).await?;

    Ok(ApiResponse::success(DashboardView {
        store_name: status.store_name,
        open: status.open,
        notice: status.notice,
        intro: status.intro,
        today_order_count: stats.count,
        today_revenue: stats.revenue,
        available_menu_count,
        latest_orders,
    }))
}

pub async fn tenant_info(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<TenantEntryView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
    let tenant = find_tenant_by_id(&state.pool, tenant_id).await?;
    let status = load_status(&state.pool, tenant_id).await?;
    Ok(ApiResponse::success(TenantEntryView {
        tenant_id: tenant.id,
        code: tenant.code,
        name: tenant.name,
        store_name: status.store_name,
        intro: status.intro,
        open: status.open,
        notice: status.notice,
    }))
}

pub async fn tenant_qrcode(
    auth: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<crate::models::ImageView>>, AppError> {
    auth.require_admin()?;
    let (tenant_id_param, tenant_code) = tenant_selector(&params);
    let tenant_id =
        resolve_tenant(&state, &auth, tenant_id_param, tenant_code.as_deref()).await?;
    let tenant = find_tenant_by_id(&state.pool, tenant_id).await?;
    let image = generate_tenant_qr(&state, &tenant.code).await?;
    Ok(ApiResponse::success(image))
}
