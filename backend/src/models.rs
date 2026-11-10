use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};

/// 时间字段沿用旧后端的 `yyyy-MM-dd HH:mm:ss` 格式。
pub fn serialize_time<S>(value: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.format("%Y-%m-%d %H:%M:%S").to_string())
}

// ========== 请求体 ==========

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuItemRequest {
    pub category_id: Option<i64>,
    pub name: Option<String>,
    pub price: Option<Decimal>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub detail_images: Option<Vec<String>>,
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub recommended: bool,
    #[serde(default)]
    pub available: bool,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRequest {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeskRequest {
    pub desk_code: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeskView {
    pub id: i64,
    pub desk_code: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KitchenItemView {
    pub item_name: String,
    pub flavor: String,
    pub remark: String,
    pub quantity: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KitchenOrderView {
    pub order_id: i64,
    pub order_no: String,
    pub status: String,
    pub items: Vec<KitchenItemView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KitchenDeskView {
    pub desk_code: String,
    pub order_count: i32,
    pub earliest_at: String,
    pub orders: Vec<KitchenOrderView>,
    /// 该桌所有订单中"菜品+口味+备注"完全一致的合并清单，用于一次性出餐
    pub merged_items: Vec<KitchenItemView>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessStatusRequest {
    #[serde(default)]
    pub open: bool,
    pub store_name: Option<String>,
    pub notice: Option<String>,
    pub intro: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminLoginRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub tenant_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerRegisterRequest {
    pub tenant_code: Option<String>,
    pub phone: Option<String>,
    pub password: Option<String>,
    pub nickname: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerLoginRequest {
    pub tenant_code: Option<String>,
    pub phone: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WxLoginRequest {
    pub tenant_code: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindPhoneRequest {
    pub phone: Option<String>,
    pub phone_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatusRequest {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub desk_code: Option<String>,
    pub remark: Option<String>,
    pub items: Option<Vec<CreateOrderItemRequest>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderItemRequest {
    pub menu_item_id: Option<i64>,
    pub quantity: Option<i32>,
    pub flavor: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantCreateRequest {
    pub code: Option<String>,
    pub name: Option<String>,
    pub store_name: Option<String>,
    pub notice: Option<String>,
    pub intro: Option<String>,
    pub open: Option<bool>,
    pub admin_username: Option<String>,
    pub admin_password: Option<String>,
    pub admin_display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantUpdateRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub store_name: Option<String>,
    pub notice: Option<String>,
    pub intro: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCreateRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub tenant_id: Option<i64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdateRequest {
    pub display_name: Option<String>,
    pub password: Option<String>,
    pub enabled: Option<bool>,
}

// ========== 数据库行 ==========

#[derive(Debug, sqlx::FromRow)]
pub struct BusinessStatusRow {
    pub open: bool,
    pub notice: Option<String>,
    pub store_name: Option<String>,
    pub intro: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AdminAccountRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub role: String,
    pub tenant_id: Option<i64>,
    pub enabled: bool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CustomerRow {
    pub id: i64,
    pub tenant_id: i64,
    pub phone: String,
    pub password_hash: String,
    pub nickname: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OrderRow {
    pub id: i64,
    pub order_no: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub desk_code: String,
    pub status: String,
    pub total_amount: Decimal,
    pub remark: Option<String>,
    pub created_time: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
pub struct MenuItemLookupRow {
    pub id: i64,
    pub tenant_id: i64,
    pub name: String,
    pub price: Decimal,
    pub available: bool,
}

// ========== 响应体 ==========

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantEntryView {
    pub tenant_id: i64,
    pub code: String,
    pub name: String,
    pub store_name: String,
    pub intro: String,
    pub open: bool,
    pub notice: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminLoginView {
    pub admin_id: i64,
    pub username: String,
    pub display_name: String,
    pub token: String,
    pub role: String,
    pub tenant_id: Option<i64>,
    pub tenant_code: Option<String>,
    pub tenant_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerAuthView {
    pub token: String,
    pub customer_id: i64,
    pub nickname: String,
    pub phone: String,
    pub tenant_id: i64,
    pub tenant_code: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeView {
    pub account_type: String,
    pub account_id: i64,
    pub display_name: String,
    pub identity: String,
    pub tenant_id: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MenuCategoryView {
    pub id: i64,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MenuItemView {
    pub id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub name: String,
    pub price: Decimal,
    pub description: String,
    pub image_url: String,
    pub detail_images: serde_json::Value,
    pub options: serde_json::Value,
    pub recommended: bool,
    pub available: bool,
    pub sort_order: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessStatusView {
    pub open: bool,
    pub notice: String,
    pub store_name: String,
    pub intro: String,
}

impl BusinessStatusView {
    pub fn from_row(row: Option<BusinessStatusRow>) -> Self {
        match row {
            Some(row) => Self {
                open: row.open,
                notice: row.notice.unwrap_or_else(|| "暂无公告".to_string()),
                store_name: row.store_name.unwrap_or_else(|| "门店".to_string()),
                intro: row.intro.unwrap_or_default(),
            },
            None => Self {
                open: false,
                notice: "暂无公告".to_string(),
                store_name: "门店".to_string(),
                intro: String::new(),
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuOverviewView {
    pub tenant: TenantInfoView,
    pub business_status: BusinessStatusView,
    pub categories: Vec<MenuCategoryView>,
    pub menu_items: Vec<MenuItemView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantInfoView {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemView {
    pub menu_item_id: i64,
    pub item_name: String,
    pub quantity: i32,
    pub price: Decimal,
    pub flavor: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderView {
    pub id: i64,
    pub order_no: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub desk_code: String,
    pub status: String,
    pub total_amount: Decimal,
    pub remark: Option<String>,
    #[serde(serialize_with = "serialize_time")]
    pub created_time: NaiveDateTime,
    pub items: Vec<OrderItemView>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ImageView {
    pub filename: String,
    pub path: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardView {
    pub store_name: String,
    pub open: bool,
    pub notice: String,
    pub intro: String,
    pub today_order_count: i64,
    pub today_revenue: Decimal,
    pub available_menu_count: i64,
    pub latest_orders: Vec<OrderView>,
}

// ========== 超级管理员 ==========

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuperTenantView {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub enabled: bool,
    pub store_name: String,
    pub open: bool,
    pub notice: String,
    pub intro: String,
    #[serde(serialize_with = "serialize_time")]
    pub created_time: NaiveDateTime,
    pub order_count: i64,
    pub menu_item_count: i64,
    pub admin_count: i64,
    pub customer_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuperOrderView {
    pub tenant_id: i64,
    pub tenant_code: String,
    pub tenant_name: String,
    pub order: OrderView,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuperOverviewView {
    pub total_tenants: i64,
    pub enabled_tenants: i64,
    pub total_orders: i64,
    pub today_orders: i64,
    pub total_revenue: Decimal,
    pub today_revenue: Decimal,
    pub total_customers: i64,
    pub total_menu_items: i64,
    pub recent_orders: Vec<SuperOrderView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAccountView {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub tenant_id: Option<i64>,
    pub tenant_code: Option<String>,
    pub tenant_name: Option<String>,
    pub enabled: bool,
    #[serde(serialize_with = "serialize_time")]
    pub created_time: NaiveDateTime,
}
