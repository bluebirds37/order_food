use std::collections::HashMap;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use chrono::{Duration, Local};
use rand::Rng;
use sqlx::PgPool;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountType {
    Customer,
    TenantAdmin,
    SuperAdmin,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Customer => "CUSTOMER",
            Self::TenantAdmin => "TENANT_ADMIN",
            Self::SuperAdmin => "SUPER_ADMIN",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "CUSTOMER" => Some(Self::Customer),
            "TENANT_ADMIN" => Some(Self::TenantAdmin),
            "SUPER_ADMIN" => Some(Self::SuperAdmin),
            _ => None,
        }
    }
}

/// 当前登录用户，由 `Authorization: Bearer <token>` 解析得到。
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub token: String,
    pub account_type: AccountType,
    pub account_id: i64,
    pub tenant_id: Option<i64>,
    pub display_name: String,
    pub identity: String,
}

impl CurrentUser {
    pub fn is_customer(&self) -> bool {
        self.account_type == AccountType::Customer
    }

    pub fn is_tenant_admin(&self) -> bool {
        self.account_type == AccountType::TenantAdmin
    }

    pub fn is_super_admin(&self) -> bool {
        self.account_type == AccountType::SuperAdmin
    }

    pub fn require_admin(&self) -> Result<(), AppError> {
        if self.is_tenant_admin() || self.is_super_admin() {
            Ok(())
        } else {
            Err(AppError::forbidden("需要管理员权限"))
        }
    }

    pub fn require_super_admin(&self) -> Result<(), AppError> {
        if self.is_super_admin() {
            Ok(())
        } else {
            Err(AppError::forbidden("仅超级管理员可访问"))
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SessionRow {
    account_type: String,
    account_id: i64,
    tenant_id: Option<i64>,
    display_name: Option<String>,
    identity: Option<String>,
    enabled: Option<bool>,
    tenant_enabled: Option<bool>,
}

async fn load_user(state: &AppState, token: &str) -> Result<CurrentUser, AppError> {
    let row = sqlx::query_as::<_, SessionRow>(
        r#"
SELECT s.account_type,
       s.account_id,
       s.tenant_id,
       COALESCE(aa.display_name, ca.nickname) AS display_name,
       COALESCE(aa.username, ca.phone) AS identity,
       COALESCE(aa.enabled, ca.id IS NOT NULL) AS enabled,
       t.enabled AS tenant_enabled
FROM auth_session s
LEFT JOIN admin_account aa
       ON s.account_type IN ('TENANT_ADMIN', 'SUPER_ADMIN') AND aa.id = s.account_id
LEFT JOIN customer_account ca
       ON s.account_type = 'CUSTOMER' AND ca.id = s.account_id
LEFT JOIN tenant t ON t.id = s.tenant_id
WHERE s.token = $1 AND s.expires_time > NOW()
"#,
    )
    .bind(token)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::unauthorized("登录状态已失效，请重新登录"))?;

    if row.enabled == Some(false) {
        return Err(AppError::unauthorized("账号已停用"));
    }
    if row.tenant_id.is_some() && row.tenant_enabled == Some(false) {
        return Err(AppError::forbidden("租户已停用"));
    }

    let account_type =
        AccountType::parse(&row.account_type).ok_or_else(|| AppError::unauthorized("登录状态异常"))?;

    Ok(CurrentUser {
        token: token.to_string(),
        account_type,
        account_id: row.account_id,
        tenant_id: row.tenant_id,
        display_name: row.display_name.unwrap_or_default(),
        identity: row.identity.unwrap_or_default(),
    })
}

fn bearer_token(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts).ok_or_else(|| AppError::unauthorized("请先登录"))?;
        load_user(state, &token).await
    }
}

impl OptionalFromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        match bearer_token(parts) {
            Some(token) => load_user(state, &token).await.map(Some),
            None => Ok(None),
        }
    }
}

/// 租户选择器：租户管理员固定使用自己所属租户；
/// 超级管理员必须显式指定 `tenantId` 或 `tenantCode`。
pub async fn resolve_tenant(
    state: &AppState,
    user: &CurrentUser,
    tenant_id: Option<i64>,
    tenant_code: Option<&str>,
) -> Result<i64, AppError> {
    let target = match user.account_type {
        AccountType::Customer => {
            user.tenant_id
                .ok_or_else(|| AppError::unauthorized("登录状态缺少租户信息"))?
        }
        AccountType::TenantAdmin => {
            let own = user
                .tenant_id
                .ok_or_else(|| AppError::unauthorized("登录状态缺少租户信息"))?;
            if tenant_id.is_some_and(|id| id != own) {
                return Err(AppError::forbidden("无权操作其他租户的数据"));
            }
            if let Some(code) = tenant_code {
                let code_tenant = find_tenant_id_by_code(&state.pool, code).await?;
                if code_tenant != own {
                    return Err(AppError::forbidden("无权操作其他租户的数据"));
                }
            }
            own
        }
        AccountType::SuperAdmin => {
            if let Some(id) = tenant_id {
                id
            } else if let Some(code) = tenant_code {
                find_tenant_id_by_code(&state.pool, code).await?
            } else {
                return Err(AppError::business("请先指定要操作的租户"));
            }
        }
    };

    let enabled = sqlx::query_scalar::<_, bool>("SELECT enabled FROM tenant WHERE id = $1")
        .bind(target)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::business("租户不存在"))?;
    if !enabled {
        return Err(AppError::forbidden("租户已停用"));
    }
    Ok(target)
}

/// 公开接口的租户解析：必须携带租户标识（tenantCode 或 tenantId）。
pub async fn resolve_public_tenant(
    state: &AppState,
    user: Option<&CurrentUser>,
    tenant_id: Option<i64>,
    tenant_code: Option<&str>,
    require_enabled: bool,
) -> Result<i64, AppError> {
    if let Some(user) = user {
        return resolve_tenant(state, user, tenant_id, tenant_code).await;
    }

    let target = if let Some(id) = tenant_id {
        id
    } else if let Some(code) = tenant_code {
        find_tenant_id_by_code(&state.pool, code).await?
    } else {
        return Err(AppError::business("缺少租户标识，请扫描门店二维码进入"));
    };

    if require_enabled {
        let enabled = sqlx::query_scalar::<_, bool>("SELECT enabled FROM tenant WHERE id = $1")
            .bind(target)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| AppError::business("租户不存在"))?;
        if !enabled {
            return Err(AppError::business("该门店已暂停服务"));
        }
    }
    Ok(target)
}

#[derive(Debug, sqlx::FromRow)]
pub struct TenantRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub enabled: bool,
}

pub async fn find_tenant_by_code(pool: &PgPool, code: &str) -> Result<TenantRow, AppError> {
    let code = normalize_tenant_code(code)?;
    sqlx::query_as::<_, TenantRow>("SELECT id, code, name, enabled FROM tenant WHERE code = $1")
        .bind(code)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::business("门店不存在，请确认二维码是否正确"))
}

async fn find_tenant_id_by_code(pool: &PgPool, code: &str) -> Result<i64, AppError> {
    Ok(find_tenant_by_code(pool, code).await?.id)
}

pub async fn find_tenant_by_id(pool: &PgPool, id: i64) -> Result<TenantRow, AppError> {
    sqlx::query_as::<_, TenantRow>("SELECT id, code, name, enabled FROM tenant WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::business("租户不存在"))
}

pub fn normalize_tenant_code(code: &str) -> Result<String, AppError> {
    let code = code.trim().to_ascii_uppercase();
    if code.len() < 2 || code.len() > 32 {
        return Err(AppError::business("租户编码长度需在 2-32 之间"));
    }
    if !code
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::business(
            "租户编码仅支持字母、数字、短横线和下划线",
        ));
    }
    Ok(code)
}

pub fn tenant_selector(params: &HashMap<String, String>) -> (Option<i64>, Option<String>) {
    let tenant_id = params
        .get("tenantId")
        .and_then(|value| value.parse::<i64>().ok());
    let tenant_code = params
        .get("tenantCode")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    (tenant_id, tenant_code)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| {
            tracing::error!(error = %error, "密码哈希失败");
            AppError::internal()
        })
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn generate_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub async fn create_session(
    pool: &PgPool,
    account_type: AccountType,
    account_id: i64,
    tenant_id: Option<i64>,
    ttl_hours: i64,
) -> Result<String, AppError> {
    let token = generate_token();
    let expires_time = Local::now().naive_local() + Duration::hours(ttl_hours);
    sqlx::query(
        r#"
INSERT INTO auth_session (token, account_type, account_id, tenant_id, created_time, expires_time)
VALUES ($1, $2, $3, $4, NOW(), $5)
"#,
    )
    .bind(&token)
    .bind(account_type.as_str())
    .bind(account_id)
    .bind(tenant_id)
    .bind(expires_time)
    .execute(pool)
    .await?;
    Ok(token)
}

pub async fn remove_session(pool: &PgPool, token: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM auth_session WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

pub fn qr_content(code: &str) -> String {
    format!("orderfood://tenant/{code}")
}
