use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::error::AppError;
use crate::state::AppState;

pub mod admin;
pub mod auth;
pub mod business;
pub mod categories;
pub mod desks;
pub mod images;
pub mod kitchen;
pub mod menu;
pub mod orders;
pub mod super_admin;
pub mod tenants;

pub fn router() -> Router<AppState> {
    Router::new()
        // 三类登录与登录态
        .route("/api/auth/customer/register", post(auth::customer_register))
        .route("/api/auth/customer/login", post(auth::customer_login))
        .route("/api/auth/customer/wx-login", post(auth::customer_wx_login))
        .route("/api/auth/customer/bind-phone", post(auth::bind_phone))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/admin/login", post(admin::login))
        .route("/api/admin/logout", post(auth::logout))
        .route("/api/admin/dashboard", get(admin::dashboard))
        .route("/api/admin/tenant", get(admin::tenant_info))
        .route("/api/admin/tenant/qrcode", get(admin::tenant_qrcode))
        // 小程序扫码入口（公开）
        .route("/api/tenants/{code}/entry", get(tenants::entry))
        .route("/api/desks", get(desks::list_desks))
        .route("/api/desks", post(desks::create_desk))
        .route("/api/desks/{id}", put(desks::update_desk))
        .route("/api/desks/{id}", delete(desks::delete_desk))
        .route("/api/kitchen/pending", get(kitchen::kitchen_pending))
        // 菜单
        .route("/api/menu/overview", get(menu::overview))
        .route("/api/menu/items", get(menu::list_items))
        .route("/api/menu/items", post(menu::create_item))
        .route("/api/menu/items/{id}", put(menu::update_item))
        .route("/api/menu/items/{id}", delete(menu::delete_item))
        .route("/api/menu/categories", post(categories::create_category))
        .route("/api/menu/categories/{id}", put(categories::update_category))
        .route("/api/menu/categories/{id}", delete(categories::delete_category))
        // 订单
        .route("/api/orders", post(orders::create_order))
        .route("/api/orders", get(orders::list_orders))
        .route("/api/orders/query", get(orders::query_by_order_no))
        .route("/api/orders/history", get(orders::history))
        .route("/api/orders/{order_id}", get(orders::detail))
        .route("/api/orders/{order_id}/items", post(orders::append_items))
        .route("/api/orders/{order_id}/status", put(orders::update_status))
        // 营业状态与店铺介绍
        .route("/api/business/status", get(business::get_status))
        .route("/api/business/status", put(business::update_status))
        // 统一图片存储
        .route("/api/images", post(images::upload))
        .route("/api/images/{filename}", put(images::overwrite))
        .route("/api/images/{filename}", delete(images::remove))
        // 超级管理员专属：租户、全局概览、账号管理
        .route(
            "/api/super/tenants",
            get(super_admin::list_tenants).post(super_admin::create_tenant),
        )
        .route("/api/super/tenants/{id}", put(super_admin::update_tenant))
        .route(
            "/api/super/tenants/{id}/qrcode",
            get(super_admin::tenant_qrcode),
        )
        .route("/api/super/overview", get(super_admin::overview))
        .route(
            "/api/super/accounts",
            get(super_admin::list_accounts).post(super_admin::create_account),
        )
        .route(
            "/api/super/accounts/{id}",
            put(super_admin::update_account).delete(super_admin::delete_account),
        )
}

/// 校验必填文本，错误提示与旧后端保持一致。
pub fn required_text(value: Option<String>, message: &str) -> Result<String, AppError> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(value.trim().to_string()),
        _ => Err(AppError::business(message)),
    }
}

/// 路径参数 ID 解析失败时退化为旧后端的通用错误提示。
pub fn parse_id(raw: &str) -> Result<i64, AppError> {
    raw.parse::<i64>().map_err(|_| AppError::internal())
}
