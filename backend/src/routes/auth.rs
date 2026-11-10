use axum::extract::State;
use axum::Json;

use crate::auth::{
    create_session, find_tenant_by_code, hash_password, remove_session, verify_password,
    AccountType, CurrentUser,
};
use crate::error::{ApiJson, AppError};
use crate::models::{
    BindPhoneRequest, CustomerAuthView, CustomerLoginRequest, CustomerRegisterRequest, CustomerRow,
    MeView, WxLoginRequest,
};
use crate::response::ApiResponse;
use crate::routes::required_text;
use crate::state::AppState;

pub async fn customer_register(
    State(state): State<AppState>,
    ApiJson(request): ApiJson<CustomerRegisterRequest>,
) -> Result<Json<ApiResponse<CustomerAuthView>>, AppError> {
    let tenant_code = required_text(request.tenant_code.clone(), "缺少门店标识，请扫码进入")?;
    let tenant = find_tenant_by_code(&state.pool, &tenant_code).await?;
    if !tenant.enabled {
        return Err(AppError::business("该门店已暂停服务"));
    }

    let phone = validate_phone(request.phone.clone())?;
    let password = validate_password(request.password.clone())?;
    let nickname = request
        .nickname
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("顾客{}", suffix(&phone)));

    let password_hash = hash_password(&password)?;

    let customer_id = sqlx::query_scalar::<_, i64>(
        r#"
INSERT INTO customer_account (tenant_id, phone, password_hash, nickname)
VALUES ($1, $2, $3, $4)
RETURNING id
"#,
    )
    .bind(tenant.id)
    .bind(&phone)
    .bind(password_hash)
    .bind(&nickname)
    .fetch_one(&state.pool)
    .await
    .map_err(|error| {
        if is_unique_violation(&error) {
            AppError::business("该手机号已注册")
        } else {
            AppError::from(error)
        }
    })?;

    let token = create_session(
        &state.pool,
        AccountType::Customer,
        customer_id,
        Some(tenant.id),
        state.token_ttl_hours,
    )
    .await?;

    Ok(ApiResponse::success_with_message(
        "注册成功",
        CustomerAuthView {
            token,
            customer_id,
            nickname,
            phone,
            tenant_id: tenant.id,
            tenant_code: tenant.code,
        },
    ))
}

pub async fn customer_login(
    State(state): State<AppState>,
    ApiJson(request): ApiJson<CustomerLoginRequest>,
) -> Result<Json<ApiResponse<CustomerAuthView>>, AppError> {
    let tenant_code = required_text(request.tenant_code.clone(), "缺少门店标识，请扫码进入")?;
    let tenant = find_tenant_by_code(&state.pool, &tenant_code).await?;
    if !tenant.enabled {
        return Err(AppError::business("该门店已暂停服务"));
    }

    let phone = validate_phone(request.phone.clone())?;
    let password = validate_password(request.password.clone())?;

    let customer = sqlx::query_as::<_, CustomerRow>(
        r#"
SELECT id, tenant_id, phone, password_hash, nickname
FROM customer_account
WHERE tenant_id = $1 AND phone = $2
"#,
    )
    .bind(tenant.id)
    .bind(&phone)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::business("手机号或密码错误"))?;

    if !verify_password(&customer.password_hash, &password) {
        return Err(AppError::business("手机号或密码错误"));
    }

    let token = create_session(
        &state.pool,
        AccountType::Customer,
        customer.id,
        Some(tenant.id),
        state.token_ttl_hours,
    )
    .await?;

    Ok(ApiResponse::success_with_message(
        "登录成功",
        CustomerAuthView {
            token,
            customer_id: customer.id,
            nickname: customer.nickname,
            phone: customer.phone,
            tenant_id: customer.tenant_id,
            tenant_code: tenant.code,
        },
    ))
}

pub async fn customer_wx_login(
    State(state): State<AppState>,
    ApiJson(request): ApiJson<WxLoginRequest>,
) -> Result<Json<ApiResponse<CustomerAuthView>>, AppError> {
    let tenant_code = required_text(request.tenant_code.clone(), "缺少门店标识，请扫码进入")?;
    let tenant = find_tenant_by_code(&state.pool, &tenant_code).await?;
    if !tenant.enabled {
        return Err(AppError::business("该门店已暂停服务"));
    }
    let code = required_text(request.code.clone(), "微信登录凭证不能为空")?;

    let openid = resolve_openid(&code).await?;
    let placeholder_phone = format!("wx_{}", openid);

    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM customer_account WHERE tenant_id = $1 AND openid = $2",
    )
    .bind(tenant.id)
    .bind(&openid)
    .fetch_optional(&state.pool)
    .await?;

    let customer_id = match existing {
        Some(id) => id,
        None => {
            let nickname = format!("微信顾客{}", suffix(&openid));
            let inserted = sqlx::query_scalar::<_, i64>(
                r#"
INSERT INTO customer_account (tenant_id, openid, phone, password_hash, nickname)
VALUES ($1, $2, $3, '', $4)
RETURNING id
"#,
            )
            .bind(tenant.id)
            .bind(&openid)
            .bind(&placeholder_phone)
            .bind(&nickname)
            .fetch_one(&state.pool)
            .await
            .map_err(|error| {
                if is_unique_violation(&error) {
                    AppError::business("微信账号登录冲突，请重试")
                } else {
                    AppError::from(error)
                }
            })?;
            inserted
        }
    };

    let phone = sqlx::query_scalar::<_, String>(
        "SELECT phone FROM customer_account WHERE id = $1",
    )
    .bind(customer_id)
    .fetch_one(&state.pool)
    .await?;
    let nickname = sqlx::query_scalar::<_, String>(
        "SELECT nickname FROM customer_account WHERE id = $1",
    )
    .bind(customer_id)
    .fetch_one(&state.pool)
    .await?;

    let token = create_session(
        &state.pool,
        AccountType::Customer,
        customer_id,
        Some(tenant.id),
        state.token_ttl_hours,
    )
    .await?;

    Ok(ApiResponse::success_with_message(
        "登录成功",
        CustomerAuthView {
            token,
            customer_id,
            nickname,
            phone,
            tenant_id: tenant.id,
            tenant_code: tenant.code,
        },
    ))
}

pub async fn bind_phone(
    auth: CurrentUser,
    State(state): State<AppState>,
    ApiJson(request): ApiJson<BindPhoneRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    if !auth.is_customer() {
        return Err(AppError::forbidden("请使用顾客账号绑定手机号"));
    }
    let tenant_id = auth
        .tenant_id
        .ok_or_else(|| AppError::unauthorized("登录状态缺少租户信息"))?;

    let mut phone: Option<String> = None;
    if let Some(phone_code) = request
        .phone_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        match fetch_wx_phone(phone_code).await {
            Ok(number) => phone = Some(number),
            Err(error) => {
                tracing::warn!("微信一键绑定手机号失败，尝试手动输入降级: {:?}", error);
            }
        }
    }
    let phone = match phone {
        Some(phone) => phone,
        None => {
            let manual = validate_phone(request.phone.clone())?;
            if manual.starts_with("wx_") || manual.len() != 11 || !manual.chars().all(|c| c.is_ascii_digit())
            {
                return Err(AppError::business("请输入 11 位手机号"));
            }
            manual
        }
    };

    let updated = sqlx::query(
        "UPDATE customer_account SET phone = $1 WHERE id = $2 AND tenant_id = $3",
    )
    .bind(&phone)
    .bind(auth.account_id)
    .bind(tenant_id)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        if is_unique_violation(&error) {
            AppError::business("该手机号已被其他账号使用")
        } else {
            AppError::from(error)
        }
    })?;
    if updated.rows_affected() == 0 {
        return Err(AppError::business("账号不存在"));
    }

    Ok(ApiResponse::success_with_message(
        "绑定成功",
        serde_json::json!({ "phone": phone }),
    ))
}

async fn resolve_openid(code: &str) -> Result<String, AppError> {
    let appid = std::env::var("WX_APPID").unwrap_or_default();
    let secret = std::env::var("WX_APP_SECRET").unwrap_or_default();
    if appid.is_empty() || secret.is_empty() {
        tracing::warn!("未配置 WX_APPID/WX_APP_SECRET，微信登录使用开发模式伪 openid（仅限本地调试）");
        return Ok(format!("dev_{:016x}", fnv1a(code.as_bytes())));
    }

    let url = format!(
        "https://api.weixin.qq.com/sns/jscode2session?appid={appid}&secret={secret}&js_code={code}&grant_type=authorization_code"
    );
    let response = reqwest::get(&url)
        .await
        .map_err(|_| AppError::business("微信登录服务调用失败，请稍后重试"))?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| AppError::business("微信登录响应解析失败"))?;

    if let Some(errcode) = response.get("errcode").and_then(|value| value.as_i64()) {
        if errcode != 0 {
            let errmsg = response
                .get("errmsg")
                .and_then(|value| value.as_str())
                .unwrap_or("未知错误");
            return Err(AppError::business(format!("微信登录失败（{}）", errmsg)));
        }
    }
    response
        .get("openid")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| AppError::business("微信登录响应缺少 openid"))
}

async fn fetch_wx_phone(phone_code: &str) -> Result<String, AppError> {
    let appid = std::env::var("WX_APPID").unwrap_or_default();
    let secret = std::env::var("WX_APP_SECRET").unwrap_or_default();
    if appid.is_empty() || secret.is_empty() {
        return Err(AppError::business("未配置微信密钥，无法一键获取手机号"));
    }

    let token_url = format!(
        "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={appid}&secret={secret}"
    );
    let token_response = reqwest::get(&token_url)
        .await
        .map_err(|_| AppError::business("微信凭证服务调用失败"))?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| AppError::business("微信凭证响应解析失败"))?;
    let access_token = token_response
        .get("access_token")
        .and_then(|value| value.as_str())
        .ok_or_else(|| AppError::business("获取微信接口凭证失败"))?;

    let phone_url = format!(
        "https://api.weixin.qq.com/wxa/business/getuserphonenumber?access_token={access_token}"
    );
    let phone_response = reqwest::Client::new()
        .post(&phone_url)
        .json(&serde_json::json!({ "code": phone_code }))
        .send()
        .await
        .map_err(|_| AppError::business("微信手机号服务调用失败"))?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| AppError::business("微信手机号响应解析失败"))?;

    if let Some(errcode) = phone_response.get("errcode").and_then(|value| value.as_i64()) {
        if errcode != 0 {
            let errmsg = phone_response
                .get("errmsg")
                .and_then(|value| value.as_str())
                .unwrap_or("未知错误");
            return Err(AppError::business(format!("微信手机号获取失败（{}）", errmsg)));
        }
    }
    phone_response
        .pointer("/phone_info/purePhoneNumber")
        .or_else(|| phone_response.pointer("/phone_info/phoneNumber"))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| AppError::business("微信手机号响应缺少手机号"))
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub async fn me(auth: CurrentUser) -> Result<Json<ApiResponse<MeView>>, AppError> {
    Ok(ApiResponse::success(MeView {
        account_type: auth.account_type.as_str().to_string(),
        account_id: auth.account_id,
        display_name: auth.display_name,
        identity: auth.identity,
        tenant_id: auth.tenant_id,
    }))
}

pub async fn logout(
    auth: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    remove_session(&state.pool, &auth.token).await?;
    Ok(ApiResponse::<()>::success_empty("已退出登录"))
}

fn validate_phone(value: Option<String>) -> Result<String, AppError> {
    let phone = required_text(value, "手机号不能为空")?;
    if phone.len() > 32 {
        return Err(AppError::business("手机号过长"));
    }
    Ok(phone)
}

fn validate_password(value: Option<String>) -> Result<String, AppError> {
    let password = required_text(value, "密码不能为空")?;
    if password.len() < 6 {
        return Err(AppError::business("密码至少 6 位"));
    }
    if password.len() > 128 {
        return Err(AppError::business("密码过长"));
    }
    Ok(password)
}

fn suffix(phone: &str) -> String {
    let chars: Vec<char> = phone.chars().collect();
    let start = chars.len().saturating_sub(4);
    chars[start..].iter().collect()
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(|database| database.code())
        .is_some_and(|code| code == "23505")
}
